import { getDatabase } from '../database/connection'

export async function verifyDatabase() {
  const database = await getDatabase()
  await database.execute(`
    CREATE TABLE IF NOT EXISTS app_health (
      id INTEGER PRIMARY KEY CHECK (id = 1),
      message TEXT NOT NULL,
      updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
    )
  `)
  await database.execute(
    `INSERT INTO app_health (id, message) VALUES (1, $1)
     ON CONFLICT(id) DO UPDATE SET message = excluded.message, updated_at = CURRENT_TIMESTAMP`,
    ['SQLite listo'],
  )
  const rows = await database.select<{ message: string }[]>('SELECT message FROM app_health WHERE id = 1')
  if (rows.length !== 1) {
    throw new Error('SQLite no devolvió el registro de prueba')
  }
  return rows[0].message
}
