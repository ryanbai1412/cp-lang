//! `cpl report`: self-contained, privacy-redacted bug reports and ICE crash bundles.

use std::backtrace::Backtrace;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use cpl_json::{obj, Value};

use crate::{EXIT_ERRORS, EXIT_USAGE, VERSION};

pub const USAGE: &str = "\
usage: cpl report <file.cp> [--input <in.txt> | < stdin] [--shrink]
       cpl report --last

Write a paste-ready Markdown bug report (`cpl-report-<hash>.md`) containing:
  - cpl version (version, channel, commit), OS/arch, g++ version, judge profile
  - source and input (truncated if large; `--shrink` minimizes failing input)
  - interpreter and C++ stdout, stderr, exit codes, and the first differing line
  - any ICE backtrace (`--last`: from the newest `~/.cpl/crashes/<ts>/` bundle)

Privacy: omits environment variables, absolute paths (basenames only), hostname,
and username. When the report is under 6 KB, also prints a prefilled GitHub
issue URL.

options:
  -i, --input <file>  read program input from <file> instead of stdin
  --shrink            minimize the input while the failure still reproduces
  --last              build the report from the newest ICE crash bundle
  --json              print one JSON object instead: the report's `path` and
                      `bytes`, the `sections` it includes, the `comparison`
                      of the two runs and the prefilled `issue_url` (`null`
                      when there is none)
  -h, --help          print this help
";

const GIT_COMMIT: &str = env!("CPL_GIT_COMMIT");
const CHANNEL: &str = env!("CPL_CHANNEL");
const MAX_INPUT_BYTES: usize = 4096;
const MAX_OUTPUT_BYTES: usize = 8192;
const PREFILL_URL_LIMIT: usize = 6144;
const ISSUE_BASE_URL: &str = "https://github.com/ryanbai1412/cp-lang/issues/new?template=bug.yml";

#[derive(Default, Clone)]
struct CrashContext {
    args: Vec<String>,
    source_name: Option<String>,
    source_text: Option<String>,
    input: Option<Vec<u8>>,
}

fn context_cell() -> &'static Mutex<CrashContext> {
    static CELL: OnceLock<Mutex<CrashContext>> = OnceLock::new();
    CELL.get_or_init(|| Mutex::new(CrashContext::default()))
}

/// Records the active CLI invocation so an ICE panic hook can save a bundle.
pub fn set_crash_context(args: &[String]) {
    let mut ctx = CrashContext {
        args: args.iter().map(|a| redact_arg(a)).collect(),
        source_name: None,
        source_text: None,
        input: None,
    };
    for arg in args.iter().skip(1) {
        if arg.ends_with(".cp") {
            let p = Path::new(arg);
            if let Ok(text) = std::fs::read_to_string(p) {
                ctx.source_name = Some(basename(arg));
                ctx.source_text = Some(text);
                break;
            }
        }
    }
    if let Ok(mut guard) = context_cell().lock() {
        *guard = ctx;
    }
}

/// Directory holding ICE crash bundles (`CPL_CRASH_DIR`, or `~/.cpl/crashes`).
pub fn crash_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CPL_CRASH_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    if let Some(home) = cpl_check::home_dir() {
        return home.join(".cpl").join("crashes");
    }
    std::env::temp_dir().join("cpl-crashes")
}

/// Installs the global panic hook that saves `~/.cpl/crashes/<ts>/` and prints
/// the one-line `cpl report --last` hint.
pub fn install_ice_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "internal compiler error".to_string()
        };
        if reader_went_away(&msg) {
            std::process::exit(0);
        }
        let bt = Backtrace::force_capture().to_string();
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", basename(l.file()), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());
        let ctx = context_cell().lock().map(|g| g.clone()).unwrap_or_default();
        let _ = write_crash_bundle(&crash_dir(), &ctx, &msg, &location, &bt);
        eprintln!("this is a cpl bug: run `cpl report --last`");
    }));
}

/// `print!` panics when whatever reads cpl's output stops early
/// (`cpl --version | head -1`): the reader has all it wanted, so it isn't a bug.
fn reader_went_away(msg: &str) -> bool {
    msg.starts_with("failed printing to std")
        && (msg.contains("(os error 32)") || msg.contains("(os error 232)"))
}

fn write_crash_bundle(
    root: &Path,
    ctx: &CrashContext,
    message: &str,
    location: &str,
    backtrace: &str,
) -> std::io::Result<PathBuf> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let mut dir = root.join(format!("{ts:016}"));
    let mut suffix = 0u32;
    while dir.exists() {
        suffix += 1;
        dir = root.join(format!("{ts:016}-{suffix}"));
    }
    std::fs::create_dir_all(&dir)?;

    let redactor = Redactor::new(&[]);
    let meta = format!(
        "timestamp: {ts}\nargs: {}\nsource: {}\npanic: {}\nlocation: {}\n",
        ctx.args.join(" "),
        ctx.source_name.as_deref().unwrap_or("none"),
        redactor.clean(message),
        redactor.clean(location),
    );
    std::fs::write(dir.join("meta.txt"), meta)?;
    std::fs::write(dir.join("backtrace.txt"), redactor.clean(backtrace))?;
    if let Some(src) = &ctx.source_text {
        std::fs::write(dir.join("source.cp"), src)?;
    }
    if let Some(inp) = &ctx.input {
        std::fs::write(dir.join("input.txt"), inp)?;
    }
    Ok(dir)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunCapture {
    code: u8,
    stdout: String,
    stderr: String,
}

/// Entry point for `cpl report`.
pub fn report(
    args: &[&str],
    stdin: &mut (dyn FnMut() -> Vec<u8> + Send),
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    if args.iter().any(|a| matches!(*a, "-h" | "--help")) {
        let _ = stdout.write_all(USAGE.as_bytes());
        return 0;
    }

    let mut file: Option<&str> = None;
    let mut input_path: Option<&str> = None;
    let mut shrink = false;
    let mut last = false;
    let mut json = false;

    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "--last" => last = true,
            "--shrink" => shrink = true,
            "--json" => json = true,
            "-i" | "--input" => {
                i += 1;
                let Some(p) = args.get(i) else {
                    *stderr += "cpl: `--input` needs a file path\n";
                    return EXIT_USAGE;
                };
                input_path = Some(p);
            }
            arg if arg.starts_with("--input=") => {
                input_path = Some(&arg["--input=".len()..]);
            }
            arg if arg.starts_with('-') => {
                let message = cpl_diag::usage::no_option("report", arg);
                *stderr += &format!("cpl: {message}\n\n{USAGE}");
                return EXIT_USAGE;
            }
            arg => {
                if file.is_some() {
                    *stderr += &format!("cpl: unexpected argument `{arg}` for `report`\n");
                    return EXIT_USAGE;
                }
                file = Some(arg);
            }
        }
        i += 1;
    }

    if last && file.is_some() {
        *stderr += "cpl: pass either a source file or `--last`, not both\n";
        return EXIT_USAGE;
    }
    if !last && file.is_none() {
        *stderr += &format!("cpl: `report` needs a file or `--last`\n\n{USAGE}");
        return EXIT_USAGE;
    }

    let mut extra_paths: Vec<String> = Vec::new();
    let (source_name, source_text, mut input_bytes, ice_meta, ice_backtrace) = if last {
        match load_latest_bundle(&crash_dir()) {
            Ok(b) => (b.source_name, b.source_text, b.input, b.meta, b.backtrace),
            Err(msg) => {
                *stderr += &format!("cpl: {msg}\n");
                return EXIT_ERRORS;
            }
        }
    } else {
        let path = file.unwrap();
        extra_paths.push(path.to_string());
        if let Ok(canon) = std::fs::canonicalize(path) {
            extra_paths.push(canon.to_string_lossy().into_owned());
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                *stderr += &format!("cpl: can't read `{path}`: {e}\n");
                return EXIT_USAGE;
            }
        };
        let input = if let Some(ip) = input_path {
            extra_paths.push(ip.to_string());
            if let Ok(canon) = std::fs::canonicalize(ip) {
                extra_paths.push(canon.to_string_lossy().into_owned());
            }
            match std::fs::read(ip) {
                Ok(b) => b,
                Err(e) => {
                    *stderr += &format!("cpl: can't read input `{ip}`: {e}\n");
                    return EXIT_USAGE;
                }
            }
        } else {
            drain_stdin(stdin)
        };
        (basename(path), Some(text), input, None, None)
    };

    // Materialize source in a scratch directory when running from `--last` or
    // when we want a clean path for reproduction.
    let scratch = std::env::temp_dir().join(format!(
        "cpl-report-run-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::create_dir_all(&scratch);
    extra_paths.push(scratch.to_string_lossy().into_owned());
    if let Ok(canon) = std::fs::canonicalize(&scratch) {
        extra_paths.push(canon.to_string_lossy().into_owned());
    }

    let repro_path = if let Some(src) = &source_text {
        let p = scratch.join(&source_name);
        if std::fs::write(&p, src).is_ok() {
            extra_paths.push(p.to_string_lossy().into_owned());
            Some(p)
        } else {
            None
        }
    } else {
        None
    };

    let judge_profile = match (&repro_path, &source_text) {
        (Some(p), Some(src)) => cpl_judge::resolve(p, src, None)
            .map(|prof| prof.name.clone())
            .unwrap_or_else(|_| "default".to_string()),
        _ => "none".to_string(),
    };

    let mut shrunk_note: Option<String> = None;
    let (interp_run, cpp_run) = if let Some(p) = &repro_path {
        let mut ir = run_interp(p, &input_bytes);
        let mut cr = run_cpp(p, &input_bytes);
        if shrink && is_interesting(&ir, &cr) {
            let orig_len = input_bytes.len();
            let shrunk = shrink_input(
                source_text.as_deref().unwrap_or(""),
                &input_bytes,
                &|cand| {
                    let i2 = run_interp(p, cand);
                    let c2 = run_cpp(p, cand);
                    same_failure_kind(&ir, &cr, &i2, &c2)
                },
            );
            if shrunk.len() < orig_len {
                shrunk_note = Some(format!(
                    "shrunk from {orig_len} bytes to {} bytes",
                    shrunk.len()
                ));
                input_bytes = shrunk;
                ir = run_interp(p, &input_bytes);
                cr = run_cpp(p, &input_bytes);
            }
        }
        (Some(ir), Some(cr))
    } else {
        (None, None)
    };

    let _ = std::fs::remove_dir_all(&scratch);

    let redactor = Redactor::new(&extra_paths);
    let gxx = gxx_version();
    let version_line = format!("cpl {VERSION} ({CHANNEL}, {GIT_COMMIT})");
    let os_arch = format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH);

    let diff_summary = match (&interp_run, &cpp_run) {
        (Some(ir), Some(cr)) => Some(compare_runs(ir, cr)),
        _ => None,
    };

    let body = build_markdown(
        &version_line,
        &os_arch,
        &gxx,
        &judge_profile,
        &source_name,
        source_text.as_deref(),
        &input_bytes,
        shrunk_note.as_deref(),
        interp_run.as_ref(),
        cpp_run.as_ref(),
        diff_summary.as_ref(),
        ice_meta.as_deref(),
        ice_backtrace.as_deref(),
        &redactor,
    );

    let hash = short_hash(body.as_bytes());
    let report_filename = format!("cpl-report-{hash}.md");
    if let Err(e) = std::fs::write(&report_filename, &body) {
        *stderr += &format!("cpl: can't write `{report_filename}`: {e}\n");
        return EXIT_ERRORS;
    }

    let mut sections = vec!["version/env"];
    if source_text.is_some() {
        sections.push("source");
    }
    if !input_bytes.is_empty() {
        sections.push("input");
    }
    if interp_run.is_some() {
        sections.push("interpreter");
    }
    if cpp_run.is_some() {
        sections.push("C++");
    }
    if ice_backtrace.is_some() {
        sections.push("ICE backtrace");
    }

    let url = (body.len() <= PREFILL_URL_LIMIT).then(|| {
        let title = issue_title(&source_name, diff_summary.as_ref(), ice_backtrace.is_some());
        format!(
            "{ISSUE_BASE_URL}&title={}&body={}",
            percent_encode(&title),
            percent_encode(&body),
        )
    });
    if json {
        let or_null = |text: Option<&str>| text.map_or(Value::Null, Value::from);
        let report = obj([
            ("path", report_filename.as_str().into()),
            ("bytes", body.len().into()),
            (
                "sections",
                sections
                    .iter()
                    .map(|s| Value::from(*s))
                    .collect::<Vec<_>>()
                    .into(),
            ),
            (
                "comparison",
                or_null(diff_summary.as_ref().map(|d| d.headline.as_str())),
            ),
            ("issue_url", or_null(url.as_deref())),
        ]);
        let _ = writeln!(stdout, "{}", cpl_json::to_string(&report));
        return 0;
    }
    let _ = writeln!(stdout, "wrote {report_filename} ({} bytes)", body.len());
    let _ = writeln!(
        stdout,
        "included: {} (redacted: no env vars, no absolute paths, no hostname/username)",
        sections.join(", ")
    );
    if let Some(diff) = &diff_summary {
        let _ = writeln!(stdout, "comparison: {}", diff.headline);
    }
    if let Some(url) = url {
        let _ = writeln!(stdout, "issue URL: {url}");
    }
    0
}

fn drain_stdin(stdin: &mut (dyn FnMut() -> Vec<u8> + Send)) -> Vec<u8> {
    let mut buf = Vec::new();
    loop {
        let chunk = stdin();
        if chunk.is_empty() {
            break;
        }
        buf.extend_from_slice(&chunk);
    }
    buf
}

struct LoadedBundle {
    source_name: String,
    source_text: Option<String>,
    input: Vec<u8>,
    meta: Option<String>,
    backtrace: Option<String>,
}

fn load_latest_bundle(root: &Path) -> Result<LoadedBundle, String> {
    let entries = std::fs::read_dir(root).map_err(|_| {
        "no crash bundles found; `cpl report --last` is for after an internal compiler error"
            .to_string()
    })?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let newest = dirs.pop().ok_or_else(|| {
        "no crash bundles found; `cpl report --last` is for after an internal compiler error"
            .to_string()
    })?;

    let meta = std::fs::read_to_string(newest.join("meta.txt")).ok();
    let backtrace = std::fs::read_to_string(newest.join("backtrace.txt")).ok();
    let source_text = std::fs::read_to_string(newest.join("source.cp")).ok();
    let input = std::fs::read(newest.join("input.txt")).unwrap_or_default();
    let source_name = meta
        .as_deref()
        .and_then(|m| {
            m.lines()
                .find_map(|l| l.strip_prefix("source: "))
                .filter(|s| *s != "none" && !s.is_empty())
                .map(basename)
        })
        .unwrap_or_else(|| "source.cp".to_string());
    Ok(LoadedBundle {
        source_name,
        source_text,
        input,
        meta,
        backtrace,
    })
}

fn run_interp(path: &Path, input: &[u8]) -> RunCapture {
    let p = path.to_string_lossy();
    let out = crate::run_with_input(&["run", p.as_ref()], input);
    RunCapture {
        code: out.code,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: out.stderr,
    }
}

fn run_cpp(path: &Path, input: &[u8]) -> RunCapture {
    let p = path.to_string_lossy();
    let out = crate::run_with_input(&["run", "--release", p.as_ref()], input);
    RunCapture {
        code: out.code,
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: out.stderr,
    }
}

fn is_interesting(ir: &RunCapture, cr: &RunCapture) -> bool {
    ir.code != 0 || cr.code != 0 || ir.stdout != cr.stdout || ir.code != cr.code
}

fn same_failure_kind(
    orig_ir: &RunCapture,
    orig_cr: &RunCapture,
    ir: &RunCapture,
    cr: &RunCapture,
) -> bool {
    let orig_mismatch = orig_ir.stdout != orig_cr.stdout || orig_ir.code != orig_cr.code;
    if orig_mismatch {
        return ir.stdout != cr.stdout || ir.code != cr.code;
    }
    if orig_ir.code != 0 {
        return ir.code == orig_ir.code;
    }
    if orig_cr.code != 0 {
        return cr.code == orig_cr.code;
    }
    false
}

/// Shrinks input while `still_fails` holds. Tries typed `cpl-stress` shrinking
/// when the program has a typed `InputSpec` and a small generated failing case
/// exists; then applies line/token delta shrinking directly on the input bytes.
fn shrink_input(source: &str, original: &[u8], still_fails: &dyn Fn(&[u8]) -> bool) -> Vec<u8> {
    let mut best = original.to_vec();

    // 1. If the program has a typed InputSpec, try a few small seeds through
    //    `cpl-stress`'s typed generator + shrinker to see if it finds an even
    //    smaller counterexample.
    if let Ok(spec) = cpl_stress::InputSpec::from_source(source) {
        if !spec.is_empty() {
            for seed in 1..=20u64 {
                let mut rng = cpl_stress::rng::Rng::new(seed);
                if let Some(case) =
                    cpl_stress::gen::generate(&spec, &mut rng, cpl_stress::Size::Small(6))
                {
                    let rendered = case.render();
                    if rendered.len() < best.len() && still_fails(rendered.as_bytes()) {
                        let shrunk = cpl_stress::shrink::shrink(&spec, &case, 200, &mut |c| {
                            still_fails(c.render().as_bytes())
                        });
                        let s = shrunk.render();
                        if s.len() < best.len() {
                            best = s.into_bytes();
                        }
                        break;
                    }
                }
            }
        }
    }

    // 2. Line-level delta reduction on the current best input.
    if let Ok(text) = std::str::from_utf8(&best) {
        let mut lines: Vec<String> = text.lines().map(ToString::to_string).collect();
        let trailing_nl = text.ends_with('\n');
        let join = |ls: &[String]| {
            let mut s = ls.join("\n");
            if trailing_nl && !s.is_empty() {
                s.push('\n');
            }
            s.into_bytes()
        };
        let mut budget = 64usize;
        let mut idx = lines.len();
        while idx > 0 && budget > 0 {
            idx -= 1;
            let mut cand = lines.clone();
            cand.remove(idx);
            let bytes = join(&cand);
            budget -= 1;
            if !bytes.is_empty() && bytes.len() < best.len() && still_fails(&bytes) {
                lines = cand;
                best = bytes;
            }
        }
    }

    best
}

struct DiffSummary {
    disagree: bool,
    headline: String,
    detail: String,
}

fn compare_runs(ir: &RunCapture, cr: &RunCapture) -> DiffSummary {
    if ir.code == cr.code && ir.stdout == cr.stdout {
        return DiffSummary {
            disagree: false,
            headline: format!("agree (exit code {})", ir.code),
            detail: format!("Interpreter and C++ agree (exit code {}).", ir.code),
        };
    }

    let i_lines: Vec<&str> = ir.stdout.lines().collect();
    let c_lines: Vec<&str> = cr.stdout.lines().collect();
    let max_lines = i_lines.len().max(c_lines.len());
    for idx in 0..max_lines {
        let a = i_lines.get(idx).copied();
        let b = c_lines.get(idx).copied();
        if a != b {
            let line_no = idx + 1;
            let a_disp = a
                .map(|s| format!("`{s}`"))
                .unwrap_or_else(|| "<EOF>".into());
            let b_disp = b
                .map(|s| format!("`{s}`"))
                .unwrap_or_else(|| "<EOF>".into());
            return DiffSummary {
                disagree: true,
                headline: format!(
                    "disagree (exit interp={} cpp={}, first differing line {})",
                    ir.code, cr.code, line_no
                ),
                detail: format!(
                    "Interpreter and C++ **disagree** (exit codes: interpreter `{}`, C++ `{}`).\nFirst differing line: **line {}**\n- interpreter: {}\n- C++: {}",
                    ir.code, cr.code, line_no, a_disp, b_disp
                ),
            };
        }
    }

    DiffSummary {
        disagree: true,
        headline: format!("disagree on exit code (interp={}, cpp={})", ir.code, cr.code),
        detail: format!(
            "Interpreter and C++ **disagree** on exit code (interpreter `{}`, C++ `{}`); stdout matches.",
            ir.code, cr.code
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_markdown(
    version_line: &str,
    os_arch: &str,
    gxx: &str,
    judge_profile: &str,
    source_name: &str,
    source_text: Option<&str>,
    input_bytes: &[u8],
    shrunk_note: Option<&str>,
    interp_run: Option<&RunCapture>,
    cpp_run: Option<&RunCapture>,
    diff: Option<&DiffSummary>,
    ice_meta: Option<&str>,
    ice_backtrace: Option<&str>,
    redactor: &Redactor,
) -> String {
    let mut md = String::new();
    md.push_str("## Environment\n\n");
    md.push_str(&format!("- **cpl**: `{version_line}`\n"));
    md.push_str(&format!("- **OS / arch**: `{os_arch}`\n"));
    md.push_str(&format!("- **g++**: `{}`\n", redactor.clean(gxx)));
    md.push_str(&format!("- **judge profile**: `{judge_profile}`\n\n"));

    if let Some(src) = source_text {
        md.push_str(&format!("## Source (`{source_name}`)\n\n```cp\n"));
        md.push_str(src);
        if !src.ends_with('\n') {
            md.push('\n');
        }
        md.push_str("```\n\n");
    }

    if !input_bytes.is_empty() || shrunk_note.is_some() {
        md.push_str("## Input\n\n");
        if let Some(note) = shrunk_note {
            md.push_str(&format!("_{note}_\n\n"));
        }
        let (shown, trunc) = truncate_bytes(input_bytes, MAX_INPUT_BYTES);
        md.push_str("```text\n");
        md.push_str(&redactor.clean(&shown));
        if !shown.ends_with('\n') && !shown.is_empty() {
            md.push('\n');
        }
        md.push_str("```\n");
        if let Some(note) = trunc {
            md.push_str(&format!("_{note}_\n"));
        }
        md.push('\n');
    }

    if let Some(diff) = diff {
        md.push_str("## Comparison\n\n");
        md.push_str(&redactor.clean(&diff.detail));
        md.push_str("\n\n");
    }

    if let Some(ir) = interp_run {
        push_run_section(&mut md, "Interpreter", ir, redactor);
    }
    if let Some(cr) = cpp_run {
        push_run_section(&mut md, "C++ build", cr, redactor);
    }

    if ice_meta.is_some() || ice_backtrace.is_some() {
        md.push_str("## ICE backtrace\n\n");
        if let Some(meta) = ice_meta {
            md.push_str("```text\n");
            md.push_str(&redactor.clean(meta));
            if !meta.ends_with('\n') {
                md.push('\n');
            }
            md.push_str("```\n\n");
        }
        if let Some(bt) = ice_backtrace {
            let (shown, trunc) = truncate_bytes(bt.as_bytes(), MAX_OUTPUT_BYTES);
            md.push_str("```text\n");
            md.push_str(&redactor.clean(&shown));
            if !shown.ends_with('\n') {
                md.push('\n');
            }
            md.push_str("```\n");
            if let Some(note) = trunc {
                md.push_str(&format!("_{note}_\n"));
            }
            md.push('\n');
        }
    }

    md
}

fn push_run_section(md: &mut String, title: &str, run: &RunCapture, redactor: &Redactor) {
    md.push_str(&format!("## {title}\n\n"));
    md.push_str(&format!("- **exit code**: `{}`\n\n", run.code));
    if !run.stdout.is_empty() {
        let (shown, trunc) = truncate_bytes(run.stdout.as_bytes(), MAX_OUTPUT_BYTES);
        md.push_str("**stdout:**\n```text\n");
        md.push_str(&redactor.clean(&shown));
        if !shown.ends_with('\n') {
            md.push('\n');
        }
        md.push_str("```\n");
        if let Some(note) = trunc {
            md.push_str(&format!("_{note}_\n"));
        }
        md.push('\n');
    }
    if !run.stderr.is_empty() {
        let (shown, trunc) = truncate_bytes(run.stderr.as_bytes(), MAX_OUTPUT_BYTES);
        md.push_str("**stderr:**\n```text\n");
        md.push_str(&redactor.clean(&shown));
        if !shown.ends_with('\n') {
            md.push('\n');
        }
        md.push_str("```\n");
        if let Some(note) = trunc {
            md.push_str(&format!("_{note}_\n"));
        }
        md.push('\n');
    }
}

fn truncate_bytes(bytes: &[u8], limit: usize) -> (String, Option<String>) {
    if bytes.len() <= limit {
        return (String::from_utf8_lossy(bytes).into_owned(), None);
    }
    let shown = String::from_utf8_lossy(&bytes[..limit]).into_owned();
    let note = format!(
        "truncated: {} bytes total, showing first {limit} bytes",
        bytes.len()
    );
    (shown, Some(note))
}

fn gxx_version() -> String {
    Command::new("g++")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(str::trim).map(ToString::to_string))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "not found".to_string())
}

fn issue_title(source_name: &str, diff: Option<&DiffSummary>, is_ice: bool) -> String {
    if is_ice {
        format!("[bug]: ICE on {source_name}")
    } else if diff.is_some_and(|d| d.disagree) {
        format!("[bug]: interpreter/C++ mismatch on {source_name}")
    } else {
        format!("[bug]: issue on {source_name}")
    }
}

/// Deterministic 8-hex-char FNV-1a hash for `cpl-report-<hash>.md`.
fn short_hash(bytes: &[u8]) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for &b in bytes {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &b in input.as_bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string()
}

fn redact_arg(arg: &str) -> String {
    if arg.starts_with('/') || (arg.len() > 2 && arg.as_bytes()[1] == b':') {
        basename(arg)
    } else {
        arg.to_string()
    }
}

/// Scrubs absolute paths, usernames, and hostnames from report text.
struct Redactor {
    /// Exact prefixes/paths to replace with their basename or placeholder, longest first.
    paths: Vec<(String, String)>,
    secrets: Vec<String>,
}

impl Redactor {
    fn new(extra_paths: &[String]) -> Self {
        let mut paths: Vec<(String, String)> = Vec::new();
        for p in extra_paths {
            if !p.is_empty() {
                paths.push((p.clone(), basename(p)));
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            let s = cwd.to_string_lossy().into_owned();
            if s.len() > 1 {
                paths.push((format!("{s}/"), String::new()));
                paths.push((s, "<cwd>".to_string()));
            }
        }
        if let Some(home) = cpl_check::home_dir() {
            let s = home.to_string_lossy().into_owned();
            if s.len() > 1 {
                paths.push((format!("{s}/"), String::new()));
                paths.push((s, "<home>".to_string()));
            }
        }
        let tmp = std::env::temp_dir().to_string_lossy().into_owned();
        if tmp.len() > 1 {
            let trimmed = tmp.trim_end_matches('/').to_string();
            paths.push((format!("{trimmed}/"), String::new()));
        }
        paths.sort_by_key(|a| std::cmp::Reverse(a.0.len()));

        let mut secrets = Vec::new();
        for key in ["USER", "USERNAME", "LOGNAME", "HOSTNAME"] {
            if let Ok(v) = std::env::var(key) {
                if v.len() >= 3 && !secrets.contains(&v) {
                    secrets.push(v);
                }
            }
        }
        if let Ok(host) = std::fs::read_to_string("/etc/hostname") {
            let h = host.trim();
            if h.len() >= 3 && !secrets.iter().any(|s| s == h) {
                secrets.push(h.to_string());
            }
        }
        secrets.sort_by_key(|a| std::cmp::Reverse(a.len()));

        Self { paths, secrets }
    }

    fn clean(&self, text: &str) -> String {
        let mut s = text.to_string();
        for (pat, rep) in &self.paths {
            if !pat.is_empty() {
                s = s.replace(pat, rep);
            }
        }
        s = strip_unix_abs_paths(&s);
        for sec in &self.secrets {
            s = s.replace(sec, "<redacted>");
        }
        s
    }
}

/// Replaces remaining Unix absolute paths (`/a/b/c.cp`) with their basename
/// (`c.cp`), while leaving URLs (`https://...`) untouched.
fn strip_unix_abs_paths(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        let prev = if i > 0 { Some(bytes[i - 1]) } else { None };
        let at_path_start = bytes[i] == b'/'
            && !matches!(
                prev,
                Some(b':' | b'/' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.')
            );
        if at_path_start {
            let mut j = i + 1;
            let mut slashes = 0usize;
            while j < bytes.len() {
                let c = bytes[j];
                if c.is_ascii_alphanumeric() || matches!(c, b'/' | b'_' | b'-' | b'.' | b'+') {
                    if c == b'/' {
                        slashes += 1;
                    }
                    j += 1;
                } else {
                    break;
                }
            }
            if slashes >= 1 {
                let candidate = &text[i..j];
                let trimmed = candidate.trim_end_matches('/');
                if let Some(base) = trimmed.rsplit('/').next() {
                    if !base.is_empty() {
                        out.push_str(base);
                        i = j;
                        continue;
                    }
                }
            }
        }
        let ch = text[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}
