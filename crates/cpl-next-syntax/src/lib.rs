//! The lossless syntax tree: the one lexer and parser under every reader
//! of CP Lang source (`handbook/design/next/syntax.md`).
//!
//! - [`lex`] turns a file into tokens with layout (`Indent`, `Dedent`,
//!   `Newline`) made explicit; [`Lexed::lossless`] adds the trivia, so the
//!   tokens' texts are the file, byte for byte;
//! - [`split_items`] cuts the token list into top-level items;
//! - [`parse`] builds the file's green tree and its red view;
//! - [`ast`] reads typed nodes over it.
//!
//! None of them fails: any text gives a tree that holds every byte of it.
//! Nothing here depends on the compiler.

pub mod ast;
pub mod doc;
pub mod fall_through;
mod file;
pub mod grammar;
mod items;
mod lexer;
pub mod literal;
pub mod ops;
mod parser;
mod token;
pub mod tree;
pub mod visit;

pub use file::{parse, NodeKind, SyntaxFile};
pub use items::{split_items, ItemRange};
pub use lexer::{lex, Lexed};
pub use parser::{parse_expr, parse_stmts};
pub use token::{arg_labels, removed_keyword, Token, TokenKind, KEYWORDS, LITERAL_WORDS};
pub use tree::{GreenNode, GreenToken, SyntaxElement, SyntaxNode, SyntaxToken};

#[cfg(test)]
mod tests;
