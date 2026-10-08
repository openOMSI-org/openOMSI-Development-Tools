//! The application state: everything the UI reads and writes. Kept free of windowing or GPU
//! types so the very same `draw` runs under eframe (interactive) and under the offscreen renderer
//! (screenshots).

use std::path::PathBuf;

/// The pages in the left-hand navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// Open or create a plugin project.
    Projects,
    /// Edit the project's manifest and permissions.
    Manifest,
    /// Build the project and watch the log.
    Build,
    /// Manage signing keys.
    Keys,
    /// Drag in a `.oop` and read its header.
    Inspector,
    /// The documentation and API reference.
    Docs,
    /// Paths and preferences.
    Settings,
}

impl Page {
    /// Every page, in nav order.
    pub const ALL: [Page; 7] =
        [Page::Projects, Page::Manifest, Page::Build, Page::Keys, Page::Inspector, Page::Docs, Page::Settings];

    /// The label and a Material-ish glyph stand-in for the nav.
    pub fn title(self) -> &'static str {
        match self {
            Page::Projects => "Projects",
            Page::Manifest => "Manifest",
            Page::Build => "Build",
            Page::Keys => "Signing keys",
            Page::Inspector => "Inspector",
            Page::Docs => "Documentation",
            Page::Settings => "Settings",
        }
    }

    /// The name used on the command line for `--page`.
    pub fn slug(self) -> &'static str {
        match self {
            Page::Projects => "projects",
            Page::Manifest => "manifest",
            Page::Build => "build",
            Page::Keys => "keys",
            Page::Inspector => "inspector",
            Page::Docs => "docs",
            Page::Settings => "settings",
        }
    }

    /// Parses a `--page` name.
    pub fn from_slug(s: &str) -> Option<Page> {
        Page::ALL.into_iter().find(|p| p.slug() == s)
    }
}

/// Whether the app paints dark or light.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Follow nothing; dark.
    Dark,
    /// Light.
    Light,
}

/// The editable manifest fields, as plain strings for the form.
#[derive(Debug, Clone, Default)]
pub struct ManifestForm {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: String,
    pub is_rust: bool,
    pub entry: String,
    pub min_openomsi: String,
    pub license: String,
    pub homepage: String,
    /// Each known permission and whether it is ticked.
    pub permissions: Vec<(String, String, bool)>,
    pub obfuscation_level: u8,
}

/// A signing key the app knows about.
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub label: String,
    pub fingerprint: String,
    pub path: PathBuf,
}

/// The whole application state.
pub struct AppState {
    pub page: Page,
    pub theme: Theme,
    pub project_dir: Option<PathBuf>,
    pub recent: Vec<PathBuf>,
    pub manifest: ManifestForm,
    pub manifest_dirty: bool,
    pub build_log: Vec<(LogLevel, String)>,
    pub building: bool,
    pub keys: Vec<KeyEntry>,
    pub new_key_label: String,
    pub inspector_path: Option<PathBuf>,
    pub inspector_report: Option<InspectReport>,
    pub docs: crate::docs::Docs,
    pub docs_selected: usize,
    pub docs_search: String,
    pub settings: Settings,
    pub status: String,
}

/// A line in the build log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Ordinary progress.
    Info,
    /// Something to note.
    Warn,
    /// A failure.
    Error,
    /// Success.
    Good,
}

/// What the inspector shows for a loaded `.oop`.
#[derive(Debug, Clone)]
pub struct InspectReport {
    pub lines: Vec<(String, String)>,
    pub files: Vec<(String, u64)>,
    pub signature: String,
    pub ok: bool,
}

/// Persistent-ish settings (paths and preferences).
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub cargo_path: String,
    pub game_folder: String,
}

impl AppState {
    /// A fresh state with the known permissions listed and docs loaded.
    pub fn new() -> AppState {
        let permissions = oop_format::permissions::KNOWN
            .iter()
            .map(|p| (p.name.to_string(), p.description.to_string(), false))
            .collect();
        AppState {
            page: Page::Projects,
            theme: Theme::Dark,
            project_dir: None,
            recent: Vec::new(),
            manifest: ManifestForm {
                version: "0.1.0".into(),
                entry: "main.lua".into(),
                min_openomsi: "0.2.21".into(),
                license: "MIT".into(),
                permissions,
                obfuscation_level: 2,
                ..Default::default()
            },
            manifest_dirty: false,
            build_log: Vec::new(),
            building: false,
            keys: Vec::new(),
            new_key_label: "my openOMSI key".into(),
            inspector_path: None,
            inspector_report: None,
            docs: crate::docs::Docs::load(),
            docs_selected: 0,
            docs_search: String::new(),
            settings: Settings { cargo_path: "cargo".into(), game_folder: String::new() },
            status: "Ready".into(),
        }
    }

    /// Fills the state with representative content, so the headless screenshots show real pages
    /// rather than empty ones.
    pub fn demo() -> AppState {
        let mut s = AppState::new();
        s.project_dir = Some(PathBuf::from("~/plugins/stop-announcer"));
        s.recent = vec![
            PathBuf::from("~/plugins/stop-announcer"),
            PathBuf::from("~/plugins/speed-warner"),
            PathBuf::from("~/plugins/hello"),
        ];
        s.manifest.id = "com.example.stop-announcer".into();
        s.manifest.name = "Stop Announcer".into();
        s.manifest.description = "Announces each new bus stop and counts them across sessions.".into();
        s.manifest.authors = "you".into();
        for (name, _, on) in &mut s.manifest.permissions {
            if name == "ui" || name == "storage" {
                *on = true;
            }
        }
        s.build_log = vec![
            (LogLevel::Info, "compiled 2 Lua file(s) into main.lua (obfuscation level 2)".into()),
            (LogLevel::Info, "1 module bundled, 1 asset".into()),
            (LogLevel::Good, "built dist/com.example.stop-announcer.oop (3 KiB)".into()),
            (LogLevel::Info, "signed by 3f2a:9c01:77be:12d4".into()),
        ];
        s.keys = vec![KeyEntry {
            label: "my openOMSI key".into(),
            fingerprint: "3f2a:9c01:77be:12d4".into(),
            path: PathBuf::from("~/.config/openOMSI-devtools/keys/my-openOMSI-key.oopkey"),
        }];
        s.inspector_report = Some(InspectReport {
            lines: vec![
                ("Name".into(), "Stop Announcer 0.1.0".into()),
                ("Id".into(), "com.example.stop-announcer".into()),
                ("Kind".into(), "lua   entry: main.lua".into()),
                ("Permissions".into(), "ui, storage".into()),
                ("Built by".into(), "oopc 0.1.0 on 2026-10-08".into()),
            ],
            files: vec![("main.lua".into(), 603), ("assets/chime.ogg".into(), 20480)],
            signature: "valid, signed by 3f2a:9c01:77be:12d4".into(),
            ok: true,
        });
        s.inspector_path = Some(PathBuf::from("stop-announcer.oop"));
        s.settings.game_folder = "~/Games/openOMSI".into();
        s.status = "Built com.example.stop-announcer.oop".into();
        s
    }
}
