import { beforeEach, describe, expect, it, vi } from 'vitest'
import { checkLocalStorage } from './health-service'
import { verifyDatabase } from '../repositories/health-repository'

vi.mock('../repositories/health-repository', () => ({ verifyDatabase: vi.fn() }))

describe('local storage check', () => {
  beforeEach(() => vi.clearAllMocks())

  it('accepts the expected persisted value', async () => {
    vi.mocked(verifyDatabase).mockResolvedValue('SQLite listo')
    await expect(checkLocalStorage()).resolves.toBeUndefined()
  })

  it('rejects a different value instead of reporting success', async () => {
    vi.mocked(verifyDatabase).mockResolvedValue('unexpected value')
    await expect(checkLocalStorage()).rejects.toThrow('La lectura no coincide')
  })

  it('preserves a persistence failure', async () => {
    vi.mocked(verifyDatabase).mockRejectedValue(new Error('Disk full'))
    await expect(checkLocalStorage()).rejects.toThrow('Disk full')
  })
})
