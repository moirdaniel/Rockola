import { describe, it, expect, beforeEach } from 'vitest'
import { useErrorStore } from './errorStore'

describe('errorStore', () => {
  beforeEach(() => {
    useErrorStore.setState({ error: null })
  })

  it('starts with null error', () => {
    expect(useErrorStore.getState().error).toBeNull()
  })

  it('showError sets error', () => {
    useErrorStore.getState().showError({
      title: 'Error',
      message: 'Algo falló',
      details: 'detalle',
    })
    expect(useErrorStore.getState().error).toEqual({
      title: 'Error',
      message: 'Algo falló',
      details: 'detalle',
    })
  })

  it('clear resets error', () => {
    useErrorStore.getState().showError({ title: 'X', message: 'Y' })
    useErrorStore.getState().clear()
    expect(useErrorStore.getState().error).toBeNull()
  })
})
