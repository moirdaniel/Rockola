import { useState } from 'react'

let tvWindowRef: Window | null = null

async function openTvWindow() {
  if (typeof window === 'undefined') return

  // Detectar entorno Tauri (escritorio) si está disponible
  const isTauri = typeof (window as any).__TAURI__ !== 'undefined'

  if (isTauri) {
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/window')
      const existing = (WebviewWindow as any).getByLabel
        ? (WebviewWindow as any).getByLabel('rockola-tv')
        : null
      if (existing) {
        await existing.show()
        await existing.setFocus()
        await existing.setFullscreen(true)
      } else {
        const win = new WebviewWindow('rockola-tv', {
          url: '/tv',
          fullscreen: true,
        })
        await win.setFocus()
      }
      return
    } catch {
      // Fallback al comportamiento web normal
    }
  }

  const url = `${window.location.origin}/tv`
  tvWindowRef = window.open(url, 'rockola-tv', 'noopener,noreferrer')
}

async function closeTvWindow() {
  if (typeof window === 'undefined') return

  const isTauri = typeof (window as any).__TAURI__ !== 'undefined'
  if (isTauri) {
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/window')
      const existing = (WebviewWindow as any).getByLabel
        ? (WebviewWindow as any).getByLabel('rockola-tv')
        : null
      if (existing) {
        await existing.close()
        return
      }
    } catch {
      // ignorar y seguir con el fallback web
    }
  }

  if (tvWindowRef && !tvWindowRef.closed) {
    tvWindowRef.close()
    tvWindowRef = null
  } else {
    const handle = window.open('', 'rockola-tv')
    if (handle && !handle.closed) {
      handle.close()
    }
  }
}

export function DisplayConfig() {
  const [status, setStatus] = useState<string | null>(null)

  const handleOpen = async () => {
    setStatus(null)
    await openTvWindow()
    setStatus('Ventana de TV abierta. Mueve esta ventana al monitor de público y ponla a pantalla completa.')
  }

  const handleClose = async () => {
    await closeTvWindow()
    setStatus('Ventana de TV cerrada.')
  }

  return (
    <div className="p-6 max-w-4xl mx-auto space-y-6">
      <section>
        <h1 className="font-display text-2xl font-bold mb-2">Configuración de pantallas</h1>
        <p className="text-gray-400">
          Usa esta vista para separar la pantalla de <strong>control</strong> (búsqueda y cola) de la
          pantalla de <strong>TV</strong> (solo el video a pantalla completa).
        </p>
      </section>

      <section className="glass-panel rounded-2xl p-4 space-y-3">
        <h2 className="font-semibold">Pantalla de TV</h2>
        <p className="text-sm text-gray-400">
          La pantalla de TV muestra solo el video actual (vista <code>/tv</code>). Lo normal es tener:
        </p>
        <ul className="list-disc list-inside text-sm text-gray-400 space-y-1">
          <li>Este menú de Rockola en el monitor principal (cerca del operador).</li>
          <li>La ventana de TV en el segundo monitor (visible para el público).</li>
        </ul>

        <div className="flex gap-3 mt-3">
          <button
            type="button"
            onClick={handleOpen}
            className="px-4 py-2 rounded-lg bg-jukebox-primary hover:bg-jukebox-primary/90 text-sm font-medium transition-colors"
          >
            Abrir ventana TV
          </button>
          <button
            type="button"
            onClick={handleClose}
            className="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-sm font-medium transition-colors"
          >
            Cerrar ventana TV
          </button>
        </div>

        {status && (
          <p className="mt-2 text-xs text-gray-300">
            {status}
          </p>
        )}

        <p className="mt-3 text-xs text-gray-500">
          Consejo: arrastra la ventana de TV al monitor del público y ponla a pantalla completa (F11 o
          doble clic en la barra de título, según el sistema). El operador se queda con esta ventana de
          control para buscar y gestionar la cola.
        </p>
      </section>
    </div>
  )
}

