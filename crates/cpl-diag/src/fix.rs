//! Splicing the fixes diagnostics carry as edits: the unchecked text step
//! under the one checked fix path (`cpl_lint::fix_rounds`, `cpl_ir::fix`),
//! which `cpl fix`, `cpl lint --fix`, the editor and `cpl migrate` share.

use crate::{Diagnostic, Span};

/// Applies the first fix of every diagnostic that has one. Fixes that
/// overlap an earlier one are left for the next run; a fix with an edit
/// outside `src` (a span that ends before it starts, past the end, or inside
/// a character) is a producer bug and is skipped, never applied.
pub fn splice_fixes(src: &str, diagnostics: &[Diagnostic]) -> (String, usize) {
    let mut edits: Vec<(Span, String)> = Vec::new();
    let mut applied = 0;
    let mut fixes: Vec<_> = diagnostics.iter().filter_map(|d| d.fixes.first()).collect();
    crate::sort_by_key(&mut fixes, |fix| {
        fix.edits
            .iter()
            .map(|(span, _)| span.start as u64 + 1)
            .min()
            .unwrap_or(0)
    });
    for fix in fixes {
        let overlap = |a: &Span, b: &Span| a.start < b.end && b.start < a.end;
        let clashes = fix.edits.iter().enumerate().any(|(i, (span, _))| {
            edits.iter().any(|(taken, _)| overlap(span, taken))
                || fix.edits[..i].iter().any(|(other, _)| overlap(span, other))
        });
        let out_of_source = fix.edits.iter().any(|(span, _)| {
            span.start > span.end
                || !src.is_char_boundary(span.start as usize)
                || !src.is_char_boundary(span.end as usize)
        });
        if out_of_source || fix.edits.is_empty() {
            continue;
        }
        let fix_edits: Vec<(Span, String)> = (fix.edits.iter())
            .map(|(span, text)| crate::edit_like(src, *span, text))
            .collect();
        // Two errors can offer the same fix, e.g. one `.clone()` for every
        // later use of a moved value: it is applied once.
        if fix_edits.iter().all(|edit| edits.contains(edit)) {
            continue;
        }
        if clashes {
            continue;
        }
        edits.extend(fix_edits);
        applied += 1;
    }
    match splice(src, edits) {
        Some(out) => (out, applied),
        None => (src.to_string(), 0),
    }
}

/// Replaces each span of `src` with its text, all against the original
/// `src`: the one edit engine every fix, rename, refactor and editor change
/// goes through. An insertion at a replacement's start goes before it.
/// `None` when two edits overlap or one is outside `src` (ends before it
/// starts, past the end, or inside a character).
pub fn splice(src: &str, mut edits: Vec<(Span, String)>) -> Option<String> {
    crate::sort_by_key(&mut edits, |(span, _)| span.order());
    let mut out = String::with_capacity(src.len());
    let mut at = 0usize;
    for (span, text) in &edits {
        let (start, end) = (span.start as usize, span.end as usize);
        if start < at || start > end || !src.is_char_boundary(start) || !src.is_char_boundary(end) {
            return None;
        }
        out.push_str(&src[at..start]);
        out.push_str(text);
        at = end;
    }
    out.push_str(&src[at..]);
    Some(out)
}

/// The edit that brings `name` in from `std.<module>`: into the braces of
/// the file's `use std.<module>.{..}` when it has one, else as a line after
/// its last top-level `use`, else above its first line of code.
pub fn import_edit(text: &str, module: &str, name: &str) -> (crate::Span, String) {
    use_edit(text, &format!("std.{module}"), name)
}

/// The edit that brings `name` in from the module named `path` (`std.seq`), as `import_edit`.
pub fn use_edit(text: &str, path: &str, name: &str) -> (crate::Span, String) {
    let head = format!("use {path}.{{");
    let mut after_uses = None;
    let mut first_code = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        let body = line.trim_end();
        if let Some(close) = body.strip_prefix(&head).and_then(|_| body.rfind('}')) {
            let inside = body[head.len()..close].trim();
            let edit = if inside.is_empty() {
                name.to_string()
            } else {
                format!(", {name}")
            };
            let close = (at + close) as u32;
            return (crate::Span::point(close), edit);
        }
        if body.starts_with("use ") {
            after_uses = Some(end);
        } else if first_code.is_none()
            && !body.trim_start().is_empty()
            && !body.trim_start().starts_with("//")
        {
            first_code = Some(at);
        }
        at = end;
    }
    let line = format!("use {path}.{{{name}}}\n");
    match (after_uses, first_code) {
        (Some(end), _) if !text[..end].ends_with('\n') => {
            (crate::Span::point(end as u32), format!("\n{line}"))
        }
        (Some(end), _) => (crate::Span::point(end as u32), line),
        (None, Some(start)) => (crate::Span::point(start as u32), line),
        (None, None) => (crate::Span::point(text.len() as u32), line),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_apply_against_the_original_text_in_one_pass() {
        let edits = vec![
            (Span::new(4, 5), "yy".to_string()),
            (Span::new(0, 1), "x".to_string()),
            (Span::new(4, 4), "<".to_string()),
        ];
        assert_eq!(splice("a = b + c", edits).as_deref(), Some("x = <yy + c"));
        assert_eq!(splice("abc", Vec::new()).as_deref(), Some("abc"));
    }

    #[test]
    fn overlapping_or_out_of_source_edits_apply_nothing() {
        let overlap = vec![
            (Span::new(0, 3), String::new()),
            (Span::new(2, 4), String::new()),
        ];
        assert_eq!(splice("abcdef", overlap), None);
        assert_eq!(splice("ab", vec![(Span::new(1, 9), String::new())]), None);
        assert_eq!(
            splice("ab", vec![(Span { start: 2, end: 1 }, String::new())]),
            None
        );
        assert_eq!(splice("é", vec![(Span::new(1, 2), String::new())]), None);
    }

    #[test]
    fn a_fix_two_errors_share_is_applied_once() {
        let moved = |at: u32| {
            let mut d = Diagnostic::error(
                crate::codes::UNCLOSED_DELIMITER,
                Span::new(at, at + 1),
                "test",
            );
            d.fix("clone", Span::point(1), ".clone()");
            d
        };
        let (out, applied) = splice_fixes("k f(k) g(k)", &[moved(4), moved(9)]);
        assert_eq!((out.as_str(), applied), ("k.clone() f(k) g(k)", 1));
    }
}
