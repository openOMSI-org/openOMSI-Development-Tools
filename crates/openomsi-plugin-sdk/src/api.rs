//! Typed wrappers for the openOMSI plugin API, generated from `api.json`
//! (API version 0.2.21, ABI 1) by `cargo xtask gen-sdk`. Do not edit by hand:
//! run the command again against a newer manifest instead.
//!
//! Every function calls into the game through [`crate::call`]; a function the manifest
//! does not have yet is still reachable with that escape hatch.

#![allow(clippy::all)]
use crate::{Result, call};
use serde_json::{Value, json};

/// Whether the player drives a vehicle right now.
///
/// Returns: true while the player drives a vehicle
pub fn has_vehicle() -> Result<bool> {
    Ok(call("has_vehicle", Value::Array(vec![]))?.as_bool().unwrap_or(false))
}

/// The player's vehicle's name: manufacturer and type, as its [friendlyname] has them.
///
/// Returns: the name, or nil without a vehicle
pub fn vehicle() -> Result<Option<String>> {
    Ok(call("vehicle", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// The manufacturer part of the vehicle's name ("Solaris").
///
/// Returns: the manufacturer, or nil
pub fn vehicle_manufacturer() -> Result<Option<String>> {
    Ok(call("vehicle_manufacturer", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// The model part of the vehicle's name ("Urbino 10 / 2D").
///
/// Returns: the model, or nil
pub fn vehicle_model() -> Result<Option<String>> {
    Ok(call("vehicle_model", Value::Array(vec![]))?.as_str().map(str::to_string))
}

/// A script variable of the player's bus, by the name its .osc scripts and .opl lists use
/// (Velocity, elec_busbar_main, bus_doorfront0, ...). Without a bus, or for a name the bus does
/// not have, the result is nil.
///
/// Returns: the value, or nil
pub fn var(name: &str) -> Result<Option<f64>> {
    Ok(call("var", json!([json!(name)]))?.as_f64())
}

/// Sets a script variable of the player's bus.
///
/// Returns: true when the bus has that variable
pub fn set_var(name: &str, value: f64) -> Result<bool> {
    Ok(call("set_var", json!([json!(name), json!(value)]))?.as_bool().unwrap_or(false))
}

/// A string variable of the player's bus's scripts (IBIS_terminus_name, ...).
///
/// Returns: the text, or nil
pub fn str(name: &str) -> Result<Option<String>> {
    Ok(call("str", json!([json!(name)]))?.as_str().map(str::to_string))
}

/// Sets a string variable of the player's bus's scripts.
///
/// Returns: true when the bus has that variable
pub fn set_str(name: &str, text: &str) -> Result<bool> {
    Ok(call("set_str", json!([json!(name), json!(text)]))?.as_bool().unwrap_or(false))
}

/// A system variable of the scripts: Time, Day, Weather_Temperature, ... (read only).
///
/// Returns: the value, or nil
pub fn sys(name: &str) -> Result<Option<f64>> {
    Ok(call("sys", json!([json!(name)]))?.as_f64())
}

/// A key press on the bus: fires the trigger, then <name>_off.
///
/// Returns: nothing
pub fn trigger(name: &str) -> Result<()> {
    call("trigger", json!([json!(name)]))?;
    Ok(())
}

/// Holds a key of the bus down: fires the trigger and keeps it until release.
///
/// Returns: nothing
pub fn press(name: &str) -> Result<()> {
    call("press", json!([json!(name)]))?;
    Ok(())
}

/// Lets a key held with press go: fires <name>_off.
///
/// Returns: nothing
pub fn release(name: &str) -> Result<()> {
    call("release", json!([json!(name)]))?;
    Ok(())
}

/// Where the bus is: map metres and its heading in degrees.
///
/// Returns: x, y, z, heading; nothing on foot (several values: a list over the WASM boundary)
pub fn position() -> Result<Vec<Value>> {
    Ok(call("position", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
}

/// The other vehicles round the bus: AI traffic (kind "ai") and other players' buses in a LAN
/// game (kind "player"). Empty on foot.
///
/// Returns: a list of {id, kind, name, x, y, z, heading} within radius metres (default 300)
pub fn others(radius: Option<f64>) -> Result<Vec<Value>> {
    Ok(call(
        "others",
        json!([match radius {
            Some(v) => json!(v),
            None => Value::Null,
        }]),
    )?
    .as_array()
    .cloned()
    .unwrap_or_default())
}

/// A script variable of one of the vehicles from others().
///
/// Returns: the value, or nil
pub fn other_var(id: f64, name: &str) -> Result<Option<f64>> {
    Ok(call("other_var", json!([json!(id), json!(name)]))?.as_f64())
}

/// Sets a script variable of one of the vehicles from others(). An AI vehicle keeps it until
/// its scripts write it again; another player's bus takes its values from the network again.
///
/// Returns: true when that vehicle has the variable
pub fn set_other_var(id: f64, name: &str, value: f64) -> Result<bool> {
    Ok(call("set_other_var", json!([json!(id), json!(name), json!(value)]))?.as_bool().unwrap_or(false))
}

/// What the game is doing: map, clock (seconds since midnight), day, year, view, paused,
/// on_foot, multiplayer, traffic, speed (km/h), delay (s, late positive), version; with a bus
/// also its tile and place in it, heading, vehicle_manufacturer, vehicle_model, destination,
/// passengers; crashes, heavy_crashes and pedestrians_hit this session; on a duty line, tour,
/// trip, trips, trip_name, terminus, stops, trip_done, next_stop, next_stop_number,
/// next_stop_arrival, next_stop_departure, next_stop_id, at_stop, previous_stop,
/// previous_stop_id, next_stop_distance and previous_stop_distance.
///
/// Returns: a table of the values above
pub fn info() -> Result<Value> {
    call("info", Value::Array(vec![]))
}

/// The game's time of day.
///
/// Returns: "HH:MM:SS"
pub fn clock() -> Result<String> {
    Ok(call("clock", Value::Array(vec![]))?.as_str().unwrap_or_default().to_string())
}

/// The bus's speed in km/h (0 on foot).
///
/// Returns: km/h
pub fn speed() -> Result<f64> {
    Ok(call("speed", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
}

/// Metres from the bus to a map point.
///
/// Returns: metres, or nil on foot
pub fn distance(x: f64, y: f64) -> Result<Option<f64>> {
    Ok(call("distance", json!([json!(x), json!(y)]))?.as_f64())
}

/// The names of every variable of the bus's scripts, or with "str" of every string variable.
///
/// Returns: a list of names
pub fn vars(kind: Option<&str>) -> Result<Vec<Value>> {
    Ok(call(
        "vars",
        json!([match kind {
            Some(v) => json!(v),
            None => Value::Null,
        }]),
    )?
    .as_array()
    .cloned()
    .unwrap_or_default())
}

/// Does what a line of the game menu does: refuel, wash, repair, shot, save, load, weather,
/// later, earlier, info, timetable, reset, couple, uncouple.
///
/// Returns: true when the game knows the command
pub fn command(name: &str) -> Result<bool> {
    Ok(call("command", json!([json!(name)]))?.as_bool().unwrap_or(false))
}

/// Seconds of game time since the plugin started; stands still while the game is paused.
///
/// Returns: seconds
pub fn time() -> Result<f64> {
    Ok(call("time", Value::Array(vec![]))?.as_f64().unwrap_or(0.0))
}

/// Stops a timer or a watch.
///
/// Returns: nothing
pub fn cancel(id: f64) -> Result<()> {
    call("cancel", json!([json!(id)]))?;
    Ok(())
}

/// A line of text on the screen.
///
/// Returns: nothing (shown 5 seconds when not given)
pub fn message(text: &str, seconds: Option<f64>) -> Result<()> {
    call(
        "message",
        json!([
            json!(text),
            match seconds {
                Some(v) => json!(v),
                None => Value::Null,
            }
        ]),
    )?;
    Ok(())
}

/// Writes the plugin's saved data (omsi.data in Lua) at once instead of when the game ends.
///
/// Returns: nothing
pub fn save() -> Result<()> {
    call("save", Value::Array(vec![]))?;
    Ok(())
}

/// Sends text as one UDP datagram to 127.0.0.1:port: to another program on this computer, never
/// over the network. It does not wait and nothing comes back. Refused: ports below 1024 and the
/// game's multiplayer ports (27015-27024), more than 8 KB, more than 100 messages a second.
///
/// Returns: true, or false and the reason
pub fn send(port: f64, text: &str) -> Result<Vec<Value>> {
    Ok(call("send", json!([json!(port), json!(text)]))?.as_array().cloned().unwrap_or_default())
}

/// The `ui` group of the API.
pub mod ui {
    use super::*;
    /// Creates the panel id or replaces it. The panel is a table: anchor, x, y, width, padding,
    /// gap, background, radius, accent, visible, clickable and children (text, icon, row, bar,
    /// badge, divider, space, button elements). Set it again when its content changes; the same
    /// table again changes nothing.
    ///
    /// Returns: true, or false and the reason ("children[2].size: a number is expected")
    pub fn set(id: &str, panel: serde_json::Value) -> Result<Vec<Value>> {
        Ok(call("ui.set", json!([json!(id), json!(panel)]))?.as_array().cloned().unwrap_or_default())
    }

    /// Removes one of the plugin's panels.
    ///
    /// Returns: true when there was one
    pub fn remove(id: &str) -> Result<bool> {
        Ok(call("ui.remove", json!([json!(id)]))?.as_bool().unwrap_or(false))
    }

    /// Removes every panel of the plugin.
    ///
    /// Returns: nothing
    pub fn clear() -> Result<()> {
        call("ui.clear", Value::Array(vec![]))?;
        Ok(())
    }

    /// A notification card at the top right that slides in and goes after opts.seconds (1 to 60,
    /// default 5). opts: title, icon, color, seconds. A plugin shows 8 at most: a ninth makes its
    /// oldest go.
    ///
    /// Returns: true, or false and the reason
    pub fn toast(text: &str, opts: Option<serde_json::Value>) -> Result<Vec<Value>> {
        Ok(call(
            "ui.toast",
            json!([
                json!(text),
                match opts {
                    Some(v) => json!(v),
                    None => Value::Null,
                }
            ]),
        )?
        .as_array()
        .cloned()
        .unwrap_or_default())
    }

    /// true gives the plugin's panels the mouse (clicks become ui_click events), false gives it
    /// back to the bus. Esc or a game menu gives it back as well.
    ///
    /// Returns: the new state
    pub fn focus(on: bool) -> Result<bool> {
        Ok(call("ui.focus", json!([json!(on)]))?.as_bool().unwrap_or(false))
    }

    /// Whether the panels have the mouse.
    ///
    /// Returns: true or false
    pub fn focused() -> Result<bool> {
        Ok(call("ui.focused", Value::Array(vec![]))?.as_bool().unwrap_or(false))
    }

    /// The screen in the panels' pixels and how many of the screen's own pixels one of them is.
    ///
    /// Returns: width, height, scale
    pub fn screen() -> Result<Vec<Value>> {
        Ok(call("ui.screen", Value::Array(vec![]))?.as_array().cloned().unwrap_or_default())
    }
}
