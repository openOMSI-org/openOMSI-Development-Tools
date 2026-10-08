//! Removing everything from a WebAssembly module that could help read it back: the `name`
//! section, DWARF `.debug_*`, `producers`, `target_features`, a `sourceMappingURL`, and any other
//! custom section. A `.oop` plugin is a compiled artefact, so the module that ships in it carries
//! no symbol names or debug info - a decompiler sees only nameless, low-level code.
//!
//! This is the pure-Rust pass that always runs. `oopc` also runs `wasm-opt` first when it is on
//! the `PATH`, which optimizes as well; this pass then guarantees the result is stripped whatever
//! flags `wasm-opt` used.
//!
//! The approach is deliberately simple and safe: a WebAssembly module is a header followed by a
//! sequence of sections, each `id:u8, size:uleb128, contents[size]`. Custom sections have id 0.
//! We copy every non-custom section's bytes verbatim and drop every custom one, then check the
//! result still parses. Nothing in a known section is touched, so a valid module stays valid.

use std::fmt;

/// Why stripping failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Not a WebAssembly module (wrong magic or version).
    NotWasm,
    /// The section framing is malformed (truncated, or a size that runs off the end).
    Malformed(String),
    /// The stripped module no longer parses - a bug here, never from valid input.
    BrokeModule(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotWasm => f.write_str("not a WebAssembly module (wrong magic bytes)"),
            Error::Malformed(e) => write!(f, "malformed WebAssembly module: {e}"),
            Error::BrokeModule(e) => write!(f, "internal error: stripping broke the module: {e}"),
        }
    }
}

impl std::error::Error for Error {}

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// Reads a little-endian base-128 unsigned integer, returning the value and the new offset.
fn read_uleb(data: &[u8], mut pos: usize) -> Result<(u64, usize), Error> {
    let mut result: u64 = 0;
    let mut shift = 0;
    loop {
        let byte = *data.get(pos).ok_or_else(|| Error::Malformed("truncated section size".into()))?;
        pos += 1;
        if shift >= 64 {
            return Err(Error::Malformed("section size too large".into()));
        }
        result |= u64::from(byte & 0x7F) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    Ok((result, pos))
}

/// Strips a module. The result keeps the same header and every non-custom section unchanged.
pub fn strip(wasm: &[u8]) -> Result<Vec<u8>, Error> {
    if wasm.len() < 8 || wasm[..4] != WASM_MAGIC {
        return Err(Error::NotWasm);
    }
    let mut out = Vec::with_capacity(wasm.len());
    out.extend_from_slice(&wasm[..8]); // magic + version
    let mut pos = 8;
    let mut dropped = 0usize;
    while pos < wasm.len() {
        let section_start = pos;
        let id = wasm[pos];
        pos += 1;
        let (size, after_size) = read_uleb(wasm, pos)?;
        let size = size as usize;
        let body_end = after_size.checked_add(size).ok_or_else(|| Error::Malformed("section size overflow".into()))?;
        if body_end > wasm.len() {
            return Err(Error::Malformed(format!("section of {size} bytes runs past the end of the module")));
        }
        if id == 0 {
            // A custom section: drop it whole (name, .debug_*, producers, target_features, ...).
            dropped += 1;
        } else {
            out.extend_from_slice(&wasm[section_start..body_end]);
        }
        pos = body_end;
    }
    let _ = dropped;
    // A valid module in must give a valid module out; prove it before handing the bytes on.
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&out)
        .map_err(|e| Error::BrokeModule(e.to_string()))?;
    Ok(out)
}

/// The custom section names this pass removes (it actually removes every custom section; this is
/// the list named in the plugin spec, for documentation and messages).
pub const STRIPPED_SECTIONS: &[&str] =
    &["name", ".debug_* (DWARF)", "producers", "target_features", "sourceMappingURL"];

/// How many custom sections a module has (what stripping will remove), for a report line.
pub fn count_custom_sections(wasm: &[u8]) -> Result<usize, Error> {
    if wasm.len() < 8 || wasm[..4] != WASM_MAGIC {
        return Err(Error::NotWasm);
    }
    let mut pos = 8;
    let mut n = 0;
    while pos < wasm.len() {
        let id = wasm[pos];
        pos += 1;
        let (size, after) = read_uleb(wasm, pos)?;
        let end = after + size as usize;
        if end > wasm.len() {
            return Err(Error::Malformed("truncated".into()));
        }
        if id == 0 {
            n += 1;
        }
        pos = end;
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a tiny but valid module with a `name` custom section and a function.
    fn sample_module() -> Vec<u8> {
        // (module (func (export "f") (result i32) i32.const 7))
        let mut m = Vec::new();
        m.extend_from_slice(&WASM_MAGIC);
        m.extend_from_slice(&[0x01, 0, 0, 0]); // version 1
        // type section: one type () -> i32
        section(&mut m, 1, &[0x01, 0x60, 0x00, 0x01, 0x7F]);
        // function section: one function of type 0
        section(&mut m, 3, &[0x01, 0x00]);
        // export section: "f" -> func 0
        section(&mut m, 7, &[0x01, 0x01, b'f', 0x00, 0x00]);
        // code section: one body: no locals, i32.const 7, end
        section(&mut m, 10, &[0x01, 0x04, 0x00, 0x41, 0x07, 0x0B]);
        // custom "name" section (function names)
        let mut name = Vec::new();
        name.push(0x04);
        name.extend_from_slice(b"name");
        name.extend_from_slice(&[0x01, 0x03, 0x01, 0x00, 0x00]); // a tiny name subsection
        section(&mut m, 0, &name);
        // custom "producers"
        let mut prod = Vec::new();
        prod.push(0x09);
        prod.extend_from_slice(b"producers");
        prod.extend_from_slice(&[0x00]);
        section(&mut m, 0, &prod);
        m
    }

    fn section(m: &mut Vec<u8>, id: u8, body: &[u8]) {
        m.push(id);
        // uleb of body length (bodies here are < 128 bytes)
        assert!(body.len() < 128);
        m.push(body.len() as u8);
        m.extend_from_slice(body);
    }

    #[test]
    fn strips_custom_sections_and_stays_valid() {
        let m = sample_module();
        assert_eq!(count_custom_sections(&m).unwrap(), 2);
        assert!(m.windows(4).any(|w| w == b"name"));
        let stripped = strip(&m).unwrap();
        assert_eq!(count_custom_sections(&stripped).unwrap(), 0);
        assert!(!stripped.windows(4).any(|w| w == b"name"));
        assert!(!stripped.windows(9).any(|w| w == b"producers"));
        assert!(stripped.len() < m.len());
        // Re-stripping a stripped module changes nothing.
        assert_eq!(strip(&stripped).unwrap(), stripped);
    }

    #[test]
    fn rejects_non_wasm() {
        assert_eq!(strip(b"not wasm at all").unwrap_err(), Error::NotWasm);
        assert_eq!(strip(&[]).unwrap_err(), Error::NotWasm);
        let mut bad = WASM_MAGIC.to_vec();
        bad.extend_from_slice(&[0x01, 0, 0, 0, 0x01, 0x7F]); // section claims 127 bytes, none follow
        assert!(matches!(strip(&bad), Err(Error::Malformed(_))));
    }
}
