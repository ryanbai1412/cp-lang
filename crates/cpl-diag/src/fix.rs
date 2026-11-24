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
