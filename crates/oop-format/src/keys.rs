//! Ed25519 signing keys: making them, their fingerprints and their text files.
//!
//! A key file is a few lines of text; lines starting with `#` are comments:
//!
//! ```text
//! # openOMSI plugin signing key "Jane's key" - keep this file secret
//! openomsi-secret-key-v1 3q2+7w...base64 of the 32-byte seed...=
//! ```
//!
//! and a public key file the same with `openomsi-public-key-v1` and the 32-byte public key.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

pub use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::error::{Error, Result};

const SECRET_TAG: &str = "openomsi-secret-key-v1";
const PUBLIC_TAG: &str = "openomsi-public-key-v1";

/// Makes a new random signing key.
pub fn generate_key() -> Result<SigningKey> {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).map_err(|e| Error::Random(e.to_string()))?;
    let key = SigningKey::from_bytes(&seed);
    seed.fill(0);
    Ok(key)
}

/// The fingerprint people compare: the first 8 bytes of SHA-256 of the 32-byte public key, as
/// four groups of hex digits, `3f2a:9c01:77be:12d4`. The game shows the same string ("signed
/// by ...").
pub fn fingerprint(key: &VerifyingKey) -> String {
    let digest = Sha256::digest(key.as_bytes());
    digest[..8].chunks(2).map(|c| format!("{:02x}{:02x}", c[0], c[1])).collect::<Vec<_>>().join(":")
}

/// The secret key as the text of a key file. `label` goes into the comment line.
pub fn encode_secret_key(key: &SigningKey, label: &str) -> String {
    let label = label.replace(['\n', '\r'], " ");
    format!(
        "# openOMSI plugin signing key \"{label}\" - keep this file secret\n# fingerprint {}\n{SECRET_TAG} {}\n",
        fingerprint(&key.verifying_key()),
        STANDARD.encode(key.to_bytes())
    )
}

/// The public key as the text of a public key file (safe to publish).
pub fn encode_public_key(key: &VerifyingKey, label: &str) -> String {
    let label = label.replace(['\n', '\r'], " ");
    format!(
        "# openOMSI plugin signing key \"{label}\" (public)\n# fingerprint {}\n{PUBLIC_TAG} {}\n",
        fingerprint(key),
        STANDARD.encode(key.as_bytes())
    )
}

fn find_value<'a>(text: &'a str, tag: &str) -> Result<&'a str> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix(tag) {
            return Ok(rest.trim());
        }
        if line.starts_with(SECRET_TAG) || line.starts_with(PUBLIC_TAG) {
            let wanted = if tag == SECRET_TAG { "a secret" } else { "a public" };
            return Err(Error::Key(format!("this is not {wanted} key file")));
        }
    }
    Err(Error::Key(format!("no `{tag}` line found")))
}

fn decode_32(b64: &str) -> Result<[u8; 32]> {
    let bytes = STANDARD.decode(b64).map_err(|e| Error::Key(format!("bad base64: {e}")))?;
    bytes.try_into().map_err(|_| Error::Key("the key must be 32 bytes".into()))
}

/// Reads a secret key file's text.
pub fn decode_secret_key(text: &str) -> Result<SigningKey> {
    let mut seed = decode_32(find_value(text, SECRET_TAG)?)?;
    let key = SigningKey::from_bytes(&seed);
    seed.fill(0);
    Ok(key)
}

/// Reads a public key file's text (a secret key file works too: its public half is returned).
pub fn decode_public_key(text: &str) -> Result<VerifyingKey> {
    if let Ok(secret) = decode_secret_key(text) {
        return Ok(secret.verifying_key());
    }
    let bytes = decode_32(find_value(text, PUBLIC_TAG)?)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| Error::Key("not a valid Ed25519 public key".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_files_round_trip() {
        let key = generate_key().unwrap();
        let text = encode_secret_key(&key, "test\nkey");
        assert!(text.contains("\"test key\""));
        let back = decode_secret_key(&text).unwrap();
        assert_eq!(back.to_bytes(), key.to_bytes());
        let pubtext = encode_public_key(&key.verifying_key(), "test");
        assert_eq!(decode_public_key(&pubtext).unwrap(), key.verifying_key());
        assert_eq!(decode_public_key(&text).unwrap(), key.verifying_key());
        assert!(decode_secret_key(&pubtext).is_err());
        assert!(decode_secret_key("openomsi-secret-key-v1 AAAA").is_err());
        assert!(decode_secret_key("hello").is_err());
    }

    #[test]
    fn fingerprint_shape() {
        let key = SigningKey::from_bytes(&[7; 32]);
        let fp = fingerprint(&key.verifying_key());
        assert_eq!(fp.len(), 19);
        assert_eq!(fp.matches(':').count(), 3);
    }
}
