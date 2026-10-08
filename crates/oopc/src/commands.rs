//! The subcommands of `oopc`.

use std::path::Path;

use oop_format::{
    Oop, OopBuilder, decode_public_key, decode_secret_key, encode_public_key, encode_secret_key, fingerprint,
    generate_key, inspect_details, permissions,
};

use crate::build;
use crate::manifest::Manifest;
use crate::util;

/// Reads a secret key file.
fn load_key(path: &Path) -> Result<oop_format::SigningKey, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("reading key {}: {e}", path.display()))?;
    decode_secret_key(&text).map_err(|e| e.to_string())
}

/// Writes `bytes` to `path`, creating parent directories, with user-only permissions when secret.
fn write_file(path: &Path, bytes: &[u8], secret: bool) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
    if secret {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
    }
    Ok(())
}

/// `oopc build`.
pub fn build(dir: Option<&Path>, out: Option<&Path>, sign: Option<&Path>) -> Result<(), String> {
    let dir = dir.unwrap_or_else(|| Path::new("."));
    let manifest = Manifest::load(dir)?;
    let header = manifest.to_header();
    header.validate().map_err(|e| e.to_string())?;

    let built = build::compile(dir, &manifest)?;
    for note in &built.notes {
        println!("  {note}");
    }

    let mut builder = OopBuilder::new(header.clone()).files(built.files);
    let key = sign.map(load_key).transpose()?;
    if let Some(k) = &key {
        builder = builder.sign(k);
    }
    let bytes = builder.write().map_err(|e| e.to_string())?;

    let out_path = match out {
        Some(p) => p.to_path_buf(),
        None => dir.join("dist").join(format!("{}.oop", header.id)),
    };
    write_file(&out_path, &bytes, false)?;

    println!("built {} ({} KiB)", out_path.display(), bytes.len() / 1024);
    match &key {
        Some(k) => println!("signed by {}", fingerprint(&k.verifying_key())),
        None => println!("not signed (use --sign <key> to sign; unsigned plugins warn in the game)"),
    }
    Ok(())
}

/// `oopc pack`: pack a directory of prepared files as they are.
pub fn pack(dir: &Path, manifest_path: Option<&Path>, out: Option<&Path>, sign: Option<&Path>) -> Result<(), String> {
    let manifest = match manifest_path {
        Some(p) => {
            let text = std::fs::read_to_string(p).map_err(|e| format!("reading {}: {e}", p.display()))?;
            toml::from_str::<Manifest>(&text).map_err(|e| format!("{}: {e}", p.display()))?
        }
        None => Manifest::load(dir)?,
    };
    let header = manifest.to_header();
    header.validate().map_err(|e| e.to_string())?;

    // Everything in the directory except the manifest, build output and VCS clutter.
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || matches!(name.as_str(), "dist" | "target" | "openomsi-plugin.toml") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(dir).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
                files.push((rel, std::fs::read(&path).map_err(|e| e.to_string())?));
            }
        }
    }

    let mut builder = OopBuilder::new(header.clone()).files(files);
    let key = sign.map(load_key).transpose()?;
    if let Some(k) = &key {
        builder = builder.sign(k);
    }
    let bytes = builder.write().map_err(|e| e.to_string())?;
    let out_path = out.map(Path::to_path_buf).unwrap_or_else(|| dir.join(format!("{}.oop", header.id)));
    write_file(&out_path, &bytes, false)?;
    println!("packed {} ({} KiB)", out_path.display(), bytes.len() / 1024);
    Ok(())
}

/// `oopc keygen`.
pub fn keygen(label: &str, out: Option<&Path>, force: bool) -> Result<(), String> {
    let key = generate_key().map_err(|e| e.to_string())?;
    let fp = fingerprint(&key.verifying_key());
    let path = match out {
        Some(p) => p.to_path_buf(),
        None => {
            let safe: String = label.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
            util::keys_dir()?.join(format!("{safe}.oopkey"))
        }
    };
    if path.exists() && !force {
        return Err(format!("{} already exists (use --force to overwrite)", path.display()));
    }
    write_file(&path, encode_secret_key(&key, label).as_bytes(), true)?;
    let pub_path = path.with_extension("oopkey.pub");
    write_file(&pub_path, encode_public_key(&key.verifying_key(), label).as_bytes(), false)?;

    println!("new signing key, fingerprint {fp}");
    println!("  secret key: {} (keep this private)", path.display());
    println!("  public key: {} (share this so others can verify your plugins)", pub_path.display());
    Ok(())
}

/// `oopc sign`.
pub fn sign(file: &Path, key_path: &Path) -> Result<(), String> {
    let key = load_key(key_path)?;
    let bytes = std::fs::read(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    let oop = Oop::read(&bytes).map_err(|e| e.to_string())?;
    let rebuilt =
        OopBuilder::new(oop.header().clone()).files(oop.into_files()).sign(&key).write().map_err(|e| e.to_string())?;
    write_file(file, &rebuilt, false)?;
    println!("signed {} by {}", file.display(), fingerprint(&key.verifying_key()));
    Ok(())
}

/// `oopc verify`.
pub fn verify(file: &Path, key_path: Option<&Path>) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    let details = inspect_details(&bytes).map_err(|e| e.to_string())?;
    match (details.signer, details.signature_valid) {
        (Some(key), Some(true)) => {
            let fp = fingerprint(&key);
            println!("signed, signature valid, by {fp}");
            if let Some(kp) = key_path {
                let want = decode_public_key(&std::fs::read_to_string(kp).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
                if want == key {
                    println!("matches the key in {}", kp.display());
                } else {
                    return Err(format!("signed by {fp}, which is NOT the key in {}", kp.display()));
                }
            }
            Ok(())
        }
        (_, Some(false)) => Err("the signature does not match: the file was changed after signing".to_string()),
        (_, None) => Err("this plugin is not signed".to_string()),
        _ => Err("this plugin is not signed".to_string()),
    }
}

/// `oopc inspect`: header and file list (sizes only), using the built-in format key to read the
/// list but never printing any file's contents.
pub fn inspect(file: &Path, json: bool) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| format!("reading {}: {e}", file.display()))?;
    let details = inspect_details(&bytes).map_err(|e| e.to_string())?;
    let h = &details.header;
    if json {
        println!("{}", h.to_json());
        return Ok(());
    }
    println!("{} {}  ({})", h.name, h.version, h.id);
    if !h.description.is_empty() {
        println!("  {}", h.description);
    }
    println!("  kind: {}   entry: {}   api_abi: {}", h.kind, h.entry, h.api_abi);
    if !h.authors.is_empty() {
        println!("  authors: {}", h.authors.join(", "));
    }
    if let Some(l) = &h.license {
        println!("  license: {l}");
    }
    if let Some(m) = &h.min_openomsi {
        println!("  needs openOMSI >= {m}");
    }
    println!(
        "  permissions: {}",
        if h.permissions.is_empty() { "(none)".to_string() } else { h.permissions.join(", ") }
    );
    if let Some(tool) = &h.tool {
        println!("  built by: {tool}{}", h.created.as_ref().map(|c| format!(" on {c}")).unwrap_or_default());
    }
    match (&details.signer, details.signature_valid) {
        (Some(k), Some(true)) => println!("  signature: valid, signed by {}", fingerprint(k)),
        (_, Some(false)) => println!("  signature: PRESENT BUT INVALID (the file was changed after signing)"),
        _ => println!("  signature: none (unsigned)"),
    }
    println!("  file size: {} bytes (encrypted payload {} bytes)", details.file_len, details.payload_len);

    // The file list needs the payload; read it with the built-in key, but never show contents.
    match Oop::read(&bytes) {
        Ok(oop) => {
            println!("  files ({}):", oop.files().len());
            for (path, data) in oop.files() {
                println!("    {:>10}  {}", data.len(), path);
            }
        }
        Err(e) => println!("  files: cannot list ({e})"),
    }
    Ok(())
}

/// `oopc check`: lint a project.
pub fn check(dir: Option<&Path>) -> Result<(), String> {
    let dir = dir.unwrap_or_else(|| Path::new("."));
    let manifest = Manifest::load(dir)?;
    let header = manifest.to_header();
    let mut warnings = Vec::new();

    header.validate().map_err(|e| e.to_string())?;

    // The entry file must exist for a Lua plugin (a Rust one builds it).
    if manifest.plugin.kind == crate::manifest::ProjectKind::Lua {
        let entry = dir.join(manifest.entry());
        if !entry.is_file() {
            return Err(format!("the entry file {} does not exist", manifest.entry()));
        }
    }

    // Declared permissions should be ones the tools know.
    for p in &header.permissions {
        if permissions::find(p).is_none() {
            warnings.push(format!("permission {p:?} is not one this openOMSI knows"));
        }
    }

    // For Lua, compare the permissions the code actually uses against those declared.
    if manifest.plugin.kind == crate::manifest::ProjectKind::Lua {
        let mut source = String::new();
        collect_lua(dir, &mut source)?;
        let map = util::permission_map();
        let declared: std::collections::BTreeSet<&str> = header.permissions.iter().map(String::as_str).collect();
        let mut needed: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
        for (func, perm) in &map {
            if source.contains(&format!("omsi.{func}")) {
                needed.insert(perm.clone(), func.clone());
            }
        }
        if source.contains("omsi.data") {
            needed.insert("storage".to_string(), "omsi.data".to_string());
        }
        for (perm, func) in &needed {
            if !declared.contains(perm.as_str()) {
                warnings.push(format!(
                    "uses `{func}`, which needs the `{perm}` permission - add it to [plugin].permissions"
                ));
            }
        }
        for p in &header.permissions {
            if permissions::find(p).is_some() && !needed.contains_key(p) {
                warnings.push(format!("permission `{p}` is declared but the code does not seem to use it"));
            }
        }
    }

    if warnings.is_empty() {
        println!("{}: no problems found", manifest.plugin.id);
    } else {
        for w in &warnings {
            println!("warning: {w}");
        }
        println!("{} warning(s)", warnings.len());
    }
    Ok(())
}

fn collect_lua(dir: &Path, out: &mut String) -> Result<(), String> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || matches!(name.as_str(), "dist" | "target") {
                continue;
            }
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "lua") {
                out.push_str(&std::fs::read_to_string(&path).map_err(|e| e.to_string())?);
                out.push('\n');
            }
        }
    }
    Ok(())
}

/// `oopc new`.
pub fn new_project(name: &Path, rust: bool) -> Result<(), String> {
    crate::templates::create(name, rust)
}
