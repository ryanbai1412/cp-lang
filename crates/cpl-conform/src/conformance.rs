use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cpl_check::Database;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

/// What checks and runs a case: the linked compiler, or, when
/// `CPL_CONFORM_CLI` names a `cpl` binary, that binary. The second lets
/// HEAD's runner and cases measure an older compiler
/// (`scripts/conformance-bugcatch.py`). `CPL_CONFORM_RELEASE=1` runs
/// cases as `cpl run --release`, the C++ a judge runs, so a backend bug
/// shows up too.
enum Backend<'a> {
    Linked(&'a mut Database),
    Cli(PathBuf),
}

struct Ran {
    stdout: Vec<u8>,
    trapped: bool,
    trap_rendered: String,
}

impl Backend<'_> {
    fn check(&mut self, path: &str, text: &str) -> (String, bool) {
        match self {
            Backend::Linked(db) => runner::check_spec(db, path, text),
            Backend::Cli(cpl) => match cli(cpl, &["check", path], &[]) {
                Ok((code, out, err)) => (out + &err, code != 0),
                Err(e) => (format!("error[cli]: {e}"), true),
            },
        }
    }

    fn lint(&mut self, path: &str, text: &str) -> String {
        match self {
            Backend::Linked(db) => runner::lint_spec(db, path, text),
            Backend::Cli(cpl) => cli(cpl, &["lint", path], &[])
                .map(|(_, out, err)| out + &err)
                .unwrap_or_default(),
        }
    }

    /// `Err` holds the compile diagnostics, or says the run timed out.
    fn run(&mut self, path: &str, text: &str, input: &[u8]) -> Result<Ran, String> {
        match self {
            Backend::Linked(db) => {
                let compiled = runner::compile_program(db, path, text)
                    .map_err(|CompileError::Diagnostics(d)| d)?;
                let run = runner::run_interp_within(
                    &compiled.program,
                    path,
                    text,
                    input,
                    case_limit(text),
                )?;
                Ok(Ran {
                    stdout: run.stdout,
                    trapped: run.trap.is_some(),
                    trap_rendered: run.trap_rendered,
                })
            }
            Backend::Cli(cpl) => {
                let mut args = vec!["run", path];
                if release() {
                    args.push("--release");
                }
                let (code, out, err) = cli_within(cpl, &args, input, case_limit(text))?;
                Ok(Ran {
                    stdout: out.into_bytes(),
                    trapped: code != 0,
                    trap_rendered: err,
                })
            }
        }
    }
}

/// A program that traps in a checked build is invalid, and release runs no
/// checks (R09.2.2), so a release run has no trap to compare.
fn release_trap_outcome(release: bool) -> Option<CaseOutcome> {
    release.then(|| CaseOutcome {
        status: "skip".into(),
        detail: "release runs no checks (R09.2.2)".into(),
    })
}

fn release() -> bool {
    std::env::var_os("CPL_CONFORM_RELEASE").is_some_and(|v| v == "1")
}

/// The release build needs a `cpl` binary; without `CPL_CONFORM_CLI` it is
/// the one built next to this runner.
fn cli_override() -> Option<PathBuf> {
    std::env::var_os("CPL_CONFORM_CLI")
        .map(PathBuf::from)
        .or_else(|| {
            release()
                .then(|| {
                    std::env::current_exe()
                        .ok()
                        .map(|e| e.with_file_name("cpl"))
                })
                .flatten()
        })
}

/// Runs `cpl`, killed after `CPL_CONFORM_CASE_TIMEOUT` seconds (default
/// 20): an old compiler can hang on a case a later fix made finish.
fn cli(cpl: &Path, args: &[&str], input: &[u8]) -> Result<(i32, String, String), String> {
    cli_within(cpl, args, input, runner::case_timeout())
}

/// Seconds a case's run may take: `case_timeout()`, or its `// timeout: <secs>`
/// header when that is longer (a trap case that has to exhaust a budget).
fn case_limit(text: &str) -> u64 {
    header_values(text, "timeout:")
        .iter()
        .filter_map(|v| v.parse().ok())
        .fold(runner::case_timeout(), u64::max)
}

fn cli_within(
    cpl: &Path,
    args: &[&str],
    input: &[u8],
    limit: u64,
) -> Result<(i32, String, String), String> {
    let mut cmd = std::process::Command::new(cpl);
    cmd.args(args);
    let out = runner::run_piped(cmd, input, limit)
        .map_err(|e| format!("`cpl {}` {e}", args.join(" ")))?;
    Ok((
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expect {
    Ok,
    OkOutput(String),
    Diag { code: String, line: usize },
    Trap { code: String, line: usize },
    Exit(i32),
}

#[derive(Clone, Debug)]
pub struct ConformanceCase {
    pub id: String,
    pub path: PathBuf,
}

/// Standard input from an optional `// input: "..."` header line, escaped like `output "..."`.
pub fn header_input(text: &str) -> Vec<u8> {
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("//") else {
            break;
        };
        let Some(quoted) = rest.trim_start().strip_prefix("input:") else {
            continue;
        };
        let quoted = quoted.trim();
        let body = quoted
            .strip_prefix('"')
            .and_then(|q| q.strip_suffix('"'))
            .unwrap_or(quoted);
        return unescape_python_like(body).into_bytes();
    }
    Vec::new()
}

/// Every `// <key> ...` header value, in order.
fn header_values<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim_start().strip_prefix("//") else {
            break;
        };
        if let Some(v) = rest.trim_start().strip_prefix(key) {
            out.push(v.trim());
        }
    }
    out
}

fn unquote(v: &str) -> String {
    let body = v
        .strip_prefix('"')
        .and_then(|q| q.strip_suffix('"'))
        .unwrap_or(v);
    unescape_python_like(body)
}

pub(crate) struct ToolRun {
    pub(crate) status: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    /// The case file after the tool ran, for tools that rewrite it (`fix`, `fmt`).
    pub(crate) rewritten: String,
}

/// Runs `cpl <sub> <copy of the case> <rest>` on a copy in a scratch directory
/// (a `{case}` or `{dir}` word in `<rest>` places the copy or the directory instead),
/// so a tool that rewrites its input (`fmt`) never edits the case. The copy
/// blanks the `// has:`/`// lacks:`/`// rewritten:` lines so a tool that echoes the source
/// (`build`) can't match them.
fn run_tool(case: &ConformanceCase, text: &str, args: &str) -> Result<ToolRun, String> {
    use std::process::Command;
    let cpl = match cli_override() {
        Some(cpl) => cpl,
        None => std::env::current_exe()
            .map_err(|e| format!("cannot find cpl-conform: {e}"))?
            .with_file_name(if cfg!(windows) { "cpl.exe" } else { "cpl" }),
    };
    if !cpl.exists() {
        return Err(format!("no `cpl` next to cpl-conform ({})", cpl.display()));
    }
    let dir = std::env::temp_dir().join(format!(
        "cpl-conform-tool-{}-{}",
        std::process::id(),
        case.id.replace(['/', '\\', '.'], "_")
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    let name = case.path.file_name().unwrap_or_default();
    let copy = dir.join(name);
    let shown: String = text
        .split_inclusive('\n')
        .map(|l| {
            let rest = l.trim_start().strip_prefix("//").map(str::trim_start);
            match rest {
                Some(r)
                    if r.starts_with("has:")
                        || r.starts_with("lacks:")
                        || r.starts_with("rewritten:") =>
                {
                    if l.ends_with('\n') {
                        "//\n"
                    } else {
                        "//"
                    }
                }
                _ => l,
            }
        })
        .collect();
    std::fs::write(&copy, &shown).map_err(|e| format!("cannot write {}: {e}", copy.display()))?;
    for spec in header_values(text, "file:") {
        let (rel, body) = spec
            .split_once(char::is_whitespace)
            .ok_or("`// file:` needs a path and a quoted body")?;
        if rel.starts_with('/') || rel.split('/').any(|p| p == "..") {
            return Err(format!(
                "`// file: {rel}` must stay inside the scratch directory"
            ));
        }
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot make {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, unquote(body.trim()))
            .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    }
    let mut words = args.split_whitespace();
    let sub = words.next().ok_or("empty `// tool:` header")?;
    if sub == "lsp" {
        let args: Vec<&str> = words.collect();
        let run = crate::lsp::session(&cpl, &copy, &shown, &args);
        let _ = std::fs::remove_dir_all(&dir);
        return run;
    }
    let rest: Vec<String> = words.map(str::to_string).collect();
    let placed = rest
        .iter()
        .any(|w| w.contains("{case}") || w.contains("{dir}"));
    let mut argv = vec![sub.to_string()];
    if !placed {
        argv.push(copy.to_string_lossy().into_owned());
    }
    argv.extend(rest.iter().map(|w| {
        w.replace("{case}", &copy.to_string_lossy())
            .replace("{dir}", &dir.to_string_lossy())
    }));
    let mut cmd = Command::new(&cpl);
    cmd.args(&argv).current_dir(&dir);
    let out = runner::run_piped(cmd, &header_input(text), runner::case_timeout())
        .map_err(|e| format!("cpl {e}"))?;
    let rewritten = std::fs::read_to_string(&copy).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&dir);
    Ok(ToolRun {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        rewritten,
    })
}

/// A `// tool:` case: runs the tool twice (tool output is deterministic),
/// then checks the expectation and every `// has:` / `// lacks:` string.
fn run_tool_case(
    case: &ConformanceCase,
    text: &str,
    args: &str,
    expect: Expect,
    offset: usize,
) -> CaseOutcome {
    let fail = |detail: String| CaseOutcome {
        status: "fail".into(),
        detail,
    };
    let (first, second) = match (run_tool(case, text, args), run_tool(case, text, args)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => return fail(e),
    };
    if first.status != second.status || first.stdout != second.stdout {
        return fail(format!("`cpl {args}` gave different output on two runs"));
    }
    let all = format!("{}{}", first.stdout, first.stderr);
    let got = || runner::truncate_chars(all.trim(), 200);
    let met = match &expect {
        Expect::Ok => first.status == 0,
        Expect::OkOutput(want) => first.status == 0 && first.stdout.trim_end_matches('\n') == want,
        Expect::Diag { code, line } => diag_lines(&all, code).any(|l| l == line + offset),
        Expect::Trap { code, line } => {
            all.contains(code.as_str()) && rendered_at(&all, line + offset)
        }
        Expect::Exit(code) => first.status == *code,
    };
    if !met {
        return fail(format!(
            "wanted {expect:?} (exit {}); got: {}",
            first.status,
            got()
        ));
    }
    for want in header_values(text, "has:") {
        let want = unquote(want);
        if !all.contains(&want) {
            return fail(format!("wanted output with {want:?}; got: {}", got()));
        }
    }
    for bad in header_values(text, "lacks:") {
        let bad = unquote(bad);
        if all.contains(&bad) {
            return fail(format!("wanted output without {bad:?}; got: {}", got()));
        }
    }
    for want in header_values(text, "rewritten:") {
        let want = unquote(want);
        if !first.rewritten.contains(&want) {
            let file = runner::truncate_chars(first.rewritten.trim(), 200);
            return fail(format!(
                "wanted the rewritten case to have {want:?}; got: {file}"
            ));
        }
    }
    CaseOutcome {
        status: "pass".into(),
        detail: String::new(),
    }
}

/// Whether a rendered diagnostic points at `line` (`file:LINE:COL`); a bare
/// `:LINE` would also match a column.
fn rendered_at(rendered: &str, line: usize) -> bool {
    rendered.contains(&format!(":{line}:"))
}

/// Parses `// expect: ...` and computes the line offset after the header lines (`// expect:`, `// rule:`, `// input:`, `// timeout:` ...).
pub fn parse_header(text: &str) -> Result<(Expect, usize), String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let head = lines.first().copied().unwrap_or("").trim();
    let offset = lines
        .iter()
        .position(|l| {
            let trimmed = l.trim_start();
            if let Some(rest) = trimmed.strip_prefix("//") {
                let rest = rest.trim_start();
                !(rest.starts_with("expect:")
                    || rest.starts_with("rule:")
                    || rest.starts_with("input:")
                    || rest.starts_with("tool:")
                    || rest.starts_with("file:")
                    || rest.starts_with("has:")
                    || rest.starts_with("lacks:")
                    || rest.starts_with("rewritten:")
                    || rest.starts_with("timeout:"))
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

    if let Some(n) = spec.strip_prefix("exit ") {
        if let Ok(code) = n.trim().parse::<i32>() {
            return Ok((Expect::Exit(code), offset));
        }
    }
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

/// `E0601`, or the name of a library's `@lint` (`square_by_hand`, R06.4.9).
fn is_diag_code(s: &str) -> bool {
    let bytes = s.as_bytes();
    let numbered = bytes.len() == 5
        && matches!(bytes[0], b'E' | b'L' | b'R' | b'W')
        && bytes[1..].iter().all(|b| b.is_ascii_digit());
    let named = bytes.first().is_some_and(u8::is_ascii_lowercase)
        && bytes.contains(&b'_')
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_');
    numbered || named
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
    if let Some(args) = header_values(&text, "tool:").first() {
        return run_tool_case(case, &text, args, expect, offset);
    }
    let path_str = case.path.to_string_lossy();
    let mut backend = match cli_override() {
        Some(cpl) => Backend::Cli(cpl),
        None => Backend::Linked(db),
    };
    let (diag, check_failed) = backend.check(&path_str, &text);

    match expect {
        Expect::Exit(_) => CaseOutcome {
            status: "bad".into(),
            detail: "`// expect: exit N` needs a `// tool:` header".into(),
        },
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
            if let Some(skip) = release_trap_outcome(release()) {
                return skip;
            }
            let target_line = line + offset;
            let run = match backend.run(&path_str, &text, &header_input(&text)) {
                Ok(r) => r,
                Err(d) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted trap {code} at {target_line}; got: {}",
                            runner::truncate_chars(d.trim(), 200)
                        ),
                    };
                }
            };
            if run.trapped
                && run.trap_rendered.contains(&code)
                && rendered_at(&run.trap_rendered, target_line)
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
            let diag = if code.starts_with('L') || code.contains('_') {
                diag + &backend.lint(&path_str, &text)
            } else {
                diag
            };
            let target_line = line + offset;
            let unmet = header_values(&text, "has:")
                .into_iter()
                .map(unquote)
                .find(|w| !diag.contains(w.as_str()))
                .map(|w| format!("wanted output with {w:?}"))
                .or_else(|| {
                    header_values(&text, "lacks:")
                        .into_iter()
                        .map(unquote)
                        .find(|b| diag.contains(b.as_str()))
                        .map(|b| format!("wanted output without {b:?}"))
                });
            if let Some(why) = unmet {
                return CaseOutcome {
                    status: "fail".into(),
                    detail: format!("{why}; got: {}", runner::truncate_chars(diag.trim(), 200)),
                };
            }
            if diag_lines(&diag, &code).any(|l| l == target_line) {
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
                let hint = diag_line(&diag, &code)
                    .filter(|&l| l > offset && l != target_line)
                    .map(|l| {
                        format!(
                            " (it is on file line {l}: write `at {}`, counted from the first line after the headers)",
                            l - offset
                        )
                    })
                    .unwrap_or_default();
                CaseOutcome {
                    status: "fail".into(),
                    detail: format!("wanted {code} at {target_line}{hint}; got: {got}"),
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
            let run = match backend.run(&path_str, &text, &header_input(&text)) {
                Ok(r) => r,
                Err(d) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted output {want:?}; got: {}",
                            runner::truncate_chars(d.trim(), 200)
                        ),
                    };
                }
            };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_header_is_stdin_and_not_a_source_line() {
        let text = "// expect: ok, output \"1\"\n// input: \"3 4\\n\"\nprint(1)\n";
        assert_eq!(header_input(text), b"3 4\n");
        assert_eq!(parse_header(text).unwrap().1, 2);
        assert!(header_input("// expect: ok\nprint(1)\n").is_empty());
    }

    #[test]
    fn a_trap_line_is_not_matched_by_its_column() {
        let rendered = "error[R0202]: overflow\n  --> a.cp:6:7\n";
        assert!(rendered_at(rendered, 6));
        assert!(!rendered_at(rendered, 7));
    }

    #[test]
    fn a_timeout_header_only_lengthens_the_case_limit() {
        let base = runner::case_timeout();
        assert_eq!(case_limit("// expect: ok\nprint(1)\n"), base);
        assert_eq!(
            case_limit("// expect: R0601\n// timeout: 900\nf()\n"),
            base.max(900)
        );
        assert_eq!(case_limit("// timeout: 1\nf()\n"), base);
    }
}

#[cfg(test)]
mod release_tests {
    use super::*;

    #[test]
    fn release_skips_trap_cases_and_checked_runs_them() {
        assert_eq!(release_trap_outcome(true).unwrap().status, "skip");
        assert!(release_trap_outcome(false).is_none());
    }
}

/// The line of the first `--> file:L:C` after the first diagnostic with `code`.
fn diag_line(diag: &str, code: &str) -> Option<usize> {
    diag_lines(diag, code).next()
}

/// The line of each `code` diagnostic in `diag`, read off its own `-->`.
fn diag_lines(diag: &str, code: &str) -> std::vec::IntoIter<usize> {
    let tag = format!("[{code}]");
    let lines: Vec<usize> = diag
        .match_indices(&tag)
        .filter_map(|(at, _)| {
            let rest = &diag[at..];
            let arrow = rest.find("-->")?;
            // The `-->` must belong to this diagnostic, not to the next one.
            if rest[tag.len()..arrow].contains("]: ") {
                return None;
            }
            let loc = rest[arrow + 3..].lines().next()?.trim();
            let mut parts = loc.rsplitn(3, ':');
            parts.next()?;
            parts.next()?.parse().ok()
        })
        .collect();
    lines.into_iter()
}

#[cfg(test)]
mod diag_line_tests {
    #[test]
    fn finds_the_line_of_the_named_code() {
        let d = "error[E0101]: x\n  --> a.cp:2:1\nerror[E0608]: y\n  --> /t/a.cp:9:17\n";
        assert_eq!(super::diag_line(d, "E0608"), Some(9));
        assert_eq!(super::diag_line(d, "E0999"), None);
        // E0101's line is 2, not E0608's 9.
        assert!(!super::diag_lines(d, "E0101").any(|l| l == 9));
    }
}
