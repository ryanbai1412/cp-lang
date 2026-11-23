//! Labels for the other half of an error. An error that names something
//! the file declares on another line shows that declaration, so `x = 2`
//! on a `let x` points at the `let` and the user need not search for it.
//! Rule: `errors/R4`, handbook/guidelines/errors-checklist.md#r4-a-label-marks-the-other-half.

use cpl_diag::{Diagnostic, Label, Severity, Span};

use crate::db::Analysis;
use crate::hover::innermost;

/// Adds a "declared here" label to each error that has none and names, in
/// backticks, a name whose declaration is on another line: the one the
/// name on the error's line resolves to, or else the top-level one.
pub(crate) fn label_declarations(analysis: &Analysis, diagnostics: &mut [Diagnostic]) {
    for d in diagnostics {
        if d.severity != Severity::Error
            || !d.labels.is_empty()
            || d.span.end as usize > analysis.text.len()
        {
            continue;
        }
        let line = analysis.lines.line_of(d.span.start);
        for name in d
            .message
            .split('`')
            .skip(1)
            .step_by(2)
            .filter(|n| is_ident(n))
        {
            let Some(decl) = (match occurrence(analysis, d.span, line, name) {
                Some(at) => declaration(analysis, at, name),
                None => analysis.decl_span(name),
            }) else {
                continue;
            };
            if analysis.lines.line_of(decl.start) != line {
                d.labels.push(Label {
                    span: decl,
                    message: format!("`{name}` is declared here"),
                });
                break;
            }
        }
    }
}

fn is_ident(s: &str) -> bool {
    s.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Where `name` is written as a whole word on the error's line, inside its
/// span if it is there.
fn occurrence(analysis: &Analysis, span: Span, line: u32, name: &str) -> Option<u32> {
    let text = &*analysis.text;
    let start = analysis.lines.line_start(line) as usize;
    let end = text[start..].find('\n').map_or(text.len(), |n| start + n);
    let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
    let hits: Vec<u32> = text[start..end]
        .match_indices(name)
        .map(|(at, _)| start + at)
        .filter(|&at| {
            !word(text[..at].chars().next_back()) && !word(text[at + name.len()..].chars().next())
        })
        .map(|at| at as u32)
        .collect();
    hits.iter()
        .copied()
        .find(|&at| span.start <= at && at < span.end)
        .or_else(|| hits.first().copied())
}

/// The declaration of the name used at `at`, if it is in this file.
fn declaration(analysis: &Analysis, at: u32, name: &str) -> Option<Span> {
    let (item, rel) = analysis.item_at(at)?;
    let entry = innermost(&item.check.hovers, rel)?;
    if entry.name != name || entry.keyword == "field" {
        return None;
    }
    match entry.decl {
        Some(decl) => Some(decl.shift(item.offset)),
        None => analysis.decl_span(name),
    }
}

/// `analysis` with its own errors labelled.
pub(crate) fn labelled(mut analysis: Analysis) -> Analysis {
    let mut diagnostics = std::mem::take(&mut analysis.diagnostics);
    label_declarations(&analysis, &mut diagnostics);
    analysis.diagnostics = diagnostics;
    analysis
}

impl Analysis {
    /// Labels the declarations that `diagnostics` from later passes (lowering)
    /// name, the way the checker's own errors are labelled.
    pub fn label_declarations(&self, diagnostics: &mut [Diagnostic]) {
        label_declarations(self, diagnostics);
    }
}
