import React, { useEffect, useRef } from "react";
import type { QueueItem } from "../types";

export default function KaraokeOverlay(props: {
  open: boolean;
  onClose: () => void;
  nowPlaying: QueueItem | null;
  queue: QueueItem[];
  onPlayNext: () => void;
}) {
  const { open, onClose, nowPlaying, queue, onPlayNext } = props;
  const rootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!open) return;

    // “F11-like”: pedir fullscreen del contenedor overlay
    const el = rootRef.current;
    if (el?.requestFullscreen) el.requestFullscreen().catch(() => {});

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
      if (e.key.toLowerCase() === "f") toggleFullscreen(el);
      if (e.key === "ArrowRight") onPlayNext();
    };

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose, onPlayNext]);

  if (!open) return null;

  return (
    <div className="karaoke" ref={rootRef}>
      <div className="karaoke-menu">
        <button className="btn" onClick={() => toggleFullscreen(rootRef.current)}>⛶ Fullscreen (F)</button>
        <button className="btn" onClick={onPlayNext}>⏭ Siguiente (→)</button>
        <button className="btn" onClick={onClose}>Cerrar (Esc)</button>
      </div>

      <div className="karaoke-stage">
        <div className="karaoke-title">
          {nowPlaying ? `${nowPlaying.artistName} — ${nowPlaying.title}` : "Sin reproducción"}
        </div>

        <div className="karaoke-sub muted">
          Cola: {queue.length} • (Esc cerrar / F fullscreen / → siguiente)
        </div>

        {/* Placeholder para letras: luego metemos lyrics provider o archivo .lrc */}
        <div className="karaoke-lyrics muted">
          Aquí van las letras (LRC / provider). Por ahora es overlay + menú.
        </div>
      </div>
    </div>
  );
}

function toggleFullscreen(el: any) {
  try {
    if (!document.fullscreenElement) {
      el?.requestFullscreen?.();
    } else {
      document.exitFullscreen?.();
    }
  } catch {}
}
