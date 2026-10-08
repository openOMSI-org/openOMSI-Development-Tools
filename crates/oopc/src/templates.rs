//! Project templates for `oopc new`.

use std::path::Path;

/// The git dependency new Rust plugins use for the SDK.
const SDK_GIT: &str = "https://github.com/openOMSI-org/openOMSI-Development-Tools";

/// Creates a new project directory with a manifest and starter sources.
pub fn create(dir: &Path, rust: bool) -> Result<(), String> {
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let project = dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "plugin".to_string());
    let id = format!("com.example.{}", sanitize_id(&project));
    std::fs::create_dir_all(dir.join("assets")).map_err(|e| e.to_string())?;

    if rust {
        write(dir, "openomsi-plugin.toml", &rust_manifest(&id, &project))?;
        write(dir, "Cargo.toml", &rust_cargo(&project))?;
        write(dir, "src/lib.rs", RUST_LIB)?;
        write(dir, ".gitignore", "/target\n/dist\n")?;
        println!("created Rust plugin {} in {}", project, dir.display());
        println!("  build it: oopc build {}", dir.display());
    } else {
        write(dir, "openomsi-plugin.toml", &lua_manifest(&id, &project))?;
        write(dir, "main.lua", LUA_MAIN)?;
        write(dir, ".gitignore", "/dist\n")?;
        println!("created Lua plugin {} in {}", project, dir.display());
        println!("  build it: oopc build {}", dir.display());
    }
    println!("  make a signing key: oopc keygen");
    Ok(())
}

fn write(dir: &Path, rel: &str, contents: &str) -> Result<(), String> {
    let path = dir.join(rel);
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, contents).map_err(|e| format!("writing {}: {e}", path.display()))
}

fn sanitize_id(s: &str) -> String {
    s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect()
}

fn lua_manifest(id: &str, name: &str) -> String {
    format!(
        "# openOMSI plugin project. Build with `oopc build`.\n\
         [plugin]\n\
         id = \"{id}\"\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"lua\"\n\
         entry = \"main.lua\"\n\
         authors = [\"you\"]\n\
         description = \"A new openOMSI plugin\"\n\
         min_openomsi = \"0.2.21\"\n\
         # What the plugin may do. `oopc check` tells you which it needs.\n\
         permissions = [\"ui\"]\n\
         license = \"MIT\"\n\n\
         [lua]\n\
         # 0 strips formatting; 1 renames locals; 2 also encodes strings and numbers.\n\
         obfuscation_level = 2\n"
    )
}

fn rust_manifest(id: &str, name: &str) -> String {
    format!(
        "# openOMSI plugin project. Build with `oopc build`.\n\
         [plugin]\n\
         id = \"{id}\"\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"rust\"\n\
         authors = [\"you\"]\n\
         description = \"A new openOMSI plugin\"\n\
         min_openomsi = \"0.2.21\"\n\
         permissions = [\"ui\"]\n\
         license = \"MIT\"\n"
    )
}

fn rust_cargo(name: &str) -> String {
    let crate_name = sanitize_id(name);
    format!(
        "[package]\n\
         name = \"{crate_name}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2024\"\n\
         publish = false\n\n\
         [lib]\n\
         crate-type = [\"cdylib\"]\n\n\
         [dependencies]\n\
         openomsi-plugin-sdk = {{ git = \"{SDK_GIT}\" }}\n\n\
         [profile.release]\n\
         opt-level = \"z\"\n\
         lto = true\n\
         strip = true\n\
         panic = \"abort\"\n"
    )
}

const LUA_MAIN: &str = r#"-- A new openOMSI plugin. Edit me, then run `oopc build`.
-- The full plugin API is documented at the project's website.

omsi.on("start", function()
  omsi.message("Hello from my plugin!", 5)
end)

-- A gentle over-speed warning, checked once a second.
omsi.every(1, function()
  local kmh = omsi.var("Velocity")
  if kmh and kmh > 50 then
    omsi.message(string.format("Slow down: %.0f km/h", kmh), 1)
  end
end)
"#;

const RUST_LIB: &str = r#"//! A new openOMSI plugin. Edit me, then run `oopc build`.

use openomsi_plugin_sdk as omsi;

omsi::plugin!(|| {
    omsi::on_start(|| {
        let _ = omsi::api::message("Hello from my plugin!", Some(5.0));
    });

    // A gentle over-speed warning, checked once a second.
    omsi::every(1.0, || {
        if let Ok(Some(kmh)) = omsi::api::var("Velocity")
            && kmh > 50.0
        {
            let _ = omsi::api::message(&format!("Slow down: {kmh:.0} km/h"), Some(1.0));
        }
    });
});
"#;
