use std::path::{Path, PathBuf};
use std::time::Duration;

use cpl_check::Session;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};
use cpl_stress::{Judge, Judged, Launch, Program, Verdict};

#[derive(Clone, Debug)]
pub struct CorpusProblem {
    pub id: String,
    pub spec_path: PathBuf,
    pub tests: Vec<(PathBuf, PathBuf)>,
    /// `checker.py <in> <out> <ans>`, exit 0 accepts; judges instead of the token diff.
    pub checker: Option<PathBuf>,
    /// tests.toml's `--max` cases: judged on the C++ by `corpus.py verify`, only
    /// timed here (decision 556).
    pub max_tests: Vec<PathBuf>,
    /// `interactor.py <in>` judges each run; its cases have no `.ans`.
    pub interactor: Option<PathBuf>,
}

pub fn discover(root: &Path, filter: Option<&str>) -> Vec<CorpusProblem> {
    discover_file(root, filter, "spec.cp")
}

/// Problems with a `file` (`spec.cp`, `port.cp`) program; `spec_path` is that file.
pub fn discover_file(root: &Path, filter: Option<&str>, file: &str) -> Vec<CorpusProblem> {
    let mut ids: Vec<String> = cpl_corpus::files(root, file)
        .iter()
        .filter_map(|p| Some(p.parent()?.file_name()?.to_string_lossy().into_owned()))
        .filter(|name| filter.is_none_or(|f| name.contains(f)))
        .collect();
    ids.sort();

    ids.into_iter()
        .map(|id| {
            let prob_dir = cpl_corpus::package(root, &id);
            let spec_path = prob_dir.join(file);
            // `corpus.py make` marks each `--max` case with an `NN.max`.
            let (max_tests, tests): (Vec<PathBuf>, Vec<PathBuf>) =
                runner::walk_files(&tests_dir(root, &prob_dir, &id), "in")
                    .into_iter()
                    .partition(|p| p.with_extension("max").is_file());
            let tests = tests
                .into_iter()
                .map(|in_path| {
                    let ans_path = in_path.with_extension("ans");
                    (in_path, ans_path)
                })
                .collect();
            let interactor = Some(prob_dir.join("interactor.py")).filter(|c| c.is_file());
            let checker = Some(prob_dir.join("checker.py")).filter(|c| c.is_file());
            CorpusProblem {
                id,
                spec_path,
                tests,
                max_tests,
                checker,
                interactor,
            }
        })
        .collect()
}

/// A problem's cases: its committed `tests/`, or, for a problem whose cases are a seed list
/// (`tests.toml`), what `scripts/corpus.py make` wrote under `target/corpus-tests/<id>/`.
fn tests_dir(root: &Path, prob_dir: &Path, id: &str) -> PathBuf {
    if prob_dir.join("tests.toml").is_file() {
        root.join("target").join("corpus-tests").join(id)
    } else {
        prob_dir.join("tests")
    }
}

pub fn run_one(db: &mut Session, prob: &CorpusProblem) -> CaseOutcome {
    if prob.tests.is_empty() {
        return CaseOutcome {
            status: "untested".into(),
            detail: String::new(),
        };
    }
    let text = match std::fs::read_to_string(&prob.spec_path) {
        Ok(t) => t,
        Err(e) => {
            return CaseOutcome {
                status: "error".into(),
                detail: format!("cannot read: {e}"),
            };
        }
    };
    let path_str = prob.spec_path.to_string_lossy();
    let compiled = match runner::compile_program(db, &path_str, &text) {
        Ok(c) => c,
        Err(CompileError::Diagnostics(d)) => {
            let first = runner::first_line_trimmed(&d, 160);
            let status = if first.contains("`todo` reached") {
                "todo"
            } else if first.starts_with("error[E") {
                "error"
            } else {
                "trap"
            };
            return CaseOutcome {
                status: status.into(),
                detail: first,
            };
        }
    };

    if let Some(interactor) = &prob.interactor {
        for (in_path, _) in &prob.tests {
            let name = in_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let (status, detail) = match interact(interactor, &prob.spec_path, in_path) {
                Ok(Verdict::Accepted) => continue,
                Ok(Verdict::Wrong(_)) => ("wrong", name),
                Ok(Verdict::TimeLimit) => (
                    "error",
                    format!(
                        "the interpreter timed out after {}s",
                        runner::case_timeout()
                    ),
                ),
                Ok(v) => (
                    "trap",
                    runner::first_line_trimmed(&format!("{name}: {v:?}"), 160),
                ),
                Err(detail) => ("error", detail),
            };
            return CaseOutcome {
                status: status.into(),
                detail,
            };
        }
        return CaseOutcome {
            status: "pass".into(),
            detail: String::new(),
        };
    }

    for (in_path, ans_path) in &prob.tests {
        let input = match std::fs::read(in_path) {
            Ok(b) => b,
            Err(e) => {
                return CaseOutcome {
                    status: "error".into(),
                    detail: format!("cannot read {}: {e}", in_path.display()),
                };
            }
        };
        let want_text = match std::fs::read_to_string(ans_path) {
            Ok(t) => t,
            Err(e) => {
                return CaseOutcome {
                    status: "error".into(),
                    detail: format!("cannot read {}: {e}", ans_path.display()),
                };
            }
        };
        let run = match runner::run_interp_bounded(&compiled.program, &path_str, &text, &input) {
            Ok(run) => run,
            Err(detail) => {
                return CaseOutcome {
                    status: "error".into(),
                    detail,
                }
            }
        };
        if run.trap.is_some() {
            let first = runner::first_line_trimmed(&run.trap_rendered, 160);
            let status = if first.contains("`todo` reached") {
                "todo"
            } else if first.starts_with("error[E") {
                "error"
            } else {
                "trap"
            };
            return CaseOutcome {
                status: status.into(),
                detail: first,
            };
        }
        let accepted = match &prob.checker {
            Some(checker) => match run_checker(checker, in_path, ans_path, &run.stdout) {
                Ok(ok) => ok,
                Err(detail) => {
                    return CaseOutcome {
                        status: "error".into(),
                        detail,
                    }
                }
            },
            None => {
                let got_text = String::from_utf8_lossy(&run.stdout);
                got_text.split_whitespace().eq(want_text.split_whitespace())
            }
        };
        if !accepted {
            let name = in_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            return CaseOutcome {
                status: "wrong".into(),
                detail: name,
            };
        }
    }

    CaseOutcome {
        status: "pass".into(),
        detail: String::new(),
    }
}

/// `interactor.py <in>` talking to `cpl run spec.cp`, the way `cpl test
/// --interactor` judges a case.
fn interact(interactor: &Path, spec: &Path, in_path: &Path) -> Result<Verdict, String> {
    let cpl = crate::conformance::cli_binary()?.display().to_string();
    let spec = spec.display().to_string();
    let input = std::fs::read_to_string(in_path)
        .map_err(|e| format!("cannot read {}: {e}", in_path.display()))?;
    interacted(
        interactor,
        &Program::command(&spec, &[&cpl, "run", &spec]),
        &input,
    )
    .map(|judged| judged.verdict)
}

/// `interactor.py <in>` talking to `solution` on `input`.
pub(crate) fn interacted(
    interactor: &Path,
    solution: &Program,
    input: &str,
) -> Result<Judged, String> {
    let judge_path = interactor.display().to_string();
    let judge = Judge::Interactor(
        Program::command(&judge_path, &["python3", &judge_path]),
        Launch::declared(interactor),
    );
    judge
        .run(
            solution,
            input,
            None,
            Duration::from_secs(runner::case_timeout()),
        )
        .map_err(|e| e.to_string())
}

/// Whether the interpreter runs one of `prob`'s `--max` cases past `case_timeout()`.
fn max_over_time(db: &mut Session, prob: &CorpusProblem) -> bool {
    if let Some(interactor) = &prob.interactor {
        return prob.max_tests.iter().any(|p| {
            matches!(
                interact(interactor, &prob.spec_path, p),
                Ok(Verdict::TimeLimit)
            )
        });
    }
    let Ok(text) = std::fs::read_to_string(&prob.spec_path) else {
        return false;
    };
    let path = prob.spec_path.to_string_lossy();
    let Ok(compiled) = runner::compile_program(db, &path, &text) else {
        return false;
    };
    prob.max_tests.iter().any(|p| {
        std::fs::read(p).is_ok_and(|input| {
            runner::run_interp_bounded(&compiled.program, &path, &text, &input).is_err()
        })
    })
}

fn run_checker(checker: &Path, input: &Path, ans: &Path, out: &[u8]) -> Result<bool, String> {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out_path = std::env::temp_dir().join(format!("cpl-corpus-{}-{n}.out", std::process::id()));
    std::fs::write(&out_path, out).map_err(|e| format!("cannot write output: {e}"))?;
    let status = std::process::Command::new("python3")
        .arg(checker)
        .arg(input)
        .arg(&out_path)
        .arg(ans)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let _ = std::fs::remove_file(&out_path);
    status
        .map(|s| s.success())
        .map_err(|e| format!("cannot run {}: {e}", checker.display()))
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let problems = discover(root, filter);
    let outcomes = runner::parallel_with_db(&problems, |db, prob| {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, prob)))
            .unwrap_or_else(|_| CaseOutcome {
                status: "panic".into(),
                detail: "interpreter/checker panicked".into(),
            });
        (prob.id.clone(), outcome)
    });
    let mut report = SuiteReport::from_outcomes(outcomes);
    let timed: Vec<CorpusProblem> = problems
        .iter()
        .filter(|p| !p.max_tests.is_empty())
        .cloned()
        .collect();
    let over = runner::parallel_with_db(&timed, |db, prob| {
        let over =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| max_over_time(db, prob)));
        (prob.id.clone(), over.unwrap_or(false))
    });
    report.interp_max_over_20s = Some(
        over.into_iter()
            .filter_map(|(id, o)| o.then_some(id))
            .collect(),
    );
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_cases_are_timed_apart_and_an_interactive_case_needs_no_answer() {
        let root = std::env::temp_dir().join(format!("cpl-conform-corpus-{}", std::process::id()));
        let pkg = root.join("corpus/problems/p");
        let cases = root.join("target/corpus-tests/p");
        std::fs::create_dir_all(&pkg).unwrap();
        std::fs::create_dir_all(&cases).unwrap();
        for (dir, file) in [
            (&pkg, "spec.cp"),
            (&pkg, "tests.toml"),
            (&pkg, "interactor.py"),
            (&cases, "01.in"),
            (&cases, "02.in"),
            (&cases, "02.max"),
        ] {
            std::fs::write(dir.join(file), "").unwrap();
        }
        let found = discover(&root, None);
        let _ = std::fs::remove_dir_all(&root);
        let [p] = found.as_slice() else {
            panic!("{found:?}")
        };
        assert_eq!(p.tests, [(cases.join("01.in"), cases.join("01.ans"))]);
        assert_eq!(p.max_tests, [cases.join("02.in")]);
        assert_eq!(p.interactor, Some(pkg.join("interactor.py")));
    }
}
