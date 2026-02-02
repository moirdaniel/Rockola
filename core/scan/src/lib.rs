//! Scanner recursivo:
//! - sources (root_path) => recorre archivos
//! - artista = carpeta padre
//! - upsert a SQLite
//! - retorna ids upsertados + lista de seen full paths para marcar missing

use core_db::Db;
use core_domain::normalize_artist_key;
use core_events::{AppEvent, ScanProgress, LibraryDelta};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("db error: {0}")]
    Db(#[from] core_db::DbError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("walk error: {0}")]
    Walk(String),
    #[error("cancelled")]
    Cancelled,
}

pub type ScanResult<T> = Result<T, ScanError>;

pub trait EventSink: Send + Sync {
    fn emit(&self, event: AppEvent);
}

#[derive(Clone)]
pub struct Scanner {
    db: Db,
}

impl Scanner {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn scan_source(&self, source_id: i64, root_path: &str, sink: &dyn EventSink) -> ScanResult<()> {
        self.scan_source_with_cancel(source_id, root_path, sink, &AtomicBool::new(false))
    }

    pub fn scan_source_with_cancel(
        &self,
        source_id: i64,
        root_path: &str,
        sink: &dyn EventSink,
        cancelled: &AtomicBool,
    ) -> ScanResult<()> {
        let mut conn = self.db.connect()?;

        let root = PathBuf::from(root_path);
        if !root.exists() {
            // source offline
            sink.emit(AppEvent::Error(core_events::AppError {
                code: "SOURCE_NOT_FOUND".into(),
                message: format!("Source no existe: {root_path}"),
                context: None,
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
                severity: "error".into(),
            }));
            return Ok(());
        }

        let mut processed: u64 = 0;
        let mut seen_full_paths: Vec<String> = Vec::new();
        let mut upserted_ids: Vec<i64> = Vec::new();

        sink.emit(AppEvent::ScanProgress(ScanProgress {
            source_id,
            processed,
            total: None,
            phase: "walking".into(),
            current_path: Some(root_path.into()),
            progress_percent: None,
        }));

        // Obtener el número total de archivos para mostrar progreso
        let total_files = self.count_candidate_files(&root);
        sink.emit(AppEvent::ScanProgress(ScanProgress {
            source_id,
            processed: 0,
            total: Some(total_files),
            phase: "counting".into(),
            current_path: Some("Contando archivos...".into()),
            progress_percent: Some(0.0),
        }));

        for entry in WalkDir::new(&root).follow_links(true).into_iter() {
            // Verificar si se ha solicitado cancelación
            if cancelled.load(Ordering::Relaxed) {
                return Err(ScanError::Cancelled);
            }

            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    // No abortamos todo por un archivo raro
                    sink.emit(AppEvent::Toast(core_events::Toast {
                        level: "warn".into(),
                        message: format!("Walk warning: {e}"),
                        duration: None,
                        action: None,
                    }));
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();

            // Filtrado MVP (amplio). Puedes expandir sin miedo.
            if !is_candidate_media(path) {
                continue;
            }

            let full_path = path.to_string_lossy().to_string();
            let rel_path = path.strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let artist_folder = path.parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown Artist".into());

            let artist_key = normalize_artist_key(&artist_folder).0;

            let title = path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".into());

            let media_type = guess_media_type(path);
            let meta = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size_bytes = meta.len() as i64;
            let mtime_unix = meta.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // upsert artist + media_item
            let artist_id = core_db::Db::upsert_artist(&conn, &artist_folder, &artist_key)?;
            let item_id = core_db::Db::upsert_media_item(
                &conn,
                source_id,
                artist_id,
                &full_path,
                &rel_path,
                &title,
                media_type,
                None,            // duration_ms: luego con probe
                size_bytes,
                mtime_unix,
                "unknown",       // playable: luego con probe
            )?;

            processed += 1;
            seen_full_paths.push(full_path);
            upserted_ids.push(item_id);

            if processed % 50 == 0 {
                sink.emit(AppEvent::ScanProgress(ScanProgress {
                    source_id,
                    processed,
                    total: Some(total_files),
                    phase: "walking".into(),
                    current_path: Some(rel_path.clone()),
                    progress_percent: Some((processed as f64 / total_files as f64) * 100.0),
                }));
                
                // Emitir delta solo con IDs relevantes
                if !upserted_ids.is_empty() {
                    sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                        reason: "upsert".into(),
                        item_ids: upserted_ids.clone(),
                        affected_artists: vec![],
                        timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
                    }));
                    upserted_ids.clear();
                }
            }
        }

        // flush deltas
        if !upserted_ids.is_empty() {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                reason: "upsert".into(),
                item_ids: upserted_ids,
                affected_artists: vec![],
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));
        }

        // marcar missing lo que ya no está
        let missing_changed = core_db::Db::mark_missing_for_source(&mut conn, source_id, &seen_full_paths)?;
        
        if missing_changed > 0 {
            sink.emit(AppEvent::LibraryDelta(LibraryDelta {
                reason: "missing".into(),
                item_ids: vec![],
                affected_artists: vec![],
                timestamp: time::OffsetDateTime::now_utc().unix_timestamp(),
            }));
        }

        core_db::Db::mark_source_scan_done(&conn, source_id)?;

        sink.emit(AppEvent::ScanProgress(ScanProgress {
            source_id,
            processed,
            total: Some(total_files),
            phase: "done".into(),
            current_path: None,
            progress_percent: Some(100.0),
        }));

        Ok(())
    }

    // Método auxiliar para contar archivos candidatos
    fn count_candidate_files(&self, root: &Path) -> u64 {
        let mut count = 0u64;
        for entry in WalkDir::new(root).follow_links(true).into_iter().flatten() {
            if entry.file_type().is_file() && is_candidate_media(entry.path()) {
                count += 1;
            }
        }
        count
    }
}

// MVP: set amplio por extensión.
// Puedes ampliar en caliente sin romper DB.
fn is_candidate_media(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(
        ext.as_str(),
        "mp3"|"flac"|"wav"|"m4a"|"aac"|"ogg"|"opus"|"wma"|"aiff"|
        "mp4"|"mkv"|"webm"|"mov"|"avi"|"m4v"|"mpg"|"mpeg"|"ts"| 
        "wmv"|"flv"|"f4v"|"webm"|"m2ts"|"mts"|"3gp"|"3g2"|"mxf"|
        "alac"|"ape"|"dsf"|"dff"|"wv"|"tta"|"m4b"|"m4r"|"mp2"|"mp1"|
        "oga"|"spx"|"opus"|"weba"|"mka"|"mks"|"iso"|"img"|"m2p"|"m2v"|
        "ogv"|"ogm"|"mov"|"qt"|"rm"|"rmvb"|"asf"|"asx"|"wm"|"wmx"|
        "avi"|"divx"|"mpg"|"mpeg"|"mpe"|"mpv"|"m2v"|"svi"|"3gpp"|
        "3gpp2"|"m4v"|"h3d"|"3dv"|"3dm"|"3ds"|"3dt"|"3dv"|"3dx"|
        "flac"|"opus"|"ra"|"ram"|"snd"|"thd"|"tta"|"voc"|"vqf"|"w64"|
        "wma"|"wv"|"webm"|"wmv"|"xvid"|"yuv"|"264"|"265"|"3g2"|"3gp"|
        "3gpp"|"3iv"|"aaf"|"aec"|"aep"|"aepx"|"aet"|"aetx"|"ajp"|"ale"|
        "all"|"am"|"amr"|"aob"|"ape"|"arf"|"asf"|"asm"|"ass"|"ast"|
        "asd"|"asx"|"avb"|"avd"|"avi"|"avp"|"avs"|"bdm"|"bdmv"|"bdt2"|
        "bdt3"|"bin"|"bix"|"bmk"|"box"|"bs4"|"bsf"|"bu"|"bvr"|"byu"|
        "camproj"|"camrec"|"camv"|"ced"|"cel"|"cine"|"cip"|"clpi"|"cmmp"|
        "cmmtpl"|"cmp"|"cmrec"|"cmsd"|"cmsdp"|"cmv"|"cmx"|"cpi"|"cpvc"|
        "crec"|"cst"|"csv"|"cue"|"cv2"|"cx3"|"d2v"|"d3v"|"dat"|"dav"|
        "dce"|"dck"|"dcr"|"dct"|"ddat"|"dif"|"dir"|"divx"|"dl"|"dm2"|
        "dmb"|"dmsd"|"dmsdp"|"dmsf"|"dmv"|"dmx"|"dnc"|"dpa"|"dpg"|"dsy"|
        "dv"|"dv-avi"|"dv4"|"dvdmedia"|"dvr"|"dvr-ms"|"dvx"|"dxr"|"dzm"|
        "dzp"|"dzt"|"edl"|"evo"|"eye"|"ezt"|"f4a"|"f4b"|"f4p"|"f4v"|"f64"|
        "flc"|"flh"|"fli"|"flv"|"flx"|"fpf"|"ftc"|"g2m"|"g64"|"gcs"|
        "gfp"|"gifv"|"gl"|"gom"|"grasp"|"gts"|"gvi"|"gvp"|"h264"|"hdmov"|
        "hkm"|"ifo"|"imovieproj"|"imovieproject"|"ircp"|"irf"|"ism"|"ismc"|
        "ismclip"|"isms"|"iva"|"ivf"|"ivr"|"ivs"|"izz"|"izzy"|"jss"|"jts"|
        "jtv"|"k3g"|"kdenlive"|"kit"|"kmy"|"kon"|"kpr"|"kra"|"ksh"|"ksm"|
        "kt"|"ktn"|"lrec"|"lrv"|"lsf"|"lsx"|"lvix"|"m15"|"m1pg"|"m1v"|"m21"|
        "m21"|"m2a"|"m2p"|"m2t"|"m2ts"|"m2v"|"m4e"|"m4u"|"m4v"|"m75"|"meta"|
        "mgv"|"mj2"|"mjp"|"mjpeg"|"mjpg"|"mjpg"|"mk3d"|"mks"|"mkv"|"mmv"|"mnv"|
        "mob"|"mod"|"modd"|"moff"|"moi"|"moov"|"mov"|"movie"|"mp21"|"mp2v"|"mp4"|
        "mp4v"|"mpe"|"mpeg"|"mpeg1"|"mpeg2"|"mpeg4"|"mpf"|"mpg"|"mpg2"|"mpg4"|"mpl"|
        "mpls"|"mpos"|"mpv"|"mpv2"|"mqv"|"msdvd"|"mse"|"msh"|"msv"|"mt2s"|"mts"|"mtv"|"mv"|
        "mvb"|"mvc"|"mvd"|"mve"|"mvex"|"mvp"|"mvy"|"mxf"|"mxv"|"mys"|"ncor"|"nsv"|"nut"|"nuv"|"nvc"|"ogm"|"ogv"|"ogx"|"osp"|"otrkey"|"pac"|"par"|"pds"|"pgi"|"photoshow"|"piv"|"pjs"|"playlist"|"plproj"|"pmf"|"prel"|"pro"|"pro4"|"pro5"|"pro7"|"prproj"|"prtl"|"psb"|"psd"|"psh"|"pssd"|"pva"|"pvr"|"pxv"|"qt"|"qtch"|"qtl"|"qtm"|"qtz"|"rcd"|"rcproject"|"rdb"|"rec"|"rm"|"rmd"|"rmp"|"rms"|"rmv"|"rmvb"|"roq"|"rp"|"rsx"|"rts"|"rum"|"rv"|"sbk"|"sbt"|"scc"|"scm"|"scn"|"screenflow"|"sdi"|"sdp"|"sdr"|"sds"|"sdt"|"sedprj"|"seq"|"sfd"|"sfvidcap"|"siv"|"smi"|"smil"|"smk"|"sml"|"sms"|"smv"|"spl"|"sqz"|"srt"|"ssf"|"ssm"|"stl"|"str"|"stx"|"svi"|"swf"|"swi"|"swt"|"tda3mt"|"tdt"|"theora"|"thm"|"tid"|"tix"|"tod"|"tp"|"tp0"|"tpd"|"tpr"|"trp"|"ts"|"tsp"|"ttxt"|"tvlayer"|"tvrecording"|"tvs"|"tvshow"|"usf"|"usm"|"v264"|"vbc"|"vc1"|"vcpf"|"vcr"|"vcv"|"vdo"|"vdr"|"vdx"|"veg"|"vem"|"vf"|"vft"|"vfwp"|"vga"|"vgz"|"vid"|"video"|"viewlet"|"viv"|"vivo"|"vlab"|"vob"|"vp3"|"vp6"|"vp7"|"vpj"|"vro"|"vs4"|"vse"|"vsp"|"w32"|"wcp"|"webm"|"wlmp"|"wm"|"wmd"|"wmmp"|"wmv"|"wmx"|"wot"|"wp3"|"wpl"|"wtv"|"wvx"|"xej"|"xel"|"xesc"|"xfl"|"xlmv"|"xml"|"xmv"|"xvid"|"y4m"|"yog"|"yuv"|"zeg"|"zm1"|"zm2"|"zm3"|"zmv"
    )
}

fn guess_media_type(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "mp3"|"flac"|"wav"|"m4a"|"aac"|"ogg"|"opus"|"wma"|"aiff"|"alac"|"ape"|"dsf"|"dff"|"wv"|"tta"|"m4b"|"m4r"|"mp2"|"mp1"|"oga"|"spx"|"weba"|"mka"|"ra"|"ram"|"snd"|"voc"|"vqf"|"w64" => "audio",
        "mp4"|"mkv"|"webm"|"mov"|"avi"|"m4v"|"mpg"|"mpeg"|"ts"|"wmv"|"flv"|"f4v"|"m2ts"|"mts"|"3gp"|"3g2"|"mxf"|"ogv"|"ogm"|"qt"|"rm"|"rmvb"|"asf"|"m2p"|"m2v"|"svi"|"3gpp"|"3iv"|"264"|"265"|"3iv"|"aaf"|"amv"|"asx"|"bik"|"bix"|"box"|"camproj"|"camrec"|"camv"|"cdxl"|"cgi"|"cmmp"|"cmmtpl"|"cmv"|"cpi"|"d2v"|"d3v"|"dat"|"dav"|"dif"|"divx"|"dmf"|"dmv"|"dsm"|"dsv"|"dts"|"dv"|"dv4"|"dvdmedia"|"dvr"|"dvr-ms"|"dvx"|"dxr"|"evo"|"f4a"|"f4b"|"f4p"|"fbr"|"fbr!"|"fbz"|"fcp"|"fcproject"|"ffd"|"flc"|"flh"|"fli"|"flv"|"flx"|"g2m"|"g64"|"gifv"|"gl"|"gom"|"grasp"|"gts"|"gvi"|"gvp"|"h264"|"hdmov"|"hkm"|"ifo"|"imovieproj"|"imovieproject"|"ircp"|"irf"|"ism"|"ismc"|"ismclip"|"isms"|"iva"|"ivf"|"ivr"|"ivs"|"izz"|"izzy"|"jss"|"jts"|"jtv"|"k3g"|"kdenlive"|"kit"|"kmy"|"kon"|"kpr"|"kra"|"ksh"|"ksm"|"kt"|"ktn"|"lrec"|"lrv"|"lsf"|"lsx"|"lvix"|"m15"|"m1pg"|"m1v"|"m21"|"m2a"|"m2p"|"m2t"|"m2ts"|"m2v"|"m4e"|"m4u"|"m4v"|"m75"|"meta"|"mgv"|"mj2"|"mjp"|"mjpeg"|"mjpg"|"mk3d"|"mks"|"mkv"|"mmv"|"mnv"|"mob"|"mod"|"modd"|"moff"|"moi"|"moov"|"mov"|"movie"|"mp21"|"mp2v"|"mp4"|"mp4v"|"mpe"|"mpeg"|"mpeg1"|"mpeg2"|"mpeg4"|"mpf"|"mpg"|"mpg2"|"mpg4"|"mpl"|"mpls"|"mpos"|"mpv"|"mpv2"|"mqv"|"msdvd"|"mse"|"msh"|"msv"|"mt2s"|"mts"|"mtv"|"mv"|"mvb"|"mvc"|"mvd"|"mve"|"mvex"|"mvp"|"mvy"|"mxf"|"mxv"|"mys"|"ncor"|"nsv"|"nut"|"nuv"|"nvc"|"ogm"|"ogv"|"ogx"|"osp"|"otrkey"|"pac"|"par"|"pds"|"pgi"|"photoshow"|"piv"|"pjs"|"playlist"|"plproj"|"pmf"|"prel"|"pro"|"pro4"|"pro5"|"pro7"|"prproj"|"prtl"|"psb"|"psd"|"psh"|"pssd"|"pva"|"pvr"|"pxv"|"qt"|"qtch"|"qtl"|"qtm"|"qtz"|"rcd"|"rcproject"|"rdb"|"rec"|"rm"|"rmd"|"rmp"|"rms"|"rmv"|"rmvb"|"roq"|"rp"|"rsx"|"rts"|"rum"|"rv"|"sbk"|"sbt"|"scc"|"scm"|"scn"|"screenflow"|"sdi"|"sdp"|"sdr"|"sds"|"sdt"|"sedprj"|"seq"|"sfd"|"sfvidcap"|"siv"|"smi"|"smil"|"smk"|"sml"|"sms"|"smv"|"spl"|"sqz"|"srt"|"ssf"|"ssm"|"stl"|"str"|"stx"|"svi"|"swf"|"swi"|"swt"|"tda3mt"|"tdt"|"theora"|"thm"|"tid"|"tix"|"tod"|"tp"|"tp0"|"tpd"|"tpr"|"trp"|"ts"|"tsp"|"ttxt"|"tvlayer"|"tvrecording"|"tvs"|"tvshow"|"usf"|"usm"|"v264"|"vbc"|"vc1"|"vcpf"|"vcr"|"vcv"|"vdo"|"vdr"|"vdx"|"veg"|"vem"|"vf"|"vft"|"vfwp"|"vga"|"vgz"|"vid"|"video"|"viewlet"|"viv"|"vivo"|"vlab"|"vob"|"vp3"|"vp6"|"vp7"|"vpj"|"vro"|"vs4"|"vse"|"vsp"|"w32"|"wcp"|"webm"|"wlmp"|"wm"|"wmd"|"wmmp"|"wmv"|"wmx"|"wot"|"wp3"|"wpl"|"wtv"|"wvx"|"xej"|"xel"|"xesc"|"xfl"|"xlmv"|"xml"|"xmv"|"xvid"|"y4m"|"yog"|"yuv"|"zeg"|"zm1"|"zm2"|"zm3"|"zmv" => "video",
        _ => "unknown",
    }
}