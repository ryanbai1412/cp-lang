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
  lsp                   serve the language server protocol on stdio

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
        Some("lsp") => usage_error("`lsp` talks to an editor over stdio and takes no files".into()),
        Some(other) => usage_error(format!("unknown command `{other}`")),
    }
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
        assert_eq!(COMPONENTS.len(), 8);
    }
}
