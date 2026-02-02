import React from "react";
import type { Artist, ItemRow } from "../types";

type Props = {
  open: boolean;
  artist: Artist | null;
  loading: boolean;
  items: ItemRow[];
  onClose: () => void;
  onAddToQueue: (it: ItemRow) => void;
};

export default function ArtistModal({ open, artist, loading, items, onClose, onAddToQueue }: Props) {
  if (!open) return null;

  // Agrupar por tipo de medio
  const groupedItems = items.reduce((acc, item) => {
    if (!acc[item.mediaType]) {
      acc[item.mediaType] = [];
    }
    acc[item.mediaType].push(item);
    return acc;
  }, {} as Record<string, ItemRow[]>);

  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal modal-clean" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div>
            <div className="modal-title">
              🎤 {artist?.displayName ?? "Artista"}
            </div>
            <div className="muted">
              {items.length} items disponibles • Agrega canciones/videos a la cola
            </div>
          </div>
          <button className="btn" onClick={onClose} style={{ padding: '0.5rem 1rem' }}>
            ✕ Cerrar
          </button>
        </div>

        <div className="modal-body panel-scroll" style={{ maxHeight: "70vh" }}>
          {loading ? (
            <div className="muted" style={{ textAlign: 'center', padding: '2rem 0' }}>
              🔄 Cargando items...
            </div>
          ) : items.length === 0 ? (
            <div className="muted" style={{ textAlign: 'center', padding: '2rem 0' }}>
              😔 No hay items disponibles (o ya están en cola).
            </div>
          ) : (
            <div className="items">
              {Object.entries(groupedItems).map(([mediaType, itemsOfType]) => (
                <div key={mediaType} style={{ marginBottom: '1.5rem' }}>
                  <div className="card-title" style={{ marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                    {mediaType === 'video' ? '🎥' : '🎵'} {mediaType === 'video' ? 'Videos' : 'Audios'} ({itemsOfType.length})
                  </div>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                    {itemsOfType.map((it) => (
                      <div 
                        key={it.fullPath} 
                        className="item-card"
                        style={{ 
                          display: 'flex', 
                          justifyContent: 'space-between', 
                          alignItems: 'center',
                          padding: '1rem',
                          background: 'var(--bg-panel)',
                          borderRadius: 'var(--border-radius)',
                          border: '1px solid rgba(255,255,255,0.05)'
                        }}
                      >
                        <div style={{ flex: 1 }}>
                          <div className="item-title" style={{ fontWeight: '500', marginBottom: '0.25rem' }}>
                            {it.title}
                          </div>
                          <div className="muted" style={{ fontSize: '0.85em', marginBottom: '0.25rem' }}>
                            {it.mediaType === 'video' ? '🎥 Video' : '🎵 Audio'}
                          </div>
                          <div className="muted item-path" style={{ fontSize: '0.75em', wordBreak: 'break-all' }}>
                            {it.fullPath}
                          </div>
                        </div>
                        <button 
                          className="btn btn-primary" 
                          onClick={() => onAddToQueue(it)}
                          style={{ marginLeft: '1rem', whiteSpace: 'nowrap' }}
                        >
                          ➕ A cola
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}