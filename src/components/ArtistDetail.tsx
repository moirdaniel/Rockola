import { useEffect, useState } from 'react';
import type { Artist, Track, YouTubeResult } from '@shared/types';

interface Props {
  artist: Artist | null;
  tracks: Track[];
  onPlayLocal: (t: Track) => void;
  ytSearch: (term: string) => Promise<YouTubeResult[]>;
  onPlayYT: (videoId: string, title: string) => void;
}

export default function ArtistDetail({ artist, tracks, onPlayLocal, ytSearch, onPlayYT }: Props) {
  const [tab, setTab] = useState<'local' | 'youtube' | 'spotify'>('local');
  const [yt, setYt] = useState<YouTubeResult[]>([]);
  const [loadingYt, setLoadingYt] = useState(false);

  useEffect(() => {
    if (!artist) return;
    if (tab === 'youtube') {
      setLoadingYt(true);
      ytSearch(artist.name)
        .then(setYt)
        .finally(() => setLoadingYt(false));
    }
  }, [artist, tab, ytSearch]);

  useEffect(() => {
    setTab('local');
    setYt([]);
    setLoadingYt(false);
  }, [artist?.id]);

  if (!artist) return <div className="p-4 opacity-80">Selecciona un artista en la izquierda.</div>;

  return (
    <div className="flex flex-col gap-4 p-4">
      <div>
        <h2 className="text-2xl font-semibold">{artist.name}</h2>
      </div>
      <div className="flex gap-2">
        <button className={`btn ${tab === 'local' ? 'brightness-110' : ''}`} onClick={() => setTab('local')}>Local</button>
        <button className={`btn ${tab === 'youtube' ? 'brightness-110' : ''}`} onClick={() => setTab('youtube')}>YouTube</button>
        <button className={`btn ${tab === 'spotify' ? 'brightness-110' : ''}`} onClick={() => setTab('spotify')}>Spotify</button>
      </div>

      {tab === 'local' && (
        <div className="list">
          {tracks.map(t => (
            <div key={t.id} className="item">
              <div>
                <div className="font-semibold">{t.title}</div>
                <div className="badge">{t.type}</div>
              </div>
              <button className="btn" onClick={() => onPlayLocal(t)}>▶️ Reproducir</button>
            </div>
          ))}
          {tracks.length === 0 && <div className="opacity-80">Este artista no tiene pistas locales indexadas.</div>}
        </div>
      )}

      {tab === 'youtube' && (
        <div className="list">
          {loadingYt && <div className="opacity-80">Buscando en YouTube…</div>}
          {!loadingYt && yt.map(v => (
            <div key={v.videoId} className="item">
              <div>
                <div className="font-semibold">{v.title}</div>
                <div className="badge">{v.author}{v.durationSec ? ` • ${Math.round(v.durationSec / 60)}m` : ''}</div>
              </div>
              <button className="btn" onClick={() => onPlayYT(v.videoId, v.title)}>▶️</button>
            </div>
          ))}
          {!loadingYt && yt.length === 0 && <div className="opacity-80">Sin resultados en YouTube.</div>}
        </div>
      )}

      {tab === 'spotify' && (
        <div className="card p-4 text-sm text-muted">
          Vista de Spotify (metadatos) por implementar: requiere seleccionar artista vía búsqueda global.
        </div>
      )}
    </div>
  );
}
