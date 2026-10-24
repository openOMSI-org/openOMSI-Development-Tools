# Getting started

openOMSI Development Tools make **plugins** for [openOMSI](https://github.com/openOMSI-org/openOMSI).
You write a script in the built-in editor, run it in the game with one button, and build a single
signed `.oop` file to share. A plugin is written in Lua (nothing else to install) or in Rust.

## Install

Download the `.zip` for your system from the
[releases page](https://github.com/openOMSI-org/openOMSI-Development-Tools/releases):

| System | File |
| --- | --- |
| Windows | `openOMSI-Dev-Tools-<version>-windows-x64.zip` (or `windows-arm64`) |
| macOS | `openOMSI-Dev-Tools-<version>-macos-arm64.zip` (Apple silicon) or `macos-x64` (Intel) |
| Linux | `openOMSI-Dev-Tools-<version>-linux-x64.zip` (or `linux-arm64`) |

Unpack it and start **openOMSI Dev Tools**. The app updates itself: when a new version is out it asks
(or installs it by itself, see Settings), after checking the download against its SHA-256.

On Windows the app needs the WebView2 runtime that comes with Windows 10 and 11. On Linux it needs
`libwebkit2gtk-4.1`.

## Your first plugin

1. Press **New plugin**, give it a name and press **Create plugin**. A Lua plugin with a small
   example is made and opened in the editor.
2. Press **Run in openOMSI**. The first time the app asks where openOMSI is (it looks in the usual
   places first); it copies the plugin into the game's `plugins` folder and starts the game.
3. Start a drive. The plugin runs. Change the script and save: the app copies it again and the
   running game loads it within a second (the **Live** switch on top turns that on and off).

The editor knows every function of the plugin API. Type `omsi.` for the list, point at a name for
its help, and mistakes are underlined as you type. The panel below the editor lists problems (click
one to jump to it), the output of builds, and the game's log.

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

## Details and permissions

The **Details** page is the plugin's `openomsi-plugin.toml` as a form that saves itself: name,
version, description, the starting script, and the **permissions** the plugin asks for. A plugin
that shows panels needs `ui`, one that keeps data needs `storage`, and so on. The editor warns when
the code uses something that is not ticked and offers to tick it; **Match my code** ticks exactly
what the code uses.

## Build and share

On **Build and publish**: check the plugin, choose whether to sign it, press **Build**. The result is
one `.oop` file that works on Windows, macOS and Linux. Your source never goes into it (see
[the .oop format](oop-format.md)). Players put it into the `plugins` folder of openOMSI.

## Signing

Open **Signing keys**, press **New key**, and turn on **Sign this build** when you build. Players
then see "signed by" and your fingerprint in the game instead of a warning. See
[publishing and signing](publishing.md).

## Plugins in Rust

Choose **Rust** in the New plugin dialog. A Rust plugin is compiled to WebAssembly and runs in the
game's sandbox; the SDK gives a typed function for every API call. It needs the Rust toolchain
(rustup.rs) and one extra target, which **Settings** installs with one button. The app finds
`cargo` by itself; set its path in Settings only if it is somewhere unusual.

```rust
use openomsi_plugin_sdk as omsi;

omsi::plugin!(|| {
    omsi::on_start(|| {
        let _ = omsi::api::message("Hello from my plugin!", Some(5.0));
    });
});
```

## The command line

Everything the app builds with is also a program, `oopc`, for scripts and CI (download
`oopc-<version>-<system>.zip`). See the [command reference](cli.md).

```sh
oopc new hello --lua
oopc build hello
```
