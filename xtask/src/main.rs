//! Workspace tasks. The one that matters: `cargo xtask gen-sdk [api.json]` regenerates the
//! plugin SDK's typed API (`crates/openomsi-plugin-sdk/src/api.rs`) from an API manifest. The
//! game side will export a bigger manifest as the API grows; regenerating is then this one
//! command.

mod docs;

use std::path::{Path, PathBuf};

use serde_json::Value;

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("gen-sdk") => {
            let root = workspace_root();
            let manifest =
                args.next().map(PathBuf::from).unwrap_or_else(|| root.join("crates/openomsi-plugin-sdk/api.json"));
            let out = root.join("crates/openomsi-plugin-sdk/src/api.rs");
            match gen_sdk(&manifest, &out) {
                Ok(n) => println!("generated {} with {n} functions from {}", out.display(), manifest.display()),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("gen-docs") => {
            let root = workspace_root();
            let out = args.next().map(PathBuf::from).unwrap_or_else(|| docs::default_out(&root));
            if let Err(e) = docs::gen_docs(&root, &out) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage: cargo xtask <gen-sdk [api.json] | gen-docs [out_dir]>");
            std::process::exit(2);
        }
    }
}

fn workspace_root() -> PathBuf {
    // xtask lives at <root>/xtask; CARGO_MANIFEST_DIR points there.
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask has a parent").to_path_buf()
}

const KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false", "fn", "for", "if", "impl",
    "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static", "struct", "super",
    "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

fn safe_ident(name: &str) -> String {
    if KEYWORDS.contains(&name) { format!("r#{name}") } else { name.to_string() }
}

/// A parameter's Rust type and the expression that turns it into a JSON value.
fn param_type(ty: &str, optional: bool) -> Option<(&'static str, bool)> {
    // Returns (rust type, is_str). Callbacks and varargs are handled by the hand-written runtime.
    let base = match ty {
        "number" => "f64",
        "string" => "str_ref",
        "bool" => "bool",
        "table" | "any" => "serde_json::Value",
        _ => return None,
    };
    let _ = optional;
    Some((base, base == "str_ref"))
}

fn gen_sdk(manifest: &Path, out: &Path) -> Result<usize, String> {
    let text = std::fs::read_to_string(manifest).map_err(|e| format!("reading {}: {e}", manifest.display()))?;
    let json: Value = serde_json::from_str(&text).map_err(|e| format!("parsing manifest: {e}"))?;
    let version = json.get("version").and_then(Value::as_str).unwrap_or("?");
    let abi = json.get("abi").and_then(Value::as_u64).unwrap_or(1);
    let functions = json.get("functions").and_then(Value::as_array).cloned().unwrap_or_default();

    let mut top = String::new(); // functions with no dot
    let mut groups: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let mut count = 0;

    for f in &functions {
        let name = f.get("name").and_then(Value::as_str).unwrap_or_default();
        let params = f.get("params").and_then(Value::as_array).cloned().unwrap_or_default();
        // Skip functions the hand-written runtime owns (callbacks, variadics).
        let has_callback_or_vararg = params.iter().any(|p| {
            let ty = p.get("type").and_then(Value::as_str).unwrap_or("");
            let pname = p.get("name").and_then(Value::as_str).unwrap_or("");
            ty == "function" || pname == "..."
        });
        if has_callback_or_vararg {
            continue;
        }
        let Some(code) = gen_fn(f, &params) else { continue };
        count += 1;
        match name.split_once('.') {
            Some((group, _)) => groups.entry(group.to_string()).or_default().push_str(&code),
            None => top.push_str(&code),
        }
    }

    let mut body = String::new();
    body.push_str(&format!(
        "//! Typed wrappers for the openOMSI plugin API, generated from `api.json`\n\
         //! (API version {version}, ABI {abi}) by `cargo xtask gen-sdk`. Do not edit by hand:\n\
         //! run the command again against a newer manifest instead.\n\
         //!\n\
         //! Every function calls into the game through [`crate::call`]; a function the manifest\n\
         //! does not have yet is still reachable with that escape hatch.\n\n\
         #![allow(clippy::all)]\n\
         use crate::{{Result, call}};\n\
         use serde_json::{{Value, json}};\n\n"
    ));
    body.push_str(&top);
    for (group, code) in &groups {
        body.push_str(&format!(
            "/// The `{group}` group of the API.\npub mod {} {{\n    use super::*;\n{}}}\n\n",
            safe_ident(group),
            indent(code)
        ));
    }

    std::fs::write(out, body).map_err(|e| format!("writing {}: {e}", out.display()))?;
    Ok(count)
}

fn indent(code: &str) -> String {
    code.lines().map(|l| if l.is_empty() { String::new() } else { format!("    {l}") }).collect::<Vec<_>>().join("\n")
        + "\n"
}

fn gen_fn(f: &Value, params: &[Value]) -> Option<String> {
    let full = f.get("name").and_then(Value::as_str)?;
    let short = full.rsplit('.').next().unwrap_or(full);
    let doc = f.get("doc").and_then(Value::as_str).unwrap_or("");
    let returns = f.get("returns").and_then(Value::as_str).unwrap_or("");
    let rtype = f.get("rtype").and_then(Value::as_str).unwrap_or("any");

    let mut sig_params = Vec::new();
    let mut arg_exprs = Vec::new();
    for p in params {
        let pname_raw = p.get("name").and_then(Value::as_str).unwrap_or("arg");
        let pname = safe_ident(pname_raw);
        let ty = p.get("type").and_then(Value::as_str).unwrap_or("any");
        let optional = p.get("optional").and_then(Value::as_bool).unwrap_or(false);
        let (base, is_str) = param_type(ty, optional)?;
        let rust_ty = if is_str { "&str" } else { base };
        if optional {
            let t = if is_str { "Option<&str>".to_string() } else { format!("Option<{rust_ty}>") };
            sig_params.push(format!("{pname}: {t}"));
            arg_exprs.push(format!("match {pname} {{ Some(v) => json!(v), None => Value::Null }}"));
        } else {
            sig_params.push(format!("{pname}: {rust_ty}"));
            arg_exprs.push(format!("json!({pname})"));
        }
    }

    let (ret_ty, body_tail) = match rtype {
        "nil" => ("()", "call(NAME, ARGS)?; Ok(())".to_string()),
        "bool" => ("bool", "Ok(call(NAME, ARGS)?.as_bool().unwrap_or(false))".to_string()),
        "number" => ("f64", "Ok(call(NAME, ARGS)?.as_f64().unwrap_or(0.0))".to_string()),
        "number?" => ("Option<f64>", "Ok(call(NAME, ARGS)?.as_f64())".to_string()),
        "string" => ("String", "Ok(call(NAME, ARGS)?.as_str().unwrap_or_default().to_string())".to_string()),
        "string?" => ("Option<String>", "Ok(call(NAME, ARGS)?.as_str().map(str::to_string))".to_string()),
        "list" | "multi" => ("Vec<Value>", "Ok(call(NAME, ARGS)?.as_array().cloned().unwrap_or_default())".to_string()),
        _ => ("Value", "call(NAME, ARGS)".to_string()),
    };

    let args_build = if arg_exprs.is_empty() {
        "Value::Array(vec![])".to_string()
    } else {
        format!("json!([{}])", arg_exprs.join(", "))
    };
    let body_tail = body_tail.replace("NAME", &format!("\"{full}\"")).replace("ARGS", &args_build);

    let mut doc_block = String::new();
    for line in wrap(doc, 92) {
        doc_block.push_str(&format!("/// {line}\n"));
    }
    if !returns.is_empty() {
        doc_block.push_str("///\n");
        for line in wrap(&format!("Returns: {returns}"), 92) {
            doc_block.push_str(&format!("/// {line}\n"));
        }
    }

    Some(format!(
        "{doc}pub fn {name}({params}) -> Result<{ret}> {{\n    {body}\n}}\n\n",
        doc = doc_block,
        name = safe_ident(short),
        params = sig_params.join(", "),
        ret = ret_ty,
        body = body_tail,
    ))
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
