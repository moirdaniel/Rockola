import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { useErrorStore } from '@/stores/errorStore'
import { ErrorModal } from './ErrorModal'

describe('ErrorModal', () => {
  beforeEach(() => {
    useErrorStore.setState({ error: null })
  })

  it('returns null when there is no error', () => {
    const { container } = render(<ErrorModal />)
    expect(container.firstChild).toBeNull()
  })

  it('shows title and message when error is set', () => {
    useErrorStore.setState({
      error: { title: 'Error de prueba', message: 'Mensaje aquí' },
    })
    render(<ErrorModal />)
    expect(screen.getByText('Error de prueba')).toBeInTheDocument()
    expect(screen.getByText('Mensaje aquí')).toBeInTheDocument()
  })

  it('calls clear when close button is clicked', async () => {
    useErrorStore.setState({
      error: { title: 'T', message: 'M' },
    })
    render(<ErrorModal />)
    fireEvent.click(screen.getByLabelText('Cerrar error'))
    expect(useErrorStore.getState().error).toBeNull()
  })
})
