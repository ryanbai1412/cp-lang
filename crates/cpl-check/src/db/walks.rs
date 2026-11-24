//! Which `for` loops may walk a std type by an impl std.seq declares.

use cpl_syntax::{Token, TokenKind};

/// Whether some `for` in `tokens` walks something other than a range
/// written in its header (`for i in 0..n`). The language walks a range
/// itself; anything else may be a std type (`Prefix`) whose `Seq` impl
/// lives in std.seq, so the file then loads std.seq.
pub(super) fn some_walk_needs_seq(tokens: &[Token]) -> bool {
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind == TokenKind::For {
            match loop_in(tokens, i + 1) {
                Some(at) if !walks_a_range(tokens, at + 1) => return true,
                Some(at) => i = at,
                None => {}
            }
        }
        i += 1;
    }
    false
}

/// True when the tokens slice a sequence, `xs[a..b]`: a slice is std's
/// `Slice<S>`, whose module the lowering reads the asserts of its methods from.
pub(super) fn some_slice_needs_seq(tokens: &[Token]) -> bool {
    // One bit per open bracket, innermost lowest: set for `xs[..]`, which
    // indexes; `[0..3]T` and `[a..b]` after anything else are a type or a list.
    let (mut open, mut prev) = (0u64, None);
    for tok in tokens {
        let after = prev.replace(tok.kind);
        match tok.kind {
            TokenKind::LBracket => {
                let index = matches!(
                    after,
                    Some(TokenKind::Ident | TokenKind::RParen | TokenKind::RBracket)
                );
                open = open << 1 | u64::from(index);
            }
            TokenKind::LParen | TokenKind::LBrace => open <<= 1,
            TokenKind::RBracket | TokenKind::RParen | TokenKind::RBrace => open >>= 1,
            TokenKind::DotDot | TokenKind::DotDotEq if open & 1 == 1 => return true,
            _ => {}
        }
    }
    false
}

/// The `in` that heads the loop whose `for` comes just before `from`.
fn loop_in(tokens: &[Token], from: usize) -> Option<usize> {
    let mut depth = 0i32;
    for (at, tok) in tokens.iter().enumerate().skip(from) {
        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
            TokenKind::In if depth == 0 => return Some(at),
            TokenKind::Colon | TokenKind::Newline | TokenKind::Eof if depth == 0 => return None,
            _ => {}
        }
    }
    None
}

/// Whether the loop iterable starting at `from` is a range at its top level.
fn walks_a_range(tokens: &[Token], from: usize) -> bool {
    let mut depth = 0i32;
    for tok in &tokens[from.min(tokens.len())..] {
        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace if depth == 0 => break,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
            TokenKind::DotDot | TokenKind::DotDotEq if depth == 0 => return true,
            TokenKind::Colon | TokenKind::Newline | TokenKind::If | TokenKind::Eof
                if depth == 0 =>
            {
                break
            }
            _ => {}
        }
    }
    false
}
