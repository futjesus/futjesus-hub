use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Url, Webview, WebviewBuilder,
    WebviewUrl, WebviewWindowBuilder, Window,
    webview::{DownloadEvent, NewWindowFeatures, NewWindowResponse},
};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;

use crate::{
    menu::Zoom,
    store::{Account, AppState, LinkTarget},
};

pub const SIDEBAR_WIDTH: f64 = 72.0;
const GMAIL_URL: &str = "https://mail.google.com/mail/u/0/";
// Goes straight to Google's sign-in form instead of Gmail's marketing page.
const SIGN_IN_URL: &str = "https://accounts.google.com/ServiceLogin?service=mail&continue=https://mail.google.com/mail/u/0/";
// Google refuses to sign in from user agents it does not recognise as a browser.
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15";
const STATUS_SCRIPT: &str = include_str!("status.js");
const LABEL_PREFIX: &str = "gmail-";

pub fn label(id: &str) -> String {
    format!("{LABEL_PREFIX}{id}")
}

pub fn account_id(label: &str) -> Option<&str> {
    label.strip_prefix(LABEL_PREFIX)
}

fn content_bounds(window: &Window) -> tauri::Result<(LogicalPosition<f64>, LogicalSize<f64>)> {
    let size = window.inner_size()?.to_logical::<f64>(window.scale_factor()?);
    Ok((
        LogicalPosition::new(SIDEBAR_WIDTH, 0.0),
        LogicalSize::new((size.width - SIDEBAR_WIDTH).max(0.0), size.height),
    ))
}

/// Keeps every Gmail webview filling the area to the right of the sidebar.
pub fn layout(window: &Window) {
    let Ok((position, size)) = content_bounds(window) else {
        return;
    };
    for webview in window.webviews() {
        if account_id(webview.label()).is_some() {
            let _ = webview.set_position(position);
            let _ = webview.set_size(size);
        }
    }
}

/// Creates the (hidden) webview of an account. Each account gets its own
/// WebKit data store, so cookies and sessions never mix between accounts.
pub fn open(window: &Window, account: &Account) -> tauri::Result<Webview> {
    let app = window.app_handle().clone();
    let (position, size) = content_bounds(window)?;
    let store_id = Uuid::parse_str(&account.id).unwrap_or_default();
    let start_url = if account.email.is_some() { GMAIL_URL } else { SIGN_IN_URL };

    let builder = WebviewBuilder::new(
        label(&account.id),
        WebviewUrl::External(start_url.parse().expect("valid start URL")),
    )
    .user_agent(USER_AGENT)
    .data_store_identifier(*store_id.as_bytes())
    .on_new_window({
        let app = app.clone();
        let id = account.id.clone();
        move |url, features| {
            if is_sign_in(&url) && !is_signed_in(&app, &id) {
                sign_in_here(&app, &id, url);
                return NewWindowResponse::Deny;
            }
            open_popup(&app, features)
        }
    })
    .on_download(move |_, event| on_download(&app, event));

    let webview = window.add_child(builder, position, size)?;
    webview.hide()?;
    if account.zoom != 1.0 {
        webview.set_zoom(account.zoom)?;
    }
    Ok(webview)
}

fn is_sign_in(url: &Url) -> bool {
    url.host_str() == Some("accounts.google.com")
}

fn is_signed_in(app: &AppHandle, id: &str) -> bool {
    app.state::<AppState>()
        .store
        .lock()
        .unwrap()
        .accounts
        .iter()
        .any(|a| a.id == id && a.email.is_some())
}

/// Google's pages open "Sign in" in a new window; before the account has a
/// session it is far simpler to sign in inside the account's own view.
fn sign_in_here(app: &AppHandle, id: &str, url: Url) {
    let app = app.clone();
    let label = label(id);
    // We're inside a WebKit callback; navigate once it has returned.
    tauri::async_runtime::spawn(async move {
        if let Some(webview) = app.get_webview(&label) {
            let _ = webview.navigate(url);
        }
    });
}

/// Gmail opens most things (links, attachments, print views) with
/// `window.open`, often starting from `about:blank`. The popup is created
/// hidden, sharing the account's data store, and its first real navigation
/// decides whether it stays in the app or goes to the system browser.
fn open_popup(app: &AppHandle, features: NewWindowFeatures) -> NewWindowResponse<tauri::Wry> {
    let label = format!("popup-{}", Uuid::new_v4());
    let shown = AtomicBool::new(false);

    let built = WebviewWindowBuilder::new(
        app,
        &label,
        WebviewUrl::External("about:blank".parse().expect("valid URL")),
    )
    .title("Futjesus Hub")
    .inner_size(1000.0, 760.0)
    .visible(false)
    .user_agent(USER_AGENT)
    .window_features(features)
    .on_document_title_changed(|window, title| {
        let _ = window.set_title(&title);
    })
    .on_navigation({
        let app = app.clone();
        let label = label.clone();
        move |url| route_popup_navigation(&app, &label, &shown, url)
    })
    .on_new_window({
        let app = app.clone();
        move |_, features| open_popup(&app, features)
    })
    .on_download({
        let app = app.clone();
        move |_, event| on_download(&app, event)
    })
    .build();

    match built {
        Ok(window) => NewWindowResponse::Create { window },
        Err(e) => {
            eprintln!("failed to open popup: {e}");
            NewWindowResponse::Deny
        }
    }
}

fn route_popup_navigation(app: &AppHandle, label: &str, shown: &AtomicBool, url: &Url) -> bool {
    if url.scheme() == "about" {
        return true;
    }
    let target = unwrap_google_redirect(url);
    let link_target = app.state::<AppState>().store.lock().unwrap().settings.link_target;
    let app = app.clone();
    let label = label.to_string();

    if link_target == LinkTarget::Browser && !keeps_session(&target) {
        let _ = app.opener().open_url(target.as_str(), None::<&str>);
        // A popup the user never saw only existed to carry this link.
        if !shown.load(Ordering::SeqCst) {
            tauri::async_runtime::spawn(async move {
                if let Some(window) = app.get_webview_window(&label) {
                    let _ = window.close();
                }
            });
        }
        return false;
    }

    if !shown.swap(true, Ordering::SeqCst) {
        tauri::async_runtime::spawn(async move {
            if let Some(window) = app.get_webview_window(&label) {
                let _ = window.show();
                let _ = window.set_focus();
            }
        });
    }
    true
}

/// Gmail routes external links through `https://www.google.com/url?q=...`.
fn unwrap_google_redirect(url: &Url) -> Url {
    let is_redirect = url
        .host_str()
        .is_some_and(|host| host.starts_with("www.google.") || host == "google.com")
        && url.path() == "/url";
    if is_redirect
        && let Some(target) = url
            .query_pairs()
            .find(|(key, _)| key == "q" || key == "url")
            .and_then(|(_, value)| Url::parse(&value).ok())
    {
        return target;
    }
    url.clone()
}

/// Pages that only work with the account's own session (attachments,
/// sign-in), so they always stay inside the app.
fn keeps_session(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        host == "mail.google.com"
            || host == "accounts.google.com"
            || host.ends_with(".googleusercontent.com")
    })
}

fn on_download(app: &AppHandle, event: DownloadEvent<'_>) -> bool {
    if let DownloadEvent::Requested { destination, .. } = event
        && let Ok(dir) = app.path().download_dir()
    {
        let name = destination
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_else(|| "download".into());
        *destination = unique_path(&dir, &name);
    }
    true
}

fn unique_path(dir: &Path, name: &OsStr) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let name = Path::new(name);
    let stem = name.file_stem().unwrap_or_default().to_string_lossy();
    let ext = name
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    (1..)
        .map(|i| dir.join(format!("{stem} ({i}){ext}")))
        .find(|path| !path.exists())
        .expect("an unused file name")
}

const ZOOM_STEP: f64 = 0.1;
const ZOOM_RANGE: (f64, f64) = (0.5, 2.0);

/// Zooms the visible account and remembers the level for next launch.
pub fn zoom_active(app: &AppHandle, action: Zoom) {
    let state = app.state::<AppState>();
    let mut store = state.store.lock().unwrap();
    let Some(id) = store.active.clone() else {
        return;
    };
    let Some(account) = store.accounts.iter_mut().find(|a| a.id == id) else {
        return;
    };
    let zoom = match action {
        Zoom::In => account.zoom + ZOOM_STEP,
        Zoom::Out => account.zoom - ZOOM_STEP,
        Zoom::Reset => 1.0,
    };
    // Round to avoid drifting to values like 0.7999999.
    account.zoom = ((zoom * 10.0).round() / 10.0).clamp(ZOOM_RANGE.0, ZOOM_RANGE.1);
    if let Some(webview) = app.get_webview(&label(&id)) {
        let _ = webview.set_zoom(account.zoom);
    }
    state.persist(&store);
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    unread: Option<u32>,
    email: Option<String>,
    avatar: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusEvent {
    id: String,
    #[serde(flatten)]
    status: Status,
}

pub fn start_status_polling(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(5));
            let ids: Vec<String> = app
                .state::<AppState>()
                .store
                .lock()
                .unwrap()
                .accounts
                .iter()
                .map(|account| account.id.clone())
                .collect();
            // Re-applied every cycle so a stale Dock badge never lingers.
            update_badge(&app);
            for id in ids {
                let Some(webview) = app.get_webview(&label(&id)) else {
                    continue;
                };
                let app = app.clone();
                let _ = webview.eval_with_callback(STATUS_SCRIPT, move |raw| {
                    if let Some(status) = parse_status(&raw) {
                        apply_status(&app, id.clone(), status);
                    }
                });
            }
        }
    });
}

/// The script returns a JSON string, which may arrive encoded once more.
fn parse_status(raw: &str) -> Option<Status> {
    serde_json::from_str::<Status>(raw).ok().or_else(|| {
        serde_json::from_str::<String>(raw)
            .ok()
            .and_then(|inner| serde_json::from_str(&inner).ok())
    })
}

fn apply_status(app: &AppHandle, id: String, status: Status) {
    let state = app.state::<AppState>();

    {
        let mut store = state.store.lock().unwrap();
        if let Some(account) = store.accounts.iter_mut().find(|a| a.id == id) {
            let mut changed = false;
            if status.email.is_some() && account.email != status.email {
                account.email = status.email.clone();
                changed = true;
            }
            if status.avatar.is_some() && account.avatar != status.avatar {
                account.avatar = status.avatar.clone();
                changed = true;
            }
            if changed {
                state.persist(&store);
            }
        }
    }

    if let Some(unread) = status.unread {
        state.unread.lock().unwrap().insert(id.clone(), unread);
    }

    let _ = app.emit(
        "account-status",
        StatusEvent { id, status },
    );
}

pub fn update_badge(app: &AppHandle) {
    let total: u32 = app.state::<AppState>().unread.lock().unwrap().values().sum();
    if let Some(window) = app.get_window("main") {
        let _ = window.set_badge_count((total > 0).then_some(i64::from(total)));
    }
}
