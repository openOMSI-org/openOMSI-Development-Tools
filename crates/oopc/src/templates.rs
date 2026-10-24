//! Project templates for `oopc new`.

use std::path::Path;

/// The git dependency new Rust plugins use for the SDK.
const SDK_GIT: &str = "https://github.com/openOMSI-org/openOMSI-Development-Tools";

/// What a new project starts with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Template {
    /// A script that only says hello.
    #[default]
    Hello,
    /// An empty script with the entry point and a comment.
    Blank,
    /// A panel on the screen with live values and a button (Lua only).
    Panel,
}

impl Template {
    /// Parses the name the app sends (`hello`, `blank`, `panel`).
    pub fn from_name(s: &str) -> Option<Template> {
        match s {
            "hello" => Some(Template::Hello),
            "blank" => Some(Template::Blank),
            "panel" => Some(Template::Panel),
            _ => None,
        }
    }
}

/// The answers a new project needs.
#[derive(Debug, Clone)]
pub struct NewProject {
    /// The display name.
    pub name: String,
    /// The reverse-domain id; made from the author and the name when empty.
    pub id: String,
    /// The author, put into the manifest.
    pub author: String,
    /// A Rust (WebAssembly) plugin instead of a Lua one.
    pub rust: bool,
    /// The starter code.
    pub template: Template,
}

/// `author` and `name` as an id: `com.<author>.<name>`, lower case, letters and digits.
pub fn make_id(author: &str, name: &str) -> String {
    let a = sanitize_id(author);
    let n = sanitize_id(name);
    let a = if a.trim_matches('_').is_empty() { "example".to_string() } else { a };
    let n = if n.trim_matches('_').is_empty() { "plugin".to_string() } else { n };
    format!("com.{}.{}", a.trim_matches('_'), n.trim_matches('_'))
}

/// Creates a new project directory with a manifest and starter sources.
pub fn create(dir: &Path, rust: bool) -> Result<(), String> {
    let project = dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "plugin".to_string());
    let np = NewProject {
        name: project.clone(),
        id: make_id("example", &project),
        author: "you".into(),
        rust,
        template: Template::Hello,
    };
    create_with(dir, &np)?;
    println!("created {} plugin {} in {}", if rust { "Rust" } else { "Lua" }, project, dir.display());
    println!("  build it: oopc build {}", dir.display());
    println!("  make a signing key: oopc keygen");
    Ok(())
}

/// Creates the project folder `dir` as `np` asks.
pub fn create_with(dir: &Path, np: &NewProject) -> Result<(), String> {
    if dir.exists() && std::fs::read_dir(dir).map(|mut d| d.next().is_some()).unwrap_or(false) {
        return Err(format!("{} is not empty", dir.display()));
    }
    let id = if np.id.trim().is_empty() { make_id(&np.author, &np.name) } else { np.id.trim().to_string() };
    let name = if np.name.trim().is_empty() { "My plugin".to_string() } else { np.name.trim().to_string() };
    let author = if np.author.trim().is_empty() { "you".to_string() } else { np.author.trim().to_string() };
    std::fs::create_dir_all(dir.join("assets")).map_err(|e| e.to_string())?;
    if np.rust {
        write(dir, "openomsi-plugin.toml", &rust_manifest(&id, &name, &author))?;
        write(dir, "Cargo.toml", &rust_cargo(&name))?;
        write(dir, "src/lib.rs", RUST_LIB)?;
        write(dir, ".gitignore", "/target\n/dist\n")?;
    } else {
        write(dir, "openomsi-plugin.toml", &lua_manifest(&id, &name, &author, np.template))?;
        let code = match np.template {
            Template::Hello => LUA_MAIN,
            Template::Blank => LUA_BLANK,
            Template::Panel => LUA_PANEL,
        };
        write(dir, "main.lua", code)?;
        write(dir, ".gitignore", "/dist\n")?;
    }
    write(dir, "assets/.keep", "")?;
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

fn lua_manifest(id: &str, name: &str, author: &str, template: Template) -> String {
    let permissions = if template == Template::Blank { "" } else { "\"ui\"" };
    format!(
        "# openOMSI plugin project. Build with `oopc build`.\n\
         [plugin]\n\
         id = \"{id}\"\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"lua\"\n\
         entry = \"main.lua\"\n\
         authors = [\"{author}\"]\n\
         description = \"A new openOMSI plugin\"\n\
         min_openomsi = \"0.2.21\"\n\
         # What the plugin may do. `oopc check` tells you which it needs.\n\
         permissions = [{permissions}]\n\
         license = \"MIT\"\n\n\
         [lua]\n\
         # 0 strips formatting; 1 renames locals; 2 also encodes strings and numbers.\n\
         obfuscation_level = 2\n"
    )
}

fn rust_manifest(id: &str, name: &str, author: &str) -> String {
    format!(
        "# openOMSI plugin project. Build with `oopc build`.\n\
         [plugin]\n\
         id = \"{id}\"\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"rust\"\n\
         authors = [\"{author}\"]\n\
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

const LUA_BLANK: &str = r#"-- A new openOMSI plugin. The game loads this file when it starts and again after every save.
-- The reference: Documentation in the Development Tools, or the plugin API page of the site.

omsi.on("start", function()
  -- runs once, right after the plugin was loaded
end)
"#;

const LUA_PANEL: &str = r##"-- A panel on the screen: line, speed and a horn button. Press F10 in the game to use the mouse.

local ACCENT = "#F47F30"

local function show()
  local s = omsi.bus.state() or {}
  local d = omsi.duty.get() or {}
  omsi.ui.set("trip", {
    anchor = "top_left", x = 16, y = 60, width = 300, accent = ACCENT,
    children = {
      { type = "row", children = {
        { type = "icon", name = "directions_bus", color = ACCENT },
        { type = "text", text = "Line " .. (d.line or "-"), weight = "bold", grow = true },
        { type = "badge", text = string.format("%.0f km/h", s.speed or 0) },
      } },
      { type = "text", text = "Next stop: " .. (d.next_stop or "-") },
      { type = "button", id = "horn", text = "Horn", icon = "campaign" },
    },
  })
end

omsi.every(0.5, show)

omsi.on("key", function(key, down)
  if key == "F10" and down then omsi.ui.focus(not omsi.ui.focused()) end
end)

omsi.on("ui_click", function(panel, element)
  if element == "horn" then omsi.trigger("bus_horn") end
end)
"##;

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
