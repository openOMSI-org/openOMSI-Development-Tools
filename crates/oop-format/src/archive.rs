//! The file table inside the payload: `u32 count`, then per file `u16 path_len`, path,
//! `u64 len`, bytes; sorted by path (byte order), every path once.

use crate::Limits;
use crate::error::{Error, Result};

/// Checks that `path` may be stored in a plugin: UTF-8, `/` separators, relative, no `.` or
/// `..` parts, no empty parts, no backslashes, no drive letters, no control characters, at most
/// 1024 bytes. These rules make sure the game's virtual folder can never point outside itself,
/// whatever the reader does with the names later.
pub fn check_path(path: &str) -> Result<()> {
    let bad = || Err(Error::BadPath(path.to_string()));
    if path.is_empty() || path.len() > 1024 {
        return bad();
    }
    if path.starts_with('/') || path.contains('\\') || path.contains(':') {
        return bad();
    }
    if path.chars().any(|c| c.is_control()) {
        return bad();
    }
    for part in path.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return bad();
        }
    }
    Ok(())
}

/// Serialises the files (already checked, sorted and unique) into the table.
pub(crate) fn encode(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let total: usize = files.iter().map(|(p, b)| 2 + p.len() + 8 + b.len()).sum();
    let mut out = Vec::with_capacity(4 + total);
    out.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (path, bytes) in files {
        out.extend_from_slice(&(path.len() as u16).to_le_bytes());
        out.extend_from_slice(path.as_bytes());
        out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}

/// Sorts the files, checks every path, and rejects duplicates and oversized sets.
pub(crate) fn prepare(mut files: Vec<(String, Vec<u8>)>, limits: &Limits) -> Result<Vec<(String, Vec<u8>)>> {
    if files.len() as u64 > limits.max_files {
        return Err(Error::TooManyFiles(files.len() as u64));
    }
    files.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut total: u64 = 4;
    for (i, (path, bytes)) in files.iter().enumerate() {
        check_path(path)?;
        if i > 0 && files[i - 1].0 == *path {
            return Err(Error::DuplicatePath(path.clone()));
        }
        total += 2 + path.len() as u64 + 8 + bytes.len() as u64;
    }
    if total > limits.max_unpacked {
        return Err(Error::TooLarge);
    }
    Ok(files)
}

/// A cursor that never reads past the end and says which field was cut off.
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: u64, what: &'static str) -> Result<&'a [u8]> {
        let left = (self.data.len() - self.pos) as u64;
        if n > left {
            return Err(Error::Archive(format!("it ends inside the {what}")));
        }
        let s = &self.data[self.pos..self.pos + n as usize];
        self.pos += n as usize;
        Ok(s)
    }
    fn u16(&mut self, what: &'static str) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2, what)?.try_into().expect("2 bytes")))
    }
    fn u32(&mut self, what: &'static str) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4, what)?.try_into().expect("4 bytes")))
    }
    fn u64(&mut self, what: &'static str) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8, what)?.try_into().expect("8 bytes")))
    }
}

/// Parses the table. Every count and length is checked against what is left before anything
/// is allocated, so a hostile table cannot ask for more memory than the (already size-limited)
/// decompressed bytes it came in.
pub(crate) fn decode(data: &[u8], limits: &Limits) -> Result<Vec<(String, Vec<u8>)>> {
    let mut c = Cursor { data, pos: 0 };
    let count = c.u32("file count")? as u64;
    if count > limits.max_files {
        return Err(Error::TooManyFiles(count));
    }
    // Each entry takes at least 2 + 1 + 8 bytes.
    if count * 11 > (data.len() as u64).saturating_sub(4) {
        return Err(Error::Archive(format!("{count} files cannot fit into {} bytes", data.len())));
    }
    let mut files: Vec<(String, Vec<u8>)> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let plen = c.u16("path length")? as u64;
        let path = std::str::from_utf8(c.take(plen, "path")?)
            .map_err(|_| Error::Archive("a path is not UTF-8".into()))?
            .to_string();
        check_path(&path)?;
        if let Some((last, _)) = files.last()
            && last.as_bytes() >= path.as_bytes()
        {
            return Err(Error::DuplicatePath(path));
        }
        let len = c.u64("file length")?;
        let bytes = c.take(len, "file contents")?.to_vec();
        files.push((path, bytes));
    }
    if c.pos != data.len() {
        return Err(Error::Archive(format!("{} bytes after the last file", data.len() - c.pos)));
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths() {
        for ok in ["main.lua", "lib/util.lua", "assets/icons/a b.png", "ü/ñ.txt", "a..b/c"] {
            assert!(check_path(ok).is_ok(), "{ok}");
        }
        for bad in [
            "",
            "/etc/passwd",
            "../x",
            "a/../../x",
            "a/./b",
            "a//b",
            "a/",
            "C:/x",
            "c:x",
            "a\\b",
            "..",
            ".",
            "a\0b",
            "a\nb",
        ] {
            assert!(check_path(bad).is_err(), "{bad:?}");
        }
        assert!(check_path(&"a".repeat(1025)).is_err());
    }

    #[test]
    fn round_trip_and_order() {
        let files = vec![("b.lua".to_string(), b"2".to_vec()), ("a.lua".to_string(), b"1".to_vec())];
        let files = prepare(files, &Limits::default()).unwrap();
        assert_eq!(files[0].0, "a.lua");
        let enc = encode(&files);
        assert_eq!(decode(&enc, &Limits::default()).unwrap(), files);
    }

    #[test]
    fn rejects_unsorted_and_duplicates() {
        let unsorted = vec![("b".to_string(), vec![]), ("a".to_string(), vec![])];
        let enc = encode(&unsorted);
        assert!(matches!(decode(&enc, &Limits::default()), Err(Error::DuplicatePath(_))));
        let dup = vec![("a".to_string(), vec![]), ("a".to_string(), vec![])];
        assert!(matches!(prepare(dup, &Limits::default()), Err(Error::DuplicatePath(_))));
    }

    #[test]
    fn rejects_huge_counts_without_allocating() {
        let mut enc = u32::MAX.to_le_bytes().to_vec();
        enc.extend_from_slice(&[0; 16]);
        assert!(matches!(decode(&enc, &Limits::default()), Err(Error::TooManyFiles(_))));
        let mut enc = 9999u32.to_le_bytes().to_vec();
        enc.extend_from_slice(&[0; 16]);
        assert!(matches!(decode(&enc, &Limits::default()), Err(Error::Archive(_))));
        // One file claiming 2^63 bytes.
        let mut enc = 1u32.to_le_bytes().to_vec();
        enc.extend_from_slice(&1u16.to_le_bytes());
        enc.push(b'a');
        enc.extend_from_slice(&(1u64 << 63).to_le_bytes());
        assert!(matches!(decode(&enc, &Limits::default()), Err(Error::Archive(_))));
    }
}
