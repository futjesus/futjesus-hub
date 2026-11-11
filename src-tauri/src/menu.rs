use tauri::{
    AppHandle, Wry,
    menu::{IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu},
};

use crate::store::Account;

pub const ACCOUNT_PREFIX: &str = "account:";
pub const RELOAD_ID: &str = "reload";
pub const MEMORY_ID: &str = "memory";
const ZOOM_IN_ID: &str = "zoom-in";
const ZOOM_OUT_ID: &str = "zoom-out";
const ZOOM_RESET_ID: &str = "zoom-reset";

pub enum Zoom {
    In,
    Out,
    Reset,
}

pub fn zoom_action(id: &str) -> Option<Zoom> {
    match id {
        ZOOM_IN_ID => Some(Zoom::In),
        ZOOM_OUT_ID => Some(Zoom::Out),
        ZOOM_RESET_ID => Some(Zoom::Reset),
        _ => None,
    }
}

/// The default macOS menu plus an "Accounts" menu with ⌘1…⌘9 shortcuts.
/// Native shortcuts work even while a Gmail webview has keyboard focus.
pub fn build(app: &AppHandle, accounts: &[Account]) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::default(app)?;

    let items = accounts
        .iter()
        .take(9)
        .enumerate()
        .map(|(i, account)| {
            MenuItem::with_id(
                app,
                format!("{ACCOUNT_PREFIX}{}", account.id),
                &account.name,
                true,
                Some(format!("CmdOrCtrl+{}", i + 1)),
            )
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let separator = PredefinedMenuItem::separator(app)?;
    let reload = MenuItem::with_id(
        app,
        RELOAD_ID,
        "Recargar cuenta",
        true,
        Some("CmdOrCtrl+Shift+R"),
    )?;

    let zoom_separator = PredefinedMenuItem::separator(app)?;
    let zoom_in = MenuItem::with_id(app, ZOOM_IN_ID, "Acercar", true, Some("CmdOrCtrl+="))?;
    let zoom_out = MenuItem::with_id(app, ZOOM_OUT_ID, "Alejar", true, Some("CmdOrCtrl+-"))?;
    let zoom_reset =
        MenuItem::with_id(app, ZOOM_RESET_ID, "Tamaño real", true, Some("CmdOrCtrl+0"))?;

    let memory_separator = PredefinedMenuItem::separator(app)?;
    let memory = MenuItem::with_id(
        app,
        MEMORY_ID,
        "Consumo de memoria…",
        true,
        Some("CmdOrCtrl+Shift+M"),
    )?;

    let mut entries: Vec<&dyn IsMenuItem<Wry>> =
        items.iter().map(|item| item as &dyn IsMenuItem<Wry>).collect();
    entries.extend([
        &separator as &dyn IsMenuItem<Wry>,
        &reload,
        &zoom_separator,
        &zoom_in,
        &zoom_out,
        &zoom_reset,
        &memory_separator,
        &memory,
    ]);

    let submenu = Submenu::with_items(app, "Cuentas", true, &entries)?;
    // Place it right before the standard "Window" menu.
    let position = menu.items()?.len().saturating_sub(2);
    menu.insert(&submenu, position)?;
    Ok(menu)
}
