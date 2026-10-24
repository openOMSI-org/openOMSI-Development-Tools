//! Updates from the project's GitHub releases, the way openOMSI and openMapper update
//! themselves: the newest release is looked for when the app starts; depending on the setting
//! it is offered, or installed at once; the download is checked against the SHA-256 GitHub
//! lists for it before anything is replaced.
//!
//! The release workflow publishes one `.zip` per platform
//! (`openOMSI-Dev-Tools-<version>-<os>-<arch>.zip`). Installing unpacks it beside the running
//! program, puts the running program aside as `*.old-update` (Windows lets a running program
//! be renamed, not overwritten), moves the new one into its place and starts it; the next start
//! deletes the old one. A development build (run from a cargo `target` folder) looks for
//! updates but never replaces itself.
//!
//! `OPENOMSI_DEVTOOLS_UPDATE_URL=<url or file:///...json>` points the look at another release
//! description (a file in the GitHub API's format, whose asset URLs may be `file://` too), for
//! tests; `OPENOMSI_DEVTOOLS_NO_UPDATE=1` switches the look off.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REPO: &str = "openOMSI-org/openOMSI-Development-Tools";
const LATEST_API: &str = "https://api.github.com/repos/openOMSI-org/openOMSI-Development-Tools/releases/latest";

/// A release newer than this build, with the file for this platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Release {
    pub version: String,
    pub page: String,
    pub notes: String,
    pub asset_name: String,
    pub asset_url: String,
    pub size: u64,
    pub sha256: Option<String>,
    pub sums_url: Option<String>,
}

pub fn current() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn parts(v: &str) -> Vec<u64> {
    v.trim().trim_start_matches(['v', 'V']).split(['.', '-', '+']).map_while(|p| p.parse::<u64>().ok()).collect()
}

/// Whether `candidate` is a newer version than `current`.
pub fn newer(candidate: &str, current: &str) -> bool {
    let (mut a, mut b) = (parts(candidate), parts(current));
    if a.is_empty() {
        return false;
    }
    let n = a.len().max(b.len());
    a.resize(n, 0);
    b.resize(n, 0);
    a > b
}

/// The release file for this platform, as `release.yml` names it.
pub fn asset_name(version: &str) -> Option<String> {
    let suffix = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "windows-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else {
        return None;
    };
    Some(format!("openOMSI-Dev-Tools-{}-{suffix}.zip", version.trim_start_matches(['v', 'V'])))
}

/// Run from a cargo target folder: never replaced.
pub fn is_dev_build() -> bool {
    std::env::current_exe()
        .ok()
        .is_some_and(|p| p.ancestors().skip(1).take(4).any(|a| a.join("CACHEDIR.TAG").is_file()))
}

/// Updates are switched off for this run.
pub fn disabled() -> bool {
    std::env::var_os("OPENOMSI_DEVTOOLS_NO_UPDATE").is_some_and(|v| !v.is_empty() && v != "0")
}

fn agent(redirects: u32) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(redirects)
        .timeout_connect(std::time::Duration::from_secs(20))
        .timeout_read(std::time::Duration::from_secs(30))
        .user_agent(&format!("openOMSI-Dev-Tools/{} (updater)", current()))
        .build()
}

fn fetch_text(url: &str) -> Result<String, String> {
    if let Some(path) = url.strip_prefix("file://") {
        return std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"));
    }
    let mut last = String::new();
    for pause in [0u64, 2, 6] {
        if pause > 0 {
            std::thread::sleep(std::time::Duration::from_secs(pause));
        }
        match agent(5).get(url).set("Accept", "application/vnd.github+json").call() {
            Ok(r) => return r.into_string().map_err(|e| e.to_string()),
            // (not worth another try: the answer would be the same)
            Err(ureq::Error::Status(code, _)) if code < 500 && code != 429 => return Err(format!("{code}")),
            Err(e) => last = e.to_string(),
        }
    }
    Err(last)
}

/// A release as the GitHub API describes it, when newer than this build and with a file for
/// this platform.
fn parse(v: &serde_json::Value) -> Option<Release> {
    let tag = v.get("tag_name")?.as_str()?;
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if !newer(&version, current()) || v.get("draft").and_then(|d| d.as_bool()).unwrap_or(false) {
        return None;
    }
    let want = asset_name(&version)?;
    let assets = v.get("assets")?.as_array()?;
    let asset = assets.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some(want.as_str()))?;
    let sums = assets.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some("SHA256SUMS"));
    Some(Release {
        version,
        page: v.get("html_url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
        notes: v.get("body").and_then(|b| b.as_str()).unwrap_or("").to_string(),
        asset_name: want,
        asset_url: asset.get("browser_download_url")?.as_str()?.to_string(),
        size: asset.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
        sha256: asset
            .get("digest")
            .and_then(|d| d.as_str())
            .and_then(|d| d.strip_prefix("sha256:"))
            .map(str::to_string),
        sums_url: sums.and_then(|s| s.get("browser_download_url")).and_then(|u| u.as_str()).map(str::to_string),
    })
}

/// When the API does not answer (403 after 60 requests an hour from one address): github.com
/// itself says which release is the latest (`releases/latest` redirects to its tag).
fn latest_from_site() -> Result<Option<Release>, String> {
    let url = format!("https://github.com/{REPO}/releases/latest");
    let r = agent(0).get(&url).call().map_err(|e| e.to_string())?;
    let location = r.header("location").ok_or("no release page")?.to_string();
    let tag = location.rsplit('/').next().unwrap_or("").to_string();
    let version = tag.trim_start_matches(['v', 'V']).to_string();
    if !newer(&version, current()) {
        return Ok(None);
    }
    let Some(asset) = asset_name(&version) else { return Ok(None) };
    let base = format!("https://github.com/{REPO}/releases/download/{tag}");
    Ok(Some(Release {
        version,
        page: location,
        notes: String::new(),
        asset_url: format!("{base}/{asset}"),
        asset_name: asset,
        size: 0,
        sha256: None,
        sums_url: Some(format!("{base}/SHA256SUMS")),
    }))
}

/// The latest release when it is newer than this build.
pub fn check() -> Result<Option<Release>, String> {
    let url = std::env::var("OPENOMSI_DEVTOOLS_UPDATE_URL").unwrap_or_else(|_| LATEST_API.to_string());
    match fetch_text(&url) {
        Ok(text) => {
            let v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("the release description is not JSON: {e}"))?;
            Ok(parse(&v))
        }
        Err(e) if url == LATEST_API && (e == "403" || e == "429") => latest_from_site(),
        Err(e) => Err(format!("Could not check for updates: {e}")),
    }
}

fn staging() -> PathBuf {
    std::env::temp_dir().join("openOMSI-Dev-Tools-update")
}

/// Downloads the release file, checking it against its SHA-256.
fn download(r: &Release, to: &Path, progress: &dyn Fn(u64, u64)) -> Result<(), String> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::create(to).map_err(|e| format!("cannot write {}: {e}", to.display()))?;
    let mut reader: Box<dyn Read> = if let Some(path) = r.asset_url.strip_prefix("file://") {
        Box::new(std::fs::File::open(path).map_err(|e| e.to_string())?)
    } else {
        Box::new(agent(5).get(&r.asset_url).call().map_err(|e| format!("the download failed: {e}"))?.into_reader())
    };
    let total = r.size;
    let mut done = 0u64;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("the download broke off: {e}"))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        done += n as u64;
        progress(done, total.max(done));
    }
    file.flush().map_err(|e| e.to_string())?;
    let got: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    let want = match (&r.sha256, &r.sums_url) {
        (Some(h), _) => Some(h.clone()),
        (None, Some(url)) => fetch_text(url).ok().and_then(|text| {
            text.lines().find_map(|l| {
                let mut it = l.split_whitespace();
                let (hash, name) = (it.next()?, it.next()?);
                (name.trim_start_matches('*') == r.asset_name).then(|| hash.to_string())
            })
        }),
        _ => None,
    };
    match want {
        Some(w) if w.eq_ignore_ascii_case(&got) => Ok(()),
        Some(_) => Err("The download does not match its checksum; nothing was installed".to_string()),
        None => Err("The release lists no checksum for this file; nothing was installed".to_string()),
    }
}

fn unpack(zip_path: &Path, to: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("the download is not a zip file: {e}"))?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let Some(rel) = entry.enclosed_name() else { continue };
        let out = to.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        #[cfg(unix)]
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            // a symbolic link inside a bundle: its text is the target
            let mut text = String::new();
            entry.read_to_string(&mut text).map_err(|e| e.to_string())?;
            std::os::unix::fs::symlink(text.trim(), &out).map_err(|e| e.to_string())?;
            continue;
        }
        let mut f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&out, std::fs::Permissions::from_mode(mode & 0o777));
        }
    }
    Ok(())
}

/// The first file under `dir` (any depth) that `keep` accepts.
fn find_file(dir: &Path, keep: &dyn Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).ok()?.flatten() {
            let p = e.path();
            if p.is_dir() && p.extension().is_none_or(|x| x != "app") {
                stack.push(p);
            } else if keep(&p) {
                return Some(p);
            }
        }
    }
    None
}

/// The `.app` bundle the running program is in (macOS).
fn running_bundle() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    exe.ancestors().find(|a| a.extension().is_some_and(|e| e == "app")).map(Path::to_path_buf)
}

fn aside(p: &Path) -> PathBuf {
    let mut name = p.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    name.push(".old-update");
    p.with_file_name(name)
}

/// Downloads `r` and puts it in the place of the running program. The new program is started
/// by `relaunch`.
pub fn install(r: &Release, progress: &dyn Fn(u64, u64)) -> Result<(), String> {
    if is_dev_build() {
        return Err("This is a development build: it looks for updates but never replaces itself".into());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let zip_path = staging().join(&r.asset_name);
    download(r, &zip_path, progress)?;

    if cfg!(target_os = "macos") {
        let bundle = running_bundle().ok_or("The program is not inside an .app bundle")?;
        let parent = bundle.parent().ok_or("no folder around the app")?;
        let stage = parent.join(".openOMSI-Dev-Tools-update");
        let _ = std::fs::remove_dir_all(&stage);
        std::fs::create_dir_all(&stage).map_err(|e| format!("cannot write to {}: {e}. Move the app to a folder you can write to, or download the new version by hand", parent.display()))?;
        let result = (|| {
            unpack(&zip_path, &stage)?;
            let new = std::fs::read_dir(&stage)
                .map_err(|e| e.to_string())?
                .flatten()
                .map(|e| e.path())
                .find(|p| p.extension().is_some_and(|x| x == "app"))
                .ok_or("the download holds no app")?;
            let old = aside(&bundle);
            let _ = std::fs::remove_dir_all(&old);
            std::fs::rename(&bundle, &old).map_err(|e| format!("cannot move the old app aside: {e}"))?;
            if let Err(e) = std::fs::rename(&new, &bundle) {
                let _ = std::fs::rename(&old, &bundle);
                return Err(format!("cannot put the new app in place: {e}"));
            }
            Ok(())
        })();
        let _ = std::fs::remove_dir_all(&stage);
        let _ = std::fs::remove_file(&zip_path);
        return result;
    }

    let dir = staging().join(format!("unpacked-{}", r.version));
    let _ = std::fs::remove_dir_all(&dir);
    unpack(&zip_path, &dir)?;
    let new = if cfg!(windows) {
        find_file(&dir, &|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")))
    } else {
        find_file(&dir, &|p| p.file_name().is_some_and(|n| n == "openomsi-devtools"))
    }
    .ok_or("the download holds no program")?;
    let old = aside(&exe);
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&exe, &old).map_err(|e| format!("cannot move the old program aside: {e}"))?;
    if let Err(e) = std::fs::copy(&new, &exe) {
        let _ = std::fs::rename(&old, &exe);
        return Err(format!("cannot put the new program in place: {e}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755));
    }
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_file(&zip_path);
    Ok(())
}

/// Starts the program (the new one after an update) and returns; the caller ends this one.
pub fn relaunch() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    if let Some(bundle) = running_bundle() {
        return std::process::Command::new("open").arg("-n").arg(bundle).spawn().map(|_| ()).map_err(|e| e.to_string());
    }
    std::process::Command::new(&exe).spawn().map(|_| ()).map_err(|e| format!("cannot start {}: {e}", exe.display()))
}

/// At start: what an earlier update left aside goes.
pub fn cleanup() {
    let Ok(exe) = std::env::current_exe() else { return };
    let target = if cfg!(target_os = "macos") { running_bundle().unwrap_or(exe) } else { exe };
    let old = aside(&target);
    if old.is_dir() {
        let _ = std::fs::remove_dir_all(&old);
    } else {
        let _ = std::fs::remove_file(&old);
    }
    let _ = std::fs::remove_dir_all(staging());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert!(newer("0.10.0", "0.9.3"));
        assert!(newer("v1.0", "0.9.9"));
        assert!(!newer("0.2.0", "0.2.0"));
        assert!(!newer("0.1.9", "0.2.0"));
        assert!(!newer("latest", "0.2.0"));
    }

    #[test]
    fn a_release_needs_this_platforms_file() {
        let want = asset_name("9.9.9").unwrap();
        let v = serde_json::json!({
            "tag_name": "v9.9.9", "body": "notes", "html_url": "https://example.test/r",
            "assets": [
                {"name": want, "browser_download_url": "https://example.test/a.zip", "size": 10, "digest": "sha256:abcd"},
                {"name": "SHA256SUMS", "browser_download_url": "https://example.test/sums"}
            ]
        });
        let r = parse(&v).unwrap();
        assert_eq!(r.version, "9.9.9");
        assert_eq!(r.sha256.as_deref(), Some("abcd"));
        let none = serde_json::json!({"tag_name": "v9.9.9", "assets": []});
        assert!(parse(&none).is_none());
    }

    #[test]
    fn a_zip_installs_and_a_wrong_checksum_does_not() {
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("pack.zip");
        {
            let f = std::fs::File::create(&zip_path).unwrap();
            let mut z = zip::ZipWriter::new(f);
            let opts = zip::write::SimpleFileOptions::default();
            z.start_file("dir/openomsi-devtools", opts).unwrap();
            z.write_all(b"new program").unwrap();
            z.finish().unwrap();
        }
        let bytes = std::fs::read(&zip_path).unwrap();
        let good: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        let mut r = Release {
            version: "9.9.9".into(),
            page: String::new(),
            notes: String::new(),
            asset_name: "pack.zip".into(),
            asset_url: format!("file://{}", zip_path.display()),
            size: bytes.len() as u64,
            sha256: Some("00".repeat(32)),
            sums_url: None,
        };
        let out = tmp.path().join("got.zip");
        assert!(download(&r, &out, &|_, _| {}).is_err());
        r.sha256 = Some(good);
        download(&r, &out, &|_, _| {}).unwrap();
        let dir = tmp.path().join("un");
        unpack(&out, &dir).unwrap();
        assert!(find_file(&dir, &|p| p.file_name().is_some_and(|n| n == "openomsi-devtools")).is_some());
    }
}
