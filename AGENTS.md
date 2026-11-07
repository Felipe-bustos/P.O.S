# Instrucciones del proyecto

## Marco adoptado

El responsable humano solicitó «implementa ese marco de trabajo» el 2026-11-07,
aportando los dos documentos originales. Esta solicitud adopta para este proyecto:

- [Constitución 2.0](docs/marco-de-trabajo/constitucion-2.0.md).
- [Ley General 01 de Ejecución y Control de Cambios, versión 1.0](docs/marco-de-trabajo/ley-general-01-v1.0.md).

Lee ambos textos completos antes de realizar cambios. Las copias conservan el
contenido textual de los documentos aportados; sus originales no se modifican.
No hay otras leyes especiales ni reglamentos adoptados en esta incorporación.
No inventes versiones anteriores ni aprobaciones adicionales.

La jerarquía del proyecto es Constitución, leyes generales, leyes especiales,
reglamentos e instrucciones operativas. No sustituye las restricciones superiores
del entorno ni los permisos efectivos de las herramientas. Si aparece un conflicto
material, detén solo la acción dependiente y explica la incompatibilidad.

## Restricción expresa sobre SQL

Instrucción humana conservada literalmente:

> nunca pero nunca hacer archivos sql ni consultas directas sin que se solicite explicitamente hacer hardcode

Aplica el artículo 6 de la Ley General 01: una petición genérica de implementar,
corregir o revisar no autoriza archivos SQL ni consultas directas. La autorización
debe ser explícita e identificar su alcance. No traslades SQL a otro archivo,
script, consola o herramienta para eludir la restricción. Crear un archivo no
autoriza ejecutarlo. Usa únicamente mecanismos del proyecto ya autorizados.
Las autorizaciones históricas no son permisos generales para nuevas operaciones.

## Aplicación durante el trabajo

- Establece el resultado esperado, el alcance y una condición verificable de cierre.
- Inspecciona el estado actual y conserva los cambios existentes ajenos al encargo.
- Ejecuta las acciones necesarias y proporcionales ya autorizadas sin pedir de nuevo
  el mismo permiso. Reserva decisiones humanas para lo que exceda ese alcance.
- No elimines recursos, publiques, despliegues, comuniques a terceros ni cambies
  permisos sin autorización que cubra la acción y su alcance.
- Comprueba los efectos de un fallo antes de reintentarlo; no debilites controles.
- Valida de forma proporcional y distingue comprobaciones ejecutadas y pendientes.
- Entrega el resultado real, los archivos afectados y las limitaciones materiales.
- Al traspasar trabajo, conserva objetivo, estado, normas, autorizaciones y evidencia.

## Contexto técnico para retomar

La aplicación activa es Rust + Slint + SQLite y está en native/.
Consulta README.md para ejecución, arquitectura y verificaciones. No se adopta
React o Tauri como interfaz por la presencia de archivos heredados.

La incorporación del marco solo modifica documentación. No autoriza migraciones,
consultas nuevas, cambios de datos, limpieza de archivos heredados ni publicación.
Las comprobaciones que ejecuten operaciones de base de datos deben respetar la
restricción anterior; no las ejecutes automáticamente por adoptar este marco.

Registro de incorporación: Felipe Bustos, 2026-11-07. Se incorporan las normas
para hacerlas accesibles a los agentes que retomen este repositorio. Los cambios
posteriores al marco requieren la aprobación prevista en sus propios textos.
