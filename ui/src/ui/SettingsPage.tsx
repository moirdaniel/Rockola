import React, { useState } from "react";
import { addSource, startScan } from "../lib/tauri";

export default function SettingsPage(props: {
  open: boolean;
  onClose: () => void;
  sourcePath: string;
  setSourcePath: (v: string) => void;
  onScanned?: () => void; // callback para refrescar catálogo
}) {
  const { open, onClose, sourcePath, setSourcePath, onScanned } = props;
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string>("");

  if (!open) return null;

  async function handleSave() {
    const sp = sourcePath.trim();
    console.log("💾 [SETTINGS] handleSave()", { sp });

    if (!sp) {
      setMsg("Ruta vacía");
      return;
    }

    setBusy(true);
    setMsg("Guardando…");
    try {
      await addSource(sp);
      setMsg("OK: source registrado");
    } catch (e) {
      console.error("❌ [SETTINGS] addSource error:", e);
      setMsg("Error registrando source (ver consola)");
    } finally {
      setBusy(false);
    }
  }

  async function handleScan() {
    const sp = sourcePath.trim();
    console.log("🚀 [SETTINGS] handleScan()", { sp });

    if (!sp) {
      setMsg("Ruta vacía");
      return;
    }

    setBusy(true);
    setMsg("Escaneando…");
    try {
      // IMPORTANTÍSIMO: si antes lo moviste y quedó sin llamarse, acá se arregla
      await addSource(sp);
      await startScan(sp);

      setMsg("Scan OK");
      onScanned?.(); // fuerza refresh en catálogo
    } catch (e) {
      console.error("❌ [SETTINGS] startScan error:", e);
      setMsg("Error en scan (ver consola)");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="modal-backdrop">
      <div className="modal modal-clean">
        <div className="modal-header">
          <div className="modal-title">Configuración</div>
          <button className="btn btn-ghost" onClick={onClose}>
            ✕ Cerrar
          </button>
        </div>

        <div className="modal-content">
          <label className="label">Ruta source</label>
          <input
            className="input"
            placeholder="/storage/cloud/opendrive/Video Music/"
            value={sourcePath}
            onChange={(e) => setSourcePath(e.target.value)}
          />

          <div style={{ display: "flex", gap: 10, marginTop: 12 }}>
            <button className="btn" disabled={busy} onClick={handleSave}>
              Guardar
            </button>

            <button className="btn btn-primary" disabled={busy} onClick={handleScan}>
              Scan biblioteca
            </button>
          </div>

          {msg && <div className="muted" style={{ marginTop: 10 }}>{msg}</div>}
        </div>
      </div>
    </div>
  );
}
