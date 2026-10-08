//! An obfuscating Lua-to-Lua compiler for openOMSI plugins.
//!
//! A `.oop` plugin is a compiled artefact, not a copy of its author's source. For Lua plugins
//! that compilation happens here: the sources are parsed, their comments and formatting are
//! dropped, every local, parameter and loop variable is renamed to a meaningless name, string
//! literals are encoded and decoded at run time by a small generated helper, plain integers are
//! rewritten as arithmetic, and a plugin's own modules are merged into one chunk (its own
//! `require` calls are resolved at build time). The result runs identically but carries none of
//! the original structure or names.
//!
//! It is **not** Lua bytecode: openOMSI refuses binary chunks, because Lua 5.4 does not verify
//! bytecode and crafted bytecode could break out of the plugin sandbox. The output is ordinary
//! Lua text that the game loads the same way it loads a plain script.
//!
//! The names the game's API uses are globals (`omsi`, `require`, `print`, and the `on_<event>`
//! functions a plugin may define), so they are never renamed.
//!
//! ```
//! use lua_obfuscator::{Module, Options};
//! let out = lua_obfuscator::compile(
//!     "local greeting = require(\"greet\")\nomsi.message(greeting)\n",
//!     &[Module { require_name: "greet".into(), source: "return \"hi\"\n".into() }],
//!     &Options::level(2),
//! ).unwrap();
//! assert!(out.contains("message")); // the API name is kept
//! assert!(!out.contains("greeting")); // the local was renamed
//! ```

mod bundle;
mod emit;
mod resolve;
mod unescape;

use std::fmt;

/// What the compiler does, chosen by level (see [`Options::level`]) or set field by field.
#[derive(Debug, Clone)]
pub struct Options {
    /// Rename locals, parameters and loop variables.
    pub rename: bool,
    /// Encode string literals (decoded at run time).
    pub encode_strings: bool,
    /// Rewrite plain integer constants as arithmetic.
    pub rewrite_numbers: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options::level(2)
    }
}

impl Options {
    /// A preset: 1 strips and renames; 2 (the default) also encodes strings and numbers; 3 is the
    /// same as 2 today and reserved for heavier transforms later.
    pub fn level(level: u8) -> Options {
        Options { rename: level >= 1, encode_strings: level >= 2, rewrite_numbers: level >= 2 }
    }

    /// Strip comments and formatting only, change nothing else (`--level 0`).
    pub fn none() -> Options {
        Options { rename: false, encode_strings: false, rewrite_numbers: false }
    }
}

/// One of the plugin's own Lua modules, besides the entry file.
#[derive(Debug, Clone)]
pub struct Module {
    /// How the plugin's code names it in `require`, dots and all: `"lib.util"` for `lib/util.lua`.
    pub require_name: String,
    /// Its source.
    pub source: String,
}

/// A compile error with the file it happened in.
#[derive(Debug, Clone)]
pub struct Error {
    /// The module's `require_name`, or `"main"` for the entry file.
    pub file: String,
    /// What went wrong (the Lua parse errors, usually).
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.file, self.message)
    }
}

impl std::error::Error for Error {}

fn obfuscate_one(file: &str, source: &str, opts: &Options, helper: &str) -> Result<String, Error> {
    let ast = full_moon::parse(source).map_err(|errors| Error {
        file: file.to_string(),
        message: errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "),
    })?;
    let plan = resolve::plan(&ast, opts);
    Ok(emit::emit(&ast, &plan, helper))
}

/// Collects the identifiers that stay global in any of the sources, so the bundler can choose
/// helper names that do not clash with them.
fn globals_of(sources: &[&str]) -> std::collections::BTreeSet<String> {
    let mut set = std::collections::BTreeSet::new();
    let scan = Options::level(1);
    for src in sources {
        if let Ok(ast) = full_moon::parse(src) {
            set.extend(resolve::plan(&ast, &scan).globals);
        }
    }
    set
}

/// Compiles an entry script and its own modules into one obfuscated chunk.
///
/// `require` of one of `modules` is resolved to the bundled copy at run time; `require` of
/// anything else is left to the game (which, in the plugin sandbox, only allows the plugin's own
/// files anyway).
pub fn compile(entry_source: &str, modules: &[Module], opts: &Options) -> Result<String, Error> {
    // Validate every module parses first, with the clearest error.
    let mut all_sources: Vec<&str> = vec![entry_source];
    all_sources.extend(modules.iter().map(|m| m.source.as_str()));
    let globals = globals_of(&all_sources);
    let names = bundle::HelperNames::fresh(&globals);

    let entry = obfuscate_one("main", entry_source, opts, &names.string_helper)?;
    let mut compiled_modules = Vec::with_capacity(modules.len());
    for m in modules {
        let code = obfuscate_one(&m.require_name, &m.source, opts, &names.string_helper)?;
        compiled_modules.push((m.require_name.clone(), code));
    }

    Ok(bundle::assemble(&entry, &compiled_modules, &names))
}

/// Compiles a single script with no modules.
pub fn compile_single(source: &str, opts: &Options) -> Result<String, Error> {
    compile(source, &[], opts)
}
