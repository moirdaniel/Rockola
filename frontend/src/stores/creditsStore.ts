import { create } from 'zustand'
import type { UserCredits } from '@/types/credits'

interface CreditsStore {
  credits: UserCredits | null
  costPerSong: number
  isLoading: boolean
  error: string | null
  setCredits: (credits: UserCredits | null) => void
  setCostPerSong: (cost: number) => void
  setLoading: (loading: boolean) => void
  setError: (err: string | null) => void
}

export const useCreditsStore = create<CreditsStore>((set) => ({
  credits: null,
  costPerSong: 100,
  isLoading: false,
  error: null,
  setCredits: (credits) => set({ credits, error: null }),
  setCostPerSong: (costPerSong) => set({ costPerSong }),
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}))
