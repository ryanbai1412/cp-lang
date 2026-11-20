//! The `cpl` command-line tool. `cpl` is the working name of the binary
//! (`cp` clashes with Unix `cp`); source files end in `.cp`.

mod interact;
mod laws;
mod trace;

use std::io::Write;

/// Version of the toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
  run <file> [input]    check, then run in the debug interpreter, reading
                        the input file or standard input; a file with an
                        `@interactor` runs against it, on the hidden test
                        (`cpl run --help` has the options for traces)
  judge <file> <input> [<output> <answer>]
                        run the file's `@interactor` as a judge program on
                        standard input and output, or with an output its
                        `@checker`; exit status 0 accepts
  build <file> [-o out] check, then write one C++17 file to submit: next to
                        the source by default, to standard output with `-o -`
  fmt [path]...         format files in the one canonical style
  lint [path]...        warn about code that is legal but probably wrong or slow
  doc [path]...         build HTML documentation from `///` comments
  new <name>            create a problem from its constraints or samples
  test <problem>        run every case in <problem>.tests/
  stress <file>         run against a brute on random inputs; shrink the failure
  bench <problem>       run on maximum-size worst-case inputs against the limits
  watch <problem>       check, lint and test on every save
  lsp                   serve the language server protocol on stdio

`cpl <command> --help` describes a command's options.

options:
  -V, --version    print the version
  -h, --help       print this help
";

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
    let code = exec(args, &mut || input.to_vec(), &mut stdout, &mut stderr);
    Outcome {
        stdout,
        stderr,
        code,
    }
}

/// Runs `cpl` with `args`, writing to `stdout` as it goes. `stdin` is
/// called only by a command that reads standard input.
pub fn exec<S: AsRef<str>>(
    args: &[S],
    stdin: &mut dyn FnMut() -> Vec<u8>,
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
            let _ = writeln!(stdout, "cpl {VERSION}");
            0
        }
        Some("check") if rest.is_empty() => {
            usage_error("`check` needs a file: `cpl check a.cp`".into())
        }
        Some("check") => check(&rest, stdout, stderr),
        Some("run") if rest.iter().any(|arg| matches!(*arg, "-h" | "--help")) => {
            let _ = stdout.write_all(RUN_USAGE.as_bytes());
            0
        }
        Some("run") => match RunArgs::parse(&rest) {
            Ok(args) => {
                let input = match args.input {
                    None => stdin(),
                    Some(input) => match std::fs::read(input) {
                        Ok(input) => input,
                        Err(error) => {
                            *stderr += &format!("cpl: can't read `{input}`: {error}\n");
                            return EXIT_USAGE;
                        }
                    },
                };
                run_file(&args, &input, stdout, stderr)
            }
            Err(message) => usage_error(message),
        },
        Some("judge") => match rest.as_slice() {
            [file, input] => judge_file(file, input, stderr),
            [file, input, output] | [file, input, output, _] => check_output(file, input, output, stderr),
            _ => usage_error(
                "`judge` takes a file and the hidden test: `cpl judge a.cp in.txt`".into(),
            ),
        },
        // `cpl stress` traces a failure in the debug interpreter, which
        // this crate links and `cpl-stress` does not.
        Some("stress") => {
            let args: Vec<String> = rest.iter().map(|arg| arg.to_string()).collect();
            let mut errors = Vec::new();
            let code = cpl_stress::cli::run_with(&args, stdout, &mut errors, Some(&trace::against_brute));
            *stderr += &String::from_utf8_lossy(&errors);
            code
        }
        Some("build") => match rest.as_slice() {
            [file] => build_file(file, None, stdout, stderr),
            [file, "-o", out] | ["-o", out, file] => build_file(file, Some(out), stdout, stderr),
            _ => usage_error(
                "`build` takes one file: `cpl build a.cp` or `cpl build a.cp -o a.cpp`".into(),
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
        Some(other) => usage_error(format!("unknown command `{other}`")),
    }
}

const RUN_USAGE: &str = "\
usage: cpl run <file.cp> [input] [options]

Checks the file, then runs it in the debug interpreter on the input file
or on standard input. What `debug(...)` shows goes to standard error.

options:
  --trace [FN]      record the run: every change to a top-level variable,
                    every print and every debug(...); with FN, only while
                    that fn runs. Writes <file>.trace.html unless --format
                    says where the trace goes
  --format json     without --trace: the value of each top-level variable,
                    the output and the time, for an editor.
                    with --trace: the trace as JSON on standard output
  --format html     with --trace: the trace viewer with the trace in it,
                    one self-contained page on standard output
  --brute           run the file's `@brute(main)` fn in place of the solution
";

/// What `cpl run` was asked for.
struct RunArgs<'a> {
    file: &'a str,
    input: Option<&'a str>,
    trace: bool,
    /// `--trace FN`: record only while this fn runs.
    only_fn: Option<&'a str>,
    format: Option<&'a str>,
    brute: bool,
}

impl<'a> RunArgs<'a> {
    fn parse(rest: &[&'a str]) -> Result<RunArgs<'a>, String> {
        let mut args = RunArgs {
            file: "",
            input: None,
            trace: false,
            only_fn: None,
            format: None,
            brute: false,
        };
        let mut paths = Vec::new();
        let mut iter = rest.iter().copied().peekable();
        while let Some(arg) = iter.next() {
            match arg {
                "--trace" => {
                    args.trace = true;
                    // A name after it that is no file is the fn to trace.
                    if let Some(next) = iter.peek().copied() {
                        let is_name = next.chars().all(|c| c.is_alphanumeric() || c == '_');
                        if is_name && !next.is_empty() && !std::path::Path::new(next).exists() {
                            args.only_fn = Some(next);
                            iter.next();
                        }
                    }
                }
                "--brute" => args.brute = true,
                "--format" => match iter.next() {
                    Some(format @ ("json" | "html" | "text")) => args.format = Some(format),
                    Some(other) => return Err(format!("`--format` is text, json or html, not `{other}`")),
                    None => return Err("`--format` needs a value: json or html".into()),
                },
                flag if flag.starts_with("--trace=") => {
                    args.trace = true;
                    args.only_fn = Some(&flag["--trace=".len()..]);
                }
                flag if flag.starts_with('-') && flag.len() > 1 => {
                    return Err(format!("`run` has no option `{flag}`; `cpl run --help` lists them"))
                }
                path => paths.push(path),
            }
        }
        match paths.as_slice() {
            [file] => args.file = file,
            [file, input] => (args.file, args.input) = (file, Some(*input)),
            _ => return Err("`run` takes a file and at most one input: `cpl run a.cp in.txt`".into()),
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
    let mut db = cpl_check::Database::new();
    db.set_file(path, text.as_str());
    let analysis = db.analyze(path).expect("the file was just set");
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
            Some((name, _)) => cpl_ir::lower_checker(&analysis, &name).map(|program| (program, None)),
            None => return Err(missing("@checker", stderr)),
        },
    };
    match lowered {
        Ok((program, judge)) => Ok(Compiled {
            text,
            program,
            judge,
        }),
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
    let diagnostic =
        cpl_diag::Diagnostic::error(cpl_diag::codes::RUNTIME, trap.span, trap.message.clone());
    cpl_diag::render(&diagnostic, path, text, &lines)
}

/// `cpl run`: the program's output goes straight to `stdout`.
fn run_file(args: &RunArgs, input: &[u8], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let path = args.file;
    let side = if args.brute { Side::Brute } else { Side::Solution };
    let compiled = match compile_side(path, side, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let json = args.format == Some("json");
    if args.trace || json {
        return observe_file(args, &compiled, input, stdout, stderr);
    }
    // An interactive problem with its judge in the file: the input is
    // the hidden test, and the two talk to each other.
    if let Some((name, judge)) = &compiled.judge {
        let ruling = interact::run(&compiled.program, judge, input, stdout);
        // A side that stops leaves the other with nothing to read: that
        // second failure only follows from the first.
        let starved = |trap: &cpl_interp::Trap| trap.message.starts_with("the input ended");
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
    let observed = cpl_interp::run_observed(&compiled.program, cpl_interp::Io::fixed(input), stdout, None);
    *stderr += &String::from_utf8_lossy(&observed.debug);
    match observed.trap {
        None => 0,
        Some(trap) => {
            *stderr += &render_trap(path, &compiled.text, &trap);
            EXIT_FAILED
        }
    }
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
    let config = trace::config(&compiled.text, args.only_fn);
    // The program's output is kept: with `--format` the report or the
    // trace is what goes to standard output.
    let mut printed = Vec::new();
    let quiet = args.format.is_some_and(|format| format != "text");
    let observed = match &compiled.judge {
        Some((_, judge)) => interact::run_traced(&compiled.program, judge, input, &mut printed, config),
        None => cpl_interp::run_observed(&compiled.program, cpl_interp::Io::fixed(input), &mut printed, Some(config)),
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
                        recording.steps.len()
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
fn build_file(
    path: &str,
    out: Option<&str>,
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    let mut cpp = cpl_codegen::emit(&compiled.program, name, Some(&compiled.text));
    if cpp.len() > MAX_CPP_BYTES {
        // The source in the header is for the reader; a file the judge
        // turns away helps nobody.
        let bytes = cpp.len();
        cpp = cpl_codegen::emit(&compiled.program, name, None);
        if cpp.len() > MAX_CPP_BYTES {
            *stderr += &format!(
                "cpl: the C++ for `{path}` is {} bytes, over the {MAX_CPP_BYTES}-byte limit judges accept\n",
                cpp.len()
            );
            return EXIT_ERRORS;
        }
        *stderr += &format!(
            "cpl: with its source in the header the C++ for `{path}` would be {bytes} bytes, over the \
             {MAX_CPP_BYTES}-byte limit judges accept; the header is left out\n"
        );
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
            let _ = writeln!(stdout, "wrote {} ({} bytes)", target.display(), cpp.len());
            0
        }
        Err(error) => {
            *stderr += &format!("cpl: can't write `{}`: {error}\n", target.display());
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
            "-h" | "--help" | "--fast" | "--checker" | "--interactor" | "--time-limit"
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
        let tests_dir = p.with_extension("tests");
        if tests_dir.is_dir() {
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
fn run_unit_tests(
    args: &[&str],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
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
                                        let _ = writeln!(stdout, "  FAIL {label} (cpp run): {first}");
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
    if failed == 0 { 0 } else { EXIT_FAILED }
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
        let diagnostics = cpl_ir::check(&analysis);
        if diagnostics.is_empty() {
            let _ = writeln!(stdout, "{path}: no errors");
        } else {
            *stderr += &cpl_diag::render_all(&diagnostics, path, &text);
            if code == 0 {
                code = EXIT_ERRORS;
            }
        }
    }
    code
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
    fn version_prints_the_binary_name() {
        let out = run(&["--version"]);
        assert_eq!(out.code, 0);
        assert_eq!(out.stdout, format!("cpl {VERSION}\n").into_bytes());
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
            "fmt", "lint", "doc", "stress", "new", "test", "bench", "watch",
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
                "error[E0301]: `a` is indexed by V, but `n` is a 1..=9\n  --> {path}:4:9\n  |\n4 | print(a[n])\n  |         ^\n  = help: convert it with `V(n)`, or write `a.raw[n]` to index by number\n\n{path}: 1 error\n"
            )
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
}
