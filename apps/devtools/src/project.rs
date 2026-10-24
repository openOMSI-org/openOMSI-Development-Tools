//! A plugin project on disk: its summary, its files (read and written by the editor), its
//! manifest, new projects, and the game's copy of it.

use std::path::{Component, Path, PathBuf};

use oopc::manifest::{Manifest, ProjectKind};
use oopc::templates::{NewProject, Template};
use serde::Serialize;

/// What the lists show about a project.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub dir: String,
    pub name: String,
    pub id: String,
    pub version: String,
    pub kind: String,
    pub entry: String,
    /// The manifest could not be read: why.
    pub error: Option<String>,
}

pub fn summarize(dir: &Path) -> Summary {
    let folder = dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    match Manifest::load(dir) {
        Ok(m) => Summary {
            dir: dir.to_string_lossy().into_owned(),
            name: m.plugin.name.clone(),
            id: m.plugin.id.clone(),
            version: m.plugin.version.clone(),
            kind: if m.plugin.kind == ProjectKind::Rust { "rust" } else { "lua" }.into(),
            entry: m.entry(),
            error: None,
        },
        Err(e) => Summary {
            dir: dir.to_string_lossy().into_owned(),
            name: folder,
            id: String::new(),
            version: String::new(),
            kind: String::new(),
            entry: String::new(),
            error: Some(e.lines().next().unwrap_or("").to_string()),
        },
    }
}

/// A file or folder of the project's tree.
#[derive(Debug, Clone, Serialize)]
pub struct Node {
    pub name: String,
    /// Relative to the project, with `/`.
    pub path: String,
    pub dir: bool,
    pub size: u64,
    pub children: Vec<Node>,
}

/// Folders and files the tree leaves out: build output and the clutter of other tools.
fn hidden(name: &str) -> bool {
    name.starts_with('.') && name != ".gitignore" || matches!(name, "dist" | "target" | "node_modules")
}

pub fn tree(dir: &Path) -> Vec<Node> {
    fn walk(root: &Path, dir: &Path, depth: usize) -> Vec<Node> {
        let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
        let mut nodes: Vec<Node> = entries
            .flatten()
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                if hidden(&name) {
                    return None;
                }
                let path = e.path();
                let meta = e.metadata().ok()?;
                let rel = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
                let is_dir = meta.is_dir();
                let children = if is_dir && depth < 8 { walk(root, &path, depth + 1) } else { Vec::new() };
                Some(Node { name, path: rel, dir: is_dir, size: if is_dir { 0 } else { meta.len() }, children })
            })
            .collect();
        // folders first, then by name
        nodes.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        nodes
    }
    walk(dir, dir, 0)
}

/// `rel` inside `root`, refusing a way out of it (`..`, an absolute path).
pub fn safe_join(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = Path::new(rel);
    if rel.is_absolute()
        || rel.components().any(|c| matches!(c, Component::ParentDir | Component::Prefix(_) | Component::RootDir))
    {
        return Err(format!("{} is outside the project", rel.display()));
    }
    Ok(root.join(rel))
}

/// A file as the editor gets it.
#[derive(Debug, Serialize)]
pub struct FileText {
    /// The text, empty for a binary file.
    pub text: String,
    pub binary: bool,
    pub size: u64,
}

pub fn read_file(root: &Path, rel: &str) -> Result<FileText, String> {
    let path = safe_join(root, rel)?;
    let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {rel}: {e}"))?;
    let size = bytes.len() as u64;
    if size > 4 * 1024 * 1024 || bytes.contains(&0) {
        return Ok(FileText { text: String::new(), binary: true, size });
    }
    match String::from_utf8(bytes) {
        Ok(text) => Ok(FileText { text, binary: false, size }),
        Err(_) => Ok(FileText { text: String::new(), binary: true, size }),
    }
}

pub fn write_file(root: &Path, rel: &str, text: &str) -> Result<(), String> {
    let path = safe_join(root, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| format!("cannot write {rel}: {e}"))
}

/// A new, empty file or folder; the name must be free.
pub fn create_entry(root: &Path, rel: &str, dir: bool) -> Result<(), String> {
    let path = safe_join(root, rel)?;
    if path.exists() {
        return Err(format!("{rel} is already there"));
    }
    if dir {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&path, "").map_err(|e| e.to_string())
    }
}

pub fn rename_entry(root: &Path, from: &str, to: &str) -> Result<(), String> {
    let (a, b) = (safe_join(root, from)?, safe_join(root, to)?);
    if from == "openomsi-plugin.toml" {
        return Err("The manifest keeps its name".into());
    }
    if b.exists() {
        return Err(format!("{to} is already there"));
    }
    if let Some(parent) = b.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&a, &b).map_err(|e| format!("cannot rename {from}: {e}"))
}

pub fn delete_entry(root: &Path, rel: &str) -> Result<(), String> {
    if rel.is_empty() || rel == "openomsi-plugin.toml" {
        return Err("The manifest cannot be deleted".into());
    }
    let path = safe_join(root, rel)?;
    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("cannot delete {rel}: {e}"))
    } else {
        std::fs::remove_file(&path).map_err(|e| format!("cannot delete {rel}: {e}"))
    }
}

const HEADER: &str =
    "# openOMSI plugin project. The openOMSI Development Tools edit this file;\n# `oopc build` reads it too.\n";

pub fn load_manifest(dir: &Path) -> Result<Manifest, String> {
    Manifest::load(dir)
}

/// Writes the manifest after checking that it makes a valid header.
pub fn save_manifest(dir: &Path, manifest: &Manifest) -> Result<(), String> {
    manifest.to_header().validate().map_err(|e| e.to_string())?;
    let text = toml::to_string_pretty(manifest).map_err(|e| format!("cannot encode the manifest: {e}"))?;
    std::fs::write(dir.join("openomsi-plugin.toml"), format!("{HEADER}{text}"))
        .map_err(|e| format!("cannot write the manifest: {e}"))
}

/// What the new-project dialog sends.
#[derive(Debug, serde::Deserialize)]
pub struct NewRequest {
    pub folder: String,
    pub name: String,
    pub id: String,
    pub author: String,
    pub kind: String,
    pub template: String,
}

/// Makes the project `<folder>/<name as a folder name>` and returns its path.
pub fn create(req: &NewRequest) -> Result<PathBuf, String> {
    let name = req.name.trim();
    if name.is_empty() {
        return Err("Give the plugin a name".into());
    }
    if req.folder.trim().is_empty() {
        return Err("Choose where the project is made".into());
    }
    let slug: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let slug = if slug.is_empty() { "plugin".to_string() } else { slug };
    let dir = Path::new(req.folder.trim()).join(&slug);
    let np = NewProject {
        name: name.to_string(),
        id: req.id.trim().to_string(),
        author: req.author.clone(),
        rust: req.kind == "rust",
        template: Template::from_name(&req.template).unwrap_or_default(),
    };
    oopc::templates::create_with(&dir, &np)?;
    Ok(dir)
}

/// Copies the project's sources into the game's `plugins/<id>/` (a Lua plugin the game loads
/// as it is and loads again when a file changes). Files that did not change are left alone,
/// files the project no longer has are removed. Returns how many files changed.
pub fn sync_to_game(dir: &Path, plugins: &Path, manifest: &Manifest) -> Result<usize, String> {
    const MARK: &str = ".openomsi-devtools";
    let dest = plugins.join(&manifest.plugin.id);
    std::fs::create_dir_all(&dest).map_err(|e| format!("cannot make {}: {e}", dest.display()))?;
    let marker = dest.join(MARK);
    let has_files = std::fs::read_dir(&dest).map(|d| d.flatten().any(|e| e.file_name() != MARK)).unwrap_or(false);
    if has_files && !marker.exists() {
        return Err(format!(
            "{} already holds a plugin that did not come from the Development Tools; remove it or change this plugin's id",
            dest.display()
        ));
    }
    let mut changed = 0;
    let mut keep: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).map_err(|e| e.to_string())?.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if (name.starts_with('.') && name != ".gitignore")
                || matches!(name.as_str(), "dist" | "target" | "node_modules" | "openomsi-plugin.toml" | ".gitignore")
            {
                continue;
            }
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?.to_path_buf();
            let to = dest.join(&rel);
            let bytes = std::fs::read(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            if std::fs::read(&to).ok().as_deref() != Some(bytes.as_slice()) {
                if let Some(parent) = to.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(&to, &bytes).map_err(|e| format!("cannot write {}: {e}", to.display()))?;
                changed += 1;
            }
            keep.push(to);
        }
    }
    // (the game starts a folder plugin at main.lua: another entry file gets one that loads it)
    let entry = manifest.entry();
    if entry != "main.lua" {
        let module = entry.strip_suffix(".lua").unwrap_or(&entry).replace('/', ".");
        let main = dest.join("main.lua");
        let text = format!(
            "-- made by the openOMSI Development Tools: the entry file of this plugin is {entry}\nrequire(\"{module}\")\n"
        );
        if std::fs::read_to_string(&main).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&main, text).map_err(|e| e.to_string())?;
            changed += 1;
        }
        keep.push(main);
    }
    // stale files
    let mut stack = vec![dest.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).map_err(|e| e.to_string())?.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path.clone());
                continue;
            }
            if path == marker || keep.contains(&path) {
                continue;
            }
            if std::fs::remove_file(&path).is_ok() {
                changed += 1;
            }
        }
    }
    std::fs::write(
        &marker,
        "Kept up to date by the openOMSI Development Tools; delete this folder when the plugin is built as a .oop.\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_cannot_leave_the_project() {
        let root = Path::new("/p");
        assert!(safe_join(root, "../x").is_err());
        assert!(safe_join(root, "/etc/passwd").is_err());
        assert!(safe_join(root, "a/b.lua").is_ok());
    }

    #[test]
    fn a_new_project_opens_and_syncs() {
        let tmp = tempfile::tempdir().unwrap();
        let req = NewRequest {
            folder: tmp.path().to_string_lossy().into_owned(),
            name: "Speed Warner".into(),
            id: String::new(),
            author: "Ann".into(),
            kind: "lua".into(),
            template: "hello".into(),
        };
        let dir = create(&req).unwrap();
        assert!(dir.ends_with("speed-warner"));
        let s = summarize(&dir);
        assert_eq!(s.id, "com.ann.speed_warner");
        assert!(s.error.is_none());
        let m = load_manifest(&dir).unwrap();
        let plugins = tmp.path().join("game").join("plugins");
        assert!(sync_to_game(&dir, &plugins, &m).unwrap() >= 1);
        assert!(plugins.join(&m.plugin.id).join("main.lua").is_file());
        // a second sync changes nothing
        assert_eq!(sync_to_game(&dir, &plugins, &m).unwrap(), 0);
        // a file removed from the project goes from the game
        write_file(&dir, "extra.lua", "return 1").unwrap();
        assert_eq!(sync_to_game(&dir, &plugins, &m).unwrap(), 1);
        delete_entry(&dir, "extra.lua").unwrap();
        assert_eq!(sync_to_game(&dir, &plugins, &m).unwrap(), 1);
        assert!(!plugins.join(&m.plugin.id).join("extra.lua").exists());
    }
}
