import Database from '@tauri-apps/plugin-sql'
import { isTauri } from '@tauri-apps/api/core'

let connection: Promise<Database> | undefined

export function getDatabase() {
  if (!isTauri()) {
    return Promise.reject(new Error('Abre la aplicación de escritorio para comprobar el almacenamiento local.'))
  }
  connection ??= Database.load('sqlite:almacen-local.db').catch((error: unknown) => {
    connection = undefined
    throw error
  })
  return connection
}
