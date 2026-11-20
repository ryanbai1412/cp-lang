//! The `cpl` command-line tool. `cpl` is the working name of the binary
//! (`cp` clashes with Unix `cp`); source files end in `.cp`.

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
                        the input file or standard input
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
        Some("run") => match rest.as_slice() {
            [file] => run_file(file, &stdin(), stdout, stderr),
            [file, input] => match std::fs::read(input) {
                Ok(input) => run_file(file, &input, stdout, stderr),
                Err(error) => {
                    *stderr += &format!("cpl: can't read `{input}`: {error}\n");
                    EXIT_USAGE
                }
            },
            _ => usage_error(
                "`run` takes a file and at most one input: `cpl run a.cp in.txt`".into(),
            ),
        },
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
}

/// Reads and checks `path` through the shared front end, then lowers it.
/// This is the gate `run` and `build` both go through: the same
/// [`cpl_check::Database`] the language server holds, so a file means the
/// same thing to every tool.
fn compile(path: &str, stderr: &mut String) -> Result<Compiled, u8> {
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
    match cpl_ir::lower(&analysis) {
        Ok(program) => Ok(Compiled { text, program }),
        Err(diagnostics) => {
            *stderr += &cpl_diag::render_all(&diagnostics, path, &text);
            Err(EXIT_ERRORS)
        }
    }
}

/// `cpl run`: the program's output goes straight to `stdout`.
fn run_file(path: &str, input: &[u8], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let compiled = match compile(path, stderr) {
        Ok(compiled) => compiled,
        Err(code) => return code,
    };
    match cpl_interp::run(&compiled.program, input, stdout) {
        Ok(()) => 0,
        Err(trap) => {
            let lines = cpl_diag::LineIndex::new(&compiled.text);
            let diagnostic =
                cpl_diag::Diagnostic::error(cpl_diag::codes::RUNTIME, trap.span, trap.message);
            *stderr += &cpl_diag::render(&diagnostic, path, &compiled.text, &lines);
            EXIT_FAILED
        }
    }
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
        let diagnostics = db.diagnostics(path);
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
