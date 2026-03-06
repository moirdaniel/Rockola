/**
 * Interfaz extensible para adapters de fuentes de media.
 * Permite agregar YouTube, Spotify, música local sin que el usuario note la fuente.
 */

import type { MediaItem, ArtistSummary } from '@/types/media'

export interface SearchAdapterResult {
  items: MediaItem[]
  artists?: ArtistSummary[]
}

export interface MediaSourceAdapter {
  readonly sourceId: string
  readonly displayName: string

  /** Búsqueda unificada. Retorna items y opcionalmente artistas (para filtro). */
  search(query: string): Promise<SearchAdapterResult>

  /** Obtiene URL de stream o datos para reproducir. */
  getStreamUrl(id: string): Promise<string>

  /** Opcional: validar si un id es válido para esta fuente */
  validateId?(id: string): boolean
}
