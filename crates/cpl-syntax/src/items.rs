//! Splitting a file into top-level items.
//!
//! An item is one top-level statement with everything that belongs to it:
//! its decorators, the block indented under it, and `elif`/`else` branches.
//! Items are the unit of incremental work. Each is parsed and checked on its
//! own and cached by its text, so typing inside one fn re-parses that fn and
//! nothing else.

use crate::token::{Token, TokenKind as T};
use cpl_diag::Span;

/// Where one item sits in the file.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemRange {
    /// The item's tokens, as a range into the file's token list.
    pub tokens: (usize, usize),
    /// The item's text, from its first token to its last.
    pub span: Span,
}

pub fn split_items(tokens: &[Token]) -> Vec<ItemRange> {
    let mut items = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if matches!(tokens[i].kind, T::Eof | T::Newline) {
            i += 1;
            continue;
        }
        let first = i;
        let mut depth = 0i32;
        // A line of decorators belongs to the statement after it.
        let mut decorator_line = tokens[i].kind == T::At;
        loop {
            match tokens[i].kind {
                T::Eof => break,
                T::Indent => depth += 1,
                T::Dedent => depth -= 1,
                T::Newline if depth == 0 => {
                    let next = tokens.get(i + 1).map_or(T::Eof, |t| t.kind);
                    let continues = decorator_line || matches!(next, T::Indent | T::Elif | T::Else);
                    if !continues {
                        i += 1;
                        break;
                    }
                    if decorator_line {
                        decorator_line = next == T::At;
                    }
                }
                _ => {}
            }
            i += 1;
            // After a block closes, only `elif`/`else` continue the item.
            if depth == 0 && tokens[i - 1].kind == T::Dedent && !matches!(tokens[i].kind, T::Elif | T::Else | T::Dedent) {
                break;
            }
            if depth < 0 {
                break;
            }
        }
        let start = tokens[first].span.start;
        // Layout tokens sit where the next line starts; the item's text ends
        // at its last real token.
        let end = tokens[first..i]
            .iter()
            .rev()
            .find(|t| !matches!(t.kind, T::Newline | T::Indent | T::Dedent))
            .map_or(start, |t| t.span.end);
        items.push(ItemRange {
            tokens: (first, i),
            span: Span::new(start, end),
        });
    }
    items
}
