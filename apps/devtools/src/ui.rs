//! The whole user interface, as one `draw` over [`AppState`] into a root [`egui::Ui`]. It uses
//! only egui, so the same code runs interactively under eframe and offscreen for screenshots.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Ui};

use crate::actions;
use crate::state::{AppState, LogLevel, Page, Theme};

/// openOMSI's amber accent.
const ACCENT: Color32 = Color32::from_rgb(0xF4, 0x7F, 0x30);

/// Applies the theme and draws every panel for the current frame into the root `ui`.
pub fn draw(ui: &mut Ui, state: &mut AppState) {
    apply_theme(ui, state.theme);
    top_bar(ui, state);
    nav(ui, state);
    status_bar(ui, state);
    egui::CentralPanel::default().show(ui, |ui| {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match state.page {
            Page::Projects => projects(ui, state),
            Page::Manifest => manifest(ui, state),
            Page::Build => build(ui, state),
            Page::Keys => keys(ui, state),
            Page::Inspector => inspector(ui, state),
            Page::Docs => docs(ui, state),
            Page::Settings => settings(ui, state),
        });
    });
}

fn apply_theme(ui: &Ui, theme: Theme) {
    let ctx = ui.ctx();
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.45);
    visuals.hyperlink_color = ACCENT;
    if theme == Theme::Dark {
        visuals.panel_fill = Color32::from_rgb(0x15, 0x17, 0x1C);
        visuals.window_fill = Color32::from_rgb(0x1B, 0x1E, 0x25);
    }
    ctx.set_visuals(visuals);
}

fn top_bar(ui: &mut Ui, state: &mut AppState) {
    egui::Panel::top("top").show(ui, |ui| {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("openOMSI").size(20.0).strong().color(ACCENT));
            ui.label(RichText::new("Development Tools").size(20.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (label, next) = match state.theme {
                    Theme::Dark => ("Light theme", Theme::Light),
                    Theme::Light => ("Dark theme", Theme::Dark),
                };
                if ui.button(label).clicked() {
                    state.theme = next;
                }
            });
        });
        ui.add_space(4.0);
    });
}

fn nav(ui: &mut Ui, state: &mut AppState) {
    egui::Panel::left("nav").resizable(false).show(ui, |ui| {
        ui.add_space(8.0);
        for page in Page::ALL {
            let selected = state.page == page;
            let text = RichText::new(page.title()).size(15.0);
            let text = if selected { text.strong().color(ACCENT) } else { text };
            if ui.add_sized([150.0, 30.0], egui::Button::selectable(selected, text)).clicked() {
                state.page = page;
            }
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(8.0);
            ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).weak());
            ui.hyperlink_to("openOMSI", "https://github.com/openOMSI-org/openOMSI");
        });
    });
}

fn status_bar(ui: &mut Ui, state: &AppState) {
    egui::Panel::bottom("status").show(ui, |ui| {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("●").color(if state.building {
                ACCENT
            } else {
                Color32::from_rgb(0x3A, 0xA6, 0x55)
            }));
            ui.label(&state.status);
        });
        ui.add_space(2.0);
    });
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(text).size(22.0).strong());
    ui.add_space(8.0);
}

fn card(ui: &mut Ui, add: impl FnOnce(&mut Ui)) {
    let fill =
        if ui.visuals().dark_mode { Color32::from_rgb(0x1B, 0x1E, 0x25) } else { Color32::from_rgb(0xF2, 0xF3, 0xF6) };
    Frame::NONE.fill(fill).inner_margin(Margin::same(14)).corner_radius(CornerRadius::same(10)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui);
    });
    ui.add_space(10.0);
}

fn projects(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Projects");
    card(ui, |ui| {
        ui.label(RichText::new("Open a project").strong());
        ui.horizontal(|ui| {
            let mut path = state.project_dir.clone().map(|p| p.display().to_string()).unwrap_or_default();
            if ui
                .add(egui::TextEdit::singleline(&mut path).hint_text("path to a plugin project").desired_width(360.0))
                .changed()
            {
                state.project_dir = Some(path.into());
            }
            if ui.button("Open").clicked() {
                actions::open_project(state);
            }
        });
    });
    card(ui, |ui| {
        ui.label(RichText::new("Start a new project").strong());
        ui.horizontal(|ui| {
            if ui.button("New Lua plugin").clicked() {
                actions::new_project(state, false);
            }
            if ui.button("New Rust plugin").clicked() {
                actions::new_project(state, true);
            }
        });
        ui.label(RichText::new("Templates include an openomsi-plugin.toml manifest and a starter script.").weak());
    });
    if !state.recent.is_empty() {
        card(ui, |ui| {
            ui.label(RichText::new("Recent").strong());
            for r in state.recent.clone() {
                if ui.link(r.display().to_string()).clicked() {
                    state.project_dir = Some(r.clone());
                    actions::open_project(state);
                }
            }
        });
    }
}

fn manifest(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Manifest");
    card(ui, |ui| {
        let m = &mut state.manifest;
        egui::Grid::new("manifest_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            field(ui, "Id", &mut m.id);
            field(ui, "Name", &mut m.name);
            field(ui, "Version", &mut m.version);
            field(ui, "Description", &mut m.description);
            field(ui, "Authors", &mut m.authors);
            ui.label("Language");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut m.is_rust, false, "Lua");
                ui.selectable_value(&mut m.is_rust, true, "Rust (wasm)");
            });
            ui.end_row();
            if !m.is_rust {
                field(ui, "Entry", &mut m.entry);
            }
            field(ui, "Min openOMSI", &mut m.min_openomsi);
            field(ui, "License", &mut m.license);
            field(ui, "Homepage", &mut m.homepage);
        });
    });
    card(ui, |ui| {
        ui.label(RichText::new("Permissions").strong());
        ui.label(RichText::new("Only what you tick is allowed in the game.").weak());
        ui.add_space(4.0);
        for (name, desc, on) in &mut state.manifest.permissions {
            ui.checkbox(on, format!("{name} — {desc}"));
        }
    });
    if !state.manifest.is_rust {
        card(ui, |ui| {
            ui.label(RichText::new("Lua obfuscation").strong());
            ui.horizontal(|ui| {
                ui.radio_value(&mut state.manifest.obfuscation_level, 0u8, "0 strip");
                ui.radio_value(&mut state.manifest.obfuscation_level, 1, "1 rename");
                ui.radio_value(&mut state.manifest.obfuscation_level, 2, "2 full");
            });
        });
    }
    if ui.button(RichText::new("Save manifest").strong()).clicked() {
        actions::save_manifest(state);
    }
}

fn field(ui: &mut Ui, label: &str, value: &mut String) {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).desired_width(360.0));
    ui.end_row();
}

fn build(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Build");
    card(ui, |ui| {
        ui.horizontal(|ui| {
            if ui.add_enabled(!state.building, egui::Button::new(RichText::new("Build").strong())).clicked() {
                actions::build(state);
            }
            if ui.button("Build & install to game").clicked() {
                actions::build_and_install(state);
            }
            ui.label(
                state.project_dir.clone().map(|p| p.display().to_string()).unwrap_or_else(|| "no project open".into()),
            );
        });
    });
    card(ui, |ui| {
        ui.label(RichText::new("Log").strong());
        ui.add_space(4.0);
        let bg = if ui.visuals().dark_mode {
            Color32::from_rgb(0x10, 0x12, 0x17)
        } else {
            Color32::from_rgb(0xFA, 0xFA, 0xFC)
        };
        Frame::NONE.fill(bg).inner_margin(Margin::same(8)).corner_radius(CornerRadius::same(6)).show(ui, |ui| {
            ui.set_min_height(160.0);
            ui.set_width(ui.available_width());
            if state.build_log.is_empty() {
                ui.label(RichText::new("Press Build to compile and pack the plugin.").weak());
            }
            for (level, line) in &state.build_log {
                let color = match level {
                    LogLevel::Info => ui.visuals().text_color(),
                    LogLevel::Warn => Color32::from_rgb(0xE0, 0xA0, 0x30),
                    LogLevel::Error => Color32::from_rgb(0xE2, 0x3A, 0x3A),
                    LogLevel::Good => Color32::from_rgb(0x3A, 0xA6, 0x55),
                };
                ui.label(RichText::new(line).monospace().color(color));
            }
        });
    });
}

fn keys(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Signing keys");
    card(ui, |ui| {
        ui.label(RichText::new("Signing proves who built a plugin. Keep the secret key private.").weak());
        ui.horizontal(|ui| {
            ui.label("Label");
            ui.add(egui::TextEdit::singleline(&mut state.new_key_label).desired_width(240.0));
            if ui.button("Create key").clicked() {
                actions::create_key(state);
            }
            if ui.button("Import key").clicked() {
                actions::import_key(state);
            }
        });
    });
    card(ui, |ui| {
        ui.label(RichText::new("Your keys").strong());
        if state.keys.is_empty() {
            ui.label(RichText::new("No keys yet.").weak());
        }
        for k in &state.keys {
            ui.horizontal(|ui| {
                ui.label("🔑");
                ui.label(RichText::new(&k.label).strong());
                ui.label(RichText::new(&k.fingerprint).monospace().color(ACCENT));
            });
            ui.label(RichText::new(k.path.display().to_string()).weak().small());
            ui.separator();
        }
    });
}

fn inspector(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Inspector");
    card(ui, |ui| {
        ui.label(RichText::new("Open a .oop to read its header and file list (never its contents).").weak());
        ui.horizontal(|ui| {
            let mut path = state.inspector_path.clone().map(|p| p.display().to_string()).unwrap_or_default();
            if ui
                .add(egui::TextEdit::singleline(&mut path).hint_text("path to a .oop file").desired_width(360.0))
                .changed()
            {
                state.inspector_path = Some(path.into());
            }
            if ui.button("Inspect").clicked() {
                actions::inspect(state);
            }
        });
    });
    if let Some(report) = state.inspector_report.clone() {
        card(ui, |ui| {
            egui::Grid::new("inspect_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                for (k, v) in &report.lines {
                    ui.label(RichText::new(k).strong());
                    ui.label(v);
                    ui.end_row();
                }
                ui.label(RichText::new("Signature").strong());
                let col =
                    if report.ok { Color32::from_rgb(0x3A, 0xA6, 0x55) } else { Color32::from_rgb(0xE2, 0x3A, 0x3A) };
                ui.label(RichText::new(&report.signature).color(col));
                ui.end_row();
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Files").strong());
            for (path, size) in &report.files {
                ui.label(RichText::new(format!("{size:>10}  {path}")).monospace());
            }
        });
    }
}

fn docs(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Documentation");
    ui.horizontal(|ui| {
        ui.label("Search API");
        ui.add(
            egui::TextEdit::singleline(&mut state.docs_search).hint_text("e.g. speed, ui, var").desired_width(260.0),
        );
    });
    ui.separator();
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(165.0);
            for (i, page) in state.docs.pages.iter().enumerate() {
                if ui
                    .add_sized([150.0, 26.0], egui::Button::selectable(state.docs_selected == i, &page.title))
                    .clicked()
                {
                    state.docs_selected = i;
                }
            }
        });
        ui.separator();
        ui.vertical(|ui| {
            let search = state.docs_search.to_lowercase();
            if !search.is_empty() {
                ui.label(RichText::new("API reference").size(19.0).strong().color(ACCENT));
                ui.add_space(6.0);
                let mut shown = 0;
                for entry in &state.docs.api {
                    if entry.name.to_lowercase().contains(&search) || entry.doc.to_lowercase().contains(&search) {
                        api_entry(ui, entry);
                        shown += 1;
                    }
                }
                if shown == 0 {
                    ui.label(RichText::new("No API function matches.").weak());
                }
            } else if let Some(page) = state.docs.pages.get(state.docs_selected) {
                crate::md::render(ui, &page.body, ACCENT);
            }
        });
    });
}

fn api_entry(ui: &mut Ui, entry: &crate::docs::ApiEntry) {
    card(ui, |ui| {
        ui.label(RichText::new(&entry.signature).monospace().strong().color(ACCENT));
        ui.label(&entry.doc);
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("group: {}", entry.group)).weak().small());
            if let Some(p) = &entry.permission {
                ui.label(RichText::new(format!("permission: {p}")).small().color(ACCENT));
            }
        });
    });
}

fn settings(ui: &mut Ui, state: &mut AppState) {
    heading(ui, "Settings");
    card(ui, |ui| {
        egui::Grid::new("settings_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            ui.label("Path to cargo");
            ui.add(egui::TextEdit::singleline(&mut state.settings.cargo_path).desired_width(360.0));
            ui.end_row();
            ui.label("openOMSI game folder");
            ui.add(
                egui::TextEdit::singleline(&mut state.settings.game_folder)
                    .hint_text("so Build can install into plugins/")
                    .desired_width(360.0),
            );
            ui.end_row();
        });
        ui.label(RichText::new("\"Install to game\" copies the built .oop into this folder's plugins/.").weak());
    });
}
