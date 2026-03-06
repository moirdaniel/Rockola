import { create } from 'zustand'
import type { MediaItem, ArtistSummary } from '@/types/media'

interface SearchStore {
  query: string
  results: MediaItem[]
  artists: ArtistSummary[]
  isSearching: boolean
  error: string | null
  setQuery: (q: string) => void
  setResults: (results: MediaItem[]) => void
  setArtists: (artists: ArtistSummary[]) => void
  setSearchResponse: (artists: ArtistSummary[], results: MediaItem[]) => void
  setSearching: (v: boolean) => void
  setError: (err: string | null) => void
  clear: () => void
}

const initial = {
  query: '',
  results: [] as MediaItem[],
  artists: [] as ArtistSummary[],
  isSearching: false,
  error: null as string | null,
}

export const useSearchStore = create<SearchStore>((set) => ({
  ...initial,
  setQuery: (query) => set({ query }),
  setResults: (results) => set({ results, error: null }),
  setArtists: (artists) => set({ artists }),
  setSearchResponse: (artists, results) => set({ artists, results, error: null }),
  setSearching: (isSearching) => set({ isSearching }),
  setError: (error) => set({ error }),
  clear: () => set(initial),
}))
