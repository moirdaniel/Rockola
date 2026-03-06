import { create } from 'zustand'

export interface GlobalError {
  title: string
  message: string
  details?: string
}

interface ErrorStore {
  error: GlobalError | null
  showError: (error: GlobalError) => void
  clear: () => void
}

export const useErrorStore = create<ErrorStore>((set) => ({
  error: null,
  showError: (error) => set({ error }),
  clear: () => set({ error: null }),
}))
