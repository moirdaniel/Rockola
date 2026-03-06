import { create } from 'zustand'

type View = 'home' | 'now-playing' | 'queue' | 'credits' | 'admin'

interface UiStore {
  view: View
  sidebarOpen: boolean
  setView: (view: View) => void
  toggleSidebar: () => void
  setSidebarOpen: (open: boolean) => void
}

export const useUiStore = create<UiStore>((set) => ({
  view: 'home',
  sidebarOpen: true,
  setView: (view) => set({ view }),
  toggleSidebar: () => set((s) => ({ sidebarOpen: !s.sidebarOpen })),
  setSidebarOpen: (sidebarOpen) => set({ sidebarOpen }),
}))
