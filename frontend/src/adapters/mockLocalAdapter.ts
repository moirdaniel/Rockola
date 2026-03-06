/**
 * Adapter mock para música local.
 * Datos demo para desarrollo y pruebas.
 */

import type { MediaSourceAdapter } from './MediaSourceAdapter'
import type { MediaItem } from '@/types/media'

const DEMO_AUDIO: MediaItem[] = [
  {
    id: 'local-audio-1',
    source: 'local',
    title: 'Bohemian Rhapsody',
    artist: 'Queen',
    album: 'A Night at the Opera',
    durationSeconds: 354,
    thumbnailUrl: 'https://picsum.photos/seed/queen1/300/300',
    type: 'audio',
    streamId: '/demo/audio1.mp3',
  },
  {
    id: 'local-audio-2',
    source: 'local',
    title: 'Hotel California',
    artist: 'Eagles',
    album: 'Hotel California',
    durationSeconds: 391,
    thumbnailUrl: 'https://picsum.photos/seed/eagles1/300/300',
    type: 'audio',
    streamId: '/demo/audio2.mp3',
  },
  {
    id: 'local-audio-3',
    source: 'local',
    title: 'Sweet Child O\' Mine',
    artist: 'Guns N\' Roses',
    album: 'Appetite for Destruction',
    durationSeconds: 356,
    thumbnailUrl: 'https://picsum.photos/seed/gnr1/300/300',
    type: 'audio',
    streamId: '/demo/audio3.mp3',
  },
]

const DEMO_VIDEO: MediaItem[] = [
  {
    id: 'local-video-1',
    source: 'local',
    title: 'Live Concert 2024',
    artist: 'Various',
    durationSeconds: 7200,
    thumbnailUrl: 'https://picsum.photos/seed/video1/300/300',
    type: 'video',
    streamId: '/demo/video1.mp4',
  },
]

export const mockLocalAdapter: MediaSourceAdapter = {
  sourceId: 'local',
  displayName: 'Música local',

  async search(query: string) {
    const q = query.toLowerCase()
    const all = [...DEMO_AUDIO, ...DEMO_VIDEO]
    const items = all.filter(
      (item) =>
        item.title.toLowerCase().includes(q) ||
        (item.artist?.toLowerCase().includes(q) ?? false)
    )
    return { items, artists: [] }
  },

  async getStreamUrl(id: string): Promise<string> {
    const item = [...DEMO_AUDIO, ...DEMO_VIDEO].find((i) => i.id === id)
    if (!item?.streamId) throw new Error(`Media no encontrada: ${id}`)
    return item.streamId
  },

  validateId(id: string): boolean {
    return id.startsWith('local-')
  },
}
