# The Rust SDK

`openomsi-plugin-sdk` is the guest library for Rust plugins. A plugin is a `cdylib` built for
`wasm32-unknown-unknown`.

```rust
use openomsi_plugin_sdk as omsi;

omsi::plugin!(|| {
    // Register handlers here; this closure runs once at start.
    omsi::on_vehicle(|name| {
        if let Some(name) = name {
            let _ = omsi::api::message(&format!("You drive the {name}"), Some(6.0));
        }
    });

    omsi::every(0.5, || {
        let kmh = omsi::api::speed().unwrap_or(0.0);
        // build a HUD panel, etc.
        let _ = kmh;
    });
});
```

## What the SDK gives you

* **`plugin!`** defines the ABI exports the host calls - you never write `extern "C"` yourself.
* **Events**: `on_start`, `on_frame`, `on_key`, `on_vehicle`, and the generic `on(event, ...)`.
* **Timers and watches**: `after`, `every`, `watch`, `cancel`.
* **Typed API**: everything in `omsi::api`, generated from the game's API manifest, so each call
  has real argument and return types.
* **Escape hatch**: `omsi::call(name, json!([...]))` reaches any function the manifest lists,
  even one newer than this SDK's generated wrappers.

Data crosses the boundary as JSON (`serde_json::Value`). The SDK and the `oopc` build take care of
compiling to a stripped WebAssembly module.

## Regenerating the typed API

When the game ships a bigger API manifest, regenerate the wrappers with one command:

```sh
cargo xtask gen-sdk path/to/api.json
```
