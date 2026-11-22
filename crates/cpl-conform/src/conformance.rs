use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cpl_check::Database;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expect {
    Ok,
    OkOutput(String),
    Diag { code: String, line: usize },
    Trap { code: String, line: usize },
}

#[derive(Clone, Debug)]
pub struct ConformanceCase {
    pub id: String,
    pub path: PathBuf,
}

/// Parses `// expect: ...` and computes the line offset after `// expect:` / `// rule:` headers.
pub fn parse_header(text: &str) -> Result<(Expect, usize), String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let head = lines.first().copied().unwrap_or("").trim();
    let offset = lines
        .iter()
        .position(|l| {
            let trimmed = l.trim_start();
            if let Some(rest) = trimmed.strip_prefix("//") {
                let rest = rest.trim_start();
                !(rest.starts_with("expect:") || rest.starts_with("rule:"))
            } else {
                true
            }
        })
        .unwrap_or(lines.len());

    let Some(after_slashes) = head.strip_prefix("//") else {
        return Err(format!("first line is not `// expect: ...`: {head}"));
    };
    let Some(spec) = after_slashes.trim_start().strip_prefix("expect:") else {
        return Err(format!("first line is not `// expect: ...`: {head}"));
    };
    let spec = spec.trim();

    if spec == "ok" {
        return Ok((Expect::Ok, offset));
    }
    if let Some(rest) = spec.strip_prefix("ok,") {
        let rest = rest.trim();
        if let Some(rest) = rest.strip_prefix("output") {
            let rest = rest.trim();
            if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                return Ok((Expect::OkOutput(unescape_python_like(inner)), offset));
            }
        }
        return Err(format!("first line is not `// expect: ...`: {head}"));
    }

    // `E0601 at 4` or `R0301 at 2 (checked)`
    let parts: Vec<&str> = spec.split_whitespace().collect();
    if parts.len() >= 3 && parts[1] == "at" {
        let code = parts[0];
        if is_diag_code(code) {
            if let Ok(line) = parts[2].parse::<usize>() {
                if parts.len() == 3
                    || (parts.len() == 4
                        && parts[3].starts_with('(')
                        && parts[3].ends_with(')')
                        && parts[3][1..parts[3].len() - 1]
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_'))
                {
                    if code.starts_with('R') {
                        return Ok((
                            Expect::Trap {
                                code: code.to_string(),
                                line,
                            },
                            offset,
                        ));
                    }
                    return Ok((
                        Expect::Diag {
                            code: code.to_string(),
                            line,
                        },
                        offset,
                    ));
                }
            }
        }
    }

    Err(format!("first line is not `// expect: ...`: {head}"))
}

fn is_diag_code(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 5
        && matches!(bytes[0], b'E' | b'L' | b'R' | b'W')
        && bytes[1..].iter().all(|b| b.is_ascii_digit())
}

fn unescape_python_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('0') => out.push('\0'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

pub fn discover(root: &Path, filter: Option<&str>) -> Vec<ConformanceCase> {
    let dir = root.join("spec").join("conformance");
    runner::walk_files(&dir, "cp")
        .into_iter()
        .filter_map(|path| {
            let rel = path.strip_prefix(&dir).ok()?;
            let id = rel.to_string_lossy().replace('\\', "/");
            if let Some(f) = filter {
                if !id.contains(f) {
                    return None;
                }
            }
            Some(ConformanceCase { id, path })
        })
        .collect()
}

pub fn run_one(db: &mut Database, case: &ConformanceCase) -> CaseOutcome {
    let text = match std::fs::read_to_string(&case.path) {
        Ok(t) => t,
        Err(e) => {
            return CaseOutcome {
                status: "bad".into(),
                detail: format!("cannot read: {e}"),
            };
        }
    };
    let (expect, offset) = match parse_header(&text) {
        Ok(h) => h,
        Err(why) => {
            return CaseOutcome {
                status: "bad".into(),
                detail: why,
            };
        }
    };
    let path_str = case.path.to_string_lossy();
    let (diag, check_failed) = runner::check_spec(db, &path_str, &text);

    match expect {
        Expect::Trap { code, line } => {
            if check_failed || diag.contains("error[") {
                return CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "wanted a run that traps; got: {}",
                        runner::truncate_chars(diag.trim(), 200)
                    ),
                };
            }
            let target_line = line + offset;
            let compiled = match runner::compile_program(db, &path_str, &text) {
                Ok(c) => c,
                Err(CompileError::Diagnostics(d)) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted trap {code} at {target_line}; got compile error: {}",
                            runner::truncate_chars(d.trim(), 200)
                        ),
                    };
                }
            };
            let run = runner::run_interp(&compiled.program, &path_str, &text, b"");
            if run.trap.is_some()
                && run.trap_rendered.contains(&code)
                && run.trap_rendered.contains(&format!(":{target_line}"))
            {
                CaseOutcome {
                    status: "pass".into(),
                    detail: String::new(),
                }
            } else {
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "wanted trap {code} at {target_line}; got: {}",
                        runner::truncate_chars(run.trap_rendered.trim(), 200)
                    ),
                }
            }
        }
        Expect::Diag { code, line } => {
            let diag = if code.starts_with('L') {
                diag + &runner::lint_spec(&path_str, &text)
            } else {
                diag
            };
            let target_line = line + offset;
            let line_needle = format!(":{target_line}:");
            if diag.contains(&code) && diag.contains(&line_needle) {
                CaseOutcome {
                    status: "pass".into(),
                    detail: String::new(),
                }
            } else {
                let got = if diag.trim().is_empty() {
                    "no diagnostic".to_string()
                } else {
                    runner::truncate_chars(diag.trim(), 200)
                };
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!("wanted {code} at {target_line}; got: {got}"),
                }
            }
        }
        Expect::Ok => {
            if check_failed || diag.contains("error[") {
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "wanted ok; got: {}",
                        runner::truncate_chars(diag.trim(), 200)
                    ),
                }
            } else {
                CaseOutcome {
                    status: "pass".into(),
                    detail: String::new(),
                }
            }
        }
        Expect::OkOutput(want) => {
            if check_failed || diag.contains("error[") {
                return CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "wanted ok; got: {}",
                        runner::truncate_chars(diag.trim(), 200)
                    ),
                };
            }
            let compiled = match runner::compile_program(db, &path_str, &text) {
                Ok(c) => c,
                Err(CompileError::Diagnostics(d)) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted output {want:?}; got compile error: {}",
                            runner::truncate_chars(d.trim(), 200)
                        ),
                    };
                }
            };
            let run = runner::run_interp(&compiled.program, &path_str, &text, b"");
            let stdout = String::from_utf8_lossy(&run.stdout);
            let actual = stdout.trim_end_matches('\n');
            if actual == want {
                CaseOutcome {
                    status: "pass".into(),
                    detail: String::new(),
                }
            } else {
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!(
                        "wanted output {want:?}; got {:?} {}",
                        runner::truncate_chars(stdout.trim(), 160),
                        runner::truncate_chars(run.trap_rendered.trim(), 160)
                    ),
                }
            }
        }
    }
}

pub fn run(root: &Path, filter: Option<&str>) -> SuiteReport {
    let cases = discover(root, filter);
    let outcomes = runner::parallel_with_db(&cases, |db, case| {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, case)))
            .unwrap_or_else(|_| CaseOutcome {
                status: "panic".into(),
                detail: "interpreter/checker panicked".into(),
            });
        (case.id.clone(), outcome)
    });
    SuiteReport::from_outcomes(outcomes)
}

pub struct RuleCoverage {
    pub defined: BTreeMap<String, String>,
    pub cited: BTreeSet<String>,
    pub missing: Vec<(String, String)>,
    pub unknown: Vec<String>,
}

/// Checks rule coverage: `(R\d\d[a-z]?(?:\.\d+)+)` in `spec/core/*.md` vs `// rule:` in `spec/conformance/**/*.cp`.
pub fn check_rules(root: &Path) -> RuleCoverage {
    let mut defined: BTreeMap<String, String> = BTreeMap::new();
    let core_dir = root.join("spec").join("core");
    for path in runner::walk_files(&core_dir, "md") {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        for (idx, line) in text.lines().enumerate() {
            for r in extract_rule_defs(line) {
                defined
                    .entry(r)
                    .or_insert_with(|| format!("{rel}:{}", idx + 1));
            }
        }
    }

    let mut cited: BTreeSet<String> = BTreeSet::new();
    let conf_dir = root.join("spec").join("conformance");
    for path in runner::walk_files(&conf_dir, "cp") {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("//") {
                if let Some(rules_str) = rest.trim_start().strip_prefix("rule:") {
                    for tok in rules_str.split_whitespace() {
                        cited.insert(tok.to_string());
                    }
                }
            }
        }
    }

    let missing: Vec<(String, String)> = defined
        .iter()
        .filter(|(r, _)| !cited.contains(*r))
        .map(|(r, loc)| (r.clone(), loc.clone()))
        .collect();
    let unknown: Vec<String> = cited
        .iter()
        .filter(|r| !defined.contains_key(*r))
        .cloned()
        .collect();

    RuleCoverage {
        defined,
        cited,
        missing,
        unknown,
    }
}

fn extract_rule_defs(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 5 < bytes.len() {
        if bytes[i] == b'(' && bytes[i + 1] == b'R' {
            let start = i + 2;
            let mut j = start;
            if j + 2 <= bytes.len() && bytes[j].is_ascii_digit() && bytes[j + 1].is_ascii_digit() {
                j += 2;
                if j < bytes.len() && bytes[j].is_ascii_lowercase() {
                    j += 1;
                }
                let mut dot_groups = 0;
                while j < bytes.len() && bytes[j] == b'.' {
                    let dstart = j + 1;
                    let mut dj = dstart;
                    while dj < bytes.len() && bytes[dj].is_ascii_digit() {
                        dj += 1;
                    }
                    if dj > dstart {
                        dot_groups += 1;
                        j = dj;
                    } else {
                        break;
                    }
                }
                if dot_groups > 0 && j < bytes.len() && bytes[j] == b')' {
                    out.push(line[start..j].to_string());
                    i = j + 1;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}
