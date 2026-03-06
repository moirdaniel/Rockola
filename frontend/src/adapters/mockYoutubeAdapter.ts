/**
 * Adapter mock para YouTube.
 * Simula búsqueda y resolución de URLs sin API real.
 */

import type { MediaSourceAdapter } from './MediaSourceAdapter'
import type { MediaItem } from '@/types/media'

const MOCK_VIDEOS: MediaItem[] = [
  {
    id: 'yt-mock-1',
    source: 'youtube',
    title: 'Official Music Video - Demo',
    artist: 'Artist Channel',
    durationSeconds: 245,
    thumbnailUrl: 'https://picsum.photos/seed/yt1/300/300',
    type: 'video',
    streamId: 'dQw4w9WgXcQ',
  },
  {
    id: 'yt-mock-2',
    source: 'youtube',
    title: 'Acoustic Session',
    artist: 'Live Band',
    durationSeconds: 312,
    thumbnailUrl: 'https://picsum.photos/seed/yt2/300/300',
    type: 'video',
    streamId: 'yt-mock-id-2',
  },
]

export const mockYoutubeAdapter: MediaSourceAdapter = {
  sourceId: 'youtube',
  displayName: 'YouTube',

  async search(query: string) {
    const q = query.toLowerCase()
    const items = MOCK_VIDEOS.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        (item.artist?.toLowerCase().includes(q) ?? false)
    )
    return { items, artists: [] }
  },

  async getStreamUrl(id: string): Promise<string> {
    // En producción aquí se resolvería la URL real (proxy/backend).
    return `https://www.youtube.com/watch?v=${id}`
  },

  validateId(id: string): boolean {
    return id.startsWith('yt-') || id.length === 11
  },
}
