//! What `cpl stress --format json` shows in an editor (spec/tools/cli.md,
//! "The view"): the CLI's own text as the panel, and on a failure a mark
//! on each line a `debug(...)` stage ran, on the solution's line of the
//! first divergence, and the counterexample itself on line 0.

use crate::cli::divergence_title;
use crate::report::StressResult;
use cpl_diag::view::{code_block, Diagnostic, Kind, Line, Status, View};
use cpl_interp::trace::Divergence;

pub fn view(result: &StressResult, panel: String) -> View {
    let report = &result.report;
    let Some(failure) = &report.failure else {
        let label = format!("stress: {} cases passed", report.cases);
        let status = Status {
            line: 0,
            ok: true,
            label,
            detail: None,
        };
        return View {
            panel,
            status: Some(status),
            ..View::default()
        };
    };
    let input = failure
        .described
        .as_deref()
        .unwrap_or(failure.input.trim_end());
    let (got, expected) = (failure.got.trim_end(), failure.expected.trim_end());
    let mut lines = Vec::new();
    // Later differences usually follow from the first, so only it differs.
    let first = result.stages.iter().position(|s| s.fast != s.brute);
    for (index, stage) in result.stages.iter().enumerate() {
        let Some(line) = stage.line.checked_sub(1) else {
            continue;
        };
        let (fast, brute) = (&stage.fast, &stage.brute);
        let (kind, label) = match (Some(index) == first, fast == brute) {
            (true, _) => (
                Kind::Differs,
                format!("fast {fast}  \u{2260}  brute {brute}  \u{2190} first difference"),
            ),
            (false, true) => (Kind::Agrees, format!("fast = brute = {fast}")),
            (false, false) => (
                Kind::Agrees,
                format!("fast {fast}  \u{2260}  brute {brute}"),
            ),
        };
        let shown = code_block(&format!("{input}\nfast  = {fast}\nbrute = {brute}"), "cp");
        let hover = format!("`{}` on the counterexample:\n\n{shown}", stage.name);
        lines.push(Line {
            line,
            kind,
            label,
            hover: Some(hover),
        });
    }
    let marked = |lines: &[Line], line: u32| lines.iter().any(|l| l.line == line);
    if let Some(Ok(Some(d))) = &result.explained {
        let at = d.fast.as_ref().and_then(|(line, _)| line.checked_sub(1));
        if let Some(line) = at.filter(|&line| !marked(&lines, line)) {
            let fast = d.fast.as_ref().map_or("nothing", |(_, value)| value);
            let brute = d.brute.as_ref().map_or("nothing", |(_, value)| value);
            let what = if d.call {
                "first wrong call"
            } else {
                "first differing value"
            };
            let label = format!(
                "{} = {fast}  \u{2260}  brute {brute}  \u{2190} {what}",
                d.name
            );
            let mut shown = vec![
                input.to_string(),
                format!("fast  = {fast}"),
                format!("brute = {brute}"),
            ];
            shown.extend(
                d.inside
                    .iter()
                    .map(|(line, locals)| format!("line {line}: {locals}")),
            );
            let hover = format!(
                "{}\n\n{}",
                divergence_title(d),
                code_block(&shown.join("\n"), "cp")
            );
            lines.push(Line {
                line,
                kind: Kind::Differs,
                label,
                hover: Some(hover),
            });
        }
    }
    if !marked(&lines, 0) {
        let shown = [input, failure.input.trim_end()]
            .into_iter()
            .find(|s| !s.is_empty());
        let shown = shown.unwrap_or("(empty input)");
        let label = match (result.stages.is_empty(), expected.is_empty()) {
            (true, false) => {
                format!("counterexample: {shown}  (fast {got} \u{2260} brute {expected})")
            }
            (true, true) => format!("counterexample ({}): {shown}", failure.verdict.code()),
            (false, _) => format!("counterexample: {shown}"),
        };
        let mut hover =
            "The smallest input `cpl stress` found where the fast code and the reference disagree."
                .to_string();
        if let Some(saved) = &result.saved {
            hover += &format!(" Saved as `{saved}`.");
        }
        if result.stages.is_empty() {
            let raw = Some(failure.input.trim_end())
                .filter(|raw| !raw.is_empty())
                .unwrap_or(input);
            let mut details = vec![raw.to_string(), format!("fast  = {got}")];
            if !expected.is_empty() {
                details.push(format!("brute = {expected}"));
            }
            hover += &format!("\n\n{}", code_block(&details.join("\n"), "cp"));
        }
        lines.push(Line {
            line: 0,
            kind: Kind::Input,
            label,
            hover: Some(hover),
        });
    }
    let diagnostic = match &result.explained {
        Some(Ok(Some(d))) => wrong_call(d),
        _ => None,
    };
    View {
        panel,
        lines,
        diagnostic,
        ..View::default()
    }
}

/// The first wrong call as a diagnostic on the solution's line, its
/// headline the message: what an editor without line marks draws.
pub fn wrong_call(d: &Divergence) -> Option<Diagnostic> {
    let line = d.fast.as_ref()?.0.checked_sub(1)?;
    Some(Diagnostic {
        line,
        col: None,
        code: None,
        message: divergence_title(d),
        help: None,
        source: Some("cpl".to_string()),
    })
}

/// `cpl locate --json`'s view: its text, and where the part first parts
/// from its alt, as `cpl stress` marks it.
pub fn locate(panel: String, d: Option<&Divergence>) -> View {
    let Some((d, diagnostic)) = d.and_then(|d| Some((d, wrong_call(d)?))) else {
        return View {
            panel,
            ..View::default()
        };
    };
    let fast = d.fast.as_ref().map_or("nothing", |(_, value)| value);
    let brute = d.brute.as_ref().map_or("nothing", |(_, value)| value);
    let line = Line {
        line: diagnostic.line,
        kind: Kind::Differs,
        label: format!("{} = {fast}  \u{2260}  alt {brute}", d.name),
        hover: None,
    };
    View {
        panel,
        lines: vec![line],
        diagnostic: Some(diagnostic),
        ..View::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Stage;
    use crate::judge::Verdict;
    use crate::run::{Failure, Report};
    use cpl_interp::trace::Divergence;

    fn failing(stages: &[(&str, u32, &str, &str)], expected: &str) -> StressResult {
        let failure = Failure {
            case: 17,
            seed: 42,
            input: "2\n7 7\n".into(),
            described: Some("n = 2  a = [7, 7]".into()),
            expected: expected.into(),
            got: "0\n".into(),
            verdict: Verdict::Wrong("differs".into()),
            shrunk: None,
            saved: false,
            transcript: None,
            ruling: None,
            trap: None,
        };
        let stages = (stages.iter())
            .map(|&(name, line, fast, brute)| Stage {
                name: name.into(),
                line,
                fast: fast.into(),
                brute: brute.into(),
            })
            .collect();
        StressResult {
            saved: Some("d.tests/stress-17.in".into()),
            stages,
            ..StressResult::new(Report {
                cases: 17,
                failure: Some(failure),
                ..Report::default()
            })
        }
    }

    fn marks(view: &View) -> Vec<(u32, Kind, &str)> {
        (view.lines.iter())
            .map(|l| (l.line, l.kind, l.label.as_str()))
            .collect()
    }

    #[test]
    fn a_pass_is_a_status_and_the_panel_is_the_text() {
        let passed = StressResult::new(Report {
            cases: 1000,
            ..Report::default()
        });
        let view = view(&passed, "  1000 cases agree\n".into());
        assert_eq!(view.panel, "  1000 cases agree\n");
        let status = view.status.unwrap();
        assert_eq!(
            (status.line, status.ok, status.label.as_str()),
            (0, true, "stress: 1000 cases passed")
        );
        assert!(view.lines.is_empty());
    }

    #[test]
    fn stages_mark_the_first_difference_and_the_rest_agree() {
        let result = failing(
            &[
                ("last_seen", 9, "[none, 1]", "[none, 1]"),
                ("answer", 14, "[0]", "[1]"),
                ("later", 20, "[5]", "[6]"),
                ("unplaced", 0, "1", "1"),
            ],
            "1\n",
        );
        let view = view(&result, String::new());
        assert_eq!(
            marks(&view),
            [
                (8, Kind::Agrees, "fast = brute = [none, 1]"),
                (
                    13,
                    Kind::Differs,
                    "fast [0]  \u{2260}  brute [1]  \u{2190} first difference"
                ),
                (19, Kind::Agrees, "fast [5]  \u{2260}  brute [6]"),
                (0, Kind::Input, "counterexample: n = 2  a = [7, 7]"),
            ]
        );
        assert_eq!(
            view.lines[1].hover.as_deref(),
            Some("`answer` on the counterexample:\n\n```cp\nn = 2  a = [7, 7]\nfast  = [0]\nbrute = [1]\n```")
        );
        assert!(view.status.is_none());
        // A stage on line 0 keeps it: the input mark stays off.
        let first_line = failing(&[("n", 1, "2", "2")], "1\n");
        assert_eq!(marks(&super::view(&first_line, String::new())).len(), 1);
    }

    #[test]
    fn a_whole_program_diff_shows_both_answers_on_line_0() {
        let view = view(&failing(&[], "1\n"), String::new());
        assert_eq!(
            marks(&view),
            [(
                0,
                Kind::Input,
                "counterexample: n = 2  a = [7, 7]  (fast 0 \u{2260} brute 1)"
            )]
        );
        let hover = view.lines[0].hover.as_deref().unwrap();
        assert!(
            hover.contains(" Saved as `d.tests/stress-17.in`."),
            "{hover}"
        );
        assert!(
            hover.ends_with("```cp\n2\n7 7\nfast  = 0\nbrute = 1\n```"),
            "{hover}"
        );
        // A checker's verdict has no answer to show: the verdict instead.
        let checked = super::view(&failing(&[], ""), String::new());
        assert_eq!(
            checked.lines[0].label,
            "counterexample (WA): n = 2  a = [7, 7]"
        );
        assert!(!checked.lines[0]
            .hover
            .as_deref()
            .unwrap()
            .contains("brute ="));
        // With no names for it, the raw input; with no input at all, says so.
        let mut raw = failing(&[], "1\n");
        raw.report.failure.as_mut().unwrap().described = None;
        assert!(super::view(&raw, String::new()).lines[0]
            .label
            .starts_with("counterexample: 2\n7 7  (fast"));
        raw.report.failure.as_mut().unwrap().input = String::new();
        assert!(super::view(&raw, String::new()).lines[0]
            .label
            .starts_with("counterexample: (empty input)"));
    }

    #[test]
    fn the_first_divergence_marks_the_solutions_line() {
        let divergence = |call, nth, brute: Option<(u32, String)>| Divergence {
            name: if call { "tri(k: 1)".into() } else { "s".into() },
            nth,
            fast: Some((4, "0".into())),
            brute,
            inside: vec![(5, "k: 1, s: 0".into())],
            blame: vec![],
            call,
            stuck: false,
        };
        let mut result = failing(&[], "1\n");
        result.explained = Some(Ok(Some(divergence(true, 1, Some((10, "1".into()))))));
        let view = view(&result, String::new());
        assert_eq!(view.lines[0].line, 3);
        assert_eq!(
            view.lines[0].label,
            "tri(k: 1) = 0  \u{2260}  brute 1  \u{2190} first wrong call"
        );
        assert_eq!(
            view.lines[0].hover.as_deref(),
            Some(
                "first wrong call: `tri(k: 1)`, the 1st call of `tri`\n\n```cp\nn = 2  a = [7, 7]\nfast  = 0\nbrute = 1\nline 5: k: 1, s: 0\n```"
            )
        );
        for (nth, said) in [
            (2, "2nd"),
            (3, "3rd"),
            (11, "11th"),
            (12, "12th"),
            (13, "13th"),
            (21, "21st"),
            (102, "102nd"),
            (111, "111th"),
        ] {
            result.explained = Some(Ok(Some(divergence(false, nth, None))));
            let view = super::view(&result, String::new());
            assert_eq!(
                view.lines[0].label,
                "s = 0  \u{2260}  brute nothing  \u{2190} first differing value"
            );
            let hover = view.lines[0].hover.as_deref().unwrap();
            assert!(
                hover.starts_with(&format!("first differing value: `s`, its {said} value")),
                "{hover}"
            );
        }
        // A stage already on that line keeps it.
        let mut staged = failing(&[("t", 4, "0", "1")], "1\n");
        staged.explained = Some(Ok(Some(divergence(true, 1, None))));
        let kinds: Vec<_> = super::view(&staged, String::new())
            .lines
            .iter()
            .map(|l| (l.line, l.kind))
            .collect();
        assert_eq!(kinds, [(3, Kind::Differs), (0, Kind::Input)]);
        // No divergence found, or the run explained nothing: no mark.
        staged.explained = Some(Ok(None));
        assert_eq!(super::view(&staged, String::new()).lines.len(), 2);
    }

    #[test]
    fn the_report_carries_the_view_only_when_the_cli_gives_a_panel() {
        let mut result = failing(&[], "1\n");
        assert!(result.to_json().get("view").is_none());
        result.panel = Some("  FAIL on case 17\n".into());
        let json = result.to_json();
        assert_eq!(
            json.at(&["view", "panel"]).as_str(),
            Some("  FAIL on case 17\n")
        );
        assert_eq!(
            json.at(&["view", "lines"]).as_array().map(<[_]>::len),
            Some(1)
        );
        assert!(result.render().ends_with("\"view\":{\"lines\":[{\"hover\":\"The smallest input `cpl stress` found where the fast code and the reference disagree. Saved as `d.tests/stress-17.in`.\\n\\n```cp\\n2\\n7 7\\nfast  = 0\\nbrute = 1\\n```\",\"kind\":\"input\",\"label\":\"counterexample: n = 2  a = [7, 7]  (fast 0 \u{2260} brute 1)\",\"line\":0}],\"panel\":\"  FAIL on case 17\\n\"}}\n"), "{}", result.render());
    }
}
