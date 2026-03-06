import { useState } from 'react'
import { Link } from 'react-router-dom'
import { creditsApi, adminApi } from '@/services/queueApi'
import { useCreditsStore } from '@/stores/creditsStore'
import { useErrorStore } from '@/stores/errorStore'
import { useAdminStore } from '@/stores/adminStore'
import { ApiError } from '@/services/api'
import { useQueryClient } from '@tanstack/react-query'

export function Admin() {
  const [amount, setAmount] = useState(500)
  const { setCredits } = useCreditsStore()
  const { showError } = useErrorStore()
  const token = useAdminStore((s) => s.token)
  const queryClient = useQueryClient()
  const [isResetting, setIsResetting] = useState(false)

  const handleAddCredits = async () => {
    try {
      const updated = await creditsApi.add(amount)
      setCredits(updated)
      queryClient.invalidateQueries({ queryKey: ['credits'] })
    } catch (e) {
      if (e instanceof ApiError) {
        showError({
          title: 'Error al agregar créditos',
          message: e.message,
          details:             e.body && typeof e.body === 'object' ? JSON.stringify(e.body, null, 2) :                         `HTTP ${e.status}`,
        })
      } else {
        showError({
          title: 'Error al agregar créditos',
          message: 'No se pudo actualizar el saldo',
          details: e instanceof Error ? e.message : String(e),
        })
      }
    }
  }

  const handleResetData = async () => {
    const confirmReset = window.confirm(
      'Esto borrará la cola, la biblioteca cacheada y reseteará los créditos a 1000. ¿Seguro que quieres continuar?'
    )
    if (!confirmReset) return
    try {
      setIsResetting(true)
      await adminApi.resetData(token ?? undefined)
      // Limpiar caches en React Query
      await queryClient.invalidateQueries({ queryKey: ['queue'] })
      await queryClient.invalidateQueries({ queryKey: ['credits'] })
      // Forzar recarga en stores
      useCreditsStore.getState().setCredits(null)
      // El Layout volverá a cargar créditos/cola desde backend ya limpios
    } catch (e) {
      if (e instanceof ApiError) {
        if (e.status === 401) {
          showError({
            title: 'No autorizado',
            message: 'El backend exige PIN de administrador. Inicia sesión en Vista mantenedores y vuelve a intentar.',
            details: '',
          })
          return
        }
        showError({
          title: 'Error al resetear datos',
          message: e.message,
          details:
            e.body && typeof e.body === 'object'
              ? JSON.stringify(e.body, null, 2)
              : `HTTP ${e.status}`,
        })
      } else {
        showError({
          title: 'Error al resetear datos',
          message: 'No se pudo borrar la data',
          details: e instanceof Error ? e.message : String(e),
        })
      }
    } finally {
      setIsResetting(false)
    }
  }

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="font-display text-2xl font-bold mb-6">Admin básico</h1>
      <div className="glass-panel p-6 rounded-xl space-y-6">
        <p className="text-sm text-gray-400">
          <Link to="/mantenedores" className="text-jukebox-primary hover:underline">
            Vista mantenedores
          </Link>
          {' — ver cola, biblioteca, créditos y diagnosticar problemas.'}
        </p>
        <h2 className="font-semibold">Agregar créditos</h2>
        <div className="flex gap-2">
          <input
            type="number"
            min={1}
            value={amount}
            onChange={(e) => setAmount(Number(e.target.value))}
            className="flex-1 px-4 py-2 rounded-lg glass-panel border border-white/10"
          />
          <button
            type="button"
            onClick={handleAddCredits}
            className="px-4 py-2 rounded-lg bg-jukebox-secondary hover:bg-jukebox-secondary/80 font-medium"
          >
            Agregar
          </button>
        </div>
        <p className="text-sm text-gray-500">
          El costo por canción se configura en el backend (COST_PER_SONG).
        </p>

        <div className="border-t border-white/10 pt-4 space-y-3">
          <h2 className="font-semibold text-red-300">Resetear datos</h2>
          <p className="text-sm text-gray-400">
            Borra la cola de reproducción, la caché de medios y resetea los créditos al valor
            inicial. Úsalo solo si la data quedó en un estado raro o corrupto.
          </p>
          <button
            type="button"
            onClick={handleResetData}
            disabled={isResetting}
            className="px-4 py-2 rounded-lg bg-red-600/80 hover:bg-red-600 text-white font-medium disabled:opacity-60"
          >
            {isResetting ? 'Reseteando...' : 'Borrar data y resetear créditos'}
          </button>
        </div>
      </div>
    </div>
  )
}
