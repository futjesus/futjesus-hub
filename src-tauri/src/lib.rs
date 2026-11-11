mod gmail;
mod hibernate;
mod memory;
mod menu;
mod store;

use std::collections::HashMap;

use serde::Serialize;
use tauri::{
    AppHandle, Emitter, LogicalPosition, Manager, State, WebviewBuilder, WebviewUrl, Window,
    WindowEvent, window::WindowBuilder,
};
use uuid::Uuid;

use store::{Account, AppState, Settings, Store};

type CommandResult<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn main_window(app: &AppHandle) -> CommandResult<Window> {
    app.get_window("main").ok_or_else(|| "main window not found".into())
}

/// Shows the active account's webview and hides the rest. Everything is
/// hidden while the sidebar UI shows a modal.
fn apply_visibility(app: &AppHandle) {
    let state = app.state::<AppState>();
    let active = state.store.lock().unwrap().active.clone();
    let overlay = *state.overlay.lock().unwrap();
    let Some(window) = app.get_window("main") else {
        return;
    };
    for webview in window.webviews() {
        let Some(id) = gmail::account_id(webview.label()) else {
            continue;
        };
        if !overlay && active.as_deref() == Some(id) {
            let _ = webview.show();
            let _ = webview.set_focus();
        } else {
            let _ = webview.hide();
        }
    }
}

fn refresh_menu(app: &AppHandle) {
    let accounts = app.state::<AppState>().store.lock().unwrap().accounts.clone();
    match menu::build(app, &accounts) {
        Ok(menu) => {
            let _ = app.set_menu(menu);
        }
        Err(e) => eprintln!("failed to build menu: {e}"),
    }
}

fn select(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    let previous = {
        let mut store = state.store.lock().unwrap();
        if !store.accounts.iter().any(|a| a.id == id) {
            return;
        }
        let previous = store.active.replace(id.to_string());
        state.persist(&store);
        previous
    };
    // The account being left starts its idle time now.
    if let Some(previous) = previous {
        hibernate::touch(app, &previous);
    }
    hibernate::touch(app, id);
    if let Err(e) = hibernate::wake(app, id) {
        eprintln!("failed to wake account: {e}");
    }
    apply_visibility(app);
    let _ = app.emit("active-changed", id);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    accounts: Vec<Account>,
    settings: Settings,
    active: Option<String>,
    unread: HashMap<String, u32>,
    hibernated: Vec<String>,
}

#[tauri::command]
async fn get_snapshot(state: State<'_, AppState>) -> CommandResult<Snapshot> {
    let store = state.store.lock().unwrap().clone();
    Ok(Snapshot {
        accounts: store.accounts,
        settings: store.settings,
        active: store.active,
        unread: state.unread.lock().unwrap().clone(),
        hibernated: state.hibernated.lock().unwrap().iter().cloned().collect(),
    })
}

#[tauri::command]
async fn add_account(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> CommandResult<Account> {
    let account = Account {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        email: None,
        avatar: None,
        zoom: 1.0,
    };
    gmail::open(&main_window(&app)?, &account).map_err(err)?;
    {
        let mut store = state.store.lock().unwrap();
        store.accounts.push(account.clone());
        store.active = Some(account.id.clone());
        state.persist(&store);
    }
    refresh_menu(&app);
    apply_visibility(&app);
    Ok(account)
}

#[tauri::command]
async fn update_account(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
) -> CommandResult<Vec<Account>> {
    let accounts = {
        let mut store = state.store.lock().unwrap();
        let account = store
            .accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or("account not found")?;
        account.name = name;
        account.color = color;
        state.persist(&store);
        store.accounts.clone()
    };
    refresh_menu(&app);
    Ok(accounts)
}

/// Removes the account and wipes its cookies and cached mail.
#[tauri::command]
async fn remove_account(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Snapshot> {
    if let Some(webview) = app.get_webview(&gmail::label(&id)) {
        let _ = webview.clear_all_browsing_data();
        webview.close().map_err(err)?;
    }
    {
        let mut store = state.store.lock().unwrap();
        store.accounts.retain(|a| a.id != id);
        if store.active.as_deref() == Some(id.as_str()) {
            store.active = store.accounts.first().map(|a| a.id.clone());
        }
        state.persist(&store);
    }
    state.hibernated.lock().unwrap().remove(&id);
    state.unread.lock().unwrap().remove(&id);
    gmail::update_badge(&app);
    refresh_menu(&app);
    apply_visibility(&app);
    get_snapshot(state).await
}

#[tauri::command]
async fn reorder_accounts(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> CommandResult<Vec<Account>> {
    let accounts = {
        let mut store = state.store.lock().unwrap();
        store
            .accounts
            .sort_by_key(|a| ids.iter().position(|id| *id == a.id).unwrap_or(usize::MAX));
        state.persist(&store);
        store.accounts.clone()
    };
    refresh_menu(&app);
    Ok(accounts)
}

#[tauri::command]
async fn select_account(app: AppHandle, id: String) -> CommandResult<()> {
    select(&app, &id);
    Ok(())
}

#[tauri::command]
async fn reload_account(app: AppHandle, id: String) -> CommandResult<()> {
    if hibernate::is_hibernated(&app, &id) {
        return hibernate::wake(&app, &id).map_err(err);
    }
    app.get_webview(&gmail::label(&id))
        .ok_or("account not found")?
        .reload()
        .map_err(err)
}

#[tauri::command]
async fn hibernate_account(app: AppHandle, id: String) -> CommandResult<()> {
    hibernate::hibernate(&app, &id);
    Ok(())
}

#[tauri::command]
async fn set_overlay(app: AppHandle, state: State<'_, AppState>, visible: bool) -> CommandResult<()> {
    *state.overlay.lock().unwrap() = visible;
    apply_visibility(&app);
    Ok(())
}

#[tauri::command]
async fn update_settings(state: State<'_, AppState>, settings: Settings) -> CommandResult<Settings> {
    let mut store = state.store.lock().unwrap();
    store.settings = settings;
    state.persist(&store);
    Ok(store.settings.clone())
}

#[tauri::command]
async fn get_memory_usage(app: AppHandle) -> CommandResult<memory::MemoryReport> {
    Ok(memory::report(&app))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("accounts.json");
            let store = Store::load(&path);
            let accounts = store.accounts.clone();
            app.manage(AppState::new(store, path));

            let builder = WindowBuilder::new(app, "main")
                .title("Futjesus Hub")
                .inner_size(1280.0, 820.0)
                .min_inner_size(820.0, 520.0);
            #[cfg(target_os = "macos")]
            let builder = builder
                .title_bar_style(tauri::TitleBarStyle::Overlay)
                .hidden_title(true);
            let window = builder.build()?;

            // The React UI fills the whole window; Gmail webviews sit on top
            // of it, to the right of the sidebar.
            let size = window.inner_size()?.to_logical::<f64>(window.scale_factor()?);
            window.add_child(
                WebviewBuilder::new("ui", WebviewUrl::App("index.html".into())).auto_resize(),
                LogicalPosition::new(0.0, 0.0),
                size,
            )?;
            for account in &accounts {
                gmail::open(&window, account)?;
            }

            let resized = window.clone();
            window.on_window_event(move |event| {
                if matches!(
                    event,
                    WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }
                ) {
                    gmail::layout(&resized);
                }
            });

            refresh_menu(app.handle());
            apply_visibility(app.handle());
            gmail::start_status_polling(app.handle().clone());
            hibernate::start_watch(app.handle().clone());
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if let Some(account_id) = id.strip_prefix(menu::ACCOUNT_PREFIX) {
                select(app, account_id);
            } else if let Some(zoom) = menu::zoom_action(id) {
                gmail::zoom_active(app, zoom);
            } else if id == menu::MEMORY_ID {
                let _ = app.emit("show-memory", ());
            } else if id == menu::RELOAD_ID {
                let active = app.state::<AppState>().store.lock().unwrap().active.clone();
                if let Some(webview) = active.and_then(|id| app.get_webview(&gmail::label(&id))) {
                    let _ = webview.reload();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            add_account,
            update_account,
            remove_account,
            reorder_accounts,
            select_account,
            reload_account,
            set_overlay,
            update_settings,
            get_memory_usage,
            hibernate_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
