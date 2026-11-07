// Autor: Felipe Bustos | Fecha: 2026-11-01
// Selector real con parámetros, filtro recursivo y páginas acotadas. Sin migraciones automáticas.
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};

pub const PAGE_SIZE: usize = 40;

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub id: i64,
    pub name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub category_id: Option<i64>,
    pub search: String,
    pub page: u32,
}

pub struct ProductPage {
    pub products: Vec<Product>,
    pub has_next: bool,
}

pub fn categories(connection: &Connection) -> Result<Vec<Category>, AppError> {
    let mut statement = connection.prepare(
        "SELECT ID, Nombre, CategoriaPadre_ID FROM Categoria WHERE Activo = 1 ORDER BY Nombre COLLATE NOCASE, ID",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            parent_id: row.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn read_product(row: &rusqlite::Row<'_>) -> rusqlite::Result<Product> {
    Ok(Product {
        id: row.get(0)?,
        name: row.get(1)?,
        category_id: row.get(2)?,
        category_name: row.get(3)?,
    })
}

pub fn page(connection: &Connection, filter: &Filter) -> Result<ProductPage, AppError> {
    let escaped = filter
        .search
        .trim()
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    let pattern = format!("%{escaped}%");
    // UNION elimina duplicados y garantiza terminación incluso ante ciclos en datos externos.
    let mut statement = connection.prepare(
        "WITH RECURSIVE descendants(ID) AS (
            SELECT ID FROM Categoria WHERE ID = ?1 AND Activo = 1
            UNION
            SELECT c.ID FROM Categoria c JOIN descendants d ON c.CategoriaPadre_ID = d.ID WHERE c.Activo = 1
        )
        SELECT p.ID, COALESCE(p.Nombre, ''), p.Categoria_ID, c.Nombre
        FROM Producto p LEFT JOIN Categoria c ON c.ID = p.Categoria_ID
        WHERE p.Activo = 1
          AND (?1 IS NULL OR p.Categoria_ID IN (SELECT ID FROM descendants))
          AND (?2 = '' OR p.Nombre LIKE ?3 ESCAPE '!' OR CAST(p.ID AS TEXT) = ?2)
        ORDER BY p.Nombre COLLATE NOCASE, p.ID LIMIT ?4 OFFSET ?5",
    )?;
    let rows = statement.query_map(
        params![
            filter.category_id,
            filter.search.trim(),
            pattern,
            (PAGE_SIZE + 1) as i64,
            i64::from(filter.page) * PAGE_SIZE as i64
        ],
        read_product,
    )?;
    let mut products = rows.collect::<Result<Vec<_>, _>>()?;
    let has_next = products.len() > PAGE_SIZE;
    products.truncate(PAGE_SIZE);
    Ok(ProductPage { products, has_next })
}

// Conserva el punto de entrada anterior, ahora limitado a la primera página.
pub fn list_active(connection: &Connection) -> Result<Vec<Product>, AppError> {
    Ok(page(connection, &Filter::default())?.products)
}

pub fn find_active(connection: &Connection, id: i64) -> Result<Product, AppError> {
    connection
        .query_row(
            "SELECT p.ID, COALESCE(p.Nombre, ''), p.Categoria_ID, c.Nombre FROM Producto p
         LEFT JOIN Categoria c ON c.ID = p.Categoria_ID WHERE p.ID = ?1 AND p.Activo = 1",
            [id],
            read_product,
        )
        .optional()?
        .ok_or_else(|| {
            AppError::Validation(
                "El producto ya no está activo o no existe. Actualiza la lista.".into(),
            )
        })
}

pub fn insert(connection: &Connection, name: &str) -> Result<(), AppError> {
    connection.execute("INSERT INTO Producto (Nombre) VALUES (?1)", [name])?;
    Ok(())
}

// Autor: Felipe Bustos | Fecha: 2026-11-01
// Cambia solo la relación del producto. Valida la rama activa en la misma sentencia.
pub fn update_category(
    connection: &Connection,
    id: i64,
    category_id: Option<i64>,
) -> Result<(), AppError> {
    let affected = connection.execute(
        "WITH RECURSIVE available(ID, Level) AS (
            SELECT ID, 1 FROM Categoria WHERE CategoriaPadre_ID IS NULL AND Activo = 1
            UNION ALL
            SELECT c.ID, a.Level + 1 FROM Categoria c JOIN available a ON c.CategoriaPadre_ID = a.ID
            WHERE c.Activo = 1 AND a.Level < 3
        )
        UPDATE Producto SET Categoria_ID = ?1 WHERE ID = ?2 AND Activo = 1
        AND (?1 IS NULL OR ?1 IN (SELECT ID FROM available))",
        params![category_id, id],
    )?;
    if affected != 1 {
        return Err(AppError::Validation(
            "No se pudo cambiar la categoría: el producto o la categoría ya no están disponibles. Actualiza la lista.".into(),
        ));
    }
    Ok(())
}

pub fn deactivate(connection: &Connection, id: i64) -> Result<(), AppError> {
    let affected = connection.execute(
        "UPDATE Producto SET Activo = 0 WHERE ID = ?1 AND Activo = 1",
        [id],
    )?;
    if affected != 1 {
        return Err(AppError::Validation(
            "El producto ya no está activo o no existe. Actualiza la lista.".into(),
        ));
    }
    Ok(())
}
