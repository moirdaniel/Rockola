import { useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { queueApi } from '@/services/queueApi'
import { useQueueStore } from '@/stores/queueStore'
import { usePlayerStore } from '@/stores/playerStore'

export function Queue() {
  const navigate = useNavigate()
  const { data: items = [], isLoading, refetch } = useQuery({
    queryKey: ['queue'],
    queryFn: () => queueApi.getQueue(),
  })
  const { setItems } = useQueueStore()
  const { currentItem, setCurrentItem, reset } = usePlayerStore()

  // Sincronizar cola con store cuando llega del servidor
  useEffect(() => {
    setItems(items)
    if (items.length === 0) {
      // Si la cola está realmente vacía, asegúrate de que no quede reproducción "fantasma"
      reset()
    }
  }, [items, setItems])

  const handleClear = async () => {
    await queueApi.clear()
    setItems([])
    usePlayerStore.getState().setCurrentItem(null)
    refetch()
  }

  const handleSelect = (item: (typeof items)[number]) => {
    setCurrentItem(item)
    // Forzar reproducción cuando el usuario elige desde la cola
    usePlayerStore.getState().setPlaying(true)
    // Si es video, ir a la vista donde se muestra el reproductor
    if (item.type === 'video') {
      navigate('/now-playing')
    }
  }

  if (isLoading) {
    return (
      <div className="p-6 max-w-4xl mx-auto">
        <p className="text-gray-500">Cargando cola...</p>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h1 className="font-display text-2xl font-bold">Cola de reproducción</h1>
        {items.length > 0 && (
          <button
            type="button"
            onClick={handleClear}
            className="px-4 py-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500/30 transition-colors text-sm"
          >
            Vaciar cola
          </button>
        )}
      </div>
      {items.length === 0 ? (
        <p className="text-gray-500 py-8 text-center">La cola está vacía.</p>
      ) : (
        <ul className="space-y-2">
          {items.map((item, index) => {
            const isCurrent =
              currentItem != null && currentItem.queueId === item.queueId
            return (
              <li
                key={item.queueId}
                onClick={() => handleSelect(item)}
                className={`glass-panel p-4 flex items-center gap-4 rounded-xl cursor-pointer transition-colors ${
                  isCurrent ? 'ring-2 ring-jukebox-primary/70 bg-jukebox-card' : ''
                }`}
              >
                <span className="text-gray-500 w-8">{index + 1}</span>
                {item.thumbnailUrl && (
                  <img
                    src={item.thumbnailUrl}
                    alt=""
                    className="w-12 h-12 rounded object-cover"
                  />
                )}
                <div className="flex-1 min-w-0">
                  <p className="font-medium truncate">
                    {item.title}
                    {isCurrent && (
                      <span className="ml-2 text-xs text-jukebox-primary align-middle">
                        (reproduciendo)
                      </span>
                    )}
                  </p>
                  <p className="text-sm text-gray-400 truncate">{item.artist ?? '—'}</p>
                </div>
                <span className="text-sm text-gray-500">
                  {Math.floor(item.durationSeconds / 60)}:
                  {String(item.durationSeconds % 60).padStart(2, '0')}
                </span>
              </li>
            )
          })}
        </ul>
      )}
    </div>
  )
}
