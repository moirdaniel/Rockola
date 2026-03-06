/**
 * Sección de actualizaciones (solo en app Tauri).
 * Muestra versión actual, busca/descarga/instala updates y registra auditoría en el backend.
 * Usa settings del backend (enabled, channel, autoCheck, checkIntervalMinutes, endpointOverride).
 */

import { useState, useEffect, useCallback } from 'react'
import { adminApi } from '@/services/queueApi'
import type { UpdateSettings } from './UpdateSettingsSection'

type UpdateStatus = {
  phase: string
  progress: number
  lastError?: string
  currentVersion: string
  availableVersion?: string
}

type CheckResult = {
  hasUpdate: boolean
  currentVersion: string
  availableVersion?: string
  notes?: string
  pubDate?: string
}

export function UpdatesSection({
  token,
  updateSettings,
}: {
  token: string
  updateSettings?: UpdateSettings | null
}) {
  const [isTauri, setIsTauri] = useState<boolean | null>(null)
  const [status, setStatus] = useState<UpdateStatus | null>(null)
  const [checkResult, setCheckResult] = useState<CheckResult | null>(null)
  const [checking, setChecking] = useState(false)
  const [downloading, setDownloading] = useState(false)
  const [installing, setInstalling] = useState(false)
  const [downloadProgress, setDownloadProgress] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const [recoveryStatus, setRecoveryStatus] = useState<{
    inRecoveryMode: boolean
    restartsAfterUpdate: number
  } | null>(null)

  const postAudit = useCallback(
    async (action: string, payload: Record<string, unknown>) => {
      try {
        await adminApi.postAudit(token, {
          action,
          entityType: 'update',
          payloadJson: JSON.stringify(payload),
        })
      } catch {
        // ignore audit errors
      }
    },
    [token]
  )

  useEffect(() => {
    let cancelled = false
    async function detect() {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const s = await invoke<UpdateStatus>('updater_get_status')
        if (!cancelled) {
          setIsTauri(true)
          setStatus(s)
        }
      } catch {
        if (!cancelled) setIsTauri(false)
      }
    }
    detect()
    return () => { cancelled = true }
  }, [])

  useEffect(() => {
    if (!isTauri) return
    const unlisten = import('@tauri-apps/api/event').then(({ listen }) =>
      listen<{ event: string; progress?: number }>('update', (e) => {
        if (e.payload.event === 'progress' && e.payload.progress != null) {
          setDownloadProgress(Math.round(e.payload.progress * 100))
        }
        if (e.payload.event === 'recovery_mode_activated') {
          setRecoveryStatus((prev) =>
            prev ? { ...prev, inRecoveryMode: true } : { inRecoveryMode: true, restartsAfterUpdate: 0 }
          )
          postAudit('UPDATE_RECOVERY_MODE_ACTIVATED', { reason: 'crash_loop_after_update' })
        }
      })
    )
    return () => {
      unlisten.then((u) => u())
    }
  }, [isTauri, postAudit])

  useEffect(() => {
    if (!isTauri || !token) return
    let cancelled = false
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke<{ inRecoveryMode: boolean; restartsAfterUpdate: number }>('updater_get_recovery_status'))
      .then((s) => {
        if (!cancelled) setRecoveryStatus(s)
      })
      .catch(() => {})
    return () => { cancelled = true }
  }, [isTauri, token])

  const updatesDisabled = updateSettings != null && updateSettings.enabled === false

  useEffect(() => {
    if (!isTauri || !token || !updateSettings?.autoCheck || updatesDisabled) return
    const minutes = Math.max(1, updateSettings.checkIntervalMinutes ?? 720)
    const ms = minutes * 60 * 1000
    const id = setInterval(async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const result = await invoke<CheckResult>('updater_check', {
          channel: updateSettings?.channel ?? 'stable',
          endpointOverride: updateSettings?.endpointOverride?.trim() || null,
        })
        if (result.hasUpdate) {
          setCheckResult(result)
          const s = await invoke<UpdateStatus>('updater_get_status')
          setStatus(s)
        }
      } catch {
        // ignore background check errors
      }
    }, ms)
    return () => clearInterval(id)
  }, [isTauri, token, updateSettings?.autoCheck, updateSettings?.channel, updateSettings?.checkIntervalMinutes, updateSettings?.endpointOverride, updatesDisabled])

  const handleCheck = async () => {
    setError(null)
    setChecking(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      await postAudit('UPDATE_CHECK', {})
      const channel = updateSettings?.channel ?? 'stable'
      const endpointOverride = updateSettings?.endpointOverride?.trim() || undefined
      const result = await invoke<CheckResult>('updater_check', {
        channel: channel || undefined,
        endpointOverride: endpointOverride ?? null,
      })
      setCheckResult(result)
      if (result.hasUpdate) {
        await postAudit('UPDATE_AVAILABLE', {
          version_current: result.currentVersion,
          version_target: result.availableVersion,
          notes: result.notes,
        })
      }
      const s = await invoke<UpdateStatus>('updater_get_status')
      setStatus(s)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      await postAudit('UPDATE_FAILED', { step: 'check', error: String(e) })
    } finally {
      setChecking(false)
    }
  }

  const handleDownload = async () => {
    setError(null)
    setDownloading(true)
    setDownloadProgress(0)
    try {
      await postAudit('UPDATE_DOWNLOAD_STARTED', {
        version_target: checkResult?.availableVersion,
      })
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke<{ status: string }>('updater_download')
      setDownloadProgress(100)
      const s = await invoke<UpdateStatus>('updater_get_status')
      setStatus(s)
      await postAudit('UPDATE_DOWNLOAD_DONE', {
        version_target: checkResult?.availableVersion,
      })
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      await postAudit('UPDATE_FAILED', { step: 'download', error: String(e) })
    } finally {
      setDownloading(false)
    }
  }

  const handleInstall = async () => {
    const confirmInstall = window.confirm(
      'Se instalará la actualización y la app se reiniciará. ¿Continuar?'
    )
    if (!confirmInstall) return
    setError(null)
    setInstalling(true)
    try {
      await postAudit('UPDATE_INSTALL_STARTED', {
        version_current: status?.currentVersion,
        version_target: checkResult?.availableVersion,
      })
      const { invoke } = await import('@tauri-apps/api/core')
      await invoke('updater_install')
      await postAudit('UPDATE_INSTALL_DONE', {
        version_target: checkResult?.availableVersion,
      })
      const { relaunch } = await import('@tauri-apps/plugin-process')
      await relaunch()
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      await postAudit('UPDATE_FAILED', { step: 'install', error: String(e) })
      setInstalling(false)
    }
  }

  if (isTauri === null) {
    return (
      <section className="glass-panel p-6 rounded-xl">
        <h2 className="font-semibold text-lg mb-4">Actualizaciones</h2>
        <p className="text-gray-500">Comprobando…</p>
      </section>
    )
  }

  if (!isTauri) {
    return (
      <section className="glass-panel p-6 rounded-xl">
        <h2 className="font-semibold text-lg mb-4">Actualizaciones</h2>
        <p className="text-gray-500 text-sm">
          Solo disponible en la app de escritorio (Tauri). En el navegador no hay actualizaciones.
        </p>
      </section>
    )
  }

  return (
    <section className="glass-panel p-6 rounded-xl">
      <h2 className="font-semibold text-lg mb-4">Actualizaciones</h2>
      {recoveryStatus?.inRecoveryMode && (
        <div className="mb-4 p-4 rounded-lg bg-amber-900/40 border border-amber-600/50">
          <p className="text-amber-200 font-medium mb-1">Modo recuperación</p>
          <p className="text-sm text-amber-200/90 mb-3">
            La app detectó varios reinicios seguidos tras una actualización. Las actualizaciones automáticas están desactivadas. Solo un administrador puede desactivar el modo recuperación.
          </p>
          <button
            type="button"
            onClick={async () => {
              try {
                const { invoke } = await import('@tauri-apps/api/core')
                await invoke('updater_clear_recovery')
                setRecoveryStatus((s) => (s ? { ...s, inRecoveryMode: false } : s))
                postAudit('UPDATE_RECOVERY_MODE_CLEARED', {})
              } catch {
                // ignore
              }
            }}
            className="px-3 py-1.5 rounded-lg bg-amber-600/80 hover:bg-amber-600 text-white text-sm font-medium"
          >
            Desactivar modo recuperación
          </button>
        </div>
      )}
      {updatesDisabled ? (
        <p className="text-amber-200/90 text-sm">
          Las actualizaciones están desactivadas en la configuración. Actívalas en &quot;Configuración de actualizaciones&quot; para buscar e instalar actualizaciones.
        </p>
      ) : recoveryStatus?.inRecoveryMode ? (
        <p className="text-amber-200/90 text-sm">
          No puedes buscar ni instalar actualizaciones hasta desactivar el modo recuperación con el botón de arriba.
        </p>
      ) : (
        <>
      <p className="text-sm text-gray-400 mb-2">
        Versión actual: <strong className="text-white">{status?.currentVersion ?? '—'}</strong>
      </p>
      <div className="flex flex-wrap items-center gap-2 mb-4">
        <button
          type="button"
          onClick={handleCheck}
          disabled={checking}
          className="px-4 py-2 rounded-lg bg-jukebox-secondary hover:bg-jukebox-secondary/80 font-medium disabled:opacity-60"
        >
          {checking ? 'Buscando…' : 'Buscar actualización'}
        </button>
      </div>
      {error && (
        <p className="text-sm text-red-400 mb-2">{error}</p>
      )}
      {checkResult?.hasUpdate && (
        <div className="rounded-lg bg-white/5 p-4 space-y-3">
          <p className="text-sm">
            Disponible: <strong>{checkResult.availableVersion}</strong>
            {checkResult.pubDate && (
              <span className="text-gray-500 ml-2">({checkResult.pubDate})</span>
            )}
          </p>
          {checkResult.notes && (
            <p className="text-sm text-gray-400 whitespace-pre-wrap">{checkResult.notes}</p>
          )}
          <div className="flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={handleDownload}
              disabled={downloading || installing}
              className="px-4 py-2 rounded-lg bg-amber-600/80 hover:bg-amber-600 text-white font-medium disabled:opacity-60"
            >
              {downloading ? `Descargando ${downloadProgress}%…` : 'Descargar'}
            </button>
            {status?.phase === 'downloaded' && (
              <button
                type="button"
                onClick={handleInstall}
                disabled={installing}
                className="px-4 py-2 rounded-lg bg-green-600/80 hover:bg-green-600 text-white font-medium disabled:opacity-60"
              >
                {installing ? 'Instalando…' : 'Instalar y reiniciar'}
              </button>
            )}
          </div>
          {downloading && (
            <div className="w-full h-2 rounded-full bg-white/10 overflow-hidden">
              <div
                className="h-full bg-jukebox-primary transition-all duration-300"
                style={{ width: `${downloadProgress}%` }}
              />
            </div>
          )}
          <p className="text-xs text-gray-500">
            La app se reiniciará para aplicar la actualización.
          </p>
        </div>
      )}
      {checkResult && !checkResult.hasUpdate && !checking && (
        <p className="text-sm text-gray-500">No hay actualizaciones disponibles.</p>
      )}
        </>
      )}
    </section>
  )
}
