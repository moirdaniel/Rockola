/**
 * Store para sesión del panel de mantenedor (admin).
 * El token se persiste en sessionStorage para sobrevivir recargas.
 */

import { create } from 'zustand'

const STORAGE_KEY = 'rockola_admin_token'

interface AdminStore {
  token: string | null
  setToken: (token: string | null) => void
  logout: () => void
  getAuthHeaders: () => Record<string, string> | undefined
}

function loadStoredToken(): string | null {
  try {
    return sessionStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}

export const useAdminStore = create<AdminStore>((set, get) => ({
  token: loadStoredToken(),

  setToken: (token) => {
    set({ token })
    try {
      if (token) sessionStorage.setItem(STORAGE_KEY, token)
      else sessionStorage.removeItem(STORAGE_KEY)
    } catch {
      // ignore
    }
  },

  logout: () => {
    set({ token: null })
    try {
      sessionStorage.removeItem(STORAGE_KEY)
    } catch {
      // ignore
    }
  },

  getAuthHeaders: () => {
    const token = get().token
    if (!token) return undefined
    return { Authorization: `Bearer ${token}` }
  },
}))
