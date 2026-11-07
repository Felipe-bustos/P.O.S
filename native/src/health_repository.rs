// Autor: Felipe Bustos | Fecha: 2026-10-31
// Solo abre SQLite en lectura: el traslado de las consultas espera autorización explícita.
use crate::{database, error::AppError};
use std::path::Path;

pub fn verify_open(path: &Path) -> Result<(), AppError> {
    let connection = database::open_existing(path)?;
    drop(connection);
    Ok(())
}
