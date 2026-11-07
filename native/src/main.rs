#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = almacen_local::database::database_path(std::env::args().skip(1))?;
    // Autor: Felipe Bustos | Fecha: 2026-10-31
    // El renderizador por software evita exigir una GPU moderna.
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    almacen_local::controller::run(path)
}
