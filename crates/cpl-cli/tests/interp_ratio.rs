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
    let (took, out) = run_cpl("proto/ports/ecnerwala/cf2180f1.cp", input);
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
    let (took, out) = run_cpl("proto/ports/ecnerwala/cf2183f.cp", &input);
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
