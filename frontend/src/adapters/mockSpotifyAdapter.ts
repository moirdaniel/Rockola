/**
 * Adapter mock para Spotify.
 * Simula búsqueda de canciones sin API real.
 */

import type { MediaSourceAdapter } from './MediaSourceAdapter'
import type { MediaItem } from '@/types/media'

const MOCK_TRACKS: MediaItem[] = [
  {
    id: 'spotify-mock-1',
    source: 'spotify',
    title: 'Blinding Lights',
    artist: 'The Weeknd',
    album: 'After Hours',
    durationSeconds: 200,
    thumbnailUrl: 'https://picsum.photos/seed/weeknd/300/300',
    type: 'audio',
    streamId: 'spotify:track:mock1',
  },
  {
    id: 'spotify-mock-2',
    source: 'spotify',
    title: 'Shape of You',
    artist: 'Ed Sheeran',
    album: '÷',
    durationSeconds: 234,
    thumbnailUrl: 'https://picsum.photos/seed/edsheeran/300/300',
    type: 'audio',
    streamId: 'spotify:track:mock2',
  },
]

export const mockSpotifyAdapter: MediaSourceAdapter = {
  sourceId: 'spotify',
  displayName: 'Spotify',

  async search(query: string) {
    const q = query.toLowerCase()
    const items = MOCK_TRACKS.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        (item.artist?.toLowerCase().includes(q) ?? false) ||
        (item.album?.toLowerCase().includes(q) ?? false)
    )
    return { items, artists: [] }
  },

  async getStreamUrl(id: string): Promise<string> {
    return `spotify:track:${id}`
  },

  validateId(id: string): boolean {
    return id.startsWith('spotify-') || id.startsWith('spotify:')
  },
}
