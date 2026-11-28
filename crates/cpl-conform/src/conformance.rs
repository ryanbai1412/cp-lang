use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use cpl_check::Session;

use crate::runner::{self, CompileError};
use crate::scoreboard::{CaseOutcome, SuiteReport};

/// What checks and runs a case: the linked compiler, or, when
/// `CPL_CONFORM_CLI` names a `cpl` binary, that binary. The second lets
/// HEAD's runner and cases measure an older compiler
/// (`scripts/spec-gate.py bugcatch`). `CPL_CONFORM_RELEASE=1` runs
/// cases as `cpl run --release`, the C++ a judge runs, so a backend bug
/// shows up too.
enum Backend<'a> {
    Linked(&'a mut Session),
    Cli(PathBuf),
    /// A `// engines: next` case, std the frozen `cpl` lacks: the `cpl-next` beside this runner,
    /// whose interpreter and engine have to print the same.
    Next(PathBuf),
}

struct Ran {
    stdout: Vec<u8>,
    trapped: bool,
    trap_rendered: String,
}

/// cpl-next's `--json` diagnostics as `cpl check` renders them, `error[CODE]: message` then
/// `--> file:L:C`, so one reader finds a case's code and line on either pipeline.
fn next_rendered(json_lines: &str) -> String {
    let mut out = String::new();
    for line in json_lines.lines() {
        let d = cpl_json::parse(line).ok();
        let field = |k: &str| d.as_ref()?.get(k);
        let (Some(code), Some(msg)) = (field("code"), field("message")) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        let file = field("file").and_then(|f| f.as_str()).unwrap_or("program");
        let at = |k: &str| field("start")?.get(k)?.as_i64();
        let (l, c) = (at("line").unwrap_or(0), at("col").unwrap_or(0));
        let (code, msg) = (code.as_str().unwrap_or(""), msg.as_str().unwrap_or(""));
        out.push_str(&format!("error[{code}]: {msg}\n  --> {file}:{l}:{c}\n"));
    }
    out
}

impl Backend<'_> {
    fn check(&mut self, path: &str, text: &str) -> (String, bool) {
        match self {
            Backend::Linked(db) => runner::check_spec(db, path, text),
            Backend::Cli(cpl) => match cli(cpl, &["check", path], &[]) {
                Ok((code, out, err)) => (out + &err, code != 0),
                Err(e) => (format!("error[cli]: {e}"), true),
            },
            Backend::Next(next) => match cli(next, &[path, "--check", "--json"], &[]) {
                Ok((code, out, err)) => (next_rendered(&out) + &err, code != 0),
                Err(e) => (format!("error[cpl-next]: {e}"), true),
            },
        }
    }

    fn lint(&mut self, path: &str, text: &str) -> String {
        match self {
            Backend::Linked(db) => runner::lint_spec(db, path, text),
            Backend::Cli(cpl) => cli(cpl, &["lint", path], &[])
                .map(|(_, out, err)| out + &err)
                .unwrap_or_default(),
            Backend::Next(_) => String::new(),
        }
    }

    /// `Err` holds the compile diagnostics, or says the run timed out.
    fn run(&mut self, path: &str, text: &str, input: &[u8]) -> Result<Ran, String> {
        match self {
            Backend::Linked(db) => {
                let compiled = runner::compile_program(db, path, text)
                    .map_err(|CompileError::Diagnostics(d)| d)?;
                if !compiled.program.cpp_bodies.is_empty() {
                    return run_cpp_build(&compiled.program, path, text, input);
                }
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
            Backend::Next(next) => {
                let oracle = oracle_binary(next);
                let mut runs = [(oracle.as_path(), "--raw"), (next.as_path(), "--engine")]
                    .map(|(bin, form)| cli_within(bin, &[path, form], input, case_limit(text)));
                let [raw, engine] = &mut runs;
                let (code, out, err) = std::mem::replace(raw, Ok(Default::default()))?;
                if code != 0 && code != 3 {
                    return Err(out + &err);
                }
                let (_, engine_out, _) = std::mem::replace(engine, Ok(Default::default()))?;
                if engine_out != out {
                    return Err(format!(
                        "cpl-next's interpreter printed {:?}, its engine {:?}",
                        runner::truncate_chars(&out, 80),
                        runner::truncate_chars(&engine_out, 80)
                    ));
                }
                Ok(Ran {
                    stdout: out.into_bytes(),
                    trapped: code == 3,
                    trap_rendered: err,
                })
            }
        }
    }
}

/// Whether the case runs on cpl-next (`// engines: next`).
fn on_next(text: &str) -> bool {
    header_values(text, "engines:").contains(&"next")
}

/// The reference interpreter, `cpl-next-oracle`, beside `next`.
fn oracle_binary(next: &Path) -> PathBuf {
    next.with_file_name(if cfg!(windows) {
        "cpl-next-oracle.exe"
    } else {
        "cpl-next-oracle"
    })
}

/// The `cpl-next` beside cpl-conform, with the `cpl-next-oracle` it is compared against.
fn next_binary() -> Result<PathBuf, String> {
    let next = std::env::current_exe()
        .map_err(|e| format!("cannot find cpl-conform: {e}"))?
        .with_file_name(if cfg!(windows) {
            "cpl-next.exe"
        } else {
            "cpl-next"
        });
    for bin in [next.clone(), oracle_binary(&next)] {
        if !bin.exists() {
            return Err(format!(
                "a `// engines: next` case needs `cpl-next` and `cpl-next-oracle` next to cpl-conform ({}); build them: cargo build -p cpl-next",
                bin.display()
            ));
        }
        if let Some(why) = stale_cli(&bin) {
            return Err(why);
        }
    }
    Ok(next)
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
    cpl_env::CONFORM_RELEASE.is("1")
}

/// The release build needs a `cpl` binary; without `CPL_CONFORM_CLI` it is
/// the one built next to this runner.
fn cli_override() -> Option<PathBuf> {
    let snapshot = held_cli();
    cpl_env::CONFORM_CLI
        .path()
        .or_else(|| {
            release()
                .then(|| {
                    std::env::current_exe()
                        .ok()
                        .map(|e| e.with_file_name("cpl"))
                })
                .flatten()
        })
        .map(|cpl| snapshot.unwrap_or(cpl))
}

/// The run's own link to the `cpl` its cases exec, taken once by `run`: a
/// `cargo build` that relinks `target/*/cpl` mid-run (check.sh runs one
/// beside the suite) would otherwise fail every tool case in flight.
static SNAPSHOT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

fn held_cli() -> Option<PathBuf> {
    SNAPSHOT.lock().ok()?.clone()
}

fn hold_cli(cpl: Option<PathBuf>) {
    if let Ok(mut held) = SNAPSHOT.lock() {
        *held = cpl;
    }
}

/// The `cpl` a case execs: `CPL_CONFORM_CLI`, else the one beside cpl-conform.
pub(crate) fn cli_binary() -> Result<PathBuf, String> {
    if let Some(held) = held_cli() {
        return Ok(held);
    }
    let cpl = match cli_override() {
        Some(cpl) => cpl,
        None => std::env::current_exe()
            .map_err(|e| format!("cannot find cpl-conform: {e}"))?
            .with_file_name(if cfg!(windows) { "cpl.exe" } else { "cpl" }),
    };
    if !cpl.exists() {
        return Err(format!("no `cpl` next to cpl-conform ({})", cpl.display()));
    }
    static STALE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    match STALE.get_or_init(|| stale_cli(&cpl)) {
        Some(why) => Err(why.clone()),
        None => Ok(cpl),
    }
}

/// Why `cpl` is older than a file cargo built it from (its dep-info, `cpl.d`
/// beside it), if it is: `cargo run -p cpl-conform` rebuilds only this runner,
/// and a `cpl` left from another revision checks against this run's std with
/// that revision's compiler.
pub(crate) fn stale_cli(cpl: &Path) -> Option<String> {
    let linked = std::fs::metadata(cpl).and_then(|m| m.modified()).ok()?;
    let deps = std::fs::read_to_string(cpl.with_extension("d")).ok()?;
    let (_, inputs) = deps.split_once(": ")?;
    let newer = inputs
        .replace("\\ ", "\0")
        .split_whitespace()
        .map(|input| input.replace('\0', " "))
        .find(|input| {
            std::fs::metadata(input)
                .and_then(|m| m.modified())
                .map_or(true, |at| at > linked)
        })?;
    Some(format!(
        "`{}` is older than {newer}, which it is built from; rebuild it (`cargo build -p cpl-cli`, with --release for a release run)",
        cpl.display()
    ))
}

/// Links `bin` (or copies it, where links fail) into `dir`, keeping its file
/// name, so the run keeps the binary it started with whatever replaces `bin`.
pub fn snapshot(bin: &Path, dir: &Path) -> Result<PathBuf, String> {
    let name = bin
        .file_name()
        .ok_or_else(|| format!("{} names no file", bin.display()))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let to = dir.join(name);
    let _ = std::fs::remove_file(&to);
    if std::fs::hard_link(bin, &to).is_err() {
        std::fs::copy(bin, &to).map_err(|e| format!("cannot copy {}: {e}", bin.display()))?;
    }
    Ok(to)
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

/// An `@cpp` body is C++ only, so the program runs as its C++ build, as
/// `cpl run` runs it.
/// The directory C++ builds of `what` are cached in: under `CPL_CXX_CACHE_DIR`, else the temp dir.
pub(crate) fn cxx_cache(what: &str) -> std::path::PathBuf {
    let cache = match cpl_env::CXX_CACHE_DIR.path() {
        Some(dir) => dir.join(format!("conform-{what}")),
        None => std::env::temp_dir().join(format!("cpl-conform-{what}")),
    };
    let _ = std::fs::create_dir_all(&cache);
    cache
}

fn run_cpp_build(
    program: &cpl_ir::Program,
    path: &str,
    text: &str,
    input: &[u8],
) -> Result<Ran, String> {
    let cpp = cpl_codegen::emit(program, path, Some(text));
    let cache = cxx_cache("cpp-body");
    let bin = crate::diff::compile_cpp(&cpp, &cache).map_err(|e| match e {
        crate::diff::CppCompileError::Cxx(m) | crate::diff::CppCompileError::Infra(m) => m,
    })?;
    let out = runner::run_piped(std::process::Command::new(bin), input, case_limit(text))
        .map_err(|e| format!("the C++ build {e}"))?;
    Ok(Ran {
        stdout: out.stdout,
        trapped: !out.status.success(),
        trap_rendered: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

fn cli_within(
    cpl: &Path,
    args: &[&str],
    input: &[u8],
    limit: u64,
) -> Result<(i32, String, String), String> {
    let mut cmd = crate::pin::command(cpl);
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
/// The first `// has:` string missing from `out`, or `// lacks:` string in it.
fn unmet_has_lacks(text: &str, out: &str) -> Option<String> {
    header_values(text, "has:")
        .into_iter()
        .map(unquote)
        .find(|w| !out.contains(w.as_str()))
        .map(|w| format!("wanted output with {w:?}"))
        .or_else(|| {
            header_values(text, "lacks:")
                .into_iter()
                .map(unquote)
                .find(|b| out.contains(b.as_str()))
                .map(|b| format!("wanted output without {b:?}"))
        })
}

pub fn header_values<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
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

pub fn unquote(v: &str) -> String {
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
    let cpl = cli_binary()?;
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
    if sub == "dap" {
        let header_lines = text
            .lines()
            .take_while(|l| l.trim_start().starts_with("//"))
            .count() as u32;
        let args: Vec<&str> = words.collect();
        let run = crate::dap::session(&cpl, &copy, &header_input(text), header_lines, &args);
        let _ = std::fs::remove_dir_all(&dir);
        return run;
    }
    let argv = |sub: &str, rest: Vec<&str>| {
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
        argv
    };
    for before in header_values(text, "before:") {
        let mut words = before.split_whitespace();
        let first = words.next().ok_or("empty `// before:` header")?;
        let mut cmd = crate::pin::command(&cpl);
        cmd.args(argv(first, words.collect())).current_dir(&dir);
        let out = runner::run_piped(cmd, &header_input(text), runner::case_timeout())
            .map_err(|e| format!("cpl {e}"))?;
        if !out.status.success() {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(format!(
                "`// before: {before}` failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
    }
    let mut cmd = crate::pin::command(&cpl);
    cmd.args(argv(sub, words.collect())).current_dir(&dir);
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

/// `text` with each `<digits> ms` and the padding before it blanked: a tool that reports how long a case
/// ran (`cpl test`) is deterministic in everything but that.
fn untimed(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(|c: char| c.is_ascii_digit()) {
        let end = rest[start..]
            .find(|c: char| !c.is_ascii_digit())
            .map_or(rest.len(), |e| start + e);
        out.push_str(&rest[..start]);
        if rest[end..].starts_with(" ms") {
            out.truncate(out.trim_end_matches(' ').len());
            out.push('#');
        } else {
            out.push_str(&rest[start..end]);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// A `// tool:` case: runs the tool twice (tool output is deterministic),
/// then checks the expectation and every `// has:` / `// lacks:` string.
fn run_tool_case(case: &ConformanceCase, text: &str, args: &str, expect: Expect) -> CaseOutcome {
    let fail = |detail: String| CaseOutcome {
        status: "fail".into(),
        detail,
    };
    let (first, second) = match (run_tool(case, text, args), run_tool(case, text, args)) {
        (Ok(a), Ok(b)) => (a, b),
        (Err(e), _) | (_, Err(e)) => return fail(e),
    };
    if first.status != second.status || untimed(&first.stdout) != untimed(&second.stdout) {
        return fail(format!("`cpl {args}` gave different output on two runs"));
    }
    let all = format!("{}{}", first.stdout, first.stderr);
    let got = || runner::truncate_chars(all.trim(), 200);
    let met = match &expect {
        Expect::Ok => first.status == 0,
        Expect::OkOutput(want) => first.status == 0 && first.stdout.trim_end_matches('\n') == want,
        Expect::Diag { code, line } => diag_lines(&all, code).any(|l| l == *line),
        Expect::Trap { code, line } => all.contains(code.as_str()) && rendered_at(&all, *line),
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

/// Whether a rendered diagnostic points at `line` (`file:LINE:COL`, or cpl-next's
/// `trap: CODE: line LINE: ..`); a bare `:LINE` would also match a column.
fn rendered_at(rendered: &str, line: usize) -> bool {
    rendered.contains(&format!(":{line}:")) || rendered.contains(&format!(" line {line}:"))
}

/// The marker that ends the line a `// expect: CODE` diagnostic or trap
/// points at (decision 506): the case says where, so no line count can drift
/// when a header line comes or goes.
pub const HERE: &str = "// <- here";

/// The 1-based file lines that end with the `// <- here` marker.
pub fn marked_lines(text: &str) -> Vec<usize> {
    text.split('\n')
        .enumerate()
        .filter(|(_, l)| l.trim_end().ends_with(HERE))
        .map(|(i, _)| i + 1)
        .collect()
}

/// Parses `// expect: ...`. A diagnostic or trap's line is the file line
/// marked `// <- here`.
pub fn parse_header(text: &str) -> Result<Expect, String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let head = lines.first().copied().unwrap_or("").trim();
    let bad = || format!("first line is not `// expect: ...`: {head}");
    let Some(after_slashes) = head.strip_prefix("//") else {
        return Err(bad());
    };
    let Some(spec) = after_slashes.trim_start().strip_prefix("expect:") else {
        return Err(bad());
    };
    let spec = spec.trim();
    let marks = marked_lines(text);
    let unmarked = |e: Expect| {
        if marks.is_empty() {
            Ok(e)
        } else {
            Err(format!(
                "line {} ends with `{HERE}`, but `// expect: {spec}` names no diagnostic to point at",
                marks[0]
            ))
        }
    };

    if let Some(n) = spec.strip_prefix("exit ") {
        if let Ok(code) = n.trim().parse::<i32>() {
            return unmarked(Expect::Exit(code));
        }
    }
    if spec == "ok" {
        return unmarked(Expect::Ok);
    }
    if let Some(rest) = spec.strip_prefix("ok,") {
        let rest = rest.trim();
        if let Some(rest) = rest.strip_prefix("output") {
            let rest = rest.trim();
            if let Some(inner) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                return unmarked(Expect::OkOutput(unescape_python_like(inner)));
            }
        }
        return Err(bad());
    }

    // `E0601` or `R0301 (checked)`, with the line marked `// <- here`.
    let parts: Vec<&str> = spec.split_whitespace().collect();
    if parts.get(1) == Some(&"at") {
        return Err(format!(
            "`// expect: {spec}`: `at N` is gone (decision 506); write `// expect: {}` and end the line it points at with `{HERE}`",
            parts[0]
        ));
    }
    let tag_ok = |t: &str| {
        t.len() > 2
            && t.starts_with('(')
            && t.ends_with(')')
            && t[1..t.len() - 1]
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
    };
    let Some(&code) = parts.first() else {
        return Err(bad());
    };
    if !is_diag_code(code) || parts.len() > 2 || (parts.len() == 2 && !tag_ok(parts[1])) {
        return Err(bad());
    }
    let line = match marks.as_slice() {
        [line] => *line,
        [] => {
            return Err(format!(
                "`// expect: {spec}` needs the line it points at to end with `{HERE}`"
            ))
        }
        more => {
            return Err(format!(
                "`{HERE}` marks lines {more:?}; mark only the line `{code}` points at"
            ))
        }
    };
    let code = code.to_string();
    Ok(if code.starts_with('R') {
        Expect::Trap { code, line }
    } else {
        Expect::Diag { code, line }
    })
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

pub fn run_one(db: &mut Session, case: &ConformanceCase) -> CaseOutcome {
    let text = match std::fs::read_to_string(&case.path) {
        Ok(t) => t,
        Err(e) => {
            return CaseOutcome {
                status: "bad".into(),
                detail: format!("cannot read: {e}"),
            };
        }
    };
    let expect = match parse_header(&text) {
        Ok(h) => h,
        Err(why) => {
            return CaseOutcome {
                status: "bad".into(),
                detail: why,
            };
        }
    };
    if let Some(args) = header_values(&text, "tool:").first() {
        return run_tool_case(case, &text, args, expect);
    }
    let path_str = case.path.to_string_lossy();
    let mut backend = match (on_next(&text), cli_override()) {
        (true, _) => match next_binary() {
            Ok(next) => Backend::Next(next),
            Err(why) => {
                return CaseOutcome {
                    status: "fail".into(),
                    detail: why,
                }
            }
        },
        (false, Some(cpl)) => Backend::Cli(cpl),
        (false, None) => Backend::Linked(db),
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
            let target_line = line;
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
                match unmet_has_lacks(&text, &run.trap_rendered) {
                    Some(why) => CaseOutcome {
                        status: "fail".into(),
                        detail: format!(
                            "{why}; got: {}",
                            runner::truncate_chars(run.trap_rendered.trim(), 200)
                        ),
                    },
                    None => CaseOutcome {
                        status: "pass".into(),
                        detail: String::new(),
                    },
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
            let target_line = line;
            let unmet = unmet_has_lacks(&text, &diag);
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
                    .filter(|&l| l != target_line)
                    .map(|l| format!(" (it is on line {l}: move `{HERE}` there)"))
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
    let held = cli_binary().ok().and_then(|bin| {
        let dir = bin
            .parent()?
            .join(format!(".cpl-conform-{}", std::process::id()));
        hold_cli(Some(snapshot(&bin, &dir).ok()?));
        Some(dir)
    });
    let _pin = match crate::pin::Pin::new(cpl_check::loaded_std()) {
        Ok(pin) => pin,
        Err(e) => {
            let detail = format!("cannot pin std for the run: {e}");
            return SuiteReport::from_outcomes(
                cases
                    .iter()
                    .map(|case| {
                        let outcome = CaseOutcome {
                            status: "fail".into(),
                            detail: detail.clone(),
                        };
                        (case.id.clone(), outcome)
                    })
                    .collect(),
            );
        }
    };
    let outcomes = runner::parallel_with_db(&cases, |db, case| {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_one(db, case)))
            .unwrap_or_else(|_| CaseOutcome {
                status: "panic".into(),
                detail: "interpreter/checker panicked".into(),
            });
        (case.id.clone(), outcome)
    });
    hold_cli(None);
    if let Some(dir) = held {
        let _ = std::fs::remove_dir_all(dir);
    }
    SuiteReport::from_outcomes(outcomes)
}

pub struct RuleCoverage {
    pub defined: BTreeMap<String, String>,
    pub cited: BTreeSet<String>,
    pub missing: Vec<(String, String)>,
    pub unknown: Vec<String>,
}

/// Checks rule coverage: `(R\d\d[a-z]?(?:\.\d+)+)`, optionally `, permits)`, in `spec/core/*.md` vs `// rule:` in `spec/conformance/**/*.cp`.
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
                let close = if line[j..].starts_with(", permits)") {
                    j + 9
                } else {
                    j
                };
                if dot_groups > 0 && close < bytes.len() && bytes[close] == b')' {
                    out.push(line[start..j].to_string());
                    i = close + 1;
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
    #[test]
    fn next_diagnostics_render_with_their_code_and_line() {
        let json = r#"{"code":"E0501","file":"program","message":"m","start":{"col":5,"line":7}}"#;
        let d = super::next_rendered(&format!("{json}\nplain\n"));
        assert_eq!(d, "error[E0501]: m\n  --> program:7:5\nplain\n");
        assert_eq!(super::diag_line(&d, "E0501"), Some(7));
    }

    #[test]
    fn an_engines_next_header_runs_the_case_on_cpl_next() {
        assert!(super::on_next(
            "// expect: ok\n// engines: next\nwrite(1)\n"
        ));
        assert!(!super::on_next("// expect: ok\nprint(1)\n"));
        assert!(!super::on_next(
            "// expect: ok\nprint(1)\n// engines: next\n"
        ));
    }

    use super::*;

    #[test]
    fn a_permits_rule_is_still_a_definition() {
        assert_eq!(
            extract_rule_defs("defines it. (R02.4.3, permits)"),
            vec!["02.4.3"]
        );
        assert_eq!(extract_rule_defs("rejects it. (R02.4.4)"), vec!["02.4.4"]);
        assert!(extract_rule_defs("see (R02.4.4, forbids)").is_empty());
    }

    #[test]
    fn input_header_is_stdin_and_not_a_source_line() {
        let text = "// expect: ok, output \"1\"\n// input: \"3 4\\n\"\nprint(1)\n";
        assert_eq!(header_input(text), b"3 4\n");
        assert_eq!(parse_header(text).unwrap(), Expect::OkOutput("1".into()));
        assert!(header_input("// expect: ok\nprint(1)\n").is_empty());
    }

    #[test]
    fn has_and_lacks_are_checked_against_any_output() {
        let text = "// expect: R0203\n// has: \"call stack: f\"\n// lacks: zzz\nprint(1)\n";
        assert_eq!(unmet_has_lacks(text, "note: call stack: f (a.cp:3)"), None);
        assert!(unmet_has_lacks(text, "note: locals: a = 1").is_some());
        assert!(unmet_has_lacks(text, "call stack: f zzz").is_some());
    }

    #[test]
    fn a_trap_line_is_not_matched_by_its_column() {
        let rendered = "error[R0202]: overflow\n  --> a.cp:6:7\n";
        assert!(rendered_at(rendered, 6));
        assert!(!rendered_at(rendered, 7));
        let next = "trap: R0502: line 11: a value in the input is outside its contract\ncall stack: P.read (line 4)\n";
        assert!(rendered_at(next, 11));
        assert!(!rendered_at(next, 4));
    }

    #[test]
    fn a_cpl_older_than_a_source_it_was_built_from_is_refused() {
        let dir = std::env::temp_dir().join(format!("cpl-conform-stale-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("my src")).unwrap();
        let (cpl, source) = (dir.join("cpl"), dir.join("my src/gen.rs"));
        let at = |path: &Path, secs: u64| {
            let file = std::fs::File::options().write(true).open(path).unwrap();
            let when = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
            file.set_modified(when).unwrap();
        };
        std::fs::write(&cpl, "").unwrap();
        std::fs::write(&source, "").unwrap();
        assert_eq!(stale_cli(&cpl), None, "no dep-info, nothing to compare");
        let deps = format!(
            "{}: {}\n",
            cpl.display(),
            source.display().to_string().replace(' ', "\\ ")
        );
        std::fs::write(cpl.with_extension("d"), deps).unwrap();
        at(&source, 1_000);
        at(&cpl, 2_000);
        assert_eq!(stale_cli(&cpl), None);
        at(&source, 3_000);
        let why = stale_cli(&cpl).expect("a source changed after cpl was linked");
        assert!(
            why.contains("my src/gen.rs") && why.contains("rebuild"),
            "{why}"
        );
        std::fs::remove_file(&source).unwrap();
        assert!(
            stale_cli(&cpl).is_some(),
            "a source that is gone is a change too"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    /// What `text`'s `@gen fn input` draws and its program prints on that
    /// input, the same on the VM and the C++ build, and both C++ sources,
    /// built in `db`.
    fn built(db: &mut Session, text: &str) -> [String; 4] {
        let analysis = db.analyze_text("t.cp", text);
        db.remove_file("t.cp");
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let gen = cpl_ir::lower_gen(&analysis, "input", 3).expect("the @gen fn lowers");
        let program = cpl_ir::lower(&analysis).expect("the program lowers");
        let both = |program: &cpl_ir::Program, input: &[u8]| {
            let vm = runner::run_interp(program, "t.cp", text, input);
            assert!(vm.trap.is_none(), "{}", vm.trap_rendered);
            let native = run_cpp_build(program, "t.cp", text, input).expect("the C++ builds");
            assert!(!native.trapped, "{}", native.trap_rendered);
            assert_eq!(
                vm.stdout, native.stdout,
                "the VM and C++ disagree on {text}"
            );
            vm.stdout
        };
        let drawn = both(&gen, b"");
        let printed = both(&program, &drawn);
        [
            String::from_utf8_lossy(&drawn).into_owned(),
            String::from_utf8_lossy(&printed).into_owned(),
            cpl_codegen::emit(&gen, "t", Some(text)),
            cpl_codegen::emit(&program, "t", Some(text)),
        ]
    }

    #[test]
    fn a_program_builds_the_same_after_another_in_one_session() {
        let graph = "\
domain V = 0..5
@gen fn input():
    let g = gen<Graph<V, (to: V, w: 1..=9)>>(simple: true, connected: true)
    assert V.all(v => g.out(v).all(e => e.to != v and 1 <= e.w and e.w <= 9))
    print(V.count)
let k: Int = read()
print(k)
";
        let plain = "\
@gen fn input():
    print(gen(1..=9))
let k: Int = read()
print(k + 1)
";
        let alone = [graph, plain].map(|text| built(&mut Session::new(), text));
        assert_eq!(alone[0][..2], ["5\n", "5\n"]);
        let mut db = Session::new();
        for order in [[0, 1], [1, 0], [0, 0]] {
            for i in order {
                let text = [graph, plain][i];
                assert_eq!(built(&mut db, text), alone[i], "{text}");
            }
        }
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
