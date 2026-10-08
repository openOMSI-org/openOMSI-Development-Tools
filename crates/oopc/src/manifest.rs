//! The `openomsi-plugin.toml` project manifest and its mapping to an `.oop` header.

use std::path::Path;

use oop_format::{Header, Kind};
use serde::{Deserialize, Serialize};

/// The project manifest at the root of a plugin project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// The `[plugin]` table.
    pub plugin: Plugin,
    /// Options for a Lua plugin.
    #[serde(default)]
    pub lua: LuaOptions,
    /// Options for a Rust plugin.
    #[serde(default)]
    pub rust: RustOptions,
}

/// The `[plugin]` table: what becomes the `.oop` header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Reverse-domain id, `com.author.name`.
    pub id: String,
    /// The display name.
    pub name: String,
    /// The plugin's version, `MAJOR.MINOR.PATCH`.
    pub version: String,
    /// `lua` or `rust`.
    pub kind: ProjectKind,
    /// The entry file for a Lua plugin (default `main.lua`). Ignored for Rust.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry: Option<String>,
    /// Authors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// A sentence or two.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// The oldest openOMSI this plugin supports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_openomsi: Option<String>,
    /// What the plugin may do (see `oop_format::permissions`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    /// SPDX license.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Homepage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// Whether a project is built from Lua sources or a Rust crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectKind {
    /// Lua sources, compiled by the obfuscating Lua compiler.
    Lua,
    /// A Rust crate, compiled to WebAssembly.
    Rust,
}

/// Lua build options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuaOptions {
    /// 0 strips formatting only; 1 renames; 2 (default) also encodes strings and numbers; 3 is
    /// reserved for heavier transforms.
    #[serde(default = "default_level")]
    pub obfuscation_level: u8,
}

fn default_level() -> u8 {
    2
}

impl Default for LuaOptions {
    fn default() -> Self {
        LuaOptions { obfuscation_level: 2 }
    }
}

/// Rust build options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RustOptions {
    /// Extra features to pass to `cargo build`.
    #[serde(default)]
    pub features: Vec<String>,
}

impl Manifest {
    /// Reads and parses `openomsi-plugin.toml` from a project directory.
    pub fn load(dir: &Path) -> Result<Manifest, String> {
        let path = dir.join("openomsi-plugin.toml");
        let text = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "cannot read {}: {e}\nhint: run `oopc new` to start a project, or `oopc build <dir>`",
                path.display()
            )
        })?;
        let manifest: Manifest = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(manifest)
    }

    /// The entry file name the header should carry.
    pub fn entry(&self) -> String {
        match self.plugin.kind {
            ProjectKind::Lua => self.plugin.entry.clone().unwrap_or_else(|| "main.lua".to_string()),
            ProjectKind::Rust => "plugin.wasm".to_string(),
        }
    }

    /// Builds the `.oop` header from the manifest, with `created` set to now and `tool` to oopc.
    pub fn to_header(&self) -> Header {
        let kind = match self.plugin.kind {
            ProjectKind::Lua => Kind::Lua,
            ProjectKind::Rust => Kind::Wasm,
        };
        let mut h = Header::new(&self.plugin.id, &self.plugin.name, &self.plugin.version, kind, self.entry());
        h.authors = self.plugin.authors.clone();
        h.description = self.plugin.description.clone();
        h.min_openomsi = self.plugin.min_openomsi.clone();
        h.permissions = self.plugin.permissions.clone();
        h.license = self.plugin.license.clone();
        h.homepage = self.plugin.homepage.clone();
        h.created = Some(crate::util::now_rfc3339());
        h.tool = Some(format!("oopc {}", env!("CARGO_PKG_VERSION")));
        h
    }
}
