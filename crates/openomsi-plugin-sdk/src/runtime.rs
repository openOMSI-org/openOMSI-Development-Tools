//! The plugin runtime the generated API and the macro build on: the callback registry, the
//! event, timer and watch helpers, and JSON encoding of call arguments and results.

use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::host;
use crate::{Error, Result};

type Callback = Box<dyn FnMut(Vec<Value>)>;

#[derive(Default)]
struct State {
    next_id: u64,
    callbacks: HashMap<u64, Callback>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// Registers a closure and returns the id the host uses to call it back.
fn register(cb: Callback) -> u64 {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.next_id += 1;
        let id = s.next_id;
        s.callbacks.insert(id, cb);
        id
    })
}

/// Encodes a callback id the way the ABI carries it over JSON.
fn cb_ref(id: u64) -> Value {
    json!({ "$cb": id })
}

/// Calls a registry function with a JSON array of arguments.
pub(crate) fn raw_call(name: &str, args: Value) -> Result<Value> {
    let bytes = serde_json::to_vec(&args).map_err(|e| Error(format!("encoding arguments: {e}")))?;
    let out = host::host_call(name, &bytes).map_err(Error)?;
    if out.is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_slice(&out).map_err(|e| Error(format!("decoding result: {e}")))
}

/// Invoked by the `oop_callback` export: decode the JSON argument array and run the callback.
///
/// # Safety
/// `ptr`/`len` must be the arguments the host allocated for this call.
pub unsafe fn callback(id: i64, ptr: i32, len: i32) {
    let bytes = unsafe { host::take_callback_args(ptr, len) };
    let args: Vec<Value> =
        if bytes.is_empty() { Vec::new() } else { serde_json::from_slice(&bytes).unwrap_or_default() };
    // Take the callback out while running it so a handler may register or cancel others.
    let cb = STATE.with(|s| s.borrow_mut().callbacks.remove(&(id as u64)));
    if let Some(mut cb) = cb {
        cb(args);
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            // Put it back unless the handler cancelled itself in the meantime.
            s.callbacks.entry(id as u64).or_insert(cb);
        });
    }
}

/// Runs the plugin's init closure once.
pub fn start(init: impl FnOnce()) {
    init();
}

/// The plugin is unloading: drop every registered callback.
pub fn stop() {
    STATE.with(|s| s.borrow_mut().callbacks.clear());
}

/// The id of a timer or watch, for [`cancel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerId(pub u64);

fn id_from(value: Value) -> TimerId {
    TimerId(value.as_u64().unwrap_or(0))
}

/// Adds a handler for an event. The arguments are whatever the event carries (see the events in
/// the documentation); use [`on_frame`], [`on_key`] and [`on_vehicle`] for the common ones.
pub fn on(event: &str, handler: impl FnMut(Vec<Value>) + 'static) {
    let id = register(Box::new(handler));
    let _ = raw_call("on", json!([event, cb_ref(id)]));
}

/// Removes handlers added with [`on`] for `event` (all of this plugin's, as the game tracks them
/// per plugin).
pub fn off(event: &str) {
    let _ = raw_call("off", json!([event]));
}

/// `start`: right after the plugin is loaded (also after a reload).
pub fn on_start(mut handler: impl FnMut() + 'static) {
    on("start", move |_| handler());
}

/// `frame`: every frame, with the seconds since the last one. Keep it short.
pub fn on_frame(mut handler: impl FnMut(f64) + 'static) {
    on("frame", move |args| handler(args.first().and_then(Value::as_f64).unwrap_or(0.0)));
}

/// `key`: a key went down (`true`) or came up (`false`).
pub fn on_key(mut handler: impl FnMut(String, bool) + 'static) {
    on("key", move |args| {
        let key = args.first().and_then(Value::as_str).unwrap_or("").to_string();
        let down = args.get(1).and_then(Value::as_bool).unwrap_or(false);
        handler(key, down);
    });
}

/// `vehicle`: the player got into a vehicle, changed it, or left it (`None`).
pub fn on_vehicle(mut handler: impl FnMut(Option<String>) + 'static) {
    on("vehicle", move |args| {
        handler(args.first().and_then(Value::as_str).map(str::to_string));
    });
}

/// Sends an event of your own to this plugin's handlers.
pub fn emit(event: &str, args: Vec<Value>) {
    let mut call_args = vec![Value::from(event)];
    call_args.extend(args);
    let _ = raw_call("emit", Value::Array(call_args));
}

/// Runs `handler` once after `seconds` of game time. Returns an id for [`cancel`].
pub fn after(seconds: f64, handler: impl FnMut() + 'static) -> TimerId {
    let mut handler = handler;
    let id = register(Box::new(move |_| handler()));
    id_from(raw_call("after", json!([seconds, cb_ref(id)])).unwrap_or(Value::Null))
}

/// Runs `handler` every `seconds` of game time. Returns an id for [`cancel`].
pub fn every(seconds: f64, handler: impl FnMut() + 'static) -> TimerId {
    let mut handler = handler;
    let id = register(Box::new(move |_| handler()));
    id_from(raw_call("every", json!([seconds, cb_ref(id)])).unwrap_or(Value::Null))
}

/// Runs `handler(new, old)` whenever a variable changes. `kind` is `"var"` (a script variable of
/// the bus), `"str"` (a string variable) or `"sys"` (a system variable).
pub fn watch(kind: &str, name: &str, handler: impl FnMut(Value, Value) + 'static) -> TimerId {
    let mut handler = handler;
    let id = register(Box::new(move |args: Vec<Value>| {
        let mut it = args.into_iter();
        handler(it.next().unwrap_or(Value::Null), it.next().unwrap_or(Value::Null));
    }));
    id_from(raw_call("watch", json!([kind, name, cb_ref(id)])).unwrap_or(Value::Null))
}

/// Stops a timer or a watch.
pub fn cancel(id: TimerId) {
    STATE.with(|s| {
        s.borrow_mut().callbacks.remove(&id.0);
    });
    let _ = raw_call("cancel", json!([id.0]));
}

/// A line in `game.log`, tagged with the plugin's name.
pub fn log(msg: &str) {
    host::host_log(1, msg);
}

/// A warning line in `game.log`.
pub fn warn(msg: &str) {
    host::host_log(2, msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    // On the host the call stub panics, so these exercise only the pure parts.
    #[test]
    fn cb_ref_shape() {
        assert_eq!(cb_ref(7), json!({ "$cb": 7 }));
    }

    #[test]
    fn ids_and_registry() {
        let a = register(Box::new(|_| {}));
        let b = register(Box::new(|_| {}));
        assert_ne!(a, b);
        assert_eq!(id_from(json!(5)), TimerId(5));
        assert_eq!(id_from(Value::Null), TimerId(0));
    }
}
