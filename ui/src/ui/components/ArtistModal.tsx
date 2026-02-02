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

  return (
    <div className="modal-backdrop">
      <div className="modal modal-clean">
        <div className="modal-header">
          <div>
            <div className="modal-title">{artist?.displayName ?? "Artista"}</div>
            <div className="muted">Agrega canciones/videos a la cola</div>
          </div>
          <button className="btn" onClick={onClose}>Cerrar</button>
        </div>

        <div className="modal-body panel-scroll" style={{ maxHeight: "70vh" }}>
          {loading ? (
            <div className="muted">Cargando…</div>
          ) : items.length === 0 ? (
            <div className="muted">No hay items disponibles (o ya están en cola).</div>
          ) : (
            <div className="items">
              {items.map((it) => (
                <div key={it.fullPath} className="item-card">
                  <div className="item-title">{it.title}</div>
                  <div className="muted">{it.mediaType}</div>
                  <div className="muted item-path">{it.fullPath}</div>
                  <button className="btn btn-primary" onClick={() => onAddToQueue(it)}>+ Cola</button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
