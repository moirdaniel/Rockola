import { useQuery } from '@tanstack/react-query'
import { creditsApi } from '@/services/queueApi'
import { useCreditsStore } from '@/stores/creditsStore'

export function Credits() {
  const { setCredits, setCostPerSong, costPerSong } = useCreditsStore()
  const { data: credits, isLoading } = useQuery({
    queryKey: ['credits'],
    queryFn: async () => {
      const c = await creditsApi.get()
      setCredits(c)
      return c
    },
  })

  if (isLoading || !credits) {
    return (
      <div className="p-6 max-w-lg mx-auto">
        <p className="text-gray-500">Cargando créditos...</p>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="font-display text-2xl font-bold mb-6">Créditos</h1>
      <div className="glass-panel p-6 rounded-xl">
        <div className="flex items-center justify-between mb-4">
          <span className="text-gray-400">Saldo actual</span>
          <span className="text-3xl font-bold text-jukebox-primary">{credits.balance}</span>
        </div>
        <p className="text-sm text-gray-500">
          Costo por canción: <strong>{costPerSong}</strong> créditos (configuración del servidor).
        </p>
        <p className="text-xs text-gray-600 mt-4">
          Última actualización: {new Date(credits.updatedAt).toLocaleString()}
        </p>
      </div>
    </div>
  )
}
