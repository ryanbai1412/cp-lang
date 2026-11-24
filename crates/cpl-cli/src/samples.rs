//! `cpl run <file> --samples`: the file in the debug interpreter on each
//! of its samples, each output judged against the sample's answer, and
//! what an editor shows for that laid out as a `view`.

use crate::{compile_side, render_trap, Compiled, RunArgs, Side, EXIT_FAILED, EXIT_USAGE};
use cpl_diag::view::{Status, View};
use std::io::Write;

/// What one sample did.
struct Ran {
    name: String,
    millis: u128,
    output: String,
    expected: Option<String>,
    /// The first difference from `expected`.
    differs: Option<String>,
    trap: Option<cpl_interp::Trap>,
    lives: Vec<cpl_interp::trace::Live>,
}

impl Ran {
    fn verdict(&self) -> Option<&'static str> {
        match (&self.trap, &self.differs, &self.expected) {
            (Some(_), ..) => Some("RE"),
            (None, Some(_), _) => Some("WA"),
            (None, None, Some(_)) => Some("ok"),
            (None, None, None) => None,
        }
    }

    fn failed(&self) -> bool {
        self.trap.is_some() || self.differs.is_some()
    }

    /// Its line in the summary: `i1: passed (3 ms)`.
    fn head(&self) -> String {
        let (name, ms) = (&self.name, self.millis);
        match self.verdict() {
            Some("RE") => format!("{name}: runtime error ({ms} ms)"),
            Some("WA") => format!("{name}: FAILED ({ms} ms)"),
            Some(_) => format!("{name}: passed ({ms} ms)"),
            None => format!("{name}: {ms} ms"),
        }
    }
}

pub(crate) fn run(args: &RunArgs, stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let source = cpl_contest::problem::source_path(args.file);
    let mut samples = cpl_contest::problem::samples(&source);
    if let Some(name) = args.case {
        samples.retain(|s| s.name == name);
        if samples.is_empty() {
            *stderr += &format!("cpl: {} has no sample `{name}`\n", source.display());
            return EXIT_USAGE;
        }
    }
    let compiled = match compile_side(args.file, Side::Solution, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let mut ran = Vec::new();
    if samples.is_empty() {
        ran.push(run_one(&compiled, "no input", b"", None));
    }
    for sample in &samples {
        let read = |path: &std::path::Path| {
            std::fs::read(path).map_err(|e| format!("cpl: can't read `{}`: {e}\n", path.display()))
        };
        let (input, expected) = match (
            read(&sample.input),
            sample.expected.as_deref().map(read).transpose(),
        ) {
            (Ok(input), Ok(expected)) => (input, expected),
            (Err(error), _) | (_, Err(error)) => {
                *stderr += &error;
                return EXIT_USAGE;
            }
        };
        let expected = expected.map(|e| String::from_utf8_lossy(&e).into_owned());
        ran.push(run_one(&compiled, &sample.name, &input, expected));
    }
    let live = args
        .live_sample
        .and_then(|name| ran.iter().position(|r| r.name == name));
    let view = view(args.file, &compiled.text, &ran, live.unwrap_or(0));
    let code = if ran.iter().any(Ran::failed) {
        EXIT_FAILED
    } else {
        0
    };
    if args.format != Some("json") {
        let _ = stdout.write_all(view.panel.as_bytes());
        return code;
    }
    let _ = writeln!(stdout, "{}", report(&ran, &view));
    code
}

fn run_one(compiled: &Compiled, name: &str, input: &[u8], expected: Option<String>) -> Ran {
    let started = std::time::Instant::now();
    let mut printed = Vec::new();
    let observed = match &compiled.judge {
        Some((_, judge)) => crate::interact::run_traced(
            &compiled.program,
            judge,
            input,
            &mut printed,
            crate::trace::config(&compiled.text, None),
            true,
        ),
        None => cpl_interp::run_observed(
            &compiled.program,
            cpl_interp::Io::fixed(input).with_last_values(&compiled.text),
            &mut printed,
            None,
        ),
    };
    let output = String::from_utf8_lossy(&printed).into_owned();
    let differs = match (&observed.trap, &expected) {
        (None, Some(expected)) => cpl_stress::judge::first_difference(expected, &output),
        _ => None,
    };
    Ran {
        name: name.to_string(),
        millis: started.elapsed().as_millis(),
        output,
        expected,
        differs,
        trap: observed.trap,
        lives: observed.recording.map(|r| r.lives).unwrap_or_default(),
    }
}

/// `text` indented two spaces, a line each, without its trailing blank lines.
fn indent(text: &str) -> String {
    text.trim_end()
        .lines()
        .map(|line| format!("  {line}\n"))
        .collect()
}

/// The verdict on all of them: `2 of 3 samples passed`.
fn summary(ran: &[Ran]) -> String {
    let printed = ran.iter().filter(|r| r.verdict().is_none()).count();
    let checked = ran.len() - printed;
    let passed = ran.iter().filter(|r| r.verdict() == Some("ok")).count();
    match (checked, printed) {
        (0, 1) => "it had no expected output".to_string(),
        (0, n) => format!("ran {n} samples; none has an expected output"),
        (_, 0) => format!("{passed} of {checked} samples passed"),
        (_, n) => format!("{passed} of {checked} samples passed; {n} had no expected output"),
    }
}

fn view(path: &str, text: &str, ran: &[Ran], live: usize) -> View {
    let lines = cpl_diag::LineIndex::new(text);
    let mut panel = String::new();
    let mut detail = Vec::new();
    for r in ran {
        panel += &r.head();
        panel.push('\n');
        match (&r.trap, &r.differs) {
            (Some(trap), _) => {
                panel += &indent(&render_trap(path, text, trap));
                detail.push(format!("{}: {}", r.head(), trap.message));
            }
            (None, Some(why)) => {
                panel += &indent(why);
                detail.push(format!("{}: {why}", r.head()));
            }
            (None, None) => {
                if r.expected.is_none() {
                    panel += &indent(&r.output);
                }
                detail.push(r.head());
            }
        }
    }
    let ok = !ran.iter().any(Ran::failed);
    let status = match ran {
        [one] => Status {
            line: 0,
            ok,
            label: one.head(),
            detail: Some(match (&one.trap, &one.differs) {
                (Some(trap), _) => trap.message.clone(),
                (None, Some(why)) => why.clone(),
                (None, None) if one.output.trim().is_empty() => "(empty output)".to_string(),
                (None, None) => one.output.trim_end().to_string(),
            }),
        },
        _ => {
            let summary = summary(ran);
            panel += &summary;
            panel.push('\n');
            Status {
                line: 0,
                ok,
                label: summary,
                detail: Some(detail.join("\n")),
            }
        }
    };
    // The values shown are the failing run's when one trapped, else those of `live`.
    let trapped = ran.iter().find(|r| r.trap.is_some());
    let Some(shown) = trapped.or(ran.get(live)) else {
        return View {
            panel,
            status: Some(status),
            ..View::default()
        };
    };
    let diagnostic = shown
        .trap
        .as_ref()
        .map(|trap| trap.to_diagnostic(path, &lines));
    let trap = diagnostic.as_ref().map(|d| {
        let at = lines.utf16(d.span.start, text);
        (d, at.line, at.col)
    });
    let mut view = crate::run_view::view(Some(&shown.name), &shown.lives, panel, trap);
    view.status = Some(status);
    view
}

fn report(ran: &[Ran], view: &View) -> String {
    cpl_json::write_with(|w| {
        w.object().key("samples").array();
        for r in ran {
            w.object().key("name").str(&r.name);
            match r.verdict() {
                Some(verdict) => w.key("verdict").str(verdict),
                None => w.key("verdict").null(),
            };
            w.key("time_ms").int(r.millis).key("output").str(&r.output);
            match &r.expected {
                Some(expected) => w.key("expected").str(expected),
                None => w.key("expected").null(),
            };
            let why = r
                .trap
                .as_ref()
                .map(|t| t.message.as_str())
                .or(r.differs.as_deref());
            match why {
                Some(why) => w.key("why").str(why),
                None => w.key("why").null(),
            };
            w.end();
        }
        let passed = ran.iter().filter(|r| !r.failed()).count();
        w.end().key("passed_count").int(passed);
        w.key("case_count").int(ran.len());
        view.write(w);
        w.end();
    })
}

#[cfg(test)]
mod tests {
    use cpl_json::Value;

    /// `cpl run <file> --samples --format json` in a fresh folder of `files`.
    fn samples(test: &str, files: &[(&str, &str)], extra: &[&str]) -> (u8, Value) {
        let dir = std::env::temp_dir().join(format!("cpl-samples-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (name, text) in files {
            let path = dir.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let file = dir.join("a.cp").display().to_string();
        let mut args = vec!["run", &file, "--samples", "--format", "json"];
        args.extend(extra);
        let out = crate::run(&args);
        let _ = std::fs::remove_dir_all(&dir);
        let json = cpl_json::parse(&String::from_utf8_lossy(&out.stdout)).expect(&out.stderr);
        (out.code, json)
    }

    fn at<'v>(value: &'v Value, path: &[&str]) -> &'v Value {
        path.iter()
            .fold(value, |v, key| match (v, key.parse::<usize>()) {
                (Value::Array(items), Ok(i)) => &items[i],
                (Value::Object(map), _) => &map[*key],
                _ => panic!("no {key} in {v:?}"),
            })
    }

    fn text<'v>(value: &'v Value, path: &[&str]) -> &'v str {
        match at(value, path) {
            Value::String(s) => s,
            other => panic!("{path:?} is {other:?}"),
        }
    }

    const DOUBLE: &str = "let n: Int = read()\nprint(n * 2)\n";

    #[test]
    fn each_sample_is_judged_against_its_answer() {
        let (code, json) = samples(
            "judged",
            &[
                ("a.cp", DOUBLE),
                ("i1", "1\n"),
                ("o1", "2\n"),
                ("i2", "2\n"),
                ("o2", "5\n"),
                ("i10", "3\n"),
                ("a.tests/1.in", "4\n"),
                ("a.tests/1.out", "8\n"),
            ],
            &[],
        );
        assert_eq!(code, crate::EXIT_FAILED);
        let names: Vec<&str> = (0..4)
            .map(|i| text(&json, &["samples", &i.to_string(), "name"]))
            .collect();
        assert_eq!(names, ["i1", "i2", "i10", "1"]);
        let verdict = |i: &str| at(&json, &["samples", i, "verdict"]).clone();
        assert_eq!(verdict("0"), Value::String("ok".into()));
        assert_eq!(verdict("1"), Value::String("WA".into()));
        assert_eq!(verdict("2"), Value::Null);
        assert_eq!(
            text(&json, &["samples", "1", "why"]),
            "first difference at line 1, token 1: expected `5`, got `4`"
        );
        assert_eq!(
            text(&json, &["view", "status", "label"]),
            "2 of 3 samples passed; 1 had no expected output"
        );
        assert_eq!(at(&json, &["view", "status", "ok"]), &Value::Bool(false));
        let panel = text(&json, &["view", "panel"]);
        assert!(panel.contains("\ni2: FAILED ("), "{panel}");
        assert!(panel.contains("  first difference at line 1"), "{panel}");
        assert!(
            panel.contains("\ni10: ") && panel.contains("\n  6\n"),
            "{panel}"
        );
        assert!(
            panel.ends_with("2 of 3 samples passed; 1 had no expected output\n"),
            "{panel}"
        );
    }

    #[test]
    fn one_sample_is_the_badge_and_its_values_are_live_lines() {
        let (code, json) = samples(
            "one",
            &[
                ("a.cp", DOUBLE),
                ("i1", "1\n"),
                ("o1", "2\n"),
                ("i2", "7\n"),
            ],
            &["--case", "i1"],
        );
        assert_eq!(code, 0);
        assert_eq!(at(&json, &["case_count"]), &Value::Int(1));
        assert!(text(&json, &["view", "status", "label"]).starts_with("i1: passed ("));
        assert_eq!(text(&json, &["view", "status", "detail"]), "2");
        assert_eq!(text(&json, &["view", "lines", "0", "kind"]), "live");
        assert_eq!(at(&json, &["view", "lines", "0", "line"]), &Value::Int(0));
        assert_eq!(text(&json, &["view", "lines", "0", "label"]), "i1: 1");
    }

    #[test]
    fn a_runtime_error_is_an_error_line_and_the_diagnostic() {
        let (code, json) = samples(
            "trap",
            &[
                (
                    "a.cp",
                    "let xs = [1, 2]\nlet n: Int = read()\nprint(xs[n])\n",
                ),
                ("i1", "0\n"),
                ("i2", "5\n"),
            ],
            &["--values-of", "i2"],
        );
        assert_eq!(code, crate::EXIT_FAILED);
        assert_eq!(text(&json, &["samples", "1", "verdict"]), "RE");
        let diagnostic = at(&json, &["view", "diagnostic"]);
        assert_eq!(at(diagnostic, &["line"]), &Value::Int(2));
        assert_eq!(at(diagnostic, &["col"]), &Value::Int(6));
        assert_eq!(text(diagnostic, &["code"]), "R0101");
        assert_eq!(text(diagnostic, &["source"]), "cpl (i2)");
        let Value::Array(lines) = at(&json, &["view", "lines"]) else {
            panic!()
        };
        let error = lines
            .iter()
            .find(|l| text(l, &["kind"]) == "error")
            .unwrap();
        assert_eq!(at(error, &["line"]), &Value::Int(2));
        assert!(text(error, &["label"]).starts_with("i2 error: [R0101] index 5"));
        assert!(lines.iter().any(|l| text(l, &["label"]) == "i2: 5"));
        assert!(text(&json, &["view", "panel"]).contains("\ni2: runtime error ("));
        assert!(text(&json, &["view", "panel"]).contains("\n  error[R0101]"));
    }

    #[test]
    fn a_file_without_samples_runs_once_on_no_input() {
        let (code, json) = samples("none", &[("a.cp", "print(3)\n")], &[]);
        assert_eq!(code, 0);
        assert_eq!(text(&json, &["samples", "0", "name"]), "no input");
        assert!(text(&json, &["view", "status", "label"]).starts_with("no input: "));
        assert_eq!(text(&json, &["view", "status", "detail"]), "3");
    }
}
