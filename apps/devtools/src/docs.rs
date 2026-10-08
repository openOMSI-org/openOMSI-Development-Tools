//! The documentation shown in the app: the guide pages (embedded Markdown) and the API reference
//! (built from the embedded `api.json`). Kept as data so the UI can render and search it.

use serde_json::Value;

/// One documentation page.
pub struct DocPage {
    /// Its title in the list.
    pub title: String,
    /// Its Markdown body.
    pub body: String,
}

/// An API function, for the reference page.
pub struct ApiEntry {
    pub name: String,
    pub group: String,
    pub signature: String,
    pub doc: String,
    pub permission: Option<String>,
}

/// Everything the Documentation page shows.
pub struct Docs {
    /// The guide pages.
    pub pages: Vec<DocPage>,
    /// The API reference entries.
    pub api: Vec<ApiEntry>,
}

const GETTING_STARTED: &str = include_str!("../../../docs/getting-started.md");
const OOP_FORMAT: &str = include_str!("../../../docs/oop-format.md");
const CLI: &str = include_str!("../../../docs/cli.md");
const SDK: &str = include_str!("../../../docs/sdk.md");
const PUBLISHING: &str = include_str!("../../../docs/publishing.md");
const API_JSON: &str = include_str!("../../../crates/openomsi-plugin-sdk/api.json");

impl Docs {
    /// Loads the embedded documentation.
    pub fn load() -> Docs {
        let pages = vec![
            DocPage { title: "Getting started".into(), body: GETTING_STARTED.into() },
            DocPage { title: "The .oop format".into(), body: OOP_FORMAT.into() },
            DocPage { title: "CLI reference".into(), body: CLI.into() },
            DocPage { title: "Rust SDK".into(), body: SDK.into() },
            DocPage { title: "Publishing".into(), body: PUBLISHING.into() },
        ];
        Docs { pages, api: parse_api(API_JSON) }
    }
}

/// Builds the one-line signature shown in the reference, e.g. `ui.set(id: string, panel: table) -> bool, reason`.
fn signature(f: &Value) -> String {
    let name = f.get("name").and_then(Value::as_str).unwrap_or("");
    let params = f
        .get("params")
        .and_then(Value::as_array)
        .map(|ps| {
            ps.iter()
                .map(|p| {
                    let n = p.get("name").and_then(Value::as_str).unwrap_or("");
                    let t = p.get("type").and_then(Value::as_str).unwrap_or("any");
                    let opt = p.get("optional").and_then(Value::as_bool).unwrap_or(false);
                    if opt { format!("{n}?: {t}") } else { format!("{n}: {t}") }
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default();
    let ret = f.get("returns").and_then(Value::as_str).unwrap_or("");
    format!("{name}({params}) -> {ret}")
}

fn parse_api(json: &str) -> Vec<ApiEntry> {
    let mut out = Vec::new();
    if let Ok(Value::Object(root)) = serde_json::from_str::<Value>(json)
        && let Some(Value::Array(funcs)) = root.get("functions")
    {
        for f in funcs {
            out.push(ApiEntry {
                name: f.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
                group: f.get("group").and_then(Value::as_str).unwrap_or("").to_string(),
                signature: signature(f),
                doc: f.get("doc").and_then(Value::as_str).unwrap_or("").to_string(),
                permission: f.get("permission").and_then(Value::as_str).map(str::to_string),
            });
        }
    }
    out
}
