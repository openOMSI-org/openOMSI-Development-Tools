//! What the buttons do: thin glue from the UI to the `oopc` library and `oop-format`. Every
//! action reports through the status line and, where it makes sense, the build log; none panics.

use std::path::PathBuf;

use oop_format::{Oop, OopBuilder, fingerprint, inspect_details};
use oopc::manifest::{Manifest, Plugin, ProjectKind};

use crate::state::{AppState, InspectReport, KeyEntry, LogLevel};

fn set_status(state: &mut AppState, msg: impl Into<String>) {
    state.status = msg.into();
}

/// Loads a project's manifest into the form.
pub fn open_project(state: &mut AppState) {
    let Some(dir) = state.project_dir.clone() else { return };
    match Manifest::load(&dir) {
        Ok(m) => {
            fill_form(state, &m);
            if !state.recent.contains(&dir) {
                state.recent.insert(0, dir.clone());
            }
            set_status(state, format!("Opened {}", dir.display()));
        }
        Err(e) => set_status(state, format!("Cannot open: {e}")),
    }
}

fn fill_form(state: &mut AppState, m: &Manifest) {
    let f = &mut state.manifest;
    f.id = m.plugin.id.clone();
    f.name = m.plugin.name.clone();
    f.version = m.plugin.version.clone();
    f.description = m.plugin.description.clone();
    f.authors = m.plugin.authors.join(", ");
    f.is_rust = m.plugin.kind == ProjectKind::Rust;
    f.entry = m.entry();
    f.min_openomsi = m.plugin.min_openomsi.clone().unwrap_or_default();
    f.license = m.plugin.license.clone().unwrap_or_default();
    f.homepage = m.plugin.homepage.clone().unwrap_or_default();
    f.obfuscation_level = m.lua.obfuscation_level;
    for (name, _, on) in &mut f.permissions {
        *on = m.plugin.permissions.iter().any(|p| p == name);
    }
}

fn form_to_manifest(state: &AppState) -> Manifest {
    let f = &state.manifest;
    let permissions = f.permissions.iter().filter(|(_, _, on)| *on).map(|(n, _, _)| n.clone()).collect();
    Manifest {
        plugin: Plugin {
            id: f.id.clone(),
            name: f.name.clone(),
            version: f.version.clone(),
            kind: if f.is_rust { ProjectKind::Rust } else { ProjectKind::Lua },
            entry: if f.is_rust { None } else { Some(f.entry.clone()) },
            authors: f.authors.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            description: f.description.clone(),
            min_openomsi: non_empty(&f.min_openomsi),
            permissions,
            license: non_empty(&f.license),
            homepage: non_empty(&f.homepage),
        },
        lua: oopc::manifest::LuaOptions { obfuscation_level: f.obfuscation_level },
        rust: oopc::manifest::RustOptions::default(),
    }
}

fn non_empty(s: &str) -> Option<String> {
    (!s.trim().is_empty()).then(|| s.trim().to_string())
}

/// Creates a new project and opens it.
pub fn new_project(state: &mut AppState, rust: bool) {
    let base = state.project_dir.clone().unwrap_or_else(|| PathBuf::from("new-plugin"));
    match oopc::commands::new_project(&base, rust) {
        Ok(()) => {
            state.project_dir = Some(base.clone());
            open_project(state);
            set_status(state, format!("Created {}", base.display()));
        }
        Err(e) => set_status(state, format!("Cannot create: {e}")),
    }
}

/// Writes the form back to `openomsi-plugin.toml`.
pub fn save_manifest(state: &mut AppState) {
    let Some(dir) = state.project_dir.clone() else {
        set_status(state, "Open or create a project first");
        return;
    };
    let manifest = form_to_manifest(state);
    match toml::to_string_pretty(&manifest) {
        Ok(text) => match std::fs::write(dir.join("openomsi-plugin.toml"), text) {
            Ok(()) => {
                state.manifest_dirty = false;
                set_status(state, "Manifest saved");
            }
            Err(e) => set_status(state, format!("Cannot write manifest: {e}")),
        },
        Err(e) => set_status(state, format!("Cannot encode manifest: {e}")),
    }
}

/// Builds the current project, filling the log.
pub fn build(state: &mut AppState) {
    let Some(dir) = state.project_dir.clone() else {
        set_status(state, "Open or create a project first");
        return;
    };
    state.building = true;
    state.build_log.clear();
    let manifest = form_to_manifest(state);
    let header = manifest.to_header();
    if let Err(e) = header.validate() {
        state.build_log.push((LogLevel::Error, format!("invalid manifest: {e}")));
        state.building = false;
        return;
    }
    match oopc::build::compile(&dir, &manifest) {
        Ok(built) => {
            for n in &built.notes {
                state.build_log.push((LogLevel::Info, n.clone()));
            }
            let out = dir.join("dist").join(format!("{}.oop", header.id));
            let _ = std::fs::create_dir_all(dir.join("dist"));
            match OopBuilder::new(header).files(built.files).write() {
                Ok(bytes) => match std::fs::write(&out, &bytes) {
                    Ok(()) => {
                        state
                            .build_log
                            .push((LogLevel::Good, format!("built {} ({} KiB)", out.display(), bytes.len() / 1024)));
                        set_status(state, "Build succeeded");
                    }
                    Err(e) => state.build_log.push((LogLevel::Error, format!("cannot write {}: {e}", out.display()))),
                },
                Err(e) => state.build_log.push((LogLevel::Error, format!("packing failed: {e}"))),
            }
        }
        Err(e) => {
            state.build_log.push((LogLevel::Error, e));
            set_status(state, "Build failed");
        }
    }
    state.building = false;
}

/// Builds, then copies the `.oop` into the game's `plugins` folder.
pub fn build_and_install(state: &mut AppState) {
    build(state);
    let Some(dir) = state.project_dir.clone() else { return };
    if state.settings.game_folder.trim().is_empty() {
        state.build_log.push((LogLevel::Warn, "set the game folder in Settings to install".into()));
        return;
    }
    let id = state.manifest.id.clone();
    let src = dir.join("dist").join(format!("{id}.oop"));
    let dest_dir = PathBuf::from(&state.settings.game_folder).join("plugins");
    let _ = std::fs::create_dir_all(&dest_dir);
    let dest = dest_dir.join(format!("{id}.oop"));
    match std::fs::copy(&src, &dest) {
        Ok(_) => state.build_log.push((LogLevel::Good, format!("installed to {}", dest.display()))),
        Err(e) => state.build_log.push((LogLevel::Error, format!("install failed: {e}"))),
    }
}

/// Creates a signing key in the config directory.
pub fn create_key(state: &mut AppState) {
    match oopc::util::keys_dir() {
        Ok(dir) => {
            let safe: String =
                state.new_key_label.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
            let path = dir.join(format!("{safe}.oopkey"));
            match oop_format::generate_key() {
                Ok(key) => {
                    let _ = std::fs::create_dir_all(&dir);
                    let secret = oop_format::encode_secret_key(&key, &state.new_key_label);
                    let public = oop_format::encode_public_key(&key.verifying_key(), &state.new_key_label);
                    if std::fs::write(&path, secret).is_ok() {
                        let _ = std::fs::write(path.with_extension("oopkey.pub"), public);
                        state.keys.push(KeyEntry {
                            label: state.new_key_label.clone(),
                            fingerprint: fingerprint(&key.verifying_key()),
                            path,
                        });
                        set_status(state, "Key created");
                    } else {
                        set_status(state, "Cannot write key file");
                    }
                }
                Err(e) => set_status(state, format!("Cannot make key: {e}")),
            }
        }
        Err(e) => set_status(state, e),
    }
}

/// Imports a key given by path in the label field.
pub fn import_key(state: &mut AppState) {
    let path = PathBuf::from(state.new_key_label.trim());
    match std::fs::read_to_string(&path).ok().and_then(|t| oop_format::decode_public_key(&t).ok()) {
        Some(key) => {
            state.keys.push(KeyEntry {
                label: path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
                fingerprint: fingerprint(&key),
                path,
            });
            set_status(state, "Key imported");
        }
        None => set_status(state, "Give a key file path in the label field to import"),
    }
}

/// Reads a `.oop` into the inspector report.
pub fn inspect(state: &mut AppState) {
    let Some(path) = state.inspector_path.clone() else { return };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            set_status(state, format!("Cannot read: {e}"));
            return;
        }
    };
    match inspect_details(&bytes) {
        Ok(d) => {
            let h = &d.header;
            let mut lines = vec![
                ("Name".into(), format!("{} {}", h.name, h.version)),
                ("Id".into(), h.id.clone()),
                ("Kind".into(), format!("{}   entry: {}", h.kind, h.entry)),
                (
                    "Permissions".into(),
                    if h.permissions.is_empty() { "(none)".into() } else { h.permissions.join(", ") },
                ),
            ];
            if let Some(tool) = &h.tool {
                lines.push(("Built by".into(), tool.clone()));
            }
            let (signature, ok) = match (&d.signer, d.signature_valid) {
                (Some(k), Some(true)) => (format!("valid, signed by {}", fingerprint(k)), true),
                (_, Some(false)) => ("present but INVALID (changed after signing)".into(), false),
                _ => ("none (unsigned)".into(), false),
            };
            let files = match Oop::read(&bytes) {
                Ok(oop) => oop.files().iter().map(|(p, b)| (p.clone(), b.len() as u64)).collect(),
                Err(_) => Vec::new(),
            };
            state.inspector_report = Some(InspectReport { lines, files, signature, ok });
            set_status(state, format!("Inspected {}", path.display()));
        }
        Err(e) => set_status(state, format!("Not a readable .oop: {e}")),
    }
}
