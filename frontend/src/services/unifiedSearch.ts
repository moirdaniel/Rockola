/**
 * Unified Search Service.
 * Consulta todos los adapters y combina resultados; el usuario no nota la fuente.
 */

import type { MediaItem, ArtistSummary } from '@/types/media'
import { getAdapters } from '@/adapters'

export interface SearchResult {
  items: MediaItem[]
  artists: ArtistSummary[]
  query: string
  sourcesQueried: string[]
}

export async function searchMedia(query: string): Promise<SearchResult> {
  const trimmed = query.trim()
  if (!trimmed) {
    return { items: [], artists: [], query: trimmed, sourcesQueried: [] }
  }

  const adapters = getAdapters()
  const results = await Promise.all(
    adapters.map(async (adapter) => {
      try {
        const { items, artists = [] } = await adapter.search(trimmed)
        return { sourceId: adapter.sourceId, items, artists }
      } catch (err) {
        console.warn(`Search failed for ${adapter.sourceId}:`, err)
        return { sourceId: adapter.sourceId, items: [] as MediaItem[], artists: [] as ArtistSummary[] }
      }
    })
  )

  const items: MediaItem[] = results.flatMap((r) => r.items)
  const artists: ArtistSummary[] = results.flatMap((r) => r.artists ?? [])
  const sourcesQueried = results.map((r) => r.sourceId)

  return { items, artists, query: trimmed, sourcesQueried }
}
