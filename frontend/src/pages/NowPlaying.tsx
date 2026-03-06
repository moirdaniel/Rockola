import { usePlayerStore } from '@/stores/playerStore'

export function NowPlaying() {
  const { currentItem, isPlaying, currentTime, duration, error } = usePlayerStore()

  if (!currentItem) {
    return (
      <div className="p-6 max-w-2xl mx-auto text-center">
        <p className="text-gray-500 text-lg">No hay nada reproduciéndose.</p>
        <p className="text-gray-600 text-sm mt-2">Agrega canciones desde Inicio.</p>
      </div>
    )
  }

  const progress = duration > 0 ? (currentTime / duration) * 100 : 0

  return (
    <div className="absolute inset-0 flex flex-col justify-end p-6 pointer-events-none">
      <div className="pointer-events-auto max-w-2xl w-full mx-auto">
        <div className="glass-panel p-4 rounded-t-xl">
          <h2 className="text-lg font-semibold truncate">{currentItem.title}</h2>
          <p className="text-gray-400 text-sm truncate">{currentItem.artist ?? '—'}</p>
          <p className="text-xs text-gray-500 mt-1">
            {isPlaying ? '▶ Reproduciendo' : '⏸ Pausado'}
          </p>
          {error && <p className="mt-1 text-red-400 text-sm">{error}</p>}
          <div className="mt-3">
            <div className="h-2 rounded-full bg-white/10 overflow-hidden">
              <div
                className="h-full bg-jukebox-primary rounded-full transition-all duration-300"
                style={{ width: `${progress}%` }}
              />
            </div>
            <div className="flex justify-between text-xs text-gray-500 mt-1">
              <span>{formatTime(currentTime)}</span>
              <span>{formatTime(duration)}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

function formatTime(s: number) {
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${m}:${sec.toString().padStart(2, '0')}`
}
