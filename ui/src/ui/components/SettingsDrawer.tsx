import React, { useEffect, useState } from "react";
import { isTauri, addSource, startScan } from "../../lib/tauri";

export default function SettingsDrawer(props: {
  open: boolean;
  onClose: () => void;
  sourcePath: string;
  setSourcePath: (v: string) => void;
  scale: number;
  setScale: (v: number) => void;
}) {
  const { open, onClose, sourcePath, setSourcePath, scale, setScale } = props;

  const [tmpSource, setTmpSource] = useState(sourcePath);
  const tauriMode = isTauri();

  useEffect(() => {
    if (open) setTmpSource(sourcePath);
  }, [open, sourcePath]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    if (open) window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  async function runScan() {
    const sp = (sourcePath || "").trim();
    if (!sp) return;
    await addSource(sp);
    await startScan(sp);
  }

  return (
    <div className="drawer-backdrop" onMouseDown={onClose}>
      <div className="drawer" onMouseDown={(e) => e.stopPropagation()}>
        <div className="drawer-head">
          <div>
            <div style={{ fontWeight: 900, fontSize: 18 }}>Configuración</div>
            <div className="muted">Esc para cerrar</div>
          </div>
          <button className="btn btn-ghost" onClick={onClose}>
            ✕
          </button>
        </div>

        <div className="drawer-body">
          {/* Ruta Source */}
          <div className="setting">
            <div>
              <b>Ruta Source</b>
              <div className="muted">Carpeta raíz con artistas (subcarpetas).</div>
            </div>

            <div className="setting-controls" style={{ flex: 1 }}>
              <input
                className="input"
                value={tmpSource}
                onChange={(e) => setTmpSource(e.target.value)}
                placeholder="/storage/cloud/opendrive/Video Music"
              />
              <button
                className="btn"
                disabled={!tauriMode}
                onClick={() => setSourcePath(tmpSource.trim())}
              >
                Guardar
              </button>
            </div>
          </div>

          {/* Scan biblioteca (movido acá) */}
          <div className="setting">
            <div>
              <b>Biblioteca</b>
              <div className="muted">
                Escanea la ruta source e indexa artistas/canciones en SQLite.
              </div>
            </div>

            <div className="setting-controls">
              <button
                className="btn"
                disabled={!tauriMode || !sourcePath.trim()}
                onClick={runScan}
              >
                Scan biblioteca
              </button>
            </div>
          </div>

          {/* Escala UI */}
          <div className="setting">
            <div>
              <b>Escala UI</b>
              <div className="muted">Tamaño general de la interfaz.</div>
            </div>

            <div className="setting-controls">
              <input
                type="range"
                min={0.8}
                max={1.4}
                step={0.05}
                value={scale}
                onChange={(e) => setScale(Number(e.target.value))}
              />
              <div className="pill">{Math.round(scale * 100)}%</div>
            </div>
          </div>

          {!tauriMode && (
            <div className="muted" style={{ marginTop: 16 }}>
              Nota: Estás en modo web. Algunas acciones (scan/rutas locales) requieren Tauri.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
