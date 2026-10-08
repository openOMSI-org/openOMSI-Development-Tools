//! The plain-text JSON header of an `.oop` file.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::archive::check_path;
use crate::error::{Error, Result};

/// The API ABI this format version was made for (`api_abi` in the header).
pub const API_ABI: u32 = 1;

/// What kind of code a plugin holds, the header's `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Lua 5.4 sources: the archive is the plugin's folder, `entry` its `main.lua`.
    Lua,
    /// A WebAssembly module (wasm32-unknown-unknown): `entry` is the `.wasm` file.
    Wasm,
    /// A kind a later openOMSI may know. Kept so that `inspect` still shows such a file.
    Other(String),
}

impl Kind {
    /// The header's spelling: `"lua"`, `"wasm"`, or the unknown kind as it was written.
    pub fn as_str(&self) -> &str {
        match self {
            Kind::Lua => "lua",
            Kind::Wasm => "wasm",
            Kind::Other(s) => s,
        }
    }

    /// The usual entry file of this kind (`main.lua`, `plugin.wasm`).
    pub fn default_entry(&self) -> &'static str {
        match self {
            Kind::Lua => "main.lua",
            Kind::Wasm | Kind::Other(_) => "plugin.wasm",
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Kind {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "lua" => Kind::Lua,
            "wasm" => Kind::Wasm,
            other => Kind::Other(other.to_string()),
        })
    }
}

impl Serialize for Kind {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(s.parse().unwrap_or_else(|never: std::convert::Infallible| match never {}))
    }
}

fn default_abi() -> u32 {
    API_ABI
}

/// The header of an `.oop` file: who made the plugin, what it is and what it may do.
///
/// It is stored as plain JSON before the encrypted payload, so `inspect` (and the game's plugin
/// list) can show it without decrypting anything. It is still authenticated: the payload's AEAD
/// tag covers it, so a changed header makes [`crate::Oop::read`] fail.
///
/// Keys a later version adds are kept in [`Header::extra`] and written back unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// Reverse-domain identifier, unique per plugin: `com.author.name`.
    pub id: String,
    /// The name people see.
    pub name: String,
    /// The plugin's own version, `MAJOR.MINOR.PATCH` (semver).
    pub version: String,
    /// Who wrote it.
    #[serde(default)]
    pub authors: Vec<String>,
    /// One or two sentences about what it does.
    #[serde(default)]
    pub description: String,
    /// `lua` or `wasm`.
    pub kind: Kind,
    /// The file the game starts: `main.lua` or `plugin.wasm`.
    pub entry: String,
    /// The oldest openOMSI version the plugin works with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_openomsi: Option<String>,
    /// The plugin API ABI it was built against (1).
    #[serde(default = "default_abi")]
    pub api_abi: u32,
    /// What the plugin may do: `ui`, `storage`, `vehicle_write`, ... (see [`crate::permissions`]).
    #[serde(default)]
    pub permissions: Vec<String>,
    /// SPDX license expression.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Where to find out more.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// When it was built, RFC 3339.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// What built it: `oopc 0.1.0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Keys this version does not know, kept as they are.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Header {
    /// A header with the required fields; the rest is empty or default (`api_abi` = 1).
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        kind: Kind,
        entry: impl Into<String>,
    ) -> Header {
        Header {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            authors: Vec::new(),
            description: String::new(),
            kind,
            entry: entry.into(),
            min_openomsi: None,
            api_abi: API_ABI,
            permissions: Vec::new(),
            license: None,
            homepage: None,
            created: None,
            tool: None,
            extra: serde_json::Map::new(),
        }
    }

    /// Parses the header JSON.
    pub fn from_json(bytes: &[u8]) -> Result<Header> {
        let text = std::str::from_utf8(bytes).map_err(|e| Error::HeaderJson(e.to_string()))?;
        serde_json::from_str(text).map_err(|e| Error::HeaderJson(e.to_string()))
    }

    /// The header as the compact JSON that goes into the file.
    pub fn to_json(&self) -> String {
        // A struct of strings, numbers and JSON values always serialises.
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Checks the fields both the writer and the reader rely on: the game uses `id` for the
    /// plugin's storage folder, so it must never be able to name a path.
    pub fn validate(&self) -> Result<()> {
        check_id(&self.id).map_err(Error::InvalidHeader)?;
        if self.name.trim().is_empty() || self.name.chars().count() > 128 {
            return Err(Error::InvalidHeader("`name` must be 1 to 128 characters".into()));
        }
        check_version(&self.version, true).map_err(|e| Error::InvalidHeader(format!("`version`: {e}")))?;
        if let Some(v) = &self.min_openomsi {
            check_version(v, false).map_err(|e| Error::InvalidHeader(format!("`min_openomsi`: {e}")))?;
        }
        check_path(&self.entry)
            .map_err(|_| Error::InvalidHeader(format!("`entry` {:?} is not a valid path", self.entry)))?;
        if self.api_abi == 0 {
            return Err(Error::InvalidHeader("`api_abi` must be 1 or more".into()));
        }
        for p in &self.permissions {
            if p.is_empty()
                || p.len() > 64
                || !p.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            {
                return Err(Error::InvalidHeader(format!(
                    "permission {p:?}: only lowercase letters, digits and `_` are allowed"
                )));
            }
        }
        Ok(())
    }
}

/// Checks a plugin id: 2 to 16 dot-separated segments of ASCII letters, digits, `_` and `-`,
/// at most 128 characters (`com.author.name`).
pub fn check_id(id: &str) -> std::result::Result<(), String> {
    if id.is_empty() || id.len() > 128 {
        return Err(format!("`id` {id:?} must be 1 to 128 characters"));
    }
    let segments: Vec<&str> = id.split('.').collect();
    if segments.len() < 2 || segments.len() > 16 {
        return Err(format!("`id` {id:?} must be reverse-domain style, like com.author.plugin"));
    }
    for s in segments {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            return Err(format!("`id` {id:?}: each part between dots needs letters, digits, `_` or `-` only"));
        }
    }
    Ok(())
}

/// Checks a version: `MAJOR.MINOR.PATCH` with an optional `-pre` and `+build` (semver) when
/// `strict`, otherwise also `MAJOR.MINOR` (openOMSI versions are written both ways).
pub fn check_version(v: &str, strict: bool) -> std::result::Result<(), String> {
    let core = v.split(['-', '+']).next().unwrap_or("");
    let rest = &v[core.len()..];
    let parts: Vec<&str> = core.split('.').collect();
    let n_ok = if strict { parts.len() == 3 } else { parts.len() == 2 || parts.len() == 3 };
    let nums_ok = parts.iter().all(|p| {
        !p.is_empty() && p.len() <= 9 && p.bytes().all(|b| b.is_ascii_digit()) && (p.len() == 1 || !p.starts_with('0'))
    });
    let rest_ok = rest.is_empty()
        || (rest.len() > 1
            && rest[1..].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-' || b == b'+'));
    if n_ok && nums_ok && rest_ok {
        Ok(())
    } else if strict {
        Err(format!("{v:?} is not a version like 1.2.0"))
    } else {
        Err(format!("{v:?} is not a version like 0.2.22"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids() {
        assert!(check_id("com.author.name").is_ok());
        assert!(check_id("io.github.some-one.bus_tool").is_ok());
        for bad in ["", "plugin", "com..x", ".com.x", "com/x.y", "com.x y", "com.x.", "../etc.passwd", "a.b\\c"] {
            assert!(check_id(bad).is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn versions() {
        for ok in ["0.1.0", "1.2.3", "10.0.1-beta.2", "1.0.0+build.5"] {
            assert!(check_version(ok, true).is_ok(), "{ok}");
        }
        for bad in ["1.2", "1", "01.2.3", "1.2.3-", "a.b.c", "1.2.3.4", ""] {
            assert!(check_version(bad, true).is_err(), "{bad}");
        }
        assert!(check_version("0.2", false).is_ok());
        assert!(check_version("0.2.22", false).is_ok());
    }

    #[test]
    fn unknown_keys_and_kinds_survive() {
        let json = br#"{"id":"a.b","name":"N","version":"1.0.0","kind":"native","entry":"x.so","future":{"x":1}}"#;
        let h = Header::from_json(json).unwrap();
        assert_eq!(h.kind, Kind::Other("native".into()));
        assert_eq!(h.extra["future"]["x"], 1);
        assert_eq!(h.api_abi, 1);
        let again = Header::from_json(h.to_json().as_bytes()).unwrap();
        assert_eq!(again, h);
    }
}
