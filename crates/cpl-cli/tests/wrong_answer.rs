//! The wrong-answer workflow end to end, through the `cpl` binary, as
//! `handbook/product/workflows/wrong-answer.md` walks it: stress with no
//! brute finds a crash, stress with a brute finds a wrong answer, each
//! report names the next commands, the trace records the saved case, and
//! after each fix `cpl test` and `cpl stress` are green. Use cases
//! `stress/U1`, `stress/U4`, `stress/U13` and `stress/U14` in
//! `handbook/product/cli/stress-usecases.md`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cpl(args: &[&OsStr]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .output()
        .expect("cpl runs");
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text)
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("c.cp")
}

const READS: &str = "let n: 1..=10 = read()\nlet a: [n]0..=9 = read()\n";

const CRASHES: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i + 1])\nprint(best)\n";
const SKIPS_LAST: &str =
    "var best = 0\nfor i in 0..n - 1:\n    best = max(best, a[i])\nprint(best)\n";
const RIGHT: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i])\nprint(best)\n";

const BRUTE: &str = "
@alt(program)
fn brute():
    let n: 1..=10 = read()
    let a: [n]0..=9 = read()
    var m = 0
    for x in a: m = max(m, x)
    print(m)
";

fn saved(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path.with_file_name("c.tests"))
        .map(|dir| {
            dir.map(|e| e.unwrap().file_name().into_string().unwrap())
                .filter(|n| n.ends_with(".in"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_wrong_answer_goes_from_a_crash_and_a_counterexample_to_a_clean_stress() {
    let path = scratch("wrong-answer");
    let file = path.as_os_str();
    let stress = ["stress".as_ref(), file, "--cases".as_ref(), "200".as_ref()];
    let test = ["test".as_ref(), file];

    // No brute yet: a case fails only on a crash or the time limit.
    std::fs::write(&path, format!("{READS}{CRASHES}")).unwrap();
    let (code, report) = cpl(&stress);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "only runtime errors and the time limit fail a case",
        ": RE",
        "outside an array",
        "locals: ",
        "next: `cpl run ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 1, "{report}");

    // The trace records the case stress saved.
    let (_, traced) = cpl(&["run".as_ref(), file, "--trace".as_ref()]);
    assert!(traced.contains("the case saved last"), "{traced}");
    assert!(path.with_file_name("c.trace.html").is_file(), "{traced}");

    // Fixed: the saved case passes and stress runs clean.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(clean.contains("the saved case passes; "), "{clean}");
    assert!(
        clean.contains("200 cases ran with no runtime error"),
        "{clean}"
    );

    // With a brute, the wrong answer that was left is found and explained.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}{BRUTE}")).unwrap();
    let mut explained = stress.to_vec();
    explained.push("--explain".as_ref());
    let (code, report) = cpl(&explained);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "that is: n = 1",
        "expected:",
        "look at line ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 2, "{report}");
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(1), "{rerun}");

    // Fixed again: every saved case passes and the brute agrees.
    std::fs::write(&path, format!("{READS}{RIGHT}{BRUTE}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    assert!(rerun.contains("2/2 passed"), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(clean.contains("200 cases agree"), "{clean}");
}

/// Off the happy path: the brute is the one that is wrong (`stress/U18`).
#[test]
fn a_brute_that_traps_is_blamed_with_its_line_and_locals_and_not_saved() {
    let path = scratch("brute-traps");
    let source = format!(
        "{READS}{RIGHT}
@alt(program)
fn brute():
    let n: 1..=10 = read()
    let a: [n]0..=9 = read()
    var m = 0
    for i in 0..n: m = max(m, a[i + 1])
    print(m)
"
    );
    std::fs::write(&path, source).unwrap();
    let (code, report) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "brute RE",
        "that is: n = 1",
        "failed, so there is nothing to compare with",
        "error[R0101]",
        "c.cp:13:",
        "locals: ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert!(!report.contains("Code("), "{report}");
    assert!(!report.contains("could not be saved"), "{report}");
    assert!(saved(&path).is_empty(), "{report}");
}

/// The editor's lens reads a runtime error as `cpl check --format json` gives it (`stress/U23`).
#[test]
fn a_runtime_error_in_the_json_report_is_a_diagnostic() {
    let path = scratch("json-trap");
    std::fs::write(&path, format!("{READS}print(a[n])\n")).unwrap();
    let (code, report) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        r#""verdict":"RE","error":{"code":"R0101""#,
        r#""start":{"col":7,"line":3"#,
        r#""locals: n = 1"#,
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
}

/// Small cases never reach the bounds, so stress runs the input a bound
/// lint names first, and saves it when it traps (`stress/U28`).
#[test]
fn stress_runs_the_input_a_bound_lint_names() {
    let file = scratch("bound_lint");
    std::fs::write(
        &file,
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\n\
         if a % 1000 == 37:\n    print(a * b)\n",
    )
    .unwrap();
    let _ = std::fs::remove_dir_all(file.with_extension("tests"));
    let (code, text) = cpl(&[OsStr::new("stress"), file.as_os_str()]);
    assert_ne!(code, Some(0), "{text}");
    let witness = file.with_extension("tests").join("lint-L0306-4.in");
    assert!(
        text.contains("FAIL on the input `cpl lint` names at the bounds, saved as")
            && text.contains("lint-L0306-4.in: RE")
            && text.contains("error[R0202]"),
        "{text}"
    );
    assert!(witness.exists(), "{text}");
    let (code, tested) = cpl(&[OsStr::new("test"), file.as_os_str()]);
    assert_ne!(code, Some(0), "{tested}");
    assert!(
        tested.contains("error[R0202]: integer overflow"),
        "{tested}"
    );
    assert!(tested.contains("0/1 passed"), "{tested}");
}

/// A swapped-in `@alt(f)` fn that traps is blamed as an `@alt(program)`
/// brute is: its error, line and locals, in text and in JSON (`stress/U18`).
#[test]
fn a_swapped_in_brute_fn_that_traps_is_blamed_with_its_line_and_locals() {
    let path = scratch("part-brute-traps");
    let source = format!(
        "{READS}fn best() -> Int:
    var m = 0
    for i in 0..n: m = max(m, a[i])
    m

@alt(best)
fn best_slow() -> Int:
    var m = 0
    for i in 0..n: m = max(m, a[i + 1])
    m

print(best())
"
    );
    std::fs::write(&path, source).unwrap();
    let (code, report) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_eq!(code, Some(1), "{report}");
    for said in ["brute RE", "error[R0101]", "c.cp:11:", "locals: "] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert!(!report.contains("exit code"), "{report}");
    let (_, json) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert!(json.contains("\"code\":\"R0101\""), "{json}");
}

/// Stress is a checked build: a case that overflows fails as RE with
/// the diagnostic `cpl run` gives, instead of wrapping (`stress/U34`).
#[test]
fn stress_traps_an_overflow_as_cpl_run_does() {
    let file = scratch("overflow");
    std::fs::write(
        &file,
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\nprint(a * b)\n\n\
         @gen\nfn at_bounds():\n    print(9223372037)\n    print(1000000000)\n",
    )
    .unwrap();
    let (code, text) = cpl(&[
        OsStr::new("stress"),
        file.as_os_str(),
        OsStr::new("--cases"),
        OsStr::new("1"),
    ]);
    assert_eq!(code, Some(1), "{text}");
    // The input L0306 names at the bounds runs first and traps.
    assert!(text.contains("lint-L0306-3.in: RE"), "{text}");
    assert!(text.contains("error[R0202]: integer overflow"), "{text}");
}

/// A shift past an `Int` traps under stress as under `cpl run`, though
/// C++ would shift it without a word (`stress/U34`).
#[test]
fn stress_traps_a_shift_overflow_as_cpl_run_does() {
    let file = scratch("shift-overflow");
    std::fs::write(
        &file,
        "let k: 0..=63 = read()\nprint(1 << k)\n\n@gen\nfn top():\n    print(63)\n",
    )
    .unwrap();
    let (code, text) = cpl(&[
        OsStr::new("stress"),
        file.as_os_str(),
        OsStr::new("--cases"),
        OsStr::new("1"),
    ]);
    assert_eq!(code, Some(1), "{text}");
    assert!(text.contains("lint-L0302-2.in: RE"), "{text}");
    assert!(
        text.contains("`1 << 63` does not fit in an `Int`"),
        "{text}"
    );
}

/// A g++ that miscompiles `n * 2` is the toolchain's bug, not a WA: the
/// debug interpreter prints the brute's answer (`stress/U29`).
#[cfg(unix)]
#[test]
fn a_build_the_interpreter_disagrees_with_is_a_toolchain_bug() {
    use std::os::unix::fs::PermissionsExt;
    let path = scratch("toolchain-bug");
    let dir = path.parent().unwrap();
    std::fs::write(
        &path,
        "let n: 1..=5 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=5 = read()\n    print(n + n)\n",
    )
    .unwrap();
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let gxx = bin.join("g++");
    let real = String::from_utf8(
        Command::new("sh")
            .args(["-c", "command -v g++"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    std::fs::write(
        &gxx,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do last=\"$a\"; done\ncase \"$last\" in *.cpp) sed -i 's/put_int(n \\* 2)/put_int(n * 2 + (n == 3))/' \"$last\" ;; esac\nexec {} \"$@\"\n",
            real.trim()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&gxx, std::fs::Permissions::from_mode(0o755)).unwrap();
    let run = |json: bool| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cpl"));
        command.args(["stress".as_ref(), path.as_os_str()]);
        if json {
            command.args(["--format", "json"]);
        }
        let paths = std::env::var_os("PATH").unwrap_or_default();
        let paths =
            std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(&paths)))
                .unwrap();
        let out = command
            .env("PATH", paths)
            .env("CPL_CACHE", dir.join("cache"))
            .env("CPL_NO_PCH", "1")
            .output()
            .expect("cpl runs");
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr)
    };
    let text = run(false);
    assert!(text.contains("FAIL on case"), "{text}");
    assert!(
        text.contains("toolchain bug: the debug interpreter prints the expected answer"),
        "{text}"
    );
    let json = run(true);
    assert!(json.contains("\"toolchain_bug\":true"), "{json}");
    assert!(json.contains("\"build\":\"judge `"), "{json}");
}

/// The cases stress saved run first, whatever the seed, until they pass
/// (`stress/U15`).
#[test]
fn a_saved_case_runs_first_and_fails_until_it_is_fixed() {
    let path = scratch("saved-first");
    std::fs::write(&path, "let n: 0..=20 = read()\nprint(10 / (n - 3))\n").unwrap();
    let (code, found) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_eq!(code, Some(1), "{found}");
    assert_eq!(saved(&path).len(), 1, "{found}");
    let once = |seed: &str| {
        cpl(&[
            "stress".as_ref(),
            path.as_os_str(),
            "--cases".as_ref(),
            "1".as_ref(),
            "--seed".as_ref(),
            seed.as_ref(),
        ])
    };
    let (code, again) = once("777");
    assert_eq!(code, Some(1), "{again}");
    assert!(again.contains("FAIL on the saved case "), "{again}");
    assert!(again.contains("error[R0"), "{again}");
    assert!(!again.contains("shrunk from"), "{again}");
    assert!(!again.contains("\n  saved "), "{again}");
    std::fs::write(&path, "let n: 0..=20 = read()\nprint(10 / (n - 30))\n").unwrap();
    let (code, clean) = once("777");
    assert_eq!(code, Some(0), "{clean}");
    assert!(
        clean.contains("the saved case passes; 1 cases ran"),
        "{clean}"
    );
}

/// When the line that should set a value never runs, `--explain` names the
/// loop that ran 0 times as well as the start value it left (`stress/U12`).
#[test]
fn explain_names_the_loop_that_never_ran() {
    let path = scratch("unrun-loop");
    std::fs::write(
        &path,
        concat!(
            "let n: 1..=8 = read()\n",
            "let a: [n]0..=9 = read()\n",
            "var best = 0\n",
            "for i in 1..n:\n",
            "    best max= a[i]\n",
            "print(best)\n",
            "\n",
            "@alt(program)\n",
            "fn brute():\n",
            "    let n: 1..=8 = read()\n",
            "    let a: [n]0..=9 = read()\n",
            "    print(a.max()!)\n",
        ),
    )
    .unwrap();
    let (code, out) = cpl(&["stress".as_ref(), path.as_os_str(), "--explain".as_ref()]);
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("that is: n = 1"), "{out}");
    assert!(
        out.contains(
            "look at line 4: `best` is set on line 5 only in this loop, which runs 0 times"
        ),
        "{out}"
    );
}

/// A checker, interactor or `@gen` fn that traps is blamed with its trap,
/// and the case is no test of the solution (`stress/U30`).
#[test]
fn a_broken_helper_is_blamed_with_its_trap_not_the_solution() {
    let helpers = [
        (
            "checker",
            "@check\nfn valid():\n    let n: 1..=5 = read()\n    let out: Int = read_output()\n    let a: [n]Int = 0\n    assert a[out] == 0\n",
        ),
        (
            "interactor",
            "@interactor\nfn judge():\n    let n: 1..=5 = read()\n    print(n)\n    let q: Int = read()\n    let a: [2]Int = 0\n    assert a[q + 5] == 0\n",
        ),
    ];
    for (kind, helper) in helpers {
        let path = scratch(&format!("broken-{kind}"));
        let solution = match kind {
            "checker" => "let n: 1..=5 = read()\nprint(n)\n",
            _ => "let n: 1..=5 = read()\nlet q: Int = read()\nprint(q)\n",
        };
        std::fs::write(&path, format!("{solution}\n{helper}")).unwrap();
        let (code, out) = cpl(&["stress".as_ref(), path.as_os_str()]);
        assert_eq!(code, Some(1), "{out}");
        assert!(out.contains(&format!(": {kind} RE\n")), "{out}");
        assert!(out.contains(&format!("the {kind} failed")), "{out}");
        assert!(out.contains("error[R0101]"), "{out}");
        assert!(out.contains("c.cp:"), "{out}");
        assert!(saved(&path).is_empty(), "{out}");
        {
            let (_, json) = cpl(&[
                "stress".as_ref(),
                path.as_os_str(),
                "--format".as_ref(),
                "json".as_ref(),
            ]);
            let error = json
                .split("\"error\":")
                .nth(1)
                .unwrap_or_else(|| panic!("{json}"));
            assert!(error.contains("\"code\":\"R0101\""), "{json}");
            let line = if kind == "checker" { 9 } else { 11 };
            assert!(error.contains(&format!("\"line\":{line}")), "{json}");
        }
    }
    let path = scratch("broken-gen");
    std::fs::write(
        &path,
        "let n: 1..=5 = read()\nprint(n)\n\n@gen\nfn small():\n    let a: [3]Int = 0\n    print(a[3 + rand(0, 1)] + 1)\n",
    )
    .unwrap();
    let (code, out) = cpl(&["stress".as_ref(), path.as_os_str()]);
    assert_ne!(code, Some(0), "{out}");
    assert!(
        out.contains("(`small`) failed on seed 1 at line 7:"),
        "{out}"
    );
    assert!(out.contains("is outside an array dimension"), "{out}");
    assert!(saved(&path).is_empty(), "{out}");
}

/// `--time` stops starting cases when the budget ends and reports as usual
/// (`stress/U32`).
#[test]
fn stress_runs_for_the_time_it_is_given() {
    let path = scratch("time-budget");
    std::fs::write(
        &path,
        "let n: 1..=5 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=5 = read()\n    print(n + n)\n",
    )
    .unwrap();
    let started = std::time::Instant::now();
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--time".as_ref(),
        "1s".as_ref(),
    ]);
    assert_eq!(code, Some(0), "{out}");
    assert!(out.contains(" cases agree in 1s"), "{out}");
    assert!(
        started.elapsed() < std::time::Duration::from_secs(120),
        "{out}"
    );
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--time".as_ref(),
        "1h".as_ref(),
    ]);
    assert_ne!(code, Some(0), "{out}");
    assert!(out.contains("`--time` is a duration like `30s`"), "{out}");
}

/// A clean report says how far its cases reached, so `1000 cases agree`
/// can be weighed (`stress/U31`).
#[test]
fn a_clean_stress_says_how_far_its_cases_reached() {
    let path = scratch("reach");
    std::fs::write(
        &path,
        "let n: 1..=100 = read()\nlet a: [n]Int = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=100 = read()\n    let a: [n]Int = read()\n    print(n + n)\n",
    )
    .unwrap();
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--cases".as_ref(),
        "200".as_ref(),
    ]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("  tried: n from 1 to 8, a up to 8 long from -1000000000 to 1000000000\n"),
        "{out}"
    );
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--cases".as_ref(),
        "200".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
    ]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("\"tried\":\"n from 1 to 8, a up to 8 long from -1000000000 to 1000000000\""),
        "{out}"
    );
    let generated = scratch("reach-gen");
    std::fs::write(
        &generated,
        "let n: 1..=100 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=100 = read()\n    print(n + n)\n\n@gen\nfn small():\n    print(rand(3, 40))\n",
    )
    .unwrap();
    let (code, out) = cpl(&["stress".as_ref(), generated.as_os_str()]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("  tried: inputs of up to 1 number from 3 to "),
        "{out}"
    );
}

/// A lint warning in a brute, checker or interactor, in the file or not, stays
/// out of the stress report; `cpl lint` is where it shows (`stress/U33`).
#[test]
fn helper_warnings_stay_out_of_the_stress_report() {
    let sloppy = "    let unused = 3\n    var never_changed = 0\n";
    let files = [
        format!("let n: 1..=5 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=5 = read()\n{sloppy}    print(n + n)\n"),
        format!("let n: 1..=5 = read()\nprint(n)\n\n@check\nfn valid():\n    let n: 1..=5 = read()\n{sloppy}    let out: Int = read_output()\n    assert out == n\n"),
        format!("let n: Int = read()\nprint(n)\n\n@interactor\nfn judge(mut sol: Interactor):\n    let n: 1..=5 = read()\n{sloppy}    sol.send(n)\n    let q: Int = sol.recv()\n    assert q == n\n"),
    ];
    for (at, source) in files.iter().enumerate() {
        let path = scratch(&format!("helper-lints-{at}"));
        std::fs::write(&path, source).unwrap();
        for format in ["text", "json"] {
            let (code, out) = cpl(&[
                "stress".as_ref(),
                path.as_os_str(),
                "--cases".as_ref(),
                "20".as_ref(),
                "--format".as_ref(),
                format.as_ref(),
            ]);
            assert_eq!(code, Some(0), "{out}");
            assert!(!out.contains("warning") && !out.contains("[L0"), "{out}");
        }
    }
    let path = scratch("helper-lints-brute");
    std::fs::write(&path, "let n: 1..=5 = read()\nprint(n * 2)\n").unwrap();
    let brute = path.with_file_name("brute.cp");
    std::fs::write(
        &brute,
        "let n: 1..=5 = read()\nlet unused = 3\nvar never_changed = 0\nprint(n + n)\n",
    )
    .unwrap();
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--brute".as_ref(),
        brute.as_os_str(),
        "--cases".as_ref(),
        "20".as_ref(),
    ]);
    assert_eq!(code, Some(0), "{out}");
    assert!(!out.contains("warning") && !out.contains("[L0"), "{out}");
}

/// The header names the build the cases ran, so a judge's verdict can be
/// laid beside it (`stress/U29`).
#[test]
fn stress_names_the_build_its_cases_ran() {
    let path = scratch("build-named");
    std::fs::write(
        &path,
        "// judge: codeforces\nlet n: 1..=5 = read()\nprint(n * 2)\n\n@alt(program)\nfn slow():\n    let n: 1..=5 = read()\n    print(n + n)\n",
    )
    .unwrap();
    let (code, out) = cpl(&[
        "stress".as_ref(),
        path.as_os_str(),
        "--cases".as_ref(),
        "5".as_ref(),
    ]);
    assert_eq!(code, Some(0), "{out}");
    let build = out.lines().nth(1).unwrap_or("");
    assert!(build.starts_with("  build: judge `codeforces`: "), "{out}");
    assert!(build.ends_with("; checked, as `cpl run`"), "{out}");
}
