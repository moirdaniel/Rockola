import { describe, it, expect, vi, beforeEach } from 'vitest'
import { getBaseUrl, ApiError, api } from './api'

describe('getBaseUrl', () => {
  it('returns empty string when VITE_API_BASE_URL is not set', () => {
    expect(getBaseUrl()).toBe('')
  })
})

describe('ApiError', () => {
  it('sets name and message', () => {
    const err = new ApiError('test', 404)
    expect(err.name).toBe('ApiError')
    expect(err.message).toBe('test')
    expect(err.status).toBe(404)
  })

  it('accepts optional body', () => {
    const err = new ApiError('error', 500, { code: 'E001' })
    expect(err.body).toEqual({ code: 'E001' })
  })
})

describe('api', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn())
  })

  it('get builds GET request with JSON headers', async () => {
    const mockFetch = vi.mocked(fetch)
    mockFetch.mockResolvedValueOnce({
      ok: true,
      text: () => Promise.resolve('{"a":1}'),
    } as Response)
    const result = await api.get<{ a: number }>('/test')
    expect(result).toEqual({ a: 1 })
    expect(mockFetch).toHaveBeenCalledWith(
      '/test',
      expect.objectContaining({
        method: 'GET',
        headers: expect.objectContaining({ 'Content-Type': 'application/json' }),
      })
    )
  })

  it('throws ApiError when response is not ok', async () => {
    const mockFetch = vi.mocked(fetch)
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 402,
      statusText: 'Payment Required',
      text: () => Promise.resolve('Créditos insuficientes'),
    } as Response)
    await expect(api.get('/test')).rejects.toMatchObject({
      message: 'Créditos insuficientes',
      status: 402,
    })
  })
})
