/**
 * Modelo de créditos del usuario.
 */

export interface UserCredits {
  id: string
  balance: number
  updatedAt: string
}

export interface CreditsConfig {
  costPerSong: number
}
