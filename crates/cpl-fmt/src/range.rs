//! Formatting part of a file: a selection ([`format_range`]) or the lines a
//! keystroke finished ([`format_on_type`]), for the editor. The whole file is
//! laid out once and the laid-out text of whole statements is spliced back
//! between two cuts; every byte outside them is left as it was.
//!
//! A cut is a line start between two statements, in the source and in the
//! laid-out text alike, found through the tokens the two share. A splice
//! that re-indents part of a block whose siblings keep another indentation
//! may not lex the same; then the cuts widen to the top-level statements
//! around it.

use std::ops::Range;

use cpl_syntax::{Token, TokenKind};

use crate::{laid_out, verify, FormatError, Style};

/// Replace the bytes `range` of the source with `text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub text: String,
}

/// Formats the statements that `range` (bytes) touches, widened to whole
/// statements. `None` when they are already formatted. `use` runs are only
/// sorted by formatting the whole file.
pub fn format_range(
    src: &str,
    style: &Style,
    range: Range<usize>,
) -> Result<Option<Edit>, FormatError> {
    in_lf(src, range, |lf, range| splice(lf, style, range, None))
}

/// Formats what typing `ch` just before byte `at` finished: after a newline
/// the statement on the line above, leaving the new line (and the editor's
/// indentation on it) alone; after `:` or anything else, the statement on
/// the cursor's line. `None` when that statement is not yet whole, already
/// formatted, or the file does not lex.
pub fn format_on_type(
    src: &str,
    style: &Style,
    at: usize,
    ch: char,
) -> Result<Option<Edit>, FormatError> {
    in_lf(src, at..at, |lf, range| {
        let at = range.start.min(lf.len());
        let line = line_start(lf, at);
        if ch != '\n' {
            return splice(lf, style, line..line_end(lf, at), None);
        }
        if line == 0 {
            return Ok(None);
        }
        splice(lf, style, line_start(lf, line - 1)..line - 1, Some(line))
    })
}

/// Runs `f` on `src` with LF line ends, mapping offsets both ways for a
/// file whose every line ends in CRLF (as `format_with` keeps it).
fn in_lf(
    src: &str,
    range: Range<usize>,
    f: impl FnOnce(&str, Range<usize>) -> Result<Option<Edit>, FormatError>,
) -> Result<Option<Edit>, FormatError> {
    let lines = src.matches('\n').count();
    if lines == 0 || src.matches("\r\n").count() != lines {
        return f(src, range);
    }
    let lf = src.replace("\r\n", "\n");
    let to_lf = |o: usize| {
        let o = o.min(src.len());
        o - src[..o].matches("\r\n").count() - usize::from(src[..o].ends_with('\r'))
    };
    let to_crlf = |o: usize| o + lf[..o].matches('\n').count();
    Ok(f(&lf, to_lf(range.start)..to_lf(range.end))?.map(|e| Edit {
        range: to_crlf(e.range.start)..to_crlf(e.range.end),
        text: e.text.replace('\n', "\r\n"),
    }))
}

/// A cut: the same place between two statements in the source and in the
/// laid-out text, and whether it is between top-level statements.
#[derive(Clone, Copy)]
struct Cut {
    src: usize,
    out: usize,
    top: bool,
}

fn splice(
    src: &str,
    style: &Style,
    range: Range<usize>,
    limit: Option<usize>,
) -> Result<Option<Edit>, FormatError> {
    let (tokens, out) = laid_out(src, style)?;
    let lexed = cpl_syntax::lex(&out);
    let (a, b) = (kept(src, &tokens), kept(&out, &lexed.tokens));
    if a.len() != b.len() {
        return Err(FormatError::Internal(
            "the laid-out tokens do not line up".into(),
        ));
    }
    let layout = |k: usize| {
        matches!(
            a[k].kind,
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
        )
    };
    let column0 = |k: usize| b[k].span.start as usize == line_start(&out, b[k].span.start as usize);
    let (mut starts, mut ends) = (Vec::new(), Vec::new());
    for k in (0..a.len()).filter(|&k| !layout(k)) {
        if k == 0 || layout(k - 1) {
            let (s, o) = (a[k].span.start as usize, b[k].span.start as usize);
            if let (Some(src_at), Some(out_at)) = (start_cut(src, s), start_cut(&out, o)) {
                starts.push(Cut {
                    src: src_at,
                    out: out_at,
                    top: column0(k),
                });
            }
        }
        if k + 1 == a.len() || layout(k + 1) {
            let (s, o) = (a[k].span.end as usize, b[k].span.end as usize);
            if let (Some(src_at), Some(mut out_at)) = (end_cut(src, s), end_cut(&out, o)) {
                // The file's last line keeps its missing line end.
                if src_at == src.len() && !src.ends_with('\n') && out[..out_at].ends_with('\n') {
                    out_at -= 1;
                }
                let next = (k + 1..a.len()).find(|&j| !layout(j));
                ends.push(Cut {
                    src: src_at,
                    out: out_at,
                    top: next.is_none_or(column0),
                });
            }
        }
    }
    let Some(mut from) = starts
        .iter()
        .rev()
        .find(|c| c.src <= range.start)
        .or(starts.first())
        .copied()
    else {
        return Ok(None);
    };
    let Some(mut to) = ends
        .iter()
        .find(|c| c.src >= range.end && c.src > from.src)
        .copied()
    else {
        return Ok(None);
    };
    if limit.is_some_and(|l| to.src > l) {
        return Ok(None);
    }
    let joined = |from: Cut, to: Cut| {
        format!(
            "{}{}{}",
            &src[..from.src],
            &out[from.out..to.out],
            &src[to.src..]
        )
    };
    if verify(src, &tokens, &joined(from, to)).is_err() {
        if limit.is_some() {
            return Ok(None);
        }
        let outer_from = starts.iter().rev().find(|c| c.top && c.src <= from.src);
        let outer_to = ends.iter().find(|c| c.top && c.src >= to.src);
        let (Some(&f), Some(&t)) = (outer_from, outer_to) else {
            return Ok(None);
        };
        (from, to) = (f, t);
        verify(src, &tokens, &joined(from, to))?;
    }
    Ok(edit(src, from.src..to.src, &out[from.out..to.out]))
}

/// The tokens `significant` compares, so that the two lists line up.
fn kept<'t>(src: &str, tokens: &'t [Token]) -> Vec<&'t Token> {
    let wanted = crate::significant(src, tokens).len();
    let kept: Vec<&Token> = tokens
        .iter()
        .enumerate()
        .filter(|(i, tok)| {
            !(tok.kind == TokenKind::Comma
                && tokens.get(i + 1).is_some_and(|next| {
                    matches!(
                        next.kind,
                        TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
                    )
                }))
        })
        .map(|(_, tok)| tok)
        .collect();
    debug_assert_eq!(kept.len(), wanted);
    kept
}

/// The start of the line `at` is on, when only indentation comes before it.
fn start_cut(text: &str, at: usize) -> Option<usize> {
    let line = line_start(text, at);
    text[line..at].bytes().all(|b| b == b' ').then_some(line)
}

/// The start of the next line, when only a comment follows `at` on its line.
fn end_cut(text: &str, at: usize) -> Option<usize> {
    let end = line_end(text, at);
    let tail = text[at..end].trim_start();
    (tail.is_empty() || tail.starts_with("//")).then_some((end + 1).min(text.len()))
}

fn line_start(text: &str, at: usize) -> usize {
    text[..at].rfind('\n').map_or(0, |i| i + 1)
}

fn line_end(text: &str, at: usize) -> usize {
    text[at..].find('\n').map_or(text.len(), |i| at + i)
}

/// The smallest edit that turns `src[range]` into `new`.
fn edit(src: &str, range: Range<usize>, new: &str) -> Option<Edit> {
    let old = &src[range.clone()];
    if old == new {
        return None;
    }
    let mut pre = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(pre) || !new.is_char_boundary(pre) {
        pre -= 1;
    }
    let (old_rest, new_rest) = (&old[pre..], &new[pre..]);
    let mut suf = old_rest
        .bytes()
        .rev()
        .zip(new_rest.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    while !old_rest.is_char_boundary(old_rest.len() - suf)
        || !new_rest.is_char_boundary(new_rest.len() - suf)
    {
        suf -= 1;
    }
    Some(Edit {
        range: range.start + pre..range.end - suf,
        text: new_rest[..new_rest.len() - suf].to_string(),
    })
}

#[cfg(test)]
#[path = "range_tests.rs"]
mod tests;
