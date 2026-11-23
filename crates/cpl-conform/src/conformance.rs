use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cpl_check::Database;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

/// What checks and runs a case: the linked compiler, or, when
/// `CPL_CONFORM_CLI` names a `cpl` binary, that binary. The second lets
/// HEAD's runner and cases measure an older compiler
/// (`scripts/conformance-bugcatch.py`).
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
            Backend::Linked(_) => runner::lint_spec(path, text),
            Backend::Cli(cpl) => cli(cpl, &["lint", path], &[])
                .map(|(_, out, err)| out + &err)
                .unwrap_or_default(),
        }
    }

    /// `Err` holds the compile diagnostics.
    fn run(&mut self, path: &str, text: &str, input: &[u8]) -> Result<Ran, String> {
        match self {
            Backend::Linked(db) => {
                let compiled = runner::compile_program(db, path, text)
                    .map_err(|CompileError::Diagnostics(d)| d)?;
                let run = runner::run_interp_bounded(&compiled.program, path, text, input)?;
                Ok(Ran {
                    stdout: run.stdout,
                    trapped: run.trap.is_some(),
                    trap_rendered: run.trap_rendered,
                })
            }
            Backend::Cli(cpl) => {
                let (code, out, err) = cli(cpl, &["run", path], input)?;
                Ok(Ran {
                    stdout: out.into_bytes(),
                    trapped: code != 0,
                    trap_rendered: err,
                })
            }
        }
    }
}

fn cli_override() -> Option<PathBuf> {
    std::env::var_os("CPL_CONFORM_CLI").map(PathBuf::from)
}

/// Runs `cpl`, killed after `CPL_CONFORM_CASE_TIMEOUT` seconds (default
/// 20): an old compiler can hang on a case a later fix made finish.
fn cli(cpl: &Path, args: &[&str], input: &[u8]) -> Result<(i32, String, String), String> {
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    let limit = runner::case_timeout();
    let mut child = Command::new(cpl)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run {}: {e}", cpl.display()))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(input);
    }
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            String::from_utf8_lossy(&buf).into_owned()
        })
    };
    let out = drain(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = drain(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let deadline = Instant::now() + Duration::from_secs(limit);
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("cpl failed: {e}"))? {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("`cpl {}` timed out after {limit}s", args.join(" ")));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    Ok((
        status.code().unwrap_or(-1),
        out.join().unwrap_or_default(),
        err.join().unwrap_or_default(),
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

struct ToolRun {
    status: i32,
    stdout: String,
    stderr: String,
}

/// Runs `cpl <sub> <copy of the case> <rest>` on a copy in a scratch directory,
/// so a tool that rewrites its input (`fmt`) never edits the case. The copy
/// blanks the `// has:`/`// lacks:` lines so a tool that echoes the source
/// (`build`) can't match them.
fn run_tool(case: &ConformanceCase, text: &str, args: &str) -> Result<ToolRun, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
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
                Some(r) if r.starts_with("has:") || r.starts_with("lacks:") => {
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
    std::fs::write(&copy, shown).map_err(|e| format!("cannot write {}: {e}", copy.display()))?;
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
    let mut child = Command::new(&cpl)
        .arg(sub)
        .arg(&copy)
        .args(words)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run cpl: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&header_input(text));
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("cpl failed: {e}"))?;
    let _ = std::fs::remove_dir_all(&dir);
    Ok(ToolRun {
        status: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
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
        Expect::Diag { code, line } => {
            all.contains(code.as_str()) && all.contains(&format!(":{}:", line + offset))
        }
        Expect::Trap { code, line } => {
            all.contains(code.as_str()) && all.contains(&format!(":{}", line + offset))
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
    CaseOutcome {
        status: "pass".into(),
        detail: String::new(),
    }
}

/// Parses `// expect: ...` and computes the line offset after the `// expect:`, `// rule:` and `// input:` headers.
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
                    || rest.starts_with("lacks:"))
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
            let target_line = line + offset;
            let run = match backend.run(&path_str, &text, &header_input(&text)) {
                Ok(r) => r,
                Err(d) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted trap {code} at {target_line}; got compile error: {}",
                            runner::truncate_chars(d.trim(), 200)
                        ),
                    };
                }
            };
            if run.trapped
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
                diag + &backend.lint(&path_str, &text)
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
            let run = match backend.run(&path_str, &text, &header_input(&text)) {
                Ok(r) => r,
                Err(d) => {
                    return CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "wanted output {want:?}; got compile error: {}",
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
}
