//! The errors of reading and writing `.oop` files.

use std::fmt;

/// Everything that can go wrong with an `.oop` file.
///
/// The messages are meant for people: the game puts them into `game.log` and `oopc` prints them,
/// so they say what is wrong in words, not in offsets alone.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The file ends before a field it must have (`what` names the field).
    Truncated {
        /// The field that is cut off.
        what: &'static str,
    },
    /// The first four bytes are not `OOP\x01`: not an openOMSI plugin at all.
    BadMagic,
    /// A format version this crate does not know (a newer devtools made it).
    UnsupportedVersion(u16),
    /// Flag bits this crate does not know are set.
    UnknownFlags(u16),
    /// The header is larger than [`crate::Limits::max_header_len`].
    HeaderTooLarge(u64),
    /// The header is not valid UTF-8 JSON of the expected shape.
    HeaderJson(String),
    /// The header parsed but one of its fields is not acceptable.
    InvalidHeader(String),
    /// The payload does not decrypt: the file was changed after it was written (or it is
    /// corrupt). The AEAD tag covers the header too, so this is also what an edited header gives.
    Decrypt,
    /// The decrypted payload is not a valid zstd frame.
    Decompress(String),
    /// The file table inside the payload is malformed.
    Archive(String),
    /// A file path inside the archive is not allowed (absolute, `..`, backslashes, ...).
    BadPath(String),
    /// Two files have the same path, or the table is not sorted.
    DuplicatePath(String),
    /// More files than [`crate::Limits::max_files`].
    TooManyFiles(u64),
    /// The unpacked archive would be larger than [`crate::Limits::max_unpacked`].
    TooLarge,
    /// The header's `entry` is not one of the archive's files.
    MissingEntry(String),
    /// The file is flagged as signed but the Ed25519 signature does not match.
    BadSignature,
    /// Bytes after the end of the last field.
    TrailingData(u64),
    /// No randomness for the salt and the nonce (only on exotic targets).
    Random(String),
    /// A key file could not be parsed.
    Key(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated { what } => write!(f, "the file is truncated: it ends inside the {what}"),
            Error::BadMagic => f.write_str("not an openOMSI plugin (.oop) file: wrong magic bytes"),
            Error::UnsupportedVersion(v) => {
                write!(f, "the .oop format version {v} is newer than this reader understands (it knows 1)")
            }
            Error::UnknownFlags(fl) => write!(f, "unknown flags 0x{fl:04x} are set"),
            Error::HeaderTooLarge(n) => write!(f, "the header is {n} bytes, more than allowed"),
            Error::HeaderJson(e) => write!(f, "the header is not valid JSON: {e}"),
            Error::InvalidHeader(e) => write!(f, "invalid header: {e}"),
            Error::Decrypt => f.write_str(
                "the plugin's contents do not decrypt: the file was changed after it was built, or it is damaged",
            ),
            Error::Decompress(e) => write!(f, "the plugin's contents do not decompress: {e}"),
            Error::Archive(e) => write!(f, "the file table is malformed: {e}"),
            Error::BadPath(p) => write!(f, "file path not allowed in a plugin: {p:?}"),
            Error::DuplicatePath(p) => write!(f, "duplicate or unsorted file path: {p:?}"),
            Error::TooManyFiles(n) => write!(f, "{n} files: more than a plugin may have"),
            Error::TooLarge => f.write_str("the unpacked plugin would be larger than allowed"),
            Error::MissingEntry(e) => write!(f, "the entry file {e:?} is not in the plugin"),
            Error::BadSignature => {
                f.write_str("the signature does not match: the file was changed after it was signed")
            }
            Error::TrailingData(n) => write!(f, "{n} unexpected bytes after the end of the plugin"),
            Error::Random(e) => write!(f, "no random numbers available: {e}"),
            Error::Key(e) => write!(f, "invalid key: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// `Result` with this crate's [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
