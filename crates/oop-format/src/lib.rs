//! Reading and writing `.oop` files, the packed plugins of [openOMSI](https://github.com/openOMSI-Project/openOMSI).
//!
//! ```text
//! offset  size  field
//! 0       4     magic "OOP\x01"
//! 4       2     format version, u16 LE = 1
//! 6       2     flags u16 LE: bit0 signed
//! 8       4     header_len u32 LE
//! 12      N     header: UTF-8 JSON (plain text)
//! ..      32    salt (random per file)
//! ..      24    nonce (random per file)
//! ..      8     payload_len u64 LE
//! ..      P     payload: XChaCha20-Poly1305 ciphertext+tag of zstd(archive); AAD = bytes 0..12+N
//! ..      64    (if signed) Ed25519 signature over everything before it
//! ..      32    (if signed) the signer's Ed25519 public key
//! ```
//!
//! The payload key is `HKDF-SHA256(ikm = FORMAT_KEY_V1, salt, info = "openomsi-oop-v1" ||
//! SHA-256(header))`. The archive is a sorted file table (see [`check_path`] for the path rules).
//!
//! **What the encryption is for, honestly:** it keeps a plugin's sources from being read or
//! edited by casual users, and any change to the file is detected (the AEAD tag covers the
//! header too). The key ships inside the game, so it does not stop a determined reverse
//! engineer. Signing proves who built a file: [`Oop::signer`] is the Ed25519 key that signed it,
//! and [`fingerprint`] the string the game shows for it.
//!
//! ```
//! use oop_format::{Header, Kind, Oop, OopBuilder, generate_key};
//!
//! let key = generate_key()?;
//! let header = Header::new("com.example.hello", "Hello", "1.0.0", Kind::Lua, "main.lua");
//! let bytes = OopBuilder::new(header)
//!     .file("main.lua", b"omsi.message('hi')".to_vec())
//!     .sign(&key)
//!     .write()?;
//!
//! let header = oop_format::inspect(&bytes)?; // without decrypting
//! assert_eq!(header.id, "com.example.hello");
//!
//! let oop = Oop::read(&bytes)?; // decrypts, checks the tag, the signature and every path
//! assert_eq!(oop.files()[0].0, "main.lua");
//! assert_eq!(oop.signer(), Some(&key.verifying_key()));
//! # Ok::<(), oop_format::Error>(())
//! ```
//!
//! The crate is pure Rust (RustCrypto, `ruzstd`) and builds for `wasm32-unknown-unknown` too.
//!
//! About compression: the only pure-Rust zstd encoder, `ruzstd`'s, implements its "fastest"
//! level (about zstd level 1). That is what [`OopBuilder`] uses by default. Every reader decodes
//! any valid zstd frame, so files made later with a stronger encoder stay readable here.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod archive;
mod error;
mod header;
mod keys;
pub mod permissions;

use std::io::Read as _;

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use ed25519_dalek::{Signature, Signer as _};
use sha2::{Digest, Sha256};

pub use archive::check_path;
pub use error::{Error, Result};
pub use header::{API_ABI, Header, Kind, check_id, check_version};
pub use keys::{
    SigningKey, VerifyingKey, decode_public_key, decode_secret_key, encode_public_key, encode_secret_key, fingerprint,
    generate_key,
};

/// The first four bytes of every `.oop` file.
pub const MAGIC: [u8; 4] = *b"OOP\x01";
/// The format version this crate reads and writes.
pub const FORMAT_VERSION: u16 = 1;
/// Flag bit 0: the file ends in a signature and the signer's public key.
pub const FLAG_SIGNED: u16 = 1;
/// The HKDF info prefix (the SHA-256 of the header follows it).
pub const HKDF_INFO_V1: &[u8] = b"openomsi-oop-v1";

/// The input key material of format version 1. It is public by nature (every game has it): it
/// makes the payload unreadable to casual users, it is not a secret against reverse engineers.
pub const FORMAT_KEY_V1: [u8; 32] = [
    0x81, 0xae, 0x5c, 0x68, 0x19, 0x36, 0x60, 0xed, 0xbf, 0x2c, 0x6e, 0x23, 0x8b, 0x20, 0x3d, 0x56, 0x96, 0x1c, 0x28,
    0x49, 0x31, 0xc7, 0x52, 0xa7, 0x76, 0x45, 0xf3, 0x0f, 0x86, 0x66, 0x67, 0x82,
];

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const TAG_LEN: u64 = 16;
const SIG_LEN: usize = 64;
const PUBKEY_LEN: usize = 32;

/// The guards against hostile files ("zip bombs" and the like). [`Limits::default`] is what the
/// format allows; a reader may choose smaller ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// The largest header in bytes (default 1 MiB).
    pub max_header_len: u64,
    /// The most files in a plugin (default 10 000).
    pub max_files: u64,
    /// The largest unpacked archive in bytes, file table included (default 512 MiB).
    pub max_unpacked: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { max_header_len: 1 << 20, max_files: 10_000, max_unpacked: 512 << 20 }
    }
}

/// How much [`OopBuilder`] compresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    /// zstd's fastest level (the only one the pure-Rust encoder has).
    #[default]
    Fastest,
    /// A zstd frame of raw blocks: no compression (still a valid zstd stream).
    Stored,
}

/// The parts of a file up to the payload, located but not decrypted.
struct Layout<'a> {
    flags: u16,
    header_bytes: &'a [u8],
    aad: &'a [u8],
    salt: &'a [u8],
    nonce: &'a [u8],
    payload: &'a [u8],
    /// Everything the signature covers.
    signed_part: &'a [u8],
    signature: Option<(&'a [u8], &'a [u8])>,
}

fn take<'a>(data: &'a [u8], pos: &mut usize, n: u64, what: &'static str) -> Result<&'a [u8]> {
    let left = (data.len() - *pos) as u64;
    if n > left {
        return Err(Error::Truncated { what });
    }
    let s = &data[*pos..*pos + n as usize];
    *pos += n as usize;
    Ok(s)
}

fn layout<'a>(data: &'a [u8], limits: &Limits) -> Result<Layout<'a>> {
    let mut pos = 0;
    if data.len() < 4 {
        // A file too short to tell is "not an .oop" unless what is there matches the magic.
        return Err(if MAGIC.starts_with(data) { Error::Truncated { what: "magic" } } else { Error::BadMagic });
    }
    if take(data, &mut pos, 4, "magic")? != MAGIC {
        return Err(Error::BadMagic);
    }
    let version = u16::from_le_bytes(take(data, &mut pos, 2, "format version")?.try_into().expect("2"));
    if version != FORMAT_VERSION {
        return Err(Error::UnsupportedVersion(version));
    }
    let flags = u16::from_le_bytes(take(data, &mut pos, 2, "flags")?.try_into().expect("2"));
    if flags & !FLAG_SIGNED != 0 {
        return Err(Error::UnknownFlags(flags & !FLAG_SIGNED));
    }
    let header_len = u32::from_le_bytes(take(data, &mut pos, 4, "header length")?.try_into().expect("4")) as u64;
    if header_len > limits.max_header_len {
        return Err(Error::HeaderTooLarge(header_len));
    }
    let header_bytes = take(data, &mut pos, header_len, "header")?;
    let aad = &data[..pos];
    let salt = take(data, &mut pos, SALT_LEN as u64, "salt")?;
    let nonce = take(data, &mut pos, NONCE_LEN as u64, "nonce")?;
    let payload_len = u64::from_le_bytes(take(data, &mut pos, 8, "payload length")?.try_into().expect("8"));
    if payload_len < TAG_LEN {
        return Err(Error::Truncated { what: "payload" });
    }
    let payload = take(data, &mut pos, payload_len, "payload")?;
    let signed_part = &data[..pos];
    let signature = if flags & FLAG_SIGNED != 0 {
        let sig = take(data, &mut pos, SIG_LEN as u64, "signature")?;
        let key = take(data, &mut pos, PUBKEY_LEN as u64, "signer's public key")?;
        Some((sig, key))
    } else {
        None
    };
    if pos != data.len() {
        return Err(Error::TrailingData((data.len() - pos) as u64));
    }
    Ok(Layout { flags, header_bytes, aad, salt, nonce, payload, signed_part, signature })
}

fn payload_cipher(salt: &[u8], header_bytes: &[u8]) -> XChaCha20Poly1305 {
    let mut info = Vec::with_capacity(HKDF_INFO_V1.len() + 32);
    info.extend_from_slice(HKDF_INFO_V1);
    info.extend_from_slice(&Sha256::digest(header_bytes));
    let hk = hkdf::Hkdf::<Sha256>::new(Some(salt), &FORMAT_KEY_V1);
    let mut key = [0u8; 32];
    hk.expand(&info, &mut key).expect("32 bytes is a valid HKDF-SHA256 length");
    let cipher = XChaCha20Poly1305::new(&Key::from(key));
    key.fill(0);
    cipher
}

fn verify_signature(signed_part: &[u8], sig: &[u8], key: &[u8]) -> Result<VerifyingKey> {
    let key: [u8; 32] = key.try_into().map_err(|_| Error::BadSignature)?;
    let key = VerifyingKey::from_bytes(&key).map_err(|_| Error::BadSignature)?;
    let sig = Signature::from_slice(sig).map_err(|_| Error::BadSignature)?;
    key.verify_strict(signed_part, &sig).map_err(|_| Error::BadSignature)?;
    Ok(key)
}

/// A decrypted, checked plugin.
#[derive(Debug, Clone)]
pub struct Oop {
    header: Header,
    header_json: String,
    files: Vec<(String, Vec<u8>)>,
    signer: Option<VerifyingKey>,
}

impl Oop {
    /// Reads a whole `.oop` file with the default [`Limits`]: checks the layout, the signature
    /// (when the file is signed), decrypts and authenticates the payload, decompresses it within
    /// the limits, and checks every path and that the header's `entry` is one of the files.
    pub fn read(bytes: &[u8]) -> Result<Oop> {
        Oop::read_with_limits(bytes, &Limits::default())
    }

    /// [`Oop::read`] with other limits.
    pub fn read_with_limits(bytes: &[u8], limits: &Limits) -> Result<Oop> {
        let l = layout(bytes, limits)?;
        let header = Header::from_json(l.header_bytes)?;
        header.validate()?;
        let signer = match l.signature {
            Some((sig, key)) => Some(verify_signature(l.signed_part, sig, key)?),
            None => None,
        };
        let cipher = payload_cipher(l.salt, l.header_bytes);
        let compressed = cipher
            .decrypt(&XNonce::try_from(l.nonce).expect("24 bytes"), Payload { msg: l.payload, aad: l.aad })
            .map_err(|_| Error::Decrypt)?;
        let archive = decompress(&compressed, limits.max_unpacked)?;
        let files = archive::decode(&archive, limits)?;
        if !files.iter().any(|(p, _)| *p == header.entry) {
            return Err(Error::MissingEntry(header.entry.clone()));
        }
        let header_json = String::from_utf8(l.header_bytes.to_vec()).expect("checked by Header::from_json");
        Ok(Oop { header, header_json, files, signer })
    }

    /// The header.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// The header exactly as it is stored in the file.
    pub fn header_json(&self) -> &str {
        &self.header_json
    }

    /// The files, sorted by path.
    pub fn files(&self) -> &[(String, Vec<u8>)] {
        &self.files
    }

    /// One file's contents.
    pub fn file(&self, path: &str) -> Option<&[u8]> {
        self.files.binary_search_by(|(p, _)| p.as_bytes().cmp(path.as_bytes())).ok().map(|i| self.files[i].1.as_slice())
    }

    /// The entry file's contents (`main.lua`, `plugin.wasm`).
    pub fn entry(&self) -> &[u8] {
        self.file(&self.header.entry).expect("checked by Oop::read")
    }

    /// The key that signed the file, `None` for an unsigned one.
    pub fn signer(&self) -> Option<&VerifyingKey> {
        self.signer.as_ref()
    }

    /// [`fingerprint`] of the signer.
    pub fn signer_fingerprint(&self) -> Option<String> {
        self.signer.as_ref().map(fingerprint)
    }

    /// Takes the files out.
    pub fn into_files(self) -> Vec<(String, Vec<u8>)> {
        self.files
    }
}

fn decompress(data: &[u8], max: u64) -> Result<Vec<u8>> {
    let mut source = data;
    let mut decoder =
        ruzstd::decoding::StreamingDecoder::new(&mut source).map_err(|e| Error::Decompress(e.to_string()))?;
    let mut out = Vec::new();
    // One byte more than allowed tells "exactly at the limit" from "over it".
    (&mut decoder).take(max.saturating_add(1)).read_to_end(&mut out).map_err(|e| Error::Decompress(e.to_string()))?;
    if out.len() as u64 > max {
        return Err(Error::TooLarge);
    }
    drop(decoder);
    if !source.is_empty() {
        return Err(Error::Decompress(format!("{} bytes after the zstd frame", source.len())));
    }
    Ok(out)
}

/// What [`inspect_details`] can tell without decrypting.
#[derive(Debug, Clone)]
pub struct Inspection {
    /// The parsed header.
    pub header: Header,
    /// The header exactly as stored.
    pub header_json: String,
    /// The format version (1).
    pub format_version: u16,
    /// The flag bits.
    pub flags: u16,
    /// The size of the encrypted payload in bytes (the tag included).
    pub payload_len: u64,
    /// The size of the whole file.
    pub file_len: u64,
    /// The signer's key, for a signed file.
    pub signer: Option<VerifyingKey>,
    /// For a signed file: whether the signature matches (it can be checked without the
    /// payload key). `None` for an unsigned file.
    pub signature_valid: Option<bool>,
}

/// Reads the header without decrypting anything (what `oopc inspect` and a plugin list show).
/// It checks the layout but not the payload: use [`Oop::read`] to know the file is intact.
pub fn inspect(bytes: &[u8]) -> Result<Header> {
    Ok(inspect_details(bytes)?.header)
}

/// [`inspect`] with everything else that is readable without the payload key, including
/// whether the signature matches.
pub fn inspect_details(bytes: &[u8]) -> Result<Inspection> {
    let l = layout(bytes, &Limits::default())?;
    let header = Header::from_json(l.header_bytes)?;
    let (signer, signature_valid) = match l.signature {
        Some((sig, key)) => match verify_signature(l.signed_part, sig, key) {
            Ok(k) => (Some(k), Some(true)),
            Err(_) => {
                let k = <[u8; 32]>::try_from(key).ok().and_then(|k| VerifyingKey::from_bytes(&k).ok());
                (k, Some(false))
            }
        },
        None => (None, None),
    };
    Ok(Inspection {
        header_json: String::from_utf8_lossy(l.header_bytes).into_owned(),
        header,
        format_version: FORMAT_VERSION,
        flags: l.flags,
        payload_len: l.payload.len() as u64,
        file_len: bytes.len() as u64,
        signer,
        signature_valid,
    })
}

/// Makes an `.oop` file.
#[derive(Debug, Clone)]
pub struct OopBuilder {
    header: Header,
    files: Vec<(String, Vec<u8>)>,
    key: Option<SigningKey>,
    compression: Compression,
    limits: Limits,
}

impl OopBuilder {
    /// Starts a plugin with this header.
    pub fn new(header: Header) -> OopBuilder {
        OopBuilder {
            header,
            files: Vec::new(),
            key: None,
            compression: Compression::default(),
            limits: Limits::default(),
        }
    }

    /// Adds a file. The order does not matter (the table is sorted); a path given twice is an
    /// error at [`OopBuilder::write`].
    pub fn file(mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> OopBuilder {
        self.files.push((path.into(), bytes.into()));
        self
    }

    /// Adds several files.
    pub fn files<P: Into<String>, B: Into<Vec<u8>>>(mut self, files: impl IntoIterator<Item = (P, B)>) -> OopBuilder {
        self.files.extend(files.into_iter().map(|(p, b)| (p.into(), b.into())));
        self
    }

    /// Signs the file with this key.
    pub fn sign(mut self, key: &SigningKey) -> OopBuilder {
        self.key = Some(key.clone());
        self
    }

    /// Chooses the compression.
    pub fn compression(mut self, compression: Compression) -> OopBuilder {
        self.compression = compression;
        self
    }

    /// Chooses the limits the files are checked against (default: the format's).
    pub fn limits(mut self, limits: Limits) -> OopBuilder {
        self.limits = limits;
        self
    }

    /// Writes the file with a random salt and nonce.
    pub fn write(self) -> Result<Vec<u8>> {
        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        getrandom::fill(&mut salt).map_err(|e| Error::Random(e.to_string()))?;
        getrandom::fill(&mut nonce).map_err(|e| Error::Random(e.to_string()))?;
        self.write_with(salt, nonce)
    }

    /// Writes the file with the given salt and nonce. Only for reproducible test vectors: a
    /// nonce must never be used twice with the same salt and header.
    pub fn write_with(self, salt: [u8; SALT_LEN], nonce: [u8; NONCE_LEN]) -> Result<Vec<u8>> {
        self.header.validate()?;
        let header_json = self.header.to_json();
        if header_json.len() as u64 > self.limits.max_header_len {
            return Err(Error::HeaderTooLarge(header_json.len() as u64));
        }
        let files = archive::prepare(self.files, &self.limits)?;
        if !files.iter().any(|(p, _)| *p == self.header.entry) {
            return Err(Error::MissingEntry(self.header.entry.clone()));
        }
        let table = archive::encode(&files);
        let level = match self.compression {
            Compression::Fastest => ruzstd::encoding::CompressionLevel::Fastest,
            Compression::Stored => ruzstd::encoding::CompressionLevel::Uncompressed,
        };
        let compressed = ruzstd::encoding::compress_to_vec(table.as_slice(), level);

        let flags = if self.key.is_some() { FLAG_SIGNED } else { 0 };
        let mut out = Vec::with_capacity(12 + header_json.len() + 64 + compressed.len() + 16 + 96);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&(header_json.len() as u32).to_le_bytes());
        out.extend_from_slice(header_json.as_bytes());
        let aad_len = out.len();

        let cipher = payload_cipher(&salt, header_json.as_bytes());
        let payload = cipher
            .encrypt(&XNonce::from(nonce), Payload { msg: &compressed, aad: &out[..aad_len] })
            .map_err(|_| Error::TooLarge)?;
        out.extend_from_slice(&salt);
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        out.extend_from_slice(&payload);
        if let Some(key) = &self.key {
            let sig = key.sign(&out);
            out.extend_from_slice(&sig.to_bytes());
            out.extend_from_slice(key.verifying_key().as_bytes());
        }
        Ok(out)
    }
}
