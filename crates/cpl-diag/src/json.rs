//! Diagnostics as JSON: the shape every `--json` command prints.

use crate::{Diagnostic, LineIndex};
use cpl_json::{obj, Value};

fn text(s: &str) -> Value {
    Value::String(s.to_string())
}

/// A byte offset as its 1-based line and byte column, and the offset itself.
fn position(offset: u32, lines: &LineIndex) -> Value {
    let at = lines.line_col(offset);
    obj([
        ("line", Value::Int(i64::from(at.line) + 1)),
        ("col", Value::Int(i64::from(at.col) + 1)),
        ("offset", Value::Int(offset.into())),
    ])
}

/// One diagnostic with everything the text rendering shows.
pub fn diagnostic(d: &Diagnostic, lines: &LineIndex) -> Value {
    let labels = d.labels.iter().map(|label| {
        obj([
            ("message", text(&label.message)),
            ("start", position(label.span.start, lines)),
            ("end", position(label.span.end, lines)),
        ])
    });
    let fixes = d.fixes.iter().map(|fix| {
        let edits = fix.edits.iter().map(|(span, new)| {
            obj([
                ("start", position(span.start, lines)),
                ("end", position(span.end, lines)),
                ("text", text(new)),
            ])
        });
        obj([
            ("title", text(&fix.title)),
            ("edits", Value::Array(edits.collect())),
        ])
    });
    obj([
        ("code", text(d.code.0)),
        ("severity", text(d.severity.as_str())),
        ("message", text(&d.message)),
        ("start", position(d.span.start, lines)),
        ("end", position(d.span.end, lines)),
        ("labels", Value::Array(labels.collect())),
        (
            "notes",
            Value::Array(d.notes.iter().map(|n| text(n)).collect()),
        ),
        ("help", d.help.as_deref().map_or(Value::Null, text)),
        ("fixes", Value::Array(fixes.collect())),
        (
            "witness",
            Value::Array(
                d.witness
                    .iter()
                    .map(|(name, value)| {
                        let value = i64::try_from(*value)
                            .map_or_else(|_| text(&value.to_string()), Value::Int);
                        obj([("name", text(name)), ("value", value)])
                    })
                    .collect(),
            ),
        ),
        (
            "page",
            crate::registry::page(d.code.0).map_or(Value::Null, |p| text(&p)),
        ),
    ])
}
