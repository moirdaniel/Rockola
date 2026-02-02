import React from "react";
import type { ReactNode, Dispatch, SetStateAction } from "react";

import type { Artist, ItemRow, QueueItem } from "./types";
import { isTauri, addSource, startScan, listArtists, listItemsByArtist } from "../lib/tauri";
import ArtistModal from "./components/ArtistModal";
import PlayerPanel from "./PlayerPanel";

type Props = {
  sourcePath: string;
  setTopbarActions: Dispatch<SetStateAction<ReactNode>>;
};

const AUTOSTART_SECONDS = 10;
const AUTO_FULLSCREEN_IDLE_SECONDS = 10;

export default function CatalogPage({ sourcePath, setTopbarActions }: Props) {
  // ===== State =====
  const [artists, setArtists] = React.useState<Artist[]>([]);
  const [artistFilter, setArtistFilter] = React.useState("");
  const [loadingArtists, setLoadingArtists] = React.useState(false);

  const [queue, setQueue] = React.useState<QueueItem[]>([]);
  const [nowPlaying, setNowPlaying] = React.useState<QueueItem | null>(null);

  // pending start
  const [pending, setPending] = React.useState<QueueItem | null>(null);
  const [pendingLeft, setPendingLeft] = React.useState<number>(AUTOSTART_SECONDS);

  // artist modal
  const [artistModalOpen, setArtistModalOpen] = React.useState(false);
  const [selectedArtist, setSelectedArtist] = React.useState<Artist | null>(null);
  const [artistItems, setArtistItems] = React.useState<ItemRow[]>([]);
  const [artistItemsLoading, setArtistItemsLoading] = React.useState(false);

  // fullscreen
  const [isFullscreen, setIsFullscreen] = React.useState(false);

  // theme
  const [dark, setDark] = React.useState(true);

  // inactivity timer
  const idleTimerRef = React.useRef<number | null>(null);
  const pendingTimerRef = React.useRef<number | null>(null);

  const queuePaths = React.useMemo(() => new Set(queue.map((q) => q.fullPath)), [queue]);

  const filteredArtists = React.useMemo(() => {
    const f = artistFilter.trim().toLowerCase();
    if (!f) return artists;
    return artists.filter((a) => a.displayName.toLowerCase().includes(f));
  }, [artists, artistFilter]);

  // ===== Backend calls =====
  const refreshArtists = React.useCallback(async () => {
    setLoadingArtists(true);
    try {
      const sp = (sourcePath || "").trim();
      if (isTauri() && sp.length > 0) {
        await addSource(sp);
      }
      const res = await listArtists();
      setArtists(res);
    } catch (e) {
      console.error("[CatalogPage] refreshArtists failed:", e);
      setArtists([]);
    } finally {
      setLoadingArtists(false);
    }
  }, [sourcePath]);

  const runScan = React.useCallback(async () => {
    try {
      const sp = (sourcePath || "").trim();
      if (!sp) return;
      if (!isTauri()) return;

      await startScan(sp);
      await refreshArtists();
    } catch (e) {
      console.error("[CatalogPage] runScan failed:", e);
    }
  }, [sourcePath, refreshArtists]);

  React.useEffect(() => {
    void refreshArtists();
  }, [refreshArtists]);

  // ===== Open artist modal =====
  async function openArtist(artist: Artist) {
    setSelectedArtist(artist);
    setArtistModalOpen(true);
    setArtistItems([]);
    setArtistItemsLoading(true);

    try {
      const items = await listItemsByArtist(artist.id);
      setArtistItems(items.filter((it) => !queuePaths.has(it.fullPath)));
    } catch (e) {
      console.error("[CatalogPage] openArtist failed:", e);
      setArtistItems([]);
    } finally {
      setArtistItemsLoading(false);
    }
  }

  // ===== Queue ops =====
  function addToQueue(it: ItemRow) {
    setQueue((prev) => {
      if (prev.some((q) => q.fullPath === it.fullPath)) return prev;

      const next: QueueItem = {
        id: it.id,
        title: it.title,
        fullPath: it.fullPath,
        mediaType: it.mediaType,
        artistName: selectedArtist?.displayName ?? "",
      };

      return [...prev, next];
    });

    // desaparecer del modal
    setArtistItems((prev) => prev.filter((x) => x.fullPath !== it.fullPath));
  }

  function removeFromQueue(fullPath: string) {
    setQueue((prev) => prev.filter((q) => q.fullPath !== fullPath));

    // si removiste el que está pending/playing, lo limpiamos
    setPending((p) => (p?.fullPath === fullPath ? null : p));
    setNowPlaying((n) => (n?.fullPath === fullPath ? null : n));
  }

  // ===== Pending start logic =====
  function schedulePlay(q: QueueItem) {
    // set pending + countdown
    setPending(q);
    setPendingLeft(AUTOSTART_SECONDS);

    // limpiar timer previo
    if (pendingTimerRef.current) {
      window.clearInterval(pendingTimerRef.current);
      pendingTimerRef.current = null;
    }

    // tick cada 1s
    pendingTimerRef.current = window.setInterval(() => {
      setPendingLeft((s) => {
        if (s <= 1) return 0;
        return s - 1;
      });
    }, 1000);
  }

  function cancelPending() {
    if (pendingTimerRef.current) {
      window.clearInterval(pendingTimerRef.current);
      pendingTimerRef.current = null;
    }
    setPending(null);
    setPendingLeft(AUTOSTART_SECONDS);
  }

  function playNow() {
    if (!pending) return;
    // stop timer
    if (pendingTimerRef.current) {
      window.clearInterval(pendingTimerRef.current);
      pendingTimerRef.current = null;
    }
    setNowPlaying(pending);
    setPending(null);
    setPendingLeft(AUTOSTART_SECONDS);

    // al iniciar reproducción, programamos idle fullscreen
    resetIdleTimer(true);
  }

  // Cuando countdown llega a 0 -> playNow
  React.useEffect(() => {
    if (!pending) return;
    if (pendingLeft > 0) return;
    playNow();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pendingLeft, pending]);

  // ===== Inactivity -> fullscreen =====
  function clearIdleTimer() {
    if (idleTimerRef.current) {
      window.clearTimeout(idleTimerRef.current);
      idleTimerRef.current = null;
    }
  }

  function resetIdleTimer(onlyIfPlaying: boolean) {
    clearIdleTimer();

    if (onlyIfPlaying && !nowPlaying) return;

    idleTimerRef.current = window.setTimeout(() => {
      // si sigue reproduciendo y no está fullscreen, entra fullscreen
      if (nowPlaying && !isFullscreen) setIsFullscreen(true);
    }, AUTO_FULLSCREEN_IDLE_SECONDS * 1000);
  }

  // Registrar actividad global (mouse/teclado) para resetear idle
  React.useEffect(() => {
    const onActivity = () => {
      // Si el usuario está interactuando, evitamos “autofullscreen”
      resetIdleTimer(true);
    };

    window.addEventListener("mousemove", onActivity, { passive: true });
    window.addEventListener("mousedown", onActivity, { passive: true });
    window.addEventListener("keydown", onActivity);
    window.addEventListener("wheel", onActivity, { passive: true });
    window.addEventListener("touchstart", onActivity, { passive: true });

    return () => {
      window.removeEventListener("mousemove", onActivity);
      window.removeEventListener("mousedown", onActivity);
      window.removeEventListener("keydown", onActivity);
      window.removeEventListener("wheel", onActivity);
      window.removeEventListener("touchstart", onActivity);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nowPlaying, isFullscreen]);

  // Si termina la reproducción (nowPlaying null), salimos de fullscreen y limpiamos timers
  React.useEffect(() => {
    if (!nowPlaying) {
      clearIdleTimer();
      setIsFullscreen(false);
    } else {
      resetIdleTimer(true);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [nowPlaying]);

  // ===== Click en cola: selecciona y programa countdown =====
  function selectQueueItem(q: QueueItem) {
    schedulePlay(q);
  }

  function playNextFromQueue() {
    setQueue((prev) => {
      if (prev.length === 0) return prev;
      const [next, ...rest] = prev;
      schedulePlay(next);
      return rest;
    });
  }

  // ===== Topbar =====
  React.useEffect(() => {
    setTopbarActions(
      <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
        <button className="btn" onClick={playNextFromQueue}>Siguiente</button>

        <button
          className="btn"
          onClick={() => {
            // tu AppShell abre settings con este event (si lo estás usando)
            window.dispatchEvent(new CustomEvent("open-settings"));
          }}
        >
          Configuración
        </button>

        <button className="btn" onClick={() => setDark((v) => !v)}>
          {dark ? "Oscuro" : "Claro"}
        </button>

        {isTauri() && (
          <button className="btn" onClick={() => void runScan()}>
            Scan biblioteca
          </button>
        )}
      </div>
    );

    return () => setTopbarActions(null);
  }, [setTopbarActions, dark, runScan]);

  // ===== Render =====
  return (
    <div className={`catalog ${dark ? "theme-dark" : "theme-light"}`}>
      <div className="grid">
        {/* LEFT */}
        <section className="panel panel-left">
          <div className="card">
            <input
              className="input"
              placeholder="Buscar artista..."
              value={artistFilter}
              onChange={(e) => setArtistFilter(e.target.value)}
            />
            <div className="muted" style={{ marginTop: 8 }}>
              Source: {sourcePath || "—"}
            </div>

            <div style={{ display: "flex", gap: 8, marginTop: 10 }}>
              <button className="btn" disabled={loadingArtists} onClick={() => void refreshArtists()}>
                {loadingArtists ? "Cargando..." : "Refresh"}
              </button>
            </div>
          </div>

          <div className="card panel-scroll" style={{ marginTop: 12 }}>
            <div className="card-title">Artistas ({filteredArtists.length})</div>

            {filteredArtists.length === 0 ? (
              <div className="muted" style={{ marginTop: 8 }}>
                Sin resultados.
              </div>
            ) : (
              <div className="list chips">
                {filteredArtists.map((a) => (
                  <button key={a.id} className="chip" onClick={() => void openArtist(a)}>
                    {a.displayName}
                  </button>
                ))}
              </div>
            )}
          </div>
        </section>

        {/* RIGHT */}
        <section className="panel panel-right">
          <PlayerPanel
            nowPlaying={nowPlaying}
            pending={pending}
            pendingSecondsLeft={pendingLeft}
            onCancelPending={cancelPending}
            onPlayNow={playNow}
            isFullscreen={isFullscreen}
            onRequestFullscreen={() => setIsFullscreen(true)}
            onExitFullscreen={() => setIsFullscreen(false)}
            statusText={`Auto-start: ${AUTOSTART_SECONDS}s • Auto-fullscreen idle: ${AUTO_FULLSCREEN_IDLE_SECONDS}s`}
          />

          <div className="card" style={{ marginTop: 12 }}>
            <div className="card-title">Cola ({queue.length})</div>
            <div className="muted">Click en un item para programar reproducción (10s).</div>

            <div className="panel-scroll" style={{ marginTop: 10, maxHeight: 320 }}>
              {queue.length === 0 ? (
                <div className="muted">Vacía.</div>
              ) : (
                <div className="queue grid-2">
                  {queue.map((q) => (
                    <div key={q.fullPath} className="queue-row">
                      <button className="queue-pick" onClick={() => selectQueueItem(q)}>
                        <div className="queue-title">{q.artistName} — {q.title}</div>
                        <div className="muted">{q.mediaType}</div>
                      </button>

                      <button className="btn" onClick={() => removeFromQueue(q.fullPath)}>
                        Quitar
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </section>
      </div>

      <ArtistModal
        open={artistModalOpen}
        artist={selectedArtist}
        loading={artistItemsLoading}
        items={artistItems}
        onClose={() => setArtistModalOpen(false)}
        onAddToQueue={addToQueue}
      />
    </div>
  );
}
