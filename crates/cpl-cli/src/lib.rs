//! The `cpl` command-line tool. `cpl` is the working name of the binary
//! (`cp` clashes with Unix `cp`); source files end in `.cp`.

mod build;
mod check;
mod checked_fix;
mod commands;
use commands::{command_usage, unknown_option, usage as main_usage};

mod dap;
mod explain;
use explain::explain_code;
mod interact;
mod json_out;
mod laws;
mod live;
mod migrate;
mod profile;
mod replay;
mod report;
mod setup;
mod sha256;
mod submit;
use submit::{list_judges, submit};
pub mod toolchain;
pub use checked_fix::checked_fix;
mod trace;
mod unit_tests;

use build::{build_file, take_judge, BuildOptions};
pub use check::apply_error_fixes;
use check::{check, fix};
use unit_tests::{run_unit_tests, should_run_unit_tests};

pub use report::{install_ice_hook, set_crash_context};

use std::io::Write;

/// Version of the toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The release channel and commit this binary was built from: `CPL_CHANNEL`
/// and `CPL_COMMIT` at build time, else `dev` and the checkout's commit.
pub const CHANNEL: &str = env!("CPL_CHANNEL");
pub const COMMIT: &str = env!("CPL_GIT_COMMIT");

/// Crates linked into the binary, in pipeline order.
pub const COMPONENTS: &[&str] = &[
    cpl_diag::NAME,
    cpl_syntax::NAME,
    cpl_check::NAME,
    cpl_ir::NAME,
    cpl_codegen::NAME,
    cpl_interp::NAME,
    cpl_fmt::NAME,
    cpl_lint::NAME,
    cpl_doc::NAME,
    cpl_stress::NAME,
    cpl_contest::NAME,
    cpl_lsp::NAME,
];

/// `cpl 0.3.0-edge+1a2b3c4d`: what `cpl --version` prints; a stable build
/// drops the channel, `cpl 0.3.0+1a2b3c4d`.
pub fn version_line() -> String {
    let channel = match CHANNEL {
        "stable" => String::new(),
        channel => format!("-{channel}"),
    };
    format!("cpl {VERSION}{channel}+{}", &COMMIT[..COMMIT.len().min(8)])
}

/// What one invocation prints and the status it exits with.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    /// Bytes, not text: a program's output is whatever bytes it prints.
    pub stdout: Vec<u8>,
    pub stderr: String,
    pub code: u8,
}

/// Exit status when the source has errors.
pub const EXIT_ERRORS: u8 = 1;
/// Exit status when the command line is wrong or a file can't be read.
pub const EXIT_USAGE: u8 = 2;
/// Exit status when the program was fine to run and then failed: a failed
/// `assert`, an index out of range, input outside its declared range. The
/// emitted C++ exits with the same status when an `assert` fails.
pub const EXIT_FAILED: u8 = 3;

/// The emitted C++ has to fit a judge's source limit.
pub const MAX_CPP_BYTES: usize = 64 * 1024;

/// Runs `cpl` with `args` (without the program name) and nothing on
/// standard input, collecting what it prints. `cpl lsp` is served by
/// [`lsp`], because it streams instead of printing once.
pub fn run<S: AsRef<str>>(args: &[S]) -> Outcome {
    run_with_input(args, b"")
}

thread_local! {
    /// [`run_with_input`] is handing `exec` its stdin, so the process's own
    /// stdin says nothing about whether input was piped.
    static INPUT_GIVEN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// [`run`], with `input` as the program's standard input.
pub fn run_with_input<S: AsRef<str>>(args: &[S], input: &[u8]) -> Outcome {
    let mut stdout = Vec::new();
    let mut stderr = String::new();
    // The whole input arrives at once, and then it has ended.
    let mut rest = Some(input.to_vec());
    INPUT_GIVEN.set(true);
    let code = exec(
        args,
        &mut || rest.take().unwrap_or_default(),
        &mut stdout,
        &mut stderr,
    );
    INPUT_GIVEN.set(false);
    Outcome {
        stdout,
        stderr,
        code,
    }
}

pub fn exec<S: AsRef<str>>(
    args: &[S],
    stdin: &mut (dyn FnMut() -> Vec<u8> + Send),
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let args = commands::long_forms(args);
    let usage = args.first().and_then(|c| command_usage(c.as_ref()));
    let mut usage_error = |message: String| {
        *stderr += &format!("cpl: {message}\n\n{}", usage.unwrap_or(main_usage()));
        EXIT_USAGE
    };
    let rest: Vec<&str> = args.iter().skip(1).map(AsRef::as_ref).collect();
    let rest = commands::json_spelling(args.first().map_or("", AsRef::as_ref), rest);
    let named = commands::file_names(args.first().map_or("", AsRef::as_ref), rest);
    let rest: Vec<&str> = named.iter().map(|arg| arg.as_ref()).collect();
    match args.first().map(AsRef::as_ref) {
        Some("-h" | "--help") if rest == ["--json"] => {
            let _ = writeln!(stdout, "{}", commands::help_json());
            0
        }
        None | Some("-h" | "--help" | "help") => {
            let _ = stdout.write_all(main_usage().as_bytes());
            0
        }
        Some("-V" | "--version") => {
            let _ = writeln!(stdout, "{}", version_line());
            0
        }
        Some(_) if usage.is_some() && rest.iter().any(|arg| matches!(*arg, "-h" | "--help")) => {
            let _ = stdout.write_all(usage.unwrap_or(main_usage()).as_bytes());
            0
        }
        Some(c) if unknown_option(c, &rest).is_some() => usage_error(format!(
            "unknown option `{}` for `{c}`",
            unknown_option(c, &rest).unwrap()
        )),
        Some("replay") => replay::cli(&rest, stdout, stderr),
        Some("profile") => profile::profile(&rest, stdout, stderr),
        Some("cover") => profile::cover(&rest, stdout, stderr),
        Some("report") => report::report(&rest, stdin, stdout, stderr),
        Some("self") => toolchain::self_command(&rest, stdout, stderr),
        Some("explain") => explain_code(&rest, stdout, stderr),
        Some("migrate") => migrate::migrate(&rest, stdout, stderr),
        Some("setup") => setup::setup(&rest, stdout, stderr),
        Some("check") if rest.iter().all(|arg| *arg == "--json") => {
            usage_error("`check` needs a file: `cpl check a.cp`".into())
        }
        Some("check") => check(&rest, stdout, stderr),
        Some("fix") if rest.iter().all(|arg| *arg == "--json") => {
            usage_error("`fix` needs a file: `cpl fix a.cp`".into())
        }
        Some("fix") => fix(&rest, stdout, stderr),
        Some("run") => match RunArgs::parse(&rest) {
            Ok(args) if args.live => run_live(&args, stdin),
            Ok(args) => {
                let typed = nothing_piped;
                if let Some(test) = samples_instead(&args, typed()) {
                    if test.len() == 1 {
                        *stderr += "cpl: no input named or piped: running the samples \
                                    (name a file, or pipe one, to run once)\n";
                    }
                    let mut errors = Vec::new();
                    let code = cpl_contest::test::run(&test, stdout, &mut errors);
                    *stderr += &String::from_utf8_lossy(&errors);
                    if test.len() == 1 {
                        *stderr += &next_after_test(args.file, code);
                    }
                    return code;
                }
                let pinned = (args.trace && args.input.is_none() && typed())
                    .then(|| trace::pinned_input(std::path::Path::new(args.file)))
                    .flatten();
                if let Some(path) = &pinned {
                    *stderr += &format!(
                        "cpl: input {} (the case saved last for this file)\n",
                        path.display()
                    );
                }
                let named = pinned.as_ref().and_then(|p| p.to_str()).or(args.input);
                let input = match named {
                    None => Input::Live(stdin),
                    Some(input) => match std::fs::read(input) {
                        Ok(input) => Input::Whole(input),
                        Err(error) => {
                            *stderr += &format!("cpl: can't read `{input}`: {error}\n");
                            return EXIT_USAGE;
                        }
                    },
                };
                run_file(&args, input, stdout, stderr)
            }
            Err(message) => usage_error(message),
        },
        Some("judge") => {
            let json = rest.contains(&"--json");
            let args: Vec<&str> = rest.iter().copied().filter(|a| *a != "--json").collect();
            match args.as_slice() {
                [file, input] => judge_file(file, input, json, stdout, stderr),
                [file, input, output] | [file, input, output, _] => {
                    check_output(file, input, output, json, stdout, stderr)
                }
                _ => usage_error(
                    "`judge` takes a file and the hidden test: `cpl judge a.cp in.txt`".into(),
                ),
            }
        }
        // `cpl stress` traces a failure in the debug interpreter, which
        // this crate links and `cpl-stress` does not.
        Some("stress") => {
            let args: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            let mut errors = Vec::new();
            let code =
                cpl_stress::cli::run_with(&args, stdout, &mut errors, Some(&trace::against_brute));
            *stderr += &String::from_utf8_lossy(&errors);
            code
        }
        Some("locate") => {
            let args: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            let mut errors = Vec::new();
            let code = cpl_stress::locate::run_with(
                &args,
                stdout,
                &mut errors,
                Some(&trace::against_brute),
            );
            *stderr += &String::from_utf8_lossy(&errors);
            code
        }
        Some("build") => {
            let checked = rest.contains(&"--checked");
            let size = rest.contains(&"--size");
            let copy = rest.contains(&"--copy");
            let json = rest.contains(&"--json");
            let mut rest: Vec<&str> = rest
                .iter()
                .copied()
                .filter(|arg| !matches!(*arg, "--checked" | "--size" | "--copy" | "--json"))
                .collect();
            let judge = match take_judge(&mut rest, "build") {
                Ok(judge) => judge,
                Err(message) => return usage_error(message),
            };
            let checks = match build::take_checks(&mut rest, checked) {
                Ok(checks) => checks,
                Err(message) => return usage_error(message),
            };
            let (file, out) =
                match rest.as_slice() {
                    [file] => (*file, None),
                    [file, "-o", out] | ["-o", out, file] => (*file, Some(*out)),
                    _ => return usage_error(
                        "`build` takes one file: `cpl build a.cp`, `cpl build a.cp -o a.cpp` or \
                         `cpl build a.cp --checked`"
                            .into(),
                    ),
                };
            let options = BuildOptions {
                out,
                judge,
                checked,
                checks,
                size,
                copy,
                json,
            };
            build_file(file, &options, stdout, stderr)
        }
        Some("submit") => {
            let copy = rest.contains(&"--copy");
            let json = rest.contains(&"--json");
            let rest: Vec<&str> = rest
                .iter()
                .copied()
                .filter(|arg| !matches!(*arg, "--copy" | "--json"))
                .collect();
            match rest.as_slice() {
                [file] => submit(file, copy, json, stdout, stderr),
                _ => usage_error(
                    "`submit` takes one file, for the judge it names: `cpl submit a.cp` \
                     (a `// judge:` line or cp.toml says which)"
                        .into(),
                ),
            }
        }
        Some("judges") => {
            let json = rest.contains(&"--json");
            match rest
                .iter()
                .filter(|arg| **arg != "--json")
                .collect::<Vec<_>>()[..]
            {
                [] => list_judges(None, json, stdout, stderr),
                [file] => list_judges(Some(file), json, stdout, stderr),
                _ => usage_error(
                    "`judges` takes at most a file: `cpl judges` or `cpl judges a.cp`".into(),
                ),
            }
        }
        Some("test") if should_run_unit_tests(&rest) => run_unit_tests(&rest, stdout, stderr),
        Some(command) if tool(command).is_some() => {
            let args: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            let mut errors = Vec::new();
            let code = tool(command).unwrap()(&args, stdout, &mut errors);
            *stderr += &String::from_utf8_lossy(&errors);
            let flags = args.iter().any(|a| a.starts_with("--"));
            if let Some(file) = args.first().filter(|_| command == "test" && !flags) {
                *stderr += &next_after_test(file, code);
            }
            code
        }
        Some("lsp") => usage_error("`lsp` talks to an editor over stdio and takes no files".into()),
        Some("dap") => usage_error("`dap` talks to an editor over stdio and takes no files".into()),
        Some(other) if other.ends_with(".cp") || std::path::Path::new(other).is_file() => {
            usage_error(format!(
                "`{other}` is a file, not a command: `cpl run {other}` runs it"
            ))
        }
        Some(other) => usage_error(match commands::guess(other) {
            Some(guess) => format!("unknown command `{other}`; did you mean `cpl {guess}`?"),
            None => format!("unknown command `{other}`"),
        }),
    }
}

/// What `cpl run` was asked for.
struct RunArgs<'a> {
    file: &'a str,
    input: Option<&'a str>,
    trace: bool,
    /// `--trace SPEC`: fn globs / `!` exclusions to record.
    only_fn: Option<&'a str>,
    /// `--test SPEC`: 1-based multitest case ranges to record.
    test_cases: Vec<(u64, u64)>,
    /// `--watch VAR`: top-level variable names to watch.
    watch: Vec<String>,
    /// `--last N`: keep only the last N steps.
    last_steps: Option<usize>,
    /// `--from N`: skip the first N steps, so any part of a long run can be kept.
    from_step: Option<usize>,
    /// `--depth N`: maximum call depth to record.
    max_depth: Option<usize>,
    format: Option<&'a str>,
    brute: bool,
    release: bool,
    /// `--checks LEVEL` with `--release`: the judge build's checks.
    checks: Option<cpl_codegen::Checks>,
    live: bool,
    /// `--time-limit MS`: how long the run may take before it is stopped.
    time_limit: Option<std::time::Duration>,
}

fn parse_test_cases(spec: &str) -> Result<Vec<(u64, u64)>, String> {
    let mut out = Vec::new();
    for part in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        if let Some((a, b)) = part.split_once("..=") {
            let lo: u64 = a
                .parse()
                .map_err(|_| format!("`--test` expects a positive case number, not `{part}`"))?;
            let hi: u64 = b
                .parse()
                .map_err(|_| format!("`--test` expects a positive case number, not `{part}`"))?;
            if lo == 0 || hi < lo {
                return Err(format!("`--test` range `{part}` is empty or starts at 0"));
            }
            out.push((lo, hi));
        } else {
            let n: u64 = part
                .parse()
                .map_err(|_| format!("`--test` expects a positive case number, not `{part}`"))?;
            if n == 0 {
                return Err("`--test` case numbers are 1-based".into());
            }
            out.push((n, n));
        }
    }
    if out.is_empty() {
        return Err("`--test` needs a case number or range (e.g. `3` or `2..=4`)".into());
    }
    Ok(out)
}

fn is_trace_spec(token: &str) -> bool {
    !token.is_empty()
        && !token.starts_with('-')
        && token
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '*' | '?' | '!' | ','))
        && !std::path::Path::new(token).exists()
}

impl<'a> RunArgs<'a> {
    fn parse(rest: &[&'a str]) -> Result<RunArgs<'a>, String> {
        let mut args = RunArgs {
            file: "",
            input: None,
            trace: false,
            only_fn: None,
            test_cases: Vec::new(),
            watch: Vec::new(),
            last_steps: None,
            from_step: None,
            max_depth: None,
            format: None,
            brute: false,
            release: false,
            checks: None,
            live: false,
            time_limit: None,
        };
        let mut paths = Vec::new();
        let mut iter = rest.iter().copied().peekable();
        while let Some(arg) = iter.next() {
            match arg {
                "--trace" => {
                    args.trace = true;
                    // A fn spec after it that is not an existing file is the fn filter.
                    if let Some(next) = iter.peek().copied() {
                        if is_trace_spec(next) {
                            args.only_fn = Some(next);
                            iter.next();
                        }
                    }
                }
                "--test" => match iter.next() {
                    Some(spec) => args.test_cases.extend(parse_test_cases(spec)?),
                    None => return Err("`--test` needs a case number or range".into()),
                },
                flag if flag.starts_with("--test=") => {
                    args.test_cases
                        .extend(parse_test_cases(&flag["--test=".len()..])?);
                }
                "--watch" => match iter.next() {
                    Some(spec) => {
                        for name in spec.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                            args.watch.push(name.to_string());
                        }
                    }
                    None => return Err("`--watch` needs a variable name".into()),
                },
                flag if flag.starts_with("--watch=") => {
                    for name in flag["--watch=".len()..]
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    {
                        args.watch.push(name.to_string());
                    }
                }
                "--last" => match iter.next() {
                    Some(n) => {
                        let parsed: usize = n.parse().map_err(|_| {
                            format!("`--last` expects a positive integer, not `{n}`")
                        })?;
                        if parsed == 0 {
                            return Err("`--last` must be at least 1".into());
                        }
                        args.last_steps = Some(parsed);
                    }
                    None => return Err("`--last` needs a step count".into()),
                },
                flag if flag.starts_with("--last=") => {
                    let n = &flag["--last=".len()..];
                    let parsed: usize = n
                        .parse()
                        .map_err(|_| format!("`--last` expects a positive integer, not `{n}`"))?;
                    if parsed == 0 {
                        return Err("`--last` must be at least 1".into());
                    }
                    args.last_steps = Some(parsed);
                }
                "--from" => match iter.next() {
                    Some(n) => args.from_step = Some(step_count("--from", n)?),
                    None => return Err("`--from` needs a step number".into()),
                },
                flag if flag.starts_with("--from=") => {
                    args.from_step = Some(step_count("--from", &flag["--from=".len()..])?);
                }
                "--depth" => match iter.next() {
                    Some(n) => {
                        let parsed: usize = n.parse().map_err(|_| {
                            format!("`--depth` expects a non-negative integer, not `{n}`")
                        })?;
                        args.max_depth = Some(parsed);
                    }
                    None => return Err("`--depth` needs a call depth".into()),
                },
                flag if flag.starts_with("--depth=") => {
                    let n = &flag["--depth=".len()..];
                    let parsed: usize = n.parse().map_err(|_| {
                        format!("`--depth` expects a non-negative integer, not `{n}`")
                    })?;
                    args.max_depth = Some(parsed);
                }
                "--brute" => args.brute = true,
                "--release" => args.release = true,
                "--checks" => {
                    match iter.next().and_then(cpl_codegen::Checks::parse) {
                        Some(level) => args.checks = Some(level),
                        None => {
                            return Err(format!(
                            "`--checks` takes one of {}: `cpl run a.cp --release --checks none`",
                            cpl_codegen::Checks::NAMES.map(|n| format!("`{n}`")).join(", ")
                        ))
                        }
                    }
                }
                "--live" => args.live = true,
                "--time-limit" => {
                    let ms = iter.next().and_then(|ms| ms.parse::<u64>().ok()).ok_or(
                        "`--time-limit` takes milliseconds: `cpl run a.cp --time-limit 2000`",
                    )?;
                    args.time_limit = Some(std::time::Duration::from_millis(ms));
                }
                "--format" => match iter.next() {
                    Some(format @ ("json" | "html" | "text")) => args.format = Some(format),
                    Some(other) => {
                        return Err(format!("`--format` is text, json or html, not `{other}`"))
                    }
                    None => return Err("`--format` needs a value: json or html".into()),
                },
                flag if flag.starts_with("--trace=") => {
                    args.trace = true;
                    args.only_fn = Some(&flag["--trace=".len()..]);
                }
                flag if flag.starts_with('-') && flag.len() > 1 => {
                    return Err(format!(
                        "`run` has no option `{flag}`; `cpl run --help` lists them"
                    ))
                }
                path => paths.push(path),
            }
        }
        match paths.as_slice() {
            [file] => args.file = file,
            [file, input] => (args.file, args.input) = (file, Some(*input)),
            _ => {
                return Err(
                    "`run` takes a file and at most one input: `cpl run a.cp in.txt`".into(),
                )
            }
        }
        let has_trace_filter = !args.test_cases.is_empty()
            || !args.watch.is_empty()
            || args.last_steps.is_some()
            || args.from_step.is_some()
            || args.max_depth.is_some();
        if has_trace_filter && !args.trace {
            return Err(
                "`--test`, `--watch`, `--last`, `--from`, and `--depth` filter a trace: add `--trace`".into(),
            );
        }
        if args.checks.is_some() && !args.release {
            return Err(
                "`--checks` picks what the judge build traps on: use it with `--release`".into(),
            );
        }
        if args.release && (args.trace || args.format.is_some()) {
            return Err("`--release` runs the built C++, which records no trace: drop `--trace` and `--format`".into());
        }
        if args.live && (!args.trace || args.format.is_some()) {
            return Err(
                "`--live` serves the trace viewer: use it with `--trace` and without `--format`"
                    .into(),
            );
        }
        if args.format == Some("html") && !args.trace {
            return Err("`--format html` is the trace viewer: add `--trace`".into());
        }
        Ok(args)
    }
}

/// A command served by one of the tooling crates.
pub type Tool = fn(&[String], &mut dyn Write, &mut dyn Write) -> u8;

/// The tooling crate behind `command`, if it is one of theirs.
pub fn tool(command: &str) -> Option<Tool> {
    Some(match command {
        "fmt" => json_out::fmt,
        "lint" => json_out::lint,
        "doc" => cpl_doc::cli::run,
        "stress" => cpl_stress::cli::run,
        "new" => cpl_contest::new::run,
        "fetch" => cpl_contest::fetch::run,
        "test" => cpl_contest::test::run,
        "bench" => cpl_contest::bench::run,
        "watch" => cpl_contest::watch::run,
        _ => return None,
    })
}

/// A file that passed the front end, lowered for the back ends.
struct Compiled {
    text: String,
    program: cpl_ir::Program,
    /// The file's `@interactor`, by name and lowered as a program.
    judge: Option<(String, cpl_ir::Program)>,
}

/// Which program of a file to lower.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Solution,
    /// The `@brute(main)` fn in place of the solution.
    Brute,
    /// The `@checker` fn.
    Checker,
}

/// Reads and checks `path` through the shared front end, then lowers it.
/// This is the gate `run` and `build` both go through: the same
/// [`cpl_check::Database`] the language server holds, so a file means the
/// same thing to every tool.
fn compile(path: &str, stderr: &mut String) -> Result<Compiled, u8> {
    compile_side(path, Side::Solution, stderr)
}

fn compile_side(path: &str, side: Side, stderr: &mut String) -> Result<Compiled, u8> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            *stderr += &format!("cpl: can't read `{path}`: {error}\n");
            return Err(EXIT_USAGE);
        }
    };
    compile_text(path, text, side, stderr)
}

/// Checks and lowers `text` as the file at `path`.
fn compile_text(path: &str, text: String, side: Side, stderr: &mut String) -> Result<Compiled, u8> {
    // One database per thread, as the language server keeps one: a process
    // that compiles many files (the test suites, `cpl watch`) checks the
    // standard library once. The loader rereads imports on every analysis.
    thread_local! {
        static DB: std::cell::RefCell<cpl_check::Database> = Default::default();
    }
    let analysis = DB.with_borrow_mut(|db| {
        db.set_file(path, text.as_str());
        db.analyze(path).expect("the file was just set")
    });
    if !analysis.diagnostics.is_empty() {
        *stderr += &cpl_diag::render_all(&analysis.diagnostics, path, &text);
        if analysis
            .diagnostics
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error)
        {
            return Err(EXIT_ERRORS);
        }
    }
    let missing = |what: &str, stderr: &mut String| {
        *stderr += &format!("cpl: `{path}` declares no `{what}` fn\n");
        EXIT_USAGE
    };
    let lowered = match side {
        Side::Solution => cpl_ir::lower(&analysis).and_then(|program| {
            let judge = match cpl_ir::interactor(&analysis) {
                Some(name) => Some((name.clone(), cpl_ir::lower_judge(&analysis, &name)?)),
                None => None,
            };
            Ok((program, judge))
        }),
        Side::Brute => match cpl_ir::brute(&analysis) {
            Some((name, _)) => cpl_ir::lower_brute(&analysis, &name).map(|program| (program, None)),
            None => return Err(missing("@brute(main)", stderr)),
        },
        Side::Checker => match cpl_ir::checker(&analysis) {
            Some((name, _)) => {
                cpl_ir::lower_checker(&analysis, &name).map(|program| (program, None))
            }
            None => return Err(missing("@checker", stderr)),
        },
    };
    match lowered {
        Ok((program, judge)) => {
            *stderr += &cpl_diag::render_all(&program.warnings, path, &text);
            Ok(Compiled {
                text,
                program,
                judge,
            })
        }
        Err(diagnostics) => {
            *stderr += &cpl_diag::render_all(&diagnostics, path, &text);
            Err(EXIT_ERRORS)
        }
    }
}

/// `cpl judge`: the file's `@interactor`, talking to a solution on
/// standard input and output. The first line of what it says on failure
/// is the reason, which is what `cpl stress` reports.
fn judge_file(
    path: &str,
    input: &str,
    json: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let input = match std::fs::read(input) {
        Ok(input) => input,
        Err(error) => {
            *stderr += &format!("cpl: can't read `{input}`: {error}\n");
            return EXIT_USAGE;
        }
    };
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let Some((_, judge)) = &compiled.judge else {
        *stderr += &format!("cpl: `{path}` declares no `@interactor`\n");
        return EXIT_USAGE;
    };
    let result = interact::judge_alone(judge, &input);
    verdict(path, &compiled.text, result, json, stdout, stderr)
}

/// `cpl judge <file> <input> <output>`: the file's `@checker` on what a
/// solution printed. The first line it says on failure is the reason.
fn check_output(
    path: &str,
    input: &str,
    output: &str,
    json: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let read = |file: &str, stderr: &mut String| match std::fs::read(file) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            *stderr += &format!("cpl: can't read `{file}`: {error}\n");
            Err(EXIT_USAGE)
        }
    };
    let (input, output) = match (read(input, stderr), read(output, stderr)) {
        (Ok(input), Ok(output)) => (input, output),
        (Err(code), _) | (_, Err(code)) => return code,
    };
    let compiled = match compile_side(path, Side::Checker, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let mut printed = Some(output);
    let io = cpl_interp::Io::judge(&input, Box::new(move || printed.take()));
    let mut sink = Vec::new();
    let result = cpl_interp::run_with(&compiled.program, io, &mut sink);
    verdict(path, &compiled.text, result, json, stdout, stderr)
}

/// What `cpl judge` says of the judge's verdict: nothing when it accepts,
/// else the reason and where it failed; with `--json`, one object with
/// `accepted`, `reason` and `diagnostic`.
fn verdict(
    path: &str,
    text: &str,
    result: Result<(), cpl_interp::Trap>,
    json: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let code = if result.is_ok() { 0 } else { EXIT_FAILED };
    if json {
        let lines = cpl_diag::LineIndex::new(text);
        let (reason, diagnostic) = match &result {
            Ok(()) => (cpl_json::Value::Null, cpl_json::Value::Null),
            Err(trap) => (
                trap.message.as_str().into(),
                json_out::diagnostic(&trap.to_diagnostic(path, &lines), &lines),
            ),
        };
        let report = cpl_json::obj([
            ("accepted", (code == 0).into()),
            ("reason", reason),
            ("diagnostic", diagnostic),
        ]);
        let _ = writeln!(stdout, "{}", cpl_json::to_string(&report));
    } else if let Err(trap) = &result {
        *stderr += &format!("{}\n", trap.message);
        *stderr += &render_trap(path, text, trap);
    }
    code
}

fn render_trap(path: &str, text: &str, trap: &cpl_interp::Trap) -> String {
    render_trap_noting(path, text, trap, None)
}

/// The trap's diagnostic, with a note naming `locals` when there are some.
fn render_trap_noting(
    path: &str,
    text: &str,
    trap: &cpl_interp::Trap,
    locals: Option<String>,
) -> String {
    let lines = cpl_diag::LineIndex::new(text);
    let mut diagnostic = trap.to_diagnostic(path, &lines);
    if let Some(locals) = locals {
        diagnostic = diagnostic.with_note(format!("locals: {locals}"));
    }
    cpl_diag::render(&diagnostic, path, text, &lines)
}

/// The `@interactor` of the file at `path`, lowered, with the file's
/// text; `None` if the file has none or is not CP source.
fn release_judge(
    path: &str,
    stderr: &mut String,
) -> Result<Option<(String, cpl_ir::Program, String)>, u8> {
    if !path.ends_with(".cp") {
        return Ok(None);
    }
    let compiled = compile_side(path, Side::Solution, &mut String::new())
        .inspect_err(|_| *stderr += &format!("cpl: `{path}` no longer checks\n"))?;
    Ok(compiled
        .judge
        .map(|(name, judge)| (name, judge, compiled.text)))
}

/// `cpl run --release` on an interactive file: the built solution, as a
/// process, against the file's `@interactor`.
#[allow(clippy::too_many_arguments)]
fn run_release_judged(
    args: &RunArgs,
    program: &cpl_stress::Program,
    name: &str,
    judge: &cpl_ir::Program,
    text: &str,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let ruling = match interact::run_built(&program.argv, &program.env, judge, input, stdout) {
        Ok(ruling) => ruling,
        Err(error) => {
            *stderr += &format!("cpl: can't run the build of `{}`: {error}\n", args.file);
            return EXIT_USAGE;
        }
    };
    *stderr += &ruling.stderr;
    let solved = ruling.status.is_some_and(|status| status.success());
    if !solved {
        *stderr += &match ruling.status.and_then(|status| status.code()) {
            Some(code) => format!("cpl: the solution exited with status {code}\n"),
            None => format!(
                "cpl: the solution crashed; `cpl run {}` without `--release` says where\n",
                args.file
            ),
        };
    }
    // A judge left with nothing to read after the solution failed only
    // follows from that failure.
    let starved = |trap: &cpl_interp::Trap| trap.message.starts_with("the solution's output ended");
    match &ruling.judge {
        Some(trap) if solved || !starved(trap) => {
            *stderr += &format!("cpl: the interactor `{name}` rejected the run\n");
            *stderr += &render_trap(args.file, text, trap);
        }
        _ => {}
    }
    if solved && ruling.judge.is_none() {
        *stderr += &format!(
            "cpl: the interactor `{name}` accepted the run ({} lines from the solution)\n",
            ruling.lines
        );
        return 0;
    }
    EXIT_FAILED
}

/// What `cpl test` suggests next: `submit` once the samples pass; on a
/// failure, `stress` when the file has an `@alt` to compare.
/// The `cpl test` arguments `cpl run` stands for: every sample when no
/// input is named and stdin is a terminal, one sample when the input is a
/// sample's name rather than a file.
/// Stdin is a terminal, or a device such as `/dev/null` (a script, an
/// editor task or CI), so nothing was piped in to run on.
fn nothing_piped() -> bool {
    if INPUT_GIVEN.get() {
        return false;
    }
    let stdin = std::io::stdin();
    if std::io::IsTerminal::is_terminal(&stdin) {
        return true;
    }
    #[cfg(unix)]
    {
        use std::os::fd::AsFd;
        use std::os::unix::fs::FileTypeExt;
        let device = stdin
            .as_fd()
            .try_clone_to_owned()
            .ok()
            .and_then(|fd| std::fs::File::from(fd).metadata().ok())
            .is_some_and(|m| m.file_type().is_char_device());
        if device {
            return true;
        }
    }
    false
}

fn samples_instead(args: &RunArgs, typed: bool) -> Option<Vec<String>> {
    if args.trace || args.live || args.release || args.brute || args.format.is_some() {
        return None;
    }
    let source = cpl_contest::problem::source_path(args.file);
    let cases = cpl_contest::problem::cases_in(&cpl_contest::problem::find_tests_dir(&source));
    match args.input {
        None if typed && !cases.is_empty() => Some(vec![args.file.to_string()]),
        Some(name)
            if !std::path::Path::new(name).exists() && cases.iter().any(|c| c.name == name) =>
        {
            Some(vec![
                args.file.to_string(),
                "--case".into(),
                name.to_string(),
            ])
        }
        _ => None,
    }
}

fn next_after_test(file: &str, code: u8) -> String {
    let src = std::fs::read_to_string(cpl_contest::problem::source_path(file)).unwrap_or_default();
    match code {
        0 => format!(
            "next: `cpl submit {file}` builds what the judge gets and checks it on these cases\n"
        ),
        1 if src.contains("@alt(") => {
            format!("next: `cpl stress {file}` looks for a small case where the brute disagrees\n")
        }
        _ => String::new(),
    }
}

/// `cpl run`: the program's output goes straight to `stdout`.
/// `cpl run --release`: the file's C++, built as a judge builds it.
fn run_release(
    args: &RunArgs,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let path = std::path::Path::new(args.file);
    let built = match (args.brute, args.checks) {
        (true, _) => cpl_stress::Program::resolve_brute(path, "the brute"),
        (false, Some(checks)) => cpl_stress::Program::resolve_checks(path, checks),
        (false, None) => cpl_stress::Program::resolve(path),
    };
    let program = match built {
        Ok(program) => program,
        Err(cpl_stress::BuildError(message)) => {
            *stderr += &format!("cpl: {message}\n");
            return EXIT_ERRORS;
        }
    };
    // An interactive problem with its judge in the file: the input is the
    // hidden test, which the judge reads, as without `--release`.
    if !args.brute {
        match release_judge(args.file, stderr) {
            Ok(Some((name, judge, text))) => {
                return run_release_judged(
                    args, &program, &name, &judge, &text, input, stdout, stderr,
                )
            }
            Ok(None) => {}
            Err(code) => return code,
        }
    }
    // With no `--time-limit`, a day: longer than anyone waits.
    let limit = (args.time_limit).unwrap_or(std::time::Duration::from_secs(86_400));
    let outcome = match program.run(&String::from_utf8_lossy(input), limit) {
        Ok(outcome) => outcome,
        Err(error) => {
            *stderr += &format!("cpl: can't run the build of `{}`: {error}\n", args.file);
            return EXIT_USAGE;
        }
    };
    let _ = stdout.write_all(outcome.stdout.as_bytes());
    *stderr += &outcome.stderr;
    match outcome.exit {
        cpl_stress::Exit::Code(0) => 0,
        cpl_stress::Exit::Code(code) => {
            *stderr += &format!("cpl: the program exited with status {code}\n");
            EXIT_FAILED
        }
        cpl_stress::Exit::Signal | cpl_stress::Exit::TimedOut => {
            *stderr += &format!(
                "cpl: the program crashed; `cpl run {}` without `--release` says where\n",
                args.file
            );
            EXIT_FAILED
        }
    }
}

/// What `cpl run` reads: a file, or standard input as it arrives.
enum Input<'a> {
    Whole(Vec<u8>),
    /// Gives the next bytes, and nothing once the input has ended.
    Live(&'a mut (dyn FnMut() -> Vec<u8> + Send)),
}

impl Input<'_> {
    /// The whole input, waiting for its end.
    fn whole(self) -> Vec<u8> {
        match self {
            Input::Whole(input) => input,
            Input::Live(more) => {
                let mut input = Vec::new();
                loop {
                    let chunk = more();
                    if chunk.is_empty() {
                        return input;
                    }
                    input.extend(chunk);
                }
            }
        }
    }
}

fn run_file(
    args: &RunArgs,
    input: Input,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    if args.release {
        return run_release(args, &input.whole(), stdout, stderr);
    }
    let path = args.file;
    let side = if args.brute {
        Side::Brute
    } else {
        Side::Solution
    };
    let compiled = match compile_side(path, side, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let json = args.format == Some("json");
    if args.trace || json {
        return observe_file(args, &compiled, &input.whole(), stdout, stderr);
    }
    // An interactive problem with its judge in the file: the input is
    // the hidden test, and the two talk to each other.
    if let Some((name, judge)) = &compiled.judge {
        let ruling = interact::run(&compiled.program, judge, &input.whole(), stdout);
        // A side that stops leaves the other with nothing to read: that
        // second failure only follows from the first.
        let starved = |trap: &cpl_interp::Trap| {
            trap.message.starts_with("the input ended")
                || trap.message.starts_with("the solution's output ended")
        };
        let (judge_trap, solution_trap) = match (ruling.judge, ruling.solution) {
            (Some(judge), Some(solution)) if starved(&judge) && !starved(&solution) => {
                (None, Some(solution))
            }
            (Some(judge), Some(solution)) if starved(&solution) && !starved(&judge) => {
                (Some(judge), None)
            }
            both => both,
        };
        if judge_trap.is_none() && solution_trap.is_none() {
            *stderr += &format!(
                "cpl: the interactor `{name}` accepted the run ({} lines from the solution)\n",
                ruling.lines
            );
            return 0;
        }
        if let Some(trap) = &solution_trap {
            *stderr += "cpl: the solution failed\n";
            *stderr += &render_trap(path, &compiled.text, trap);
        }
        if let Some(trap) = &judge_trap {
            *stderr += &format!("cpl: the interactor `{name}` rejected the run\n");
            *stderr += &render_trap(path, &compiled.text, trap);
        }
        return EXIT_FAILED;
    }
    // Standard input is read as the program asks for it, so a program
    // that answers a person or a judge at the other end of a pipe is not
    // left waiting for the input to end.
    // What it read is kept, to run it again and name its locals if it traps.
    let read = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let whole;
    let io = match input {
        Input::Whole(input) => {
            whole = input;
            cpl_interp::Io::fixed(&whole)
        }
        Input::Live(more) => {
            whole = Vec::new();
            let read = std::sync::Arc::clone(&read);
            cpl_interp::Io::live(Box::new(move || {
                let chunk = more();
                read.lock().unwrap().extend_from_slice(&chunk);
                (!chunk.is_empty()).then_some(chunk)
            }))
        }
    };
    let io = match args.time_limit {
        Some(limit) => io.until(cpl_interp::Stop::after(limit)),
        None => io,
    };
    let started = std::time::Instant::now();
    let observed = cpl_interp::run_observed(&compiled.program, io, stdout, None);
    *stderr += &String::from_utf8_lossy(&observed.debug);
    match observed.trap {
        None => 0,
        Some(trap) => {
            // The recorded run is several times slower: only a quick one is repeated.
            let locals = (started.elapsed() < TRAP_RERUN).then(|| {
                let read = read.lock().unwrap();
                let input = if whole.is_empty() {
                    &read[..]
                } else {
                    &whole[..]
                };
                trace::trap_locals(&compiled.program, &compiled.text, input, &trap)
            });
            *stderr += &render_trap_noting(path, &compiled.text, &trap, locals.flatten());
            EXIT_FAILED
        }
    }
}

/// The longest run whose trap is run again, recorded, to name its locals.
const TRAP_RERUN: std::time::Duration = std::time::Duration::from_secs(1);

/// `cpl run --trace --live`: the input is read once and pinned; with no
/// input named, it is the case `cpl stress` last saved for the file.
fn run_live(args: &RunArgs, stdin: &mut (dyn FnMut() -> Vec<u8> + Send)) -> u8 {
    let pinned = args
        .input
        .map(std::path::PathBuf::from)
        .or_else(|| trace::pinned_input(std::path::Path::new(args.file)));
    let (input, note) = match &pinned {
        Some(path) => match std::fs::read(path) {
            Ok(input) => (input, path.display().to_string()),
            Err(error) => {
                eprintln!("cpl: can't read `{}`: {error}", path.display());
                return EXIT_USAGE;
            }
        },
        None => (Input::Live(stdin).whole(), "standard input".to_string()),
    };
    let side = if args.brute {
        Side::Brute
    } else {
        Side::Solution
    };
    let record = |from: Option<usize>| {
        let mut stderr = String::new();
        let compiled = compile_side(args.file, side, &mut stderr).map_err(|_| stderr.clone())?;
        let config = cpl_interp::trace::TraceConfig {
            from_step: from.or(args.from_step),
            ..trace_config(args, &compiled.text)
        };
        let observed = traced(&compiled.program, &input, &mut Vec::new(), config);
        Ok(trace::single(
            args.file,
            &compiled.text,
            &observed.recording.unwrap_or_default(),
        ))
    };
    live::serve(args.file, &input, &note, &record)
}

/// The run on `input`, recorded; `--last` traces only the run's end again,
/// from a checkpoint of its untraced run.
fn traced(
    program: &cpl_ir::Program,
    input: &[u8],
    output: &mut (dyn Write + Send),
    config: cpl_interp::trace::TraceConfig,
) -> cpl_interp::Observed {
    match config.last_steps.filter(|_| config.from_step.is_none()) {
        Some(steps) => cpl_interp::trace::resumed_tail(program, input, output, config, steps),
        None => {
            cpl_interp::run_observed(program, cpl_interp::Io::fixed(input), output, Some(config))
        }
    }
}

fn trace_config(args: &RunArgs, text: &str) -> cpl_interp::trace::TraceConfig {
    let mut config = trace::config(text, args.only_fn);
    config.test_cases = args.test_cases.clone();
    config.watch = args.watch.clone();
    config.last_steps = args.last_steps;
    config.from_step = args.from_step;
    config.max_depth = args.max_depth;
    config
}

fn step_count(flag: &str, n: &str) -> Result<usize, String> {
    n.parse()
        .map_err(|_| format!("`{flag}` expects a step number from 0, not `{n}`"))
}

/// `cpl run --trace` and `cpl run --format json`: the run, recorded.
/// An interactive file is recorded on the solution's side, against its
/// `@interactor`.
fn observe_file(
    args: &RunArgs,
    compiled: &Compiled,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let path = args.file;
    let started = std::time::Instant::now();
    let config = trace_config(args, &compiled.text);
    // The program's output is kept: with `--format` the report or the
    // trace is what goes to standard output.
    let mut printed = Vec::new();
    let quiet = args.format.is_some_and(|format| format != "text");
    let observed = match &compiled.judge {
        Some((_, judge)) => {
            interact::run_traced(&compiled.program, judge, input, &mut printed, config)
        }
        None => traced(&compiled.program, input, &mut printed, config),
    };
    let millis = started.elapsed().as_millis();
    let recording = observed.recording.unwrap_or_default();
    *stderr += &String::from_utf8_lossy(&observed.debug);
    let lines = cpl_diag::LineIndex::new(&compiled.text);
    let code = match &observed.trap {
        None => 0,
        Some(_) => EXIT_FAILED,
    };
    if !quiet {
        let _ = stdout.write_all(&printed);
    }
    if let (Some(trap), false) = (&observed.trap, args.format == Some("json") && !args.trace) {
        *stderr += &render_trap(path, &compiled.text, trap);
    }
    if !args.trace {
        let error = observed
            .trap
            .as_ref()
            .map(|trap| (lines.line_of(trap.span.start) + 1, trap.message.as_str()));
        let _ = stdout.write_all(trace::run_report(&recording, &printed, millis, error).as_bytes());
        return code;
    }
    let document = trace::single(path, &compiled.text, &recording);
    match args.format {
        Some("json") => {
            let _ = stdout.write_all(document.as_bytes());
            let _ = stdout.write_all(b"\n");
        }
        Some("html") => {
            let _ = stdout.write_all(trace::html(&document).as_bytes());
        }
        _ => {
            let target = std::path::Path::new(path).with_extension("trace.html");
            match std::fs::write(&target, trace::html(&document)) {
                Ok(()) => {
                    *stderr += &format!(
                        "cpl: wrote {} ({} steps); open it in a browser\n",
                        target.display(),
                        recording.len()
                    )
                }
                Err(error) => {
                    *stderr += &format!("cpl: can't write `{}`: {error}\n", target.display());
                    return EXIT_USAGE;
                }
            }
        }
    }
    code
}

/// Serves the Debug Adapter Protocol on stdin and stdout until the editor
/// ends the session. Returns the exit status.
pub fn dap() -> u8 {
    dap::serve_stdio()
}

/// Serves the language server on stdin and stdout until the editor ends
/// the session. Returns the exit status.
pub fn lsp() -> u8 {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    match cpl_lsp::serve(stdin.lock(), stdout.lock()) {
        Ok(code) => code as u8,
        Err(error) => {
            eprintln!("cpl lsp: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_with_no_input_runs_the_samples_and_a_sample_name_runs_one() {
        let dir = std::env::temp_dir().join(format!("cpl-run-samples-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a.tests")).unwrap();
        std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
        std::fs::write(dir.join("a.tests/1.in"), "").unwrap();
        let file = dir.join("a.cp");
        let file = file.to_str().unwrap();
        let samples = |rest: &[&str], typed| samples_instead(&RunArgs::parse(rest).unwrap(), typed);
        assert_eq!(samples(&[file], true), Some(vec![file.to_string()]));
        assert_eq!(samples(&[file], false), None, "piped input runs once");
        let one = Some(vec![file.to_string(), "--case".into(), "1".into()]);
        assert_eq!(samples(&[file, "1"], false), one);
        assert_eq!(
            samples(&[file, "9"], false),
            None,
            "not a sample: read as a file"
        );
        assert_eq!(samples(&[file, "--trace"], true), None);
        assert_eq!(samples(&[file, "--release"], true), None);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn version_prints_the_binary_name() {
        let out = run(&["--version"]);
        assert_eq!(out.code, 0);
        assert_eq!(out.stdout, format!("{}\n", version_line()).into_bytes());
        assert!(version_line().starts_with(&format!("cpl {VERSION}-{CHANNEL}+")));
    }

    #[test]
    fn no_arguments_prints_usage() {
        let out = run::<&str>(&[]);
        assert_eq!(out.code, 0);
        assert!(out.stdout.starts_with(b"usage: cpl "));
    }

    #[test]
    fn unknown_command_fails_with_usage_on_stderr() {
        let out = run(&["frobnicate"]);
        assert_eq!(out.code, 2);
        assert!(out.stdout.is_empty());
        assert!(out.stderr.starts_with("cpl: unknown command `frobnicate`"));
    }

    #[test]
    fn tooling_commands_reach_their_crates() {
        for command in [
            "check", "fix", "run", "build", "judge", "submit", "judges", "explain", "lsp", "fmt",
            "lint", "doc", "stress", "new", "fetch", "test", "bench", "watch", "migrate",
        ] {
            let out = run(&[command, "--help"]);
            assert_eq!(out.code, 0, "{command}");
            let help = String::from_utf8(out.stdout).unwrap();
            assert!(
                help.starts_with(&format!("usage: cpl {command}")),
                "{command}: {help}"
            );
            assert!(
                main_usage().contains(&format!("\n  {command} ")),
                "{command} is not in the usage"
            );
        }
        assert!(tool("check").is_none());
        // A tool's own usage errors keep its exit code.
        let out = run(&["test"]);
        assert_eq!(out.code, EXIT_USAGE);
        assert!(
            out.stderr.contains("`test` needs a problem"),
            "{}",
            out.stderr
        );
    }

    #[test]
    fn fmt_and_lint_run_on_a_file() {
        let path = temp_file("tools.cp", "let n: 1..=9 = read()\nprint(n + 1)\n");
        let fmt = run(&["fmt", "--check", &path]);
        assert_eq!(fmt.code, 0, "{}", fmt.stderr);
        let lint = run(&["lint", &path]);
        assert_eq!(lint.code, 0, "{}", lint.stderr);
    }

    fn temp_file(name: &str, text: &str) -> String {
        let dir = std::env::temp_dir().join(format!("cpl-cli-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, text).unwrap();
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn check_passes_a_clean_file() {
        let path = temp_file("clean.cp", "let n: 1..=9 = read()\nprint(n + 1)\n");
        let out = run(&["check", &path]);
        assert_eq!((out.code, out.stderr.as_str()), (0, ""));
        assert_eq!(out.stdout, format!("{path}: no errors\n").into_bytes());
    }

    #[test]
    fn check_renders_errors_and_fails() {
        let path = temp_file(
            "bad.cp",
            "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nprint(a[n])\n",
        );
        let out = run(&["check", &path]);
        assert_eq!(out.code, EXIT_ERRORS);
        assert_eq!(
            out.stderr,
            format!(
                "error[E0301]: `a` is indexed by V, but `n` is a 1..=9\n  --> {path}:4:9\n  |\n1 | let n: 1..=9 = read()\n  |     - `n` is declared as a 1..=9 here\n3 | let a: [V]Int = read()\n  |     - `a` is declared over V here\n4 | print(a[n])\n  |         ^\n  = help: convert it with `V(n)`, or write `a.raw[n]` to index by number\n\n{path}: 1 error\n"
            )
        );
    }

    #[test]
    fn fix_applies_the_fixes_and_reports_what_is_left() {
        let path = temp_file(
            "fixable.cp",
            "let n = 3\nif n > 1 && n < 5:\n    print(n)\nprint(m)\n",
        );
        let out = run(&["fix", &path]);
        assert_eq!(out.code, EXIT_ERRORS);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "let n = 3\nif n > 1 and n < 5:\n    print(n)\nprint(m)\n"
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            format!("{path}: 1 fix applied, 1 left\n")
        );
        assert!(out.stderr.contains("no `m` in scope"), "{}", out.stderr);

        let path = temp_file(
            "lintable.cp",
            "let x = read::<Int>()\nif x > 1 && x < 5:\n    print(x)\n",
        );
        let out = run(&["fix", &path]);
        assert_eq!(out.code, 0);
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "let x = read<Int>()\nif x > 1 and x < 5:\n    print(x)\n"
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            format!("{path}: 2 fixes applied, no errors\n")
        );
    }

    #[test]
    fn run_stops_at_its_time_limit() {
        let path = temp_file(
            "forever.cp",
            "var i = 0\nwhile i >= 0:\n    i = (i + 1) % 2\nprint(i)\n",
        );
        let started = std::time::Instant::now();
        let out = run(&["run", &path, "--time-limit", "200"]);
        assert!(started.elapsed().as_secs() < 5, "{:?}", started.elapsed());
        assert_eq!(out.code, EXIT_FAILED, "{}", out.stderr);
        assert!(
            out.stderr.contains(cpl_interp::OUT_OF_TIME),
            "{}",
            out.stderr
        );
        let out = run(&["run", &path, "--time-limit", "soon"]);
        assert_eq!(out.code, EXIT_USAGE, "{}", out.stderr);
    }

    #[test]
    fn run_and_build_go_through_the_same_check() {
        let bad = temp_file("bad_run.cp", "print(x)\n");
        for command in ["run", "build"] {
            let out = run(&[command, &bad]);
            assert_eq!(out.code, EXIT_ERRORS);
            assert_eq!(out.stderr, run(&["check", &bad]).stderr);
        }
    }

    /// The output with `\r\n` as `\n`: the built C++ writes text-mode
    /// stdout, which is `\r\n` on Windows.
    fn lf(bytes: &[u8]) -> Vec<u8> {
        String::from_utf8_lossy(bytes)
            .replace("\r\n", "\n")
            .into_bytes()
    }

    #[test]
    fn release_runs_the_built_cpp_and_checked_builds_a_sanitized_program() {
        if std::process::Command::new("g++")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let path = temp_file("release.cp", "let n: Int = read()\nprint(n * 2)\n");
        let out = run_with_input(&["run", &path, "--release"], b"21\n");
        assert_eq!(
            (out.code, lf(&out.stdout).as_slice()),
            (0, b"42\n".as_slice()),
            "{}",
            out.stderr
        );
        let out = run(&["run", &path, "--release", "--trace"]);
        assert_eq!(out.code, EXIT_USAGE);
        // `--checks` picks what the release build traps on, as for `build`.
        let divide = temp_file("divide.cp", "let d: Int = read()\nprint(6 / d)\n");
        let out = run_with_input(&["run", &divide, "--release"], b"0\n");
        assert_eq!(out.code, EXIT_FAILED, "{}", out.stderr);
        assert!(out.stderr.contains("division by zero"), "{}", out.stderr);
        let out = run_with_input(&["run", &divide, "--release", "--checks", "none"], b"0\n");
        assert_eq!(
            (out.code, lf(&out.stdout).as_slice()),
            (0, b"0\n".as_slice()),
            "{}",
            out.stderr
        );
        let out = run(&["run", &divide, "--checks", "none"]);
        assert_eq!(out.code, EXIT_USAGE);
        assert!(
            out.stderr.contains("use it with `--release`"),
            "{}",
            out.stderr
        );
        let out = run(&["run", &divide, "--release", "--checks", "some"]);
        assert_eq!(out.code, EXIT_USAGE);
        let out = run(&["build", &path, "--checked"]);
        assert_eq!(out.code, 0, "{}", out.stderr);
        assert!(std::path::Path::new(&path)
            .with_extension("checked")
            .is_file());
        assert_eq!(
            run(&["build", &path, "--checked", "-o", "-"]).code,
            EXIT_USAGE
        );
    }

    /// The input of an interactive file is the hidden test, for the
    /// judge: the built solution must never read it as the judge's replies.
    #[test]
    fn release_runs_the_built_cpp_against_the_interactor() {
        if std::process::Command::new("g++")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let guesser = |answer: &str| {
            format!(
                "let n: Int = read()\nprint(\"?\", n)\nflush()\nlet hidden: Int = read()\nprint(\"!\", {answer})\n\n\
                 @interactor\nfn judge(mut sol: Interactor):\n    let (n, hidden): (Int, Int) = read()\n    sol.send(n)\n    \
                 let q: Str = sol.recv()\n    let asked: Int = sol.recv()\n    assert q == \"?\" and asked == n\n    \
                 sol.send(hidden)\n    let a: Str = sol.recv()\n    let said: Int = sol.recv()\n    assert said == hidden + 1\n"
            )
        };
        let right = temp_file("release_judged.cp", &guesser("hidden + 1"));
        for flags in [&["run", &right][..], &["run", &right, "--release"][..]] {
            let out = run_with_input(flags, b"7 41\n");
            assert_eq!(
                (out.code, lf(&out.stdout).as_slice()),
                (0, b"? 7\n! 42\n".as_slice()),
                "{}",
                out.stderr
            );
            assert!(
                out.stderr.contains("accepted the run (2 lines"),
                "{}",
                out.stderr
            );
        }
        let wrong = temp_file("release_misjudged.cp", &guesser("hidden"));
        let out = run_with_input(&["run", &wrong, "--release"], b"7 41\n");
        assert_eq!(out.code, EXIT_FAILED, "{}", out.stderr);
        assert!(
            out.stderr
                .contains("the interactor `judge` rejected the run"),
            "{}",
            out.stderr
        );
    }

    #[test]
    fn missing_files_and_arguments_are_usage_errors() {
        let out = run(&["check", "/nonexistent/x.cp"]);
        assert_eq!(out.code, EXIT_USAGE);
        assert!(out
            .stderr
            .starts_with("cpl: can't read `/nonexistent/x.cp`"));
        assert_eq!(run(&["check"]).code, EXIT_USAGE);
        assert_eq!(run(&["run"]).code, EXIT_USAGE);
    }

    #[test]
    fn every_pipeline_crate_is_linked() {
        assert_eq!(COMPONENTS.len(), 12);
    }

    #[test]
    fn explain_lists_registry_and_looks_up_codes() {
        let e = run(&["explain", "E0301"]);
        assert_eq!(e.code, 0, "{}", e.stderr);
        let text = String::from_utf8(e.stdout).unwrap();
        assert!(text.contains("E0301"), "{text}");
        assert!(text.contains("Fix:"), "{text}");

        let l = run(&["explain", "L0501"]);
        assert_eq!(l.code, 0, "{}", l.stderr);
        let ltext = String::from_utf8(l.stdout).unwrap();
        assert!(
            ltext.contains("L0501") && ltext.contains("search_loop"),
            "{ltext}"
        );

        let j = run(&["explain", "--json"]);
        assert_eq!(j.code, 0, "{}", j.stderr);
        let json = String::from_utf8(j.stdout).unwrap();
        assert!(json.starts_with('[') && json.trim_end().ends_with(']'));
        assert!(json.contains("\"E0301\"") && json.contains("\"L0506\""));

        assert_eq!(run(&["explain", "E9998"]).code, EXIT_USAGE);
    }

    #[test]
    fn explain_file_shows_view_representations_and_costs() {
        let dir = std::env::temp_dir().join("cpl_explain_views_test");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("views.cp");
        std::fs::write(
            &file,
            "let s = \"abcdef\"\nlet xs = [1, 2, 3]\nvar ys = [3, 1, 2]\nys[0..2].sort()\nprint(s[1..4], s.rev, xs.rev, s[1..4].to_str(), xs[0..2].to_list(), ys)\n",
        )
        .unwrap();
        let path = file.to_string_lossy().into_owned();
        let out = run(&["explain", &path]);
        assert_eq!(out.code, 0, "{}", out.stderr);
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("slice view -> cpl::StrView"), "{text}");
        assert!(text.contains("reverse view -> cpl::StrView"), "{text}");
        assert!(text.contains("reverse view -> cpl::ListView"), "{text}");
        assert!(
            text.contains("mutable slice view -> cpl::MutView<Int> (T* + len), O(1)"),
            "{text}"
        );
        assert!(text.contains("O(1), read-only"), "{text}");
        assert!(text.contains("explicit copy -> std::string"), "{text}");
        assert!(text.contains("explicit copy -> std::vector"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
