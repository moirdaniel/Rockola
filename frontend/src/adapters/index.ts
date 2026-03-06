/**
 * Registro de adapters y factory para Unified Search.
 */

import type { MediaSourceAdapter } from './MediaSourceAdapter'
import { mockLocalAdapter } from './mockLocalAdapter'
import { mockYoutubeAdapter } from './mockYoutubeAdapter'
import { mockSpotifyAdapter } from './mockSpotifyAdapter'
import { backendAdapter } from './backendAdapter'

export type { MediaSourceAdapter } from './MediaSourceAdapter'
export { mockLocalAdapter, mockYoutubeAdapter, mockSpotifyAdapter, backendAdapter }

// Para entorno real usamos solo el backend (YouTube + caché local).
// Los mocks quedan disponibles para desarrollo si se quisieran reactivar.
const adapters: MediaSourceAdapter[] = [
  backendAdapter,
]

export function getAdapters(): MediaSourceAdapter[] {
  return adapters
}

export function getAdapterBySourceId(sourceId: string): MediaSourceAdapter | undefined {
  const found = adapters.find((a) => a.sourceId === sourceId)
  if (found) return found
  // La cola guarda source "youtube" o "local" (viene del backend); el stream siempre lo sirve el backend
  if (sourceId === 'youtube' || sourceId === 'local') return backendAdapter
  return undefined
}
