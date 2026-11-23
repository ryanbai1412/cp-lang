//! `cpl test` on `.cp` files and directories: the inline `test "..."`
//! blocks, in the interpreter and, with `--cpp`, as sanitized C++.

use std::io::Write;

use crate::{laws, EXIT_FAILED, EXIT_USAGE};

/// Decides whether `cpl test ...` targets inline `test "..."` blocks in `.cp` files
/// or directories, versus the contest `<problem>.tests/` runner.
pub(crate) fn should_run_unit_tests(args: &[&str]) -> bool {
    if args.is_empty() {
        return false;
    }
    // An option of `cpl test` names a problem's cases, not unit tests.
    let case_option = |a: &str| {
        a.starts_with("--")
            && cpl_contest::test::USAGE
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("{a} ")))
    };
    if args
        .iter()
        .any(|a| matches!(*a, "-h" | "--help") || case_option(a))
    {
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
pub(crate) fn run_unit_tests(
    args: &[&str],
    stdout: &mut (dyn Write + Send),
    stderr: &mut String,
) -> u8 {
    let run_cpp = args.contains(&"--cpp");
    let mut files = Vec::new();
    for arg in args {
        if matches!(*arg, "--cpp" | "--json") {
            continue;
        }
        // An unknown option's value would be read as a file (`--judge generic`).
        if arg.starts_with('-') {
            let hint = match *arg {
                "--judge" => "; a judge profile is for `cpl build a.cp --judge generic`",
                _ => "",
            };
            *stderr +=
                &format!("cpl: `cpl test` has no `{arg}` option (see `cpl test --help`){hint}\n");
            return EXIT_USAGE;
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
        // A test that never ends fails at the judge's time limit, as a run would.
        let limit = cpl_judge::resolve(path, &text, None).map_or(2000, |judge| judge.time_ms);
        let limit = std::time::Duration::from_millis(limit);
        for (idx, name) in tests.iter().enumerate() {
            let label = format!("{path_str}::{name}");
            match cpl_ir::lower_test(&analysis, name) {
                Ok(program) => {
                    let Some((out, ran)) = cpl_interp::run_for(&program, &[], limit) else {
                        let _ = writeln!(
                            stdout,
                            "  FAIL {label}: still running after the {} ms time limit",
                            limit.as_millis()
                        );
                        failed += 1;
                        continue;
                    };
                    if let Err(trap) = ran {
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
