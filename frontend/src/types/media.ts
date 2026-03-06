/**
 * Tipos compartidos para items de media (música/video).
 * El usuario no debe notar la fuente (local, YouTube, Spotify).
 */

export type MediaType = 'audio' | 'video'

export type MediaSource = 'local' | 'youtube' | 'spotify' | 'backend'

export interface MediaItem {
  id: string
  source: MediaSource
  title: string
  artist?: string
  album?: string
  durationSeconds: number
  thumbnailUrl?: string
  type: MediaType
  /** URL directa de stream o identificador para resolver */
  streamId?: string
}

export interface QueueItem extends MediaItem {
  queueId: string
  addedAt: string
  order: number
}

/** Resumen de artista/canal para la fila de filtro en búsqueda. */
export interface ArtistSummary {
  id: string
  name: string
  thumbnailUrl?: string
}

/** Respuesta de búsqueda del backend: artistas + canciones. */
export interface SearchResponse {
  artists: ArtistSummary[]
  songs: MediaItem[]
}

export interface PlaybackState {
  currentItem: QueueItem | null
  isPlaying: boolean
  currentTime: number
  duration: number
  volume: number
  isFullscreen: boolean
  error: string | null
  /** URL del stream actual (para mostrar video en NowPlaying/TV) */
  streamUrl: string | null
}
