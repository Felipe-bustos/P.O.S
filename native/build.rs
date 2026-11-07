// Autor: Felipe Bustos | Fecha: 2026-10-31
// Compila la interfaz nativa sin herramientas web.
fn main() {
    slint_build::compile("ui/app-window.slint").expect("No se pudo compilar la interfaz Slint");
}
