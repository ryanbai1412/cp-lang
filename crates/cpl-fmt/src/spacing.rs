//! Where spaces go between two tokens on one line.
//!
//! The formatter never parses: it decides from the two tokens, what the
//! tokens before them were, and, for the few pairs that read both ways
//! without a parse (`[V]Int` against `[I] i => 0`), from whether the source
//! had a space there.

use crate::Tok;
use cpl_syntax::TokenKind as K;

/// What a token is doing, where its kind alone does not say.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Role {
    Plain,
    /// `-x`, `~x`, `?Int`: binds to what follows.
    Prefix,
    /// `x?` where the source wrote it tight against a value.
    Postfix,
    /// The `<` of `List<Int>`.
    GenericOpen,
    /// The `>` or `>>` that closes it.
    GenericClose,
    /// The `{` of `use std.seq.{a, b}`, written tight.
    ImportOpen,
    /// The `}` that closes it.
    ImportClose,
    /// The second half of a C spelling the parser explains only when its
    /// halves touch (`&&`, `||`, `..<`): kept tight to the first half.
    Glued,
    /// `without` between a row and the fields it drops: `..R without (u)`.
    WordOperator,
}

fn ends_value(kind: K, role: Role) -> bool {
    match kind {
        K::Ident => role != Role::WordOperator,
        K::Int
        | K::Float
        | K::Str
        | K::RawStr
        | K::Char
        | K::RParen
        | K::RBracket
        | K::RBrace
        | K::SelfKw
        | K::True
        | K::False
        | K::None => true,
        K::Bang => role != Role::Prefix,
        K::Gt | K::Shr => role == Role::GenericClose,
        K::Question => role == Role::Postfix,
        _ => false,
    }
}

fn is_binary(kind: K) -> bool {
    matches!(
        kind,
        K::Arrow
            | K::FatArrow
            | K::Eq
            | K::OpEq
            | K::EqEq
            | K::NotEq
            | K::Lt
            | K::LtEq
            | K::Gt
            | K::GtEq
            | K::Spaceship
            | K::QuestionQuestion
            | K::Plus
            | K::Minus
            | K::Star
            | K::StarStar
            | K::Slash
            | K::Percent
            | K::Caret
            | K::Amp
            | K::Pipe
            | K::Shl
            | K::Shr
    )
}

fn starts_upper(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_uppercase())
}

/// Works out the role of every token of one logical line.
pub(crate) fn roles(toks: &[Tok]) -> Vec<Role> {
    let mut roles: Vec<Role> = Vec::with_capacity(toks.len());
    let mut angle = 0u32;
    // For each open `[`, whether it indexes a value (`a[i]`) or stands on its
    // own (`[V]Int`, `[1, 2]`); and whether the last `]` closed the second.
    let mut brackets: Vec<bool> = Vec::new();
    let mut closed_standalone = false;
    // For each open `{`, whether it lists the names of an import path.
    let mut braces: Vec<bool> = Vec::new();
    for (i, tok) in toks.iter().enumerate() {
        let prev = i.checked_sub(1).map(|p| (toks[p].kind, roles[p]));
        let after_value = prev.is_some_and(|(kind, role)| ends_value(kind, role));
        let after_standalone =
            closed_standalone && prev.is_some_and(|(kind, _)| kind == K::RBracket);
        match tok.kind {
            K::LBracket => brackets.push(after_value),
            K::RBracket => closed_standalone = brackets.pop() == Some(false),
            _ => {}
        }
        let role = match tok.kind {
            K::LBrace => {
                let imports = prev.is_some_and(|(kind, _)| kind == K::Dot);
                braces.push(imports);
                if imports {
                    Role::ImportOpen
                } else {
                    Role::Plain
                }
            }
            K::RBrace if braces.pop() == Some(true) => Role::ImportClose,
            // `[I]-100..=100`: the element type starts with a sign.
            K::Minus if after_standalone && toks.get(i + 1).is_some_and(|n| !n.space_before) => {
                Role::Prefix
            }
            K::Minus
                if i >= 2
                    && toks[i - 1].kind == K::Minus
                    && toks[i - 2].kind == K::Minus
                    && toks.get(i + 1).is_some_and(|n| n.space_before) =>
            {
                Role::Plain
            }
            K::Amp | K::Pipe
                if !tok.space_before && prev.is_some_and(|(kind, _)| kind == tok.kind) =>
            {
                Role::Glued
            }
            K::Lt if !tok.space_before && prev.is_some_and(|(kind, _)| kind == K::DotDot) => {
                Role::Glued
            }
            K::Ident if tok.text == "without" && after_value => Role::WordOperator,
            K::Minus | K::Plus | K::Amp if !after_value => Role::Prefix,
            K::Tilde => Role::Prefix,
            // `@!multitest`, and C's `!x`, which the parser explains.
            K::Bang if !after_value => Role::Prefix,
            K::Question if !after_value || tok.space_before => Role::Prefix,
            K::Question => Role::Postfix,
            K::Lt => {
                let tight_after = toks.get(i + 1).is_some_and(|next| !next.space_before);
                let declares = i >= 2 && toks[i - 1].kind == K::Ident && toks[i - 2].kind == K::Fn
                    || i >= 1 && toks[i - 1].kind == K::Impl;
                let generic_call = i > 0
                    && toks[i - 1].kind == K::Ident
                    && !tok.space_before
                    && tight_after
                    && closes_before_lparen(toks, i);
                let names_type = declares
                    || generic_call
                    || i > 0
                        && (toks[i - 1].kind == K::ColonColon
                            || (toks[i - 1].kind == K::Ident && starts_upper(toks[i - 1].text)));
                if names_type && !tok.space_before && tight_after {
                    angle += 1;
                    Role::GenericOpen
                } else {
                    Role::Plain
                }
            }
            K::Gt if angle >= 1 && !tok.space_before => {
                angle -= 1;
                Role::GenericClose
            }
            K::Shr if angle >= 2 && !tok.space_before => {
                angle -= 2;
                Role::GenericClose
            }
            _ => Role::Plain,
        };
        roles.push(role);
    }
    roles
}

/// Whether a space goes between `prev` and `next`.
pub(crate) fn space_between(prev: &Tok, prev_role: Role, next: &Tok, next_role: Role) -> bool {
    let wanted = wanted_space(prev, prev_role, next, next_role);
    // Two tokens written tight must not read as one (`? ?` is not `??`).
    wanted || fuses(prev, next)
}

fn wanted_space(prev: &Tok, prev_role: Role, next: &Tok, next_role: Role) -> bool {
    let after_value = ends_value(prev.kind, prev_role);
    if next_role == Role::Glued {
        return false;
    }
    if prev_role == Role::Glued {
        return prev.kind != K::Lt || next.space_before;
    }
    if prev_role == Role::ImportOpen || next_role == Role::ImportClose {
        return false;
    }
    match next.kind {
        K::Comma | K::Semi | K::RParen | K::RBracket | K::Colon => return false,
        K::RBrace => return prev.kind != K::LBrace,
        _ => {}
    }
    match prev.kind {
        K::LParen | K::LBracket | K::At => return false,
        K::LBrace => return true,
        _ => {}
    }
    if matches!(prev.kind, K::Dot | K::ColonColon) || matches!(next.kind, K::Dot | K::ColonColon) {
        return false;
    }
    if prev.kind == K::Minus && next.kind == K::Minus && !next.space_before {
        return false;
    }
    if next.kind == K::At || matches!(prev.kind, K::Comma | K::Semi | K::Colon) {
        return true;
    }
    if matches!(next.kind, K::DotDot | K::DotDotEq) {
        return !after_value && prev.kind != K::Lt;
    }
    if matches!(prev.kind, K::DotDot | K::DotDotEq) {
        return is_word_operator(next.kind);
    }

    match next.kind {
        // `[Spare]?Int` and `x ?? y` aside, a `?` after a value is written
        // the way the source had it.
        K::Question if after_value => return next.space_before,
        K::Bang if next_role != Role::Prefix => return false,
        // `-> yields (v: V, d: Int)` names a type; `yields(x)` would read as a call.
        K::LParen if prev.kind == K::Ident && prev.text == "yields" => return true,
        K::LParen | K::LBracket if after_value => {
            // `[I] (i, j) => ..` is a table, `fs[i](x)` a call.
            return prev.kind == K::RBracket && next.kind == K::LParen && next.space_before;
        }
        // `-> lends(a) T` names the parameters it borrows from.
        K::LParen if prev.kind == K::Lends => return false,
        // `fn(Int) -> Int` as a type keeps the shape it was written in.
        K::LParen if prev.kind == K::Fn => return next.space_before,
        // `best max= x`.
        K::Eq
            if prev.kind == K::Ident
                && matches!(prev.text, "max" | "min")
                && !next.space_before
                && prev.space_before =>
        {
            return false
        }
        _ => {}
    }
    if matches!(next_role, Role::GenericOpen | Role::GenericClose) {
        return false;
    }
    if next_role == Role::Plain && is_binary(next.kind) {
        return true;
    }

    match prev_role {
        Role::Prefix | Role::GenericOpen => return false,
        Role::Postfix => return next.space_before,
        Role::GenericClose => return true,
        Role::Plain | Role::ImportOpen | Role::ImportClose | Role::Glued | Role::WordOperator => {}
    }
    if is_binary(prev.kind) {
        return true;
    }
    // `[V]Int` is a type and `[I] i => i.idx` a table: only the source knows.
    if prev.kind == K::RBracket && starts_value(next.kind) {
        return next.space_before;
    }
    true
}

fn starts_value(kind: K) -> bool {
    matches!(
        kind,
        K::Ident
            | K::Int
            | K::Float
            | K::Str
            | K::RawStr
            | K::Char
            | K::SelfKw
            | K::True
            | K::False
            | K::None
            | K::Question
            | K::Minus
    )
}

/// Keywords that sit between two values, as `by` in `0..n by 2`.
fn is_word_operator(kind: K) -> bool {
    matches!(
        kind,
        K::And | K::Or | K::In | K::If | K::Else | K::For | K::Not | K::Lends
    )
}

/// Whether the two tokens, written with nothing between them, lex as
/// something other than themselves.
fn fuses(prev: &Tok, next: &Tok) -> bool {
    let single = |kind: K| {
        matches!(
            kind,
            K::LParen
                | K::RParen
                | K::LBracket
                | K::RBracket
                | K::LBrace
                | K::RBrace
                | K::Comma
                | K::Semi
        )
    };
    if single(prev.kind) || single(next.kind) {
        return false;
    }
    // `t.0`: alone, `.0` lexes as a malformed float, but after a value it
    // is a field.
    if prev.kind == K::Dot && next.kind == K::Int {
        return false;
    }
    let joined = format!("{}{}", prev.text, next.text);
    let lexed = cpl_syntax::lex(&joined);
    let kinds: Vec<K> = lexed
        .tokens
        .iter()
        .map(|t| t.kind)
        .filter(|k| !matches!(k, K::Newline | K::Eof))
        .collect();
    let split = lexed.tokens.first().map(|t| t.span.end as usize);
    kinds != [prev.kind, next.kind] || split != Some(prev.text.len())
}

/// True when `<` at `open` closes with a matching `>` followed immediately by `(`.
fn closes_before_lparen(toks: &[Tok], open: usize) -> bool {
    let mut angle = 0usize;
    let mut group = 0usize;
    for (j, t) in toks.iter().enumerate().skip(open) {
        match t.kind {
            K::LParen | K::LBracket | K::LBrace => group += 1,
            K::RParen | K::RBracket | K::RBrace => {
                if group == 0 {
                    return false;
                }
                group -= 1;
            }
            K::Lt if group == 0 => angle += 1,
            K::Gt if group == 0 => {
                angle = angle.saturating_sub(1);
                if angle == 0 {
                    return toks.get(j + 1).is_some_and(|n| n.kind == K::LParen);
                }
            }
            K::Shr if group == 0 => {
                angle = angle.saturating_sub(2);
                if angle == 0 {
                    return toks.get(j + 1).is_some_and(|n| n.kind == K::LParen);
                }
            }
            _ => {}
        }
    }
    false
}
