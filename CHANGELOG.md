# Changelog

## 0.2.0 - 2026-10-23

The Development Tools are rebuilt from the ground up as a Tauri 2 app in the design of the openOMSI
launcher, with a built-in code editor and a one-button way to try a plugin in the game.

### New

- **A built-in code editor** (CodeMirror 6) for Lua, Rust, TOML and JSON: syntax highlighting,
  completion of every function and event of the plugin API (type `omsi.`), help for a name under
  the pointer, Lua syntax errors underlined as you type, search, comment toggling, a tab for every
  open file that keeps its own undo and position, and autosave.
- **Make and edit scripts in the app**: a file tree with new file, new folder, rename, delete and
  *add a file to assets*; three starters for Lua plugins (Hello, Panel, Blank) and one for Rust.
- **Run in openOMSI**: one button copies the plugin into the game's `plugins` folder and starts the
  game. With **Live** on, every save copies the changed files again and the running game loads the
  plugin again within a second. A Rust plugin is built and installed as a `.oop`.
- **Problems panel**: syntax errors, missing permissions (with an *Add permission* button that
  edits the manifest) and unused permissions, each one a click away from its line. Next to it, the
  output of builds and the game's own log.
- **Details page** instead of the manifest form: name, version with +patch/+minor/+major, authors,
  the starting script chosen from your files, the protection level explained in words, the
  permission cards with *Used by omsi.xxx* tags and **Match my code**, the license from a list.
  It saves itself and says why when a value is not valid.
- **Build and publish page**: check, sign, build, then show the file, copy its path, save a copy
  anywhere, install it into the game or open it in the inspector.
- **Every path has a Browse button** and the app finds openOMSI and `cargo` by itself. *Path to
  cargo* and *openOMSI game folder* are checked as you type, with a green or yellow line that says
  what was found. A button installs the WebAssembly target for Rust.
- **Signing keys** with a default key for builds, import, export of the public key, copy of the
  fingerprint and delete. **Inspector** opens a `.oop` (also by dropping it on the window) and can
  sign a file that was built without a key.
- **Documentation inside the app**: the guides and a searchable reference of all 271 functions and
  59 events, each with its permission and the version it came with.
- **Icons and a logo** in the openOMSI style for every system (`.ico`, `.icns`, PNG, SVG).
- **Updates itself like openOMSI and openMapper**: looks for a new release at start, offers it or
  installs it by itself (Settings), verifies the SHA-256, replaces the program and restarts. A
  development build only looks.
- **Every download is a `.zip`** (the app and `oopc`, for Windows, macOS and Linux, x64 and ARM64),
  with `SHA256SUMS`. No more `.tar.gz`.
- `oopc check` now reports the line and column of a Lua syntax error and fails on errors.

### Changes

- The app is no longer an egui program. It is a Tauri 2 window (the organisation's fork, as in the
  openOMSI launcher) with its own Inter font and Material Symbols, dark and light looks.
- The plugin API information in the app is the manifest of openOMSI 0.2.22 (271 functions, 59
  events), the same the SDK is generated from.
- CI builds and tests the app on Linux, macOS and Windows; the formatting check of the generated
  SDK is clean again.

## 0.1.0 - 2026-10-08

First release: `oopc`, the `.oop` format, the Lua obfuscating compiler, the WebAssembly stripper,
the Rust plugin SDK and a first desktop app.
