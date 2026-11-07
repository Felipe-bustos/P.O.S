// Autor: Felipe Bustos | Fecha: 2026-10-31
// Conserva la ruta Windows de Tauri y prohíbe crear bases accidentalmente.
use crate::error::AppError;
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

pub fn database_path(args: impl IntoIterator<Item = String>) -> Result<PathBuf, AppError> {
    let args: Vec<_> = args.into_iter().collect();
    match args.as_slice() {
        [] => default_path(),
        [flag, path] if flag == "--database" && !path.is_empty() => Ok(PathBuf::from(path)),
        _ => Err(AppError::Arguments),
    }
}

fn default_path() -> Result<PathBuf, AppError> {
    let dirs = directories::BaseDirs::new().ok_or(AppError::MissingDataDirectory)?;
    #[cfg(target_os = "windows")]
    let root = dirs.config_dir();
    #[cfg(not(target_os = "windows"))]
    let root = dirs.data_dir();
    Ok(root.join("cl.almacen.local").join("almacen-local.db"))
}

pub fn open_existing(path: &Path) -> Result<Connection, AppError> {
    Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|source| {
        AppError::Database {
            path: path.to_owned(),
            source,
        }
    })
}

// Autor: Felipe Bustos | Fecha: 2026-11-01
// Escritura autorizada para productos, sin crear bases ausentes.
pub fn open_writable(path: &Path) -> Result<Connection, AppError> {
    let connection =
        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE).map_err(|source| {
            AppError::Database {
                path: path.to_owned(),
                source,
            }
        })?;
    connection.busy_timeout(std::time::Duration::from_secs(2))?;
    // Autor: Felipe Bustos | Fecha: 2026-11-01
    // SQLite exige activar la validación de relaciones por cada conexión.
    connection.pragma_update(None, "foreign_keys", true)?;
    Ok(connection)
}
