//! The `cpl` command-line tool. `cpl` is the working name of the binary
//! (`cp` clashes with Unix `cp`); source files end in `.cp`.

/// Version of the toolchain.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Crates linked into the binary, in pipeline order.
pub const COMPONENTS: &[&str] = &[
    cpl_diag::NAME,
    cpl_syntax::NAME,
    cpl_check::NAME,
    cpl_codegen::NAME,
    cpl_interp::NAME,
    cpl_fmt::NAME,
    cpl_lsp::NAME,
];

const USAGE: &str = "\
usage: cpl <command> [args]

commands:
  check <file>...  report every error in the files
  run <file>       check, then run in the debug interpreter (not built yet)
  build <file>     check, then emit C++17 (not built yet)
  lsp              serve the language server protocol on stdio

options:
  -V, --version    print the version
  -h, --help       print this help
";

/// What one invocation prints and the status it exits with.
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub code: u8,
}

/// Exit status when the source has errors.
pub const EXIT_ERRORS: u8 = 1;
/// Exit status when the command line is wrong or a file can't be read.
pub const EXIT_USAGE: u8 = 2;
/// Exit status when the source is fine but the command's back end is not
/// built yet.
pub const EXIT_NOT_BUILT: u8 = 3;

/// Runs `cpl` with `args` (without the program name). `cpl lsp` is served
/// by [`lsp`], because it streams instead of printing once.
pub fn run<S: AsRef<str>>(args: &[S]) -> Outcome {
    let ok = |stdout: String| Outcome {
        stdout,
        stderr: String::new(),
        code: 0,
    };
    let usage_error = |message: String| Outcome {
        stdout: String::new(),
        stderr: format!("cpl: {message}\n\n{USAGE}"),
        code: EXIT_USAGE,
    };
    let files: Vec<&str> = args.iter().skip(1).map(AsRef::as_ref).collect();
    match args.first().map(AsRef::as_ref) {
        None | Some("-h" | "--help" | "help") => ok(USAGE.to_string()),
        Some("-V" | "--version") => ok(format!("cpl {VERSION}\n")),
        Some("check") if files.is_empty() => {
            usage_error("`check` needs a file: `cpl check a.cp`".into())
        }
        Some("check") => check(&files),
        Some(command @ ("run" | "build")) => match files.as_slice() {
            [file] => {
                let mut out = check(&[file]);
                if out.code == 0 {
                    let stage = if command == "run" {
                        "the debug interpreter"
                    } else {
                        "the C++ back end"
                    };
                    out.stdout.clear();
                    out.stderr = format!("cpl: `{file}` has no errors, but `cpl {command}` needs {stage}, which is not built yet\n");
                    out.code = EXIT_NOT_BUILT;
                }
                out
            }
            _ => usage_error(format!("`{command}` takes one file: `cpl {command} a.cp`")),
        },
        Some("lsp") => usage_error("`lsp` talks to an editor over stdio and takes no files".into()),
        Some(other) => usage_error(format!("unknown command `{other}`")),
    }
}

/// Checks each file through the shared front end and renders what it
/// finds. This is the gate `run` and `build` go through too.
fn check(files: &[&str]) -> Outcome {
    let mut db = cpl_check::Database::new();
    let mut out = Outcome {
        stdout: String::new(),
        stderr: String::new(),
        code: 0,
    };
    for path in files {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                out.stderr += &format!("cpl: can't read `{path}`: {error}\n");
                out.code = EXIT_USAGE;
                continue;
            }
        };
        db.set_file(path, text.as_str());
        let diagnostics = db.diagnostics(path);
        if diagnostics.is_empty() {
            out.stdout += &format!("{path}: no errors\n");
        } else {
            out.stderr += &cpl_diag::render_all(&diagnostics, path, &text);
            if out.code == 0 {
                out.code = EXIT_ERRORS;
            }
        }
    }
    out
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
        assert_eq!(out.stdout, format!("cpl {VERSION}\n"));
    }

    #[test]
    fn no_arguments_prints_usage() {
        let out = run::<&str>(&[]);
        assert_eq!(out.code, 0);
        assert!(out.stdout.starts_with("usage: cpl "));
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
        assert_eq!(out.stdout, format!("{path}: no errors\n"));
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
        let good = temp_file("good_run.cp", "print(1)\n");
        let out = run(&["run", &good]);
        assert_eq!(out.code, EXIT_NOT_BUILT);
        assert!(out
            .stderr
            .contains("has no errors, but `cpl run` needs the debug interpreter"));
        assert!(run(&["build", &good])
            .stderr
            .contains("needs the C++ back end"));
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
        assert_eq!(COMPONENTS.len(), 7);
    }
}
