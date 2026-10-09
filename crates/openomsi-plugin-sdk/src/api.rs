//! Typed wrappers for the openOMSI plugin API, generated from `api.json`
//! (API version 0.2.22, ABI 1) by `cargo xtask gen-sdk`. Do not edit by hand:
//! run the command again against a newer manifest instead.
//!
//! Every function calls into the game through [`crate::call`]; a function the manifest
//! does not have yet is still reachable with that escape hatch.

#![allow(clippy::all)]
use crate::{Result, call};
use serde_json::{Value, json};

/// The game's time of day as `"HH:MM:SS"`.
///
/// Returns: string
pub fn clock() -> Result<String> {
    Ok(call("clock", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
}

/// Does what a line of the game menu does: `refuel`, `wash`, `repair`, `shot`, `save`, `load`,
/// `weather`, `later`, `earlier`, `info`, `timetable`, `reset`, `couple`, `uncouple`; `true`
/// when the game knows it (it runs after the frame).
///
/// Returns: boolean
pub fn command(name: &str) -> Result<bool> {
    Ok(call("command", json!([json!(name)]))?.as_bool().unwrap_or(false))
}

/// Metres from the bus to a map point, or `nil` on foot.
///
/// Returns: number or nil
pub fn distance(x: f64, y: f64) -> Result<Option<f64>> {
    Ok(call("distance", json!([json!(x), json!(y)]))?.as_f64())
}

/// `true` while the player drives a vehicle.
///
/// Returns: boolean
pub fn has_vehicle() -> Result<bool> {
    Ok(call("has_vehicle", Value::Array(vec![]))?.as_bool().unwrap_or(false))
}

/// What the game is doing: `map`, `clock` (seconds since midnight), `day`, `year`, `view`,
/// `paused`, `on_foot`, `multiplayer`, `traffic`, `speed`, `delay`, `map_path`, `version`; with
/// a bus also `tile_x`, `tile_y`, `tile_pos_x`, `tile_pos_y`, `heading`,
/// `vehicle_manufacturer`, `vehicle_model`, `destination`, `passengers`; `crashes`,
/// `heavy_crashes`, `pedestrians_hit`; `situation`; on a duty also `line`, `tour`, `trip`,
/// `trips`, `trip_name`, `terminus`, `stops`, `trip_done`, `next_stop`, `next_stop_number`,
/// `next_stop_arrival`, `next_stop_departure`, `next_stop_id`, `at_stop`, `previous_stop`,
/// `previous_stop_id`, `next_stop_distance`, `previous_stop_distance` (see the plugin docs for
/// each).
///
/// Returns: table
pub fn info() -> Result<Value> {
    call("info", Value::Array(vec![]))
}

/// One value of `info()` without building the whole table: cheaper for a plugin that reads one
/// or two every frame.
///
/// Returns: any
pub fn info_value(key: &str) -> Result<Value> {
    call("info_value", json!([json!(key)]))
}

/// A line of text on the screen (5 seconds when not given).
///
/// Returns: nil
pub fn message(text: &str, seconds: Option<f64>) -> Result<()> {
    call("message", json!([json!(text), match seconds { Some(v) => json!(v), None => Value::Null }]))?; Ok(())
}

/// The other vehicles within `radius` m of the bus (default 300): each `{id, kind, name, x, y,
/// z, heading}`, `kind` being `"ai"` (the traffic) or `"player"` (another player's bus in a LAN
/// game); empty on foot.
///
/// Returns: list of tables
pub fn others(radius: Option<f64>) -> Result<Vec<Value>> {
    Ok(call("others", json!([match radius { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
}

/// Where the bus is: map metres (x east, y north, z up) and its heading in degrees clockwise
/// from north; nothing on foot.
///
/// Returns: x, y, z, heading
pub fn position() -> Result<Vec<Value>> {
    Ok(call("position", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
}

/// Holds a key of the bus down: fires the trigger `name` (let it go with `release`).
///
/// Returns: nil
pub fn press(name: &str) -> Result<()> {
    call("press", json!([json!(name)]))?; Ok(())
}

/// Lets a key go: fires `<name>_off`.
///
/// Returns: nil
pub fn release(name: &str) -> Result<()> {
    call("release", json!([json!(name)]))?; Ok(())
}

/// Sets a string variable; `true` when the bus has it.
///
/// Returns: boolean
pub fn set_str(name: &str, text: &str) -> Result<bool> {
    Ok(call("set_str", json!([json!(name), json!(text)]))?.as_bool().unwrap_or(false))
}

/// Sets a script variable; `true` when the bus has that variable.
///
/// Returns: boolean
pub fn set_var(name: &str, value: f64) -> Result<bool> {
    Ok(call("set_var", json!([json!(name), json!(value)]))?.as_bool().unwrap_or(false))
}

/// The bus's speed in km/h, forwards or backwards (0 on foot).
///
/// Returns: number
pub fn speed() -> Result<f64> {
    Ok(call("speed", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
}

/// A string variable of the bus's scripts (`IBIS_terminus_name`).
///
/// Returns: string or nil
pub fn str(name: &str) -> Result<Option<String>> {
    Ok(call("str", json!([json!(name)]))?.as_str().map(str::to_string))
}

/// A system variable of the scripts: `Time`, `Day`, `Weather_Temperature`, `SunAlt`, ... (read
/// only).
///
/// Returns: number or nil
pub fn sys(name: &str) -> Result<Option<f64>> {
    Ok(call("sys", json!([json!(name)]))?.as_f64())
}

/// Seconds of game time since the plugin started (stands still while paused).
///
/// Returns: number
pub fn time() -> Result<f64> {
    Ok(call("time", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
}

/// A key press of the bus: fires the trigger, then `<name>_off`.
///
/// Returns: nil
pub fn trigger(name: &str) -> Result<()> {
    call("trigger", json!([json!(name)]))?; Ok(())
}

/// A script variable of the bus (`Velocity`, `elec_busbar_main`, the names the `.osc` files and
/// `.opl` lists use); `nil` without a bus or for a name it does not have.
///
/// Returns: number or nil
pub fn var(name: &str) -> Result<Option<f64>> {
    Ok(call("var", json!([json!(name)]))?.as_f64())
}

/// The names of every variable of the bus's scripts, or of every string variable with `"str"`.
///
/// Returns: list of strings
pub fn vars(kind: Option<&str>) -> Result<Vec<Value>> {
    Ok(call("vars", json!([match kind { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
}

/// The vehicle's name (manufacturer and type), or `nil` on foot.
///
/// Returns: string or nil
pub fn vehicle() -> Result<Option<String>> {
    Ok(call("vehicle", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// The manufacturer part of the vehicle's name, as its `[friendlyname]` has it (`"Solaris III
/// Gen"`).
///
/// Returns: string or nil
pub fn vehicle_manufacturer() -> Result<Option<String>> {
    Ok(call("vehicle_manufacturer", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// The model part of the vehicle's name (`"Urbino 10 / 2D"`).
///
/// Returns: string or nil
pub fn vehicle_model() -> Result<Option<String>> {
    Ok(call("vehicle_model", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// The `audio` group of the API.
pub mod audio {
    use super::*;
    /// Plays a WAV file of the plugin's folder. `opts`: `volume` (1), `pitch` (1), `loop`, `range`
    /// (metres heard at full volume, 5), and either `x, y, z` (a sound at a map point) or `on_bus =
    /// true` (it moves with the player's bus); none: heard alike everywhere. The game's volume
    /// setting applies. A plugin plays 32 at most.
    ///
    /// Returns: integer id or nil, reason
    pub fn play(file: &str, opts: Option<serde_json::Value>) -> Result<Vec<Value>> {
        Ok(call("audio.play", json!([json!(file), match opts { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// Stops every sound of the plugin's.
    ///
    /// Returns: nil
    pub fn stop_all() -> Result<()> {
        call("audio.stop_all", Value::Array(vec![]))?; Ok(())
    }

    /// The game's volume setting, 0 to 1.
    ///
    /// Returns: number or nil
    pub fn volume() -> Result<Option<f64>> {
        Ok(call("audio.volume", Value::Array(vec![]))?.as_f64())
    }

}

/// The `bus` group of the API.
pub mod bus {
    use super::*;
    /// Its acceleration in its own frame, m/s² without gravity: to the right, forwards, up.
    ///
    /// Returns: across, along, up
    pub fn acceleration() -> Result<Vec<Value>> {
        Ok(call("bus.acceleration", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// A key action of the vehicles (`[vehicles]` of keyboard.cfg: `horn`, `parking_brake_toggle`,
    /// `kw_scheinwerfer_toggle`, `blinker_left_set`, `ticket_give`, ...): pressed and let go, or
    /// held (`down` true) and let go (`false`). `true` when the bus knows it.
    ///
    /// Returns: boolean
    pub fn action(name: &str, down: Option<bool>) -> Result<bool> {
        Ok(call("bus.action", json!([json!(name), match down { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

    /// What the bus drives with now: the pedals (0 to 1) and the steering (-1 left to 1 right),
    /// from the keys, the mouse or a controller.
    ///
    /// Returns: throttle, brake, clutch, steering
    pub fn controls() -> Result<Vec<Value>> {
        Ok(call("bus.controls", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// `{crashes, last_impact_kj, repair_minutes}`: the bus's crashes, the energy of the last one
    /// and how long a repair would take (`nil`: nothing to repair).
    ///
    /// Returns: table or nil
    pub fn damage() -> Result<Value> {
        call("bus.damage", Value::Array(vec![]))
    }

    /// The destination the bus shows and its index in `destinations` (nothing when none).
    ///
    /// Returns: name, index
    pub fn destination() -> Result<Vec<Value>> {
        Ok(call("bus.destination", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The destinations of the bus's depot file: `{index, code, name, all_exit}` (`index` from 1,
    /// for `set_destination`).
    ///
    /// Returns: list of tables
    pub fn destinations() -> Result<Vec<Value>> {
        Ok(call("bus.destinations", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// How dirty the bus is, 0 (clean) to 1.
    ///
    /// Returns: number or nil
    pub fn dirt() -> Result<Option<f64>> {
        Ok(call("bus.dirt", Value::Array(vec![]))?.as_f64())
    }

    /// The doorways the door keys work, front to back.
    ///
    /// Returns: integer
    pub fn door_count() -> Result<f64> {
        Ok(call("bus.door_count", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
    }

    /// Each door leaf's position, front to back: 0 shut, 1 open (the scripts' `door_0`, `door_1`,
    /// ...).
    ///
    /// Returns: list of numbers
    pub fn doors() -> Result<Vec<Value>> {
        Ok(call("bus.doors", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Whether any door is open (by the passengers' door flags where the bus has them).
    ///
    /// Returns: boolean or nil
    pub fn doors_open() -> Result<bool> {
        Ok(call("bus.doors_open", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Whether the bus's electrics are on (the main switch).
    ///
    /// Returns: boolean or nil
    pub fn electrics() -> Result<bool> {
        Ok(call("bus.electrics", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The engine: whether it runs, its rpm (where the bus shows one) and whether the electrics are
    /// on.
    ///
    /// Returns: running, rpm, electrics
    pub fn engine() -> Result<Vec<Value>> {
        Ok(call("bus.engine", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Whether the engine runs.
    ///
    /// Returns: boolean or nil
    pub fn engine_running() -> Result<bool> {
        Ok(call("bus.engine_running", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The bus's `.bus` file, relative to the game folder.
    ///
    /// Returns: string or nil
    pub fn file() -> Result<Option<String>> {
        Ok(call("bus.file", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The fuel in the tank, litres (the scripts' `engine_tank_content`).
    ///
    /// Returns: number or nil
    pub fn fuel() -> Result<Option<f64>> {
        Ok(call("bus.fuel", Value::Array(vec![]))?.as_f64())
    }

    /// The gear engaged (-1 reverse, 0 neutral), where the bus shows one.
    ///
    /// Returns: integer or nil
    pub fn gear() -> Result<Option<f64>> {
        Ok(call("bus.gear", Value::Array(vec![]))?.as_f64())
    }

    /// The same for string variables.
    ///
    /// Returns: table
    pub fn get_strings(names: Option<serde_json::Value>) -> Result<Value> {
        call("bus.get_strings", json!([match names { Some(v) => json!(v), None => Value::Null }]))
    }

    /// Many script variables at once: a table name -> value of the names given, or of every
    /// variable of the bus without a list.
    ///
    /// Returns: table
    pub fn get_vars(names: Option<serde_json::Value>) -> Result<Value> {
        call("bus.get_vars", json!([match names { Some(v) => json!(v), None => Value::Null }]))
    }

    /// Whether the parking brake is on.
    ///
    /// Returns: boolean or nil
    pub fn handbrake() -> Result<bool> {
        Ok(call("bus.handbrake", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The headlights: 0 off, 1 side lights, 2 dipped, 3 high beam.
    ///
    /// Returns: integer or nil
    pub fn headlights() -> Result<Option<f64>> {
        Ok(call("bus.headlights", Value::Array(vec![]))?.as_f64())
    }

    /// Whether the horn sounds.
    ///
    /// Returns: boolean or nil
    pub fn horn() -> Result<bool> {
        Ok(call("bus.horn", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The indicators: `"off"`, `"left"`, `"right"` or `"hazard"`.
    ///
    /// Returns: string or nil
    pub fn indicator() -> Result<Option<String>> {
        Ok(call("bus.indicator", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The passenger room's light, 0 (off) to 1.
    ///
    /// Returns: number or nil
    pub fn interior_light() -> Result<Option<f64>> {
        Ok(call("bus.interior_light", Value::Array(vec![]))?.as_f64())
    }

    /// Kilometres driven this session (as the personnel file counts them).
    ///
    /// Returns: number or nil
    pub fn km_today() -> Result<Option<f64>> {
        Ok(call("bus.km_today", Value::Array(vec![]))?.as_f64())
    }

    /// Whether the bus kneels.
    ///
    /// Returns: boolean or nil
    pub fn kneeling() -> Result<bool> {
        Ok(call("bus.kneeling", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The bus's mass in kg.
    ///
    /// Returns: number or nil
    pub fn mass() -> Result<Option<f64>> {
        Ok(call("bus.mass", Value::Array(vec![]))?.as_f64())
    }

    /// The bus's fleet number.
    ///
    /// Returns: string or nil
    pub fn number() -> Result<Option<String>> {
        Ok(call("bus.number", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The bus's odometer in km.
    ///
    /// Returns: number or nil
    pub fn odometer() -> Result<Option<f64>> {
        Ok(call("bus.odometer", Value::Array(vec![]))?.as_f64())
    }

    /// Heading (degrees clockwise from north), pitch (nose up positive) and bank (right side down
    /// positive).
    ///
    /// Returns: heading, pitch, bank
    pub fn orientation() -> Result<Vec<Value>> {
        Ok(call("bus.orientation", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The people aboard the bus: all, sitting, standing.
    ///
    /// Returns: total, seated, standing
    pub fn passengers() -> Result<Vec<Value>> {
        Ok(call("bus.passengers", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Plays the bus's own sound of this event (a trigger name of its sound files, `ev_...`).
    ///
    /// Returns: boolean
    pub fn play_sound(event: &str) -> Result<bool> {
        Ok(call("bus.play_sound", json!([json!(event)]))?.as_bool().unwrap_or(false))
    }

    /// The retarder's step, where the bus has one.
    ///
    /// Returns: number or nil
    pub fn retarder() -> Result<Option<f64>> {
        Ok(call("bus.retarder", Value::Array(vec![]))?.as_f64())
    }

    /// The engine's revolutions per minute.
    ///
    /// Returns: number or nil
    pub fn rpm() -> Result<Option<f64>> {
        Ok(call("bus.rpm", Value::Array(vec![]))?.as_f64())
    }

    /// Tickets sold this session and the money taken (the game knows no currency).
    ///
    /// Returns: tickets, money
    pub fn sales() -> Result<Vec<Value>> {
        Ok(call("bus.sales", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Sets the indicators: `"off"`, `"left"`, `"right"` or `"hazard"`.
    ///
    /// Returns: boolean
    pub fn set_indicator(state: &str) -> Result<bool> {
        Ok(call("bus.set_indicator", json!([json!(state)]))?.as_bool().unwrap_or(false))
    }

    /// Switches the passenger room's light on or off.
    ///
    /// Returns: boolean
    pub fn set_interior_light(on: bool) -> Result<bool> {
        Ok(call("bus.set_interior_light", json!([json!(on)]))?.as_bool().unwrap_or(false))
    }

    /// Types a line (route) into the bus's IBIS, as the player would.
    ///
    /// Returns: boolean
    pub fn set_line(line: &str) -> Result<bool> {
        Ok(call("bus.set_line", json!([json!(line)]))?.as_bool().unwrap_or(false))
    }

    /// Sets many script variables at once (a table name -> number); how many the bus has.
    ///
    /// Returns: integer
    pub fn set_vars(values: serde_json::Value) -> Result<f64> {
        Ok(call("bus.set_vars", json!([json!(values)]))?.as_f64().unwrap_or(0.0))
    }

    /// Holds the horn (`true`) or lets it go.
    ///
    /// Returns: boolean
    pub fn sound_horn(down: bool) -> Result<bool> {
        Ok(call("bus.sound_horn", json!([json!(down)]))?.as_bool().unwrap_or(false))
    }

    /// Starts the bus up the way Shift+U does (the battery, the electrics, the engine) - or shuts a
    /// running bus down; what the game says it does.
    ///
    /// Returns: string or nil
    pub fn start_up() -> Result<Option<String>> {
        Ok(call("bus.start_up", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// Everything a dashboard shows, in one table: `speed` (km/h, signed), `gear`, `engine`
    /// (running), `rpm`, `electrics`, `doors_open`, `indicator`, `headlights`, `dirt`,
    /// `passengers`, `odometer`, `km_today`, `throttle`, `brake`, `clutch`, `steering`, `fuel`,
    /// `handbrake`, `horn`, `stop_request` (a key is `nil` where the bus does not have it).
    ///
    /// Returns: table or nil
    pub fn state() -> Result<Value> {
        call("bus.state", Value::Array(vec![]))
    }

    /// The front wheels' angle and the most they turn, degrees (right positive).
    ///
    /// Returns: angle, max
    pub fn steering_angle() -> Result<Vec<Value>> {
        Ok(call("bus.steering_angle", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Whether the stop brake (the door brake) holds the bus.
    ///
    /// Returns: boolean or nil
    pub fn stop_brake() -> Result<bool> {
        Ok(call("bus.stop_brake", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Whether a passenger has asked to stop.
    ///
    /// Returns: boolean or nil
    pub fn stop_requested() -> Result<bool> {
        Ok(call("bus.stop_requested", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The ticket the passenger at the cash desk asks for (nothing when nobody asks).
    ///
    /// Returns: name, price
    pub fn ticket_request() -> Result<Vec<Value>> {
        Ok(call("bus.ticket_request", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The tickets the map sells: `{name, price, day_ticket}`.
    ///
    /// Returns: list of tables
    pub fn tickets() -> Result<Vec<Value>> {
        Ok(call("bus.tickets", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Puts the parking brake on or off, as its key does.
    ///
    /// Returns: boolean
    pub fn toggle_handbrake() -> Result<bool> {
        Ok(call("bus.toggle_handbrake", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Parts coupled behind the bus (an articulated bus's rear counts).
    ///
    /// Returns: integer or nil
    pub fn trailers() -> Result<Option<f64>> {
        Ok(call("bus.trailers", Value::Array(vec![]))?.as_f64())
    }

    /// The names of the bus's script triggers (for `trigger`, `press`).
    ///
    /// Returns: list of strings
    pub fn triggers() -> Result<Vec<Value>> {
        Ok(call("bus.triggers", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The bus's speed in km/h, negative backwards.
    ///
    /// Returns: number or nil
    pub fn velocity() -> Result<Option<f64>> {
        Ok(call("bus.velocity", Value::Array(vec![]))?.as_f64())
    }

    /// Its velocity in the world, m/s (x east, y north, z up).
    ///
    /// Returns: x, y, z
    pub fn velocity_vector() -> Result<Vec<Value>> {
        Ok(call("bus.velocity_vector", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Each wheel: `{axle, side, rpm, radius, suspension, driven}` (`side` 0 left, 1 right;
    /// `suspension` the spring's travel).
    ///
    /// Returns: list of tables
    pub fn wheels() -> Result<Vec<Value>> {
        Ok(call("bus.wheels", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `camera` group of the API.
pub mod camera {
    use super::*;
    /// Its vertical field of view, degrees.
    ///
    /// Returns: number or nil
    pub fn fov() -> Result<Option<f64>> {
        Ok(call("camera.fov", Value::Array(vec![]))?.as_f64())
    }

    /// The camera: `{view, x, y, z, yaw, pitch, roll, fov, in_cab, zoom, look_yaw, look_pitch,
    /// width, height}` - map metres, degrees (yaw clockwise from north, pitch up positive), the
    /// vertical field of view, the picture's pixels.
    ///
    /// Returns: table or nil
    pub fn get() -> Result<Value> {
        call("camera.get", Value::Array(vec![]))
    }

    /// Whether the camera is in the player's own bus (driver or passenger view).
    ///
    /// Returns: boolean
    pub fn in_cab() -> Result<bool> {
        Ok(call("camera.in_cab", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Turns the head (driver, passenger view) or swings the outside camera round the bus: degrees
    /// from straight ahead.
    ///
    /// Returns: boolean
    pub fn look(yaw: f64, pitch: f64) -> Result<bool> {
        Ok(call("camera.look", json!([json!(yaw), json!(pitch)]))?.as_bool().unwrap_or(false))
    }

    /// Where it looks, degrees.
    ///
    /// Returns: yaw, pitch, roll
    pub fn orientation() -> Result<Vec<Value>> {
        Ok(call("camera.orientation", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Where the camera is (map metres).
    ///
    /// Returns: x, y, z
    pub fn position() -> Result<Vec<Value>> {
        Ok(call("camera.position", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Where a map point is seen on the screen, in the panels' pixels (as `ui.screen` measures
    /// them); nothing when it is behind the camera. For labels over buses, stops, people.
    ///
    /// Returns: screen_x, screen_y
    pub fn project(x: f64, y: f64, z: f64) -> Result<Vec<Value>> {
        Ok(call("camera.project", json!([json!(x), json!(y), json!(z)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Puts the free camera at a map point looking along `yaw` and `pitch` (degrees); the view
    /// becomes `"free"` (the player moves it on from there).
    ///
    /// Returns: boolean
    pub fn set_free(x: f64, y: f64, z: f64, yaw: Option<f64>, pitch: Option<f64>) -> Result<bool> {
        Ok(call("camera.set_free", json!([json!(x), json!(y), json!(z), match yaw { Some(v) => json!(v), None => Value::Null }, match pitch { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

    /// Switches the view: `"driver"`, `"pax"`, `"outside"`, `"map"` (the free camera above the
    /// bus), `"ego"` (walking), or a `view_*` action of keyboard.cfg.
    ///
    /// Returns: boolean
    pub fn set_view(view: &str) -> Result<bool> {
        Ok(call("camera.set_view", json!([json!(view)]))?.as_bool().unwrap_or(false))
    }

    /// The zoom of the view now (its field of view times this, 0.2 to 3).
    ///
    /// Returns: boolean
    pub fn set_zoom(zoom: f64) -> Result<bool> {
        Ok(call("camera.set_zoom", json!([json!(zoom)]))?.as_bool().unwrap_or(false))
    }

    /// The view: `"driver"`, `"pax"`, `"outside"`, `"free"` or `"foot"`.
    ///
    /// Returns: string or nil
    pub fn view() -> Result<Option<String>> {
        Ok(call("camera.view", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

}

/// The `duty` group of the API.
pub mod duty {
    use super::*;
    /// Whether the player drives a duty (a line and tour of the timetable).
    ///
    /// Returns: boolean
    pub fn active() -> Result<bool> {
        Ok(call("duty.active", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Whether the bus stands at the next stop (within 25 m of it).
    ///
    /// Returns: boolean
    pub fn at_stop() -> Result<bool> {
        Ok(call("duty.at_stop", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Seconds the bus is late (early negative), worked out between the stops as the game's
    /// timetable does.
    ///
    /// Returns: number or nil
    pub fn delay() -> Result<Option<f64>> {
        Ok(call("duty.delay", Value::Array(vec![]))?.as_f64())
    }

    /// Gives the duty up (the bus drives on without a timetable); `true` when there was one.
    ///
    /// Returns: boolean
    pub fn finish() -> Result<bool> {
        Ok(call("duty.finish", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The duty now: `line`, `tour`, `trip` (its number in the duty, from 1), `trips`, `trip_name`,
    /// `terminus`, `departure`, `arrival`, `stops`, `next_stop` and `previous_stop` (each a stop:
    /// `{number, name, id, arrival, departure, stops, x, y, z}`), `at_stop`, `trip_done`, `delay`
    /// (seconds, late positive).
    ///
    /// Returns: table or nil
    pub fn get() -> Result<Value> {
        call("duty.get", Value::Array(vec![]))
    }

    /// The next stop of the trip (as `duty.stops` gives them).
    ///
    /// Returns: table or nil
    pub fn next_stop() -> Result<Value> {
        call("duty.next_stop", Value::Array(vec![]))
    }

    /// Skips the next stop (the duty goes on to the one after); the name of the stop skipped.
    ///
    /// Returns: string or nil
    pub fn skip_stop() -> Result<Option<String>> {
        Ok(call("duty.skip_stop", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The stops of the trip now: `{number, name, id, arrival, departure, stops, passed, x, y, z}`
    /// (`stops` false: the bus passes it; `x, y, z` where the stop's place is known; `id` the map's
    /// object id).
    ///
    /// Returns: list of tables
    pub fn stops() -> Result<Vec<Value>> {
        Ok(call("duty.stops", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The trips of the duty: `{number, name, line, terminus, departure, arrival, stops}`.
    ///
    /// Returns: list of tables
    pub fn trips() -> Result<Vec<Value>> {
        Ok(call("duty.trips", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `files` group of the API.
pub mod files {
    use super::*;
    /// Adds to the end of a file of the data folder (a log, a CSV of trips).
    ///
    /// Returns: true, or false and the reason
    pub fn append(path: &str, data: &str) -> Result<Vec<Value>> {
        Ok(call("files.append", json!([json!(path), json!(data)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Removes a file or an empty folder of the data folder.
    ///
    /// Returns: boolean
    pub fn delete(path: &str) -> Result<bool> {
        Ok(call("files.delete", json!([json!(path)]))?.as_bool().unwrap_or(false))
    }

    /// Where the data folder is on this computer (to tell the player; the plugin reaches it with
    /// relative paths only).
    ///
    /// Returns: string
    pub fn dir() -> Result<String> {
        Ok(call("files.dir", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
    }

    /// Whether the data folder has this file or folder.
    ///
    /// Returns: boolean
    pub fn exists(path: &str) -> Result<bool> {
        Ok(call("files.exists", json!([json!(path)]))?.as_bool().unwrap_or(false))
    }

    /// The entries of the data folder (or a folder in it): `{name, dir, size}`.
    ///
    /// Returns: list of tables, or nil and the reason
    pub fn list(dir: Option<&str>) -> Result<Vec<Value>> {
        Ok(call("files.list", json!([match dir { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// Makes a folder (and those above it) in the data folder.
    ///
    /// Returns: boolean
    pub fn mkdir(path: &str) -> Result<bool> {
        Ok(call("files.mkdir", json!([json!(path)]))?.as_bool().unwrap_or(false))
    }

    /// A file of the plugin's data folder (`path` relative to it; `..` and absolute paths are
    /// refused).
    ///
    /// Returns: text, or nil and the reason
    pub fn read(path: &str) -> Result<Vec<Value>> {
        Ok(call("files.read", json!([json!(path)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Writes a file of the data folder (its folders are made); at most 16 MB at once and 256 MB in
    /// all.
    ///
    /// Returns: true, or false and the reason
    pub fn write(path: &str, data: &str) -> Result<Vec<Value>> {
        Ok(call("files.write", json!([json!(path), json!(data)]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `fmt` group of the API.
pub mod fmt {
    use super::*;
    /// Seconds since midnight as `"HH:MM"` (`"HH:MM:SS"` with `true`); past midnight wraps.
    ///
    /// Returns: string
    pub fn clock(seconds: f64, with_seconds: Option<bool>) -> Result<String> {
        Ok(call("fmt.clock", json!([json!(seconds), match with_seconds { Some(v) => json!(v), None => Value::Null }]))?.as_str().unwrap_or_default().to_string())
    }

    /// A delay as a timetable display shows it: `"+2:30"` late, `"-0:45"` early, `"0:00"`.
    ///
    /// Returns: string
    pub fn delay(seconds: f64) -> Result<String> {
        Ok(call("fmt.delay", json!([json!(seconds)]))?.as_str().unwrap_or_default().to_string())
    }

    /// A distance as `"350 m"` or `"2.4 km"`.
    ///
    /// Returns: string
    pub fn distance(metres: f64) -> Result<String> {
        Ok(call("fmt.distance", json!([json!(metres)]))?.as_str().unwrap_or_default().to_string())
    }

    /// A length of time as people read it: `"45 s"`, `"3 min 05 s"`, `"1 h 20 min"` (negative with
    /// a minus).
    ///
    /// Returns: string
    pub fn duration(seconds: f64) -> Result<String> {
        Ok(call("fmt.duration", json!([json!(seconds)]))?.as_str().unwrap_or_default().to_string())
    }

    /// An amount with two decimals and a currency symbol after it (`"12.50 €"`; the game knows no
    /// currency of its own).
    ///
    /// Returns: string
    pub fn money(amount: f64, symbol: Option<&str>) -> Result<String> {
        Ok(call("fmt.money", json!([json!(amount), match symbol { Some(v) => json!(v), None => Value::Null }]))?.as_str().unwrap_or_default().to_string())
    }

    /// A speed as `"42 km/h"`, or in `"mph"` or `"m/s"`.
    ///
    /// Returns: string
    pub fn speed(kmh: f64, unit: Option<&str>) -> Result<String> {
        Ok(call("fmt.speed", json!([json!(kmh), match unit { Some(v) => json!(v), None => Value::Null }]))?.as_str().unwrap_or_default().to_string())
    }

    /// A text cut at every `separator` (`","` by default; plainly, no patterns).
    ///
    /// Returns: list of strings
    pub fn split(text: &str, separator: Option<&str>) -> Result<Vec<Value>> {
        Ok(call("fmt.split", json!([json!(text), match separator { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// A text without the spaces at its ends.
    ///
    /// Returns: string
    pub fn trim(text: &str) -> Result<String> {
        Ok(call("fmt.trim", json!([json!(text)]))?.as_str().unwrap_or_default().to_string())
    }

}

/// The `game` group of the API.
pub mod game {
    use super::*;
    /// A game action of keyboard.cfg's `[game]` (`sim_pause`, `view_set_map`,
    /// `view_toggle_informationdisplay`, `view_set_schedule`, ...), as its key does; `true` when
    /// the game knows it.
    ///
    /// Returns: boolean
    pub fn action(name: &str) -> Result<bool> {
        Ok(call("game.action", json!([json!(name)]))?.as_bool().unwrap_or(false))
    }

    /// The version of the plugin interface (`api_abi` of an `.oop`; raised only when something is
    /// taken away or changes).
    ///
    /// Returns: integer
    pub fn api() -> Result<f64> {
        Ok(call("game.api", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
    }

    /// Frames a second now.
    ///
    /// Returns: number or nil
    pub fn fps() -> Result<Option<f64>> {
        Ok(call("game.fps", Value::Array(vec![]))?.as_f64())
    }

    /// Whether this game has an API function (`"weather.set"`): for a plugin that should also run
    /// in an older openOMSI.
    ///
    /// Returns: boolean
    pub fn has(name: &str) -> Result<bool> {
        Ok(call("game.has", json!([json!(name)]))?.as_bool().unwrap_or(false))
    }

    /// Whether the game menu is open.
    ///
    /// Returns: boolean
    pub fn menu_open() -> Result<bool> {
        Ok(call("game.menu_open", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// A notification of the game's own (the cards the server's messages use): `kind` `"info"`,
    /// `"warning"` or `"alert"`.
    ///
    /// Returns: boolean
    pub fn notify(text: &str, kind: Option<&str>, seconds: Option<f64>) -> Result<bool> {
        Ok(call("game.notify", json!([json!(text), match kind { Some(v) => json!(v), None => Value::Null }, match seconds { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

    /// The system the game runs on: `"windows"`, `"macos"`, `"linux"`, `"android"`.
    ///
    /// Returns: string
    pub fn platform() -> Result<String> {
        Ok(call("game.platform", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
    }

    /// Takes a screenshot with the next frame, as the camera key does; the file it goes to (the
    /// `screenshot` event says when it is there).
    ///
    /// Returns: string or nil
    pub fn screenshot() -> Result<Option<String>> {
        Ok(call("game.screenshot", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The settings a plugin may read: `graphics` (`"vanilla"`, `"vanilla_plus"`, `"enhanced"`),
    /// `language` (the cockpit's, `"ENG"`), `ui_language`, `ui_scale`, `volume`, `fov`,
    /// `render_scale`, `max_fps`, `fullscreen`, `vsync`, `msaa`, `shadows`, `time_speed`, `units`
    /// (`"metric"`: speeds are km/h everywhere), ...
    ///
    /// Returns: table
    pub fn settings() -> Result<Value> {
        call("game.settings", Value::Array(vec![]))
    }

    /// This session's counts, as the personnel file has them: `km`, `stops_served`,
    /// `stops_skipped`, `crashes`, `heavy_crashes`, `pedestrians`, `tickets`, `cash`, `passengers`,
    /// ...
    ///
    /// Returns: table
    pub fn stats() -> Result<Value> {
        call("game.stats", Value::Array(vec![]))
    }

    /// The game's version (`"0.2.22"`).
    ///
    /// Returns: string
    pub fn version() -> Result<String> {
        Ok(call("game.version", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
    }

}

/// The `input` group of the API.
pub mod input {
    use super::*;
    /// The key bindings: `{action, key}` of the game's keys, or the vehicles' with `true` (`key` as
    /// the game writes it: `"Ctrl+D"`).
    ///
    /// Returns: list of tables
    pub fn bindings(vehicles: Option<bool>) -> Result<Vec<Value>> {
        Ok(call("input.bindings", json!([match vehicles { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// The steering wheels, pedals, joysticks and gamepads: `{name, gamepad, axes, buttons}`
    /// (`axes` each axis's value; `buttons` how many it has - the `controller_button` event tells
    /// presses).
    ///
    /// Returns: list of tables
    pub fn controllers() -> Result<Vec<Value>> {
        Ok(call("input.controllers", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Whether a key is held now (winit's names, as the `key` event: `"KeyW"`, `"ShiftLeft"`,
    /// `"F5"`).
    ///
    /// Returns: boolean
    pub fn key_down(key: &str) -> Result<bool> {
        Ok(call("input.key_down", json!([json!(key)]))?.as_bool().unwrap_or(false))
    }

    /// Every key held now.
    ///
    /// Returns: list of strings
    pub fn keys_down() -> Result<Vec<Value>> {
        Ok(call("input.keys_down", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The mouse: where it is in the panels' pixels and its buttons held.
    ///
    /// Returns: x, y, left, right, middle
    pub fn mouse() -> Result<Vec<Value>> {
        Ok(call("input.mouse", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `json` group of the API.
pub mod json {
    use super::*;
    /// Reads JSON text: objects and arrays become tables, `null` nil.
    ///
    /// Returns: value, or nil and the reason
    pub fn decode(text: &str) -> Result<Vec<Value>> {
        Ok(call("json.decode", json!([json!(text)]))?.as_array().cloned().unwrap_or_default())
    }

    /// A value as JSON text (a list as an array, a table with keys as an object; `pretty`:
    /// indented).
    ///
    /// Returns: string
    pub fn encode(value: serde_json::Value, pretty: Option<bool>) -> Result<String> {
        Ok(call("json.encode", json!([json!(value), match pretty { Some(v) => json!(v), None => Value::Null }]))?.as_str().unwrap_or_default().to_string())
    }

}

/// The `lan` group of the API.
pub mod lan {
    use super::*;
    /// Whether this game is in a LAN session.
    ///
    /// Returns: boolean
    pub fn active() -> Result<bool> {
        Ok(call("lan.active", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Says a line in the session's chat, as the player would (every player sees it; at most one a
    /// second).
    ///
    /// Returns: true, or false and the reason
    pub fn chat(text: &str) -> Result<Vec<Value>> {
        Ok(call("lan.chat", json!([json!(text)]))?.as_array().cloned().unwrap_or_default())
    }

    /// This player in the session: its id (the host is 1), its name, whether it hosts.
    ///
    /// Returns: id, name, host
    pub fn me() -> Result<Vec<Value>> {
        Ok(call("lan.me", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The other players of the session: `{id, name, host, bus, line, tour, x, y, z, heading,
    /// speed, on_foot, passengers}`.
    ///
    /// Returns: list of tables
    pub fn players() -> Result<Vec<Value>> {
        Ok(call("lan.players", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `map` group of the API.
pub mod map {
    use super::*;
    /// The map's start points: `{index, name, x, y, z, heading}` (`index` from 1; the place where
    /// the game has read its tile).
    ///
    /// Returns: list of tables
    pub fn entrypoints() -> Result<Vec<Value>> {
        Ok(call("map.entrypoints", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The height of the ground (the road where there is one, else the terrain) at a map point,
    /// where its tile is loaded.
    ///
    /// Returns: number or nil
    pub fn ground(x: f64, y: f64) -> Result<Option<f64>> {
        Ok(call("map.ground", json!([json!(x), json!(y)]))?.as_f64())
    }

    /// `{name, friendly_name, path, left_hand_traffic, tile_size}`: the map's names, its global.cfg
    /// and the side it drives on.
    ///
    /// Returns: table or nil
    pub fn info() -> Result<Value> {
        call("map.info", Value::Array(vec![]))
    }

    /// The traffic lane nearest to a map point: `{index, distance, speed_limit, name,
    /// traffic_light}`.
    ///
    /// Returns: table or nil
    pub fn lane(x: f64, y: f64) -> Result<Value> {
        call("map.lane", json!([json!(x), json!(y)]))
    }

    /// The map's name (its folder's).
    ///
    /// Returns: string or nil
    pub fn name() -> Result<Option<String>> {
        Ok(call("map.name", Value::Array(vec![]))?.as_str().map(str::to_string))
    }

    /// The map objects within `radius` m (default 50, at most 2000 of them, nearest first): `{id,
    /// x, y, z, heading, distance}`.
    ///
    /// Returns: list of tables
    pub fn objects_near(x: f64, y: f64, radius: Option<f64>) -> Result<Vec<Value>> {
        Ok(call("map.objects_near", json!([json!(x), json!(y), match radius { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// Puts the player's bus on the street nearest to a map point (within 300 m), along it.
    ///
    /// Returns: boolean
    pub fn place_on_road(x: f64, y: f64) -> Result<bool> {
        Ok(call("map.place_on_road", json!([json!(x), json!(y)]))?.as_bool().unwrap_or(false))
    }

    /// The speed limit (km/h) of the lane the player's bus drives on (`nil` off the lanes).
    ///
    /// Returns: number or nil
    pub fn speed_limit() -> Result<Option<f64>> {
        Ok(call("map.speed_limit", Value::Array(vec![]))?.as_f64())
    }

    /// The bus stops of the tiles loaded (around the camera): `{id, name, x, y, z, heading}`.
    ///
    /// Returns: list of tables
    pub fn stops() -> Result<Vec<Value>> {
        Ok(call("map.stops", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Moves the player's bus to a map point (z none: onto the highest ground there) facing
    /// `heading` (default north), as the game menu's move does; the `service` event says
    /// `teleport`.
    ///
    /// Returns: boolean
    pub fn teleport(x: f64, y: f64, z: Option<f64>, heading: Option<f64>) -> Result<bool> {
        Ok(call("map.teleport", json!([json!(x), json!(y), match z { Some(v) => json!(v), None => Value::Null }, match heading { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

    /// The terrain's height alone at a map point.
    ///
    /// Returns: number or nil
    pub fn terrain(x: f64, y: f64) -> Result<Option<f64>> {
        Ok(call("map.terrain", json!([json!(x), json!(y)]))?.as_f64())
    }

    /// The tile the player's bus is on (nothing on foot).
    ///
    /// Returns: tile_x, tile_y
    pub fn tile() -> Result<Vec<Value>> {
        Ok(call("map.tile", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The tile of a map point and the metres in it (x east, y north).
    ///
    /// Returns: tile_x, tile_y, local_x, local_y
    pub fn tile_at(x: f64, y: f64) -> Result<Vec<Value>> {
        Ok(call("map.tile_at", json!([json!(x), json!(y)]))?.as_array().cloned().unwrap_or_default())
    }

    /// The map's tiles: `{x, y, file, loaded}` (numbered as global.cfg's `[map]` list).
    ///
    /// Returns: list of tables
    pub fn tiles() -> Result<Vec<Value>> {
        Ok(call("map.tiles", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `people` group of the API.
pub mod people {
    use super::*;
    /// The people of the map near the camera: walking, waiting at stops, riding a bus.
    ///
    /// Returns: walking, waiting, riding
    pub fn counts() -> Result<Vec<Value>> {
        Ok(call("people.counts", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The people setting: 0 to 3, 1 the map's own amount.
    ///
    /// Returns: number or nil
    pub fn density() -> Result<Option<f64>> {
        Ok(call("people.density", Value::Array(vec![]))?.as_f64())
    }

    /// The people (within `radius` m of the player's bus, when given): `{id, x, y, z, state,
    /// aboard, ai_bus, stop, destination, ticket, complaint}`. `state`: `"strolling"`, `"idle"`,
    /// `"standing"`, `"waiting"`, `"to_bus"`, `"boarding"`, `"riding"`, `"seated"`, `"leaving"`,
    /// `"to_stop"`; `aboard` in the player's bus; `complaint` 0 to 3 (3: they leave).
    ///
    /// Returns: list of tables
    pub fn list(radius: Option<f64>) -> Result<Vec<Value>> {
        Ok(call("people.list", json!([match radius { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// Changes the people setting (0 to 3).
    ///
    /// Returns: boolean
    pub fn set_density(value: f64) -> Result<bool> {
        Ok(call("people.set_density", json!([json!(value)]))?.as_bool().unwrap_or(false))
    }

    /// The stops near the camera where people wait: `{id, name, x, y, z, waiting}`.
    ///
    /// Returns: list of tables
    pub fn stops() -> Result<Vec<Value>> {
        Ok(call("people.stops", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `plugin` group of the API.
pub mod plugin {
    use super::*;
    /// Sends a message to every other plugin loaded.
    ///
    /// Returns: nil
    pub fn broadcast(topic: &str, data: Option<serde_json::Value>) -> Result<()> {
        call("plugin.broadcast", json!([json!(topic), match data { Some(v) => json!(v), None => Value::Null }]))?; Ok(())
    }

    /// Switches the plugin off until its file changes or the game starts again (its `stop` comes).
    ///
    /// Returns: nil
    pub fn disable(reason: Option<&str>) -> Result<()> {
        call("plugin.disable", json!([match reason { Some(v) => json!(v), None => Value::Null }]))?; Ok(())
    }

    /// How many errors the plugin had (at 10 it is switched off).
    ///
    /// Returns: integer
    pub fn errors() -> Result<f64> {
        Ok(call("plugin.errors", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
    }

    /// The files of the plugin's own folder (or a folder in it): `{name, dir, size}`.
    ///
    /// Returns: list of tables
    pub fn files(dir: Option<&str>) -> Result<Vec<Value>> {
        Ok(call("plugin.files", json!([match dir { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// The names of the plugins loaded, this one too.
    ///
    /// Returns: list of strings
    pub fn list() -> Result<Vec<Value>> {
        Ok(call("plugin.list", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The plugin's name: its file's, or its folder's for a `main.lua`.
    ///
    /// Returns: string
    pub fn name() -> Result<String> {
        Ok(call("plugin.name", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
    }

    /// The permissions the plugin has (a plain `.lua` file has them all).
    ///
    /// Returns: list of strings
    pub fn permissions() -> Result<Vec<Value>> {
        Ok(call("plugin.permissions", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// A file of the plugin's own folder (a table of stops, a translation): read only, relative to
    /// the folder.
    ///
    /// Returns: text, or nil and the reason
    pub fn read(path: &str) -> Result<Vec<Value>> {
        Ok(call("plugin.read", json!([json!(path)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Sends a message to another plugin (by its name): it hears `message(from, topic, data)` in
    /// its next frame. The data is numbers, texts, booleans or tables of them.
    ///
    /// Returns: nil
    pub fn send(to: &str, topic: &str, data: Option<serde_json::Value>) -> Result<()> {
        call("plugin.send", json!([json!(to), json!(topic), match data { Some(v) => json!(v), None => Value::Null }]))?; Ok(())
    }

    /// Changes a setting (held to its range; saved); `true` when it took the value.
    ///
    /// Returns: boolean
    pub fn set_setting(key: &str, value: serde_json::Value) -> Result<bool> {
        Ok(call("plugin.set_setting", json!([json!(key), json!(value)]))?.as_bool().unwrap_or(false))
    }

    /// A setting's value (`nil`: no such setting).
    ///
    /// Returns: any
    pub fn setting(key: &str) -> Result<Value> {
        call("plugin.setting", json!([json!(key)]))
    }

    /// Declares the plugin's settings: a list of `{key, type, label, default}` with `type`
    /// `"bool"`, `"number"` (`min`, `max`, `step`), `"text"` or `"choice"` (`choices`, a list of
    /// texts). The values the player chose before come back (as a table key -> value); the game
    /// makes a settings panel of them (`plugin.show_settings`), saves them in the data folder and
    /// sends `setting(key, value)` when one changes.
    ///
    /// Returns: table
    pub fn settings(settings: serde_json::Value, title: Option<&str>) -> Result<Value> {
        call("plugin.settings", json!([json!(settings), match title { Some(v) => json!(v), None => Value::Null }]))
    }

    /// Shows the plugin's settings panel (or hides it with `false`); it can be dragged and closed
    /// while the panels have the mouse (`ui.focus`). `false` when the plugin declared no settings.
    ///
    /// Returns: boolean
    pub fn show_settings(on: Option<bool>) -> Result<bool> {
        Ok(call("plugin.show_settings", json!([match on { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

}

/// The `storage` group of the API.
pub mod storage {
    use super::*;
    /// Everything stored, as one table.
    ///
    /// Returns: table
    pub fn all() -> Result<Value> {
        call("storage.all", Value::Array(vec![]))
    }

    /// Removes every key.
    ///
    /// Returns: nil
    pub fn clear() -> Result<()> {
        call("storage.clear", Value::Array(vec![]))?; Ok(())
    }

    /// Removes a key; `true` when it was there.
    ///
    /// Returns: boolean
    pub fn delete(key: &str) -> Result<bool> {
        Ok(call("storage.delete", json!([json!(key)]))?.as_bool().unwrap_or(false))
    }

    /// A value the plugin stored, or `nil`. The storage is the plugin's own and survives the
    /// session (kept as `storage.json` in its data folder).
    ///
    /// Returns: any
    pub fn get(key: &str) -> Result<Value> {
        call("storage.get", json!([json!(key)]))
    }

    /// The keys stored, in the order they were first set.
    ///
    /// Returns: list of strings
    pub fn keys() -> Result<Vec<Value>> {
        Ok(call("storage.keys", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Writes the storage now (it is written by itself when the game ends).
    ///
    /// Returns: nil
    pub fn save() -> Result<()> {
        call("storage.save", Value::Array(vec![]))?; Ok(())
    }

    /// Stores a value under a key: a number, text, boolean or a table of them (`nil` removes it).
    /// It is written when the game ends, the plugin is loaded again, or `storage.save()` is called.
    ///
    /// Returns: nil
    pub fn set(key: &str, value: serde_json::Value) -> Result<()> {
        call("storage.set", json!([json!(key), json!(value)]))?; Ok(())
    }

}

/// The `timetable` group of the API.
pub mod timetable {
    use super::*;
    /// The timetable buses of the AI on the road: `{id, line, tour, trip, terminus, departure,
    /// next_stop_id, at_stop, trip_done, delay, x, y, number}`.
    ///
    /// Returns: list of tables
    pub fn buses() -> Result<Vec<Value>> {
        Ok(call("timetable.buses", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The map's lines: `{name, user_allowed, tours}`, each tour `{number, today, trips}` (`today`:
    /// it runs on the game's date).
    ///
    /// Returns: list of tables
    pub fn lines() -> Result<Vec<Value>> {
        Ok(call("timetable.lines", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The stops the timetable knows: `{id, name}` (`id` the map's object id).
    ///
    /// Returns: list of tables
    pub fn stop_names() -> Result<Vec<Value>> {
        Ok(call("timetable.stop_names", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `traffic` group of the API.
pub mod traffic {
    use super::*;
    /// The AI vehicle ahead of the player's bus within `reach` m (default 100) and 20° of its
    /// heading, with its `distance`: for a distance warning or a cruise control.
    ///
    /// Returns: table or nil
    pub fn ahead(reach: Option<f64>) -> Result<Value> {
        call("traffic.ahead", json!([match reach { Some(v) => json!(v), None => Value::Null }]))
    }

    /// Takes every car not running to a timetable off the road; how many went.
    ///
    /// Returns: integer or nil
    pub fn clear() -> Result<Option<f64>> {
        Ok(call("traffic.clear", Value::Array(vec![]))?.as_f64())
    }

    /// The AI vehicles: driving, timetable buses among them, asleep out of range, parked.
    ///
    /// Returns: driving, buses, asleep, parked
    pub fn counts() -> Result<Vec<Value>> {
        Ok(call("traffic.counts", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// How many cars the traffic keeps around the camera, and the share of them that do not run to
    /// a timetable (0 to 1).
    ///
    /// Returns: cars, share
    pub fn density() -> Result<Vec<Value>> {
        Ok(call("traffic.density", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// The traffic light the player's bus comes to within `reach` m (default 80): `{aspect,
    /// change_in, distance}` - `aspect` `"red"`, `"red_yellow"`, `"green"`, `"green_yellow"`,
    /// `"yellow"` or `"dark"`, `change_in` the seconds to its next change.
    ///
    /// Returns: table or nil
    pub fn light_ahead(reach: Option<f64>) -> Result<Value> {
        call("traffic.light_ahead", json!([match reach { Some(v) => json!(v), None => Value::Null }]))
    }

    /// The AI vehicles (within `radius` m of the player's bus, when given): `{id, kind, name, x, y,
    /// z, heading, speed, max_speed, waiting_for, standing, braking, blinker, line, distance}`.
    /// `kind`: `"car"`, `"taxi"`, `"bus"`, `"truck"`, `"timetable_bus"`, `"tram"`, `"bicycle"`;
    /// `waiting_for` why it waits or slows (`"lead"`, `"light"`, `"yield"`, `"people"`, ...);
    /// `standing` the seconds it has stood.
    ///
    /// Returns: list of tables
    pub fn list(radius: Option<f64>) -> Result<Vec<Value>> {
        Ok(call("traffic.list", json!([match radius { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// The AI vehicle nearest to the player's bus (of that `kind`, when given), with its
    /// `distance`.
    ///
    /// Returns: table or nil
    pub fn nearest(kind: Option<&str>) -> Result<Value> {
        call("traffic.nearest", json!([match kind { Some(v) => json!(v), None => Value::Null }]))
    }

}

/// The `ui` group of the API.
pub mod ui {
    use super::*;
    /// Removes every panel of the plugin.
    ///
    /// Returns: nil
    pub fn clear() -> Result<()> {
        call("ui.clear", Value::Array(vec![]))?; Ok(())
    }

    /// `true`: the panels get the mouse (the cursor shows, a click goes to the panel under it and
    /// none to the bus); `false`, Esc or a menu of the game gives it back. Returns the new state.
    ///
    /// Returns: boolean
    pub fn focus(on: bool) -> Result<bool> {
        Ok(call("ui.focus", json!([json!(on)]))?.as_bool().unwrap_or(false))
    }

    /// Whether the panels have the mouse.
    ///
    /// Returns: boolean
    pub fn focused() -> Result<bool> {
        Ok(call("ui.focused", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// How far the player dragged a panel from where its table puts it (pixels).
    ///
    /// Returns: dx, dy
    pub fn moved(panel: &str) -> Result<Vec<Value>> {
        Ok(call("ui.moved", json!([json!(panel)]))?.as_array().cloned().unwrap_or_default())
    }

    /// The ids of the plugin's panels.
    ///
    /// Returns: list of strings
    pub fn panels() -> Result<Vec<Value>> {
        Ok(call("ui.panels", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Removes a panel; `true` when there was one.
    ///
    /// Returns: boolean
    pub fn remove(id: &str) -> Result<bool> {
        Ok(call("ui.remove", json!([json!(id)]))?.as_bool().unwrap_or(false))
    }

    /// The screen in the panels' pixels, and how many of the screen's own pixels one of them is.
    ///
    /// Returns: width, height, scale
    pub fn screen() -> Result<Vec<Value>> {
        Ok(call("ui.screen", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Creates the panel `id` or replaces it; a table that is not right gives `false` and where
    /// (`"children[2].size: a number is expected"`). The same table again changes nothing.
    ///
    /// Returns: true, or false and the reason
    pub fn set(id: &str, panel: serde_json::Value) -> Result<Vec<Value>> {
        Ok(call("ui.set", json!([json!(id), json!(panel)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Shows a panel (or hides it with `false`; it is kept); `false` when there is none.
    ///
    /// Returns: boolean
    pub fn show(panel: &str, on: Option<bool>) -> Result<bool> {
        Ok(call("ui.show", json!([json!(panel), match on { Some(v) => json!(v), None => Value::Null }]))?.as_bool().unwrap_or(false))
    }

    /// A notification card at the top right, newest at the top; it goes after `opts.seconds` (1 to
    /// 60, default 5). `opts`: `title`, `icon`, `color`. A plugin shows 8 at most: a ninth makes
    /// its oldest go.
    ///
    /// Returns: true, or false and the reason
    pub fn toast(text: &str, opts: Option<serde_json::Value>) -> Result<Vec<Value>> {
        Ok(call("ui.toast", json!([json!(text), match opts { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// Shows a hidden panel or hides a shown one; whether it shows now.
    ///
    /// Returns: boolean
    pub fn toggle(panel: &str) -> Result<bool> {
        Ok(call("ui.toggle", json!([json!(panel)]))?.as_bool().unwrap_or(false))
    }

    /// Whether the player types into a text field of a plugin now (the keys then go to the field,
    /// not to the bus).
    ///
    /// Returns: boolean
    pub fn typing() -> Result<bool> {
        Ok(call("ui.typing", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Changes one element of a panel in place, by its id: `text`, `color`, `value` (a bar, a
    /// slider), `checked`, `selected`, `name` (an icon), `values` (a chart), `rows` (a table),
    /// `src` (an image) - cheaper than setting the whole panel again.
    ///
    /// Returns: true, or false and the reason
    pub fn update(panel: &str, element: &str, values: serde_json::Value) -> Result<Vec<Value>> {
        Ok(call("ui.update", json!([json!(panel), json!(element), json!(values)]))?.as_array().cloned().unwrap_or_default())
    }

    /// The value of a checkbox, slider, text field or tabs element now (as `ui_change` gives it).
    ///
    /// Returns: any
    pub fn value(panel: &str, element: &str) -> Result<Value> {
        call("ui.value", json!([json!(panel), json!(element)]))
    }

}

/// The `util` group of the API.
pub mod util {
    use super::*;
    /// A real time (`util.now()` by default) as `{year, month, day, hour, minute, second, weekday}`
    /// in UTC (`weekday` 1 Monday).
    ///
    /// Returns: table
    pub fn date(seconds: Option<f64>) -> Result<Value> {
        call("util.date", json!([match seconds { Some(v) => json!(v), None => Value::Null }]))
    }

    /// Milliseconds of real time since the game started: for timing a plugin's own work.
    ///
    /// Returns: number
    pub fn ms() -> Result<f64> {
        Ok(call("util.ms", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
    }

    /// The real time: seconds since 1970 (UTC), with fractions.
    ///
    /// Returns: number
    pub fn now() -> Result<f64> {
        Ok(call("util.now", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
    }

    /// A random number: 0 to 1 without arguments, else a whole number from `min` to `max` (as
    /// `math.random`, but not repeating the same row in every plugin).
    ///
    /// Returns: number
    pub fn random(min: Option<f64>, max: Option<f64>) -> Result<f64> {
        Ok(call("util.random", json!([match min { Some(v) => json!(v), None => Value::Null }, match max { Some(v) => json!(v), None => Value::Null }]))?.as_f64().unwrap_or(0.0))
    }

}

/// The `vec` group of the API.
pub mod vec {
    use super::*;
    /// The turn from heading `a` to heading `b`, -180 to 180 degrees (right positive).
    ///
    /// Returns: number
    pub fn angle_diff(a: f64, b: f64) -> Result<f64> {
        Ok(call("vec.angle_diff", json!([json!(a), json!(b)]))?.as_f64().unwrap_or(0.0))
    }

    /// The heading from one map point to another.
    ///
    /// Returns: number
    pub fn bearing(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<f64> {
        Ok(call("vec.bearing", json!([json!(x1), json!(y1), json!(x2), json!(y2)]))?.as_f64().unwrap_or(0.0))
    }

    /// `x` held between `min` and `max`.
    ///
    /// Returns: number
    pub fn clamp(x: f64, min: f64, max: f64) -> Result<f64> {
        Ok(call("vec.clamp", json!([json!(x), json!(min), json!(max)]))?.as_f64().unwrap_or(0.0))
    }

    /// Metres between two map points (on the ground: x and y).
    ///
    /// Returns: number
    pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<f64> {
        Ok(call("vec.distance", json!([json!(x1), json!(y1), json!(x2), json!(y2)]))?.as_f64().unwrap_or(0.0))
    }

    /// Metres between two points in space.
    ///
    /// Returns: number
    pub fn distance3(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Result<f64> {
        Ok(call("vec.distance3", json!([json!(x1), json!(y1), json!(z1), json!(x2), json!(y2), json!(z2)]))?.as_f64().unwrap_or(0.0))
    }

    /// The dot product of two 2D vectors.
    ///
    /// Returns: number
    pub fn dot(x1: f64, y1: f64, x2: f64, y2: f64) -> Result<f64> {
        Ok(call("vec.dot", json!([json!(x1), json!(y1), json!(x2), json!(y2)]))?.as_f64().unwrap_or(0.0))
    }

    /// The heading of a direction on the map, degrees clockwise from north (0 to 360), as the
    /// game's headings.
    ///
    /// Returns: number
    pub fn heading(dx: f64, dy: f64) -> Result<f64> {
        Ok(call("vec.heading", json!([json!(dx), json!(dy)]))?.as_f64().unwrap_or(0.0))
    }

    /// The length of a vector (2D, or 3D with `z`).
    ///
    /// Returns: number
    pub fn length(x: f64, y: f64, z: Option<f64>) -> Result<f64> {
        Ok(call("vec.length", json!([json!(x), json!(y), match z { Some(v) => json!(v), None => Value::Null }]))?.as_f64().unwrap_or(0.0))
    }

    /// From `a` to `b` by `t` (0 to 1, not held to it).
    ///
    /// Returns: number
    pub fn lerp(a: f64, b: f64, t: f64) -> Result<f64> {
        Ok(call("vec.lerp", json!([json!(a), json!(b), json!(t)]))?.as_f64().unwrap_or(0.0))
    }

    /// The vector made 1 long (0 stays 0).
    ///
    /// Returns: x, y, z
    pub fn normalize(x: f64, y: f64, z: Option<f64>) -> Result<Vec<Value>> {
        Ok(call("vec.normalize", json!([json!(x), json!(y), match z { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// A vector turned clockwise by `degrees` (as headings turn).
    ///
    /// Returns: x, y
    pub fn rotate(x: f64, y: f64, degrees: f64) -> Result<Vec<Value>> {
        Ok(call("vec.rotate", json!([json!(x), json!(y), json!(degrees)]))?.as_array().cloned().unwrap_or_default())
    }

    /// A map point seen from a place facing `heading`: metres to the right and ahead of it.
    ///
    /// Returns: right, ahead
    pub fn to_local(x: f64, y: f64, ox: f64, oy: f64, heading: f64) -> Result<Vec<Value>> {
        Ok(call("vec.to_local", json!([json!(x), json!(y), json!(ox), json!(oy), json!(heading)]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `weather` group of the API.
pub mod weather {
    use super::*;
    /// The weather now: `{name, visibility (m), wind_direction (°), wind_speed (m/s), temperature
    /// (°C), humidity (%), absolute_humidity (g/m³), pressure (hPa), clouds, cloud_base (m),
    /// precipitation ("none", "rain", "snow"), precipitation_rate (0..1), snow_cover, snow_on_road,
    /// wetness (the roads, 0..1), changing, locked}` (`locked`: it follows a real weather station
    /// and cannot be set).
    ///
    /// Returns: table or nil
    pub fn get() -> Result<Value> {
        call("weather.get", Value::Array(vec![]))
    }

    /// `"none"`, `"rain"` or `"snow"`, and how hard (0 to 1).
    ///
    /// Returns: kind, rate
    pub fn precipitation() -> Result<Vec<Value>> {
        Ok(call("weather.precipitation", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Changes to a weather file (as `weather.presets` names it; `""`: the map's own, changing with
    /// the day) over `seconds` (default 1).
    ///
    /// Returns: true, or false and the reason
    pub fn preset(file: &str, seconds: Option<f64>) -> Result<Vec<Value>> {
        Ok(call("weather.preset", json!([json!(file), match seconds { Some(v) => json!(v), None => Value::Null }]))?.as_array().cloned().unwrap_or_default())
    }

    /// The weather files installed: `{file, name}`.
    ///
    /// Returns: list of tables
    pub fn presets() -> Result<Vec<Value>> {
        Ok(call("weather.presets", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Changes the weather as the game menu's sliders do; any of `visibility`, `wind_direction`,
    /// `wind_speed`, `temperature`, `pressure`, `clouds` (`"-1"` none, `"Cumulus 1"`..`"3"`,
    /// `"Overcast 1"`), `cloud_base`, `precipitation` (`"none"`, `"rain"`, `"snow"`),
    /// `precipitation_rate` (0..1), `snow_cover`, `snow_on_road`, `wetness`; the rest stays. Held
    /// to the game's ranges; refused in a LAN game as a client or while the weather follows a
    /// station.
    ///
    /// Returns: true, or false and the reason
    pub fn set(values: serde_json::Value) -> Result<Vec<Value>> {
        Ok(call("weather.set", json!([json!(values)]))?.as_array().cloned().unwrap_or_default())
    }

    /// The air temperature, °C.
    ///
    /// Returns: number or nil
    pub fn temperature() -> Result<Option<f64>> {
        Ok(call("weather.temperature", Value::Array(vec![]))?.as_f64())
    }

    /// How far one sees, metres.
    ///
    /// Returns: number or nil
    pub fn visibility() -> Result<Option<f64>> {
        Ok(call("weather.visibility", Value::Array(vec![]))?.as_f64())
    }

    /// How wet the roads are, 0 (dry) to 1.
    ///
    /// Returns: number or nil
    pub fn wetness() -> Result<Option<f64>> {
        Ok(call("weather.wetness", Value::Array(vec![]))?.as_f64())
    }

    /// The wind: where it comes from (degrees) and its speed (m/s).
    ///
    /// Returns: direction, speed
    pub fn wind() -> Result<Vec<Value>> {
        Ok(call("weather.wind", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

}

/// The `world` group of the API.
pub mod world {
    use super::*;
    /// The game's date: `{year, month, day, weekday, day_of_year}` (`weekday` 1 Monday to 7
    /// Sunday).
    ///
    /// Returns: table or nil
    pub fn date() -> Result<Value> {
        call("world.date", Value::Array(vec![]))
    }

    /// Pauses the game, as P does (not in a LAN game). The plugins stand still with it: the player
    /// resumes it.
    ///
    /// Returns: boolean
    pub fn pause() -> Result<bool> {
        Ok(call("world.pause", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Whether the game stands still (a plugin hears `pause` and runs no more until `resume`).
    ///
    /// Returns: boolean
    pub fn paused() -> Result<bool> {
        Ok(call("world.paused", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// Seconds played this session (game time, never wraps).
    ///
    /// Returns: number or nil
    pub fn play_time() -> Result<Option<f64>> {
        Ok(call("world.play_time", Value::Array(vec![]))?.as_f64())
    }

    /// The season's texture folder (`nil`: the base textures) and whether snow lies.
    ///
    /// Returns: folder, snow
    pub fn season() -> Result<Vec<Value>> {
        Ok(call("world.season", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }

    /// Sets the time of day: seconds since midnight or `"HH:MM"` / `"HH:MM:SS"`, as the game menu's
    /// clock does (the timetable starts again after a jump of minutes). Not in a LAN game as a
    /// client, nor while the clock follows the computer's.
    ///
    /// Returns: boolean
    pub fn set_time(time: serde_json::Value) -> Result<bool> {
        Ok(call("world.set_time", json!([json!(time)]))?.as_bool().unwrap_or(false))
    }

    /// Sets how much faster the clock runs (1 to 30; not in a LAN game).
    ///
    /// Returns: boolean
    pub fn set_time_speed(factor: f64) -> Result<bool> {
        Ok(call("world.set_time_speed", json!([json!(factor)]))?.as_bool().unwrap_or(false))
    }

    /// The sun's height over the horizon, degrees.
    ///
    /// Returns: number or nil
    pub fn sun_altitude() -> Result<Option<f64>> {
        Ok(call("world.sun_altitude", Value::Array(vec![]))?.as_f64())
    }

    /// The game's time of day, seconds since midnight.
    ///
    /// Returns: number or nil
    pub fn time() -> Result<Option<f64>> {
        Ok(call("world.time", Value::Array(vec![]))?.as_f64())
    }

    /// How much faster than real time the game's clock runs (1 to 30).
    ///
    /// Returns: number or nil
    pub fn time_speed() -> Result<Option<f64>> {
        Ok(call("world.time_speed", Value::Array(vec![]))?.as_f64())
    }

}

