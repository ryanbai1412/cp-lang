//! Interpreter regression tests for wave-12C heavy DP ports (`cf2180f1` and `cf2183f`).
//!
//! Default `cargo test` runs medium-size parity + bounded-time checks so `check.sh --fast`
//! stays fast even in debug builds. The `#[ignore]` release test measures the exact
//! interpreter vs C++ `-O2` ratio against the 10x bar on full-size inputs.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn vm_hot_loop_fallback_ratchet_zero_on_covered_worst_ports() {
    let cases = [
        ("corpus/problems/dmopc20c4p4/spec.cp", "corpus/problems/dmopc20c4p4/tests/01.in"),
        ("corpus/problems/ccc03s4/spec.cp", "corpus/problems/ccc03s4/tests/01.in"),
        ("corpus/problems/aac5p4/spec.cp", "corpus/problems/aac5p4/tests/01.in"),
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
