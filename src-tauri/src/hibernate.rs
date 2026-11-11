use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{gmail, store::AppState};

const CHECK_EVERY: Duration = Duration::from_secs(30);

#[derive(Clone, Serialize)]
struct HibernationEvent<'a> {
    id: &'a str,
    hibernated: bool,
}

pub fn touch(app: &AppHandle, id: &str) {
    app.state::<AppState>()
        .last_active
        .lock()
        .unwrap()
        .insert(id.to_string(), Instant::now());
}

pub fn is_hibernated(app: &AppHandle, id: &str) -> bool {
    app.state::<AppState>().hibernated.lock().unwrap().contains(id)
}

/// Closes the account's webview, which ends its WebKit process and frees all
/// of its memory. Cookies stay on disk, so waking it keeps the session.
pub fn hibernate(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    if state.store.lock().unwrap().active.as_deref() == Some(id) {
        return;
    }
    let Some(webview) = app.get_webview(&gmail::label(id)) else {
        return;
    };
    if webview.close().is_err() {
        return;
    }
    state.hibernated.lock().unwrap().insert(id.to_string());
    // A hibernated account cannot check for mail, so it stops counting.
    state.unread.lock().unwrap().remove(id);
    gmail::update_badge(app);
    let _ = app.emit("account-hibernated", HibernationEvent { id, hibernated: true });
}

pub fn wake(app: &AppHandle, id: &str) -> tauri::Result<()> {
    let state = app.state::<AppState>();
    if !state.hibernated.lock().unwrap().remove(id) {
        return Ok(());
    }
    let account = state
        .store
        .lock()
        .unwrap()
        .accounts
        .iter()
        .find(|a| a.id == id)
        .cloned();
    if let (Some(account), Some(window)) = (account, app.get_window("main")) {
        gmail::open(&window, &account)?;
    }
    let _ = app.emit("account-hibernated", HibernationEvent { id, hibernated: false });
    Ok(())
}

pub fn start_watch(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(CHECK_EVERY);
            let state = app.state::<AppState>();
            let (minutes, active, ids) = {
                let store = state.store.lock().unwrap();
                (
                    store.settings.hibernate_after,
                    store.active.clone(),
                    store.accounts.iter().map(|a| a.id.clone()).collect::<Vec<_>>(),
                )
            };
            let Some(minutes) = minutes else {
                continue;
            };
            let limit = Duration::from_secs(u64::from(minutes) * 60);
            for id in ids {
                if active.as_deref() == Some(id.as_str()) || is_hibernated(&app, &id) {
                    continue;
                }
                let idle = state
                    .last_active
                    .lock()
                    .unwrap()
                    .entry(id.clone())
                    .or_insert_with(Instant::now)
                    .elapsed();
                if idle >= limit {
                    hibernate(&app, &id);
                }
            }
        }
    });
}
