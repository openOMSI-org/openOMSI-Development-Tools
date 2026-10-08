//! A small Markdown renderer for egui - enough for the guide pages: headings, paragraphs, bullet
//! lists, fenced code blocks, inline code and bold. Not a full CommonMark implementation; the
//! docs are written to stay within what it draws.

use egui::{Color32, FontId, RichText, Ui};

/// Renders a Markdown string into the given `Ui`.
pub fn render(ui: &mut Ui, markdown: &str, accent: Color32) {
    let mut in_code = false;
    let mut code = String::new();
    for line in markdown.lines() {
        if line.trim_start().starts_with("```") {
            if in_code {
                code_block(ui, &code);
                code.clear();
                in_code = false;
            } else {
                in_code = true;
            }
            continue;
        }
        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            ui.add_space(6.0);
        } else if let Some(h) = trimmed.strip_prefix("### ") {
            ui.add_space(4.0);
            ui.label(RichText::new(h).size(16.0).strong().color(accent));
        } else if let Some(h) = trimmed.strip_prefix("## ") {
            ui.add_space(8.0);
            ui.label(RichText::new(h).size(19.0).strong().color(accent));
        } else if let Some(h) = trimmed.strip_prefix("# ") {
            ui.add_space(4.0);
            ui.label(RichText::new(h).size(24.0).strong());
        } else if trimmed.starts_with("| ") {
            // A Markdown table: show its rows as monospace, skipping the separator line.
            if !trimmed.trim_matches(|c| c == '|' || c == '-' || c == ' ' || c == ':').is_empty() {
                ui.label(RichText::new(trimmed).monospace());
            }
        } else if let Some(item) = trimmed.strip_prefix("* ").or_else(|| trimmed.strip_prefix("- ")) {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("  •").color(accent));
                inline(ui, item);
            });
        } else if let Some(item) = numbered(trimmed) {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("  {}.", item.0)).color(accent));
                inline(ui, item.1);
            });
        } else {
            ui.horizontal_wrapped(|ui| inline(ui, trimmed));
        }
    }
    if in_code && !code.is_empty() {
        code_block(ui, &code);
    }
}

fn numbered(line: &str) -> Option<(usize, &str)> {
    let dot = line.find(". ")?;
    let n: usize = line[..dot].parse().ok()?;
    Some((n, &line[dot + 2..]))
}

fn code_block(ui: &mut Ui, code: &str) {
    let bg = if ui.visuals().dark_mode { Color32::from_rgb(24, 26, 32) } else { Color32::from_rgb(240, 241, 245) };
    egui::Frame::new().fill(bg).inner_margin(8.0).corner_radius(6.0).show(ui, |ui| {
        ui.add(
            egui::Label::new(RichText::new(code.trim_end()).font(FontId::monospace(13.0)))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
    });
}

/// Renders inline text with `` `code` ``, `**bold**` and `[text](url)` link spans.
fn inline(ui: &mut Ui, text: &str) {
    let accent_code =
        if ui.visuals().dark_mode { Color32::from_rgb(244, 160, 110) } else { Color32::from_rgb(170, 70, 20) };
    let mut rest = text;
    while !rest.is_empty() {
        let code_at = rest.find('`');
        let link_at = link_start(rest);
        match (code_at, link_at) {
            (Some(c), l) if l.is_none_or(|x| c < x) => {
                if c > 0 {
                    emit_bold(ui, &rest[..c]);
                }
                if let Some(end) = rest[c + 1..].find('`') {
                    ui.label(RichText::new(&rest[c + 1..c + 1 + end]).monospace().color(accent_code));
                    rest = &rest[c + 1 + end + 1..];
                } else {
                    emit_bold(ui, &rest[c..]);
                    break;
                }
            }
            (_, Some(l)) => {
                if l > 0 {
                    emit_bold(ui, &rest[..l]);
                }
                let close = rest[l..].find("](").map(|x| x + l).unwrap();
                let text = &rest[l + 1..close];
                let url_end = rest[close + 2..].find(')').map(|x| x + close + 2).unwrap();
                let url = &rest[close + 2..url_end];
                ui.hyperlink_to(text, url);
                rest = &rest[url_end + 1..];
            }
            _ => {
                emit_bold(ui, rest);
                break;
            }
        }
    }
}

/// The byte index of the next `[text](url)` link, if any.
fn link_start(s: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(open) = s[from..].find('[') {
        let open = open + from;
        if let Some(close) = s[open..].find("](")
            && s[open + close + 2..].contains(')')
        {
            return Some(open);
        }
        from = open + 1;
    }
    None
}

fn emit_bold(ui: &mut Ui, text: &str) {
    let mut rest = text;
    while let Some(start) = rest.find("**") {
        if start > 0 {
            ui.label(rest[..start].to_string());
        }
        if let Some(end) = rest[start + 2..].find("**") {
            ui.label(RichText::new(&rest[start + 2..start + 2 + end]).strong());
            rest = &rest[start + 2 + end + 2..];
        } else {
            ui.label(rest[start..].to_string());
            return;
        }
    }
    if !rest.is_empty() {
        ui.label(rest.to_string());
    }
}
