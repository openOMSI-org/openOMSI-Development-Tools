//! Turning the text of a Lua short-string literal (what is between the quotes) into the bytes it
//! stands for, so a string can be re-encoded. Lua 5.4 escape rules.

/// Decodes the body of a `'...'` or `"..."` literal into its bytes, or `None` for an escape this
/// does not understand (then the caller leaves the string as it is).
pub fn unescape(body: &str) -> Option<Vec<u8>> {
    let b = body.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'\\' {
            out.push(b[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= b.len() {
            return None;
        }
        match b[i] {
            b'a' => out.push(7),
            b'b' => out.push(8),
            b'f' => out.push(12),
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b't' => out.push(b'\t'),
            b'v' => out.push(11),
            b'\\' => out.push(b'\\'),
            b'"' => out.push(b'"'),
            b'\'' => out.push(b'\''),
            b'\n' => out.push(b'\n'),
            b'\r' => {
                out.push(b'\n');
                if i + 1 < b.len() && b[i + 1] == b'\n' {
                    i += 1;
                }
            }
            b'x' => {
                let hex = b.get(i + 1..i + 3)?;
                out.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                i += 2;
            }
            b'z' => {
                while i + 1 < b.len() && b[i + 1].is_ascii_whitespace() {
                    i += 1;
                }
            }
            b'u' => {
                if b.get(i + 1) != Some(&b'{') {
                    return None;
                }
                let end = body[i + 2..].find('}')? + i + 2;
                let cp = u32::from_str_radix(&body[i + 2..end], 16).ok()?;
                // Lua allows up to 2^31; it emits the UTF-8 (extended for > U+10FFFF) encoding.
                push_utf8(&mut out, cp)?;
                i = end;
            }
            d if d.is_ascii_digit() => {
                let mut n = 0u32;
                let mut k = 0;
                while k < 3 && i < b.len() && b[i].is_ascii_digit() {
                    n = n * 10 + (b[i] - b'0') as u32;
                    i += 1;
                    k += 1;
                }
                i -= 1;
                if n > 255 {
                    return None;
                }
                out.push(n as u8);
            }
            _ => return None,
        }
        i += 1;
    }
    Some(out)
}

/// The UTF-8 bytes of a code point, with Lua's extension up to 6 bytes for values beyond U+10FFFF.
fn push_utf8(out: &mut Vec<u8>, cp: u32) -> Option<()> {
    match cp {
        0..=0x7F => out.push(cp as u8),
        0x80..=0x7FF => {
            out.push(0xC0 | (cp >> 6) as u8);
            out.push(0x80 | (cp & 0x3F) as u8);
        }
        0x800..=0xFFFF => {
            out.push(0xE0 | (cp >> 12) as u8);
            out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
            out.push(0x80 | (cp & 0x3F) as u8);
        }
        0x1_0000..=0x1F_FFFF => {
            out.push(0xF0 | (cp >> 18) as u8);
            out.push(0x80 | ((cp >> 12) & 0x3F) as u8);
            out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
            out.push(0x80 | (cp & 0x3F) as u8);
        }
        0x20_0000..=0x7FFF_FFFF => {
            out.push(0xF8 | (cp >> 24) as u8);
            out.push(0x80 | ((cp >> 18) & 0x3F) as u8);
            out.push(0x80 | ((cp >> 12) & 0x3F) as u8);
            out.push(0x80 | ((cp >> 6) & 0x3F) as u8);
            out.push(0x80 | (cp & 0x3F) as u8);
        }
        _ => return None,
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes() {
        assert_eq!(unescape("abc").unwrap(), b"abc");
        assert_eq!(unescape(r"a\tb\n").unwrap(), b"a\tb\n");
        assert_eq!(unescape(r"\65\66\67").unwrap(), b"ABC");
        assert_eq!(unescape(r"\x41\x42").unwrap(), b"AB");
        assert_eq!(unescape(r"\u{48}\u{49}").unwrap(), b"HI");
        assert_eq!(unescape(r"\u{20AC}").unwrap(), "€".as_bytes());
        assert_eq!(unescape("a\\z   \n  b").unwrap(), b"ab");
        assert_eq!(unescape(r#"quote\"end"#).unwrap(), b"quote\"end");
        assert_eq!(unescape(r"\0").unwrap(), b"\0");
    }
}
