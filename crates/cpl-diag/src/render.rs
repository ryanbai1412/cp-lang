//! Text rendering for the command line.

use crate::{Diagnostic, LineIndex, Span};
use std::fmt::Write;

/// Renders one diagnostic the way `cpl check` prints it:
///
/// ```text
/// error[E0301]: `deg` is indexed by V, but `e` is an E
///   --> d.cp:7:17
///   |
/// 7 | let x = deg[e]
///   |             ^
///   = help: index with a V
/// ```
pub fn render(diag: &Diagnostic, path: &str, text: &str, lines: &LineIndex) -> String {
    let mut out = String::new();
    let at = lines.line_col(diag.span.start);
    let _ = writeln!(out, "{}[{}]: {}", diag.severity.as_str(), diag.code.0, diag.message);
    let _ = writeln!(out, "  --> {}:{}:{}", path, at.line + 1, at.col + 1);

    let mut marks: Vec<(Span, char, &str)> = vec![(diag.span, '^', "")];
    for label in &diag.labels {
        marks.push((label.span, '-', &label.message));
    }
    marks.sort_by_key(|m| m.0.start);

    let width = marks
        .iter()
        .map(|m| (lines.line_of(m.0.start) + 1).to_string().len())
        .max()
        .unwrap_or(1);
    let _ = writeln!(out, "{:width$} |", "");
    for (span, mark, message) in marks {
        let line = lines.line_of(span.start);
        let (start, end) = lines.line_range(line, text);
        let src = &text[start as usize..end as usize];
        let _ = writeln!(out, "{:>width$} | {}", line + 1, src);
        // Columns are counted in characters so the caret lines up under
        // non-ASCII source.
        let from = span.start.clamp(start, end);
        let to = span.end.clamp(from, end);
        let pad = text[start as usize..from as usize].chars().count();
        let len = text[from as usize..to as usize].chars().count().max(1);
        let _ = write!(out, "{:width$} | {:pad$}{}", "", "", mark.to_string().repeat(len));
        if message.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, " {message}");
        }
    }
    for note in &diag.notes {
        let _ = writeln!(out, "{:width$} = note: {note}", "");
    }
    if let Some(help) = &diag.help {
        let _ = writeln!(out, "{:width$} = help: {help}", "");
    }
    out
}

/// Renders every diagnostic, separated by blank lines, with a summary line.
pub fn render_all(diags: &[Diagnostic], path: &str, text: &str) -> String {
    let lines = LineIndex::new(text);
    let mut out = String::new();
    for diag in diags {
        out.push_str(&render(diag, path, text, &lines));
        out.push('\n');
    }
    let errors = diags.iter().filter(|d| d.severity == crate::Severity::Error).count();
    let warnings = diags.iter().filter(|d| d.severity == crate::Severity::Warning).count();
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    match (errors, warnings) {
        (0, 0) => {}
        (e, 0) => { let _ = writeln!(out, "{path}: {}", plural(e, "error")); }
        (0, w) => { let _ = writeln!(out, "{path}: {}", plural(w, "warning")); }
        (e, w) => { let _ = writeln!(out, "{path}: {}, {}", plural(e, "error"), plural(w, "warning")); }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;

    #[test]
    fn renders_code_location_caret_and_help() {
        let text = "let deg = 1\nlet x = deg[e]\n";
        let at = text.find("[e]").unwrap() as u32 + 1;
        let diag = Diagnostic::error(codes::WRONG_DOMAIN, Span::new(at, at + 1), "`deg` is indexed by V, but `e` is an E")
            .with_help("index with a V");
        let out = render(&diag, "d.cp", text, &LineIndex::new(text));
        assert_eq!(
            out,
            "error[E0301]: `deg` is indexed by V, but `e` is an E\n  --> d.cp:2:13\n  |\n2 | let x = deg[e]\n  |             ^\n  = help: index with a V\n"
        );
    }

    #[test]
    fn labels_are_shown_in_source_order_with_their_text() {
        let text = "let n = 1\nn = 2\n";
        let diag = Diagnostic::error(codes::ASSIGN_TO_LET, Span::new(10, 11), "`n` is a `let`, so it can't be assigned")
            .with_label(Span::new(4, 5), "declared here");
        let out = render(&diag, "a.cp", text, &LineIndex::new(text));
        assert!(out.contains("1 | let n = 1\n  |     - declared here\n2 | n = 2\n  | ^\n"), "{out}");
    }

    #[test]
    fn an_empty_span_at_end_of_line_still_gets_a_caret() {
        let text = "let x =\n";
        let diag = Diagnostic::error(codes::EXPECTED_EXPR, Span::point(7), "expected a value after `=`");
        let out = render(&diag, "a.cp", text, &LineIndex::new(text));
        assert!(out.contains("1 | let x =\n  |        ^\n"), "{out}");
    }

    #[test]
    fn summary_counts_errors_and_warnings() {
        let text = "x\n";
        let diags = vec![
            Diagnostic::error(codes::UNRESOLVED_NAME, Span::new(0, 1), "a"),
            Diagnostic::warning(codes::TYPE_MISMATCH, Span::new(0, 1), "b"),
        ];
        assert!(render_all(&diags, "a.cp", text).ends_with("a.cp: 1 error, 1 warning\n"));
        assert_eq!(render_all(&[], "a.cp", text), "");
    }
}
