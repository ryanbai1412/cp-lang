//! Applying the fixes diagnostics carry as edits: the text step every
//! fix path shares (`cpl fix`, `cpl lint --fix`, the editor, migrate).

use crate::{Diagnostic, Span};

/// Applies the first fix of every diagnostic that has one. Fixes that
/// overlap an earlier one are left for the next run; a fix with an edit
/// outside `src` (a span that ends before it starts, past the end, or inside
/// a character) is a producer bug and is skipped, never applied.
pub fn apply_fixes(src: &str, diagnostics: &[Diagnostic]) -> (String, usize) {
    let mut edits: Vec<(Span, String)> = Vec::new();
    let mut applied = 0;
    let mut fixes: Vec<_> = diagnostics.iter().filter_map(|d| d.fixes.first()).collect();
    fixes.sort_by_key(|fix| fix.edits.iter().map(|(span, _)| span.start).min());
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
        if clashes || out_of_source || fix.edits.is_empty() {
            continue;
        }
        edits.extend(
            fix.edits
                .iter()
                .map(|(span, text)| crate::edit_like(src, *span, text)),
        );
        applied += 1;
    }
    // An insertion at a replacement's start goes before it.
    edits.sort_by_key(|(span, _)| (span.start, span.end));
    let mut out = String::with_capacity(src.len());
    let mut at = 0usize;
    for (span, text) in edits {
        out.push_str(&src[at..span.start as usize]);
        out.push_str(&text);
        at = span.end as usize;
    }
    out.push_str(&src[at..]);
    (out, applied)
}
