import { create } from 'zustand'
import type { QueueItem } from '@/types/media'

interface QueueStore {
  items: QueueItem[]
  isLoading: boolean
  error: string | null
  setItems: (items: QueueItem[]) => void
  setLoading: (loading: boolean) => void
  setError: (err: string | null) => void
  clear: () => void
}

export const useQueueStore = create<QueueStore>((set) => ({
  items: [],
  isLoading: false,
  error: null,
  setItems: (items) => set({ items, error: null }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
  clear: () => set({ items: [], error: null }),
}))
