//! End-to-end tests for trace filtering (`--trace` globs/exclusions,
//! `--test`, `--watch`, `--last`, `--depth`) and index-space `Bound(d)` views.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn cpl_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cpl")
}

fn write_temp(name: &str, src: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("trace_filters");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}_{}_{n}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    path
}

fn run_cpl(args: &[&str], input: &str) -> (i32, String, String) {
    let mut child = Command::new(cpl_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cpl");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn steps_json(doc: &str) -> &str {
    doc.split_once("\"steps\":[")
        .expect("trace doc should contain steps")
        .1
}

#[test]
fn trace_fn_globs_and_exclusions() {
    let path = write_temp(
        "fn_globs",
        "\
var log: Int = 0
fn dp_one():
    log += 1
fn dp_two():
    log += 10
fn helper():
    log += 100
fn solve():
    dp_one()
    helper()
    dp_two()
solve()
print(log)
",
    );
    let p = path.to_str().unwrap();

    // Glob `dp_*` records only `dp_one` and `dp_two` (not `helper` or top-level).
    let (code, stdout, stderr) = run_cpl(&["run", p, "--trace", "dp_*", "--format", "json"], "");
    assert_eq!(code, 0, "stderr: {stderr}");
    let steps = steps_json(&stdout);
    assert!(
        steps.contains("log += 1\""),
        "expected dp_one step: {steps}"
    );
    assert!(
        steps.contains("log += 10\""),
        "expected dp_two step: {steps}"
    );
    assert!(
        !steps.contains("log += 100"),
        "helper should be excluded: {steps}"
    );

    // Combined `solve,!helper` records inside `solve` and `dp_*`, excluding `helper`.
    let (code, stdout, stderr) =
        run_cpl(&["run", p, "--trace=solve,!helper", "--format", "json"], "");
    assert_eq!(code, 0, "stderr: {stderr}");
    let steps = steps_json(&stdout);
    assert!(
        steps.contains("log += 1\""),
        "expected dp_one inside solve: {steps}"
    );
    assert!(
        steps.contains("log += 10\""),
        "expected dp_two inside solve: {steps}"
    );
    assert!(
        !steps.contains("log += 100"),
        "helper should be excluded: {steps}"
    );

    // Exclusion-only `!helper` records top-level and `dp_*`, excluding `helper`.
    let (code, stdout, stderr) = run_cpl(&["run", p, "--trace=!helper", "--format", "json"], "");
    assert_eq!(code, 0, "stderr: {stderr}");
    let steps = steps_json(&stdout);
    assert!(
        steps.contains("print → 111"),
        "expected top-level print: {steps}"
    );
    assert!(
        !steps.contains("log += 100"),
        "helper should be excluded: {steps}"
    );
}

#[test]
fn trace_test_case_filter() {
    let path = write_temp(
        "multitest",
        "\
@!multitest
let x: Int = read()
print(x * 10)
",
    );
    let p = path.to_str().unwrap();
    let (code, stdout, stderr) = run_cpl(
        &["run", p, "--trace", "--test", "2", "--format", "json"],
        "3\n1\n2\n3\n",
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("print → 20"),
        "expected case 2 print: {stdout}"
    );
    assert!(
        !stdout.contains("print → 10"),
        "case 1 should be excluded: {stdout}"
    );
    assert!(
        !stdout.contains("print → 30"),
        "case 3 should be excluded: {stdout}"
    );

    let (code, stdout, stderr) = run_cpl(
        &["run", p, "--trace", "--test=1,3", "--format", "json"],
        "3\n1\n2\n3\n",
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.contains("print → 10"), "expected case 1: {stdout}");
    assert!(
        !stdout.contains("print → 20"),
        "case 2 should be excluded: {stdout}"
    );
    assert!(stdout.contains("print → 30"), "expected case 3: {stdout}");
}

#[test]
fn trace_watch_filter() {
    let path = write_temp(
        "watch",
        "\
var noise: Int = 0
var ans: Int = 0
for i in 1..=5:
    noise += i
ans = noise * 2
print(ans)
",
    );
    let p = path.to_str().unwrap();
    let (code, stdout, stderr) = run_cpl(
        &["run", p, "--trace", "--watch", "ans", "--format", "json"],
        "",
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let steps = steps_json(&stdout);
    assert!(
        steps.contains("ans = noise * 2"),
        "expected watched ans update: {steps}"
    );
    assert!(
        !steps.contains("noise += i"),
        "unwatched noise should not emit steps: {steps}"
    );
}

#[test]
fn trace_last_ring_buffer_keeps_tail_before_crash() {
    let path = write_temp(
        "last_ring",
        "\
var x: Int = 0
for i in 1..=20:
    x = i
let y = 1 / (x - 20)
print(y)
",
    );
    let p = path.to_str().unwrap();
    let (code, stdout, _stderr) = run_cpl(
        &["run", p, "--trace", "--last", "4", "--format", "json"],
        "",
    );
    assert_ne!(code, 0, "expected division by zero trap");
    // Count `"id":` occurrences in the steps array: should be exactly 4.
    let step_count = stdout.matches("\"stage\":").count();
    assert_eq!(
        step_count, 4,
        "expected exactly 4 steps in ring buffer, got {step_count}: {stdout}"
    );
    assert!(
        stdout.contains("failed: division by zero"),
        "tail must include failure step: {stdout}"
    );
}

#[test]
fn trace_depth_filter() {
    let path = write_temp(
        "depth",
        "\
var acc: Int = 0
fn inner():
    acc += 100
fn outer():
    acc += 10
    inner()
acc += 1
outer()
print(acc)
",
    );
    let p = path.to_str().unwrap();
    let (code, stdout, stderr) = run_cpl(
        &["run", p, "--trace", "--depth", "1", "--format", "json"],
        "",
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let steps = steps_json(&stdout);
    assert!(
        steps.contains("acc += 1\""),
        "depth 0 should be recorded: {steps}"
    );
    assert!(
        steps.contains("acc += 10\""),
        "depth 1 should be recorded: {steps}"
    );
    assert!(
        !steps.contains("acc += 100"),
        "depth 2 should be excluded: {steps}"
    );
}

#[test]
fn trace_bound_cut_labels_use_domain_base() {
    let path = write_temp(
        "bound_labels",
        "\
domain V = 1..=3
var pre: [V.cuts]Int = 0
var cut: V.cuts = V.first.l
for v in V:
    pre[v.r] = pre[v.l] + 1
    cut = v.r
print(pre[V.last.r], cut)
",
    );
    let p = path.to_str().unwrap();
    let (code, stdout, stderr) = run_cpl(&["run", p, "--trace", "--format", "json"], "");
    assert_eq!(code, 0, "stderr: {stderr}");
    // 1-based domain V = 1..=3 has 4 cuts labeled 1, 2, 3, 4 (matching `print(cut)`).
    assert!(
        stdout.contains("\"labels\":[1,2,3,4]"),
        "expected 1-based Bound(V) labels [1,2,3,4], got: {stdout}"
    );
    // Final value of `cut` (V.last.r) should be rendered as 4 in the trace, not 3.
    assert!(
        stdout.contains("\"cut\":4"),
        "expected Bound(V) value shifted to domain base 4, got: {stdout}"
    );
}

#[test]
fn trace_filter_flags_require_trace() {
    let path = write_temp("req_trace", "print(1)\n");
    let p = path.to_str().unwrap();
    for flag in ["--test=1", "--watch=x", "--last=5", "--depth=1"] {
        let (code, _stdout, stderr) = run_cpl(&["run", p, flag], "");
        assert_eq!(
            code, 2,
            "expected usage error for {flag} without --trace, stderr: {stderr}"
        );
        assert!(
            stderr.contains("--trace"),
            "expected --trace hint for {flag}: {stderr}"
        );
    }
}
