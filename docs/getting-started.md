# Getting started

openOMSI Development Tools build **plugins** for [openOMSI](https://github.com/openOMSI-org/openOMSI).
A plugin is a single `.oop` file you drop into the game's `plugins` folder. You write it in Lua or
in Rust; `oopc` compiles and packs it.

## Install

Download the release for your system from the
[releases page](https://github.com/openOMSI-org/openOMSI-Development-Tools/releases) and put
`oopc` (and, if you like, the `openomsi-devtools` app) on your `PATH`. Rust plugins also need the
Rust toolchain and the WebAssembly target:

```sh
rustup target add wasm32-unknown-unknown
```

## Your first plugin (Lua)

```sh
oopc new hello --lua
cd hello
oopc build
```

`oopc build` writes `dist/com.example.hello.oop`. Copy it into openOMSI's `plugins` folder and
start a map. Edit `main.lua` and build again to change it.

```lua
-- main.lua
omsi.on("start", function()
  omsi.message("Hello from my plugin!", 5)
end)

omsi.every(1, function()
  local kmh = omsi.var("Velocity")
  if kmh and kmh > 50 then
    omsi.message(string.format("Slow down: %.0f km/h", kmh), 1)
  end
end)
```

## Your first plugin (Rust)

```sh
oopc new hello --rust
cd hello
oopc build
```

```rust
use openomsi_plugin_sdk as omsi;

omsi::plugin!(|| {
    omsi::on_start(|| {
        let _ = omsi::api::message("Hello from my plugin!", Some(5.0));
    });
});
```

A Rust plugin compiles to WebAssembly and runs in the game's sandbox. The SDK gives you a typed
function for every API call and closures for events, timers and watches.

## Signing

Make a key once, then sign your builds so players see "signed by ..." instead of a warning:

```sh
oopc keygen
oopc build --sign ~/.config/openOMSI-devtools/keys/my-openOMSI-key.oopkey
```
