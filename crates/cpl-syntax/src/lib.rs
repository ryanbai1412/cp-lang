//! Lexer, parser, and syntax tree for the cp language.
//!
//! The surface syntax is `plan/17-syntax.md`. Three entry points, in the
//! order the incremental front end uses them:
//!
//! - [`lex`] turns a file into tokens with layout (`Indent`, `Dedent`,
//!   `Newline`) made explicit;
//! - [`split_items`] cuts the token list into top-level items;
//! - [`parse_item`] parses one item into a tree with item-relative spans.
//!
//! None of them fails: broken source gives a full tree with `Missing` and
//! `Error` nodes and one diagnostic per root cause.

pub mod ast;
pub mod const_int;
pub mod doc;
mod items;
mod lexer;
pub mod library_lint;
pub mod literal;
pub mod narrow;
pub mod order;
mod parser;
pub mod planned;
mod token;
pub mod visit;

pub use items::{split_items, ItemRange};
pub use lexer::{lex, Lexed};
pub use parser::{parse_item, ParsedItem};
pub use token::{removed_keyword, Token, TokenKind, KEYWORDS, LITERAL_WORDS};

use cpl_diag::{Diagnostic, Span};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// The lexer's poisoned spans that fall inside `item`, relative to it.
pub fn poison_for_item(lexed: &Lexed, item: &ItemRange) -> Vec<Span> {
    lexed
        .poisoned
        .iter()
        .filter(|p| item.span.contains(**p))
        .map(|p| Span::new(p.start - item.span.start, p.end - item.span.start))
        .collect()
}

/// A whole file parsed in one go, with file positions. The incremental
/// front end in `cpl-check` does the same work item by item with caching;
/// this is the simple form for tools and tests.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// Each item with the offset its spans are relative to.
    pub items: Vec<(u32, ParsedItem)>,
    /// Every syntax diagnostic, in file positions and source order.
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_file(src: &str) -> ParsedFile {
    let lexed = lex(src);
    let mut diagnostics = lexed.diagnostics.clone();
    let mut items = Vec::new();
    for range in split_items(&lexed.tokens) {
        let tokens = &lexed.tokens[range.tokens.0..range.tokens.1];
        let parsed = parse_item(
            src,
            tokens,
            range.span.start,
            poison_for_item(&lexed, &range),
        );
        diagnostics.extend(
            parsed
                .diagnostics
                .iter()
                .cloned()
                .map(|d| d.shift(range.span.start)),
        );
        items.push((range.span.start, parsed));
    }
    diagnostics.sort_by_key(|d| (d.span.start, d.span.end));
    ParsedFile { items, diagnostics }
}

#[cfg(test)]
mod foreign_tests;
#[cfg(test)]
mod generator_tests;
#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod str_type_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trait_tests;
