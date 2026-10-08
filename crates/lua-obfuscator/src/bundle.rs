//! Wrapping the compiled chunks into one script: the string-decode helper, a table of the
//! plugin's own modules and a `require` that serves them, then the entry chunk.

use std::collections::BTreeSet;

use crate::emit::STRING_KEY;

/// The names of the helpers the bundle introduces, chosen not to clash with any global the
/// plugin uses.
pub struct HelperNames {
    /// The string-decode function.
    pub string_helper: String,
    /// The module table.
    pub modules: String,
    /// The module result cache.
    pub cache: String,
    /// The saved original `require`.
    pub orig_require: String,
}

fn pick(globals: &BTreeSet<String>, used: &mut Vec<String>, stem: &str) -> String {
    let mut n = 0u32;
    loop {
        let candidate = if n == 0 { format!("__oop_{stem}") } else { format!("__oop_{stem}{n}") };
        if !globals.contains(&candidate) && !used.contains(&candidate) {
            used.push(candidate.clone());
            return candidate;
        }
        n += 1;
    }
}

impl HelperNames {
    /// Picks names that avoid every global in `globals`.
    pub fn fresh(globals: &BTreeSet<String>) -> HelperNames {
        let mut used = Vec::new();
        HelperNames {
            string_helper: pick(globals, &mut used, "d"),
            modules: pick(globals, &mut used, "m"),
            cache: pick(globals, &mut used, "c"),
            orig_require: pick(globals, &mut used, "r"),
        }
    }
}

/// Builds the final chunk.
pub fn assemble(entry: &str, modules: &[(String, String)], names: &HelperNames) -> String {
    let mut out = String::new();

    // The string decoder: XOR every byte back with the key. Kept on one line.
    out.push_str(&format!(
        "local function {h}(s)local t={{}}for i=1,#s do t[i]=string.char(string.byte(s,i)~{k})end return table.concat(t)end\n",
        h = names.string_helper,
        k = STRING_KEY,
    ));

    if !modules.is_empty() {
        out.push_str(&format!("local {m}={{}}\n", m = names.modules));
        out.push_str(&format!("local {c}={{}}\n", c = names.cache));
        out.push_str(&format!("local {r}=require\n", r = names.orig_require));
        // The sandboxed `require`: serve a bundled module once (cache its result, nil -> true, as
        // Lua does), otherwise fall back to the game's require.
        out.push_str(&format!(
            "local function require(n)local f={m}[n]if f then local v={c}[n]if v==nil then v=f()if v==nil then v=true end {c}[n]=v end return v end return {r}(n)end\n",
            m = names.modules,
            c = names.cache,
            r = names.orig_require,
        ));
        for (name, code) in modules {
            out.push_str(&format!("{m}[{key}]=function()\n", m = names.modules, key = lua_quote(name)));
            out.push_str(code);
            out.push_str("\nend\n");
        }
    }

    out.push_str(entry);
    out.push('\n');
    out
}

/// A Lua double-quoted string literal of `s` (module names are plain identifiers and dots).
fn lua_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
