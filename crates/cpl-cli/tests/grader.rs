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

/// IOI 2025 souvenirs' signatures: the grader, against only its header,
/// takes and gives `long long` and `std::pair`, with values past `int`.
#[test]
fn a_grader_fn_links_in_its_declared_types() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_boundary");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader_boundary");
    std::fs::create_dir_all(&out).expect("a scratch directory");
    let (cpp, binary) = (out.join("souvenirs.cpp"), out.join("souvenirs"));
    let built = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("build")
        .arg(here.join("souvenirs.cp"))
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
    let expected =
        std::fs::read_to_string(here.join("souvenirs.out")).expect("the expected output");
    assert_eq!(String::from_utf8_lossy(&ran.stdout), expected);
}

/// Runs `grader_domain/<program>` against `<grader>` and gives its output.
fn run_in_grader_domain(program: &str, grader: &str) -> String {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_domain");
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(here.join(program))
        .arg("--grader")
        .arg(here.join(grader))
        .output()
        .expect("cpl runs");
    assert!(
        ran.status.success(),
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8_lossy(&ran.stdout).into_owned()
}

/// A `@grader fn` over its own domain (`p: [C]Int -> List<C>`) is built as
/// one C++ fn over vectors, `C` bound to the list's length.
#[test]
fn a_grader_fn_over_its_own_domain_links() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_domain");
    let expected = std::fs::read_to_string(here.join("festival.out")).expect("the expected output");
    assert_eq!(run_in_grader_domain("festival.cp", "grader.cpp"), expected);
}

/// Inside a `@grader fn` its own domain is still a domain: a `List<C>` it
/// builds goes through std's generic `reversed` as a `List<C>`.
#[test]
fn a_grader_fns_own_domain_stays_a_domain_inside_it() {
    assert_eq!(run_in_grader_domain("reversed.cp", "reversed.cpp"), "2 0\n");
}

/// A range with an open end has no C++ integer to cross in.
#[test]
fn cpl_check_names_an_open_range_at_the_grader() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader_boundary");
    std::fs::create_dir_all(&out).expect("a scratch directory");
    let program = out.join("open.cp");
    std::fs::write(&program, "@grader\nfn total(a: List<0..>) -> Int = a.len\n")
        .expect("the program is written");
    let checked = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(&program)
        .output()
        .expect("cpl runs");
    assert!(!checked.status.success());
    let stderr = String::from_utf8_lossy(&checked.stderr);
    assert!(
        stderr.contains("`0..` can't cross to the judge's grader"),
        "{stderr}"
    );
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

/// A stdin port beside the package's `grader.cpp` declares no `@grader`
/// fn, so `cpl test` and `cpl run` run it alone, a comment naming
/// `@grader` notwithstanding.
#[test]
fn a_stdin_port_beside_a_grader_runs_without_it() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_package");
    let port = here.join("port.cp");
    let tested = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("test")
        .arg(&port)
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
        .arg(&port)
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

/// A two-phase package's grader runs each phase in its own process, as the
/// judge does, so a dream whose second phase reads the first's globals fails
/// `cpl test` instead of passing locally and scoring 0 on the judge.
#[test]
fn a_two_phase_dream_cannot_read_its_first_phases_globals() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/two_phase_package");
    let test = |file: &str| {
        Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("test")
            .arg(here.join(file))
            .output()
            .expect("cpl runs")
    };
    let honest = test("honest.cp");
    assert!(
        honest.status.success(),
        "{}{}",
        String::from_utf8_lossy(&honest.stdout),
        String::from_utf8_lossy(&honest.stderr)
    );
    let leaky = test("leaky.cp");
    let stdout = String::from_utf8_lossy(&leaky.stdout);
    assert!(!leaky.status.success(), "{stdout}");
    assert!(stdout.contains("0/1 passed"), "{stdout}");
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

/// A grader that declares a boundary fn in another type than the program's
/// (here `std::tuple` where a 2-tuple crosses as `std::pair`) does not
/// compile, and the error names the fn; it never links and crashes.
#[test]
fn a_grader_declaring_a_fn_otherwise_does_not_build() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_mismatch");
    let ran = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(here.join("guess.cp"))
        .arg("--grader")
        .arg(here.join("grader.cpp"))
        .output()
        .expect("cpl runs");
    let stderr = String::from_utf8_lossy(&ran.stderr);
    assert!(!ran.status.success(), "{stderr}");
    assert!(stderr.contains("ambiguating new declaration"), "{stderr}");
    assert!(stderr.contains("stats"), "{stderr}");
}

/// `cpl test` of a grader-style program with cases and no grader is an
/// error that names `--grader`, not the linker's undefined references.
#[test]
fn cpl_test_without_a_grader_names_the_flag() {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader_test_without");
    std::fs::create_dir_all(out.join("tests")).expect("a scratch dir");
    let program = out.join("solve.cp");
    std::fs::write(
        &program,
        "extern fn compare(a: Int, b: Int) -> Int\n@grader\nfn solve(n: Int) -> Int = compare(1, n)\n",
    )
    .expect("the program is written");
    std::fs::write(out.join("tests/1.in"), "3\n").expect("a case");
    std::fs::write(out.join("tests/1.out"), "1\n").expect("an answer");
    let tested = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("test")
        .arg(&program)
        .output()
        .expect("cpl runs");
    let stderr = String::from_utf8_lossy(&tested.stderr);
    assert!(!tested.status.success(), "{stderr}");
    assert!(stderr.contains("--grader"), "{stderr}");
    assert!(stderr.contains("compare"), "{stderr}");
    assert!(!stderr.contains("undefined reference"), "{stderr}");
}

/// `cpl stress` on a file of `@grader` fns links the grader.cpp beside its
/// tests, as `cpl test` does, so the solution it stresses has a `main`.
#[test]
fn cpl_stress_links_the_grader_beside_the_tests() {
    let package = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/grader_package");
    let here = Path::new(env!("CARGO_TARGET_TMPDIR")).join("grader-stress");
    let _ = std::fs::remove_dir_all(&here);
    std::fs::create_dir_all(here.join("tests")).expect("a scratch directory");
    for file in ["sum.cp", "sum.h", "grader.cpp", "tests/1.in", "tests/1.out"] {
        std::fs::copy(package.join(file), here.join(file)).expect("the package copies");
    }
    std::fs::write(
        here.join("gen.py"),
        "import random, sys\nrandom.seed(int(sys.argv[1]))\nn = random.randint(1, 5)\n\
         print(n)\nprint(*[random.randint(-9, 9) for _ in range(n)])\n",
    )
    .unwrap();
    std::fs::write(
        here.join("brute.py"),
        "input()\nprint(sum(map(int, input().split())))\n",
    )
    .unwrap();
    let stress = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cpl"))
            .current_dir(&here)
            .args([
                "stress", "sum.cp", "--gen", "gen.py", "--brute", "brute.py", "--cases", "20",
            ])
            .args(extra)
            .output()
            .expect("cpl runs")
    };
    for out in [stress(&[]), stress(&["--grader", "grader.cpp"])] {
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.status.success(), "{said}");
        assert!(said.contains("20 cases"), "{said}");
    }
    let out = stress(&["--fast", "brute.py", "--grader", "grader.cpp"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
