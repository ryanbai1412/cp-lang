//! Source formatter: one opinionated style; a project may change only the
//! indent and line widths ([`Style`], `cp.toml [fmt]`).
//!
//! [`format`] works on the token stream and the comments the lexer keeps,
//! not on the syntax tree, so it formats code the parser cannot read yet
//! and can never drop a comment. It changes whitespace and trailing commas,
//! and sorts runs of `use` lines (`imports`), and nothing else. It checks
//! both before it returns: the laid-out text is lexed again and compared
//! token by token with the input, and sorting may only move whole lines.
//!
//! The style: four spaces per level, at most 100 columns (both [`Style`]), one blank line
//! at most between statements, a bracketed list either on one line or one
//! element per line with a trailing comma.

mod imports;
mod layout;
mod spacing;
mod style;

pub mod cli;

pub use style::{Style, StyleError};

#[cfg(test)]
mod tests;

use cpl_diag::Diagnostic;
use cpl_syntax::{Token, TokenKind};

/// Name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Spaces per indentation level, by default.
pub const INDENT: usize = 4;

/// Lines are broken to stay within this many columns where a break is legal,
/// by default.
pub const MAX_WIDTH: usize = 100;

/// Why a file was not formatted.
#[derive(Debug, Clone, PartialEq)]
pub enum FormatError {
    /// The lexer could not read the file: an unclosed bracket, a tab used
    /// for indentation, an unterminated string. Formatting around those
    /// would guess at the layout.
    Lex(Vec<Diagnostic>),
    /// The formatted text does not have the tokens of the input. This is a
    /// bug in the formatter; the file is left alone.
    Internal(String),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::Lex(diags) => write!(
                f,
                "{} error{} to fix before formatting",
                diags.len(),
                if diags.len() == 1 { "" } else { "s" }
            ),
            FormatError::Internal(what) => write!(f, "formatter bug: {what}"),
        }
    }
}

/// One token with what the formatter needs to know about its surroundings.
#[derive(Debug, Clone)]
pub(crate) struct Tok<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    /// The source had whitespace, a comment or a line break before it.
    pub space_before: bool,
    /// Column of the token in the source.
    pub col: u32,
    /// Comments on their own lines between the previous token and this one.
    pub leading: Vec<Comment<'a>>,
    /// The comment that follows it on its line.
    pub trailing: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Comment<'a> {
    pub text: &'a str,
    pub col: u32,
    pub blank_before: bool,
}

/// Formats a file in the default style.
pub fn format(src: &str) -> Result<String, FormatError> {
    format_with(src, &Style::default())
}

/// Formats a file in `style`. A file whose every line ends in CRLF, as on a
/// Windows checkout, stays CRLF.
pub fn format_with(src: &str, style: &Style) -> Result<String, FormatError> {
    let lines = src.matches('\n').count();
    if lines > 0 && src.matches("\r\n").count() == lines {
        let out = format_lf(&src.replace("\r\n", "\n"), style)?;
        return Ok(out.replace('\n', "\r\n"));
    }
    format_lf(src, style)
}

fn format_lf(src: &str, style: &Style) -> Result<String, FormatError> {
    let lexed = cpl_syntax::lex(src);
    if !lexed.diagnostics.is_empty() {
        return Err(FormatError::Lex(lexed.diagnostics));
    }
    let out = layout::render(src, &lexed, style);
    verify(src, &lexed.tokens, &out)?;
    let sorted = imports::sort_use_runs(&out);
    imports::verify(&out, &sorted)?;
    Ok(sorted)
}

/// Whether `src` is already formatted. A file that cannot be formatted is
/// not.
pub fn is_formatted(src: &str) -> bool {
    format(src).is_ok_and(|out| out == src)
}

/// Checks that formatting changed nothing but whitespace and trailing commas.
fn verify(src: &str, before: &[Token], out: &str) -> Result<(), FormatError> {
    let after = cpl_syntax::lex(out);
    if let Some(diag) = after.diagnostics.first() {
        return Err(FormatError::Internal(format!(
            "the result does not lex: {}",
            diag.message
        )));
    }
    let a = significant(src, before);
    let b = significant(out, &after.tokens);
    if let Some(at) = (0..a.len().max(b.len())).find(|&i| a.get(i) != b.get(i)) {
        let show = |t: Option<&(TokenKind, &str)>| match t {
            Some((kind, "")) => kind.describe().to_string(),
            Some((_, text)) => format!("`{text}`"),
            None => "the end of the file".to_string(),
        };
        return Err(FormatError::Internal(format!(
            "token {} was {} and became {}",
            at + 1,
            show(a.get(at)),
            show(b.get(at))
        )));
    }
    Ok(())
}

/// The tokens formatting must keep: all of them, less a comma right before
/// a closing bracket.
fn significant<'a>(src: &'a str, tokens: &[Token]) -> Vec<(TokenKind, &'a str)> {
    let mut out = Vec::with_capacity(tokens.len());
    for (i, tok) in tokens.iter().enumerate() {
        let before_closer = tokens.get(i + 1).is_some_and(|next| {
            matches!(
                next.kind,
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
            )
        });
        if tok.kind == TokenKind::Comma && before_closer {
            continue;
        }
        out.push((tok.kind, tok.span.text(src)));
    }
    out
}
