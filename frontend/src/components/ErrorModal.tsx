import { useErrorStore } from '@/stores/errorStore'

export function ErrorModal() {
  const { error, clear } = useErrorStore()

  if (!error) return null

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="glass-panel max-w-md w-full mx-4 p-6 rounded-2xl border border-red-500/40 shadow-2xl shadow-red-900/40 animate-slide-up">
        <div className="flex items-start justify-between mb-4">
          <h2 className="font-display text-lg font-semibold text-red-400">
            {error.title || 'Error'}
          </h2>
          <button
            type="button"
            onClick={clear}
            className="ml-4 text-gray-400 hover:text-white text-xl leading-none"
            aria-label="Cerrar error"
          >
            ×
          </button>
        </div>
        <p className="text-sm text-gray-200 mb-3">{error.message}</p>
        {error.details && (
          <pre className="mt-2 max-h-40 overflow-auto rounded-lg bg-black/40 text-xs text-gray-300 p-3 whitespace-pre-wrap border border-white/10">
            {error.details}
          </pre>
        )}
      </div>
    </div>
  )
}
