import React from "react";
import type { QueueItem } from "../types";

export default function QueueBar(props: {
  nowPlaying: QueueItem | null;
  queue: QueueItem[];
  onPlayNext: () => void;
  onRemove: (idx: number) => void;
  onOpenKaraoke: () => void;
}) {
  const { nowPlaying, queue, onPlayNext, onRemove, onOpenKaraoke } = props;

  return (
    <section className="card">
      <div className="card-head">
        <h3>Reproducción</h3>
        <div style={{ display: "flex", gap: 10, flexWrap: "wrap" }}>
          <button className="btn" onClick={onPlayNext}>⏭ Siguiente</button>
          <button className="btn" onClick={onOpenKaraoke}>⛶ Karaoke</button>
        </div>
      </div>

      <div className="queue">
        <div className="now">
          <div className="muted">Now playing</div>
          <div className="now-title">{nowPlaying ? `${nowPlaying.artistName} — ${nowPlaying.title}` : "—"}</div>
        </div>

        <div className="queue-list">
          <div className="muted">Cola ({queue.length})</div>
          {queue.length === 0 && <div className="muted">Agrega canciones desde el modal del artista.</div>}
          {queue.map((q, idx) => (
            <div key={`${q.fullPath}-${idx}`} className="queue-item">
              <div className="queue-item-title">{q.artistName} — {q.title}</div>
              <button className="btn" onClick={() => onRemove(idx)}>Quitar</button>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
