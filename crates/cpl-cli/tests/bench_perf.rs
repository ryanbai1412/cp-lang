use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cpl-bench-test-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn zero_cost_runner_compares_pair_and_emits_json() {
    let root = tmp_dir("zc");
    let pair = root.join("same");
    std::fs::create_dir_all(&pair).unwrap();
    let src = "\
fn kernel(n: Int) -> Int:
    var s = 0
    for i in 0..n:
        s = (s + i * 3) ^ (s >> 2)
    s

let n: Int = read()
print(kernel(n))
";
    std::fs::write(pair.join("abstract.cp"), src).unwrap();
    std::fs::write(pair.join("direct.cp"), src).unwrap();
    std::fs::write(
        pair.join("hand.cpp"),
        r#"#include <cstdio>
int main() {
    long long n;
    if (scanf("%lld", &n) != 1) return 1;
    long long s = 0;
    for (long long i = 0; i < n; i++) s = (s + i * 3) ^ (s >> 2);
    printf("%lld\n", s);
    return 0;
}
"#,
    )
    .unwrap();
    std::fs::write(pair.join("input.txt"), "2000000\n").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "bench",
            "--zero-cost",
            root.to_str().unwrap(),
            "--reps",
            "3",
            "--max-ratio",
            "1.5",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains("\"status\":\"ok\""), "stdout={stdout}");
    assert!(stdout.contains("\"name\":\"same\""), "stdout={stdout}");
    assert!(stdout.contains("\"asm\":\"equal\""), "stdout={stdout}");
    assert!(stdout.contains("\"cpp_ms\":"), "stdout={stdout}");
    assert!(stdout.contains("\"cpp_ratio\":"), "stdout={stdout}");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn corpus_bench_uses_perf_cp_checks_tests_and_writes_schema() {
    let root = tmp_dir("corpus");
    let prob = root.join("demo_p1");
    std::fs::create_dir_all(prob.join("tests")).unwrap();

    // Original C++ solution
    std::fs::write(
        prob.join("orig.cpp"),
        r#"#include <cstdio>
int main() {
    long long n;
    if (scanf("%lld", &n) != 1) return 1;
    long long s = 0;
    for (long long i = 0; i < n; i++) s = (s + i * 3) ^ (s >> 2);
    printf("%lld\n", s);
    return 0;
}
"#,
    )
    .unwrap();

    // Base spec.cp and tuned perf.cp
    let good_cp = "\
let n: Int = read()
var s = 0
for i in 0..n:
    s = (s + i * 3) ^ (s >> 2)
print(s)
";
    std::fs::write(prob.join("spec.cp"), good_cp).unwrap();
    std::fs::write(prob.join("perf.cp"), good_cp).unwrap();
    std::fs::write(prob.join("tests/1.in"), "10\n").unwrap();
    // Compute exact expected answer for n=10:
    // i=0..9: s = (s + i*3) ^ (s >> 2) -> let's use n=1 -> 0
    std::fs::write(prob.join("tests/1.in"), "1\n").unwrap();
    std::fs::write(prob.join("tests/1.ans"), "0\n").unwrap();
    std::fs::write(prob.join("gen.py"), "print(500000)\n").unwrap();

    let out_json = root.join("results.json");
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "bench",
            "--corpus",
            "--dir",
            root.to_str().unwrap(),
            "--runs",
            "3",
            "--out",
            out_json.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stdout.contains("[perf.cp]"), "stdout={stdout}\nstderr={stderr}");
    let json = std::fs::read_to_string(&out_json).unwrap();
    assert!(json.contains("\"tuned\":true"), "json={json}");
    assert!(json.contains("\"time_above_floor\":"), "json={json}");
    assert!(json.contains("\"rss_above_floor\":"), "json={json}");
    assert!(json.contains("\"target_met\":"), "json={json}");

    // Now break perf.cp on tests/1.ans and verify corpus bench rejects it
    std::fs::write(prob.join("perf.cp"), "let n: Int = read()\nprint(n + 999)\n").unwrap();
    let bad = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "bench",
            "--corpus",
            "--dir",
            root.to_str().unwrap(),
            "--runs",
            "1",
            "--no-write",
        ])
        .output()
        .unwrap();
    assert!(
        !bad.status.success(),
        "expected failure when perf.cp fails tests: {}",
        String::from_utf8_lossy(&bad.stdout)
    );
    assert!(
        String::from_utf8_lossy(&bad.stdout).contains("perf.cp WA"),
        "stdout={}",
        String::from_utf8_lossy(&bad.stdout)
    );

    let _ = std::fs::remove_dir_all(&root);
}
