// Autor: Felipe Bustos | Fecha: 2026-11-01
// Pruebas aisladas con el esquema recibido; nunca abren la base real.
use almacen_local::{product_repository, product_service};
use rusqlite::Connection;

fn fixture() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute_batch(
            "CREATE TABLE Categoria (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            Nombre TEXT NOT NULL COLLATE NOCASE,
            CategoriaPadre_ID INTEGER REFERENCES Categoria(ID),
            Activo INTEGER NOT NULL DEFAULT 1 CHECK(Activo IN (0, 1))
        );
        CREATE TABLE Producto (
            ID INTEGER NOT NULL UNIQUE,
            Nombre TEXT COLLATE NOCASE,
            Activo INTEGER NOT NULL DEFAULT 1 CHECK(Activo IN (0, 1)),
            Categoria_ID INTEGER REFERENCES Categoria(ID),
            PRIMARY KEY(ID AUTOINCREMENT)
        );",
        )
        .unwrap();
    connection
}

#[test]
fn add_list_and_deactivate_preserves_record() {
    let connection = fixture();
    product_service::add(&connection, "  Arroz  ").unwrap();
    product_service::add(&connection, "Aceite").unwrap();
    let products = product_service::list(&connection).unwrap();
    assert_eq!(products[0].name, "Aceite");
    assert_eq!(products[1].name, "Arroz");
    let id = products[1].id;
    product_service::deactivate(&connection, id).unwrap();
    assert_eq!(product_service::list(&connection).unwrap().len(), 1);
    let active: i64 = connection
        .query_row("SELECT Activo FROM Producto WHERE ID = ?1", [id], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(active, 0);
    assert!(product_service::deactivate(&connection, id).is_err());
}

#[test]
fn empty_names_are_rejected_and_sql_characters_are_literal() {
    let connection = fixture();
    assert!(product_service::add(&connection, "  \n ").is_err());
    let name = "Arroz'); DROP TABLE Producto; --";
    product_service::add(&connection, name).unwrap();
    assert_eq!(
        product_repository::list_active(&connection).unwrap()[0].name,
        name
    );
}

#[test]
fn missing_table_reports_error_without_creating_it() {
    let connection = Connection::open_in_memory().unwrap();
    let error = product_service::list(&connection).unwrap_err();
    let message = error.user_message();
    assert!(message.contains("actualizarse"));
    assert!(!message.contains("SELECT"));
    assert!(!message.contains("WITH RECURSIVE"));
    assert!(product_service::add(&connection, "Arroz").is_err());
}

// Estos registros existen solo en SQLite en memoria, nunca en la aplicación ni en la base real.
fn hierarchy_fixture() -> Connection {
    let db = fixture();
    db.execute_batch(
        "INSERT INTO Categoria(ID, Nombre, CategoriaPadre_ID, Activo) VALUES
        (1, 'Bebidas', NULL, 1), (2, 'Gaseosas', 1, 1), (3, 'Cola', 2, 1),
        (4, 'Aguas', 1, 1), (5, 'Oculta', 1, 0), (6, 'Hija oculta', 5, 1),
        (7, 'Comida', NULL, 1);
        INSERT INTO Producto(ID, Nombre, Activo, Categoria_ID) VALUES
        (1, 'Producto cola', 1, 3), (2, 'Producto agua', 1, 4),
        (3, 'Producto inactivo', 0, 3), (4, 'Producto comida', 1, 7),
        (5, 'Sin clasificar', 1, NULL), (6, 'Rama inactiva', 1, 6);",
    )
    .unwrap();
    db
}

#[test]
fn parent_filter_includes_descendants_and_excludes_inactive_branches() {
    use product_repository::Filter;
    let db = hierarchy_fixture();
    let page = product_service::browse(
        &db,
        &Filter {
            category_id: Some(1),
            ..Filter::default()
        },
    )
    .unwrap();
    assert_eq!(
        page.products.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![2, 1]
    );
    let page = product_service::browse(
        &db,
        &Filter {
            category_id: Some(2),
            ..Filter::default()
        },
    )
    .unwrap();
    assert_eq!(
        page.products.iter().map(|p| p.id).collect::<Vec<_>>(),
        vec![1]
    );
    let page = product_service::browse(
        &db,
        &Filter {
            category_id: Some(3),
            ..Filter::default()
        },
    )
    .unwrap();
    assert_eq!(page.products.len(), 1);
    assert_eq!(page.products[0].category_name.as_deref(), Some("Cola"));
    assert!(product_service::browse(
        &db,
        &Filter {
            category_id: Some(5),
            ..Filter::default()
        }
    )
    .unwrap()
    .products
    .is_empty());
}

#[test]
fn search_respects_category_and_treats_wildcards_literally() {
    use product_repository::Filter;
    let db = hierarchy_fixture();
    let filter = Filter {
        category_id: Some(1),
        search: "cola".into(),
        page: 0,
    };
    assert_eq!(
        product_service::browse(&db, &filter).unwrap().products[0].id,
        1
    );
    let filter = Filter {
        category_id: Some(7),
        ..filter
    };
    assert!(product_service::browse(&db, &filter)
        .unwrap()
        .products
        .is_empty());
    let filter = Filter {
        search: "2".into(),
        ..Filter::default()
    };
    assert_eq!(
        product_service::browse(&db, &filter).unwrap().products[0].id,
        2
    );
    product_service::add(&db, "100%_real").unwrap();
    let filter = Filter {
        search: "%_".into(),
        ..Filter::default()
    };
    assert_eq!(
        product_service::browse(&db, &filter)
            .unwrap()
            .products
            .len(),
        1
    );
}

#[test]
fn pagination_does_not_duplicate_rows_and_bounds_memory() {
    use product_repository::{Filter, PAGE_SIZE};
    let db = fixture();
    for n in 0..PAGE_SIZE + 1 {
        product_service::add(&db, &format!("Producto {n:03}")).unwrap();
    }
    let first = product_service::browse(&db, &Filter::default()).unwrap();
    assert_eq!(first.products.len(), PAGE_SIZE);
    assert!(first.has_next);
    let second = product_service::browse(
        &db,
        &Filter {
            page: 1,
            ..Filter::default()
        },
    )
    .unwrap();
    assert_eq!(second.products.len(), 1);
    assert!(!second.has_next);
    assert!(!first.products.iter().any(|p| p.id == second.products[0].id));
}

#[test]
fn selection_uses_real_id_and_rejects_deactivated_product() {
    let db = hierarchy_fixture();
    let product = product_service::select(&db, 1).unwrap();
    assert_eq!(product.name, "Producto cola");
    assert_eq!(product.category_id, Some(3));
    product_service::deactivate(&db, 1).unwrap();
    assert!(product_service::select(&db, 1).is_err());
    assert!(product_service::select(&db, 999).is_err());
}

#[test]
fn tree_expands_three_levels_and_collapses_without_mutating_expansion() {
    let db = hierarchy_fixture();
    let categories = product_service::categories(&db).unwrap();
    assert!(!categories.iter().any(|c| c.id == 5));
    let expanded = std::collections::HashSet::new();
    let collapsed = product_service::visible_categories(&categories, &expanded);
    assert_eq!(collapsed.len(), 2);
    assert!(collapsed.iter().all(|c| c.depth == 0));
    let expanded = [1, 2].into_iter().collect();
    let visible = product_service::visible_categories(&categories, &expanded);
    let cola = visible.iter().find(|c| c.category.id == 3).unwrap();
    assert_eq!(cola.depth, 2);
    assert!(!cola.has_children);
    assert!(!visible.iter().any(|c| c.category.id == 6));
    let expanded = [2].into_iter().collect();
    assert_eq!(
        product_service::visible_categories(&categories, &expanded).len(),
        2
    );
}

#[test]
fn recursive_filter_terminates_on_external_cycles() {
    let db = hierarchy_fixture();
    db.execute(
        "UPDATE Categoria SET CategoriaPadre_ID = 3 WHERE ID = 1",
        [],
    )
    .unwrap();
    let filter = product_repository::Filter {
        category_id: Some(1),
        ..Default::default()
    };
    assert_eq!(
        product_service::browse(&db, &filter)
            .unwrap()
            .products
            .len(),
        2
    );
}

#[test]
fn category_update_refreshes_product_and_filters_without_changing_other_fields() {
    let db = hierarchy_fixture();
    let updated = product_service::update_category(&db, 1, Some(7)).unwrap();
    assert_eq!(updated.id, 1);
    assert_eq!(updated.name, "Producto cola");
    assert_eq!(updated.category_id, Some(7));
    assert_eq!(updated.category_name.as_deref(), Some("Comida"));
    let old = product_repository::Filter {
        category_id: Some(3),
        ..Default::default()
    };
    assert!(product_service::browse(&db, &old)
        .unwrap()
        .products
        .is_empty());
    let new = product_repository::Filter {
        category_id: Some(7),
        ..Default::default()
    };
    assert!(product_service::browse(&db, &new)
        .unwrap()
        .products
        .iter()
        .any(|p| p.id == 1));
    let cleared = product_service::update_category(&db, 1, None).unwrap();
    assert_eq!(cleared.category_id, None);
    assert_eq!(cleared.category_name, None);
    assert_eq!(cleared.name, "Producto cola");
}

#[test]
fn category_update_rejects_unavailable_targets_and_preserves_original_relation() {
    let db = hierarchy_fixture();
    for category in [Some(5), Some(6), Some(999), Some(-1)] {
        assert!(product_service::update_category(&db, 1, category).is_err());
        assert_eq!(
            product_service::select(&db, 1).unwrap().category_id,
            Some(3)
        );
    }
    assert!(product_service::update_category(&db, 999, Some(7)).is_err());
    product_service::deactivate(&db, 1).unwrap();
    assert!(product_service::update_category(&db, 1, Some(7)).is_err());
    let original: Option<i64> = db
        .query_row(
            "SELECT Categoria_ID FROM Producto WHERE ID = 1",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(original, Some(3));
}
