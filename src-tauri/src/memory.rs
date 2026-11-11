use std::{sync::mpsc, time::Duration};

use serde::Serialize;
use tauri::{AppHandle, Manager, Webview};

use crate::{gmail, store::AppState};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntry {
    /// Account id, when the entry belongs to an account.
    id: Option<String>,
    label: String,
    bytes: u64,
}

#[derive(Serialize)]
pub struct MemoryReport {
    entries: Vec<MemoryEntry>,
    total: u64,
}

/// Memory as reported by Activity Monitor's "Memory" column.
fn footprint(pid: i32) -> Option<u64> {
    let mut info: libc::rusage_info_v2 = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::proc_pid_rusage(
            pid,
            libc::RUSAGE_INFO_V2,
            &mut info as *mut _ as *mut libc::rusage_info_t,
        )
    };
    (result == 0).then_some(info.ri_phys_footprint)
}

/// Each WKWebView renders in its own WebContent process, outside the app's
/// process, so its memory has to be read from that process.
fn web_process_id(webview: &Webview) -> Option<i32> {
    let (tx, rx) = mpsc::channel();
    webview
        .with_webview(move |platform| {
            let view = platform.inner() as *const objc2::runtime::AnyObject;
            // Private but long-standing WKWebView API.
            let pid: i32 = unsafe { objc2::msg_send![&*view, _webProcessIdentifier] };
            let _ = tx.send(pid);
        })
        .ok()?;
    rx.recv_timeout(Duration::from_secs(2))
        .ok()
        .filter(|pid| *pid > 0)
}

fn webview_footprint(webview: &Webview) -> Option<u64> {
    web_process_id(webview).and_then(footprint)
}

pub fn report(app: &AppHandle) -> MemoryReport {
    let accounts = app.state::<AppState>().store.lock().unwrap().accounts.clone();
    let mut entries = Vec::new();

    let hibernated = app.state::<AppState>().hibernated.lock().unwrap().clone();
    for account in accounts {
        if hibernated.contains(&account.id) {
            entries.push(MemoryEntry {
                id: Some(account.id),
                label: format!("{} (hibernada)", account.name),
                bytes: 0,
            });
        } else if let Some(bytes) = app
            .get_webview(&gmail::label(&account.id))
            .and_then(|webview| webview_footprint(&webview))
        {
            entries.push(MemoryEntry {
                id: Some(account.id),
                label: account.name,
                bytes,
            });
        }
    }

    let popups: u64 = app
        .webview_windows()
        .iter()
        .filter(|(label, _)| label.starts_with("popup-"))
        .filter_map(|(_, window)| webview_footprint(window.as_ref()))
        .sum();
    if popups > 0 {
        entries.push(MemoryEntry {
            id: None,
            label: "Ventanas emergentes".into(),
            bytes: popups,
        });
    }

    let app_bytes = footprint(std::process::id() as i32).unwrap_or_default()
        + app
            .get_webview("ui")
            .and_then(|webview| webview_footprint(&webview))
            .unwrap_or_default();
    entries.push(MemoryEntry {
        id: None,
        label: "Futjesus Hub (app e interfaz)".into(),
        bytes: app_bytes,
    });

    let total = entries.iter().map(|entry| entry.bytes).sum();
    MemoryReport { entries, total }
}
