//! Problems found in a project, as data: syntax errors of a Lua file with their place, and what
//! `oopc check` says about the manifest and the permissions. The command line prints them; the
//! desktop app shows them in the editor.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use oop_format::permissions;
use serde::Serialize;

use crate::manifest::{Manifest, ProjectKind};
use crate::util;

/// How serious a problem is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The build cannot go on.
    Error,
    /// The plugin builds, but something is probably wrong.
    Warning,
}

/// One problem, with its place when it has one. Lines and columns count from 1.
#[derive(Debug, Clone, Serialize)]
pub struct Problem {
    /// The file, relative to the project (empty when the problem is about the project).
    pub file: String,
    /// The first line.
    pub line: u32,
    /// The first column.
    pub col: u32,
    /// The line the problem ends on.
    pub end_line: u32,
    /// The column it ends before.
    pub end_col: u32,
    /// Error or warning.
    pub severity: Severity,
    /// What is wrong, in a sentence.
    pub message: String,
    /// The permission to add (for a missing one): the editor offers it as a fix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_permission: Option<String>,
}

impl Problem {
    fn project(severity: Severity, message: String) -> Problem {
        Problem {
            file: String::new(),
            line: 0,
            col: 0,
            end_line: 0,
            end_col: 0,
            severity,
            message,
            fix_permission: None,
        }
    }
}

/// The syntax errors of one Lua source (empty when it parses).
pub fn lua_syntax(file: &str, source: &str) -> Vec<Problem> {
    match full_moon::parse(source) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .iter()
            .map(|e| {
                let (a, b) = e.range();
                let (line, col) = (a.line() as u32, a.character() as u32);
                let (mut end_line, mut end_col) = (b.line() as u32, b.character() as u32);
                if (end_line, end_col) <= (line, col) {
                    end_line = line;
                    end_col = col + 1;
                }
                Problem {
                    file: file.to_string(),
                    line,
                    col,
                    end_line,
                    end_col,
                    severity: Severity::Error,
                    message: e.error_message().into_owned(),
                    fix_permission: None,
                }
            })
            .collect(),
    }
}

/// Where each `omsi.<function>` that needs a permission is called in `source`:
/// `(function, permission, line, column, length)`.
fn permission_calls(source: &str, map: &BTreeMap<String, String>) -> Vec<(String, String, u32, u32, u32)> {
    let mut found = Vec::new();
    for (n, text) in source.lines().enumerate() {
        // (a comment is not a call)
        let code = text.split("--").next().unwrap_or(text);
        let mut from = 0;
        while let Some(at) = code[from..].find("omsi.") {
            let start = from + at;
            from = start + 5;
            // `myomsi.x` is somebody else's
            if code[..start].chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let rest = &code[start + 5..];
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect();
            // the longest dotted name the API has (`ui.set` out of `ui.set.x`)
            let mut parts: Vec<&str> = name.split('.').collect();
            while !parts.is_empty() {
                let candidate = parts.join(".");
                if let Some(perm) = map.get(&candidate) {
                    found.push((
                        candidate.clone(),
                        perm.clone(),
                        n as u32 + 1,
                        start as u32 + 1,
                        5 + candidate.len() as u32,
                    ));
                    break;
                }
                parts.pop();
            }
        }
    }
    found
}

/// The permissions a Lua project's code needs, with the first function that needs each.
pub fn needed_permissions(dir: &Path) -> BTreeMap<String, String> {
    let map = util::permission_map();
    let mut needed = BTreeMap::new();
    for (_, source) in lua_sources(dir) {
        for (func, perm, ..) in permission_calls(&source, &map) {
            needed.entry(perm).or_insert(func);
        }
    }
    needed
}

/// Every Lua source of the project: `(relative path, text)`.
fn lua_sources(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || matches!(name.as_str(), "dist" | "target" | "node_modules") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "lua")
                && let Ok(text) = std::fs::read_to_string(&path)
                && let Ok(rel) = path.strip_prefix(dir)
            {
                out.push((rel.to_string_lossy().replace('\\', "/"), text));
            }
        }
    }
    out.sort();
    out
}

/// Everything wrong with a project: the manifest, syntax errors, permissions missing and
/// unused. Never fails: a manifest that cannot be read is the first problem.
pub fn check_project(dir: &Path) -> Vec<Problem> {
    let mut out = Vec::new();
    let manifest = match Manifest::load(dir) {
        Ok(m) => m,
        Err(e) => {
            out.push(Problem::project(Severity::Error, e.lines().next().unwrap_or("").to_string()));
            return out;
        }
    };
    let header = manifest.to_header();
    if let Err(e) = header.validate() {
        out.push(Problem::project(Severity::Error, e.to_string()));
    }
    for p in &header.permissions {
        if permissions::find(p).is_none() {
            out.push(Problem::project(
                Severity::Warning,
                format!("The permission \"{p}\" is not one this openOMSI knows"),
            ));
        }
    }
    if manifest.plugin.kind == ProjectKind::Rust {
        if !dir.join("Cargo.toml").is_file() {
            out.push(Problem::project(
                Severity::Error,
                "A Rust plugin needs a Cargo.toml (a cdylib crate)".to_string(),
            ));
        }
        return out;
    }
    let entry = manifest.entry();
    if !dir.join(&entry).is_file() {
        out.push(Problem::project(Severity::Error, format!("The entry file {entry} is missing")));
    }
    let map = util::permission_map();
    let declared: BTreeSet<&str> = header.permissions.iter().map(String::as_str).collect();
    let mut used: BTreeSet<String> = BTreeSet::new();
    for (file, source) in lua_sources(dir) {
        out.extend(lua_syntax(&file, &source));
        for (func, perm, line, col, len) in permission_calls(&source, &map) {
            used.insert(perm.clone());
            if !declared.contains(perm.as_str()) {
                out.push(Problem {
                    file: file.clone(),
                    line,
                    col,
                    end_line: line,
                    end_col: col + len,
                    severity: Severity::Warning,
                    message: format!("omsi.{func} needs the \"{perm}\" permission, which the plugin does not declare"),
                    fix_permission: Some(perm),
                });
            }
        }
        if source.contains("omsi.data") && !declared.contains("storage") {
            used.insert("storage".to_string());
        }
    }
    for p in &header.permissions {
        if permissions::find(p).is_some() && !used.contains(p) {
            out.push(Problem::project(
                Severity::Warning,
                format!("The permission \"{p}\" is declared but the code does not seem to use it"),
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_error_has_a_place() {
        let p = lua_syntax("main.lua", "local x = \nif then end");
        assert!(!p.is_empty());
        assert_eq!(p[0].file, "main.lua");
        assert!(p[0].line >= 1);
    }

    #[test]
    fn good_lua_has_no_problems() {
        assert!(lua_syntax("main.lua", "omsi.on('start', function() end)").is_empty());
    }

    #[test]
    fn permission_calls_are_found_with_places() {
        let map = util::permission_map();
        let calls = permission_calls("-- omsi.ui.set(x)\nlocal a = 1\nomsi.ui.toast('hi')\nmyomsi.ui.set()", &map);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "ui.toast");
        assert_eq!(calls[0].2, 3);
    }
}
