//! The Trap Parity Gate: every way a program stops with a runtime error
//! (`R0001`, status 3) under `cpl run` stops the C++ of `cpl build` too,
//! at `--checks traps`, compiled the way a judge compiles it: `-O2`, no
//! `-DLOCAL`, no sanitizer.
//!
//! `trap_parity.cp` holds one site per `which`. Site 0 is the control and
//! must print. A site added to the interpreter gets a line there, and
//! [`SITES`] goes up by one.
//!
//! Rule: `errors/R11`, handbook/guidelines/errors-checklist.md#r11-a-trap-names-the-values-and-the-way-out.

use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The highest `which` in `trap_parity.cp`.
const SITES: u32 = 36;

fn run(mut command: Command, input: &str) -> std::process::Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("starts");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
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
        .args(["--checks", "traps", "-o"])
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
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
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
    }
    bin
}

#[test]
fn every_runtime_error_of_cpl_run_stops_the_judge_build_too() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trap_parity.cp");
    let text = std::fs::read_to_string(&source).unwrap();
    assert!(
        text.contains(&format!("if which == {SITES}:"))
            && !text.contains(&format!("if which == {}:", SITES + 1)),
        "`SITES` must be the last `which` in trap_parity.cp"
    );
    let bin = judge_build(&source);
    let mut wrong = Vec::new();
    for which in 0..=SITES {
        // The sites that read more get a 7, which is outside `D`, and then
        // nothing; the last one gets `true` where a tuple wants a number.
        let more = if which == SITES { "7 true" } else { "7" };
        let input = format!("{which} 0 3 {more}\n");
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
    let sites = text
        .lines()
        .filter(|line| line.starts_with("if which == "))
        .count();
    let bin = judge_build(&source);
    let mut closed = Vec::new();
    for which in 1..=sites {
        let input = format!("{which} 0 3\n");
        let mut interp = Command::new(env!("CARGO_BIN_EXE_cpl"));
        interp.arg("run").arg(&source);
        let ran = run(interp, &input).status.code();
        let built = run(Command::new(&bin), &input).status.code();
        if ran == Some(3) && built == Some(3) {
            closed.push(format!(
                "site {which} is stopped by both now: move it to trap_parity.cp"
            ));
        }
    }
    assert!(closed.is_empty(), "{}", closed.join("\n"));
}

/// The judge build takes its input in pieces: a value that lies across
/// two of them reads whole.
#[test]
fn input_longer_than_one_piece_reads_the_same_in_the_judge_build() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/input_pieces.cp");
    let n = 40_000u64;
    let mut input = format!("{n}\n");
    for i in 0..n {
        // Lengths vary, so the cuts fall inside numbers and inside words.
        let x = (i * 2_654_435_761) % 1_000_000_007;
        let word = "ab".repeat((i % 7 + 1) as usize);
        input.push_str(&format!(
            "{} {word} {}.5\n",
            x as i64 - 500_000_000,
            i % 100
        ));
    }
    assert!(input.len() > 4 * (1 << 16), "several pieces");
    let mut interp = Command::new(env!("CARGO_BIN_EXE_cpl"));
    interp.arg("run").arg(&source);
    let interp = run(interp, &input);
    let judge = run(Command::new(judge_build(&source)), &input);
    assert!(interp.status.success() && judge.status.success());
    assert_eq!(
        String::from_utf8_lossy(&interp.stdout),
        String::from_utf8_lossy(&judge.stdout)
    );
}

/// A program that asks and then reads has its question out before it
/// waits, with no `@interactor` in the file: the judge answers only what
/// it has seen.
#[test]
fn the_judge_build_asks_before_it_waits_for_the_answer() {
    use std::io::{BufRead, BufReader};
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/live_guess.cp");
    let mut child = Command::new(judge_build(&source))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("starts");
    let mut answers = child.stdin.take().unwrap();
    let mut asked = BufReader::new(child.stdout.take().unwrap());
    let secret = 37;
    let mut queries = 0;
    let found = loop {
        let mut line = String::new();
        asked.read_line(&mut line).unwrap();
        let (mark, number) = line.trim().split_once(' ').expect("`? x` or `! x`");
        let number: i64 = number.parse().unwrap();
        if mark == "!" {
            break number;
        }
        queries += 1;
        let answer = if secret < number { "<" } else { ">=" };
        writeln!(answers, "{answer}").unwrap();
    };
    assert_eq!((found, queries), (secret, 7));
    assert!(child.wait().unwrap().success());
}

/// `cpl build --checks none` keeps no trap: a zero divisor gives 0, and
/// the program goes on as hand-written C++ would.
#[test]
fn a_judge_build_without_traps_divides_by_zero_into_zero() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("checks-none");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("div.cp");
    std::fs::write(&source, "let n: Int = read()\nprint(10 / n, 10 % n)\n").unwrap();
    let cpp = dir.join("div.cpp");
    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("build")
        .arg(&source)
        .args(["--checks", "none", "-o"])
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let bin = dir.join("div");
    let compiled = Command::new("g++")
        .args(["-std=c++17", "-O2", "-w", "-o"])
        .arg(&bin)
        .arg(&cpp)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let out = run(Command::new(&bin), "0\n");
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0 0\n");
    assert_eq!(run(Command::new(&bin), "3\n").stdout, b"3 1\n");
}
