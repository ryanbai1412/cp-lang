use std::path::{Path, PathBuf};

use cpl_check::Database;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

#[derive(Clone, Debug)]
pub struct CorpusProblem {
    pub id: String,
    pub spec_path: PathBuf,
    pub tests: Vec<(PathBuf, PathBuf)>,
}

pub fn discover(root: &Path, filter: Option<&str>) -> Vec<CorpusProblem> {
    let dir = root.join("corpus").join("problems");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let spec_path = e.path().join("spec.cp");
            if spec_path.is_file() {
                if let Some(f) = filter {
                    if !name.contains(f) {
                        return None;
                    }
                }
                Some(name)
            } else {
                None
            }
        })
        .collect();
    ids.sort();

    ids.into_iter()
        .map(|id| {
            let prob_dir = dir.join(&id);
            let spec_path = prob_dir.join("spec.cp");
            let tests_dir = prob_dir.join("tests");
            let tests = runner::walk_files(&tests_dir, "in")
                .into_iter()
                .map(|in_path| {
                    let ans_path = in_path.with_extension("ans");
                    (in_path, ans_path)
                })
                .collect();
            CorpusProblem {
                id,
                spec_path,
                tests,
            }
        })
        .collect()
}

pub fn run_one(db: &mut Database, prob: &CorpusProblem) -> CaseOutcome {
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
        let got_text = String::from_utf8_lossy(&run.stdout);
        let got_tokens = got_text.split_whitespace();
        let want_tokens = want_text.split_whitespace();
        if !got_tokens.eq(want_tokens) {
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
    SuiteReport::from_outcomes(outcomes)
}
