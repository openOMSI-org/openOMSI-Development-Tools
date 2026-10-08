//! A HUD panel that shows the current speed and turns red over the limit. It also shows how to
//! build an `omsi.ui` panel from the Rust SDK (the panel table is a plain JSON value).
//!
//! Build it with `oopc build examples/speed-warner`.

use openomsi_plugin_sdk as omsi;
use omsi::json;

const LIMIT_KMH: f64 = 50.0;

fn redraw() {
    let kmh = omsi::api::speed().unwrap_or(0.0);
    let over = kmh > LIMIT_KMH;
    let accent = if over { "#E23A3A" } else { "#3AA655" };
    let panel = json!({
        "anchor": "top_left",
        "x": 16, "y": 60, "width": 220,
        "accent": accent,
        "children": [
            { "type": "row", "children": [
                { "type": "icon", "name": "speed", "color": accent },
                { "type": "text", "text": "Speed", "weight": "bold", "grow": true },
                { "type": "badge", "text": format!("{kmh:.0} km/h"), "color": accent },
            ] },
            { "type": "bar", "value": (kmh / 100.0).clamp(0.0, 1.0), "color": accent },
            { "type": "text", "text": if over { "Over the limit!" } else { "Within the limit" } },
        ],
    });
    let _ = omsi::api::ui::set("speed", panel);
}

omsi::plugin!(|| {
    // Twice a second is plenty for a speed readout; on_frame would be wasteful.
    omsi::every(0.5, redraw);

    // Tidy up the panel when the player leaves the bus.
    omsi::on_vehicle(|name| {
        if name.is_none() {
            let _ = omsi::api::ui::remove("speed");
        }
    });
});
