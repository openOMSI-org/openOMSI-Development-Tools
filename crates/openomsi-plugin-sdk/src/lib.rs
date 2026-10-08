//! Write openOMSI plugins in Rust.
//!
//! A plugin is a `cdylib` that compiles to `wasm32-unknown-unknown` and is packed into a `.oop`
//! file by `oopc`. This crate is the guest side of the plugin platform: it turns the raw
//! WebAssembly boundary (see the plugin spec, section 2) into a safe, idiomatic API - closures
//! for events, timers and watches, and typed wrappers for every API function, generated from the
//! game's API manifest.
//!
//! ```no_run
//! openomsi_plugin_sdk::plugin!(|| {
//!     openomsi_plugin_sdk::on_vehicle(|name| {
//!         if let Some(name) = name {
//!             let _ = openomsi_plugin_sdk::api::message(&format!("You drive the {name}"), Some(6.0));
//!         }
//!     });
//!
//!     openomsi_plugin_sdk::every(1.0, || {
//!         if let Ok(Some(kmh)) = openomsi_plugin_sdk::api::var("Velocity")
//!             && kmh > 50.0
//!         {
//!             let _ = openomsi_plugin_sdk::api::message("Slow down", Some(1.0));
//!         }
//!     });
//! });
//! ```
//!
//! The entry point is the [`plugin!`] macro: it defines the `extern "C"` exports the host calls
//! (`oop_abi`, `oop_alloc`, `oop_free`, `oop_start`, `oop_callback`, `oop_stop`) and runs the
//! closure you give it once, when the plugin starts, so you can register handlers there.
//!
//! Data goes to and from the game as [`serde_json::Value`]. For a function the manifest does not
//! yet cover, [`call`] is the escape hatch.

#![doc(html_no_source)]

mod host;
mod runtime;

pub mod api;

pub use runtime::{
    TimerId, after, cancel, emit, every, log, off, on, on_frame, on_key, on_start, on_vehicle, warn, watch,
};
pub use serde_json::{Value, json};

/// An error from a call into the game: the message the host gave, or a problem encoding the
/// arguments or decoding the result.
#[derive(Debug, Clone)]
pub struct Error(pub String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

/// The result of a call into the game.
pub type Result<T> = std::result::Result<T, Error>;

/// Calls any API function by name with a JSON array of arguments, returning its result as a
/// [`Value`]. The typed wrappers in [`api`] are built on this; use it directly for a function the
/// manifest this SDK was generated from does not have yet.
///
/// ```no_run
/// let speed = openomsi_plugin_sdk::call("speed", openomsi_plugin_sdk::json!([]))?;
/// # Ok::<(), openomsi_plugin_sdk::Error>(())
/// ```
pub fn call(name: &str, args: Value) -> Result<Value> {
    runtime::raw_call(name, args)
}

/// The ABI version this SDK speaks (1). The game checks it against the module's `oop_abi` export.
pub const ABI: i32 = 1;

/// Defines the plugin's `extern "C"` exports and runs `init` once at start.
///
/// Put the macro at the crate root of your plugin's `cdylib`. `init` is any `FnOnce()` - usually
/// a closure that registers event handlers.
#[macro_export]
macro_rules! plugin {
    ($init:expr) => {
        /// Exported: the ABI version the host checks.
        #[unsafe(no_mangle)]
        pub extern "C" fn oop_abi() -> i32 {
            $crate::ABI
        }
        /// Exported: allocate a buffer for the host to write into.
        #[unsafe(no_mangle)]
        pub extern "C" fn oop_alloc(len: i32) -> i32 {
            $crate::__rt::alloc(len)
        }
        /// Exported: free a buffer from `oop_alloc`.
        ///
        /// # Safety
        /// `ptr`/`len` must come from `oop_alloc`.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn oop_free(ptr: i32, len: i32) {
            unsafe { $crate::__rt::free(ptr, len) }
        }
        /// Exported: run the plugin's init.
        #[unsafe(no_mangle)]
        pub extern "C" fn oop_start() {
            $crate::__rt::start($init)
        }
        /// Exported: invoke a registered callback.
        ///
        /// # Safety
        /// `args_ptr`/`args_len` must be the JSON arguments the host allocated.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn oop_callback(id: i64, args_ptr: i32, args_len: i32) {
            unsafe { $crate::__rt::callback(id, args_ptr, args_len) }
        }
        /// Exported: the plugin is unloading.
        #[unsafe(no_mangle)]
        pub extern "C" fn oop_stop() {
            $crate::__rt::stop()
        }
    };
}

/// Internals the [`plugin!`] macro calls. Not a stable API.
#[doc(hidden)]
pub mod __rt {
    pub use crate::host::{alloc, free};
    pub use crate::runtime::{callback, start, stop};
}
