//! Hostile and damaged files: every one must give an error, none may panic, succeed, or make the
//! reader allocate far more than the file is worth.
//!
//! `raw_oop` below is an independent implementation of the container written from the spec
//! alone, so the tests can build files the builder refuses to make (bad paths, bombs, ...).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use oop_format::{
    Compression, Error, FORMAT_KEY_V1, Header, Kind, Limits, Oop, OopBuilder, SigningKey, inspect, inspect_details,
};
use sha2::{Digest, Sha256};

fn header() -> Header {
    let mut h = Header::new("com.example.test", "Test", "1.0.0", Kind::Lua, "main.lua");
    h.authors = vec!["Someone".into()];
    h.permissions = vec!["ui".into()];
    h
}

fn sample(signed: bool) -> Vec<u8> {
    let mut b = OopBuilder::new(header())
        .file("main.lua", b"local u = require('lib/util')\nomsi.message(u.hi())\n".to_vec())
        .file("lib/util.lua", b"return { hi = function() return 'hi' end }\n".to_vec())
        .file("assets/blob.bin", (0..=255u8).collect::<Vec<_>>());
    if signed {
        b = b.sign(&SigningKey::from_bytes(&[9; 32]));
    }
    b.write().unwrap()
}

/// zstd frame of raw blocks holding `data` (block size <= 128 KiB).
fn zstd_raw(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x28, 0xB5, 0x2F, 0xFD, 0x00, 0x38]; // magic, FHD, window 128 KiB
    let chunks: Vec<&[u8]> = if data.is_empty() { vec![&[][..]] } else { data.chunks(128 * 1024).collect() };
    for (i, c) in chunks.iter().enumerate() {
        let last = (i + 1 == chunks.len()) as u32;
        let bh = last | ((c.len() as u32) << 3); // type 0 = raw
        out.extend_from_slice(&bh.to_le_bytes()[..3]);
        out.extend_from_slice(c);
    }
    out
}

/// zstd frame of `blocks` RLE blocks of 128 KiB each: tiny input, big output.
fn zstd_bomb(blocks: usize) -> Vec<u8> {
    let mut out = vec![0x28, 0xB5, 0x2F, 0xFD, 0x00, 0x38];
    for i in 0..blocks {
        let last = (i + 1 == blocks) as u32;
        let bh = last | (1 << 1) | ((128 * 1024u32) << 3); // type 1 = RLE
        out.extend_from_slice(&bh.to_le_bytes()[..3]);
        out.push(0);
    }
    out
}

fn table(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut t = (files.len() as u32).to_le_bytes().to_vec();
    for (p, b) in files {
        t.extend_from_slice(&(p.len() as u16).to_le_bytes());
        t.extend_from_slice(p.as_bytes());
        t.extend_from_slice(&(b.len() as u64).to_le_bytes());
        t.extend_from_slice(b);
    }
    t
}

/// A container around `compressed` (already a zstd frame, or anything at all).
fn raw_oop(header_json: &str, compressed: &[u8]) -> Vec<u8> {
    let mut out = b"OOP\x01".to_vec();
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
    out.extend_from_slice(header_json.as_bytes());
    let salt = [5u8; 32];
    let nonce = [6u8; 24];
    let mut info = b"openomsi-oop-v1".to_vec();
    info.extend_from_slice(&Sha256::digest(header_json.as_bytes()));
    let mut key = [0u8; 32];
    hkdf::Hkdf::<Sha256>::new(Some(&salt), &FORMAT_KEY_V1).expand(&info, &mut key).unwrap();
    let ct = XChaCha20Poly1305::new(&Key::from(key))
        .encrypt(&XNonce::from(nonce), Payload { msg: compressed, aad: &out })
        .unwrap();
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&(ct.len() as u64).to_le_bytes());
    out.extend_from_slice(&ct);
    out
}

const HJ: &str = r#"{"id":"com.example.raw","name":"Raw","version":"1.0.0","kind":"lua","entry":"main.lua"}"#;

#[test]
fn round_trip_signed_and_unsigned() {
    for signed in [false, true] {
        let bytes = sample(signed);
        let oop = Oop::read(&bytes).unwrap();
        assert_eq!(oop.header(), &header());
        let paths: Vec<&str> = oop.files().iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, ["assets/blob.bin", "lib/util.lua", "main.lua"]);
        assert_eq!(oop.file("assets/blob.bin").unwrap().len(), 256);
        assert!(oop.entry().starts_with(b"local u"));
        assert_eq!(oop.signer().is_some(), signed);
        let d = inspect_details(&bytes).unwrap();
        assert_eq!(d.signature_valid, signed.then_some(true));
        assert_eq!(d.header.id, "com.example.test");
        assert_eq!(d.header_json, oop.header_json());
    }
}

#[test]
fn raw_helper_matches_the_reader() {
    let t = table(&[("main.lua", b"x")]);
    let oop = Oop::read(&raw_oop(HJ, &zstd_raw(&t))).unwrap();
    assert_eq!(oop.files(), &[("main.lua".to_string(), b"x".to_vec())]);
}

#[test]
fn empty_files_and_stored_compression() {
    let bytes =
        OopBuilder::new(header()).file("main.lua", Vec::new()).compression(Compression::Stored).write().unwrap();
    let oop = Oop::read(&bytes).unwrap();
    assert_eq!(oop.entry(), b"");
}

#[test]
fn every_truncation_fails_cleanly() {
    for signed in [false, true] {
        let bytes = sample(signed);
        for len in 0..bytes.len() {
            let r = Oop::read(&bytes[..len]);
            assert!(r.is_err(), "truncated to {len} bytes was accepted");
        }
    }
}

#[test]
fn every_single_bit_flip_is_detected() {
    for signed in [false, true] {
        let bytes = sample(signed);
        for i in 0..bytes.len() {
            for bit in [0x01u8, 0x80] {
                let mut b = bytes.clone();
                b[i] ^= bit;
                assert!(Oop::read(&b).is_err(), "flipping bit {bit:#x} of byte {i} (signed {signed}) went unnoticed");
            }
        }
    }
}

#[test]
fn header_tampering_breaks_the_tag() {
    let bytes = sample(false);
    // Same length, still valid JSON and a valid header: only the AEAD tag can notice.
    let from = br#""name":"Test""#;
    let pos = bytes.windows(from.len()).position(|w| w == from).unwrap();
    let mut b = bytes.clone();
    b[pos..pos + from.len()].copy_from_slice(br#""name":"Evil""#);
    assert_eq!(inspect(&b).unwrap().name, "Evil");
    assert_eq!(Oop::read(&b).unwrap_err(), Error::Decrypt);

    // Turning on the signed flag of an unsigned file (and appending junk) is caught too.
    let mut b = bytes.clone();
    b[6] |= 1;
    b.extend_from_slice(&[0; 96]);
    assert!(matches!(Oop::read(&b), Err(Error::BadSignature)));
}

#[test]
fn wrong_signature_and_key_swap() {
    let bytes = sample(true);
    let n = bytes.len();
    // Another key's public half in place of the signer's.
    let mut b = bytes.clone();
    b[n - 32..].copy_from_slice(SigningKey::from_bytes(&[1; 32]).verifying_key().as_bytes());
    assert_eq!(Oop::read(&b).unwrap_err(), Error::BadSignature);
    assert_eq!(inspect_details(&b).unwrap().signature_valid, Some(false));
    // Re-signing someone else's file with your key is possible (and then it is yours): the
    // signer changes, which the game shows. A signature over different bytes is not.
    let mut b = bytes.clone();
    b[n - 96] ^= 0x55;
    assert_eq!(Oop::read(&b).unwrap_err(), Error::BadSignature);
    // Stripping the signature but leaving the flag.
    assert!(Oop::read(&bytes[..n - 96]).is_err());
}

#[test]
fn not_an_oop_file() {
    assert_eq!(Oop::read(b"PK\x03\x04hello").unwrap_err(), Error::BadMagic);
    assert_eq!(Oop::read(b"").unwrap_err(), Error::Truncated { what: "magic" });
    assert_eq!(Oop::read(b"OO").unwrap_err(), Error::Truncated { what: "magic" });
    assert_eq!(Oop::read(b"XY").unwrap_err(), Error::BadMagic);
    let mut b = sample(false);
    b[4] = 2;
    assert_eq!(Oop::read(&b).unwrap_err(), Error::UnsupportedVersion(2));
    let mut b = sample(false);
    b[6] = 0x04;
    assert_eq!(Oop::read(&b).unwrap_err(), Error::UnknownFlags(4));
    let mut b = sample(false);
    b.push(0);
    assert_eq!(Oop::read(&b).unwrap_err(), Error::TrailingData(1));
}

#[test]
fn oversized_header_length_is_refused_before_reading() {
    let mut b = b"OOP\x01\x01\x00\x00\x00".to_vec();
    b.extend_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(Oop::read(&b).unwrap_err(), Error::HeaderTooLarge(u32::MAX as u64));
    let mut b = b"OOP\x01\x01\x00\x00\x00".to_vec();
    b.extend_from_slice(&100u32.to_le_bytes());
    b.extend_from_slice(b"{}");
    assert_eq!(Oop::read(&b).unwrap_err(), Error::Truncated { what: "header" });
}

#[test]
fn huge_payload_length_is_refused() {
    let bytes = sample(false);
    let hl = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let at = 12 + hl + 56;
    let mut b = bytes.clone();
    b[at..at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    assert_eq!(Oop::read(&b).unwrap_err(), Error::Truncated { what: "payload" });
}

#[test]
fn path_traversal_is_refused() {
    for bad in ["../evil.lua", "/etc/passwd", "a/../../b", "C:/Windows/x", "a\\..\\b", "./main.lua", "a//b", ""] {
        let t = table(&[(bad, b"x"), ("main.lua", b"x")]);
        // Unsorted on purpose for some: either error is fine, never success.
        let r = Oop::read(&raw_oop(HJ, &zstd_raw(&t)));
        assert!(r.is_err(), "{bad:?} accepted");
        let t = table(&[("main.lua", b"x"), ("zz/../../evil", b"x")]);
        assert!(matches!(Oop::read(&raw_oop(HJ, &zstd_raw(&t))), Err(Error::BadPath(_))));
    }
    // The builder refuses them as well.
    let r = OopBuilder::new(header()).file("main.lua", b"".to_vec()).file("../x", b"".to_vec()).write();
    assert!(matches!(r, Err(Error::BadPath(_))));
}

#[test]
fn duplicate_and_unsorted_tables_are_refused() {
    let t = table(&[("main.lua", b"1"), ("main.lua", b"2")]);
    assert!(matches!(Oop::read(&raw_oop(HJ, &zstd_raw(&t))), Err(Error::DuplicatePath(_))));
    let t = table(&[("main.lua", b"1"), ("a.lua", b"2")]);
    assert!(matches!(Oop::read(&raw_oop(HJ, &zstd_raw(&t))), Err(Error::DuplicatePath(_))));
}

#[test]
fn missing_entry_is_refused() {
    let t = table(&[("other.lua", b"1")]);
    assert_eq!(Oop::read(&raw_oop(HJ, &zstd_raw(&t))).unwrap_err(), Error::MissingEntry("main.lua".into()));
    let r = OopBuilder::new(header()).file("x.lua", b"".to_vec()).write();
    assert_eq!(r.unwrap_err(), Error::MissingEntry("main.lua".into()));
}

#[test]
fn hostile_header_ids_are_refused() {
    for id in ["../../evil", "com/evil", "noseparator", "com.ev il"] {
        let hj = HJ.replace("com.example.raw", id);
        let t = table(&[("main.lua", b"1")]);
        assert!(matches!(Oop::read(&raw_oop(&hj, &zstd_raw(&t))), Err(Error::InvalidHeader(_))), "{id}");
    }
    let t = table(&[("main.lua", b"1")]);
    assert!(matches!(Oop::read(&raw_oop("[1,2]", &zstd_raw(&t))), Err(Error::HeaderJson(_))));
    assert!(matches!(Oop::read(&raw_oop("\u{0}", &zstd_raw(&t))), Err(Error::HeaderJson(_))));
}

#[test]
fn decompression_bomb_stops_at_the_limit() {
    let limits = Limits { max_unpacked: 1 << 20, ..Limits::default() };
    // 64 RLE blocks: 260 bytes that unpack to 8 MiB.
    let bomb = raw_oop(HJ, &zstd_bomb(64));
    assert!(bomb.len() < 2048);
    assert_eq!(Oop::read_with_limits(&bomb, &limits).unwrap_err(), Error::TooLarge);
    // Exactly at the limit is fine as far as the size goes (the table itself is junk here).
    let at_limit = raw_oop(HJ, &zstd_bomb(8));
    assert!(matches!(Oop::read_with_limits(&at_limit, &limits), Err(Error::TooManyFiles(_) | Error::Archive(_))));
}

#[test]
fn too_many_files_are_refused() {
    let limits = Limits { max_files: 3, ..Limits::default() };
    let files: Vec<(String, &[u8])> = (0..4).map(|i| (format!("f{i}.lua"), &b""[..])).collect();
    let refs: Vec<(&str, &[u8])> = files.iter().map(|(p, b)| (p.as_str(), *b)).collect();
    let t = table(&refs);
    let hj = HJ.replace("main.lua", "f0.lua");
    assert_eq!(Oop::read_with_limits(&raw_oop(&hj, &zstd_raw(&t)), &limits).unwrap_err(), Error::TooManyFiles(4));
    let mut b = OopBuilder::new(Header::new("a.b", "x", "1.0.0", Kind::Lua, "f0.lua")).limits(limits);
    for i in 0..4 {
        b = b.file(format!("f{i}.lua"), Vec::new());
    }
    assert_eq!(b.write().unwrap_err(), Error::TooManyFiles(4));
}

#[test]
fn garbage_payloads() {
    // Valid tag around bytes that are not zstd, and zstd followed by junk.
    assert!(matches!(Oop::read(&raw_oop(HJ, b"not zstd at all")), Err(Error::Decompress(_))));
    let t = table(&[("main.lua", b"1")]);
    let mut z = zstd_raw(&t);
    z.extend_from_slice(b"junk");
    assert!(matches!(Oop::read(&raw_oop(HJ, &z)), Err(Error::Decompress(_))));
    let mut t2 = t.clone();
    t2.push(0);
    assert!(matches!(Oop::read(&raw_oop(HJ, &zstd_raw(&t2))), Err(Error::Archive(_))));
}

#[test]
fn pseudo_random_garbage_never_panics() {
    // A small xorshift: deterministic, no extra dependency.
    let mut s = 0x2545F4914F6CDD1Du64;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let base = sample(true);
    for _ in 0..3000 {
        let mut b = base.clone();
        let edits = 1 + next() % 8;
        for _ in 0..edits {
            let i = (next() as usize) % b.len();
            b[i] = next() as u8;
        }
        if next() % 4 == 0 {
            let cut = (next() as usize) % b.len();
            b.truncate(cut);
        }
        let _ = Oop::read(&b);
        let _ = inspect_details(&b);
    }
    for len in [0usize, 1, 11, 12, 13, 64, 200] {
        let junk: Vec<u8> = (0..len).map(|_| next() as u8).collect();
        assert!(Oop::read(&junk).is_err());
        let mut j = b"OOP\x01\x01\x00\x00\x00".to_vec();
        j.extend_from_slice(&junk);
        assert!(Oop::read(&j).is_err());
    }
}
