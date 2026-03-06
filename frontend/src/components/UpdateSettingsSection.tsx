/**
 * Configuración de actualizaciones (solo admin).
 * Lee/escribe settings en el backend y los usa en la sección de actualizaciones.
 */

import { useState, useEffect } from 'react'
import { adminApi } from '@/services/queueApi'

export type UpdateSettings = {
  enabled: boolean
  channel: string
  autoCheck: boolean
  checkIntervalMinutes: number
  endpointOverride?: string | null
}

export function UpdateSettingsSection({
  token,
  onSettingsChange,
}: {
  token: string
  onSettingsChange?: (s: UpdateSettings) => void
}) {
  const [settings, setSettings] = useState<UpdateSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const load = async () => {
    setError(null)
    try {
      const data = await adminApi.getUpdateSettings(token)
      setSettings({
        enabled: data.enabled,
        channel: data.channel || 'stable',
        autoCheck: data.autoCheck,
        checkIntervalMinutes: data.checkIntervalMinutes ?? 720,
        endpointOverride: data.endpointOverride ?? null,
      })
      onSettingsChange?.({
        enabled: data.enabled,
        channel: data.channel || 'stable',
        autoCheck: data.autoCheck,
        checkIntervalMinutes: data.checkIntervalMinutes ?? 720,
        endpointOverride: data.endpointOverride ?? null,
      })
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    load()
  }, [token])

  const handleSave = async () => {
    if (!settings) return
    setSaving(true)
    setError(null)
    try {
      await adminApi.putUpdateSettings(token, {
        enabled: settings.enabled,
        channel: settings.channel,
        autoCheck: settings.autoCheck,
        checkIntervalMinutes: settings.checkIntervalMinutes,
        endpointOverride: settings.endpointOverride ?? '',
      })
      onSettingsChange?.(settings)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setSaving(false)
    }
  }

  if (loading || !settings) {
    return (
      <section className="glass-panel p-6 rounded-xl">
        <h2 className="font-semibold text-lg mb-4">Configuración de actualizaciones</h2>
        <p className="text-gray-500">Cargando…</p>
      </section>
    )
  }

  return (
    <section className="glass-panel p-6 rounded-xl">
      <h2 className="font-semibold text-lg mb-4">Configuración de actualizaciones</h2>
      {error && <p className="text-sm text-red-400 mb-3">{error}</p>}
      <div className="space-y-4 max-w-md">
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={settings.enabled}
            onChange={(e) => setSettings((s) => (s ? { ...s, enabled: e.target.checked } : s))}
            className="rounded border-white/30 bg-white/10"
          />
          <span className="text-sm">Actualizaciones habilitadas</span>
        </label>
        <div>
          <label className="block text-sm text-gray-400 mb-1">Canal</label>
          <select
            value={settings.channel}
            onChange={(e) =>
              setSettings((s) => (s ? { ...s, channel: e.target.value } : s))
            }
            className="w-full px-3 py-2 rounded-lg bg-white/10 border border-white/20 text-white text-sm"
          >
            <option value="stable">Estable (stable)</option>
            <option value="beta">Beta</option>
          </select>
        </div>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={settings.autoCheck}
            onChange={(e) =>
              setSettings((s) => (s ? { ...s, autoCheck: e.target.checked } : s))
            }
            className="rounded border-white/30 bg-white/10"
          />
          <span className="text-sm">Comprobar actualizaciones en segundo plano</span>
        </label>
        <div>
          <label className="block text-sm text-gray-400 mb-1">
            Intervalo de comprobación (minutos)
          </label>
          <input
            type="number"
            min={1}
            max={10080}
            value={settings.checkIntervalMinutes}
            onChange={(e) =>
              setSettings((s) =>
                s ? { ...s, checkIntervalMinutes: parseInt(e.target.value, 10) || 720 } : s
              )
            }
            className="w-full px-3 py-2 rounded-lg bg-white/10 border border-white/20 text-white text-sm"
          />
          <p className="text-xs text-gray-500 mt-1">Por defecto 720 (12 h). Máx. 10080 (7 días).</p>
        </div>
        <div>
          <label className="block text-sm text-gray-400 mb-1">
            URL del feed de actualizaciones (opcional)
          </label>
          <input
            type="text"
            value={settings.endpointOverride ?? ''}
            onChange={(e) =>
              setSettings((s) => (s ? { ...s, endpointOverride: e.target.value.trim() || null } : s))
            }
            placeholder="https://..."
            className="w-full px-3 py-2 rounded-lg bg-white/10 border border-white/20 text-white text-sm font-mono placeholder-gray-500"
          />
          <p className="text-xs text-gray-500 mt-1">
            Dejar vacío para usar el endpoint por defecto (GitHub Releases).
          </p>
        </div>
        <button
          type="button"
          onClick={handleSave}
          disabled={saving}
          className="px-4 py-2 rounded-lg bg-jukebox-primary hover:bg-jukebox-primary/80 font-medium disabled:opacity-60"
        >
          {saving ? 'Guardando…' : 'Guardar configuración'}
        </button>
      </div>
    </section>
  )
}
