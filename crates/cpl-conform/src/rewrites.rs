//! Rewrites keep behaviour: every corpus program with tests, rewritten by
//! `cpl fix` (`#fix`) and by `cpl fmt` (`#fmt`), prints exactly what the
//! original printed on every test input
//! (`handbook/guidelines/lints-checklist.md`, `lints/R11` and `lints/R22`).
//! A fix also keeps a formatted file formatted (`lints/R10`). A rewrite that
//! changes nothing is `untested`.

use std::path::Path;

use cpl_check::Database;

use crate::corpus::{discover_file, CorpusProblem};
use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

fn outcome(status: &str, detail: impl Into<String>) -> CaseOutcome {
    CaseOutcome {
        status: status.into(),
        detail: detail.into(),
    }
}

/// What `cpl fix` writes: each error's fix, then, once none is left, the
/// lint fixes.
fn fixed(db: &mut Database, path: &str, text: &str) -> String {
    let (fixed, _, left) = cpl_ir::fix::apply_error_fixes_in(db, path, text);
    if !left.is_empty() {
        return fixed;
    }
    let (config, _) = cpl_lint::Config::discover(Path::new(path));
    cpl_lint::fix_in(db, path, &fixed, &config).0
}

fn rewrite(db: &mut Database, how: &str, path: &str, text: &str) -> String {
    match how {
        "fix" => fixed(db, path, text),
        _ => cpl_fmt::format(text).unwrap_or_else(|_| text.to_string()),
    }
}

pub fn run_one(db: &mut Database, prob: &CorpusProblem, how: &str) -> CaseOutcome {
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
    let Ok(before) = runner::compile_program(db, &path, &text) else {
        return outcome("untested", "the original does not build");
    };
    let rewritten = match runner::compile_program(db, &path, &after) {
        Ok(c) => c,
        Err(CompileError::Diagnostics(d)) => {
            let first = runner::first_line_trimmed(&d, 160);
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
        let Ok(want) = runner::run_interp_bounded(&before.program, &path, &text, &input) else {
            continue;
        };
        let got = match runner::run_interp_bounded(&rewritten.program, &path, &after, &input) {
            Ok(got) => got,
            Err(detail) => return outcome("error", format!("{name}, after `cpl {how}`: {detail}")),
        };
        if got.trap.is_some() && want.trap.is_none() {
            let first = runner::first_line_trimmed(&got.trap_rendered, 160);
            return outcome("trap", format!("{name}, after `cpl {how}`: {first}"));
        }
        if got.stdout != want.stdout || got.trap.is_none() && want.trap.is_some() {
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
    let outcomes = runner::parallel_with_db(&cases, |db, (prob, how)| {
        let outcome =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, prob, how)))
                .unwrap_or_else(|_| outcome("panic", "a rewrite or the interpreter panicked"));
        (prob.id.clone(), outcome)
    });
    SuiteReport::from_outcomes(outcomes)
}
