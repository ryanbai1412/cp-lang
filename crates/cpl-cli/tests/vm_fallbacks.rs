//! The VM's own checks over single programs. The tree-walker fallback
//! ratchet over every program in `tests/programs` (`vm_fallbacks.txt`,
//! `debugging/R4`) runs in `trace_goldens.rs`, on the same two runs per
//! program as the saved traces.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

/// The statements the tree-walker ran, fallback or not.
fn walked(program: &Path, traced: bool) -> u64 {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cmd.arg("run").arg(program);
    if traced {
        cmd.args(["--trace", "--format", "json"]);
    }
    let input = std::fs::read(program.with_extension("in")).unwrap_or_default();
    let mut child = cmd
        .env("CPL_VM_STATS", "json")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cpl");
    child.stdin.take().unwrap().write_all(&input).ok();
    let out = child.wait_with_output().expect("wait cpl");
    let stderr = String::from_utf8_lossy(&out.stderr);
    stderr
        .split("\"walked\":")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no vm stats from {}:\n{stderr}", program.display()))
}

#[test]
fn the_code_above_multitest_runs_on_the_vm() {
    let program = tests_dir().join("programs/vm_multitest_setup.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn plain_swaps_and_local_cell_reads_run_on_the_vm() {
    let program = tests_dir().join("programs/vm_scalar_swaps_and_local_reads.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}
