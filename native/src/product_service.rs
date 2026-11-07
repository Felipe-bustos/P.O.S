// Autor: Felipe Bustos | Fecha: 2026-11-01
// Validaciones de negocio independientes de Slint y del código SQL.
use crate::{
    error::AppError,
    product_repository::{self, Product},
};
use rusqlite::Connection;

pub fn list(connection: &Connection) -> Result<Vec<Product>, AppError> {
    product_repository::list_active(connection)
}

pub fn add(connection: &Connection, name: &str) -> Result<(), AppError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "Escribe el nombre del producto.".into(),
        ));
    }
    product_repository::insert(connection, name)
}

pub fn deactivate(connection: &Connection, id: i64) -> Result<(), AppError> {
    if id <= 0 {
        return Err(AppError::Validation(
            "Selecciona un producto válido.".into(),
        ));
    }
    product_repository::deactivate(connection, id)
}

// Autor: Felipe Bustos | Fecha: 2026-11-01
// Casos de uso del selector; no dependen de la presentación Slint.
pub fn browse(
    connection: &Connection,
    filter: &product_repository::Filter,
) -> Result<product_repository::ProductPage, AppError> {
    product_repository::page(connection, filter)
}

pub fn categories(connection: &Connection) -> Result<Vec<product_repository::Category>, AppError> {
    product_repository::categories(connection)
}

pub fn select(connection: &Connection, id: i64) -> Result<Product, AppError> {
    product_repository::find_active(connection, id)
}

// Autor: Felipe Bustos | Fecha: 2026-11-01
// Devuelve el producto actualizado para conservar una selección coherente en pantalla.
pub fn update_category(
    connection: &Connection,
    id: i64,
    category_id: Option<i64>,
) -> Result<Product, AppError> {
    if id <= 0 || category_id.is_some_and(|value| value <= 0) {
        return Err(AppError::Validation(
            "Selecciona un producto y una categoría válidos.".into(),
        ));
    }
    product_repository::update_category(connection, id, category_id)?;
    product_repository::find_active(connection, id)
}

#[derive(Debug)]
pub struct VisibleCategory {
    pub category: product_repository::Category,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

// Solo raíces activas y sus hijos, hasta el tercer nivel; colapsar no cambia la selección.
pub fn visible_categories(
    categories: &[product_repository::Category],
    expanded: &std::collections::HashSet<i64>,
) -> Vec<VisibleCategory> {
    fn walk(
        categories: &[product_repository::Category],
        expanded: &std::collections::HashSet<i64>,
        parent: Option<i64>,
        depth: usize,
        output: &mut Vec<VisibleCategory>,
    ) {
        if depth >= 3 {
            return;
        }
        for category in categories
            .iter()
            .filter(|category| category.parent_id == parent)
        {
            let has_children = depth < 2
                && categories
                    .iter()
                    .any(|child| child.parent_id == Some(category.id));
            output.push(VisibleCategory {
                category: category.clone(),
                depth,
                has_children,
                expanded: expanded.contains(&category.id),
            });
            if has_children && expanded.contains(&category.id) {
                walk(categories, expanded, Some(category.id), depth + 1, output);
            }
        }
    }
    let mut output = Vec::new();
    walk(categories, expanded, None, 0, &mut output);
    output
}
