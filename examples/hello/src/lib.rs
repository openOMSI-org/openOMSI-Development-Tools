//! The smallest openOMSI plugin: it greets the driver and warns above 50 km/h.
//!
//! Build it into a `.oop` with `oopc build examples/hello`.

use openomsi_plugin_sdk as omsi;

omsi::plugin!(|| {
    // When the player gets into a bus, greet them by its name.
    omsi::on_vehicle(|name| {
        if let Some(name) = name {
            let _ = omsi::api::message(&format!("Good morning! Today you drive the {name}"), Some(6.0));
        }
    });

    // Once a second, warn if the bus is going too fast.
    omsi::every(1.0, || {
        if let Ok(Some(kmh)) = omsi::api::var("Velocity")
            && kmh > 50.0
        {
            let _ = omsi::api::message(&format!("Slow down: {kmh:.0} km/h"), Some(1.0));
        }
    });
});
