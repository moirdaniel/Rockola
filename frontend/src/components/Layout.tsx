import { Link, Outlet, useLocation } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { PlayerBar } from '@/components/PlayerBar'
import { ErrorModal } from '@/components/ErrorModal'
import { PlayerRefsProvider } from '@/contexts/PlayerRefsContext'
import { useCreditsStore, useQueueStore, usePlayerStore } from '@/stores'
import { creditsApi, queueApi } from '@/services/queueApi'

const navItems = [
  { path: '/', label: 'Inicio' },
  { path: '/now-playing', label: 'Reproduciendo' },
  { path: '/queue', label: 'Cola' },
  { path: '/credits', label: 'Créditos' },
  { path: '/admin', label: 'Admin' },
  { path: '/pantallas', label: 'Pantallas' },
]

export function Layout() {
  const location = useLocation()
  const credits = useCreditsStore((s) => s.credits)
  const setCredits = useCreditsStore((s) => s.setCredits)
  const creditsError = useCreditsStore((s) => s.error)
  const setCreditsError = useCreditsStore((s) => s.setError)
  const setQueueItems = useQueueStore((s) => s.setItems)

  useQuery({
    queryKey: ['credits'],
    queryFn: async () => {
      const c = await creditsApi.get()
      setCredits(c)
      return c
    },
  })

  // Sincroniza la cola inicial desde el backend (para que sobreviva recargas)
  useQuery({
    queryKey: ['queue'],
    queryFn: async () => {
      const q = await queueApi.getQueue()
      setQueueItems(q)
       const player = usePlayerStore.getState()
       if (q.length === 0) {
         // No hay cola: limpiamos reproducción residual
         player.reset()
       } else if (!player.currentItem) {
         // Hay cola pero no hay currentItem (ej. tras recarga): usar la primera
         player.setCurrentItem(q[0])
       }
      return q
    },
  })

  return (
    <PlayerRefsProvider pathname={location.pathname}>
    <div className="flex flex-col min-h-screen bg-jukebox-dark">
      {location.pathname !== '/tv' && (
      <header className="glass-panel sticky top-0 z-40 border-b border-white/10">
        <div className="flex items-center justify-between px-4 py-3">
          <Link to="/" className="font-display text-xl font-bold text-jukebox-primary">
            🎵 Rockola Digital
          </Link>
          <nav className="flex items-center gap-2">
            {navItems.map(({ path, label }) => (
              <Link
                key={path}
                to={path}
                className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors ${
                  location.pathname === path
                    ? 'bg-jukebox-primary/30 text-white'
                    : 'text-gray-400 hover:text-white hover:bg-white/10'
                }`}
              >
                {label}
              </Link>
            ))}
            <button
              type="button"
              onClick={() => {
                if (typeof window !== 'undefined') {
                  const url = `${window.location.origin}/tv`
                  window.open(url, 'rockola-tv', 'noopener,noreferrer')
                }
              }}
              className="px-2 py-1 rounded-lg text-xs font-medium bg-white/5 hover:bg-white/10 text-gray-300"
            >
              Vista TV
            </button>
            {credits !== null && (
              <span className="ml-2 px-2 py-1 rounded bg-jukebox-secondary/30 text-sm">
                💰 {credits.balance}
              </span>
            )}
          </nav>
        </div>
        {creditsError && (
          <div className="px-4 py-2 bg-red-500/20 text-red-400 text-sm flex items-center justify-between">
            <span>{creditsError}</span>
            <button type="button" onClick={() => setCreditsError(null)} aria-label="Cerrar">×</button>
          </div>
        )}
      </header>
      )}

      <ErrorModal />

      <main className={`flex-1 overflow-auto ${location.pathname === '/now-playing' || location.pathname === '/tv' ? 'bg-transparent' : ''}`}>
        <Outlet />
      </main>

      <PlayerBar />
    </div>
    </PlayerRefsProvider>
  )
}
