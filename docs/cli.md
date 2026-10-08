# `oopc` command reference

| Command | What it does |
| --- | --- |
| `oopc new <dir> --lua` / `--rust` | Start a new project from a template. |
| `oopc build [dir]` | Compile the project and pack a `.oop` into `dist/`. `--sign <key>` signs it, `-o <file>` chooses the output. |
| `oopc pack <dir>` | Pack a directory of already-prepared files (a lower-level tool). |
| `oopc keygen` | Make a signing key (stored in your config directory by default). |
| `oopc sign <file.oop> --key <key>` | Sign or re-sign a `.oop`. |
| `oopc verify <file.oop> [--key <pub>]` | Check the signature and show who signed it. |
| `oopc inspect <file.oop>` | Show the header and the file list (sizes only, never contents). |
| `oopc check [dir]` | Lint the project: id and version formats, the entry file, and declared vs used permissions. |

There is deliberately no `unpack`: a `.oop` holds compiled build output, and the point of the
format is that sources are not handed out casually.

## The project manifest

`openomsi-plugin.toml`:

```toml
[plugin]
id = "com.author.name"
name = "My Plugin"
version = "1.0.0"
kind = "lua"            # or "rust"
entry = "main.lua"      # Lua only
authors = ["you"]
description = "..."
min_openomsi = "0.2.21"
permissions = ["ui", "storage"]
license = "MIT"

[lua]
obfuscation_level = 2   # 0 strip only, 1 rename, 2 also encode strings and numbers
```
