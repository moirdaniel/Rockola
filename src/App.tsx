import { useEffect, useMemo, useState } from 'react';
import SearchBar from './components/SearchBar';
import ArtistList from './components/ArtistList';
import ArtistDetail from './components/ArtistDetail';
import Player from './components/Player';
import ConfigModal from './components/ConfigModal';
import type { Artist, Track, YouTubeResult } from '@shared/types';

interface NowPlaying {
  type: 'local' | 'youtube';
  src: string;
  title: string;
}

const MEDIA_DIRS_SETTING = 'mediaDirs';

export default function App() {
  const [mediaDirs, setMediaDirs] = useState<string[]>([]);
  const [q, setQ] = useState('');
  const [artists, setArtists] = useState<Artist[]>([]);
  const [selectedArtist, setSelectedArtist] = useState<Artist | null>(null);
  const [tracks, setTracks] = useState<Track[]>([]);
  const [nowPlaying, setNowPlaying] = useState<NowPlaying | null>(null);
  const [showConfig, setShowConfig] = useState(false);
  const [isScanning, setIsScanning] = useState(false);

  useEffect(() => {
    async function loadDirs() {
      const stored = await window.api.getSetting(MEDIA_DIRS_SETTING);
      const envDirs = (import.meta.env?.VITE_MEDIA_DIRS as string | undefined)?.split(';').filter(Boolean) ?? [];
      const dirs = stored ? stored.split(';').filter(Boolean) : envDirs;
      setMediaDirs(dirs);
    }
    loadDirs();
  }, []);

  useEffect(() => {
    async function initialArtists() {
      const res = await window.api.findArtists('');
      setArtists(res);
    }
    initialArtists();
  }, []);

  const canScan = useMemo(() => mediaDirs.length > 0, [mediaDirs]);

  async function handleScan() {
    if (!canScan) {
      alert('Configura rutas de medios en la pantalla de configuración.');
      return;
    }
    setIsScanning(true);
    try {
      await window.api.scanLibrary(mediaDirs);
      const res = await window.api.findArtists(q);
      setArtists(res);
      if (selectedArtist) {
        const refreshed = res.find(a => a.name === selectedArtist.name);
        if (!refreshed) {
          setSelectedArtist(null);
          setTracks([]);
        } else {
          await openArtist(refreshed);
        }
      }
    } catch (err) {
      console.error(err);
      alert('Ocurrió un error durante la indexación. Revisa la consola para más detalles.');
    } finally {
      setIsScanning(false);
    }
  }

  async function handleSearchArtists(text: string) {
    setQ(text);
    const res = await window.api.findArtists(text);
    setArtists(res);
  }

  async function openArtist(a: Artist) {
    setSelectedArtist(a);
    const t = await window.api.getArtistTracks(a.name);
    setTracks(t);
  }

  async function playLocal(track: Track) {
    setNowPlaying({ type: 'local', src: `file://${track.pathOrId}`, title: `${track.artistName} - ${track.title}` });
  }

  async function searchYT(term: string) {
    const results: YouTubeResult[] = await window.api.ytSearch(term);
    return results;
  }

  async function playYT(videoId: string, title: string) {
    const { url } = await window.api.ytStreamUrl(videoId);
    setNowPlaying({ type: 'youtube', src: url, title });
  }

  function handleSaveDirs(value: string) {
    const dirs = value.split(';').map(d => d.trim()).filter(Boolean);
    setMediaDirs(dirs);
    void window.api.setSetting(MEDIA_DIRS_SETTING, dirs.join(';'));
  }

  return (
    <>
      <header className="header">
        <button className="btn" onClick={handleScan} disabled={isScanning}>
          {isScanning ? '⏳ Reindexando…' : '📚 Reindexar'}
        </button>
        <SearchBar placeholder="Buscar artista local…" value={q} onChange={handleSearchArtists} />
        <button className="btn" onClick={() => setShowConfig(true)}>⚙️ Configurar</button>
      </header>
      <main className="maingrid">
        <aside className="sidebar">
          <ArtistList artists={artists} onPick={openArtist} />
        </aside>
        <section className="overflow-auto">
          <ArtistDetail
            artist={selectedArtist}
            tracks={tracks}
            onPlayLocal={playLocal}
            ytSearch={searchYT}
            onPlayYT={playYT}
          />
        </section>
      </main>
      <Player nowPlaying={nowPlaying} />
      {showConfig && (
        <ConfigModal
          dirs={mediaDirs}
          onClose={() => setShowConfig(false)}
          onSave={handleSaveDirs}
        />
      )}
    </>
  );
}
