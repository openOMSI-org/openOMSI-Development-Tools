# The `.oop` file

A `.oop` is a **compiled** plugin: like a DLL, but sandboxed. It never contains your source code.

* **Lua** sources are run through an obfuscating Lua-to-Lua compiler: comments and formatting are
  dropped, every local is renamed, string literals are encoded, and the plugin's own modules are
  merged into one chunk. It is ordinary Lua text, not bytecode, because the game refuses binary
  chunks (Lua 5.4 does not verify bytecode).
* **Rust** compiles to a WebAssembly module with the name section, debug info and all other
  custom sections stripped.

The archive of build output is compressed (zstd), then encrypted with XChaCha20-Poly1305. The
header stays as plain JSON so tools can read it without the key.

## An honest security note

The encryption keeps a plugin's build output from being read or edited by casual users, and any
change to the file is detected by the authentication tag. But **the key ships inside the game**,
so it is not protection against a determined reverse engineer. What such a person can get is the
*stripped, obfuscated build output* - never your original source, which never enters the file.

**Signing** (Ed25519) is the real guarantee of origin: it proves who built a file. The game shows
"signed by `<fingerprint>`" and warns on unsigned or changed plugins. Share your public key so
players can check that a plugin is really yours.

## Layout

```
magic "OOP\x01" | version u16 | flags u16 | header_len u32 | header JSON
| salt[32] | nonce[24] | payload_len u64 | payload (XChaCha20-Poly1305 of zstd(archive))
| (if signed) Ed25519 signature[64] | signer public key[32]
```

The payload key is `HKDF-SHA256(FORMAT_KEY, salt, "openomsi-oop-v1" || SHA-256(header))`. Paths in
the archive are relative, with no `..`; readers cap the file count and unpacked size so a hostile
file cannot exhaust memory.
