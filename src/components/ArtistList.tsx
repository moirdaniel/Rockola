import type { Artist } from '@shared/types';

export default function ArtistList({ artists, onPick }: { artists: Artist[]; onPick: (a: Artist) => void }) {
  return (
    <div className="list">
      {artists.map(a => (
        <div key={a.id} className="item cursor-pointer" onClick={() => onPick(a)}>
          <div>
            <div className="font-semibold">{a.name}</div>
            <div className="badge">Artista</div>
          </div>
          <span>›</span>
        </div>
      ))}
      {artists.length === 0 && (
        <div className="p-3 opacity-80">No hay artistas. Presiona "Reindexar" para construir la librería.</div>
      )}
    </div>
  );
}
