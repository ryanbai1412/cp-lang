//! Every judge profile against real programs: each port is written for
//! each judge within its source limit, and the C++ compiles with the
//! judge's own flags (as far as the compiler here accepts them) and runs
//! on the stack the judge gives.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn ports() -> Vec<PathBuf> {
    let mut ports: Vec<PathBuf> = std::fs::read_dir(root().join("proto/ports/ecnerwala"))
        .expect("the ports")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    ports.sort();
    ports
}

/// `f` over `items` on every core, the results in order.
fn parallel<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = items.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let handles: Vec<_> = items.chunks(chunk).map(|part| scope.spawn(|| part.iter().map(&f).collect::<Vec<R>>())).collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    })
}

fn build(file: &Path, judge: &str) -> Result<String, String> {
    let out = cpl_cli::run(&["build", file.to_str().unwrap(), "--judge", judge, "-o", "-"]);
    match out.code {
        0 => Ok(String::from_utf8(out.stdout).unwrap()),
        _ => Err(out.stderr),
    }
}

#[test]
fn every_port_is_written_for_every_judge_within_its_limit() {
    let ports = ports();
    assert!(ports.len() > 100, "{} ports", ports.len());
    for profile in cpl_judge::builtins() {
        let sizes = parallel(&ports, |port| {
            let cpp = build(port, &profile.name).unwrap_or_else(|e| panic!("{}: {e}", port.display()));
            let stamp = format!("// For {}.\n", profile.stamp());
            assert!(cpp.lines().take(3).any(|line| format!("{line}\n") == stamp), "{}", port.display());
            assert!(cpp.len() <= profile.source_limit, "{}: {} bytes", port.display(), cpp.len());
            if profile.deep_stack() {
                assert!(!cpp.contains("cpl_switch_stack"), "{}", port.display());
            }
            cpp.len()
        });
        let mut sorted = sizes.clone();
        sorted.sort();
        eprintln!(
            "{:<11} p50 {:>6}  max {:>6}  total {:>8} bytes",
            profile.name,
            sorted[sorted.len() / 2],
            sorted[sorted.len() - 1],
            sizes.iter().sum::<usize>()
        );
    }
}

fn compile(cpp: &Path, binary: &Path, local: &cpl_judge::Local, only_syntax: bool) -> Result<(), String> {
    let mut command = Command::new(&local.compiler);
    command.args(&local.flags).arg(cpp);
    match only_syntax {
        true => command.arg("-fsyntax-only"),
        false => command.arg("-o").arg(binary),
    };
    let out = command.output().map_err(|e| format!("{}: {e}", local.compiler))?;
    match out.status.success() {
        true => Ok(()),
        false => Err(String::from_utf8_lossy(&out.stderr).into_owned()),
    }
}

/// Runs `binary` on `input` with the stack the judge gives.
fn run_on_stack(binary: &Path, input: &str, profile: &cpl_judge::Profile) -> std::process::Output {
    let stack = match profile.stack.assured() {
        Some(bytes) => (bytes >> 10).to_string(),
        None => "unlimited".into(),
    };
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(format!("ulimit -s {stack} && exec \"$0\""))
        .arg(binary)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("sh runs");
    use std::io::Write;
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn has(compiler: &str) -> bool {
    Command::new(compiler).arg("--version").output().is_ok_and(|out| out.status.success())
}

/// A judge with a deep stack gets C++ that uses it; any other gets C++
/// that brings its own. Either way a million nested calls, which crash an
/// 8 MB stack, run as the judge runs them, and a depth the interpreter
/// also takes gives its answer.
#[test]
fn deep_recursion_runs_on_the_stack_each_judge_gives() {
    if !has("g++") {
        eprintln!("skipped: no g++");
        return;
    }
    let deep = root().join("crates/cpl-cli/tests/judges/deep.cp");
    let interpreted = cpl_cli::run_with_input(&["run", deep.to_str().unwrap()], b"1000000\n");
    assert_eq!(interpreted.code, 0, "{}", interpreted.stderr);
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("judges-deep");
    std::fs::create_dir_all(&scratch).unwrap();
    for profile in cpl_judge::builtins() {
        let cpp = scratch.join(format!("{}.cpp", profile.name));
        let binary = scratch.join(&profile.name);
        std::fs::write(&cpp, build(&deep, &profile.name).unwrap()).unwrap();
        let local = profile.local();
        let windows = profile.os == cpl_judge::Os::Windows;
        if windows && local.compiler == profile.compiler {
            eprintln!("{}: no cross compiler for Windows here; checked the syntax only", profile.name);
            compile(&cpp, &binary, &local, true).unwrap_or_else(|e| panic!("{}: {e}", profile.name));
            continue;
        }
        compile(&cpp, &binary, &local, false).unwrap_or_else(|e| panic!("{}: {e}", profile.name));
        eprintln!("{}: {} {}; dropped {:?}", profile.name, local.compiler, local.flags.join(" "), local.dropped);
        if windows {
            continue;
        }
        let ran = run_on_stack(&binary, "1000000\n", &profile);
        assert!(ran.status.success(), "{}: {:?}", profile.name, ran.status);
        assert_eq!(ran.stdout, interpreted.stdout, "{}", profile.name);
    }
}

/// Every port's C++ for every judge, compiled with that judge's flags.
/// About a minute on eight cores: `scripts/check.sh --full` runs it.
#[test]
#[ignore = "slow: scripts/check.sh --full runs it"]
fn every_port_compiles_with_every_judges_flags() {
    let ports = ports();
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("judges-matrix");
    std::fs::create_dir_all(&scratch).unwrap();
    let mut failures = Vec::new();
    for profile in cpl_judge::builtins() {
        let local = profile.local();
        let failed: Vec<String> = parallel(&ports, |port| {
            let stem = port.file_stem().unwrap().to_str().unwrap();
            let cpp = scratch.join(format!("{}-{stem}.cpp", profile.name));
            std::fs::write(&cpp, build(port, &profile.name).unwrap()).unwrap();
            compile(&cpp, &cpp, &local, true).err().map(|e| format!("{} for {}:\n{e}", port.display(), profile.name))
        })
        .into_iter()
        .flatten()
        .collect();
        eprintln!(
            "{:<11} {} ports, {} fail: {} {}; dropped {:?}",
            profile.name,
            ports.len(),
            failed.len(),
            local.compiler,
            local.flags.join(" "),
            local.dropped
        );
        failures.extend(failed);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_release_run_builds_with_the_judges_flags_on_the_judges_stack() {
    if !has("g++") {
        eprintln!("skipped: no g++");
        return;
    }
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("judges-release");
    std::fs::create_dir_all(&scratch).unwrap();
    let source = root().join("crates/cpl-cli/tests/judges/deep.cp");
    let deep = std::fs::read_to_string(&source).unwrap();
    let interpreted = cpl_cli::run_with_input(&["run", source.to_str().unwrap()], b"1000000\n");
    // DOMjudge's stack is unlimited, so the C++ keeps its recursion on the
    // judge's stack, which only the judge's stack holds.
    let file = scratch.join("deep.cp");
    std::fs::write(&file, format!("// judge: icpc\n{deep}")).unwrap();
    let cpp = build(&file, "icpc").unwrap();
    assert!(!cpp.contains("cpl_switch_stack"));
    let ran = cpl_cli::run_with_input(&["run", file.to_str().unwrap(), "--release"], b"1000000\n");
    assert_eq!(ran.code, 0, "{}", ran.stderr);
    assert_eq!(ran.stdout, interpreted.stdout);
}

/// The parts of `cpl build --size`, and the total it gives.
fn size_parts(report: &str) -> (usize, Vec<(String, usize)>) {
    let mut lines = report.lines().skip_while(|line| !line.starts_with("size of the C++ by part"));
    let head = lines.next().expect("a size report");
    let total = head.split(": ").nth(1).and_then(|rest| rest.split(' ').next()).unwrap().parse().unwrap();
    let parts = lines
        .take_while(|line| line.starts_with("  "))
        .map(|line| {
            let line = line.trim_start();
            let (name, rest) = line.split_at(12);
            (name.trim().to_string(), rest.split_whitespace().next().unwrap().parse().unwrap())
        })
        .collect();
    (total, parts)
}

#[test]
fn the_size_report_of_every_port_adds_up_to_its_file() {
    let ports = ports();
    let glue = parallel(&ports, |port| {
        let out = cpl_cli::run(&["build", port.to_str().unwrap(), "--size", "-o", "-"]);
        assert_eq!(out.code, 0, "{}: {}", port.display(), out.stderr);
        let (total, parts) = size_parts(&out.stderr);
        assert_eq!(total, out.stdout.len(), "{}", port.display());
        assert_eq!(parts.iter().map(|(_, bytes)| bytes).sum::<usize>(), total, "{}", port.display());
        let glue = parts.iter().find(|(name, _)| name == "glue").map_or(0, |(_, bytes)| *bytes);
        assert!(glue * 10 <= total, "{}: {glue} of {total} bytes are not in any piece", port.display());
        glue * 1000 / total
    });
    let mut glue = glue;
    glue.sort();
    eprintln!("glue per mille: p50 {}, max {}", glue[glue.len() / 2], glue[glue.len() - 1]);
}

#[test]
fn a_file_over_its_judges_limit_says_where_its_bytes_go() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("judges-size");
    std::fs::create_dir_all(&scratch).unwrap();
    std::fs::write(scratch.join("cp.toml"), "[judge]\njudge = \"tiny\"\n\n[judge.tiny]\nextends = \"generic\"\nsource_kb = 1\n").unwrap();
    let file = scratch.join("a.cp");
    std::fs::write(&file, "let n: Int = read()\nvar xs: List<Int> = []\nrepeat n: xs.push(read())\nxs.sort()\nfor x in xs: print(x)\n").unwrap();
    let out = cpl_cli::run(&["build", file.to_str().unwrap(), "-o", "-"]);
    assert_eq!(out.code, 1, "{}", out.stderr);
    assert!(out.stderr.contains("over the 1024-byte limit of judge `tiny`"), "{}", out.stderr);
    let (total, parts) = size_parts(&out.stderr);
    assert_eq!(parts.iter().map(|(_, bytes)| bytes).sum::<usize>(), total);
    let names: Vec<&str> = parts.iter().map(|(name, _)| name.as_str()).collect();
    assert!(names.contains(&"runtime") && names.contains(&"your code"), "{}", out.stderr);
}
