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
  const [autoPlayDelay, setAutoPlayDelay] = useState(10);
  const [idleTime, setIdleTime] = useState(10);
  const [theme, setTheme] = useState('dark');
  const tauriMode = isTauri();

  useEffect(() => {
    if (open) {
      setTmpSource(sourcePath);
      // Aquí se podrían cargar otros valores de configuración guardados
    }
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
            <div style={{ fontWeight: 900, fontSize: 18 }}>⚙️ Configuración</div>
            <div className="muted">Presiona Esc para cerrar</div>
          </div>
          <button className="btn btn-ghost" onClick={onClose} style={{ padding: '0.5rem 1rem' }}>
            ✕
          </button>
        </div>

        <div className="drawer-body panel-scroll" style={{ maxHeight: '70vh' }}>
          {/* Ruta Source */}
          <div className="setting">
            <div>
              <b>📁 Ruta de Medios</b>
              <div className="muted">Carpeta raíz con artistas (subcarpetas).</div>
            </div>

            <div className="setting-controls" style={{ flex: 1 }}>
              <input
                className="input"
                value={tmpSource}
                onChange={(e) => setTmpSource(e.target.value)}
                placeholder="/ruta/a/tus/medios"
                style={{ marginBottom: '0.5rem' }}
              />
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button
                  className="btn"
                  disabled={!tauriMode || tmpSource.trim() === sourcePath.trim()}
                  onClick={() => setSourcePath(tmpSource.trim())}
                >
                  💾 Guardar
                </button>
                <button
                  className="btn"
                  disabled={!tauriMode || !sourcePath.trim()}
                  onClick={runScan}
                >
                  🔍 Escanear
                </button>
              </div>
            </div>
          </div>

          {/* Configuración de reproducción */}
          <div className="setting">
            <div>
              <b>⏯️ Reproducción</b>
              <div className="muted">Configuración de tiempos de reproducción automática.</div>
            </div>

            <div className="setting-controls">
              <div style={{ marginBottom: '1rem' }}>
                <label className="muted" style={{ display: 'block', marginBottom: '0.5rem' }}>
                  Tiempo de espera antes de reproducir (segundos)
                </label>
                <input
                  type="range"
                  min={3}
                  max={30}
                  step={1}
                  value={autoPlayDelay}
                  onChange={(e) => setAutoPlayDelay(Number(e.target.value))}
                  style={{ width: '100%' }}
                />
                <div className="pill" style={{ marginTop: '0.5rem' }}>{autoPlayDelay}s</div>
              </div>
              
              <div>
                <label className="muted" style={{ display: 'block', marginBottom: '0.5rem' }}>
                  Tiempo de inactividad para fullscreen (segundos)
                </label>
                <input
                  type="range"
                  min={5}
                  max={60}
                  step={5}
                  value={idleTime}
                  onChange={(e) => setIdleTime(Number(e.target.value))}
                  style={{ width: '100%' }}
                />
                <div className="pill" style={{ marginTop: '0.5rem' }}>{idleTime}s</div>
              </div>
            </div>
          </div>

          {/* Escala UI */}
          <div className="setting">
            <div>
              <b>🔍 Escala de Interfaz</b>
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
                style={{ width: '100%', marginBottom: '0.5rem' }}
              />
              <div className="pill">{Math.round(scale * 100)}%</div>
            </div>
          </div>

          {/* Tema */}
          <div className="setting">
            <div>
              <b>🎨 Tema Visual</b>
              <div className="muted">Selecciona el tema de color.</div>
            </div>

            <div className="setting-controls">
              <select
                value={theme}
                onChange={(e) => setTheme(e.target.value)}
                className="input"
                style={{ width: 'auto' }}
              >
                <option value="dark">Oscuro</option>
                <option value="light">Claro</option>
              </select>
            </div>
          </div>

          {/* Información del sistema */}
          <div className="setting">
            <div>
              <b>ℹ️ Información</b>
              <div className="muted">Detalles del sistema y modo de operación.</div>
            </div>

            <div className="setting-controls">
              <div className="muted">
                <div>Modo: {tauriMode ? '🖥️ Tauri (Desktop)' : '🌐 Web'}</div>
                <div>Ruta actual: {sourcePath || 'No configurada'}</div>
              </div>
            </div>
          </div>

          {!tauriMode && (
            <div className="muted" style={{ marginTop: 16, padding: '1rem', backgroundColor: 'rgba(244,67,54,0.1)', borderRadius: 'var(--border-radius)' }}>
              📝 Nota: Estás en modo web. Algunas funciones (escaneo de rutas locales) requieren la versión de escritorio.
            </div>
          )}
        </div>
        
        <div className="drawer-footer" style={{ borderTop: '1px solid rgba(255,255,255,0.1)', padding: '1rem', textAlign: 'right' }}>
          <button className="btn" onClick={onClose}>
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
}