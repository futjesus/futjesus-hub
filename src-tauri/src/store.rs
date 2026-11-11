use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: String,
    pub color: String,
    /// Detected from the Gmail session once the user has signed in.
    #[serde(default)]
    pub email: Option<String>,
    /// Profile picture URL taken from the Gmail page.
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default = "default_zoom")]
    pub zoom: f64,
}

fn default_zoom() -> f64 {
    1.0
}

/// Where links clicked inside an email are opened.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkTarget {
    #[default]
    Browser,
    App,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub link_target: LinkTarget,
    pub theme: ThemeMode,
    /// Minutes without being opened before an account is hibernated; `None` disables it.
    pub hibernate_after: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Store {
    pub accounts: Vec<Account>,
    pub settings: Settings,
    pub active: Option<String>,
}

impl Store {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        // Write then rename so a crash never leaves a half-written file behind.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(tmp, path)
    }
}

pub struct AppState {
    pub store: Mutex<Store>,
    /// True while the sidebar UI shows a modal, so Gmail must not cover it.
    pub overlay: Mutex<bool>,
    pub unread: Mutex<HashMap<String, u32>>,
    /// When each account was last shown, for hibernation.
    pub last_active: Mutex<HashMap<String, Instant>>,
    /// Accounts whose webview is closed to save memory.
    pub hibernated: Mutex<HashSet<String>>,
    path: PathBuf,
}

impl AppState {
    pub fn new(store: Store, path: PathBuf) -> Self {
        Self {
            store: Mutex::new(store),
            overlay: Mutex::new(false),
            unread: Mutex::new(HashMap::new()),
            last_active: Mutex::new(HashMap::new()),
            hibernated: Mutex::new(HashSet::new()),
            path,
        }
    }

    pub fn persist(&self, store: &Store) {
        if let Err(e) = store.save(&self.path) {
            eprintln!("failed to save {}: {e}", self.path.display());
        }
    }
}
