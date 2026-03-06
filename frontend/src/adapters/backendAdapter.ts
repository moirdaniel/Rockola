/**
 * Adapter que usa el backend: búsqueda en índice local + YouTube (yt-dlp).
 * Los items devueltos tienen streamId apuntando al stream del backend.
 */

import type { MediaSourceAdapter, SearchAdapterResult } from './MediaSourceAdapter'
import type { MediaItem, ArtistSummary } from '@/types/media'
import { api, getBaseUrl, ApiError } from '@/services/api'
import { useErrorStore } from '@/stores/errorStore'

export const backendAdapter: MediaSourceAdapter = {
  sourceId: 'backend',
  displayName: 'Rockola (local + YouTube)',

  async search(query: string): Promise<SearchAdapterResult> {
    if (!query.trim()) return { items: [], artists: [] }
    const base = getBaseUrl()
    const path = `${base}/api/search?q=${encodeURIComponent(query.trim())}`
    try {
      const res = await api.get<{ artists: ArtistSummary[]; songs: MediaItem[] }>(path)
      const items = res.songs.map((item) => ({
        // Importante: mantenemos el id original (videoId) y la fuente real (youtube)
        // y solo rellenamos streamId con la URL del backend.
        ...item,
        streamId: `${base}/api/media/stream?id=${encodeURIComponent(item.id)}&source=${encodeURIComponent(item.source)}`,
      }))
      return { items, artists: res.artists ?? [] }
    } catch (e) {
      const { showError } = useErrorStore.getState()
      if (e instanceof ApiError) {
        showError({
          title: 'Error al buscar en Rockola',
          message: e.message,
          details: `HTTP ${e.status} - /api/search`,
        })
      } else {
        showError({
          title: 'Error al buscar en Rockola',
          message: 'No se pudo completar la búsqueda',
          details: e instanceof Error ? e.message : String(e),
        })
      }
      return { items: [], artists: [] }
    }
  },

  async getStreamUrl(id: string): Promise<string> {
    const base = getBaseUrl()
    const [source, realId] = id.includes(':') ? id.split(':', 2) : ['youtube', id]
    return `${base}/api/media/stream?id=${encodeURIComponent(realId)}&source=${encodeURIComponent(source)}`
  },

  validateId(id: string): boolean {
    return id.length > 0
  },
}
