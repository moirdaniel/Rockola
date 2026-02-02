import React from "react";
import type { QueueItem } from "./types";
import { getMediaPort } from "../lib/tauri";

/**
 * Detecta si estamos corriendo dentro de Tauri (WebView).
 * Esto evita imports que rompen el build web.
 */
function isTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    (("__TAURI_INTERNALS__" in window) ||
      ("__TAURI__" in window) ||
      (window as any).__TAURI_INTERNALS__?.invoke)
  );
}

/**
 * Convierte una ruta local (/storage/...) a un src válido para el WebView de Tauri:
 * asset://localhost/....
 *
 * Si NO estamos en Tauri, devuelve string vacío para evitar intentar cargar rutas FS crudas.
 */
async function safeConvertFileSrc(path: string): Promise<string> {
  if (!path) return "";

  if (!isTauriRuntime()) {
    console.warn("🌐 No es Tauri runtime. No se puede reproducir path local directo:", path);
    return "";
  }

  try {
    // Tauri v2: convertFileSrc vive en @tauri-apps/api/core
    const mod: any = await import("@tauri-apps/api/core");
    if (typeof mod.convertFileSrc === "function") {
      const src = mod.convertFileSrc(path);
      return src;
    }
    console.error("❌ convertFileSrc no está disponible en @tauri-apps/api/core");
    return "";
  } catch (e) {
    console.error("❌ Error importando @tauri-apps/api/core:", e);
    return "";
  }
}

type Props = {
  nowPlaying: QueueItem | null;

  // pending
  pending: QueueItem | null;
  pendingSecondsLeft: number;
  onCancelPending: () => void;
  onPlayNow: () => void;

  // fullscreen
  isFullscreen: boolean;
  onRequestFullscreen: () => void;
  onExitFullscreen: () => void;

  // debug / status
  statusText?: string;
};

export default function PlayerPanel(props: Props) {
  const {
    nowPlaying,
    pending,
    pendingSecondsLeft,
    onCancelPending,
    onPlayNow,
    isFullscreen,
    onRequestFullscreen,
    onExitFullscreen,
    statusText,
  } = props;

  const containerRef = React.useRef<HTMLDivElement | null>(null);
  const mediaRef = React.useRef<HTMLVideoElement | HTMLAudioElement | null>(null);

  const [src, setSrc] = React.useState<string>("");
  const [srcLoading, setSrcLoading] = React.useState(false);
  const [mime, setMime] = React.useState<string>("");

  // Cargar src cuando cambia nowPlaying
  React.useEffect(() => {
    let alive = true;

    async function load() {

     const port = await getMediaPort();

      if (!nowPlaying) {
        setSrc("");
        return;
      }

      setSrcLoading(true);

      console.log("🎬 fullPath:", nowPlaying.fullPath);
      const s = await safeConvertFileSrc(nowPlaying.fullPath);
    
      const encoded = encodeURIComponent(nowPlaying.fullPath);
      const httpSrc = `http://127.0.0.1:${port}/media?path=${encoded}`;
        setSrc(httpSrc);
        setMime(guessMime(nowPlaying.fullPath, nowPlaying.mediaType));

      if (!alive) return;

      console.log("🎬 resolved src:", s);


    //   setSrc(s);
    //   setMime(guessMime(nowPlaying.fullPath, nowPlaying.mediaType));
      setSrcLoading(false);

        console.log("🧪 isTauriRuntime:", isTauriRuntime());
        console.log("🎬 fullPath:", nowPlaying.fullPath);
        console.log("🎬 resolved src:", s);

    }

    void load();
    return () => {
      alive = false;
    };
  }, [nowPlaying?.fullPath]);

  // Cuando cambia src, intentar play()
  React.useEffect(() => {
  const el = mediaRef.current;
  if (!el || !src) return;

  try {
    // cuando usas <source>, necesitas load()
    (el as any).load?.();

    const p = (el as any).play?.();
    if (p && typeof p.then === "function") {
      p.catch((err: any) => {
        console.warn("⚠️ play() falló:", err);
      });
    }
  } catch (e) {
    console.warn("⚠️ load/play exception:", e);
  }
}, [src, mime]);


  // Fullscreen helpers
  function requestFs() {
    const node = containerRef.current;
    if (!node) return;
    const fn = (node as any).requestFullscreen || (node as any).webkitRequestFullscreen;
    if (fn) fn.call(node);
  }

  function exitFs() {
  const doc: any = document;
  const isFs = !!(doc.fullscreenElement || doc.webkitFullscreenElement);
  
  if (!isFs)
     return; 

  const fn = doc.exitFullscreen || doc.webkitExitFullscreen;
  if (fn) fn.call(document);
}

function getExt(path: string): string {
  const m = path.toLowerCase().match(/\.([a-z0-9]+)$/);
  return m?.[1] ?? "";
}

function guessMime(path: string, mediaType: "video" | "audio"): string {
  const ext = getExt(path);

  // video
  if (mediaType === "video") {
    if (ext === "mp4" || ext === "m4v") return "video/mp4";
    if (ext === "webm") return "video/webm";
    if (ext === "mkv") return "video/x-matroska"; // a veces no lo soporta WebKit igual
    return "video/mp4"; // fallback razonable
  }

  // audio
  if (ext === "mp3") return "audio/mpeg";
  if (ext === "m4a" || ext === "mp4") return "audio/mp4";
  if (ext === "ogg") return "audio/ogg";
  if (ext === "wav") return "audio/wav";
  return "audio/mpeg";
}


  // Sync con estado externo
  React.useEffect(() => {
    try {
        if (isFullscreen) requestFs();
        else exitFs();
    } catch (e) {
        console.warn("fullscreen sync error:", e);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isFullscreen]);

  const showVideo = nowPlaying?.mediaType === "video";
  const showAudio = nowPlaying?.mediaType === "audio";

  return (
    <div className="card player-card" ref={containerRef}>
      <div className="player-head">
        <div>
          <div className="card-title">Reproducción</div>
          <div className="muted">Now playing</div>
        </div>

        <div className="player-actions">
          <button
            className="btn"
            onClick={() => (isFullscreen ? onExitFullscreen() : onRequestFullscreen())}
          >
            {isFullscreen ? "Salir Fullscreen" : "Fullscreen"}
          </button>
        </div>
      </div>

      <div className="player-body">
        {pending && (
          <div className="pending-banner">
            <div className="pending-title">
              Iniciando en <b>{pendingSecondsLeft}s</b> — {pending.artistName} — {pending.title}
            </div>
            <div className="pending-actions">
              <button className="btn btn-primary" onClick={onPlayNow}>
                Reproducir ahora
              </button>
              <button className="btn" onClick={onCancelPending}>
                Cancelar
              </button>
            </div>
          </div>
        )}

        {!nowPlaying && !pending && (
          <div className="muted" style={{ marginTop: 8 }}>
            Selecciona un item de la cola para reproducir.
          </div>
        )}

        {srcLoading && <div className="muted" style={{ marginTop: 8 }}>Cargando media…</div>}

        {!!nowPlaying && (
          <>
            {!src && (
              <div className="muted" style={{ marginTop: 8 }}>
                No se pudo resolver el src. (Revisa assetProtocol/scope en tauri.conf.json)
              </div>
            )}

            {!!src && (
              <div className="media-wrap">
                {showVideo && (
                  <video
                        ref={(r) => {
                            mediaRef.current = r;
                        }}
                        className="media"
                        controls
                        playsInline
                        preload="auto"
                        onError={() => {
                            const el = mediaRef.current as HTMLVideoElement | null;
                            console.error("🎥 <video> error:", el?.error);
                        }}
                        onCanPlay={() => console.log("🎥 canplay")}
                        >
                        <source src={src} type={mime || "video/mp4"} />
                    </video>

                )}

                {showAudio && (
                 <audio
                    ref={(r) => {
                        mediaRef.current = r;
                    }}
                    className="media-audio"
                    controls
                    preload="auto"
                    onError={() => {
                        const el = mediaRef.current as HTMLAudioElement | null;
                        console.error("🎧 <audio> error:", el?.error);
                    }}
                    onCanPlay={() => console.log("🎧 canplay")}
                    >
                    <source src={src} type={mime || "audio/mpeg"} />
                  </audio>

                )}

                <div className="nowplaying-meta">
                  <div className="np-title">
                    {nowPlaying.artistName} — {nowPlaying.title}
                  </div>
                  <div className="muted">
                    {nowPlaying.mediaType} • {nowPlaying.fullPath}
                  </div>
                </div>
              </div>
            )}
          </>
        )}

        {statusText && <div className="muted" style={{ marginTop: 10 }}>{statusText}</div>}
      </div>
    </div>
  );
}
