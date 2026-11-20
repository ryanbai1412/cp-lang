//! What `cpl stress`, `cpl test` and `cpl bench` say when a run goes wrong.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(test: &str, name: &str, source: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stress_bench").join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, source).unwrap();
    path
}

/// `cpl args...`: its stdout and stderr together.
fn cpl(args: &[&OsStr]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl")).args(args).output().expect("cpl runs");
    format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
}

const DIVIDES_BY_ZERO: &str = "\
let n: 0..=0 = read()
print(10 / n)

@brute(main)
fn slow():
    print(0)
";

#[test]
fn stress_and_test_show_the_runtime_error_and_its_line() {
    let path = scratch("runtime_error", "d.cp", DIVIDES_BY_ZERO);
    let stressed = cpl(&["stress".as_ref(), path.as_os_str(), "--cases".as_ref(), "5".as_ref()]);
    assert!(stressed.contains("error[R0001]"), "{stressed}");
    assert!(stressed.contains("d.cp:2"), "{stressed}");
    assert!(!stressed.contains("crashed"), "{stressed}");
    let tested = cpl(&["test".as_ref(), path.as_os_str()]);
    assert!(tested.contains("error[R0001]") && tested.contains("d.cp:2"), "{tested}");
}

#[test]
fn bench_says_why_a_run_crashed() {
    let path = scratch("bench_crash", "d.cp", DIVIDES_BY_ZERO);
    let benched = cpl(&["bench".as_ref(), path.as_os_str()]);
    assert!(benched.contains("runtime error on "), "{benched}");
    assert!(benched.contains("crashed"), "{benched}");
    assert!(!benched.contains("over the limits"), "{benched}");
}

#[test]
fn a_read_inside_an_expression_is_an_error_not_half_an_input() {
    let path = scratch("nested_read", "g.cp", "let n: 1..=5 = read()\nlet m: Int = n + read()\nprint(m)\n\n@brute(main)\nfn slow():\n    print(m)\n");
    for command in ["bench", "stress"] {
        let out = cpl(&[command.as_ref(), path.as_os_str()]);
        assert!(out.contains("g.cp:2:") && out.contains("not a top-level"), "{command}: {out}");
    }
}

#[test]
fn bench_makes_inputs_for_a_multitest_fn() {
    let path = scratch(
        "multitest_fn",
        "m.cp",
        "@multitest\nfn solve():\n    let n: 1..=100 = read()\n    let a: [n]1..=9 = read()\n    print(sum(a))\n",
    );
    let benched = cpl(&["bench".as_ref(), path.as_os_str()]);
    assert!(benched.contains("within limits on every shape"), "{benched}");
}

const PAIRED: &str = "\
let n: 1..=20 = read()

fn f(x: Int) -> Int = x + 1

@brute(f)
fn f_slow(x: Int) -> Int = x + OFFSET

print(f(n))
";

#[test]
fn stress_compares_a_fn_with_its_brute() {
    let wrong = scratch("paired_wrong", "p.cp", &PAIRED.replace("OFFSET", "2"));
    let out = cpl(&["stress".as_ref(), wrong.as_os_str(), "--cases".as_ref(), "20".as_ref()]);
    assert!(out.contains("with its @brute fns swapped in"), "{out}");
    assert!(out.contains("FAIL on case 1") && out.contains("WA"), "{out}");
    let right = scratch("paired_right", "p.cp", &PAIRED.replace("OFFSET", "1"));
    let out = cpl(&["stress".as_ref(), right.as_os_str(), "--cases".as_ref(), "20".as_ref()]);
    assert!(out.contains("20 cases agree"), "{out}");
    let leftovers = std::fs::read_dir(right.parent().unwrap()).unwrap().flatten();
    assert!(leftovers.into_iter().all(|e| !e.file_name().to_string_lossy().contains(".brute-")));
}

const CHECKED: &str = "\
let n: 1..=5 = read()
print(n)

@checker
fn check():
    let x: Int = read_output()
    EXTRA
    assert x == n
";

#[test]
fn gen_takes_a_command_line() {
    let path = scratch("gen_command", "c.cp", &CHECKED.replace("EXTRA", ""));
    let out = cpl(&["stress".as_ref(), path.as_os_str(), "--gen".as_ref(), "sh -c 'echo 3'".as_ref(), "--cases".as_ref(), "4".as_ref()]);
    assert!(out.contains("4 cases agree"), "{out}");
}

#[test]
fn a_checker_that_reads_past_the_output_says_output() {
    let path = scratch("checker_output", "c.cp", &CHECKED.replace("EXTRA", "let y: Int = read_output()"));
    let out = cpl(&["stress".as_ref(), path.as_os_str(), "--cases".as_ref(), "3".as_ref()]);
    assert!(out.contains("the solution's output ended where a number was expected"), "{out}");
    assert!(!out.contains("the input ended"), "{out}");
}
