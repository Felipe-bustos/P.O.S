// Autor: Felipe Bustos | Fecha: 2026-11-01
// Selector nativo con trabajo SQLite fuera del hilo gráfico y estado conservado.
use crate::{
    database, health_service,
    product_repository::{Category, Filter, Product, ProductPage},
    product_service, AppWindow, CategoryRow, ProductRow,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, collections::HashSet, path::PathBuf, rc::Rc, sync::mpsc, thread};

#[derive(Default)]
struct Browser {
    categories: Vec<Category>,
    expanded: HashSet<i64>,
    filter: Filter,
    selected: Option<Product>,
    editor_expanded: HashSet<i64>,
    editor_category: Option<i64>,
    search_pending: bool,
}

enum Command {
    Check,
    Load(Filter),
    Add(String, Filter),
    Deactivate(i64, Filter),
    Select(i64),
    UpdateCategory(i64, Option<i64>, Filter),
}

enum Response {
    Message(String),
    Loaded {
        filter: Filter,
        categories: Vec<Category>,
        page: ProductPage,
        message: String,
        added: bool,
        deactivated: Option<i64>,
        updated: Option<Product>,
    },
    Selected(Product),
    // La escritura ya se confirmó: no sugerir reintentar y duplicar un producto.
    Written {
        message: String,
        added: bool,
        deactivated: Option<i64>,
        updated: Option<Product>,
    },
}

fn execute(path: &std::path::Path, command: Command) -> Result<Response, crate::error::AppError> {
    if matches!(&command, Command::Check) {
        return health_service::check_local_storage(path).map(Response::Message);
    }
    let writing = matches!(
        &command,
        Command::Add(..) | Command::Deactivate(..) | Command::UpdateCategory(..)
    );
    let connection = if writing {
        database::open_writable(path)?
    } else {
        database::open_existing(path)?
    };
    let (mut filter, message, added, deactivated, updated) = match command {
        Command::UpdateCategory(id, category_id, mut filter) => {
            let updated = product_service::update_category(&connection, id, category_id)?;
            filter.page = 0;
            (
                filter,
                "Categoría del producto actualizada.",
                false,
                None,
                Some(updated),
            )
        }
        Command::Select(id) => {
            return product_service::select(&connection, id).map(Response::Selected)
        }
        Command::Add(name, filter) => {
            product_service::add(&connection, &name)?;
            (
                filter,
                "Producto guardado sin categoría. Usa Todos los productos para encontrarlo.",
                true,
                None,
                None,
            )
        }
        Command::Deactivate(id, filter) => {
            product_service::deactivate(&connection, id)?;
            (
                filter,
                "Producto desactivado. El registro se conserva.",
                false,
                Some(id),
                None,
            )
        }
        Command::Load(filter) => (filter, "", false, None, None),
        Command::Check => unreachable!(),
    };
    let load = (|| {
        let categories = product_service::categories(&connection)?;
        // Una categoría eliminada o inactiva no debe dejar un filtro invisible.
        if filter
            .category_id
            .is_some_and(|id| !categories.iter().any(|c| c.id == id))
        {
            filter.category_id = None;
            filter.page = 0;
        }
        let page = product_service::browse(&connection, &filter)?;
        Ok::<_, crate::error::AppError>((categories, page))
    })();
    match load {
        Ok((categories, page)) => Ok(Response::Loaded {
            filter,
            categories,
            page,
            message: message.into(),
            added,
            deactivated,
            updated,
        }),
        Err(error) if writing => Ok(Response::Written {
            message: format!("{message} {}", error.user_message()),
            added,
            deactivated,
            updated,
        }),
        Err(error) => Err(error),
    }
}

fn refresh_tree(window: &AppWindow, state: &Browser) {
    let rows: Vec<_> = product_service::visible_categories(&state.categories, &state.expanded)
        .into_iter()
        .map(|row| CategoryRow {
            id: row.category.id.to_string().into(),
            name: row.category.name.into(),
            depth: row.depth as i32,
            has_children: row.has_children,
            expanded: row.expanded,
        })
        .collect();
    window.set_categories(ModelRc::from(Rc::new(VecModel::from(rows))));
    window.set_category_count(state.categories.len() as i32);
    window.set_category_label(
        state
            .filter
            .category_id
            .and_then(|id| state.categories.iter().find(|c| c.id == id))
            .map(|c| c.name.clone())
            .unwrap_or_else(|| "Todos los productos".into())
            .into(),
    );
    window.set_category_id(
        state
            .filter
            .category_id
            .map(|id| id.to_string())
            .unwrap_or_default()
            .into(),
    );
    window.set_category_title(if state.filter.category_id.is_none() {
        "Todos los productos".into()
    } else {
        category_path(&state.categories, state.filter.category_id).into()
    });
}

fn category_path(categories: &[Category], mut id: Option<i64>) -> String {
    if id.is_none() {
        return "Sin categoría".into();
    }
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    while let Some(current) = id {
        if !seen.insert(current) {
            break;
        }
        if let Some(category) = categories.iter().find(|c| c.id == current) {
            names.push(category.name.clone());
            id = category.parent_id;
        } else {
            break;
        }
    }
    names.reverse();
    if names.is_empty() {
        "Categoría no disponible".into()
    } else {
        names.join(" > ")
    }
}

fn refresh_editor(window: &AppWindow, state: &Browser) {
    let rows: Vec<_> =
        product_service::visible_categories(&state.categories, &state.editor_expanded)
            .into_iter()
            .map(|row| CategoryRow {
                id: row.category.id.to_string().into(),
                name: row.category.name.into(),
                depth: row.depth as i32,
                has_children: row.has_children,
                expanded: row.expanded,
            })
            .collect();
    window.set_editor_categories(ModelRc::from(Rc::new(VecModel::from(rows))));
    window.set_editor_category_id(
        state
            .editor_category
            .map(|id| id.to_string())
            .unwrap_or_default()
            .into(),
    );
    window
        .set_editor_category_title(category_path(&state.categories, state.editor_category).into());
}

fn clear_selection(window: &AppWindow, state: &mut Browser, id: Option<i64>) {
    if state.selected.as_ref().is_some_and(|p| Some(p.id) == id) {
        state.selected = None;
        window.set_selected_id("".into());
        window.set_selected_description("".into());
        window.set_selected_name("".into());
        window.set_selected_category_path("".into());
        window.set_selected_category_id("".into());
        window.set_category_editor_open(false);
    }
}

fn show_selection(window: &AppWindow, state: &mut Browser, product: Product) {
    window.set_selected_id(product.id.to_string().into());
    window.set_selected_name(product.name.clone().into());
    window.set_selected_category_path(category_path(&state.categories, product.category_id).into());
    window.set_selected_category_id(
        product
            .category_id
            .map(|id| id.to_string())
            .unwrap_or_default()
            .into(),
    );
    window.set_selected_description(
        format!(
            "{} | ID {} | {}",
            product.name,
            product.id,
            product.category_name.as_deref().unwrap_or("Sin categoría")
        )
        .into(),
    );
    state.selected = Some(product);
}

fn request(window: &AppWindow, sender: &mpsc::SyncSender<Command>, command: Command) {
    if window.get_busy() {
        return;
    }
    if matches!(&command, Command::Load(_)) {
        window.set_products(ModelRc::default());
        window.set_product_count(0);
        window.set_has_next(false);
    }
    window.set_busy(true);
    window.set_has_error(false);
    window.set_status("".into());
    if sender.try_send(command).is_err() {
        window.set_busy(false);
        window.set_has_error(true);
        window.set_status("No se pudo iniciar la operación. Vuelve a abrir la aplicación.".into());
    }
}

pub fn run(path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let window = AppWindow::new()?;
    let weak = window.as_weak();
    window.global::<crate::DialogKeys>().on_dismiss(move || {
        if let Some(window) = weak.upgrade() {
            window.invoke_close_modals();
        }
    });
    window.set_database_path(path.display().to_string().into());
    let state = Rc::new(RefCell::new(Browser::default()));
    // Autor: Felipe Bustos | Fecha: 2026-11-01
    // Una consulta por pausa de escritura; si hay trabajo pendiente se conserva la ultima busqueda.
    let search_timer = Rc::new(slint::Timer::default());
    let (sender, receiver) = mpsc::sync_channel::<Command>(1);
    // El trabajador solo recibe datos propios; Rc y los modelos Slint quedan en el hilo gráfico.
    let weak = window.as_weak();
    let (results_tx, results_rx) = mpsc::channel();
    let worker = thread::Builder::new()
        .name("sqlite-storage".into())
        .spawn(move || {
            while let Ok(command) = receiver.recv() {
                if results_tx.send(execute(&path, command)).is_err() {
                    break;
                }
                let weak = weak.clone();
                if weak
                    .upgrade_in_event_loop(|window| window.invoke_storage_result())
                    .is_err()
                {
                    break;
                }
            }
        })?;
    let weak = window.as_weak();
    let browser = state.clone();
    let results_sender = sender.clone();
    window.on_storage_result(move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        if let Ok(result) = results_rx.try_recv() {
            window.set_busy(false);
            let mut state = browser.borrow_mut();
            match result {
                Ok(Response::Message(message)) => {
                    window.set_has_error(false);
                    window.set_status(message.into());
                }
                Ok(Response::Selected(product)) => {
                    show_selection(&window, &mut state, product);
                    window.set_has_error(false);
                    window.set_status("".into());
                }
                Ok(Response::Written {
                    message,
                    added,
                    deactivated,
                    updated,
                }) => {
                    if let Some(product) = updated {
                        show_selection(&window, &mut state, product);
                        window.set_category_editor_open(false);
                    }
                    if added {
                        window.set_product_name("".into());
                        window.set_new_product_open(false);
                    }
                    clear_selection(&window, &mut state, deactivated);
                    window.set_has_error(true);
                    window.set_status(message.into());
                }
                Ok(Response::Loaded {
                    filter,
                    categories,
                    page,
                    message,
                    added,
                    deactivated,
                    updated,
                }) => {
                    if let Some(product) = updated {
                        show_selection(&window, &mut state, product);
                        window.set_category_editor_open(false);
                    }
                    if added {
                        window.set_product_name("".into());
                        window.set_new_product_open(false);
                    }
                    clear_selection(&window, &mut state, deactivated);
                    state.filter = filter;
                    state.categories = categories;
                    // Conserva el orden estable del catalogo jerarquico por su ID real.
                    state.categories.sort_by_key(|category| category.id);
                    refresh_tree(&window, &state);
                    window.set_page_number(state.filter.page as i32 + 1);
                    window.set_has_next(page.has_next);
                    window.set_product_count(page.products.len() as i32);
                    let rows: Vec<_> = page
                        .products
                        .into_iter()
                        .map(|p| ProductRow {
                            id: p.id.to_string().into(),
                            name: p.name.into(),
                            category: p
                                .category_name
                                .unwrap_or_else(|| "Sin categoría".into())
                                .into(),
                        })
                        .collect();
                    window.set_products(ModelRc::from(Rc::new(VecModel::from(rows))));
                    window.set_has_error(false);
                    window.set_status(message.into());
                }
                Err(error) => {
                    window.set_has_error(true);
                    window.set_status(error.user_message().into());
                }
            }
            if state.search_pending {
                state.search_pending = false;
                state.filter.search = window.get_search_text().to_string();
                state.filter.page = 0;
                request(
                    &window,
                    &results_sender,
                    Command::Load(state.filter.clone()),
                );
            }
        }
    });
    let weak = window.as_weak();
    let timer = search_timer.clone();
    window.on_search_edited(move || {
        let weak = weak.clone();
        timer.start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(350),
            move || {
                if let Some(w) = weak.upgrade() {
                    w.invoke_search_products();
                }
            },
        );
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    window.on_check_storage(move || {
        if let Some(w) = weak.upgrade() {
            request(&w, &tx, Command::Check);
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_refresh_products(move || {
        if let Some(w) = weak.upgrade() {
            request(&w, &tx, Command::Load(browser.borrow().filter.clone()));
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_search_products(move || {
        if let Some(w) = weak.upgrade() {
            if w.get_category_editor_open() || w.get_new_product_open() {
                return;
            }
            if w.get_busy() {
                browser.borrow_mut().search_pending = true;
                return;
            }
            let mut state = browser.borrow_mut();
            state.filter.search = w.get_search_text().to_string();
            state.filter.page = 0;
            request(&w, &tx, Command::Load(state.filter.clone()));
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_choose_category(move |id| {
        if let Some(w) = weak.upgrade() {
            if w.get_busy() {
                return;
            }
            let mut state = browser.borrow_mut();
            state.filter.category_id = id.parse().ok();
            state.filter.search = w.get_search_text().to_string();
            state.filter.page = 0;
            refresh_tree(&w, &state);
            request(&w, &tx, Command::Load(state.filter.clone()));
        }
    });
    let weak = window.as_weak();
    let browser = state.clone();
    window.on_toggle_category(move |id| {
        if let (Some(w), Ok(id)) = (weak.upgrade(), id.parse::<i64>()) {
            let mut state = browser.borrow_mut();
            if !state.expanded.remove(&id) {
                state.expanded.insert(id);
            }
            refresh_tree(&w, &state);
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_change_page(move |delta| {
        if let Some(w) = weak.upgrade() {
            if w.get_busy() {
                return;
            }
            let mut state = browser.borrow_mut();
            if delta > 0 && w.get_has_next() {
                state.filter.page += 1;
            } else if delta < 0 && state.filter.page > 0 {
                state.filter.page -= 1;
            } else {
                return;
            }
            request(&w, &tx, Command::Load(state.filter.clone()));
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_add_product(move || {
        if let Some(w) = weak.upgrade() {
            request(
                &w,
                &tx,
                Command::Add(
                    w.get_product_name().to_string(),
                    browser.borrow().filter.clone(),
                ),
            );
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_deactivate_product(move |id| {
        if let (Some(w), Ok(id)) = (weak.upgrade(), id.parse()) {
            request(
                &w,
                &tx,
                Command::Deactivate(id, browser.borrow().filter.clone()),
            );
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    window.on_select_product(move |id| {
        if let (Some(w), Ok(id)) = (weak.upgrade(), id.parse()) {
            request(&w, &tx, Command::Select(id));
        }
    });
    let weak = window.as_weak();
    let tx = sender.clone();
    let browser = state.clone();
    window.on_update_product_category(move |category| {
        if let Some(w) = weak.upgrade() {
            if w.get_busy() {
                return;
            }
            let state = browser.borrow();
            let Some(product) = &state.selected else {
                return;
            };
            let category_id = if category.is_empty() {
                None
            } else {
                match category.parse::<i64>() {
                    Ok(id) => Some(id),
                    Err(_) => return,
                }
            };
            request(
                &w,
                &tx,
                Command::UpdateCategory(product.id, category_id, state.filter.clone()),
            );
        }
    });
    let weak = window.as_weak();
    let browser = state.clone();
    let timer = search_timer.clone();
    window.on_open_category_editor(move || {
        if let Some(w) = weak.upgrade() {
            if w.get_busy() {
                return;
            }
            let mut state = browser.borrow_mut();
            let Some(product) = state.selected.clone() else {
                return;
            };
            timer.stop();
            state.editor_category = product.category_id;
            state.editor_expanded.clear();
            let mut parent = product.category_id;
            while let Some(id) = parent {
                if !state.editor_expanded.insert(id) {
                    break;
                }
                parent = state
                    .categories
                    .iter()
                    .find(|c| c.id == id)
                    .and_then(|c| c.parent_id);
            }
            refresh_editor(&w, &state);
            w.set_has_error(false);
            w.set_status("".into());
            w.set_category_editor_open(true);
            w.invoke_focus_modal();
        }
    });
    let weak = window.as_weak();
    let browser = state.clone();
    window.on_choose_editor_category(move |id| {
        if let (Some(w), Ok(id)) = (weak.upgrade(), id.parse::<i64>()) {
            if w.get_busy() {
                return;
            }
            let mut state = browser.borrow_mut();
            state.editor_category = Some(id);
            // No reconstruir las filas al seleccionar: conservar el foco del control pulsado.
            w.set_editor_category_id(id.to_string().into());
            w.set_editor_category_title(category_path(&state.categories, Some(id)).into());
        }
    });
    let weak = window.as_weak();
    let browser = state.clone();
    window.on_toggle_editor_category(move |id| {
        if let (Some(w), Ok(id)) = (weak.upgrade(), id.parse::<i64>()) {
            if w.get_busy() {
                return;
            }
            let mut state = browser.borrow_mut();
            if !state.editor_expanded.remove(&id) {
                state.editor_expanded.insert(id);
            }
            refresh_editor(&w, &state);
            w.invoke_focus_modal();
        }
    });
    request(&window, &sender, Command::Load(Filter::default()));
    drop(sender);
    let result = window.run();
    search_timer.stop();
    drop(window);
    worker
        .join()
        .map_err(|_| "El trabajador de almacenamiento terminó inesperadamente")?;
    result?;
    Ok(())
}
