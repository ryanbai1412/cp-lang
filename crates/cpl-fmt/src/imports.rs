//! Sorted `use` runs: consecutive one-line `use` statements at one
//! indentation are ordered, `std` paths first, then by path. A blank line,
//! a comment line or a `use` that spans lines ends a run, so a reader can
//! group imports by hand and the formatter keeps the groups.

use crate::FormatError;
use cpl_syntax::TokenKind;

/// `src` with every run of one-line `use` lines sorted.
pub(crate) fn sort_use_runs(src: &str) -> String {
    let lexed = cpl_syntax::lex(src);
    if !lexed.diagnostics.is_empty() || src.is_empty() {
        return src.to_string();
    }
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    let mut starts = Vec::with_capacity(lines.len());
    let mut at = 0;
    for line in &lines {
        starts.push(at);
        at += line.len();
    }
    let line_of = |offset: u32| starts.partition_point(|&s| s <= offset as usize) - 1;
    // (line, column, key) of each line that holds a whole `use` statement.
    let mut uses: Vec<(usize, usize, (bool, &str))> = Vec::new();
    let toks = &lexed.tokens;
    for (i, tok) in toks.iter().enumerate() {
        let line = line_of(tok.span.start);
        let col = tok.span.start as usize - starts[line];
        if tok.kind != TokenKind::Use || !src[starts[line]..][..col].trim().is_empty() {
            continue;
        }
        let end = toks[i..]
            .iter()
            .position(|t| t.kind == TokenKind::Newline)
            .map_or(toks.len(), |n| i + n);
        let (Some(first), Some(last)) = (toks.get(i + 1), toks[..end].last()) else {
            continue;
        };
        if end == i + 1 || line_of(last.span.start) != line || !lines[line].ends_with('\n') {
            continue;
        }
        let path = &src[first.span.start as usize..last.span.end as usize];
        let not_std = !(path == "std" || path.starts_with("std.") || path.starts_with("std "));
        uses.push((line, col, (not_std, path)));
    }
    let mut out: Vec<&str> = lines.clone();
    let mut i = 0;
    while i < uses.len() {
        let (line, col, _) = uses[i];
        let mut j = i;
        while j < uses.len() && uses[j].0 == line + (j - i) && uses[j].1 == col {
            j += 1;
        }
        let mut run: Vec<_> = uses[i..j]
            .iter()
            .map(|&(l, _, key)| (key, lines[l]))
            .collect();
        run.sort_by(|a, b| a.0.cmp(&b.0));
        for (k, (_, text)) in run.into_iter().enumerate() {
            out[line + k] = text;
        }
        i = j;
    }
    out.concat()
}

/// Checks that sorting only moved whole lines and the result still lexes.
pub(crate) fn verify(before: &str, after: &str) -> Result<(), FormatError> {
    if let Some(diag) = cpl_syntax::lex(after).diagnostics.first() {
        return Err(FormatError::Internal(format!(
            "sorting `use` lines broke the lexing: {}",
            diag.message
        )));
    }
    let mut a: Vec<&str> = before.split_inclusive('\n').collect();
    let mut b: Vec<&str> = after.split_inclusive('\n').collect();
    a.sort_unstable();
    b.sort_unstable();
    if a != b {
        return Err(FormatError::Internal(
            "sorting `use` lines changed a line".to_string(),
        ));
    }
    Ok(())
}
