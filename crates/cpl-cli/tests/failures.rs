//! A command that can't do its job says why and what to do instead.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn cpl(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("cpl runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("failures")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// What `cpl` printed, with CRLF line ends as LF.
fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).replace("\r\n", "\n")
}

/// With no C++ compiler, a release run and `submit` still answer, through
/// the interpreter, and say that the judge's C++ was not checked.
#[test]
fn a_missing_compiler_is_named_where_the_cpp_would_run() {
    let dir = scratch("no-compiler");
    std::fs::write(dir.join("a.cp"), "let n: Int = read()\nprint(n * 2)\n").unwrap();
    std::fs::create_dir_all(dir.join("a.tests")).unwrap();
    std::fs::write(dir.join("a.tests/1.in"), "3\n").unwrap();
    std::fs::write(dir.join("a.tests/1.out"), "6\n").unwrap();
    let without = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .current_dir(&dir)
            .env("PATH", dir.join("none"))
            .output()
            .expect("cpl runs")
    };
    let run = without(&["run", "--release", "a.cp", "a.tests/1.in"]);
    assert_eq!(
        (run.status.code(), text(&run.stdout).as_str()),
        (Some(0), "6\n")
    );
    let submit = without(&["submit", "a.cp"]);
    assert_eq!(submit.status.code(), Some(0), "{}", text(&submit.stderr));
    for (out, what) in [(&run, "`--release`"), (&submit, "the cases")] {
        let err = text(&out.stderr);
        assert!(
            err.contains(&format!("so {what} ran in the interpreter"))
                && err.contains("install a C++ compiler"),
            "{err}"
        );
    }
    if Command::new("g++").arg("--version").output().is_ok() {
        let err = text(&cpl(&dir, &["run", "--release", "a.cp", "a.tests/1.in"]).stderr);
        assert!(!err.contains("interpreter"), "{err}");
    }
}

/// A setup that fails says why and what to do instead.
#[cfg(unix)]
#[test]
fn setup_failures_say_why_and_what_to_do() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("setup-fails");
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    for (name, body) in [("curl", "exit 7"), ("code", "exit 0")] {
        let path = bin.join(name);
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let setup = |args: &[&str], path: &Path| {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .current_dir(&dir)
            .env("PATH", path)
            .env("HOME", &dir)
            .env("CPL_CODE", bin.join("code"))
            .output()
            .expect("cpl runs");
        assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
        text(&out.stderr)
    };
    // No network: the download fails, and the release file can be passed in.
    let err = setup(&["setup", "vscode"], &bin);
    assert!(
        err.contains("could not download") && err.contains("curl exited with code 7"),
        "{err}"
    );
    assert!(
        err.contains("help: check the network") && err.contains("--vsix FILE"),
        "{err}"
    );
    // No curl at all.
    let err = setup(&["setup", "vscode"], &dir.join("none"));
    assert!(err.contains("needs curl on the PATH"), "{err}");
    // A folder that can't be written.
    std::fs::write(dir.join("taken"), "").unwrap();
    let err = setup(&["setup", "neovim", "--dir", "taken"], &bin);
    assert!(
        err.contains("can't write taken") && err.contains("help: pass `--dir DIR`"),
        "{err}"
    );
}
