import { beforeEach, describe, expect, it, vi } from 'vitest'

const { load, isTauri } = vi.hoisted(() => ({ load: vi.fn(), isTauri: vi.fn() }))
vi.mock('@tauri-apps/plugin-sql', () => ({ default: { load } }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri }))

describe('database connection', () => {
  beforeEach(() => {
    vi.resetModules()
    load.mockReset()
    isTauri.mockReturnValue(true)
  })

  it('does not invoke the native plugin in a browser', async () => {
    isTauri.mockReturnValue(false)
    const { getDatabase } = await import('./connection')
    await expect(getDatabase()).rejects.toThrow('aplicación de escritorio')
    expect(load).not.toHaveBeenCalled()
  })

  it('shares one connection for concurrent callers', async () => {
    const database = { path: 'test' }
    load.mockResolvedValue(database)
    const { getDatabase } = await import('./connection')
    const results = await Promise.all([getDatabase(), getDatabase()])
    expect(results).toEqual([database, database])
    expect(load).toHaveBeenCalledTimes(1)
  })

  it('allows a retry after opening fails', async () => {
    load.mockRejectedValueOnce(new Error('Unavailable')).mockResolvedValueOnce({ path: 'test' })
    const { getDatabase } = await import('./connection')
    await expect(getDatabase()).rejects.toThrow('Unavailable')
    await expect(getDatabase()).resolves.toEqual({ path: 'test' })
    expect(load).toHaveBeenCalledTimes(2)
  })
})
