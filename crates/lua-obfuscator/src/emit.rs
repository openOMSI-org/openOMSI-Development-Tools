//! Turning the analysed AST back into source: comments and formatting are dropped (only the
//! significant tokens are emitted, each separated by a single space, which is always valid Lua),
//! locals are renamed, and the marked string and number literals are replaced by calls to the
//! generated decode helper and by arithmetic.

use full_moon::ast::Ast;
use full_moon::node::Node;
use full_moon::tokenizer::TokenType;

use crate::resolve::Plan;

/// The XOR key the string decoder uses. Any value 1..=255 works; the encoder and the helper must
/// agree. It is per build output, not a secret.
pub const STRING_KEY: u8 = 0x5B;

/// Emits one obfuscated Lua chunk. `helper` is the name of the string-decode function (see
/// [`crate::bundle`]); it is called for every encoded string.
pub fn emit(ast: &Ast, plan: &Plan, helper: &str) -> String {
    let mut out = String::new();
    let mut need_space = false;
    // `Node::tokens` yields tokens in the AST's structural field order, not in source order (for a
    // call the parentheses come before the arguments). Sort by byte offset to recover the text.
    let mut tokens: Vec<_> = ast.nodes().tokens().collect();
    tokens.sort_by_key(|t| t.token().start_position().bytes());
    for token_ref in tokens {
        let token = token_ref.token();
        let pos = token.start_position().bytes();
        let piece = match token.token_type() {
            TokenType::Eof => continue,
            _ if plan.renames.contains_key(&pos) => plan.renames[&pos].clone(),
            _ if plan.strings.contains_key(&pos) => encode_string(&plan.strings[&pos], helper),
            _ if plan.numbers.contains_key(&pos) => encode_number(plan.numbers[&pos]),
            _ => token.to_string(),
        };
        if piece.is_empty() {
            continue;
        }
        if need_space {
            out.push(' ');
        }
        out.push_str(&piece);
        need_space = true;
    }
    out
}

/// `H("\ddd\ddd...")`, the bytes XORed with [`STRING_KEY`]; the helper XORs them back.
fn encode_string(bytes: &[u8], helper: &str) -> String {
    let mut s = String::with_capacity(bytes.len() * 4 + helper.len() + 4);
    s.push_str(helper);
    s.push_str("(\"");
    for &b in bytes {
        // Decimal escapes work for every byte and never run into a following digit ambiguity
        // because they are always three digits here.
        s.push_str(&format!("\\{:03}", b ^ STRING_KEY));
    }
    s.push_str("\")");
    s
}

/// A plain integer as `(a+b)` in hexadecimal, which keeps Lua's integer subtype.
fn encode_number(v: u64) -> String {
    let split = v % 0x2B;
    format!("(0x{:x}+0x{:x})", v - split, split)
}
