/**
 * Cliente HTTP para la API del backend.
 */

export class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public body?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

async function request<T>(
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const base = getBaseUrl()
  const url = path.startsWith('http') ? path : `${base}${path}`
  const mergedHeaders: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (options.headers && typeof options.headers === 'object' && !Array.isArray(options.headers)) {
    Object.assign(mergedHeaders, options.headers as Record<string, string>)
  }
  const res = await fetch(url, {
    ...options,
    headers: mergedHeaders,
  })
  const text = await res.text()
  let data: T | undefined
  try {
    data = text ? (JSON.parse(text) as T) : undefined
  } catch {
    // no json
  }
  if (!res.ok) {
    const message =
      (data && typeof data === 'object' && 'message' in data && typeof (data as { message: unknown }).message === 'string')
        ? (data as { message: string }).message
        : (text && text.trim().length > 0 ? text : res.statusText)
    throw new ApiError(message, res.status, data)
  }
  return data as T
}

export const api = {
  get: <T>(path: string, init?: RequestInit) =>
    request<T>(path, { method: 'GET', ...init }),
  post: <T>(path: string, body?: unknown, init?: RequestInit) =>
    request<T>(path, {
      method: 'POST',
      body: body != null ? JSON.stringify(body) : undefined,
      ...init,
    }),
  put: <T>(path: string, body?: unknown, init?: RequestInit) =>
    request<T>(path, {
      method: 'PUT',
      body: body != null ? JSON.stringify(body) : undefined,
      ...init,
    }),
  delete: <T>(path: string, init?: RequestInit) =>
    request<T>(path, { method: 'DELETE', ...init }),
}

/** URL base del API (para construir URLs de stream, etc.). */
export function getBaseUrl(): string {
  const env = import.meta.env.VITE_API_BASE_URL
  if (env) return env
  return ''
}
