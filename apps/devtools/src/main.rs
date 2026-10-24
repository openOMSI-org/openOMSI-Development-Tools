//! openOMSI Development Tools: a Tauri 2 window (the page in `ui/`) over the plugin compiler
//! (`oopc`), the signing keys and the game's `plugins` folder.
//!
//! Everything the page can do is a command here; the page never touches the disk itself.
//! `OPENOMSI_DEVTOOLS_SELFTEST=1` starts the window hidden, lets the page check itself against
//! the real commands (a project made, built, signed, installed, inspected) and ends with its
//! report, so a build can be tried without a window on the screen.

// a window, not a console program, on Windows
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod detect;
mod project;
mod settings;
mod tools;
mod updater;

use std::path::{Path, PathBuf};

use oopc::build::BuildOptions;
use oopc::lint::{Problem, Severity};
use oopc::manifest::{Manifest, ProjectKind};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, State, WebviewUrl, WebviewWindowBuilder};

use settings::{Settings, Shared};

type Answer<T> = Result<T, String>;

fn selftest() -> bool {
    std::env::var_os("OPENOMSI_DEVTOOLS_SELFTEST").is_some_and(|v| !v.is_empty() && v != "0")
}

fn log_line(app: &AppHandle, level: &str, text: &str) {
    let _ = app.emit("build-log", json!({"level": level, "text": text}));
}

// --- the start -------------------------------------------------------------------------

#[tauri::command(async)]
fn boot(state: State<Shared>) -> Value {
    let s = state.get();
    // (a game folder not set yet is looked for once, so that Run works the first time)
    let mut game = detect::game_info(&s.game_folder);
    if !game.ok
        && s.game_folder.trim().is_empty()
        && let Some(found) = detect::find_game()
    {
        let _ = state.update(|s| s.game_folder = found.folder.clone());
        game = found;
    }
    let s = state.get();
    let recent: Vec<_> =
        s.recent.iter().filter(|d| Path::new(d).is_dir()).map(|d| project::summarize(Path::new(d))).collect();
    json!({
        "version": updater::current(),
        "os": std::env::consts::OS,
        "dev_build": updater::is_dev_build(),
        "exe": std::env::current_exe().ok().map(|p| p.to_string_lossy().into_owned()),
        "updates_disabled": updater::disabled(),
        "selftest_install": std::env::var_os("OPENOMSI_DEVTOOLS_SELFTEST_INSTALL").is_some_and(|v| !v.is_empty() && v != "0"),
        "selftest": selftest().then(|| std::env::var("OPENOMSI_DEVTOOLS_SELFTEST").unwrap_or_default()),
        "settings": s,
        "recent": recent,
        "keys": tools::list_keys(),
        "cargo": detect::cargo_info(&s.cargo_path),
        "game": game,
        "home": dirs::home_dir().map(|p| p.to_string_lossy().into_owned()),
        "documents": dirs::document_dir().map(|p| p.to_string_lossy().into_owned()),
        "api_version": oopc::util::api_version(),
    })
}

#[tauri::command(async)]
fn save_settings(state: State<Shared>, patch: Value) -> Answer<Settings> {
    state.update(|s| {
        let mut now = serde_json::to_value(&*s).unwrap_or(Value::Null);
        if let (Some(now), Some(patch)) = (now.as_object_mut(), patch.as_object()) {
            for (k, v) in patch {
                now.insert(k.clone(), v.clone());
            }
        }
        if let Ok(next) = serde_json::from_value::<Settings>(now) {
            *s = next;
        }
    })
}

// --- the system ------------------------------------------------------------------------

/// A folder picker (the window's thread: these commands are not async).
#[tauri::command]
fn pick_folder(title: String, start: Option<String>) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(&title);
    if let Some(dir) = start.filter(|s| Path::new(s).is_dir()) {
        d = d.set_directory(dir);
    }
    d.pick_folder().map(|p| p.to_string_lossy().into_owned())
}

/// A file picker; `extensions` limits it (`["oop"]`).
#[tauri::command]
fn pick_file(title: String, extensions: Vec<String>, start: Option<String>) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(&title);
    if !extensions.is_empty() {
        let exts: Vec<&str> = extensions.iter().map(String::as_str).collect();
        d = d.add_filter("Files", &exts);
    }
    if let Some(dir) = start.filter(|s| Path::new(s).is_dir()) {
        d = d.set_directory(dir);
    }
    d.pick_file().map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn pick_save(title: String, name: String, start: Option<String>) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(&title).set_file_name(&name);
    if let Some(dir) = start.filter(|s| Path::new(s).is_dir()) {
        d = d.set_directory(dir);
    }
    d.save_file().map(|p| p.to_string_lossy().into_owned())
}

fn open_with_system(what: &std::ffi::OsStr) -> Answer<()> {
    #[cfg(target_os = "macos")]
    let mut c = std::process::Command::new("open");
    #[cfg(windows)]
    let mut c = {
        let mut c = std::process::Command::new("explorer");
        c.arg("/n,");
        c
    };
    #[cfg(not(any(target_os = "macos", windows)))]
    let mut c = std::process::Command::new("xdg-open");
    c.arg(what).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Shows a file in the system's file manager (a folder is opened).
#[tauri::command(async)]
fn reveal(path: String) -> Answer<()> {
    let p = PathBuf::from(&path);
    if !p.exists() {
        return Err(format!("{path} is not there"));
    }
    #[cfg(target_os = "macos")]
    if p.is_file() {
        return std::process::Command::new("open").arg("-R").arg(&p).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    #[cfg(windows)]
    if p.is_file() {
        return std::process::Command::new("explorer")
            .arg(format!("/select,{}", p.display()))
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string());
    }
    let folder = if p.is_file() { p.parent().map(Path::to_path_buf).unwrap_or(p) } else { p };
    open_with_system(folder.as_os_str())
}

/// A web page in the browser (only http and https).
#[tauri::command(async)]
fn open_url(url: String) -> Answer<()> {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("not a web address".into());
    }
    open_with_system(std::ffi::OsStr::new(&url))
}

// --- projects --------------------------------------------------------------------------

fn dir_of(dir: &str) -> PathBuf {
    PathBuf::from(dir)
}

#[tauri::command(async)]
fn project_open(state: State<Shared>, dir: String) -> Answer<Value> {
    let path = dir_of(&dir);
    if !path.join("openomsi-plugin.toml").is_file() {
        return Err(format!(
            "{dir} is not a plugin project: it has no openomsi-plugin.toml. Open the folder that holds it, or make a new plugin."
        ));
    }
    let summary = project::summarize(&path);
    if let Some(e) = &summary.error {
        return Err(e.clone());
    }
    let manifest = project::load_manifest(&path)?;
    let _ = state.update(|s| s.remember_project(&dir));
    Ok(json!({"summary": summary, "tree": project::tree(&path), "manifest": manifest}))
}

#[tauri::command(async)]
fn project_create(state: State<Shared>, req: project::NewRequest) -> Answer<String> {
    let dir = project::create(&req)?;
    let dir = dir.to_string_lossy().into_owned();
    let _ = state.update(|s| {
        if s.author.is_empty() {
            s.author = req.author.clone();
        }
        s.projects_folder = req.folder.clone();
    });
    Ok(dir)
}

/// The id a new plugin gets from its author and name.
#[tauri::command]
fn make_id(author: String, name: String) -> String {
    oopc::templates::make_id(&author, &name)
}

#[tauri::command(async)]
fn project_tree(dir: String) -> Vec<project::Node> {
    project::tree(&dir_of(&dir))
}

#[tauri::command(async)]
fn file_read(dir: String, path: String) -> Answer<project::FileText> {
    project::read_file(&dir_of(&dir), &path)
}

#[tauri::command(async)]
fn file_write(dir: String, path: String, text: String) -> Answer<()> {
    project::write_file(&dir_of(&dir), &path, &text)
}

#[tauri::command(async)]
fn file_create(dir: String, path: String, folder: bool) -> Answer<()> {
    project::create_entry(&dir_of(&dir), &path, folder)
}

#[tauri::command(async)]
fn file_rename(dir: String, from: String, to: String) -> Answer<()> {
    project::rename_entry(&dir_of(&dir), &from, &to)
}

#[tauri::command(async)]
fn file_delete(dir: String, path: String) -> Answer<()> {
    project::delete_entry(&dir_of(&dir), &path)
}

/// Copies a file from outside the project into it (an image or a sound for `assets/`).
#[tauri::command(async)]
fn file_import(dir: String, from: String, to_dir: String) -> Answer<String> {
    let name = Path::new(&from).file_name().ok_or("no file name")?.to_string_lossy().into_owned();
    let rel = if to_dir.is_empty() { name } else { format!("{}/{name}", to_dir.trim_end_matches('/')) };
    let dest = project::safe_join(&dir_of(&dir), &rel)?;
    if dest.exists() {
        return Err(format!("{rel} is already there"));
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&from, &dest).map_err(|e| format!("cannot copy {from}: {e}"))?;
    Ok(rel)
}

#[tauri::command(async)]
fn manifest_save(dir: String, manifest: Manifest) -> Answer<()> {
    project::save_manifest(&dir_of(&dir), &manifest)
}

// --- checking --------------------------------------------------------------------------

#[tauri::command(async)]
fn check_project(dir: String) -> Vec<Problem> {
    oopc::lint::check_project(&dir_of(&dir))
}

/// The syntax problems of one Lua text as it is in the editor (not yet saved).
#[tauri::command(async)]
fn check_lua(file: String, text: String) -> Vec<Problem> {
    oopc::lint::lua_syntax(&file, &text)
}

/// Permissions the code of a Lua project needs: `{permission: first function that needs it}`.
#[tauri::command(async)]
fn needed_permissions(dir: String) -> std::collections::BTreeMap<String, String> {
    oopc::lint::needed_permissions(&dir_of(&dir))
}

// --- building --------------------------------------------------------------------------

#[derive(Serialize)]
struct BuildResult {
    out: String,
    size: u64,
    notes: Vec<String>,
    signed_by: Option<String>,
    ms: u128,
}

fn cargo_for(settings: &Settings) -> Option<PathBuf> {
    let info = detect::cargo_info(&settings.cargo_path);
    info.found.then(|| PathBuf::from(info.path))
}

fn build_oop(app: &AppHandle, settings: &Settings, dir: &Path, sign: bool, out: Option<&Path>) -> Answer<BuildResult> {
    let started = std::time::Instant::now();
    let manifest = Manifest::load(dir)?;
    let header = manifest.to_header();
    header.validate().map_err(|e| format!("The manifest is not valid: {e}"))?;
    let errors: Vec<Problem> =
        oopc::lint::check_project(dir).into_iter().filter(|p| p.severity == Severity::Error).collect();
    if let Some(first) = errors.first() {
        let place = if first.file.is_empty() { String::new() } else { format!("{}:{}: ", first.file, first.line) };
        return Err(format!("{} error(s) to fix first. {place}{}", errors.len(), first.message));
    }
    let key = if sign {
        let path = settings.default_key.trim();
        if path.is_empty() || !Path::new(path).is_file() {
            return Err("Signing is on but there is no signing key. Make one under Signing keys.".into());
        }
        let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read the key: {e}"))?;
        Some(oop_format::decode_secret_key(&text).map_err(|e| e.to_string())?)
    } else {
        None
    };
    log_line(
        app,
        "info",
        &format!(
            "Building {} {} ({})",
            manifest.plugin.name,
            manifest.plugin.version,
            if manifest.plugin.kind == ProjectKind::Rust { "Rust" } else { "Lua" }
        ),
    );
    let cargo = cargo_for(settings);
    let log = |line: &str| log_line(app, "dim", line);
    let opts = BuildOptions { cargo: cargo.as_deref(), log: Some(&log) };
    let built = oopc::build::compile_with(dir, &manifest, &opts)?;
    for n in &built.notes {
        log_line(app, "info", n);
    }
    let mut builder = oop_format::OopBuilder::new(header.clone()).files(built.files);
    if let Some(k) = &key {
        builder = builder.sign(k);
    }
    let bytes = builder.write().map_err(|e| format!("packing failed: {e}"))?;
    let out_path = out.map(Path::to_path_buf).unwrap_or_else(|| dir.join("dist").join(format!("{}.oop", header.id)));
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&out_path, &bytes).map_err(|e| format!("cannot write {}: {e}", out_path.display()))?;
    let signed_by = key.as_ref().map(|k| oop_format::fingerprint(&k.verifying_key()));
    log_line(app, "good", &format!("Built {} ({} KiB)", out_path.display(), bytes.len().div_ceil(1024)));
    if let Some(fp) = &signed_by {
        log_line(app, "info", &format!("Signed by {fp}"));
    }
    Ok(BuildResult {
        out: out_path.to_string_lossy().into_owned(),
        size: bytes.len() as u64,
        notes: built.notes,
        signed_by,
        ms: started.elapsed().as_millis(),
    })
}

#[tauri::command(async)]
fn build(app: AppHandle, state: State<Shared>, dir: String, sign: bool, out: Option<String>) -> Answer<BuildResult> {
    let r = build_oop(&app, &state.get(), &dir_of(&dir), sign, out.as_deref().map(Path::new));
    if let Err(e) = &r {
        log_line(&app, "error", e);
    }
    r
}

#[derive(Serialize)]
struct RunResult {
    /// `folder` (a Lua plugin copied as it is) or `oop` (built and installed).
    mode: &'static str,
    target: String,
    changed: usize,
    launched: bool,
}

/// Puts the plugin where the game finds it and, when asked, starts the game.
#[tauri::command(async)]
fn run_in_game(app: AppHandle, state: State<Shared>, dir: String, launch: bool) -> Answer<RunResult> {
    let settings = state.get();
    let game = detect::game_info(&settings.game_folder);
    if !game.ok {
        return Err(format!(
            "needs-game: {}",
            if game.problem.is_empty() { "openOMSI's folder is not set" } else { &game.problem }
        ));
    }
    let dir = dir_of(&dir);
    let manifest = Manifest::load(&dir)?;
    let plugins = PathBuf::from(&game.plugins);
    let mut result = if manifest.plugin.kind == ProjectKind::Lua {
        let errors = oopc::lint::check_project(&dir).into_iter().filter(|p| p.severity == Severity::Error).count();
        if errors > 0 {
            return Err(format!("{errors} error(s) to fix first (see Problems)"));
        }
        let changed = project::sync_to_game(&dir, &plugins, &manifest)?;
        let target = plugins.join(&manifest.plugin.id);
        log_line(&app, "good", &format!("{} file(s) updated in {}", changed, target.display()));
        RunResult { mode: "folder", target: target.to_string_lossy().into_owned(), changed, launched: false }
    } else {
        std::fs::create_dir_all(&plugins).map_err(|e| format!("cannot make {}: {e}", plugins.display()))?;
        let target = plugins.join(format!("{}.oop", manifest.plugin.id));
        let r = build_oop(&app, &settings, &dir, settings.sign_builds, Some(&target))
            .inspect_err(|e| log_line(&app, "error", e))?;
        log_line(&app, "good", &format!("Installed {}", r.out));
        RunResult { mode: "oop", target: r.out, changed: 1, launched: false }
    };
    if launch {
        detect::launch(&game)?;
        result.launched = true;
        log_line(&app, "info", "openOMSI started. Start a drive to see the plugin.");
    }
    Ok(result)
}

#[tauri::command(async)]
fn launch_game(state: State<Shared>) -> Answer<()> {
    detect::launch(&detect::game_info(&state.get().game_folder))
}

#[tauri::command(async)]
fn game_info(path: String) -> detect::GameInfo {
    detect::game_info(&path)
}

#[tauri::command(async)]
fn find_game() -> Option<detect::GameInfo> {
    detect::find_game()
}

#[tauri::command(async)]
fn game_log(lines: usize) -> Vec<String> {
    detect::game_log(lines.clamp(1, 2000))
}

// --- the toolchain ---------------------------------------------------------------------

#[tauri::command(async)]
fn cargo_info(path: String) -> detect::CargoInfo {
    detect::cargo_info(&path)
}

#[tauri::command(async)]
fn install_wasm_target(app: AppHandle, state: State<Shared>) -> Answer<detect::CargoInfo> {
    let cargo = state.get().cargo_path;
    detect::install_wasm_target(&cargo, &|l| log_line(&app, "dim", l))?;
    Ok(detect::cargo_info(&cargo))
}

// --- keys and files --------------------------------------------------------------------

#[tauri::command(async)]
fn keys_list() -> Vec<tools::KeyInfo> {
    tools::list_keys()
}

#[tauri::command(async)]
fn key_create(state: State<Shared>, label: String) -> Answer<tools::KeyInfo> {
    let k = tools::create_key(&label)?;
    // (the first key is the one builds are signed with)
    if state.get().default_key.is_empty() {
        let _ = state.update(|s| s.default_key = k.path.clone());
    }
    Ok(k)
}

#[tauri::command(async)]
fn key_import(state: State<Shared>, path: String) -> Answer<tools::KeyInfo> {
    let k = tools::import_key(&path)?;
    if state.get().default_key.is_empty() {
        let _ = state.update(|s| s.default_key = k.path.clone());
    }
    Ok(k)
}

#[tauri::command(async)]
fn key_delete(state: State<Shared>, path: String) -> Answer<()> {
    tools::delete_key(&path)?;
    if state.get().default_key == path {
        let _ = state.update(|s| {
            s.default_key = String::new();
            s.sign_builds = false;
        });
    }
    Ok(())
}

#[tauri::command(async)]
fn key_public(path: String) -> Answer<String> {
    tools::public_key_text(&path)
}

#[tauri::command(async)]
fn write_text(path: String, text: String) -> Answer<()> {
    std::fs::write(&path, text).map_err(|e| format!("cannot write {path}: {e}"))
}

#[tauri::command(async)]
fn inspect(path: String) -> Answer<Value> {
    tools::inspect(&path)
}

#[tauri::command(async)]
fn sign_file(file: String, key: String) -> Answer<String> {
    tools::sign(&file, &key)
}

#[tauri::command(async)]
fn docs() -> Value {
    tools::docs()
}

// --- updates ---------------------------------------------------------------------------

#[tauri::command(async)]
fn update_check() -> Answer<Option<updater::Release>> {
    if updater::disabled() {
        return Ok(None);
    }
    updater::check()
}

#[tauri::command(async)]
fn update_install(app: AppHandle, release: updater::Release) -> Answer<()> {
    updater::install(&release, &|done, total| {
        let _ = app.emit("update-progress", json!({"done": done, "total": total}));
    })
}

#[tauri::command(async)]
fn update_relaunch(app: AppHandle) -> Answer<()> {
    updater::relaunch()?;
    app.exit(0);
    Ok(())
}

/// A line of the page's self-check (stderr), so that a check that hangs is found.
#[tauri::command]
fn selftest_log(line: String) {
    if selftest() {
        eprintln!("selftest: {line}");
    }
}

/// The page's self-check is over: its report on stdout, and the program ends (failed checks:
/// exit code 1).
#[tauri::command]
fn selftest_done(app: AppHandle, report: Value) {
    if !selftest() {
        return;
    }
    println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
    let ok = report.get("failed").and_then(|f| f.as_array()).is_some_and(|f| f.is_empty());
    app.exit(if ok { 0 } else { 1 });
}

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let at = info.location().map(|l| format!(" at {}:{}", l.file(), l.line())).unwrap_or_default();
        if let Ok(dir) = oopc::util::config_dir() {
            let _ = std::fs::create_dir_all(&dir);
            let line = format!("panic{at}: {info}\n");
            let _ = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("crash.log"))
                .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
        }
        default_hook(info);
    }));
    updater::cleanup();

    let result = tauri::Builder::default()
        .manage(Shared::new())
        .invoke_handler(tauri::generate_handler![
            boot,
            save_settings,
            pick_folder,
            pick_file,
            pick_save,
            reveal,
            open_url,
            project_open,
            project_create,
            make_id,
            project_tree,
            file_read,
            file_write,
            file_create,
            file_rename,
            file_delete,
            file_import,
            manifest_save,
            check_project,
            check_lua,
            needed_permissions,
            build,
            run_in_game,
            launch_game,
            game_info,
            find_game,
            game_log,
            cargo_info,
            install_wasm_target,
            keys_list,
            key_create,
            key_import,
            key_delete,
            key_public,
            write_text,
            inspect,
            sign_file,
            docs,
            update_check,
            update_install,
            update_relaunch,
            selftest_log,
            selftest_done
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            if selftest() {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("openOMSI Dev Tools")
                .inner_size(1360.0, 860.0)
                .min_inner_size(1000.0, 640.0)
                .center()
                .prevent_overflow()
                // (the page's own background: no white window before the first picture)
                .background_color(tauri::webview::Color(23, 23, 23, 255))
                .visible(false)
                .show_when_loaded(!selftest())
                // (a build running behind another window, or a hidden check, is not slowed down)
                .background_throttling(tauri::utils::config::BackgroundThrottlingPolicy::Disabled)
                .on_page_load(|_, p| {
                    if selftest() {
                        eprintln!("page {:?}: {}", p.event(), p.url());
                    }
                })
                .app_mode(true)
                .build()?;
            if let Some(icon) = app.default_window_icon().cloned() {
                let _ = window.set_icon(icon);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .map(|app| app.run(|_, _| {}));

    if let Err(e) = result {
        eprintln!("the window could not open: {e}");
        std::process::exit(1);
    }
}
