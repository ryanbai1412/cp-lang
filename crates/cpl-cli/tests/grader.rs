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

/// `cpl test` and `cpl run` on a package whose `grader.cpp` sits beside
/// its `tests/`: the grader is linked in, and every case is judged.
#[test]
fn a_grader_package_is_tested_and_run_through_its_grader() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_package");
    let sum = here.join("sum.cp");
    let tested = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("test")
        .arg(&sum)
        .output()
        .expect("cpl runs");
    let stdout = String::from_utf8_lossy(&tested.stdout);
    assert!(
        tested.status.success(),
        "{stdout}{}",
        String::from_utf8_lossy(&tested.stderr)
    );
    assert!(stdout.contains("2/2 passed"), "{stdout}");

    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&sum)
        .arg(here.join("tests/2.in"))
        .output()
        .expect("cpl runs");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "35\n");
}

/// A file of `@grader` fns with no grader to call them is an error that
/// names `--grader`, not a run that prints nothing and passes.
#[test]
fn cpl_run_without_a_grader_names_the_flag() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader_alone");
    std::fs::create_dir_all(&out).expect("a scratch directory");
    let program = out.join("alone.cp");
    std::fs::write(&program, "@grader\nfn twice(x: Int) -> Int = 2 * x\n")
        .expect("the program is written");
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&program)
        .arg(&program)
        .output()
        .expect("cpl runs");
    assert!(!ran.status.success());
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(stderr.contains("--grader grader.cpp"), "{stderr}");

    let grader = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_package/grader.cpp");
    std::fs::write(
        &program,
        "@grader\nfn total(a: List<Int>) -> Int = a.sum() * 2\n",
    )
    .expect("the program is written");
    let input = out.join("1.in");
    std::fs::write(&input, "2\n1 2\n").expect("the input is written");
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&program)
        .arg(&input)
        .arg("--grader")
        .arg(&grader)
        .output()
        .expect("cpl runs");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&ran.stdout), "6\n");
}
