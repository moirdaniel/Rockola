import { create } from 'zustand'
import type { PlaybackState } from '@/types/media'
import type { QueueItem } from '@/types/media'

interface PlayerStore extends PlaybackState {
  setCurrentItem: (item: QueueItem | null) => void
  setPlaying: (playing: boolean) => void
  setCurrentTime: (t: number) => void
  setDuration: (d: number) => void
  setVolume: (v: number) => void
  setFullscreen: (full: boolean) => void
  setError: (err: string | null) => void
  setStreamUrl: (url: string | null) => void
  reset: () => void
}

const initialState: PlaybackState = {
  currentItem: null,
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  volume: 1,
  isFullscreen: false,
  error: null,
  streamUrl: null,
}

export const usePlayerStore = create<PlayerStore>((set) => ({
  ...initialState,
  setCurrentItem: (currentItem) => set({ currentItem, error: null }),
  setPlaying: (isPlaying) => set({ isPlaying }),
  setCurrentTime: (currentTime) => set({ currentTime }),
  setDuration: (duration) => set({ duration }),
  setVolume: (volume) => set({ volume }),
  setFullscreen: (isFullscreen) => set({ isFullscreen }),
  setError: (error) => set({ error }),
  setStreamUrl: (streamUrl) => set({ streamUrl }),
  reset: () => set(initialState),
}))
