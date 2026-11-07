import { beforeEach, describe, expect, it, vi } from 'vitest'
import { verifyDatabase } from './health-repository'

const execute = vi.fn()
const select = vi.fn()

vi.mock('../database/connection', () => ({
  getDatabase: vi.fn(async () => ({ execute, select })),
}))

describe('health repository', () => {
  beforeEach(() => {
    execute.mockReset()
    select.mockReset()
    select.mockResolvedValue([{ message: 'SQLite listo' }])
  })

  it('writes and reads the SQLite health record', async () => {
    await expect(verifyDatabase()).resolves.toBe('SQLite listo')
    expect(execute).toHaveBeenCalledTimes(2)
    expect(select).toHaveBeenCalledWith('SELECT message FROM app_health WHERE id = 1')
  })
})
