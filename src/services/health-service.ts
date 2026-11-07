import { verifyDatabase } from '../repositories/health-repository'

// The UI depends on this use case; the SQLite adapter remains in repositories.
export async function checkLocalStorage(): Promise<void> {
  const message = await verifyDatabase()
  if (message !== 'SQLite listo') {
    throw new Error('La lectura no coincide con el dato guardado. Intenta nuevamente.')
  }
}
