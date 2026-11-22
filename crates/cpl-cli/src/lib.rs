//! The `cpl` command-line tool. `cpl` is the working name of the binary
//! (`cp` clashes with Unix `cp`); source files end in `.cp`.

mod dap;
mod interact;
mod laws;
mod live;
mod profile;
mod replay;
mod report;
mod sha256;
pub mod toolchain;
mod trace;

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

const USAGE: &str = "\
usage: cpl <command> [args]

commands:
  check <file>...       report every error in the files
  fix <file>...         apply the fix each error suggests, then report what is left
  run <file> [input]    check, then run in the debug interpreter, reading
                        the input file or standard input; a file with an
                        `@interactor` runs against it, on the hidden test
                        (`cpl run --help` has the options for traces);
                        `--release` runs the C++ that `cpl build` writes
  judge <file> <input> [<output> <answer>]
                        run the file's `@interactor` as a judge program on
                        standard input and output, or with an output its
                        `@checker`; exit status 0 accepts
  build <file> [-o out] check, then write one C++17 file to submit: next to
                        the source by default, to standard output with `-o -`;
                        `--checked` also builds it with sanitizers to run here;
                        `--judge <name>` writes it for that judge
  judges [file]         list the judge profiles, marking the one a build uses
  submit <file>         build for the file's judge, run its cases on that
                        build, and say whether it is ready to send
  fmt [path]...         format files in the one canonical style
  lint [path]...        warn about code that is legal but probably wrong or slow
  explain <CODE|file|--json> show a diagnostic/lint explanation or a file's representations & view costs
  doc [path]...         build HTML documentation from `///` comments
  new <name>            create a problem from its constraints or samples
  test <problem>        run every case in <problem>.tests/
  stress <file>         run against a brute on random inputs; shrink the failure
  bench <problem>       run on maximum-size worst-case inputs against the limits
  profile <file> [input]
                        run, then show each line's hits and share of the time
  cover <file> [input]...
                        list the lines and branches no input reached
  watch <problem>       check, lint and test on every save
  report <file>         write a paste-ready bug report (`--last` after an ICE)
  lsp                   serve the language server protocol on stdio
  dap                   serve the time-travel debug adapter (DAP) on stdio
  replay <file> --trace <t.json> --step <n>
                        run one recorded call again with the file's edited fn
  self <command>        update, switch or roll back the installed toolchain
                        (`cpl self --help`); `cpl +<toolchain> <command>` runs
                        one command with another, e.g. `cpl +edge run a.cp`

`cpl <command> --help` describes a command's options.

options:
  -V, --version    print the version
  -h, --help       print this help
";

/// `cpl 0.2.2 (edge, 1a2b3c4d)`: what `cpl --version` prints.
pub fn version_line() -> String {
    format!(
        "cpl {VERSION} ({CHANNEL}, {})",
        &COMMIT[..COMMIT.len().min(8)]
    )
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

/// [`run`], with `input` as the program's standard input.
pub fn run_with_input<S: AsRef<str>>(args: &[S], input: &[u8]) -> Outcome {
    let mut stdout = Vec::new();
    let mut stderr = String::new();
    // The whole input arrives at once, and then it has ended.
    let mut rest = Some(input.to_vec());
    let code = exec(
        args,
        &mut || rest.take().unwrap_or_default(),
        &mut stdout,
        &mut stderr,
    );
    Outcome {
        stdout,
        stderr,
        code,
    }
}

/// Runs `cpl` with `args`, writing to `stdout` as it goes. `stdin` is
/// called only by a command that reads standard input: each call gives the
/// bytes that have arrived, and nothing once the input has ended.
pub fn exec<S: AsRef<str>>(
    args: &[S],
    stdin: &mut (dyn FnMut() -> Vec<u8> + Send),
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let mut usage_error = |message: String| {
        *stderr += &format!("cpl: {message}\n\n{USAGE}");
        EXIT_USAGE
    };
    let rest: Vec<&str> = args.iter().skip(1).map(AsRef::as_ref).collect();
    match args.first().map(AsRef::as_ref) {
        None | Some("-h" | "--help" | "help") => {
            let _ = stdout.write_all(USAGE.as_bytes());
            0
        }
        Some("-V" | "--version") => {
            let _ = writeln!(stdout, "{}", version_line());
            0
        }
        Some(
            command @ ("check" | "fix" | "build" | "judge" | "submit" | "judges" | "explain"
            | "lsp" | "dap"),
        ) if rest.iter().any(|arg| matches!(*arg, "-h" | "--help")) => {
            let help = match command {
                "check" => CHECK_USAGE,
                "fix" => FIX_USAGE,
                "build" => BUILD_USAGE,
                "judge" => JUDGE_USAGE,
                "submit" => SUBMIT_USAGE,
                "judges" => JUDGES_USAGE,
                "explain" => EXPLAIN_USAGE,
                "dap" => DAP_USAGE,
                _ => LSP_USAGE,
            };
            let _ = stdout.write_all(help.as_bytes());
            0
        }
        Some("replay") => replay::cli(&rest, stdout, stderr),
        Some("profile") => profile::profile(&rest, stdout, stderr),
        Some("cover") => profile::cover(&rest, stdout, stderr),
        Some("report") => report::report(&rest, stdin, stdout, stderr),
        Some("self") => toolchain::self_command(&rest, stdout, stderr),
        Some("explain") => explain_code(&rest, stdout, stderr),
        Some("check") if rest.is_empty() => {
            usage_error("`check` needs a file: `cpl check a.cp`".into())
        }
        Some("check") => check(&rest, stdout, stderr),
        Some("fix") if rest.is_empty() => usage_error("`fix` needs a file: `cpl fix a.cp`".into()),
        Some("fix") => fix(&rest, stdout, stderr),
        Some("run") if rest.iter().any(|arg| matches!(*arg, "-h" | "--help")) => {
            let _ = stdout.write_all(RUN_USAGE.as_bytes());
            0
        }
        Some("run") => match RunArgs::parse(&rest) {
            Ok(args) if args.live => run_live(&args, stdin),
            Ok(args) => {
                let typed = || std::io::IsTerminal::is_terminal(&std::io::stdin());
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
        Some("judge") => match rest.as_slice() {
            [file, input] => judge_file(file, input, stderr),
            [file, input, output] | [file, input, output, _] => {
                check_output(file, input, output, stderr)
            }
            _ => usage_error(
                "`judge` takes a file and the hidden test: `cpl judge a.cp in.txt`".into(),
            ),
        },
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
        Some("build") => {
            let checked = rest.contains(&"--checked");
            let size = rest.contains(&"--size");
            let copy = rest.contains(&"--copy");
            let mut rest: Vec<&str> = rest
                .iter()
                .copied()
                .filter(|arg| !matches!(*arg, "--checked" | "--size" | "--copy"))
                .collect();
            let judge = match take_judge(&mut rest, "build") {
                Ok(judge) => judge,
                Err(message) => return usage_error(message),
            };
            match rest.as_slice() {
                [file] => build_file(file, None, judge, checked, size, copy, stdout, stderr),
                [file, "-o", out] | ["-o", out, file] => {
                    build_file(file, Some(out), judge, checked, size, copy, stdout, stderr)
                }
                _ => usage_error(
                    "`build` takes one file: `cpl build a.cp`, `cpl build a.cp -o a.cpp` or \
                     `cpl build a.cp --checked`"
                        .into(),
                ),
            }
        }
        Some("submit") => {
            let copy = rest.contains(&"--copy");
            let rest: Vec<&str> = rest
                .iter()
                .copied()
                .filter(|arg| *arg != "--copy")
                .collect();
            match rest.as_slice() {
                [file] => submit(file, copy, stdout, stderr),
                _ => usage_error(
                    "`submit` takes one file, for the judge it names: `cpl submit a.cp` \
                     (a `// judge:` line or cp.toml says which)"
                        .into(),
                ),
            }
        }
        Some("judges") => match rest.as_slice() {
            [] => list_judges(None, stdout, stderr),
            [file] => list_judges(Some(file), stdout, stderr),
            _ => usage_error(
                "`judges` takes at most a file: `cpl judges` or `cpl judges a.cp`".into(),
            ),
        },
        Some("test") if should_run_unit_tests(&rest) => run_unit_tests(&rest, stdout, stderr),
        Some(command) if tool(command).is_some() => {
            let args: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            let mut errors = Vec::new();
            let code = tool(command).unwrap()(&args, stdout, &mut errors);
            *stderr += &String::from_utf8_lossy(&errors);
            code
        }
        Some("lsp") => usage_error("`lsp` talks to an editor over stdio and takes no files".into()),
        Some("dap") => usage_error("`dap` talks to an editor over stdio and takes no files".into()),
        Some(other) => usage_error(format!("unknown command `{other}`")),
    }
}

const CHECK_USAGE: &str = "\
usage: cpl check <file.cp>...

Reports every error in the files: names, types, borrows, input ranges.
Nothing is run and nothing is written. The exit status is 1 when a file
has an error.
";

const FIX_USAGE: &str = "\
usage: cpl fix <file.cp>...

Applies the fix each error carries (the quick fixes an editor offers),
then, once no error is left, the fixes `cpl lint --fix` makes. Rewrites the
file and reports the errors left. The exit status is 1 when an error is left.
";

const BUILD_USAGE: &str = "\
usage: cpl build <file.cp> [options]

Checks the file, then writes one C++17 file to submit, next to the source.

options:
  -o <out>          write to <out> instead; `-o -` writes to standard output
  --judge <name>    for that judge; else the file's `// judge:` line, else
                    `judge` under `[judge]` in the nearest cp.toml, else
                    `generic`. `cpl judges` lists them
  --checked         also compile it with sanitizers into a program to run here
  --size            say where the bytes go: header, runtime, library fns,
                    your code, by part and by piece (said anyway when the
                    file is over the judge's limit)
  --copy            also put the C++ on the clipboard (pbcopy, wl-copy, xclip,
                    xsel or clip, else the command in CPL_CLIPBOARD)

Over 32 KB it warns; over 64 KB, or the judge's own smaller limit, it fails
and says where the bytes go. The same file and judge give the same bytes.
";

const JUDGES_USAGE: &str = "\
usage: cpl judges [file.cp]

Lists the judge profiles: compiler, flags, stack and source limit. With a
file, marks the one its build uses (the `// judge:` line, else the nearest
cp.toml, else `generic`).
";

const EXPLAIN_USAGE: &str = "\
usage: cpl explain <CODE | file.cp | --json>

With a code such as E0302, explains that diagnostic or lint: what triggers
it and how to fix it. With a file, shows how each value is represented and
what its views cost. `--json` prints every explanation as JSON.
";

const SUBMIT_USAGE: &str = "\
usage: cpl submit <file.cp>

Gets the file ready to send to its judge (the `// judge:` line, else the
nearest cp.toml, else `generic`): writes the C++ as `cpl build` does, within
the judge's source limit, then runs every case in <file>.tests/ on that C++
built with the judge's flags on its stack, as `cpl test` does. Exits 0 when
the file is ready: it built and no case failed.

options:
  --copy            when ready, also put the C++ on the clipboard, as
                    `cpl build --copy` does

Mid-contest: put `// judge: codeforces` (or another of `cpl judges`) at the
top of the file once, save the samples as <file>.tests/1.in and 1.out, then
`cpl submit a.cp --copy` and paste, or upload a.cpp.
";

const JUDGE_USAGE: &str = "\
usage: cpl judge <file.cp> <input>
       cpl judge <file.cp> <input> <output> [answer]

With an input alone, runs the file's `@interactor` on it as a judge
program: it talks to a solution on standard input and output. With an
output, runs the file's `@checker` on that output. The exit status is 0
when the judge accepts.
";

const LSP_USAGE: &str = "\
usage: cpl lsp

Serves the language server protocol on standard input and output, for an
editor to start: errors as you type, hover, go to definition.
";

const DAP_USAGE: &str = "\
usage: cpl dap

Serves the Debug Adapter Protocol (DAP) on standard input and output,
supporting breakpoints, stepping, backward time-travel (`stepBack`,
`reverseContinue`), call stacks, domain-labeled variables, and traps.
";

const RUN_USAGE: &str = "\
usage: cpl run <file.cp> [input] [options]

Checks the file, then runs it in the debug interpreter on the input file
or on standard input. What `debug(...)` shows goes to standard error.

options:
  --trace [SPEC]    record the run: every change to a top-level variable,
                    every print and every debug(...); with SPEC, comma-
                    separated fn names or globs (`*`, `?`) with `!pat`
                    exclusions (e.g. `solve,!helper`). Writes
                    <file>.trace.html unless --format says where it goes.
                    With no input named and none piped in, it records the
                    case `cpl stress` saved last for the file
  --test SPEC       with --trace: record only these 1-based multitest cases
                    (e.g. `3`, `1,3`, `2..=4`)
  --watch VAR       with --trace: record steps only when VAR changes (repeatable
                    or comma-separated, e.g. `--watch dp,ans`)
  --last N          with --trace: keep only the last N steps (ring buffer)
  --depth N         with --trace: record only up to call depth N (0 = top level)
  --format json     without --trace: the value of each top-level variable,
                    the output and the time, for an editor.
                    with --trace: the trace as JSON on standard output
  --format html     with --trace: the trace viewer with the trace in it,
                    one self-contained page on standard output
  --live            with --trace: serve the viewer on localhost and record
                    again each time the file is saved, on the same input;
                    with no input named, the case `cpl stress` saved last.
                    The page keeps its place and shows the first change
  --brute           run the file's `@brute(main)` fn in place of the solution
  --release         run what a judge runs: the C++ of `cpl build`, built with
                    the judge's flags (those g++ here accepts) on the judge's
                    stack, and kept until the file changes.
                    Fast, and without the interpreter's checks
";

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
    /// `--depth N`: maximum call depth to record.
    max_depth: Option<usize>,
    format: Option<&'a str>,
    brute: bool,
    release: bool,
    live: bool,
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
            max_depth: None,
            format: None,
            brute: false,
            release: false,
            live: false,
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
                "--live" => args.live = true,
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
            || args.max_depth.is_some();
        if has_trace_filter && !args.trace {
            return Err(
                "`--test`, `--watch`, `--last`, and `--depth` filter a trace: add `--trace`".into(),
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
        "fmt" => cpl_fmt::cli::run,
        "lint" => cpl_lint::cli::run,
        "doc" => cpl_doc::cli::run,
        "stress" => cpl_stress::cli::run,
        "new" => cpl_contest::new::run,
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
fn judge_file(path: &str, input: &str, stderr: &mut String) -> u8 {
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
    match interact::judge_alone(judge, &input) {
        Ok(()) => 0,
        Err(trap) => {
            *stderr += &format!("{}\n", trap.message);
            *stderr += &render_trap(path, &compiled.text, &trap);
            EXIT_FAILED
        }
    }
}

/// `cpl judge <file> <input> <output>`: the file's `@checker` on what a
/// solution printed. The first line it says on failure is the reason.
fn check_output(path: &str, input: &str, output: &str, stderr: &mut String) -> u8 {
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
    match cpl_interp::run_with(&compiled.program, io, &mut sink) {
        Ok(()) => 0,
        Err(trap) => {
            *stderr += &format!("{}\n", trap.message);
            *stderr += &render_trap(path, &compiled.text, &trap);
            EXIT_FAILED
        }
    }
}

fn render_trap(path: &str, text: &str, trap: &cpl_interp::Trap) -> String {
    let lines = cpl_diag::LineIndex::new(text);
    let diagnostic = trap.to_diagnostic(path, &lines);
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

/// `cpl run`: the program's output goes straight to `stdout`.
/// `cpl run --release`: the file's C++, built as a judge builds it.
fn run_release(
    args: &RunArgs,
    input: &[u8],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let path = std::path::Path::new(args.file);
    let built = if args.brute {
        cpl_stress::Program::resolve_brute(path, "the brute")
    } else {
        cpl_stress::Program::resolve(path)
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
    // No time limit: a day is longer than anyone waits.
    let day = std::time::Duration::from_secs(86_400);
    let outcome = match program.run(&String::from_utf8_lossy(input), day) {
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
    let whole;
    let io = match input {
        Input::Whole(input) => {
            whole = input;
            cpl_interp::Io::fixed(&whole)
        }
        Input::Live(more) => cpl_interp::Io::live(Box::new(move || {
            let chunk = more();
            (!chunk.is_empty()).then_some(chunk)
        })),
    };
    let observed = cpl_interp::run_observed(&compiled.program, io, stdout, None);
    *stderr += &String::from_utf8_lossy(&observed.debug);
    match observed.trap {
        None => 0,
        Some(trap) => {
            *stderr += &render_trap(path, &compiled.text, &trap);
            EXIT_FAILED
        }
    }
}

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
    let record = || {
        let mut stderr = String::new();
        let compiled = compile_side(args.file, side, &mut stderr).map_err(|_| stderr.clone())?;
        let observed = cpl_interp::run_observed(
            &compiled.program,
            cpl_interp::Io::fixed(&input),
            &mut Vec::new(),
            Some(trace_config(args, &compiled.text)),
        );
        Ok(trace::single(
            args.file,
            &compiled.text,
            &observed.recording.unwrap_or_default(),
        ))
    };
    live::serve(args.file, &input, &note, &record)
}

fn trace_config(args: &RunArgs, text: &str) -> cpl_interp::trace::TraceConfig {
    let mut config = trace::config(text, args.only_fn);
    config.test_cases = args.test_cases.clone();
    config.watch = args.watch.clone();
    config.last_steps = args.last_steps;
    config.max_depth = args.max_depth;
    config
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
        None => cpl_interp::run_observed(
            &compiled.program,
            cpl_interp::Io::fixed(input),
            &mut printed,
            Some(config),
        ),
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

/// `cpl build`: one C++17 file, next to the source unless `out` says
/// where (`-` is standard output).
#[allow(clippy::too_many_arguments)]
fn build_file(
    path: &str,
    out: Option<&str>,
    judge: Option<&str>,
    checked: bool,
    size: bool,
    copy: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    if checked && out == Some("-") {
        *stderr += "cpl: `--checked` builds a program from the C++ file, so it can't go to standard output\n";
        return EXIT_USAGE;
    }
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let profile = match cpl_judge::resolve(std::path::Path::new(path), &compiled.text, judge) {
        Ok(profile) => profile,
        Err(error) => {
            *stderr += &format!("cpl: {error}\n");
            return EXIT_USAGE;
        }
    };
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let limit = profile.limit();
    let target = cpl_codegen::Target {
        stamp: Some(profile.stamp()),
        thread_stack: profile.thread_stack,
        windows: profile.os == cpl_judge::Os::Windows,
    };
    let cpp = match cpl_codegen::emit_within_for(
        &compiled.program,
        name,
        &compiled.text,
        limit,
        &target,
    ) {
        Ok(fitted) => {
            *stderr += &fitted.note(path, limit, &profile.name).unwrap_or_default();
            if size {
                let source = fitted.header.then_some(compiled.text.as_str());
                let sizes = cpl_codegen::sizes(
                    &compiled.program,
                    name,
                    source,
                    &target,
                    (fitted.comments, fitted.narrow),
                    fitted.cpp.len(),
                );
                *stderr += &size_report(&sizes, limit, &profile.name);
            }
            fitted.cpp
        }
        Err(bytes) => {
            *stderr += &format!(
                "cpl: the C++ for `{path}` is {bytes} bytes, over the {limit}-byte limit of judge `{}`\n",
                profile.name
            );
            let sizes =
                cpl_codegen::sizes(&compiled.program, name, None, &target, (false, true), bytes);
            *stderr += &size_report(&sizes, limit, &profile.name);
            return EXIT_ERRORS;
        }
    };
    if let Some(problem) = profile.unsupported(&cpp) {
        *stderr += &format!("cpl: `{path}` can't be submitted as it is: {problem}\n");
        return EXIT_ERRORS;
    }
    if let Some(warning) = profile.thread_warning() {
        *stderr += &format!("cpl: warning: {warning}\n");
    }
    if cpp.len() > profile.warn_at() {
        *stderr += &format!(
            "cpl: warning: the C++ for `{path}` is {} bytes, over the {}-byte target and {}% of the {limit}-byte \
             limit of judge `{}`; `--size` says where the bytes go\n",
            cpp.len(),
            profile.warn_at(),
            cpp.len() * 100 / limit,
            profile.name
        );
    }
    if copy {
        match copy_to_clipboard(&cpp) {
            Ok(tool) => {
                *stderr += &format!(
                    "cpl: copied the C++ ({} bytes) to the clipboard with `{tool}`\n",
                    cpp.len()
                )
            }
            Err(error) => {
                *stderr += &format!("cpl: warning: {error}; the C++ is only in the file\n")
            }
        }
    }
    if out == Some("-") {
        let _ = stdout.write_all(cpp.as_bytes());
        return 0;
    }
    let target = match out {
        Some(out) => std::path::PathBuf::from(out),
        None => std::path::Path::new(path).with_extension("cpp"),
    };
    match std::fs::write(&target, &cpp) {
        Ok(()) => {
            let _ = writeln!(
                stdout,
                "wrote {} ({} bytes, {}% of the limit of judge `{}`)",
                target.display(),
                cpp.len(),
                (cpp.len() * 100).div_ceil(limit),
                profile.name
            );
            if checked {
                return build_checked(&target, stdout, stderr);
            }
            0
        }
        Err(error) => {
            *stderr += &format!("cpl: can't write `{}`: {error}\n", target.display());
            EXIT_USAGE
        }
    }
}

/// Removes `--judge <name>` from `rest`, returning the name.
fn take_judge<'a>(rest: &mut Vec<&'a str>, command: &str) -> Result<Option<&'a str>, String> {
    match rest.iter().position(|arg| *arg == "--judge") {
        Some(at) if at + 1 < rest.len() => {
            let name = rest[at + 1];
            rest.drain(at..at + 2);
            Ok(Some(name))
        }
        Some(_) => Err(format!(
            "`--judge` takes a judge's name: `cpl {command} a.cp --judge codeforces`"
        )),
        None => Ok(None),
    }
}

/// `cpl submit`: `cpl build`, then `cpl test` on what it built.
fn submit(path: &str, copy: bool, stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let code = build_file(path, None, None, false, false, false, stdout, stderr);
    if code != 0 {
        *stderr += &format!("cpl: `{path}` is not ready to send: it does not build\n");
        return code;
    }
    let source = cpl_contest::problem::source_path(path);
    let cases = cpl_contest::problem::cases(&source);
    let tests = cpl_contest::problem::find_tests_dir(&source);
    if cases.is_empty() {
        *stderr += &format!(
            "cpl: warning: {} has no cases, so only the build was checked; put the samples there as `1.in` and \
             `1.out`\n",
            tests.display()
        );
        return ready_to_send(&source, copy, stdout, stderr);
    }
    let mut out = Vec::new();
    let mut errors = Vec::new();
    let code = cpl_contest::test::run(&[path.to_string()], &mut out, &mut errors);
    let _ = stdout.write_all(&out);
    *stderr += &String::from_utf8_lossy(&errors);
    if code != 0 {
        *stderr += &format!(
            "cpl: `{path}` is not ready to send: a case in {} fails\n",
            tests.display()
        );
        return EXIT_ERRORS;
    }
    ready_to_send(&source, copy, stdout, stderr)
}

fn ready_to_send(
    source: &std::path::Path,
    copy: bool,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let cpp = source.with_extension("cpp");
    if copy {
        match std::fs::read_to_string(&cpp)
            .map_err(|e| e.to_string())
            .and_then(|text| copy_to_clipboard(&text))
        {
            Ok(tool) => {
                *stderr += &format!(
                    "cpl: copied {} to the clipboard with `{tool}`\n",
                    cpp.display()
                )
            }
            Err(error) => *stderr += &format!("cpl: warning: {error}; send the file instead\n"),
        }
    }
    let _ = writeln!(stdout, "ready to send: {}", cpp.display());
    0
}

/// Puts `text` on the clipboard with the first tool this machine has, or
/// the command in `CPL_CLIPBOARD`. Returns the tool's name.
fn copy_to_clipboard(text: &str) -> Result<String, String> {
    use std::process::{Command, Stdio};
    let chosen = std::env::var("CPL_CLIPBOARD")
        .ok()
        .filter(|c| !c.trim().is_empty());
    let candidates: Vec<Vec<String>> = match &chosen {
        Some(command) => vec![vec!["sh".into(), "-c".into(), command.clone()]],
        None => [
            &["pbcopy"][..],
            &["wl-copy"],
            &["xclip", "-selection", "clipboard"],
            &["xsel", "--clipboard", "--input"],
            &["clip.exe"],
            &["clip"],
        ]
        .iter()
        .map(|words| words.iter().map(|w| w.to_string()).collect())
        .collect(),
    };
    for words in &candidates {
        let Ok(mut child) = Command::new(&words[0])
            .args(&words[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        if let Some(mut input) = child.stdin.take() {
            let _ = input.write_all(text.as_bytes());
        }
        if child.wait().is_ok_and(|status| status.success()) {
            return Ok(chosen.clone().unwrap_or_else(|| words.join(" ")));
        }
    }
    Err(match chosen {
        Some(command) => format!("`CPL_CLIPBOARD` (`{command}`) failed"),
        None => "no clipboard tool here (pbcopy, wl-copy, xclip, xsel or clip); set `CPL_CLIPBOARD` to one".into(),
    })
}

/// Where the bytes of a built file go, by part, with the largest pieces of
/// each part named.
fn size_report(sizes: &[cpl_codegen::Size], limit: usize, judge: &str) -> String {
    let total: usize = sizes.iter().map(|size| size.bytes).sum();
    let mut out =
        format!("size of the C++ by part: {total} bytes, of the {limit} of judge `{judge}`\n");
    let mut parts: Vec<cpl_codegen::Part> = sizes.iter().map(|size| size.part).collect();
    parts.sort();
    parts.dedup();
    for part in parts {
        let mut pieces: Vec<(&str, usize)> = Vec::new();
        for size in sizes.iter().filter(|size| size.part == part) {
            match pieces.iter_mut().find(|(name, _)| *name == size.name) {
                Some((_, bytes)) => *bytes += size.bytes,
                None => pieces.push((&size.name, size.bytes)),
            }
        }
        let bytes: usize = pieces.iter().map(|(_, bytes)| bytes).sum();
        if bytes == 0 {
            continue;
        }
        pieces.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let mut named: Vec<String> = pieces
            .iter()
            .take(3)
            .map(|(name, bytes)| format!("{name} {bytes}"))
            .collect();
        if pieces.len() > 3 {
            named.push(format!("{} more", pieces.len() - 3));
        }
        let named = if pieces.len() > 1 {
            named.join(", ")
        } else {
            String::new()
        };
        let share = (bytes * 100).div_ceil(total.max(1));
        out += format!("  {:<12}{bytes:>7} {share:>3}%  {named}", part.describe()).trim_end();
        out.push('\n');
    }
    out
}

/// `cpl judges`: every judge profile, and with a file, the one its build is for.
fn list_judges(file: Option<&str>, stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let path = std::path::Path::new(file.unwrap_or("./a.cp"));
    let source = match file {
        Some(file) => match std::fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{file}`: {error}\n");
                return EXIT_USAGE;
            }
        },
        None => String::new(),
    };
    let chosen = match cpl_judge::resolve(path, &source, None) {
        Ok(profile) => profile,
        Err(error) => {
            *stderr += &format!("cpl: {error}\n");
            return EXIT_USAGE;
        }
    };
    let config = cpl_judge::project_config(path).unwrap_or_default();
    for name in config.names() {
        let Some(profile) = config.profile(&name) else {
            continue;
        };
        let mark = if profile.name == chosen.name {
            "*"
        } else {
            " "
        };
        let _ = writeln!(
            stdout,
            "{mark} {:<11}{}\n    {}\n    {}, {}, {}, source up to {} KB\n    usual limits {} ms, {} MB\n    from: {}",
            profile.name,
            profile.title,
            profile.command(),
            if profile.os == cpl_judge::Os::Windows { "Windows" } else { "Linux" },
            profile.stack_text(),
            match profile.thread_stack {
                Some(bytes) => format!("the program on a thread with {} MB", bytes >> 20),
                None => "no threads".into(),
            },
            profile.limit() / 1024,
            profile.time_ms,
            profile.memory_mb,
            profile.evidence
        );
    }
    0
}

/// `cpl build --checked`: the C++ file built to run here, with the
/// sanitizers on, so overflow, a bad index or a bad shift stops the
/// program with a report instead of going on with a wrong value.
fn build_checked(
    cpp: &std::path::Path,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    // `LOCAL` turns on the checks the sanitizers have no word for: a
    // negative power, an `I128` shifted past its top, a `U64` divided by 0.
    const FLAGS: [&str; 7] = [
        "-std=c++17",
        "-O1",
        "-g",
        "-fsanitize=address,undefined",
        "-fno-sanitize-recover=all",
        "-D_GLIBCXX_ASSERTIONS",
        "-DLOCAL",
    ];
    let binary = cpp.with_extension("checked");
    let compiler = std::env::var("CXX").unwrap_or_else(|_| "g++".into());
    let built = std::process::Command::new(&compiler)
        .args(FLAGS)
        .arg("-o")
        .arg(&binary)
        .arg(cpp)
        .output();
    match built {
        Ok(output) if output.status.success() => {
            let _ = writeln!(
                stdout,
                "built {} ({compiler} {})",
                binary.display(),
                FLAGS.join(" ")
            );
            0
        }
        Ok(output) => {
            let errors = String::from_utf8_lossy(&output.stderr);
            let shown: Vec<&str> = errors.lines().take(20).collect();
            *stderr += &format!(
                "cpl: {compiler} could not build `{}`:\n{}\n",
                cpp.display(),
                shown.join("\n")
            );
            EXIT_ERRORS
        }
        Err(error) => {
            *stderr += &format!(
                "cpl: `--checked` needs a C++ compiler, and `{compiler}` did not start: {error}\n"
            );
            EXIT_USAGE
        }
    }
}

/// Decides whether `cpl test ...` targets inline `test "..."` blocks in `.cp` files
/// or directories, versus the contest `<problem>.tests/` runner.
fn should_run_unit_tests(args: &[&str]) -> bool {
    if args.is_empty() {
        return false;
    }
    if args.iter().any(|a| {
        matches!(
            *a,
            "-h" | "--help"
                | "--tests"
                | "--fast"
                | "--checker"
                | "--interactor"
                | "--time-limit"
                | "--memory-limit"
        )
    }) {
        return false;
    }
    let paths: Vec<&str> = args
        .iter()
        .copied()
        .filter(|a| !a.starts_with('-'))
        .collect();
    if paths.is_empty() {
        return false;
    }
    if paths.len() > 1 {
        return true;
    }
    let p = std::path::Path::new(paths[0]);
    if p.is_dir() {
        return true;
    }
    if p.extension().is_some_and(|e| e == "cp") && p.is_file() {
        if cpl_contest::problem::find_tests_dir(p).is_dir() {
            return false;
        }
        if let Ok(text) = std::fs::read_to_string(p) {
            let mut db = cpl_check::Database::new();
            db.set_file(paths[0], text.as_str());
            if laws::with_law_tests(&text).is_some() {
                return true;
            }
            if let Some(analysis) = db.analyze(paths[0]) {
                return !cpl_ir::list_tests(&analysis).is_empty();
            }
        }
    }
    false
}

fn collect_cp_files(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if path.is_file() {
        if path.extension().is_some_and(|e| e == "cp") {
            out.push(path.to_path_buf());
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    let mut children: Vec<std::path::PathBuf> =
        entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    children.sort();
    for child in children {
        collect_cp_files(&child, out);
    }
}

/// Runs inline `test "..."` blocks across the given `.cp` files and directories.
/// Pass `--cpp` to also emit C++17, compile with `g++ -std=c++17 -O2 -fsanitize=address,undefined`,
/// and run each test binary to verify parity and sanitizer cleanliness.
fn run_unit_tests(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let run_cpp = args.contains(&"--cpp");
    let mut files = Vec::new();
    for arg in args {
        if arg.starts_with('-') {
            continue;
        }
        let p = std::path::Path::new(arg);
        if !p.exists() {
            *stderr += &format!("cpl: `{arg}` does not exist\n");
            return EXIT_USAGE;
        }
        collect_cp_files(p, &mut files);
    }
    let mut db = cpl_check::Database::new();
    let mut passed = 0usize;
    let mut failed = 0usize;
    let tmp_dir = std::env::temp_dir().join(format!("cpl-test-{}", std::process::id()));
    if run_cpp {
        let _ = std::fs::create_dir_all(&tmp_dir);
    }
    for path in &files {
        let path_str = path.to_string_lossy().to_string();
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{path_str}`: {error}\n");
                failed += 1;
                continue;
            }
        };
        // Each top-level `@law` is checked as a test of its own.
        let text = match laws::with_law_tests(&text) {
            Some((with_laws, _)) => with_laws,
            None => text,
        };
        db.set_file(&path_str, text.as_str());
        let Some(analysis) = db.analyze(&path_str) else {
            continue;
        };
        let tests = cpl_ir::list_tests(&analysis);
        if tests.is_empty() {
            continue;
        }
        if analysis
            .diagnostics
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error)
        {
            *stderr += &cpl_diag::render_all(&analysis.diagnostics, &path_str, &text);
            failed += tests.len();
            continue;
        }
        for (idx, name) in tests.iter().enumerate() {
            let label = format!("{path_str}::{name}");
            match cpl_ir::lower_test(&analysis, name) {
                Ok(program) => {
                    let mut out = Vec::new();
                    if let Err(trap) = cpl_interp::run(&program, &[], &mut out) {
                        let _ = writeln!(stdout, "  FAIL {label}: {}", trap.message);
                        failed += 1;
                        continue;
                    }
                    if run_cpp {
                        let cpp = cpl_codegen::emit(&program, name, Some(&text));
                        let cpp_path = tmp_dir.join(format!("t_{passed}_{idx}.cpp"));
                        let bin_path = tmp_dir.join(format!("t_{passed}_{idx}"));
                        if let Err(e) = std::fs::write(&cpp_path, &cpp) {
                            let _ = writeln!(stdout, "  FAIL {label} (write cpp): {e}");
                            failed += 1;
                            continue;
                        }
                        let comp = std::process::Command::new("g++")
                            .args([
                                "-std=c++17",
                                "-O2",
                                "-fsanitize=address,undefined",
                                "-fno-sanitize-recover=all",
                                "-o",
                            ])
                            .arg(&bin_path)
                            .arg(&cpp_path)
                            .output();
                        match comp {
                            Ok(o) if o.status.success() => {
                                let run = std::process::Command::new(&bin_path).output();
                                match run {
                                    Ok(ro) if ro.status.success() => {
                                        if ro.stdout != out {
                                            let _ = writeln!(
                                                stdout,
                                                "  FAIL {label} (cpp output mismatch)"
                                            );
                                            failed += 1;
                                            continue;
                                        }
                                    }
                                    Ok(ro) => {
                                        let err = String::from_utf8_lossy(&ro.stderr);
                                        let first = err.lines().next().unwrap_or("non-zero exit");
                                        let _ =
                                            writeln!(stdout, "  FAIL {label} (cpp run): {first}");
                                        failed += 1;
                                        continue;
                                    }
                                    Err(e) => {
                                        let _ = writeln!(stdout, "  FAIL {label} (cpp exec): {e}");
                                        failed += 1;
                                        continue;
                                    }
                                }
                            }
                            Ok(o) => {
                                let err = String::from_utf8_lossy(&o.stderr);
                                let first = err
                                    .lines()
                                    .find(|l| l.contains("error:"))
                                    .or_else(|| err.lines().next())
                                    .unwrap_or("compile failed");
                                let _ = writeln!(stdout, "  FAIL {label} (g++): {first}");
                                failed += 1;
                                continue;
                            }
                            Err(e) => {
                                let _ = writeln!(stdout, "  FAIL {label} (g++): {e}");
                                failed += 1;
                                continue;
                            }
                        }
                    }
                    let _ = writeln!(stdout, "  ok   {label}");
                    passed += 1;
                }
                Err(diagnostics) => {
                    let msg = diagnostics
                        .first()
                        .map(|d| d.message.as_str())
                        .unwrap_or("lowering failed");
                    let _ = writeln!(stdout, "  FAIL {label}: {msg}");
                    failed += 1;
                }
            }
        }
    }
    if run_cpp {
        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
    let _ = writeln!(stdout, "{passed} passed, {failed} failed");
    if failed == 0 {
        0
    } else {
        EXIT_FAILED
    }
}

/// Checks each file through the shared front end and renders what it
/// finds.
fn check(files: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let mut db = cpl_check::Database::new();
    let mut code = 0;
    for path in files {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        };
        db.set_file(path, text.as_str());
        // What `cpl run` and `cpl build` would reject, `cpl check` rejects.
        let analysis = db.analyze(path).expect("the file was just set");
        let diagnostics = cpl_ir::reported(&analysis);
        let failed = diagnostics
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error);
        if !diagnostics.is_empty() {
            *stderr += &cpl_diag::render_all(&diagnostics, path, &text);
        }
        if !failed {
            let _ = writeln!(stdout, "{path}: no errors");
        } else if code == 0 {
            code = EXIT_ERRORS;
        }
    }
    code
}

/// The errors `cpl check` reports for `text`, read as the file at `path`.
fn errors_in(db: &mut cpl_check::Database, path: &str, text: &str) -> Vec<cpl_diag::Diagnostic> {
    db.set_file(path, text);
    let analysis = db.analyze(path).expect("the file was just set");
    let mut out = cpl_ir::check(&analysis);
    out.retain(|d| d.severity == cpl_diag::Severity::Error);
    out
}

/// `text` with the first fix of each error applied, round after round
/// until none applies, and how many were applied. A round can uncover
/// another error, so the rounds are capped.
pub fn apply_error_fixes(path: &str, text: &str) -> (String, usize) {
    let mut db = cpl_check::Database::new();
    let mut text = text.to_string();
    let mut applied = 0;
    for _ in 0..8 {
        let errors = errors_in(&mut db, path, &text);
        let (next, n) = cpl_lint::apply_fixes(&text, &errors);
        if n == 0 || next == text {
            break;
        }
        applied += n;
        text = next;
    }
    (text, applied)
}

/// `text` with the lint fixes `cpl lint --fix` would make, and how many.
fn apply_lint_fixes(path: &str, mut text: String) -> (String, usize) {
    let (config, _) = cpl_lint::Config::discover(std::path::Path::new(path));
    let mut applied = 0;
    for _ in 0..8 {
        let (next, n) = cpl_lint::apply_fixes(&text, &cpl_lint::lint(&text, &config));
        if n == 0 {
            break;
        }
        applied += n;
        text = next;
    }
    (text, applied)
}

fn fix(files: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let mut db = cpl_check::Database::new();
    let mut code = 0;
    for path in files {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        };
        let (fixed, mut applied) = apply_error_fixes(path, &text);
        let left = errors_in(&mut db, path, &fixed);
        let fixed = if left.is_empty() {
            let (linted, n) = apply_lint_fixes(path, fixed);
            applied += n;
            linted
        } else {
            fixed
        };
        if applied > 0 {
            if let Err(error) = std::fs::write(path, &fixed) {
                *stderr += &format!("cpl: can't write `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        }
        if !left.is_empty() {
            *stderr += &cpl_diag::render_all(&left, path, &fixed);
            if code == 0 {
                code = EXIT_ERRORS;
            }
        }
        let fixes = if applied == 1 { "fix" } else { "fixes" };
        let _ = match left.len() {
            0 => writeln!(stdout, "{path}: {applied} {fixes} applied, no errors"),
            n => writeln!(stdout, "{path}: {applied} {fixes} applied, {n} left"),
        };
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

fn explain_code(rest: &[&str], stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    match rest {
        ["--json"] => {
            let _ = writeln!(stdout, "[");
            let mut first = true;
            for info in cpl_diag::REGISTRY {
                if !first {
                    let _ = writeln!(stdout, ",");
                }
                first = false;
                let _ = write!(
                    stdout,
                    "  {{\"code\":{},\"title\":{},\"summary\":{},\"explanation\":{},\"trigger\":{},\"fix\":{},\"mode\":{},\"input\":{},\"lint_name\":null}}",
                    cpl_json::escape_string(info.code.0),
                    cpl_json::escape_string(info.title),
                    cpl_json::escape_string(info.summary),
                    cpl_json::escape_string(info.explanation),
                    cpl_json::escape_string(info.trigger),
                    cpl_json::escape_string(info.fix),
                    cpl_json::escape_string(info.mode),
                    cpl_json::escape_string(info.input),
                );
            }
            for lint in cpl_lint::LINTS {
                if !first {
                    let _ = writeln!(stdout, ",");
                }
                first = false;
                let _ = write!(
                    stdout,
                    "  {{\"code\":{},\"title\":{},\"summary\":{},\"explanation\":{},\"trigger\":{},\"fix\":{},\"mode\":\"lint\",\"lint_name\":{}}}",
                    cpl_json::escape_string(lint.code.0),
                    cpl_json::escape_string(lint.title),
                    cpl_json::escape_string(lint.summary),
                    cpl_json::escape_string(lint.explanation),
                    cpl_json::escape_string(lint.trigger),
                    cpl_json::escape_string(lint.fix),
                    cpl_json::escape_string(lint.name),
                );
            }
            let _ = writeln!(stdout, "\n]");
            0
        }
        [arg] if arg.ends_with(".cp") || std::path::Path::new(arg).is_file() => {
            explain_file(arg, false, stdout, stderr)
        }
        [arg, "--json"] | ["--json", arg]
            if arg.ends_with(".cp") || std::path::Path::new(arg).is_file() =>
        {
            explain_file(arg, true, stdout, stderr)
        }
        [code] => {
            if let Some(info) = cpl_diag::lookup(code) {
                let _ = writeln!(stdout, "{}: {}", info.code.0, info.title);
                let _ = writeln!(stdout, "\n{}", info.explanation);
                if !info.trigger.is_empty() {
                    let _ = writeln!(stdout, "\nTrigger:\n{}", info.trigger.trim_end());
                }
                if !info.fix.is_empty() {
                    let _ = writeln!(stdout, "\nFix:\n{}", info.fix.trim_end());
                }
                return 0;
            }
            if let Some(lint) = cpl_lint::LINTS
                .iter()
                .find(|l| l.code.0.eq_ignore_ascii_case(code) || l.name == *code)
            {
                let _ = writeln!(stdout, "{} ({}): {}", lint.code.0, lint.name, lint.title);
                let _ = writeln!(stdout, "\n{}", lint.explanation);
                let _ = writeln!(stdout, "\nTrigger:\n{}", lint.trigger.trim_end());
                let _ = writeln!(stdout, "\nFix:\n{}", lint.fix.trim_end());
                return 0;
            }
            *stderr += &format!("cpl: unknown diagnostic or lint code `{code}`\n");
            EXIT_USAGE
        }
        _ => {
            *stderr += "usage: cpl explain <CODE | FILE.cp> [--json]\n";
            EXIT_USAGE
        }
    }
}

fn explain_file(path: &str, json: bool, stdout: &mut dyn Write, stderr: &mut String) -> u8 {
    let compiled = match compile(path, stderr) {
        Ok(c) => c,
        Err(code) => return code,
    };
    let entries = cpl_codegen::explain_closures(&compiled.program);
    let dropped = cpl_codegen::explain_dropped_fields(&compiled.program);
    const DROPPED: &str = "never read; it and its stores are left out";
    if json {
        let _ = writeln!(stdout, "[");
        for (i, entry) in entries.iter().enumerate() {
            let comma = if i + 1 == entries.len() && dropped.is_empty() {
                ""
            } else {
                ","
            };
            let (kind, detail) = match &entry.mode {
                cpl_codegen::ClosureMode::StaticDirect { closure } => {
                    ("static_direct", format!("cpl_fn{closure}"))
                }
                cpl_codegen::ClosureMode::StaticTemplate => {
                    ("static_template", "template functor parameter".to_string())
                }
                cpl_codegen::ClosureMode::StaticLocalAuto => {
                    ("static_local_auto", "local auto lambda".to_string())
                }
                cpl_codegen::ClosureMode::Unread => ("unread", "never read, dropped".to_string()),
                cpl_codegen::ClosureMode::DynamicStdFunction { reason } => {
                    ("dynamic_std_function", reason.clone())
                }
            };
            let _ = writeln!(
                stdout,
                "  {{\"site\":{},\"lowering\":{},\"detail\":{}}}{comma}",
                cpl_json::escape_string(&entry.site),
                cpl_json::escape_string(kind),
                cpl_json::escape_string(&detail),
            );
        }
        for (i, site) in dropped.iter().enumerate() {
            let comma = if i + 1 == dropped.len() { "" } else { "," };
            let _ = writeln!(
                stdout,
                "  {{\"site\":{},\"lowering\":\"dropped_field\",\"detail\":{}}}{comma}",
                cpl_json::escape_string(site),
                cpl_json::escape_string(DROPPED),
            );
        }
        let _ = writeln!(stdout, "]");
        return 0;
    }
    let view_ops = explain_views(&compiled.text, &compiled.program);
    if !dropped.is_empty() {
        let _ = writeln!(stdout, "Dropped fields for {path}:");
        for site in &dropped {
            let _ = writeln!(stdout, "  [dropped] {site} -> {DROPPED}");
        }
    }
    if entries.is_empty() && view_ops.is_empty() {
        if !dropped.is_empty() {
            return 0;
        }
        let _ = writeln!(stdout, "{path}: no closure or fn-typed sites");
        return 0;
    }
    if !entries.is_empty() {
        let mut static_count = 0usize;
        let mut dynamic_count = 0usize;
        let _ = writeln!(stdout, "Closure & operation lowering for {path}:");
        for entry in &entries {
            match &entry.mode {
                cpl_codegen::ClosureMode::StaticDirect { closure } => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: direct]   {} -> cpl_fn{closure} (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::StaticTemplate => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: template] {} -> template functor param (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::StaticLocalAuto => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: auto]     {} -> local auto lambda (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::Unread => {
                    static_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [static: dropped]  {} -> never read, dropped (no std::function)",
                        entry.site
                    );
                }
                cpl_codegen::ClosureMode::DynamicStdFunction { reason } => {
                    dynamic_count += 1;
                    let _ = writeln!(
                        stdout,
                        "  [dynamic: std::function] {} ({reason})",
                        entry.site
                    );
                }
            }
        }
        let _ = writeln!(
            stdout,
            "Summary: {static_count} static (inlinable), {dynamic_count} dynamic (std::function)"
        );
    }
    if !view_ops.is_empty() {
        let _ = writeln!(stdout, "Views & copies:");
        for (line, col, desc) in view_ops {
            let _ = writeln!(stdout, "  {line}:{col}  {desc}");
        }
    }
    0
}

fn explain_views(text: &str, program: &cpl_ir::Program) -> Vec<(usize, usize, String)> {
    let line_col = |span: cpl_diag::Span| -> (usize, usize) {
        let off = (span.start as usize).min(text.len());
        let before = &text[..off];
        let line = before.bytes().filter(|&b| b == b'\n').count() + 1;
        let col = before.rfind('\n').map_or(off + 1, |i| off - i);
        (line, col)
    };
    let mut ops: Vec<(usize, usize, String)> = Vec::new();
    let mut visit_expr = |e: &cpl_ir::Expr| {
        if let cpl_ir::ExprKind::MutSlice(..) = &e.kind {
            let elem = match &e.ty {
                cpl_ir::Ty::MutView(t) => program.show_ty(t),
                _ => "T".to_string(),
            };
            let (line, col) = line_col(e.span);
            ops.push((
                line,
                col,
                format!(
                    "mutable slice view -> cpl::MutView<{elem}> (T* + len), O(1), fixed-length mut lend"
                ),
            ));
        }
        if let cpl_ir::ExprKind::Builtin(op, _) = &e.kind {
            let desc = match op {
                cpl_ir::Builtin::StrSlice => Some(
                    "slice view -> cpl::StrView (std::string_view + rev), O(1), read-only"
                        .to_string(),
                ),
                cpl_ir::Builtin::ListSlice => Some(format!(
                    "slice view -> cpl::ListView<{}> (const T* + len + rev), O(1), read-only",
                    match &e.ty {
                        cpl_ir::Ty::ListView(t, _) | cpl_ir::Ty::Keyed(t, _) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                cpl_ir::Builtin::Reversed if matches!(e.ty, cpl_ir::Ty::StrView) => Some(
                    "reverse view -> cpl::StrView (std::string_view + rev), O(1), read-only"
                        .to_string(),
                ),
                cpl_ir::Builtin::Reversed
                    if matches!(e.ty, cpl_ir::Ty::ListView(_, _) | cpl_ir::Ty::Keyed(_, _)) =>
                {
                    Some(format!(
                        "reverse view -> cpl::ListView<{}> (const T* + len + rev), O(1), read-only",
                        match &e.ty {
                            cpl_ir::Ty::ListView(t, _) | cpl_ir::Ty::Keyed(t, _) =>
                                program.show_ty(t),
                            _ => "T".to_string(),
                        }
                    ))
                }
                cpl_ir::Builtin::ToStr => {
                    Some("explicit copy -> std::string, O(n), owned".to_string())
                }
                cpl_ir::Builtin::ToList => Some(format!(
                    "explicit copy -> std::vector<{}>, O(n), owned",
                    match &e.ty {
                        cpl_ir::Ty::List(t) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                cpl_ir::Builtin::StrConcat => {
                    Some("concat -> std::string, O(n + m), owned".to_string())
                }
                cpl_ir::Builtin::ListConcat => Some(format!(
                    "concat -> std::vector<{}>, O(n + m), owned",
                    match &e.ty {
                        cpl_ir::Ty::List(t) => program.show_ty(t),
                        _ => "T".to_string(),
                    }
                )),
                _ => None,
            };
            if let Some(desc) = desc {
                let (line, col) = line_col(e.span);
                ops.push((line, col, desc));
            }
        }
    };
    cpl_ir::walk::deep_exprs(&program.setup, &mut visit_expr);
    cpl_ir::walk::deep_exprs(&program.main, &mut visit_expr);
    for f in &program.fns {
        cpl_ir::walk::deep_exprs(&f.body, &mut visit_expr);
    }
    ops.sort_by_key(|(l, c, _)| (*l, *c));
    ops.dedup();
    ops
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_prints_the_binary_name() {
        let out = run(&["--version"]);
        assert_eq!(out.code, 0);
        assert_eq!(out.stdout, format!("{}\n", version_line()).into_bytes());
        assert!(version_line().starts_with(&format!("cpl {VERSION} ({CHANNEL}, ")));
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
            "lint", "doc", "stress", "new", "test", "bench", "watch",
        ] {
            let out = run(&[command, "--help"]);
            assert_eq!(out.code, 0, "{command}");
            let help = String::from_utf8(out.stdout).unwrap();
            assert!(
                help.starts_with(&format!("usage: cpl {command}")),
                "{command}: {help}"
            );
            assert!(
                USAGE.contains(&format!("\n  {command} ")),
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
    fn run_and_build_go_through_the_same_check() {
        let bad = temp_file("bad_run.cp", "print(x)\n");
        for command in ["run", "build"] {
            let out = run(&[command, &bad]);
            assert_eq!(out.code, EXIT_ERRORS);
            assert_eq!(out.stderr, run(&["check", &bad]).stderr);
        }
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
            (out.code, out.stdout.as_slice()),
            (0, b"42\n".as_slice()),
            "{}",
            out.stderr
        );
        let out = run(&["run", &path, "--release", "--trace"]);
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
                (out.code, out.stdout.as_slice()),
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
