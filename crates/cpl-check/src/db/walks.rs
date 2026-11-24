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

/// The `in` that heads the loop whose `for` comes just before `from`.
fn loop_in(tokens: &[Token], from: usize) -> Option<usize> {
    use TokenKind::*;
    top_level(tokens, from, &[In], &[Colon, Newline, Eof])
}

/// Whether the loop iterable starting at `from` is a range at its top level.
fn walks_a_range(tokens: &[Token], from: usize) -> bool {
    use TokenKind::*;
    top_level(
        tokens,
        from,
        &[DotDot, DotDotEq],
        &[Colon, Newline, If, Eof],
    )
    .is_some()
}

/// The first token from `from` that is one of `find`, outside brackets,
/// before a `stop` or the bracket around `from` closes.
fn top_level(
    tokens: &[Token],
    from: usize,
    find: &[TokenKind],
    stop: &[TokenKind],
) -> Option<usize> {
    let mut depth = 0i32;
    for (at, tok) in tokens.iter().enumerate().skip(from) {
        match tok.kind {
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => depth += 1,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace if depth == 0 => break,
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => depth -= 1,
            k if depth == 0 && find.contains(&k) => return Some(at),
            k if depth == 0 && stop.contains(&k) => break,
            _ => {}
        }
    }
    None
}

/// Whether `tokens` index a value (`xs[..]`): a range index gives std's
/// `Slice` (R07.4.17), so the file then loads it as if it named it.
pub(super) fn some_index(tokens: &[Token]) -> bool {
    tokens.windows(2).any(|w| {
        w[1].kind == TokenKind::LBracket
            && matches!(
                w[0].kind,
                TokenKind::Ident | TokenKind::RParen | TokenKind::RBracket | TokenKind::Str
            )
    })
}
