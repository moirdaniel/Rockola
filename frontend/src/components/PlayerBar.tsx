import { useEffect, useRef } from 'react'
import { useLocation } from 'react-router-dom'
import { usePlayerStore } from '@/stores/playerStore'
import { useQueueStore } from '@/stores/queueStore'
import { usePlayerRefs } from '@/contexts/PlayerRefsContext'
import { getAdapterBySourceId } from '@/adapters'
import { queueApi } from '@/services/queueApi'

export function PlayerBar() {
  const { audioRef, videoRef } = usePlayerRefs()
  const pathname = useLocation().pathname
  const {
    currentItem,
    isPlaying,
    currentTime,
    duration,
    volume,
    error,
    setCurrentItem,
    setPlaying,
    setCurrentTime,
    setDuration,
    setError,
    setStreamUrl,
  } = usePlayerStore()
  const { items, setItems } = useQueueStore()

  // Cargar stream cuando cambia la pista actual
  useEffect(() => {
    if (!currentItem) {
      setStreamUrl(null)
      return
    }
    const el = currentItem.type === 'video' ? videoRef.current : audioRef.current
    if (!el) return
    setError(null)
    let cancelled = false
    ;(async () => {
      try {
        const adapter = getAdapterBySourceId(currentItem.source)
        const url =
          currentItem.streamId ||
          (adapter ? await adapter.getStreamUrl(currentItem.id) : null)
        if (cancelled || !url) return
        ;(el as HTMLMediaElement).src = url
        setStreamUrl(currentItem.type === 'video' ? url : null)
        if (usePlayerStore.getState().isPlaying) {
          await (el as HTMLMediaElement).play()
        }
      } catch (e) {
        if (!cancelled) setError(e instanceof Error ? e.message : 'Error al cargar el medio')
      }
    })()
    return () => { cancelled = true; setStreamUrl(null) }
  }, [currentItem?.id, currentItem?.source, currentItem?.type, pathname, setError, setStreamUrl])

  // Sincronizar play/pause con elemento nativo
  useEffect(() => {
    const el = currentItem?.type === 'video' ? videoRef.current : audioRef.current
    if (!el) return
    if (isPlaying) el.play().catch((e) => setError(e.message))
    else el.pause()
  }, [currentItem, isPlaying, setError])

  useEffect(() => {
    const el = currentItem?.type === 'video' ? videoRef.current : audioRef.current
    if (!el) return
    el.volume = volume
  }, [volume, currentItem?.type])

  const loadAndPlay = async (item: typeof currentItem) => {
    if (!item) return
    setError(null)
    try {
      const adapter = getAdapterBySourceId(item.source)
      const url = item.streamId || (adapter ? await adapter.getStreamUrl(item.id) : null)
      if (!url) throw new Error('No se pudo obtener la URL de reproducción')
      const el = item.type === 'video' ? videoRef.current : audioRef.current
      if (el) {
        (el as HTMLMediaElement).src = url
        setStreamUrl(item.type === 'video' ? url : null)
        await (el as HTMLMediaElement).play()
        setPlaying(true)
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Error al cargar el medio')
      setPlaying(false)
    }
  }

  const handleEnded = async () => {
    setPlaying(false)
    setStreamUrl(null)
    const nextQueue = await queueApi.next()
    setItems(nextQueue)
    const next = nextQueue[0]
    if (next) {
      setCurrentItem(next)
      loadAndPlay(next)
    } else {
      setCurrentItem(null)
      setDuration(0)
      setCurrentTime(0)
    }
  }

  const togglePlay = () => {
    if (!currentItem) return
    const el = currentItem.type === 'video' ? videoRef.current : audioRef.current
    if (!el) return
    if (isPlaying) el.pause()
    else el.play().catch((e) => setError(e.message))
    setPlaying(!isPlaying)
  }

  const skip = () => {
    handleEnded()
  }

  const handleEndedRef = useRef(handleEnded)
  handleEndedRef.current = handleEnded

  // Adjuntar handlers a los elementos de media (viven en PlayerRefsProvider)
  useEffect(() => {
    const video = videoRef.current
    const audio = audioRef.current
    if (!video && !audio) return
    const handleTimeUpdate = (e: Event) => {
      const el = e.target as HTMLMediaElement
      if (el) usePlayerStore.getState().setCurrentTime(el.currentTime)
    }
    const handleLoadedMetadata = (e: Event) => {
      const el = e.target as HTMLMediaElement
      if (el) usePlayerStore.getState().setDuration(el.duration)
    }
    const handleEndedEv = () => handleEndedRef.current()
    const handleError = (e: Event) => {
      const el = e.currentTarget as HTMLMediaElement | null
      const mediaError = el?.error
      let message = 'Error de reproducción'
      if (mediaError) {
        switch (mediaError.code) {
          case mediaError.MEDIA_ERR_ABORTED:
            message = 'Reproducción cancelada.'
            break
          case mediaError.MEDIA_ERR_NETWORK:
            message = 'Error de red al reproducir el medio.'
            break
          case mediaError.MEDIA_ERR_DECODE:
            message = 'No se pudo decodificar el archivo de media.'
            break
          case mediaError.MEDIA_ERR_SRC_NOT_SUPPORTED:
            message = 'Fuente de media no soportada o no encontrada.'
            break
        }
      }
      // Guardamos un mensaje amigable y volcamos detalles a consola para diagnóstico
      // (útil en Tauri / navegador para ver el src y el código de error).
      // eslint-disable-next-line no-console
      console.error('[Player] Media error', {
        src: el?.currentSrc,
        code: mediaError?.code,
        message,
      })
      usePlayerStore.getState().setError(message)
    }
    if (video) {
      video.ontimeupdate = handleTimeUpdate
      video.onloadedmetadata = handleLoadedMetadata
      video.onended = handleEndedEv
      video.onerror = handleError
    }
    if (audio) {
      audio.ontimeupdate = handleTimeUpdate
      audio.onloadedmetadata = handleLoadedMetadata
      audio.onended = handleEndedEv
      audio.onerror = handleError
    }
    return () => {
      if (video) {
        video.ontimeupdate = null
        video.onloadedmetadata = null
        video.onended = null
        video.onerror = null
      }
      if (audio) {
        audio.ontimeupdate = null
        audio.onloadedmetadata = null
        audio.onended = null
        audio.onerror = null
      }
    }
  }, [currentItem?.type])

  const formatTime = (s: number) => {
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  if (!currentItem) {
    return (
      <div className="h-16 glass-panel border-t border-white/10 flex items-center justify-center text-gray-500 text-sm">
        Nada en reproducción. Busca y agrega a la cola.
      </div>
    )
  }

  return (
    <div className="h-20 glass-panel border-t border-white/10 flex items-center gap-4 px-4">
        <div className="flex-1 min-w-0 flex items-center gap-3">
          {currentItem.thumbnailUrl && (
            <img
              src={currentItem.thumbnailUrl}
              alt=""
              className="w-12 h-12 rounded object-cover flex-shrink-0"
            />
          )}
          <div className="min-w-0">
            <p className="font-medium truncate">{currentItem.title}</p>
            <p className="text-sm text-gray-400 truncate">{currentItem.artist ?? '—'}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={togglePlay}
            className="w-10 h-10 rounded-full bg-jukebox-primary flex items-center justify-center text-white hover:bg-jukebox-primary/80 transition-colors"
            aria-label={isPlaying ? 'Pausar' : 'Reproducir'}
          >
            {isPlaying ? '⏸' : '▶'}
          </button>
          <button
            type="button"
            onClick={skip}
            className="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center hover:bg-white/20 transition-colors"
            aria-label="Siguiente"
          >
            ⏭
          </button>
        </div>
        <div className="hidden sm:flex items-center gap-2 text-sm text-gray-400 w-24">
          <span>{formatTime(currentTime)}</span>
          <span>/</span>
          <span>{formatTime(duration)}</span>
        </div>
        <div className="flex items-center gap-2 w-24">
          <span className="text-gray-500 text-xs" aria-hidden>🔊</span>
          <input
            type="range"
            min={0}
            max={1}
            step={0.05}
            value={volume}
            onChange={(e) => usePlayerStore.getState().setVolume(Number(e.target.value))}
            className="w-16 h-1.5 rounded-full bg-white/20 accent-jukebox-primary"
            aria-label="Volumen"
          />
        </div>
        {error && (
          <p className="text-red-400 text-sm max-w-32 truncate" title={error}>
            {error}
          </p>
        )}
    </div>
  )
}
