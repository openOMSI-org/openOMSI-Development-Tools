//! Signing keys, the `.oop` inspector and the documentation the app carries.

use std::path::{Path, PathBuf};

use oop_format::{
    Oop, OopBuilder, decode_public_key, decode_secret_key, encode_public_key, encode_secret_key, fingerprint,
    generate_key, inspect_details,
};
use serde::Serialize;
use serde_json::{Value, json};

// --- keys ------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct KeyInfo {
    pub label: String,
    pub fingerprint: String,
    pub path: String,
    pub public_path: String,
}

/// The label a key file's first comment line carries.
fn label_of(text: &str) -> String {
    text.lines().next().and_then(|l| l.split('"').nth(1)).map(str::to_string).unwrap_or_default()
}

pub fn list_keys() -> Vec<KeyInfo> {
    let Ok(dir) = oopc::util::keys_dir() else { return Vec::new() };
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut keys: Vec<KeyInfo> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "oopkey"))
        .filter_map(|e| {
            let text = std::fs::read_to_string(e.path()).ok()?;
            let key = decode_secret_key(&text).ok()?;
            let path = e.path();
            Some(KeyInfo {
                label: label_of(&text).trim().to_string(),
                fingerprint: fingerprint(&key.verifying_key()),
                public_path: path.with_extension("oopkey.pub").to_string_lossy().into_owned(),
                path: path.to_string_lossy().into_owned(),
            })
        })
        .collect();
    keys.sort_by_key(|a| a.label.to_lowercase());
    keys
}

fn write_secret(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// A free file name in the keys folder for `label`.
fn key_path(label: &str) -> Result<PathBuf, String> {
    let safe: String = label.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let safe = if safe.trim_matches('-').is_empty() { "key".to_string() } else { safe };
    let dir = oopc::util::keys_dir()?;
    let mut path = dir.join(format!("{safe}.oopkey"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{safe}-{n}.oopkey"));
        n += 1;
    }
    Ok(path)
}

pub fn create_key(label: &str) -> Result<KeyInfo, String> {
    let label = if label.trim().is_empty() { "My openOMSI key" } else { label.trim() };
    let key = generate_key().map_err(|e| e.to_string())?;
    let path = key_path(label)?;
    write_secret(&path, &encode_secret_key(&key, label))?;
    let public = path.with_extension("oopkey.pub");
    std::fs::write(&public, encode_public_key(&key.verifying_key(), label)).map_err(|e| e.to_string())?;
    Ok(KeyInfo {
        label: label.into(),
        fingerprint: fingerprint(&key.verifying_key()),
        path: path.to_string_lossy().into_owned(),
        public_path: public.to_string_lossy().into_owned(),
    })
}

/// Copies a key file made elsewhere into the keys folder.
pub fn import_key(from: &str) -> Result<KeyInfo, String> {
    let text = std::fs::read_to_string(from).map_err(|e| format!("cannot read {from}: {e}"))?;
    let key = decode_secret_key(&text).map_err(|e| format!("this is not a signing key file: {e}"))?;
    let label = match label_of(&text).trim() {
        "" => "Imported key".to_string(),
        l => l.to_string(),
    };
    if let Some(k) = list_keys().into_iter().find(|k| k.fingerprint == fingerprint(&key.verifying_key())) {
        return Err(format!("This key is already there as \"{}\"", k.label));
    }
    let path = key_path(&label)?;
    write_secret(&path, &text)?;
    let public = path.with_extension("oopkey.pub");
    std::fs::write(&public, encode_public_key(&key.verifying_key(), &label)).map_err(|e| e.to_string())?;
    Ok(KeyInfo {
        label,
        fingerprint: fingerprint(&key.verifying_key()),
        path: path.to_string_lossy().into_owned(),
        public_path: public.to_string_lossy().into_owned(),
    })
}

pub fn delete_key(path: &str) -> Result<(), String> {
    let dir = oopc::util::keys_dir()?;
    let p = PathBuf::from(path);
    if p.parent() != Some(dir.as_path()) {
        return Err("That is not one of the app's keys".into());
    }
    std::fs::remove_file(&p).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(p.with_extension("oopkey.pub"));
    Ok(())
}

// --- the inspector -----------------------------------------------------------------------

/// A `.oop` as the inspector shows it.
pub fn inspect(path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let d = inspect_details(&bytes).map_err(|e| e.to_string())?;
    let header: Value = serde_json::from_str(&d.header_json).unwrap_or(Value::Null);
    let (signed, valid, signer) = match (&d.signer, d.signature_valid) {
        (Some(k), Some(v)) => (true, v, Some(fingerprint(k))),
        _ => (false, false, None),
    };
    let (files, readable) = match Oop::read(&bytes) {
        Ok(oop) => (oop.files().iter().map(|(p, b)| json!({"path": p, "size": b.len()})).collect::<Vec<_>>(), true),
        Err(_) => (Vec::new(), false),
    };
    Ok(json!({
        "path": path,
        "header": header,
        "format": d.format_version,
        "size": d.file_len,
        "signed": signed,
        "valid": valid,
        "signer": signer,
        "files": files,
        "intact": readable,
    }))
}

/// Signs `file` in place with the key file `key`.
pub fn sign(file: &str, key: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(key).map_err(|e| format!("cannot read the key: {e}"))?;
    let key = decode_secret_key(&text).map_err(|e| e.to_string())?;
    let bytes = std::fs::read(file).map_err(|e| format!("cannot read {file}: {e}"))?;
    let oop = Oop::read(&bytes).map_err(|e| e.to_string())?;
    let rebuilt =
        OopBuilder::new(oop.header().clone()).files(oop.into_files()).sign(&key).write().map_err(|e| e.to_string())?;
    std::fs::write(file, rebuilt).map_err(|e| format!("cannot write {file}: {e}"))?;
    Ok(fingerprint(&key.verifying_key()))
}

/// The public key file (text) of the key at `path`.
pub fn public_key_text(path: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let key = decode_public_key(&text).map_err(|e| e.to_string())?;
    Ok(encode_public_key(&key, &label_of(&text)))
}

// --- documentation -----------------------------------------------------------------------

/// The pages of `docs/`, packed into the program, and the API manifest.
pub fn docs() -> Value {
    const PAGES: [(&str, &str, &str); 5] = [
        ("getting-started", "Getting started", include_str!("../../../docs/getting-started.md")),
        ("oop-format", "The .oop format", include_str!("../../../docs/oop-format.md")),
        ("cli", "Command line (oopc)", include_str!("../../../docs/cli.md")),
        ("sdk", "Rust SDK", include_str!("../../../docs/sdk.md")),
        ("publishing", "Publishing and signing", include_str!("../../../docs/publishing.md")),
    ];
    let pages: Vec<Value> =
        PAGES.iter().map(|(id, title, md)| json!({"id": id, "title": title, "markdown": md})).collect();
    let api: Value = serde_json::from_str(oopc::util::API_JSON).unwrap_or(Value::Null);
    json!({"pages": pages, "api": api, "permissions": oop_format::permissions::KNOWN.iter().map(|p| json!({"name": p.name, "description": p.description})).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_through_the_folder() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: the only test that sets it (tests of this module run on one thread each, and none reads it at once)
        unsafe { std::env::set_var("OPENOMSI_DEVTOOLS_HOME", tmp.path()) };
        let k = create_key("Test key").unwrap();
        assert_eq!(list_keys().len(), 1);
        assert_eq!(list_keys()[0].label, "Test key");
        assert!(public_key_text(&k.path).unwrap().contains("public"));
        assert!(import_key(&k.path).is_err());
        delete_key(&k.path).unwrap();
        assert!(list_keys().is_empty());
    }

    #[test]
    fn docs_are_packed() {
        let d = docs();
        assert_eq!(d["pages"].as_array().unwrap().len(), 5);
        assert!(d["api"]["functions"].as_array().unwrap().len() > 100);
    }
}
