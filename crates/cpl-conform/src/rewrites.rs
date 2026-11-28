//! Rewrites keep behaviour: every corpus program with tests, rewritten by
//! `cpl fix` (`#fix`) and by `cpl fmt` (`#fmt`), prints exactly what the
//! original printed on every test input
//! (`handbook/guidelines/lints-checklist.md`, `lints/R11` and `lints/R22`).
//! A fix also keeps a formatted file formatted (`lints/R10`). A rewrite that
//! changes nothing is `untested`.

use std::path::Path;

use cpl_facade::{Job, Outcome, Program, Request, RunInput, Session, Stop};

use crate::corpus::{discover_file, CorpusProblem};
use crate::runner;
use crate::scoreboard::{CaseOutcome, SuiteReport};

fn outcome(status: &str, detail: impl Into<String>) -> CaseOutcome {
    CaseOutcome {
        status: status.into(),
        detail: detail.into(),
    }
}

/// What `cpl fix` writes: each error's fix, then, once none is left, the
/// lint fixes.
fn fixed(db: &mut Session, path: &str, text: &str) -> String {
    let (fixed, _, left) = db.apply_error_fixes_in(path, text);
    if !left.is_empty() {
        return fixed;
    }
    let (config, _) = cpl_lint::Config::discover(Path::new(path));
    let (fixed, _) = cpl_lint::fix_in(db, path, &fixed, &config, cpl_facade::Layer::Built);
    *db = db.without_file(path);
    fixed
}

fn rewrite(db: &mut Session, how: &str, path: &str, text: &str) -> String {
    match how {
        "fix" => fixed(db, path, text),
        _ => cpl_fmt::format(text).unwrap_or_else(|_| text.to_string()),
    }
}

/// `text` at `path` as `cpl run` builds it, or what stops it, rendered.
fn built(db: &Session, path: &str, text: &str) -> Result<Program, String> {
    db.with_file(path, text)
        .program(path, &Job::program())
        .map_err(|stopped| cpl_diag::render_all(&stopped, path, text))
}

/// `program` on `input` as `cpl run` runs it, `Err` once it runs past the
/// case's time.
fn ran(db: &Session, program: &Program, input: &[u8]) -> Result<(Vec<u8>, Outcome), String> {
    let limit = runner::case_timeout();
    let stop = Stop::after(std::time::Duration::from_secs(limit));
    let mut printed = Vec::new();
    let input = RunInput::fixed(input)
        .until(stop)
        .at(cpl_facade::Checks::LOCAL);
    let outcome = db.run(program, input, &mut printed, Request::default());
    match outcome.out_of_time() {
        true => Err(format!("the interpreter timed out after {limit}s")),
        false => Ok((printed, outcome)),
    }
}

pub fn run_one(db: &mut Session, prob: &CorpusProblem, how: &str) -> CaseOutcome {
    let Ok(text) = std::fs::read_to_string(&prob.spec_path) else {
        return outcome("error", "cannot read the program");
    };
    let path = prob.spec_path.to_string_lossy().into_owned();
    let after = rewrite(db, how, &path, &text);
    if after == text {
        return outcome("untested", format!("`cpl {how}` changes nothing"));
    }
    let formatted = |s: &str| cpl_fmt::format(s).ok().as_deref() == Some(s);
    if how == "fix" && formatted(&text) && !formatted(&after) {
        return outcome("wrong", "`cpl fix` leaves a formatted file unformatted");
    }
    let Ok(before) = built(db, &path, &text) else {
        return outcome("untested", "the original does not build");
    };
    let rewritten = match built(db, &path, &after) {
        Ok(program) => program,
        Err(stopped) => {
            let first = runner::first_line_trimmed(&stopped, 160);
            return outcome("error", format!("after `cpl {how}`: {first}"));
        }
    };
    for (in_path, _) in &prob.tests {
        let name = in_path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Ok(input) = std::fs::read(in_path) else {
            return outcome("error", format!("cannot read {name}"));
        };
        let Ok((want, original)) = ran(db, &before, &input) else {
            continue;
        };
        let (got, run) = match ran(db, &rewritten, &input) {
            Ok(got) => got,
            Err(detail) => return outcome("error", format!("{name}, after `cpl {how}`: {detail}")),
        };
        if let (Some(trap), None) = (&run.trap, &original.trap) {
            let lines = cpl_diag::LineIndex::new(&after);
            let rendered =
                cpl_diag::render(&trap.to_diagnostic(&path, &lines), &path, &after, &lines);
            let first = runner::first_line_trimmed(&rendered, 160);
            return outcome("trap", format!("{name}, after `cpl {how}`: {first}"));
        }
        if got != want || run.trap.is_none() && original.trap.is_some() {
            return outcome("wrong", format!("{name}: `cpl {how}` changes the output"));
        }
    }
    outcome("pass", "")
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let mut cases = Vec::new();
    for file in ["spec.cp", "port.cp"] {
        for prob in discover_file(root, filter, file)
            .into_iter()
            .filter(|p| !p.tests.is_empty())
        {
            for how in ["fix", "fmt"] {
                let mut prob = prob.clone();
                prob.id = format!("{}/{file}#{how}", prob.id);
                cases.push((prob, how));
            }
        }
    }
    let outcomes = runner::parallel_with_db(&cases, |db: &mut Session, (prob, how)| {
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, prob, how)))
                .unwrap_or_else(|_| outcome("panic", "a rewrite or the interpreter panicked"));
        (prob.id.clone(), outcome)
    });
    SuiteReport::from_outcomes(outcomes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A case is fixed, built and run in the one session its worker holds,
    /// which it leaves as it found it: the next case sees none of its text.
    #[test]
    fn a_case_is_rewritten_built_and_run_in_one_session_it_leaves_empty() {
        let dir = std::env::temp_dir().join(format!("cpl-rewrites-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = |name: &str, text: &str| {
            let path = dir.join(name);
            std::fs::write(&path, text).unwrap();
            path
        };
        let prob = CorpusProblem {
            id: "sum".into(),
            spec_path: file("spec.cp", "let  n: Int = read()\nprint(n+1)\n"),
            tests: vec![(file("1.in", "41\n"), file("1.ans", "42\n"))],
            checker: None,
            max_tests: Vec::new(),
            interactor: None,
            make_error: None,
        };
        let mut db = Session::default();
        for how in ["fmt", "fix", "fmt"] {
            let got = run_one(&mut db, &prob, how);
            let want = match how {
                "fmt" => "pass",
                _ => "untested",
            };
            assert_eq!(got.status, want, "{how}: {}", got.detail);
            assert_eq!(db.snapshot().files().count(), 0, "after {how}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
