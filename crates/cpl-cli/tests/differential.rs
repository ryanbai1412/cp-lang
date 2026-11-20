//! The interpreter against the C++ back end, end to end.
//!
//! Every program under `tests/programs` is run by `cpl run`, and built by
//! `cpl build`, compiled with `g++ -std=c++17 -fsanitize=address,undefined`
//! and run. Both must print the bytes in the program's `.out` file and
//! exit with the same status, and the sanitizers must stay silent: C++
//! that only happens to work is a failure here.
//!
//! A program reads the `.in` file beside it, if there is one. To record
//! the `.out` of a new program, run the tests with `CPL_BLESS=1`; the
//! interpreter writes it, and the C++ is then held to it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn programs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("tests/programs exists")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    found.sort();
    assert!(
        found.len() >= 10,
        "only {} programs in {}",
        found.len(),
        dir.display()
    );
    found
}

fn input_of(program: &Path) -> Vec<u8> {
    std::fs::read(program.with_extension("in")).unwrap_or_default()
}

/// Compiles `cpp` with the sanitizers on and runs it on `input`.
/// Runs the built solution against `cpl judge`, the `@interactor` of
/// `program`: each reads what the other prints, and `input` is the
/// hidden test. The output is what the solution printed.
fn against_judge(
    binary: &Path,
    program: &Path,
    hidden: &Path,
    input: &[u8],
) -> Result<std::process::Output, String> {
    use std::io::{BufRead, BufReader};
    std::fs::write(hidden, input).map_err(|e| e.to_string())?;
    let mut judge = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("judge")
        .arg(program)
        .arg(hidden)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut solution = Command::new(binary)
        .stdin(judge.stdout.take().expect("piped"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    // Every line the solution prints is kept, and passed on to the judge.
    let mut to_judge = judge.stdin.take().expect("piped");
    let mut said = Vec::new();
    for line in BufReader::new(solution.stdout.take().expect("piped")).split(b'\n') {
        let mut line = line.map_err(|e| e.to_string())?;
        line.push(b'\n');
        said.extend_from_slice(&line);
        // A judge that has ruled no longer listens.
        let _ = to_judge.write_all(&line).and_then(|()| to_judge.flush());
    }
    drop(to_judge);
    let mut ran = solution.wait_with_output().map_err(|e| e.to_string())?;
    let judged = judge.wait_with_output().map_err(|e| e.to_string())?;
    ran.stdout = said;
    if ran.status.success() && !judged.status.success() {
        return Err(format!(
            "the interactor rejected the C++:\n{}",
            String::from_utf8_lossy(&judged.stderr)
        ));
    }
    Ok(ran)
}

fn compile_and_run(
    name: &str,
    cpp: &[u8],
    input: &[u8],
    interactor: Option<&Path>,
) -> Result<(Vec<u8>, Option<i32>), String> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("differential");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let source = dir.join(format!("{name}.cpp"));
    let binary = dir.join(name);
    std::fs::write(&source, cpp).map_err(|e| e.to_string())?;
    let compiled = Command::new("g++")
        .args([
            "-std=c++17",
            "-fsanitize=address,undefined",
            "-fno-sanitize-recover=all",
            "-Wall",
            "-Wextra",
            "-o",
        ])
        .arg(&binary)
        .arg(&source)
        .output()
        .map_err(|e| format!("g++ did not start ({e}); these tests need it"))?;
    let warnings = String::from_utf8_lossy(&compiled.stderr);
    if !compiled.status.success() || !warnings.is_empty() {
        return Err(format!(
            "g++ complained about {}:\n{warnings}",
            source.display()
        ));
    }
    let ran = match interactor {
        None => {
            let mut child = Command::new(&binary)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| e.to_string())?;
            let mut stdin = child.stdin.take().expect("piped");
            stdin.write_all(input).map_err(|e| e.to_string())?;
            drop(stdin);
            child.wait_with_output().map_err(|e| e.to_string())?
        }
        Some(program) => against_judge(&binary, program, &dir.join(format!("{name}.in")), input)?,
    };
    let stderr = String::from_utf8_lossy(&ran.stderr);
    if stderr.contains("Sanitizer") || stderr.contains("runtime error:") {
        return Err(format!(
            "the sanitizers stopped {}:\n{stderr}",
            source.display()
        ));
    }
    Ok((ran.stdout, ran.status.code()))
}

fn check(program: &Path) -> Result<(), String> {
    let name = program
        .file_stem()
        .expect("a file name")
        .to_string_lossy()
        .into_owned();
    let path = program.to_string_lossy().into_owned();
    let input = input_of(program);

    let interpreted = cpl_cli::run_with_input(&["run", &path], &input);
    if interpreted.code != 0 && interpreted.code != cpl_cli::EXIT_FAILED {
        return Err(format!(
            "`cpl run` exited with {}:\n{}",
            interpreted.code, interpreted.stderr
        ));
    }
    let golden = program.with_extension("out");
    if std::env::var_os("CPL_BLESS").is_some() {
        std::fs::write(&golden, &interpreted.stdout).map_err(|e| e.to_string())?;
    }
    let expected = std::fs::read(&golden)
        .map_err(|_| format!("no {}: run with CPL_BLESS=1 to record it", golden.display()))?;
    if interpreted.stdout != expected {
        return Err(format!(
            "`cpl run` printed\n{}\nbut {} holds\n{}",
            String::from_utf8_lossy(&interpreted.stdout),
            golden.display(),
            String::from_utf8_lossy(&expected)
        ));
    }

    let built = cpl_cli::run(&["build", &path, "-o", "-"]);
    if built.code != 0 {
        return Err(format!(
            "`cpl build` exited with {}:\n{}",
            built.code, built.stderr
        ));
    }
    if built.stdout.len() >= cpl_cli::MAX_CPP_BYTES {
        return Err(format!(
            "the C++ is {} bytes, over the 64 KB a judge takes",
            built.stdout.len()
        ));
    }
    if built.stdout != cpl_cli::run(&["build", &path, "-o", "-"]).stdout {
        return Err("two builds of one source differ".into());
    }
    let source = std::fs::read_to_string(program).map_err(|e| e.to_string())?;
    let cpp = String::from_utf8_lossy(&built.stdout);
    for line in source.lines().filter(|line| !line.trim().is_empty()) {
        if !cpp.contains(&format!("//     {line}")) {
            return Err(format!(
                "the C++ does not open with the source: `{line}` is missing"
            ));
        }
    }

    // An interactive problem is run against the judge in its file.
    let interactor = source.contains("\n@interactor").then_some(program);
    let (stdout, code) = compile_and_run(&name, &built.stdout, &input, interactor)?;
    if stdout != interpreted.stdout {
        return Err(format!(
            "the C++ printed\n{}\nbut the interpreter printed\n{}",
            String::from_utf8_lossy(&stdout),
            String::from_utf8_lossy(&interpreted.stdout)
        ));
    }
    if code != Some(i32::from(interpreted.code)) {
        return Err(format!(
            "the C++ exited with {code:?}, the interpreter with {}",
            interpreted.code
        ));
    }
    Ok(())
}

#[test]
fn interpreter_and_sanitized_cpp_agree_on_every_program() {
    let programs = programs();
    let workers = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(programs.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let failures = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(program) = programs.get(index) else {
                    break;
                };
                if let Err(why) = check(program) {
                    failures
                        .lock()
                        .expect("no panic while held")
                        .push(format!("{}: {why}", program.display()));
                }
            });
        }
    });
    let mut failures = failures.into_inner().expect("no panic while held");
    failures.sort();
    assert!(
        failures.is_empty(),
        "{} of {} programs failed:\n\n{}",
        failures.len(),
        programs.len(),
        failures.join("\n\n")
    );
}
