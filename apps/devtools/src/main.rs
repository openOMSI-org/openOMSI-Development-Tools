//! The openOMSI Development Tools desktop app.
//!
//! With no arguments it opens the window. With `--screenshot <file.png>` it renders a page
//! off-screen and writes a PNG - no window appears - which is how its pages are checked on build
//! servers and in development. `--all <dir>` renders every page.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod docs;
mod md;
mod render;
mod state;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
mod app;

use std::path::PathBuf;
use std::process::ExitCode;

use state::{Page, Theme};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return run_window();
    }
    match run_cli(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_window() -> ExitCode {
    match app::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn run_window() -> ExitCode {
    eprintln!("the interactive app is not available on this target");
    ExitCode::FAILURE
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn run_cli(args: &[String]) -> Result<(), String> {
    let theme = match flag(args, "--theme") {
        Some("light") => Theme::Light,
        _ => Theme::Dark,
    };
    let width: u32 = flag(args, "--width").and_then(|s| s.parse().ok()).unwrap_or(1100);
    let height: u32 = flag(args, "--height").and_then(|s| s.parse().ok()).unwrap_or(740);
    let ppp: f32 = flag(args, "--scale").and_then(|s| s.parse().ok()).unwrap_or(1.5);

    if let Some(dir) = flag(args, "--all") {
        let dir = PathBuf::from(dir);
        for page in Page::ALL {
            let out = dir.join(format!("{}.png", page.slug()));
            render::screenshot(page, theme, width, height, ppp, &out)?;
            println!("wrote {}", out.display());
        }
        return Ok(());
    }

    if let Some(out) = flag(args, "--screenshot") {
        let page = flag(args, "--page").and_then(Page::from_slug).unwrap_or(Page::Projects);
        render::screenshot(page, theme, width, height, ppp, &PathBuf::from(out))?;
        println!("wrote {out}");
        return Ok(());
    }

    Err("unknown arguments; use --screenshot <file.png> [--page <name>] or --all <dir>".to_string())
}
