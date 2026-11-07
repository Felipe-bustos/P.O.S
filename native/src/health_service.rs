use crate::{error::AppError, health_repository};
use std::path::Path;

pub fn check_local_storage(path: &Path) -> Result<String, AppError> {
    health_repository::verify_open(path)?;
    Ok("Conexión abierta en modo lectura. No se comprobaron tablas, datos ni escritura.".into())
}
