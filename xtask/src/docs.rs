//! `cargo xtask gen-docs [out_dir]`: build the static documentation site (for GitHub Pages) from
//! the Markdown in `docs/` plus the API reference from `api.json`. Pure Rust, so CI needs no
//! external tool.

use std::path::{Path, PathBuf};

use serde_json::Value;

struct DocPage {
    slug: &'static str,
    title: &'static str,
    file: &'static str,
}

const PAGES: &[DocPage] = &[
    DocPage { slug: "getting-started", title: "Getting started", file: "getting-started.md" },
    DocPage { slug: "oop-format", title: "The .oop format", file: "oop-format.md" },
    DocPage { slug: "cli", title: "CLI reference", file: "cli.md" },
    DocPage { slug: "sdk", title: "Rust SDK", file: "sdk.md" },
    DocPage { slug: "publishing", title: "Publishing", file: "publishing.md" },
    DocPage { slug: "api", title: "API reference", file: "" },
];

pub fn gen_docs(root: &Path, out: &Path) -> Result<(), String> {
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let docs_dir = root.join("docs");

    for page in PAGES {
        let body_html = if page.slug == "api" {
            api_html(&root.join("crates/openomsi-plugin-sdk/api.json"))?
        } else {
            let md =
                std::fs::read_to_string(docs_dir.join(page.file)).map_err(|e| format!("reading {}: {e}", page.file))?;
            markdown_to_html(&md)
        };
        let html = wrap(page.title, page.slug, &body_html);
        std::fs::write(out.join(format!("{}.html", page.slug)), html).map_err(|e| e.to_string())?;
    }

    // The landing page.
    let landing = wrap("openOMSI Development Tools", "home", LANDING);
    std::fs::write(out.join("index.html"), landing).map_err(|e| e.to_string())?;
    // Tell GitHub Pages not to run Jekyll.
    std::fs::write(out.join(".nojekyll"), "").map_err(|e| e.to_string())?;
    println!("wrote the docs site to {}", out.display());
    Ok(())
}

pub fn default_out(root: &Path) -> PathBuf {
    root.join("site")
}

fn markdown_to_html(md: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn api_html(manifest: &Path) -> Result<String, String> {
    let json: Value = serde_json::from_str(&std::fs::read_to_string(manifest).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let mut out = String::new();
    out.push_str("<h1>API reference</h1>");
    if let Some(v) = json.get("version").and_then(Value::as_str) {
        out.push_str(&format!("<p>Generated from the API manifest, version {}.</p>", esc(v)));
    }
    out.push_str("<h2>Functions</h2>");
    if let Some(funcs) = json.get("functions").and_then(Value::as_array) {
        for f in funcs {
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
            let doc = f.get("doc").and_then(Value::as_str).unwrap_or("");
            let perm = f.get("permission").and_then(Value::as_str);
            out.push_str("<div class=\"api\">");
            out.push_str(&format!("<code class=\"sig\">{}({}) &rarr; {}</code>", esc(name), esc(&params), esc(ret)));
            out.push_str(&format!("<p>{}</p>", esc(doc)));
            if let Some(p) = perm {
                out.push_str(&format!("<p class=\"perm\">permission: {}</p>", esc(p)));
            }
            out.push_str("</div>");
        }
    }
    out.push_str("<h2>Events</h2>");
    if let Some(events) = json.get("events").and_then(Value::as_array) {
        for e in events {
            let name = e.get("name").and_then(Value::as_str).unwrap_or("");
            let argv = e
                .get("args")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();
            let doc = e.get("doc").and_then(Value::as_str).unwrap_or("");
            out.push_str("<div class=\"api\">");
            out.push_str(&format!("<code class=\"sig\">{}({})</code>", esc(name), esc(&argv)));
            out.push_str(&format!("<p>{}</p></div>", esc(doc)));
        }
    }
    Ok(out)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn nav_links(current: &str) -> String {
    let mut links = String::from("<a href=\"index.html\"");
    if current == "home" {
        links.push_str(" class=\"active\"");
    }
    links.push_str(">Home</a>");
    for page in PAGES {
        links.push_str(&format!(
            "<a href=\"{slug}.html\"{active}>{title}</a>",
            slug = page.slug,
            title = page.title,
            active = if current == page.slug { " class=\"active\"" } else { "" }
        ));
    }
    links
}

fn wrap(title: &str, current: &str, body: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title} — openOMSI Development Tools</title>\n<style>{CSS}</style></head>\n\
<body><header><a class=\"brand\" href=\"index.html\"><b>openOMSI</b> Development Tools</a>\n\
<nav>{nav}</nav></header>\n<main>{body}</main>\n\
<footer>openOMSI Development Tools · <a href=\"https://github.com/openOMSI-org/openOMSI-Development-Tools\">GitHub</a></footer></body></html>\n",
        nav = nav_links(current)
    )
}

const CSS: &str = r#"
:root { --accent:#F47F30; --bg:#14161A; --panel:#1B1E25; --fg:#E6E8EC; --muted:#9aa0ab; }
* { box-sizing:border-box; }
body { margin:0; font:16px/1.6 system-ui,-apple-system,Segoe UI,Roboto,sans-serif; background:var(--bg); color:var(--fg); }
header { display:flex; align-items:center; gap:24px; padding:14px 24px; background:var(--panel); position:sticky; top:0; flex-wrap:wrap; }
.brand { color:var(--fg); text-decoration:none; font-size:18px; }
.brand b { color:var(--accent); }
nav { display:flex; gap:16px; flex-wrap:wrap; }
nav a { color:var(--muted); text-decoration:none; }
nav a.active, nav a:hover { color:var(--accent); }
main { max-width:860px; margin:0 auto; padding:32px 24px 64px; }
h1 { font-size:2em; } h2 { color:var(--accent); margin-top:2em; }
a { color:var(--accent); }
code { background:#0f1117; padding:2px 5px; border-radius:4px; font-size:0.9em; }
pre { background:#0f1117; padding:14px; border-radius:8px; overflow:auto; }
pre code { background:none; padding:0; }
table { border-collapse:collapse; width:100%; }
th,td { border:1px solid #2a2e37; padding:8px 10px; text-align:left; }
.api { background:var(--panel); border-radius:8px; padding:12px 16px; margin:12px 0; }
.api .sig { background:none; color:var(--accent); font-weight:600; font-size:0.95em; }
.api .perm { color:var(--accent); font-size:0.85em; margin:0; }
.hero { text-align:center; padding:48px 0 24px; }
.hero h1 { font-size:2.6em; margin:0; }
.hero p { color:var(--muted); font-size:1.2em; }
.cards { display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:16px; margin-top:24px; }
.card { background:var(--panel); border-radius:10px; padding:20px; }
.card h3 { margin-top:0; color:var(--accent); }
@media (prefers-color-scheme: light) { :root { --bg:#f6f6f8; --panel:#fff; --fg:#1a1c20; --muted:#666; } code,pre{ background:#eef0f4; } }
"#;

const LANDING: &str = r#"
<div class="hero">
  <h1>openOMSI Development Tools</h1>
  <p>Build, sign and ship plugins for openOMSI — a plugin compiler, a Rust SDK, and a desktop app.</p>
  <p><a href="getting-started.html">Get started →</a> ·
     <a href="https://github.com/openOMSI-org/openOMSI-Development-Tools/releases">Downloads</a> ·
     <a href="https://github.com/openOMSI-org/openOMSI">openOMSI</a></p>
</div>
<div class="cards">
  <div class="card"><h3>oopc</h3><p>A CLI that compiles Lua or Rust plugins into a single signed <code>.oop</code> file.</p><p><a href="cli.html">CLI reference →</a></p></div>
  <div class="card"><h3>Rust SDK</h3><p>Write plugins in Rust with a safe, typed API that compiles to WebAssembly.</p><p><a href="sdk.html">SDK guide →</a></p></div>
  <div class="card"><h3>Desktop app</h3><p>Open projects, edit the manifest, build with a live log, manage keys and inspect files.</p></div>
  <div class="card"><h3>The .oop format</h3><p>A compiled, sandboxed plugin — never your source. Signed with Ed25519.</p><p><a href="oop-format.html">How it works →</a></p></div>
</div>
"#;
