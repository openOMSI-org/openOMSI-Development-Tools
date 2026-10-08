# openomsi-plugin-sdk

Write [openOMSI](https://github.com/openOMSI-Project/openOMSI) plugins in Rust. The crate is the
guest side of the plugin platform: it turns the raw WebAssembly boundary into a safe, idiomatic
API - closures for events, timers and watches, and typed wrappers for every API function,
generated from the game's API manifest.

A plugin is a `cdylib` built for `wasm32-unknown-unknown` and packed into a `.oop` with `oopc`.
See the [getting started guide](https://openomsi-project.github.io/openOMSI-Development-Tools/).
