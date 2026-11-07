# Módulos funcionales

Crear cada carpeta cuando comience su implementación: `productos`, `ventas`,
`inventario`, `compras`, `proveedores` y `caja`.

Cada módulo contendrá sus componentes, esquemas Zod y estado local. Las páginas
componen módulos; los módulos llaman servicios; los servicios llaman repositorios.
Los repositorios concentran la persistencia. No importar el plugin SQLite desde UI.

Empezar por productos; continuar con venta y movimientos de inventario atómicos.
No crear entidades, tablas ni funcionalidades futuras como parte de esta base.
