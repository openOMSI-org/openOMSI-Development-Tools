//! `oopc`, the openOMSI plugin compiler.
//!
//! It turns a plugin project (Lua sources or a Rust crate) into a `.oop` file: a compiled,
//! encrypted and optionally signed artefact that openOMSI loads from its `plugins` folder. The
//! sources never go into the file - Lua is run through an obfuscating compiler, Rust is compiled
//! to a stripped WebAssembly module.

use oopc::commands;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// The openOMSI plugin compiler.
#[derive(Parser)]
#[command(name = "oopc", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start a new plugin project from a template.
    New {
        /// The directory to create.
        name: PathBuf,
        /// A Lua plugin (the default).
        #[arg(long, group = "lang")]
        lua: bool,
        /// A Rust plugin, compiled to WebAssembly.
        #[arg(long, group = "lang")]
        rust: bool,
    },
    /// Compile a project and pack it into a signed-or-unsigned `.oop`.
    Build {
        /// The project directory (default: the current directory).
        dir: Option<PathBuf>,
        /// Where to write the `.oop` (default: `<dir>/dist/<id>.oop`).
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Sign with this key file.
        #[arg(long)]
        sign: Option<PathBuf>,
    },
    /// Pack a directory of already-prepared files into a `.oop` (a lower-level tool; `build` is
    /// the usual command and compiles the sources first).
    Pack {
        /// The directory of files to pack.
        dir: PathBuf,
        /// The project manifest to take the header from (default: `<dir>/openomsi-plugin.toml`).
        #[arg(long)]
        manifest: Option<PathBuf>,
        /// Where to write the `.oop`.
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Sign with this key file.
        #[arg(long)]
        sign: Option<PathBuf>,
    },
    /// Make a new signing key.
    Keygen {
        /// A label stored in the key file's comment.
        #[arg(long, default_value = "my openOMSI key")]
        label: String,
        /// Where to write the secret key (default: the user's config dir).
        #[arg(short, long)]
        out: Option<PathBuf>,
        /// Overwrite an existing key file.
        #[arg(long)]
        force: bool,
    },
    /// Sign (or re-sign) a `.oop` with a key.
    Sign {
        /// The `.oop` file.
        file: PathBuf,
        /// The secret key file.
        #[arg(long)]
        key: PathBuf,
    },
    /// Check a `.oop`'s signature and show who signed it.
    Verify {
        /// The `.oop` file.
        file: PathBuf,
        /// A public key file the signer must match.
        #[arg(long)]
        key: Option<PathBuf>,
    },
    /// Show a `.oop`'s header and the files it holds (sizes only, never contents).
    Inspect {
        /// The `.oop` file.
        file: PathBuf,
        /// Print the header as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Lint a project: version and id formats, the entry file, and declared vs used permissions.
    Check {
        /// The project directory (default: the current directory).
        dir: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::New { name, lua, rust } => commands::new_project(&name, rust && !lua),
        Command::Build { dir, out, sign } => commands::build(dir.as_deref(), out.as_deref(), sign.as_deref()),
        Command::Pack { dir, manifest, out, sign } => {
            commands::pack(&dir, manifest.as_deref(), out.as_deref(), sign.as_deref())
        }
        Command::Keygen { label, out, force } => commands::keygen(&label, out.as_deref(), force),
        Command::Sign { file, key } => commands::sign(&file, &key),
        Command::Verify { file, key } => commands::verify(&file, key.as_deref()),
        Command::Inspect { file, json } => commands::inspect(&file, json),
        Command::Check { dir } => commands::check(dir.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
