//! Turning a project's sources into the files that go inside a `.oop`: Lua through the
//! obfuscating compiler, Rust through `cargo` and the WebAssembly stripper. The sources
//! themselves never end up in the archive.

use std::path::{Path, PathBuf};
use std::process::Command;

use lua_obfuscator::{Module, Options};

use crate::manifest::{Manifest, ProjectKind};

/// The files to pack, and a few numbers for the build report.
pub struct Built {
    /// `(path, bytes)` for every file in the archive (entry first).
    pub files: Vec<(String, Vec<u8>)>,
    /// Lines describing what happened, printed by `build`.
    pub notes: Vec<String>,
}

/// Compiles a project (dispatching on its kind).
pub fn compile(dir: &Path, manifest: &Manifest) -> Result<Built, String> {
    match manifest.plugin.kind {
        ProjectKind::Lua => compile_lua(dir, manifest),
        ProjectKind::Rust => compile_rust(dir, manifest),
    }
}

fn is_skipped_dir(name: &str) -> bool {
    matches!(name, "dist" | "target" | ".git" | "node_modules")
}

/// Collects files under `dir` matching `keep`, returning `(relative path with '/', absolute)`.
fn collect(dir: &Path, keep: &dyn Fn(&Path) -> bool) -> Result<Vec<(String, PathBuf)>, String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let entries = std::fs::read_dir(&d).map_err(|e| format!("reading {}: {e}", d.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                if !is_skipped_dir(&name) {
                    stack.push(path);
                }
            } else if keep(&path) {
                let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?;
                out.push((rel.to_string_lossy().replace('\\', "/"), path));
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// `assets/...` files, kept verbatim in the archive.
fn assets(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let adir = dir.join("assets");
    if !adir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for (_rel, abs) in collect(&adir, &|_| true)? {
        let rel = abs.strip_prefix(dir).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
        let bytes = std::fs::read(&abs).map_err(|e| format!("reading {}: {e}", abs.display()))?;
        out.push((rel, bytes));
    }
    Ok(out)
}

/// `lib/util.lua` -> `lib.util`, `lib/util/init.lua` -> `lib.util` (how Lua's `require` names it).
fn require_name(rel: &str) -> String {
    let base = rel.strip_suffix(".lua").unwrap_or(rel);
    let base = base.strip_suffix("/init").unwrap_or(base);
    base.replace('/', ".")
}

fn compile_lua(dir: &Path, manifest: &Manifest) -> Result<Built, String> {
    let entry_rel = manifest.entry();
    let entry_path = dir.join(&entry_rel);
    if !entry_path.is_file() {
        return Err(format!("the entry file {entry_rel} is missing (set `entry` in openomsi-plugin.toml)"));
    }

    // Every .lua file except those under assets/ is part of the program.
    let lua_files = collect(dir, &|p| p.extension().is_some_and(|e| e == "lua") && !p.starts_with(dir.join("assets")))?;

    let entry_source =
        std::fs::read_to_string(&entry_path).map_err(|e| format!("reading {}: {e}", entry_path.display()))?;
    let mut modules = Vec::new();
    for (rel, abs) in &lua_files {
        if *rel == entry_rel {
            continue;
        }
        let source = std::fs::read_to_string(abs).map_err(|e| format!("reading {}: {e}", abs.display()))?;
        modules.push(Module { require_name: require_name(rel), source });
    }

    let opts = Options::level(manifest.lua.obfuscation_level);
    let compiled = lua_obfuscator::compile(&entry_source, &modules, &opts)
        .map_err(|e| format!("compiling Lua: {e}\nhint: fix the syntax error above (oopc parses the whole file)"))?;

    let mut files = vec![(entry_rel.clone(), compiled.into_bytes())];
    files.extend(assets(dir)?);
    let notes = vec![
        format!(
            "compiled {} Lua file(s) into {entry_rel} (obfuscation level {})",
            lua_files.len(),
            manifest.lua.obfuscation_level
        ),
        format!("{} module(s) bundled, {} asset(s)", modules.len(), files.len() - 1),
    ];
    Ok(Built { files, notes })
}

fn compile_rust(dir: &Path, manifest: &Manifest) -> Result<Built, String> {
    let cargo_toml = dir.join("Cargo.toml");
    if !cargo_toml.is_file() {
        return Err(format!("no Cargo.toml in {} (a Rust plugin is a cdylib crate)", dir.display()));
    }
    let mut notes = Vec::new();

    // Build the cdylib for wasm, asking cargo for machine-readable artifact paths.
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--release")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--manifest-path")
        .arg(&cargo_toml)
        .arg("--message-format=json-render-diagnostics");
    if !manifest.rust.features.is_empty() {
        cmd.arg("--features").arg(manifest.rust.features.join(","));
    }
    let output = cmd.output().map_err(|e| {
        format!("running cargo: {e}\nhint: install Rust and `rustup target add wasm32-unknown-unknown`")
    })?;
    if !output.status.success() {
        return Err(format!("cargo build failed:\n{}", String::from_utf8_lossy(&output.stderr).trim()));
    }

    // Find the cdylib .wasm among the JSON artifact lines.
    let mut wasm_path: Option<PathBuf> = None;
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) else { continue };
        if msg.get("reason").and_then(|r| r.as_str()) != Some("compiler-artifact") {
            continue;
        }
        if let Some(files) = msg.get("filenames").and_then(|f| f.as_array()) {
            for f in files {
                if let Some(p) = f.as_str()
                    && p.ends_with(".wasm")
                {
                    wasm_path = Some(PathBuf::from(p));
                }
            }
        }
    }
    let wasm_path = wasm_path.ok_or("cargo built no .wasm cdylib (is `crate-type = [\"cdylib\"]` set?)")?;
    let raw = std::fs::read(&wasm_path).map_err(|e| format!("reading {}: {e}", wasm_path.display()))?;
    let raw_len = raw.len();

    // Optimize with wasm-opt if it is on the PATH; either way, strip in pure Rust afterwards.
    let optimized = run_wasm_opt(&raw, &mut notes).unwrap_or(raw);
    let customs = wasm_strip::count_custom_sections(&optimized).unwrap_or(0);
    let stripped = wasm_strip::strip(&optimized).map_err(|e| format!("stripping the module: {e}"))?;
    notes.push(format!(
        "built {} -> plugin.wasm ({} KiB raw, {} KiB stripped, {customs} custom section(s) removed)",
        wasm_path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
        raw_len / 1024,
        stripped.len() / 1024
    ));

    let mut files = vec![("plugin.wasm".to_string(), stripped)];
    files.extend(assets(dir)?);
    Ok(Built { files, notes })
}

/// Runs `wasm-opt -Oz` when it is installed, returning the optimized bytes (or `None` to skip).
fn run_wasm_opt(wasm: &[u8], notes: &mut Vec<String>) -> Option<Vec<u8>> {
    let tmp = std::env::temp_dir();
    let input = tmp.join(format!("oopc-in-{}.wasm", std::process::id()));
    let out = tmp.join(format!("oopc-out-{}.wasm", std::process::id()));
    std::fs::write(&input, wasm).ok()?;
    let status = Command::new("wasm-opt")
        .arg("-Oz")
        .arg("--strip-debug")
        .arg("--strip-producers")
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .status();
    let result = match status {
        Ok(s) if s.success() => {
            notes.push("optimized with wasm-opt -Oz".to_string());
            std::fs::read(&out).ok()
        }
        _ => {
            notes.push("wasm-opt not found: used the pure-Rust stripper only".to_string());
            None
        }
    };
    let _ = std::fs::remove_file(&input);
    let _ = std::fs::remove_file(&out);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_names() {
        assert_eq!(require_name("lib/util.lua"), "lib.util");
        assert_eq!(require_name("util.lua"), "util");
        assert_eq!(require_name("lib/util/init.lua"), "lib.util");
    }
}
