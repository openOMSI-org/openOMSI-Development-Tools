//! The app's settings: a JSON file in the tools' config folder (`settings.json` beside the
//! signing keys). A missing or damaged file gives the defaults; saving never fails the caller.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// `cargo` to build Rust plugins with; empty looks for it by itself.
    pub cargo_path: String,
    /// openOMSI's folder (the one with its `plugins` folder); empty looks for it by itself.
    pub game_folder: String,
    /// `dark`, `light` or `system`.
    pub theme: String,
    /// `ask`, `auto` or `off`: what to do about a new version of these tools.
    pub update_mode: String,
    /// The signing key builds are signed with when signing is on.
    pub default_key: String,
    /// Sign every build with the default key.
    pub sign_builds: bool,
    /// The name new plugins carry as their author.
    pub author: String,
    /// Where new projects are made.
    pub projects_folder: String,
    /// Projects opened lately, newest first.
    pub recent: Vec<String>,
    /// A new version the person said "not now" to.
    pub skipped_version: String,
    /// Keep the game's copy of a Lua plugin up to date on every save.
    pub live_in_game: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            cargo_path: String::new(),
            game_folder: String::new(),
            theme: "dark".into(),
            update_mode: "ask".into(),
            default_key: String::new(),
            sign_builds: false,
            author: String::new(),
            projects_folder: String::new(),
            recent: Vec::new(),
            skipped_version: String::new(),
            live_in_game: true,
        }
    }
}

fn file() -> Result<PathBuf, String> {
    Ok(oopc::util::config_dir()?.join("settings.json"))
}

impl Settings {
    pub fn load() -> Settings {
        file()
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = file()?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
        }
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        // (written beside and renamed: a crash in the middle leaves the old file)
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("cannot write {}: {e}", path.display()))
    }

    /// Puts `dir` first among the recent projects.
    pub fn remember_project(&mut self, dir: &str) {
        self.recent.retain(|d| d != dir);
        self.recent.insert(0, dir.to_string());
        self.recent.truncate(12);
    }
}

/// The settings the commands share.
pub struct Shared(pub Mutex<Settings>);

impl Shared {
    pub fn new() -> Shared {
        Shared(Mutex::new(Settings::load()))
    }

    pub fn get(&self) -> Settings {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Changes the settings with `f` and saves them.
    pub fn update(&self, f: impl FnOnce(&mut Settings)) -> Result<Settings, String> {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard);
        guard.save()?;
        Ok(guard.clone())
    }
}
