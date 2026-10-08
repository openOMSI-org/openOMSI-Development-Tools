//! Announces each new bus stop on the screen and keeps a count across sessions. It shows watches,
//! saved data (storage) and a notification.
//!
//! Build it with `oopc build examples/stop-announcer`.

use openomsi_plugin_sdk as omsi;
use omsi::{Value, json};

omsi::plugin!(|| {
    omsi::on_start(|| {
        let _ = omsi::api::message("Stop announcer ready", Some(3.0));
    });

    // Whenever the IBIS next-stop string changes, announce it.
    omsi::watch("str", "IBIS_busstop_name", |new: Value, _old: Value| {
        if let Some(stop) = new.as_str()
            && !stop.is_empty()
        {
            let _ = omsi::api::ui::toast(
                &format!("Next stop: {stop}"),
                Some(json!({ "title": "Announcer", "icon": "directions_bus", "seconds": 5 })),
            );
        }
    });
});
