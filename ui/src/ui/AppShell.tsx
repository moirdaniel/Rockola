import React, { useEffect, useState } from "react";
import CatalogPage from "./CatalogPage";
import SettingsDrawer from "./components/SettingsDrawer";
import { Settings } from "../lib/settings";

type Theme = "light" | "dark";

export default function AppShell() {
  const [theme, setTheme] = useState<Theme>(() => Settings.getTheme());
  const [scale, setScale] = useState<number>(() => Settings.getScale());
  const [sourcePath, setSourcePath] = useState<string>(() => Settings.getSourcePath());
  const [settingsOpen, setSettingsOpen] = useState(false);

  // Acciones dinámicas que vienen desde CatalogPage (Karaoke / Siguiente)
  const [topbarActions, setTopbarActions] = useState<React.ReactNode>(null);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.style.fontSize = `${16 * scale}px`;
    Settings.setTheme(theme);
    Settings.setScale(scale);
  }, [theme, scale]);

  const toggleTheme = () => setTheme((t) => (t === "dark" ? "light" : "dark"));

  // Manejar el evento de abrir configuración desde el catálogo
  useEffect(() => {
    const handleOpenSettings = () => setSettingsOpen(true);
    window.addEventListener("open-settings", handleOpenSettings);
    
    return () => window.removeEventListener("open-settings", handleOpenSettings);
  }, []);

  return (
    <div className="app">
      <header className="topbar">
        <div className="brand">
          <div className="title">🎵 Rockola</div>
          <div className="sub">Desktop • Tauri</div>
        </div>

        <div className="actions">
          {topbarActions}

          <button className="btn" onClick={() => setSettingsOpen(true)}>
            ⚙️ Configuración
          </button>

          <button className="btn" onClick={toggleTheme}>
            {theme === "dark" ? "☀️ Claro" : "🌙 Oscuro"}
          </button>
        </div>
      </header>

      <main className="main">
        <CatalogPage
          sourcePath={sourcePath}
          setTopbarActions={setTopbarActions}
        />
      </main>

      <SettingsDrawer
        open={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        sourcePath={sourcePath}
        setSourcePath={(v) => {
          setSourcePath(v);
          Settings.setSourcePath(v);
        }}
        scale={scale}
        setScale={setScale}
      />
    </div>
  );
}
