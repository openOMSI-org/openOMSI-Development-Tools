//! Finding the things the tools depend on without asking the person to type paths: `cargo`
//! (and whether it can build WebAssembly) and openOMSI's folder.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

fn exe(name: &str) -> String {
    if cfg!(windows) { format!("{name}.exe") } else { name.to_string() }
}

/// Where `cargo` may be: the one set in Settings, the PATH, rustup's folder.
fn cargo_candidates(configured: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !configured.trim().is_empty() {
        let p = PathBuf::from(configured.trim());
        // (a folder is taken to be the one `cargo` lives in)
        out.push(if p.is_dir() { p.join(exe("cargo")) } else { p });
    }
    if let Some(path) = std::env::var_os("PATH") {
        out.extend(std::env::split_paths(&path).map(|d| d.join(exe("cargo"))));
    }
    if let Some(home) = std::env::var_os("CARGO_HOME") {
        out.push(PathBuf::from(home).join("bin").join(exe("cargo")));
    }
    if let Some(home) = dirs::home_dir() {
        out.push(home.join(".cargo").join("bin").join(exe("cargo")));
    }
    #[cfg(target_os = "macos")]
    {
        out.push(PathBuf::from("/opt/homebrew/bin/cargo"));
        out.push(PathBuf::from("/usr/local/bin/cargo"));
    }
    out
}

/// What is known about the Rust toolchain.
#[derive(Debug, Clone, Serialize, Default)]
pub struct CargoInfo {
    /// A working `cargo` was found.
    pub found: bool,
    /// Its path.
    pub path: String,
    /// `cargo 1.97.1 (...)`.
    pub version: String,
    /// The `wasm32-unknown-unknown` target is installed (Rust plugins need it).
    pub wasm_target: bool,
    /// `rustup` is there to install the target.
    pub rustup: bool,
}

fn run(program: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// The first working `cargo` of the candidates, with the target check.
pub fn cargo_info(configured: &str) -> CargoInfo {
    for candidate in cargo_candidates(configured) {
        if !candidate.is_file() {
            continue;
        }
        let Some(version) = run(&candidate, &["--version"]) else { continue };
        let bin = candidate.parent().map(Path::to_path_buf).unwrap_or_default();
        let rustup_path = bin.join(exe("rustup"));
        let rustup = rustup_path.is_file() || run(Path::new("rustup"), &["--version"]).is_some();
        let rustc = if bin.join(exe("rustc")).is_file() { bin.join(exe("rustc")) } else { PathBuf::from("rustc") };
        // (the sysroot of the toolchain cargo belongs to holds the targets it can build)
        let wasm_target = run(&rustc, &["--print", "sysroot"])
            .map(|root| Path::new(&root).join("lib").join("rustlib").join("wasm32-unknown-unknown").is_dir())
            .unwrap_or(false);
        return CargoInfo { found: true, path: candidate.to_string_lossy().into_owned(), version, wasm_target, rustup };
    }
    CargoInfo::default()
}

/// `rustup target add wasm32-unknown-unknown`, with every output line passed to `log`.
pub fn install_wasm_target(cargo: &str, log: &dyn Fn(&str)) -> Result<(), String> {
    let info = cargo_info(cargo);
    let bin = Path::new(&info.path).parent().map(Path::to_path_buf).unwrap_or_default();
    let rustup = if bin.join(exe("rustup")).is_file() { bin.join(exe("rustup")) } else { PathBuf::from("rustup") };
    let out = Command::new(&rustup)
        .args(["target", "add", "wasm32-unknown-unknown"])
        .output()
        .map_err(|e| format!("cannot run rustup: {e}. Install Rust from rustup.rs first."))?;
    for line in String::from_utf8_lossy(&out.stderr).lines().chain(String::from_utf8_lossy(&out.stdout).lines()) {
        log(line);
    }
    if out.status.success() { Ok(()) } else { Err("rustup could not add the target".to_string()) }
}

/// What the app knows about openOMSI's folder.
#[derive(Debug, Clone, Serialize, Default)]
pub struct GameInfo {
    /// The folder is openOMSI's.
    pub ok: bool,
    /// The folder (the one that holds `plugins`).
    pub folder: String,
    /// What starts the game: the `.app`, or the program.
    pub program: String,
    /// The `plugins` folder (it may not exist yet).
    pub plugins: String,
    /// Why not, when it is not.
    pub problem: String,
}

const MARKERS: [&str; 2] = [".openomsi-content", ".omsi-rewrite-content"];

/// The program that starts the game in `folder`.
fn game_program(folder: &Path) -> Option<PathBuf> {
    for name in ["openOMSI.app", "openomsi.app"] {
        let p = folder.join(name);
        if p.is_dir() {
            return Some(p);
        }
    }
    for name in [exe("openomsi"), exe("openOMSI")] {
        let p = folder.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Whether `folder` is openOMSI's (it holds the program or the content marker).
fn is_game_folder(folder: &Path) -> bool {
    game_program(folder).is_some() || MARKERS.iter().any(|m| folder.join(m).exists())
}

/// The `plugins` folder of `folder`, as the game names it (any case).
fn plugins_dir(folder: &Path) -> PathBuf {
    if let Ok(entries) = std::fs::read_dir(folder) {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy().eq_ignore_ascii_case("plugins") && e.path().is_dir() {
                return e.path();
            }
        }
    }
    folder.join("plugins")
}

/// What `chosen` is: openOMSI's folder, its `.app`, or the program inside it.
pub fn game_info(chosen: &str) -> GameInfo {
    let chosen = chosen.trim();
    if chosen.is_empty() {
        return GameInfo { problem: "No folder chosen".into(), ..Default::default() };
    }
    let mut path = PathBuf::from(chosen);
    if !path.exists() {
        return GameInfo { problem: format!("{} is not there", path.display()), ..Default::default() };
    }
    // (an .app, or anything inside one: the content folder is the one beside the bundle)
    for anc in path.clone().ancestors() {
        if anc.extension().is_some_and(|e| e == "app") {
            path = anc.parent().map(Path::to_path_buf).unwrap_or(path);
            break;
        }
    }
    if path.is_file() {
        path = path.parent().map(Path::to_path_buf).unwrap_or(path);
    }
    if !is_game_folder(&path) {
        return GameInfo {
            folder: path.to_string_lossy().into_owned(),
            problem: "openOMSI's program is not in this folder".into(),
            ..Default::default()
        };
    }
    GameInfo {
        ok: true,
        folder: path.to_string_lossy().into_owned(),
        program: game_program(&path).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
        plugins: plugins_dir(&path).to_string_lossy().into_owned(),
        problem: String::new(),
    }
}

/// Folders openOMSI is commonly in, the first that holds the game.
pub fn find_game() -> Option<GameInfo> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(v) = std::env::var_os("OPENOMSI_GAME") {
        roots.push(PathBuf::from(v));
    }
    if let Some(home) = dirs::home_dir() {
        for sub in [
            "openOMSI",
            "Applications/openOMSI",
            "Games/openOMSI",
            "Desktop/openOMSI",
            "Downloads/openOMSI",
            ".local/share/openOMSI",
        ] {
            roots.push(home.join(sub));
        }
        // (an unpacked download: Downloads/openOMSI-0.3.12-macos-arm64 and so on)
        for parent in [home.join("Downloads"), home.join("Desktop"), home.join("Applications")] {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for e in entries.flatten() {
                    if e.file_name().to_string_lossy().to_lowercase().starts_with("openomsi") && e.path().is_dir() {
                        roots.push(e.path());
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    roots.push(PathBuf::from("/Applications/openOMSI"));
    #[cfg(windows)]
    {
        for var in ["LOCALAPPDATA", "ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(v) = std::env::var_os(var) {
                roots.push(PathBuf::from(v).join("openOMSI"));
            }
        }
    }
    roots.into_iter().map(|r| game_info(&r.to_string_lossy())).find(|g| g.ok)
}

/// Starts the game from `info`.
pub fn launch(info: &GameInfo) -> Result<(), String> {
    if !info.ok {
        return Err("openOMSI's folder is not set (Settings)".into());
    }
    let program = Path::new(&info.program);
    if program.as_os_str().is_empty() {
        return Err("openOMSI's program is not in the folder".into());
    }
    #[cfg(target_os = "macos")]
    let result = Command::new("open").arg(program).spawn();
    #[cfg(not(target_os = "macos"))]
    let result = Command::new(program).current_dir(&info.folder).spawn();
    result.map(|_| ()).map_err(|e| format!("cannot start {}: {e}", program.display()))
}

/// The newest game log of the user's openOMSI data folder, as lines (the last `n`).
pub fn game_log(n: usize) -> Vec<String> {
    let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from) else {
        return Vec::new();
    };
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for dir in [".openomsi", ".omsi-rewrite"] {
        let Ok(entries) = std::fs::read_dir(home.join(dir)) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if !(name == "game.log" || (name.starts_with("game-") && name.ends_with(".log"))) {
                continue;
            }
            if let Ok(modified) = e.metadata().and_then(|m| m.modified())
                && best.as_ref().is_none_or(|(t, _)| modified > *t)
            {
                best = Some((modified, e.path()));
            }
        }
    }
    let Some((_, path)) = best else { return Vec::new() };
    // (the end of the file only: a long session's log is large)
    let Ok(mut f) = std::fs::File::open(&path) else { return Vec::new() };
    use std::io::{Read, Seek, SeekFrom};
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let from = len.saturating_sub(256 * 1024);
    let _ = f.seek(SeekFrom::Start(from));
    let mut bytes = Vec::new();
    let _ = f.read_to_end(&mut bytes);
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let skip = lines.len().saturating_sub(n);
    lines[skip..].iter().map(|l| l.to_string()).collect()
}
