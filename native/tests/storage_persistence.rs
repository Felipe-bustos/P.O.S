// Autor: Felipe Bustos | Fecha: 2026-10-31
// Comprueba las garantías de apertura sin introducir consultas SQL.
use almacen_local::{database, health_service};

#[test]
fn missing_database_is_not_created() {
    let path = std::env::temp_dir().join(format!("almacen-missing-{}.db", std::process::id()));
    assert!(!path.exists());
    assert!(health_service::check_local_storage(&path).is_err());
    assert!(database::open_writable(&path).is_err());
    assert!(!path.exists());
}

#[test]
fn explicit_database_path_is_preserved() {
    let path =
        database::database_path(["--database".into(), "datos del almacén.db".into()]).unwrap();
    assert_eq!(path, std::path::PathBuf::from("datos del almacén.db"));
}

#[test]
fn invalid_arguments_are_rejected() {
    assert!(database::database_path(["--database".into()]).is_err());
    assert!(database::database_path(["--other".into(), "file.db".into()]).is_err());
}
