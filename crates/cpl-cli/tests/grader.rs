//! A grader-style (IOI) program: `cpl build` gives C++ with no `main`,
//! which links against the judge's grader and runs clean under
//! `g++ -std=c++17 -fsanitize=address,undefined`.

use std::path::Path;
use std::process::Command;

#[test]
fn a_grader_program_links_against_the_judges_grader() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader");
    std::fs::create_dir_all(&out).expect("a scratch directory");
    let (cpp, binary) = (out.join("guess.cpp"), out.join("guess"));

    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("build")
        .arg(here.join("guess.cp"))
        .arg("-o")
        .arg(&cpp)
        .output()
        .expect("cpl runs");
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );

    let compiled = Command::new("g++")
        .args([
            "-std=c++17",
            "-O1",
            "-fsanitize=address,undefined",
            "-Wall",
            "-Wextra",
            "-Werror",
        ])
        .arg(&cpp)
        .arg(here.join("grader.cpp"))
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("g++ starts; this test needs it");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );

    let ran = Command::new(&binary).output().expect("the program runs");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    let expected = std::fs::read_to_string(here.join("guess.out")).expect("the expected output");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), expected);
}

#[test]
fn cpl_run_stops_at_a_call_of_the_judges_fn() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader");
    std::fs::create_dir_all(&out).expect("a scratch directory");
    let program = out.join("calls.cp");
    std::fs::write(&program, "@grader fn f(x: Int) -> Int\nprint(f(3))\n")
        .expect("the program is written");
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&program)
        .output()
        .expect("cpl runs");
    assert!(!ran.status.success());
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(
        stderr.contains("the judge's grader supplies it"),
        "{stderr}"
    );
}
