use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("No se pudo operar con productos: {0}")]
    Product(#[from] rusqlite::Error),
    #[error("No se pudo determinar la carpeta de datos del usuario.")]
    MissingDataDirectory,
    #[error("No se pudo abrir la base de datos en {path}: {source}")]
    Database {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("Uso: almacen-local [--database RUTA]")]
    Arguments,
}

impl AppError {
    // Autor: Felipe Bustos | Fecha: 2026-11-01
    // Mensajes comprensibles en pantalla sin exponer consultas ni detalles internos.
    pub fn user_message(&self) -> String {
        match self {
            Self::Product(error) => {
                let detail = error.to_string();
                if detail.contains("no such column") || detail.contains("no such table") {
                    "La base de datos necesita actualizarse para usar el selector. Guarda los cambios en DB Browser y cierra su conexión para aplicar la actualización autorizada.".into()
                } else {
                    match error.sqlite_error_code() {
                        Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) =>
                            "La base está ocupada. Guarda los cambios y cierra su conexión en DB Browser; después pulsa Actualizar.".into(),
                        Some(rusqlite::ErrorCode::ReadOnly) => "La base está en modo de solo lectura. Revisa sus permisos antes de guardar.".into(),
                        Some(rusqlite::ErrorCode::ConstraintViolation) => "No se pudo guardar: los datos no cumplen las reglas de la base.".into(),
                        _ => "No se pudo completar la operación con productos. Revisa la base y vuelve a intentar.".into(),
                    }
                }
            }
            Self::Database { .. } => {
                "No se pudo abrir la base. Revisa la ruta, los permisos y las conexiones abiertas."
                    .into()
            }
            _ => self.to_string(),
        }
    }
}
