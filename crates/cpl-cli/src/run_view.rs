//! The `view` of `cpl run --format json`: each line's last values on the
//! sample, and the line a run trapped on with its problem.

use cpl_diag::view::{code_block, Diagnostic, Kind, Line, View};
use cpl_facade::Live;

/// The name labels give the input: `--sample`, else the input file's name
/// without `.in` (`i1`, `2` for `a.tests/2.in`).
pub fn sample_name<'a>(sample: Option<&'a str>, input: Option<&'a str>) -> Option<&'a str> {
    sample.or_else(|| {
        let name = std::path::Path::new(input?).file_name()?.to_str()?;
        Some(name.strip_suffix(".in").unwrap_or(name))
    })
}

/// `lives` 1-based as the recording keeps them; `trap` with its 0-based line and column.
pub fn view(
    sample: Option<&str>,
    lives: &[Live],
    panel: String,
    trap: Option<(&cpl_diag::Diagnostic, u32, u32)>,
) -> View {
    let mut lines = Vec::new();
    if let Some((d, line, _)) = trap {
        let code = format!("[{}] ", d.code.0);
        let prefix = sample.map_or("runtime error: ".into(), |s| format!("{s} error: "));
        let title = sample.map_or("Runtime error:".into(), |s| {
            format!("Runtime error on `{s}`:")
        });
        let body = match &d.help {
            Some(help) => format!("{}\nhelp: {help}", d.message),
            None => d.message.clone(),
        };
        lines.push(Line {
            line,
            kind: Kind::Error,
            label: format!("{prefix}{code}{}", d.message),
            hover: Some(format!("{title}\n\n{}", code_block(&body, "text"))),
        });
    }
    let mut at: Vec<u32> = lives.iter().map(|l| l.line.saturating_sub(1)).collect();
    at.sort_unstable();
    at.dedup();
    // A line that shows the trap shows only that.
    for line in at
        .into_iter()
        .filter(|l| trap.is_none_or(|(_, t, _)| t != *l))
    {
        let here: Vec<&Live> = lives
            .iter()
            .filter(|l| l.line.saturating_sub(1) == line)
            .collect();
        let shown = match here.as_slice() {
            [one] => one.value.clone(),
            many => many
                .iter()
                .map(|l| format!("{} = {}", l.name, l.value))
                .collect::<Vec<_>>()
                .join("  "),
        };
        let all = here
            .iter()
            .map(|l| format!("{} = {}", l.name, l.value))
            .collect::<Vec<_>>();
        let title = sample.map_or("Last values:".into(), |s| format!("On `{s}`:"));
        lines.push(Line {
            line,
            kind: Kind::Live,
            label: sample.map_or(shown.clone(), |s| format!("{s}: {shown}")),
            hover: Some(format!("{title}\n\n{}", code_block(&all.join("\n"), "cp"))),
        });
    }
    let diagnostic = trap.map(|(d, line, col)| Diagnostic {
        line,
        col: Some(col),
        code: Some(d.code.0.to_string()),
        message: d.message.clone(),
        help: d.help.clone(),
        source: Some(format!("cpl ({})", sample.unwrap_or("runtime"))),
    });
    View {
        panel,
        status: None,
        lines,
        diagnostic,
        ..View::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(line: u32, name: &str, value: &str) -> Live {
        Live {
            line,
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn groups_values_by_line_and_names_them_when_several() {
        let lives = [
            live(5, "n", "2"),
            live(4, "dist", "[0, 3,\n none]"),
            live(5, "m", "5"),
        ];
        let v = view(Some("i1"), &lives, "6\n".into(), None);
        let labels: Vec<_> = v
            .lines
            .iter()
            .map(|l| (l.line, l.kind, l.label.as_str()))
            .collect();
        assert_eq!(
            labels,
            [
                (3, Kind::Live, "i1: [0, 3,\n none]"),
                (4, Kind::Live, "i1: n = 2  m = 5")
            ]
        );
        assert_eq!(
            v.lines[1].hover.as_deref(),
            Some("On `i1`:\n\n```cp\nn = 2\nm = 5\n```")
        );
        assert_eq!(
            (v.panel.as_str(), v.diagnostic, v.status),
            ("6\n", None, None)
        );
        assert_eq!(
            view(None, &lives[1..2], String::new(), None).lines[0].label,
            "[0, 3,\n none]"
        );
    }

    #[test]
    fn a_trap_marks_its_line_alone_and_is_the_diagnostic() {
        let d = cpl_diag::Diagnostic::error(
            cpl_diag::Code("R0301"),
            cpl_diag::Span { start: 0, end: 1 },
            "index 5 out of range",
        )
        .with_help("check i < n");
        let lives = [live(3, "x", "1"), live(2, "y", "2")];
        let v = view(Some("i1"), &lives, String::new(), Some((&d, 2, 4)));
        assert_eq!(v.lines.len(), 2);
        assert_eq!((v.lines[0].line, v.lines[0].kind), (2, Kind::Error));
        assert_eq!(v.lines[0].label, "i1 error: [R0301] index 5 out of range");
        assert_eq!(
            v.lines[0].hover.as_deref(),
            Some("Runtime error on `i1`:\n\n```text\nindex 5 out of range\nhelp: check i < n\n```")
        );
        assert_eq!((v.lines[1].line, v.lines[1].label.as_str()), (1, "i1: 2"));
        let diag = v.diagnostic.unwrap();
        assert_eq!(
            (diag.line, diag.col, diag.source.as_deref()),
            (2, Some(4), Some("cpl (i1)"))
        );
        assert_eq!(diag.help.as_deref(), Some("check i < n"));
        let anon = view(None, &[], String::new(), Some((&d, 0, 0)));
        assert_eq!(
            anon.lines[0].label,
            "runtime error: [R0301] index 5 out of range"
        );
        assert_eq!(
            anon.diagnostic.unwrap().source.as_deref(),
            Some("cpl (runtime)")
        );
    }

    #[test]
    fn names_the_sample_from_the_flag_or_the_input_file() {
        assert_eq!(sample_name(Some("s"), Some("i1")), Some("s"));
        assert_eq!(sample_name(None, Some("a.tests/2.in")), Some("2"));
        assert_eq!(sample_name(None, Some("i1")), Some("i1"));
        assert_eq!(sample_name(None, None), None);
    }
}
