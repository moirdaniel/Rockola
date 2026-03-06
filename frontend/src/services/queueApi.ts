/**
 * Servicio de cola y créditos contra el backend.
 */

import { api } from './api'
import type { MediaItem } from '@/types/media'
import type { QueueItem } from '@/types/media'
import type { UserCredits } from '@/types/credits'

export interface AddToQueuePayload {
  mediaItem: MediaItem
}

export interface QueueResponse {
  queue: QueueItem[]
}

export const queueApi = {
  getQueue: () => api.get<QueueResponse>('/api/queue').then((r) => r.queue),
  add: (payload: AddToQueuePayload) =>
    api.post<QueueResponse>('/api/queue', payload).then((r) => r.queue),
  next: () => api.post<QueueResponse>('/api/queue/next').then((r) => r.queue),
  clear: () => api.delete<QueueResponse>('/api/queue').then((r) => r.queue),
}

export const creditsApi = {
  get: () => api.get<UserCredits>('/api/credits'),
  add: (amount: number) =>
    api.post<UserCredits>('/api/credits/add', { amount }).then((r) => r),
}

export const adminApi = {
  login: (pin: string) =>
    api.post<{ token: string; expiresInSecs: number }>('/api/admin/login', { pin }),

  logout: (token?: string | null) =>
    api.post<{ ok: boolean }>(
      '/api/admin/logout',
      undefined,
      token ? { headers: { Authorization: `Bearer ${token}` } } : undefined
    ),

  getSession: (token: string) =>
    api.get<{ valid: boolean }>('/api/admin/session', {
      headers: { Authorization: `Bearer ${token}` },
    }),

  resetData: (token?: string | null) =>
    api.post<{ ok: boolean }>(
      '/api/admin/reset',
      undefined,
      token ? { headers: { Authorization: `Bearer ${token}` } } : undefined
    ),

  getMaintenance: (token?: string | null) =>
    api.get<{
      queue: Array<{
        queueId: string
        id: string
        source: string
        title: string
        artist?: string
        type: string
        streamId?: string
        order: number
        addedAt: string
        downloadId?: string
        downloadStatus?: string
      }>;
      mediaLibrary: Array<{
        id: string
        title: string
        artist?: string
        source: string
        localPath?: string
        mediaType: string
        externalId?: string
      }>;
      credits: { id: string; balance: number; updatedAt: string };
      stats: { queueCount: number; mediaLibraryCount: number; mediaCacheCount: number };
    }>(
      '/api/maintenance',
      token ? { headers: { Authorization: `Bearer ${token}` } } : undefined
    ),

  getDownloads: (token: string) =>
    api.get<
      Array<{
        id: string
        youtubeVideoId: string
        requestedMediaType: string
        status: string
        progress?: number
        targetPath?: string
        errorMessage?: string
        createdAt: string
        updatedAt: string
      }>
    >('/api/downloads', { headers: { Authorization: `Bearer ${token}` } }),

  retryDownload: (id: string, token: string) =>
    api.post(
      `/api/downloads/${id}/retry`,
      undefined,
      { headers: { Authorization: `Bearer ${token}` } }
    ),

  getAuditLog: (token: string) =>
    api.get<
      Array<{
        id: string
        action: string
        entityType?: string
        entityId?: string
        payloadJson?: string
        createdAt: string
      }>
    >('/api/admin/audit-log', { headers: { Authorization: `Bearer ${token}` } }),

  postAudit: (
    token: string,
    body: { action: string; entityType?: string; entityId?: string; payloadJson?: string }
  ) =>
    api.post<{ ok: boolean }>('/api/admin/audit', body, {
      headers: { Authorization: `Bearer ${token}` },
    }),

  getUpdateSettings: (token: string) =>
    api.get<{
      enabled: boolean
      channel: string
      autoCheck: boolean
      checkIntervalMinutes: number
      endpointOverride?: string | null
    }>('/api/admin/settings/updates', {
      headers: { Authorization: `Bearer ${token}` },
    }),

  putUpdateSettings: (
    token: string,
    body: {
      enabled?: boolean
      channel?: string
      autoCheck?: boolean
      checkIntervalMinutes?: number
      endpointOverride?: string
    }
  ) =>
    api.put<{ ok: boolean }>('/api/admin/settings/updates', body, {
      headers: { Authorization: `Bearer ${token}` },
    }),
}
