//! The golden test vectors in `tests/vectors/`: files every implementation of the format (the
//! game's loader included) must read to the same header and files.
//!
//! `golden-v1-signed.oop` is made with a fixed salt, nonce and key and stored (uncompressed)
//! zstd blocks, so it is byte-for-byte reproducible: if this test fails after a change, the
//! format changed. `golden-v1-unsigned.oop` uses the default compression and is only read.
//!
//! Regenerate (only for a deliberate format change): `OOP_GOLDEN_BLESS=1 cargo test -p oop-format --test golden`.

use std::path::PathBuf;

use oop_format::{Compression, Header, Kind, Oop, OopBuilder, decode_secret_key, fingerprint, inspect_details};

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

fn header() -> Header {
    let mut h = Header::new("org.openomsi.golden", "Golden vector", "1.2.0", Kind::Lua, "main.lua");
    h.authors = vec!["openOMSI".into()];
    h.description = "Test vector of the .oop format v1".into();
    h.min_openomsi = Some("0.2.22".into());
    h.permissions = vec!["ui".into(), "storage".into()];
    h.license = Some("MIT".into());
    h.homepage = Some("https://github.com/openOMSI-Project/openOMSI-Development-Tools".into());
    h.created = Some("2026-10-08T12:00:00Z".into());
    h.tool = Some("oopc 0.1.0".into());
    h
}

fn files() -> Vec<(String, Vec<u8>)> {
    vec![
        ("main.lua".into(), b"local util = require(\"lib.util\")\nomsi.message(util.greeting(), 5)\n".to_vec()),
        (
            "lib/util.lua".into(),
            b"return { greeting = function() return \"Hello from the golden vector\" end }\n".to_vec(),
        ),
        ("assets/bytes.bin".into(), (0..=255u8).collect()),
        ("empty.txt".into(), Vec::new()),
    ]
}

const SALT: [u8; 32] = [0x11; 32];
const NONCE: [u8; 24] = [0x22; 24];

fn signed_bytes() -> Vec<u8> {
    let key = decode_secret_key(&std::fs::read_to_string(dir().join("test-key.oopkey")).unwrap()).unwrap();
    OopBuilder::new(header())
        .files(files())
        .compression(Compression::Stored)
        .sign(&key)
        .write_with(SALT, NONCE)
        .unwrap()
}

fn check(oop: &Oop) {
    assert_eq!(oop.header(), &header());
    let mut want = files();
    want.sort();
    assert_eq!(oop.files(), want.as_slice());
}

#[test]
fn golden_vectors() {
    let bless = std::env::var_os("OOP_GOLDEN_BLESS").is_some();
    let signed_path = dir().join("golden-v1-signed.oop");
    let unsigned_path = dir().join("golden-v1-unsigned.oop");
    if bless {
        std::fs::write(&signed_path, signed_bytes()).unwrap();
        let unsigned = OopBuilder::new(header()).files(files()).write().unwrap();
        std::fs::write(&unsigned_path, unsigned).unwrap();
        let oop = Oop::read(&signed_bytes()).unwrap();
        let mut sorted = files();
        sorted.sort();
        let desc = serde_json::json!({
            "about": "Test vectors of the .oop format v1. golden-v1-signed.oop: salt 32 x 0x11, nonce 24 x 0x22, stored zstd blocks, signed with test-key.oopkey (seed 32 x 0x42; never trust it). golden-v1-unsigned.oop: random salt and nonce, default compression. Both hold the header and files below.",
            "header": header(),
            "signer_fingerprint": oop.signer_fingerprint(),
            "signer_public_key_hex": oop.signer().unwrap().as_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "files": sorted.iter().map(|(p, b)| serde_json::json!({"path": p, "size": b.len()})).collect::<Vec<_>>(),
        });
        std::fs::write(dir().join("golden-v1.json"), serde_json::to_string_pretty(&desc).unwrap() + "\n").unwrap();
    }

    let signed = std::fs::read(&signed_path).unwrap();
    assert_eq!(signed, signed_bytes(), "the writer no longer produces the golden bytes: the format changed");
    let oop = Oop::read(&signed).unwrap();
    check(&oop);
    let fp = std::fs::read_to_string(dir().join("golden-v1.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(&fp).unwrap();
    assert_eq!(oop.signer_fingerprint().as_deref(), expected["signer_fingerprint"].as_str());
    assert_eq!(fingerprint(oop.signer().unwrap()), expected["signer_fingerprint"].as_str().unwrap());
    assert_eq!(inspect_details(&signed).unwrap().signature_valid, Some(true));

    let unsigned = std::fs::read(&unsigned_path).unwrap();
    let oop = Oop::read(&unsigned).unwrap();
    check(&oop);
    assert!(oop.signer().is_none());

    // The description file lists the same files and sizes.
    let listed: Vec<(String, u64)> = expected["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["path"].as_str().unwrap().to_string(), f["size"].as_u64().unwrap()))
        .collect();
    let actual: Vec<(String, u64)> = oop.files().iter().map(|(p, b)| (p.clone(), b.len() as u64)).collect();
    assert_eq!(listed, actual);
}
