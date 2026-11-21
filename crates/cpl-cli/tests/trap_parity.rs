//! The Trap Parity Gate: every way a program stops with a runtime error
//! (`R0001`, status 3) under `cpl run` stops the C++ of `cpl build` too,
//! compiled the way a judge compiles it: `-O2`, no `-DLOCAL`, no sanitizer.
//!
//! `trap_parity.cp` holds one site per `which`. Site 0 is the control and
//! must print. A site added to the interpreter gets a line there, and
//! [`SITES`] goes up by one.

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The highest `which` in `trap_parity.cp`.
const SITES: u32 = 33;

fn run(mut command: Command, input: &str) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("starts");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

/// The judge build of `source`, kept under a hash of its C++.
fn judge_build(source: &Path) -> PathBuf {
    let name = source.file_stem().unwrap().to_str().unwrap();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let cpp = dir.join(format!("{name}.cpp"));
    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("build")
        .arg(source)
        .arg("-o")
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(built.status.success(), "{}", String::from_utf8_lossy(&built.stderr));
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::fs::read(&cpp).unwrap().hash(&mut hasher);
    let bin = dir.join(format!("judge-{:016x}", hasher.finish()));
    if !bin.exists() {
        let compiled = Command::new("g++")
            .args(["-std=c++17", "-O2", "-w", "-o"])
            .arg(&bin)
            .arg(&cpp)
            .output()
            .unwrap();
        assert!(compiled.status.success(), "{}", String::from_utf8_lossy(&compiled.stderr));
    }
    bin
}

#[test]
fn every_runtime_error_of_cpl_run_stops_the_judge_build_too() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trap_parity.cp");
    let text = std::fs::read_to_string(&source).unwrap();
    assert!(
        text.contains(&format!("if which == {SITES}:")) && !text.contains(&format!("if which == {}:", SITES + 1)),
        "`SITES` must be the last `which` in trap_parity.cp"
    );
    let bin = judge_build(&source);
    let mut wrong = Vec::new();
    for which in 0..=SITES {
        let input = format!("{which} 0 3\n");
        let mut interp = Command::new(env!("CARGO_BIN_EXE_cpl"));
        interp.arg("run").arg(&source);
        let interp = run(interp, &input);
        let judge = run(Command::new(&bin), &input);
        let want = if which == 0 { 0 } else { 3 };
        let (ran, built) = (interp.status.code(), judge.status.code());
        if ran != Some(want) || built != Some(want) {
            wrong.push(format!(
                "site {which}: want status {want}; `cpl run` gave {ran:?}, the judge build gave {built:?} and printed {:?}",
                String::from_utf8_lossy(&judge.stdout)
            ));
        }
        if which == 0 {
            assert_eq!(interp.stdout, judge.stdout, "the control prints the same");
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// `known_traps.cp` is the register of what the gate does not hold yet:
/// each site is a wrong program that `cpl run`, the judge build, or both
/// let through. `scripts/check.sh` prints how many there are. A site that
/// both now stop fails here, and moves to `trap_parity.cp`.
#[test]
fn every_known_trap_is_still_open() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/known_traps.cp");
    let text = std::fs::read_to_string(&source).unwrap();
    let sites = text.lines().filter(|line| line.starts_with("if which == ")).count();
    let bin = judge_build(&source);
    let mut closed = Vec::new();
    for which in 1..=sites {
        let input = format!("{which} 0 3\n");
        let mut interp = Command::new(env!("CARGO_BIN_EXE_cpl"));
        interp.arg("run").arg(&source);
        let ran = run(interp, &input).status.code();
        let built = run(Command::new(&bin), &input).status.code();
        if ran == Some(3) && built == Some(3) {
            closed.push(format!("site {which} is stopped by both now: move it to trap_parity.cp"));
        }
    }
    assert!(closed.is_empty(), "{}", closed.join("\n"));
}
