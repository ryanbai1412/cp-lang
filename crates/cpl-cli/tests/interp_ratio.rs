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
    // The ports' cases are seed lists (problem-packages/R9), so each input is written here;
    // ccc03s4's is three
    // strings of 1500 letters from a fixed LCG, the size of its old committed 01.in.
    let mut x: u32 = 1;
    let mut letters = || {
        (0..1500)
            .map(|_| {
                x = x.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                (b'a' + (x >> 16) as u8 % 3) as char
            })
            .collect::<String>()
    };
    let strings = format!("3\n{}\n{}\n{}\n", letters(), letters(), letters());
    let cases = [
        (
            "corpus/problems/dmopc20c4p4/spec.cp",
            b"51\n0 1 2 2 3 1 0 4 7 9 3 11 2 12 14 15 13 4 16 13 2 17 2 11 18 1 19 20 6 21 18 12 22 28 29 2 30 27 31 32 22 13 33 5 34 37 30 38 42 3 8\n".to_vec(),
        ),
        ("corpus/problems/ccc03s4/spec.cp", strings.into_bytes()),
        (
            "corpus/problems/aac5p4/spec.cp",
            b"26 1\n".to_vec(),
        ),
    ];
    for (port_rel, sample) in cases {
        let port = repo_root().join(port_rel);
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
