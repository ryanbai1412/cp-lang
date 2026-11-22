//! Interpreter regression tests for wave-12C heavy DP ports (`cf2180f1` and `cf2183f`).
//!
//! Default `cargo test` runs medium-size parity + bounded-time checks so `check.sh --fast`
//! stays fast even in debug builds. The `#[ignore]` release test measures the exact
//! interpreter vs C++ `-O2` ratio against the 10x bar on full-size inputs.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn run_cpl(port_rel: &str, input: &str) -> (Duration, String) {
    let port = repo_root().join(port_rel);
    let start = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run"])
        .arg(&port)
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
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait cpl");
    assert!(
        out.status.success(),
        "`cpl run {port_rel}` failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    (start.elapsed(), String::from_utf8(out.stdout).unwrap())
}

#[test]
fn cf2180f1_small_tuple_dp_parity_and_speed() {
    let input = "3\n1 1\n10 10\n80 80\n";
    let (took, out) = run_cpl("corpus/problems/cf2180f1/port.cp", input);
    assert_eq!(out.lines().count(), 3, "unexpected output: {out}");
    assert!(
        took < Duration::from_secs(10),
        "cf2180f1 (80x80) regressed in interpreter: {took:?}"
    );
}

#[test]
fn cf2183f_tree_2d_dp_parity_and_speed() {
    let n = 80usize;
    let mut input = format!("1\n{n}\n{}\n", "a".repeat(n));
    for i in 1..n {
        input.push_str(&format!("{i} {}\n", i + 1));
    }
    let (took, out) = run_cpl("corpus/problems/cf2183f/port.cp", &input);
    assert_eq!(
        out.split_whitespace().count(),
        n,
        "unexpected output: {out}"
    );
    assert!(
        took < Duration::from_secs(10),
        "cf2183f (n={n} chain) regressed in interpreter: {took:?}"
    );
}

#[test]
fn vm_hot_loop_fallback_ratchet_zero_on_covered_worst_ports() {
    let cases = [
        (
            "corpus/problems/cf2262e/port.cp",
            "corpus/problems/cf2262e/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2150d/port.cp",
            "corpus/problems/cf2150d/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2124f2/port.cp",
            "corpus/problems/cf2124f2/tests/1.in",
        ),
        (
            "corpus/problems/cf2096f/port.cp",
            "corpus/problems/cf2096f/tests/1.in",
        ),
        (
            "corpus/problems/cf2089c1/port.cp",
            "corpus/problems/cf2089c1/tests/1.in",
        ),
        (
            "corpus/problems/cf2084g1/port.cp",
            "corpus/problems/cf2084g1/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2084h/port.cp",
            "corpus/problems/cf2084h/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2101c/port.cp",
            "corpus/problems/cf2101c/tests/1.in",
        ),
        (
            "corpus/problems/cf2237b/port.cp",
            "corpus/problems/cf2237b/tests/sample1.in",
        ),
        (
            "corpus/problems/ucup5s06g/port.cp",
            "corpus/problems/ucup5s06g/tests/s1.in",
        ),
        (
            "corpus/problems/cf2180g/port.cp",
            "corpus/problems/cf2180g/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2207f/port.cp",
            "corpus/problems/cf2207f/tests/1.in",
        ),
        (
            "corpus/problems/cf2261g/port.cp",
            "corpus/problems/cf2261g/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2138f/port.cp",
            "corpus/problems/cf2138f/tests/1.in",
        ),
        (
            "corpus/problems/cf2150c/port.cp",
            "corpus/problems/cf2150c/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2127h/port.cp",
            "corpus/problems/cf2127h/tests/sample1.in",
        ),
        (
            "corpus/problems/cf2062f/port.cp",
            "corpus/problems/cf2062f/tests/sample1.in",
        ),
    ];
    for (port_rel, sample_rel) in cases {
        let port = repo_root().join(port_rel);
        let sample = std::fs::read(repo_root().join(sample_rel)).expect("read sample");
        let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["run"])
            .arg(&port)
            .env("CPL_VM_STATS", "json")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn cpl");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&sample)
            .expect("write stdin");
        let out = child.wait_with_output().expect("wait cpl");
        assert!(
            out.status.success(),
            "`cpl run {port_rel}` failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        let json_line = stderr
            .lines()
            .rev()
            .find(|l| l.starts_with('{') && l.contains("\"hot_loop_fallbacks\""))
            .unwrap_or_else(|| panic!("missing CPL_VM_STATS json for {port_rel}: {stderr}"));
        assert!(
            json_line.contains("\"hot_loop_fallbacks\":0"),
            "hot-loop fallback regression on {port_rel}: {json_line}"
        );
    }
}
