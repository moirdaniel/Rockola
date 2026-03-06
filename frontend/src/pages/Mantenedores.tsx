import { useQuery } from '@tanstack/react-query'
import { adminApi } from '@/services/queueApi'
import { useErrorStore } from '@/stores/errorStore'
import { useAdminStore } from '@/stores/adminStore'
import { ApiError } from '@/services/api'
import { useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { useState, useEffect } from 'react'
import { UpdatesSection } from '@/components/UpdatesSection'
import { UpdateSettingsSection, type UpdateSettings } from '@/components/UpdateSettingsSection'

function RetryButton({
  id,
  token,
  onDone,
}: {
  id: string
  token: string
  onDone: () => void
}) {
  const [loading, setLoading] = useState(false)
  const handleRetry = async () => {
    try {
      setLoading(true)
      await adminApi.retryDownload(id, token)
      onDone()
    } finally {
      setLoading(false)
    }
  }
  return (
    <button
      type="button"
      onClick={handleRetry}
      disabled={loading}
      className="px-2 py-1 rounded bg-amber-600/80 hover:bg-amber-600 text-white text-xs font-medium disabled:opacity-60"
    >
      {loading ? '…' : 'Reintentar'}
    </button>
  )
}

export function Mantenedores() {
  const queryClient = useQueryClient()
  const { showError } = useErrorStore()
  const token = useAdminStore((s) => s.token)
  const setToken = useAdminStore((s) => s.setToken)
  const logout = useAdminStore((s) => s.logout)
  const [isResetting, setIsResetting] = useState(false)
  const [pin, setPin] = useState('')
  const [isLoggingIn, setIsLoggingIn] = useState(false)
  const [loginError, setLoginError] = useState<string | null>(null)
  const [updateSettings, setUpdateSettings] = useState<UpdateSettings | null>(null)

  const {
    data,
    isLoading,
    error,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ['admin', 'maintenance', token ?? 'none'],
    queryFn: () => adminApi.getMaintenance(token ?? undefined),
    refetchOnWindowFocus: false,
    retry: (_, err) => {
      if (err instanceof ApiError && err.status === 401) return false
      return true
    },
  })

  const { data: downloadsData, refetch: refetchDownloads } = useQuery({
    queryKey: ['admin', 'downloads', token ?? ''],
    queryFn: () => adminApi.getDownloads(token!),
    enabled: !!token,
    refetchOnWindowFocus: false,
  })

  const { data: auditData } = useQuery({
    queryKey: ['admin', 'audit', token ?? ''],
    queryFn: () => adminApi.getAuditLog(token!),
    enabled: !!token,
    refetchOnWindowFocus: false,
  })

  const handleResetData = async () => {
    const confirmReset = window.confirm(
      'Esto borrará la cola, la biblioteca cacheada y reseteará los créditos a 1000. ¿Seguro?'
    )
    if (!confirmReset) return
    try {
      setIsResetting(true)
      await adminApi.resetData(token ?? undefined)
      await queryClient.invalidateQueries({ queryKey: ['admin', 'maintenance'] })
      await queryClient.invalidateQueries({ queryKey: ['queue'] })
      await queryClient.invalidateQueries({ queryKey: ['credits'] })
      refetch()
    } catch (e) {
      if (e instanceof ApiError && e.status === 401) {
        logout()
        setLoginError('Sesión expirada. Vuelve a iniciar sesión.')
        return
      }
      if (e instanceof ApiError) {
        showError({
          title: 'Error al resetear',
          message: e.message,
          details: e.body && typeof e.body === 'object' ? JSON.stringify(e.body, null, 2) : `HTTP ${e.status}`,
        })
      } else {
        showError({
          title: 'Error al resetear',
          message: 'No se pudo borrar la data',
          details: e instanceof Error ? e.message : String(e),
        })
      }
    } finally {
      setIsResetting(false)
    }
  }

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setLoginError(null)
    if (!pin.trim()) return
    try {
      setIsLoggingIn(true)
      const res = await adminApi.login(pin.trim())
      setToken(res.token)
      setPin('')
      await queryClient.invalidateQueries({ queryKey: ['admin'] })
      refetch()
    } catch (e) {
      if (e instanceof ApiError) {
        if (e.status === 401) setLoginError('PIN incorrecto')
        else if (e.status === 429) setLoginError('Demasiados intentos. Espera unos minutos.')
        else setLoginError(e.message)
      } else {
        setLoginError('Error de conexión')
      }
    } finally {
      setIsLoggingIn(false)
    }
  }

  const needsLogin = error instanceof ApiError && error.status === 401

  useEffect(() => {
    if (needsLogin) logout()
  }, [needsLogin, logout])

  if (needsLogin) {
    return (
      <div className="p-6 max-w-md mx-auto">
        <div className="glass-panel p-6 rounded-xl">
          <h1 className="font-display text-2xl font-bold mb-4">Acceso mantenedores</h1>
          <p className="text-gray-400 mb-4">
            Introduce el PIN de administrador para ver la cola, descargas y ajustes.
          </p>
          <form onSubmit={handleLogin} className="space-y-4">
            <div>
              <label htmlFor="pin" className="block text-sm font-medium text-gray-400 mb-1">
                PIN
              </label>
              <input
                id="pin"
                type="password"
                inputMode="numeric"
                autoComplete="off"
                value={pin}
                onChange={(e) => setPin(e.target.value)}
                className="w-full px-4 py-2 rounded-lg bg-white/10 border border-white/20 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-jukebox-primary"
                placeholder="••••"
                disabled={isLoggingIn}
              />
            </div>
            {loginError && (
              <p className="text-sm text-red-400">{loginError}</p>
            )}
            <button
              type="submit"
              disabled={isLoggingIn || !pin.trim()}
              className="w-full px-4 py-2 rounded-lg bg-jukebox-primary hover:bg-jukebox-primary/80 font-medium disabled:opacity-60"
            >
              {isLoggingIn ? 'Entrando…' : 'Entrar'}
            </button>
          </form>
          <p className="mt-4 text-sm text-gray-500">
            <Link to="/admin" className="text-jukebox-primary hover:underline">
              ← Volver a Admin
            </Link>
          </p>
        </div>
      </div>
    )
  }

  if (error) {
    const is404 = error instanceof ApiError && error.status === 404
    const msg = error instanceof ApiError ? error.message : (error as Error).message
    return (
      <div className="p-6 max-w-4xl mx-auto">
        <div className="glass-panel p-6 rounded-xl border border-red-500/30">
          <h1 className="font-display text-2xl font-bold text-red-400 mb-2">Error al cargar datos</h1>
          <p className="text-gray-300 mb-4">{msg}</p>
          {is404 ? (
            <>
              <p className="text-sm text-gray-400 mb-2">
                La ruta <code className="bg-white/10 px-1 rounded">GET /api/maintenance</code> no existe en el backend que está corriendo.
              </p>
              <p className="text-sm text-gray-500 mb-4">
                Recompila y reinicia el backend: en la carpeta <code className="bg-white/10 px-1 rounded">backend</code> ejecuta <code className="bg-white/10 px-1 rounded">./run.sh</code> (o <code className="bg-white/10 px-1 rounded">cargo run</code>). Comprueba que el backend escucha en el puerto 3000 y que el proxy de Vite apunta a <code className="bg-white/10 px-1 rounded">http://localhost:3000</code>.
              </p>
              <p className="text-sm text-amber-200/90 mb-4">
                Si ya había un proceso en el puerto 3000, ese es el que responde (y no tiene esta ruta). Para el otro proceso (Ctrl+C en su terminal o <code className="bg-white/10 px-1 rounded">pkill -f rockola-backend</code>) y vuelve a ejecutar <code className="bg-white/10 px-1 rounded">./run.sh</code> desde <code className="bg-white/10 px-1 rounded">backend</code>.
              </p>
            </>
          ) : (
            <p className="text-sm text-gray-500 mb-4">
              Revisa que el backend esté corriendo y que la ruta GET /api/maintenance exista.
            </p>
          )}
          <button
            type="button"
            onClick={() => refetch()}
            className="px-4 py-2 rounded-lg bg-jukebox-primary hover:bg-jukebox-primary/80 font-medium"
          >
            Reintentar
          </button>
        </div>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-6xl mx-auto">
      <div className="flex flex-wrap items-center justify-between gap-4 mb-6">
        <h1 className="font-display text-2xl font-bold">Vista mantenedores</h1>
        <div className="flex items-center gap-2">
          <Link
            to="/admin"
            className="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-sm font-medium transition-colors"
          >
            ← Admin
          </Link>
          {token && (
            <button
              type="button"
              onClick={() => {
                logout()
                queryClient.invalidateQueries({ queryKey: ['admin'] })
              }}
              className="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 text-sm font-medium"
            >
              Cerrar sesión
            </button>
          )}
          <button
            type="button"
            onClick={() => refetch()}
            disabled={isFetching}
            className="px-4 py-2 rounded-lg bg-jukebox-secondary hover:bg-jukebox-secondary/80 font-medium disabled:opacity-60"
          >
            {isFetching ? 'Actualizando…' : 'Actualizar datos'}
          </button>
          <button
            type="button"
            onClick={handleResetData}
            disabled={isResetting}
            className="px-4 py-2 rounded-lg bg-red-600/80 hover:bg-red-600 text-white font-medium disabled:opacity-60"
          >
            {isResetting ? 'Reseteando…' : 'Borrar data y resetear créditos'}
          </button>
        </div>
      </div>

      {isLoading ? (
        <p className="text-gray-500 py-8">Cargando datos del sistema…</p>
      ) : data ? (
        <div className="space-y-8">
          {/* Stats */}
          <section className="glass-panel p-6 rounded-xl">
            <h2 className="font-semibold text-lg mb-4">Resumen</h2>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div className="p-4 rounded-lg bg-white/5">
                <p className="text-sm text-gray-400">Cola (pendientes)</p>
                <p className="text-2xl font-bold text-jukebox-primary">{data.stats.queueCount}</p>
              </div>
              <div className="p-4 rounded-lg bg-white/5">
                <p className="text-sm text-gray-400">Biblioteca (media_library)</p>
                <p className="text-2xl font-bold">{data.stats.mediaLibraryCount}</p>
              </div>
              <div className="p-4 rounded-lg bg-white/5">
                <p className="text-sm text-gray-400">Caché (media_cache)</p>
                <p className="text-2xl font-bold">{data.stats.mediaCacheCount}</p>
              </div>
            </div>
            <div className="mt-4 p-3 rounded-lg bg-white/5 text-sm">
              <span className="text-gray-400">Créditos: </span>
              <strong>{data.credits.balance}</strong>
              <span className="text-gray-500 ml-2">(actualizado: {data.credits.updatedAt})</span>
            </div>
          </section>

          {/* Cola */}
          <section className="glass-panel p-6 rounded-xl">
            <h2 className="font-semibold text-lg mb-4">Cola de reproducción (pendientes)</h2>
            {data.queue.length === 0 ? (
              <p className="text-gray-500">La cola está vacía.</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead>
                    <tr className="border-b border-white/10">
                      <th className="py-2 pr-4">#</th>
                      <th className="py-2 pr-4">queueId</th>
                      <th className="py-2 pr-4">id (media_id)</th>
                      <th className="py-2 pr-4">source</th>
                      <th className="py-2 pr-4">title</th>
                      <th className="py-2 pr-4">type</th>
                      <th className="py-2 pr-4">streamId</th>
                      <th className="py-2 pr-4">addedAt</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.queue.map((item, idx) => (
                      <tr key={item.queueId} className="border-b border-white/5">
                        <td className="py-2 pr-4 text-gray-500">{idx + 1}</td>
                        <td className="py-2 pr-4 font-mono text-xs">{item.queueId.slice(0, 8)}…</td>
                        <td className="py-2 pr-4 font-mono text-xs" title={item.id}>{item.id.slice(0, 20)}…</td>
                        <td className="py-2 pr-4">{item.source}</td>
                        <td className="py-2 pr-4 max-w-[200px] truncate" title={item.title}>{item.title}</td>
                        <td className="py-2 pr-4">{item.type}</td>
                        <td className="py-2 pr-4 max-w-[180px] truncate font-mono text-xs" title={item.streamId ?? ''}>
                          {item.streamId ? (item.streamId.length > 30 ? item.streamId.slice(0, 30) + '…' : item.streamId) : '—'}
                        </td>
                        <td className="py-2 pr-4 text-gray-500">{item.addedAt}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          {/* Media library (reciente) */}
          <section className="glass-panel p-6 rounded-xl">
            <h2 className="font-semibold text-lg mb-4">Biblioteca (últimas 50)</h2>
            {data.mediaLibrary.length === 0 ? (
              <p className="text-gray-500">No hay entradas en media_library.</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead>
                    <tr className="border-b border-white/10">
                      <th className="py-2 pr-4">id</th>
                      <th className="py-2 pr-4">title</th>
                      <th className="py-2 pr-4">artist</th>
                      <th className="py-2 pr-4">source</th>
                      <th className="py-2 pr-4">mediaType</th>
                      <th className="py-2 pr-4">externalId</th>
                      <th className="py-2 pr-4">localPath</th>
                    </tr>
                  </thead>
                  <tbody>
                    {data.mediaLibrary.map((row) => (
                      <tr key={row.id} className="border-b border-white/5">
                        <td className="py-2 pr-4 font-mono text-xs">{row.id.slice(0, 8)}…</td>
                        <td className="py-2 pr-4 max-w-[180px] truncate" title={row.title}>{row.title}</td>
                        <td className="py-2 pr-4 max-w-[120px] truncate text-gray-400">{row.artist ?? '—'}</td>
                        <td className="py-2 pr-4">{row.source}</td>
                        <td className="py-2 pr-4">{row.mediaType}</td>
                        <td className="py-2 pr-4 font-mono text-xs">{row.externalId ?? '—'}</td>
                        <td className="py-2 pr-4 max-w-[220px] truncate font-mono text-xs text-gray-500" title={row.localPath ?? ''}>
                          {row.localPath ?? '—'}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          {/* Descargas */}
          {token && (
            <section className="glass-panel p-6 rounded-xl">
              <h2 className="font-semibold text-lg mb-4">Cola de descargas</h2>
              {!downloadsData ? (
                <p className="text-gray-500">Cargando…</p>
              ) : downloadsData.length === 0 ? (
                <p className="text-gray-500">No hay jobs de descarga.</p>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full text-left text-sm">
                    <thead>
                      <tr className="border-b border-white/10">
                        <th className="py-2 pr-4">youtubeVideoId</th>
                        <th className="py-2 pr-4">tipo</th>
                        <th className="py-2 pr-4">estado</th>
                        <th className="py-2 pr-4">progreso</th>
                        <th className="py-2 pr-4">error</th>
                        <th className="py-2 pr-4">creado</th>
                        <th className="py-2 pr-4"></th>
                      </tr>
                    </thead>
                    <tbody>
                      {downloadsData.map((job) => (
                        <tr key={job.id} className="border-b border-white/5">
                          <td className="py-2 pr-4 font-mono text-xs">{job.youtubeVideoId}</td>
                          <td className="py-2 pr-4">{job.requestedMediaType}</td>
                          <td className="py-2 pr-4">{job.status}</td>
                          <td className="py-2 pr-4">{job.progress != null ? `${Math.round(job.progress * 100)}%` : '—'}</td>
                          <td className="py-2 pr-4 max-w-[200px] truncate text-red-400 text-xs" title={job.errorMessage ?? ''}>
                            {job.errorMessage ?? '—'}
                          </td>
                          <td className="py-2 pr-4 text-gray-500">{job.createdAt}</td>
                          <td className="py-2 pr-4">
                            {job.status === 'failed' && (
                              <RetryButton
                                id={job.id}
                                token={token}
                                onDone={() => { refetchDownloads(); refetch() }}
                              />
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </section>
          )}

          {/* Configuración de actualizaciones (solo en app Tauri, usa backend) */}
          {token && (
            <UpdateSettingsSection
              token={token}
              onSettingsChange={setUpdateSettings}
            />
          )}

          {/* Actualizaciones (solo en app Tauri) */}
          {token && <UpdatesSection token={token} updateSettings={updateSettings} />}

          {/* Audit log */}
          {token && (
            <section className="glass-panel p-6 rounded-xl">
              <h2 className="font-semibold text-lg mb-4">Registro de auditoría</h2>
              {!auditData ? (
                <p className="text-gray-500">Cargando…</p>
              ) : auditData.length === 0 ? (
                <p className="text-gray-500">No hay entradas.</p>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full text-left text-sm">
                    <thead>
                      <tr className="border-b border-white/10">
                        <th className="py-2 pr-4">fecha</th>
                        <th className="py-2 pr-4">acción</th>
                        <th className="py-2 pr-4">entidad</th>
                        <th className="py-2 pr-4">id</th>
                      </tr>
                    </thead>
                    <tbody>
                      {auditData.map((entry) => (
                        <tr key={entry.id} className="border-b border-white/5">
                          <td className="py-2 pr-4 text-gray-500">{entry.createdAt}</td>
                          <td className="py-2 pr-4">{entry.action}</td>
                          <td className="py-2 pr-4">{entry.entityType ?? '—'}</td>
                          <td className="py-2 pr-4 font-mono text-xs">{entry.entityId ?? '—'}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </section>
          )}
        </div>
      ) : null}
    </div>
  )
}
