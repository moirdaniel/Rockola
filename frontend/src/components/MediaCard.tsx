import type { MediaItem } from '@/types/media'
import { useNavigate } from 'react-router-dom'
import { useCreditsStore } from '@/stores/creditsStore'
import { useQueueStore } from '@/stores/queueStore'
import { usePlayerStore } from '@/stores/playerStore'
import { useErrorStore } from '@/stores/errorStore'
import { queueApi, creditsApi } from '@/services/queueApi'
import { ApiError } from '@/services/api'

interface MediaCardProps {
  item: MediaItem
  onPlay?: () => void
}

export function MediaCard({ item, onPlay }: MediaCardProps) {
  const navigate = useNavigate()
  const { credits, costPerSong, setCredits, setError } = useCreditsStore()
  const { items: queueItems, setItems } = useQueueStore()
  const { currentItem } = usePlayerStore()

  const isInQueue = queueItems.some(
    (q) => q.id === item.id && q.source === item.source
  )
  const isPlaying =
    currentItem != null &&
    currentItem.id === item.id &&
    currentItem.source === item.source

  let disabledReason: 'no_credits' | 'in_queue' | 'playing' | null = null
  if (credits === null || credits.balance < costPerSong) {
    disabledReason = 'no_credits'
  } else if (isPlaying) {
    disabledReason = 'playing'
  } else if (isInQueue) {
    disabledReason = 'in_queue'
  }

  const canAdd = disabledReason === null

  const handleAddToQueue = async () => {
    if (!canAdd) return
    try {
      const updated = await queueApi.add({ mediaItem: item })
      useQueueStore.getState().setItems(updated)
      const fresh = await creditsApi.get()
      setCredits(fresh)
      // Reproducir siempre la canción recién agregada (última en la cola)
      const last = updated[updated.length - 1]
      if (last) {
        usePlayerStore.getState().setCurrentItem(last)
        usePlayerStore.getState().setPlaying(true)
        if (last.type === 'video') {
          navigate('/now-playing')
        }
      }
      onPlay?.()
    } catch (e) {
      const { showError } = useErrorStore.getState()
      if (e instanceof ApiError) {
        setError(e.message)
        showError({
          title: 'No se pudo agregar a la cola',
          message: e.message,
          details:             e.body && typeof e.body === 'object' ? JSON.stringify(e.body, null, 2) :                         `HTTP ${e.status}`,
        })
      } else {
        const msg = 'No se pudo agregar a la cola'
        setError(msg)
        showError({
          title: 'No se pudo agregar a la cola',
          message: msg,
          details: e instanceof Error ? e.message : String(e),
        })
      }
    }
  }

  return (
    <div className="glass-panel p-4 card-hover rounded-xl overflow-hidden group">
      <div className="aspect-square rounded-lg overflow-hidden bg-jukebox-card mb-3">
        {item.thumbnailUrl ? (
          <img
            src={item.thumbnailUrl}
            alt=""
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center text-4xl text-gray-600">
            {item.type === 'video' ? '🎬' : '🎵'}
          </div>
        )}
      </div>
      <h3 className="font-medium truncate" title={item.title}>
        {item.title}
      </h3>
      {item.artist && (
        <p className="text-sm text-gray-400 truncate">{item.artist}</p>
      )}
      <p className="text-xs text-gray-500 mt-1">
        {Math.floor(item.durationSeconds / 60)}:{String(item.durationSeconds % 60).padStart(2, '0')}
      </p>
      <button
        type="button"
        onClick={handleAddToQueue}
        disabled={!canAdd}
        className="mt-3 w-full py-2 rounded-lg bg-jukebox-primary/80 hover:bg-jukebox-primary disabled:opacity-60 disabled:cursor-not-allowed text-sm font-medium transition-colors"
      >
        {canAdd &&
          `Agregar a cola (${costPerSong} 💰)`}
        {!canAdd && disabledReason === 'no_credits' && 'Sin créditos'}
        {!canAdd && disabledReason === 'in_queue' && 'Ya en cola'}
        {!canAdd && disabledReason === 'playing' && 'Reproduciéndose'}
      </button>
      {!canAdd && disabledReason === 'in_queue' && (
        <p className="mt-1 text-xs text-jukebox-primary/80">
          Esta canción ya está en la cola.
        </p>
      )}
      {!canAdd && disabledReason === 'playing' && (
        <p className="mt-1 text-xs text-jukebox-primary/80">
          Esta canción se está reproduciendo.
        </p>
      )}
    </div>
  )
}
