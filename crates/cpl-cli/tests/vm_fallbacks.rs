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
fn a_tuple_local_lent_as_a_mut_arg_runs_on_the_vm() {
    let program = tests_dir().join("programs/mut_tuple_arg_write_back.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn a_query_over_a_library_domain_runs_on_the_vm() {
    for program in [
        "programs/generic_library_domain_seq_queries.cp",
        "../../../spec/conformance/std-window/window-fold-picks-op-strategy.cp",
    ] {
        for traced in [false, true] {
            assert_eq!(
                walked(&tests_dir().join(program), traced),
                0,
                "{program} {traced}"
            );
        }
    }
}

#[test]
fn a_pop_from_any_list_place_runs_on_the_vm() {
    let program = tests_dir().join("programs/vm_pop_places.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn int_calls_past_the_mixed_frame_run_on_the_vm() {
    let program = tests_dir().join("programs/vm_calls_past_the_mixed_frame.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn an_optional_int_runs_on_the_vm() {
    let program = tests_dir().join("programs/vm_opt_int_shapes.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn a_small_tuple_from_a_field_array_runs_on_the_vm() {
    let program = tests_dir().join("programs/vm_small_tuple_from_field_array.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}

#[test]
fn sort_print_and_debug_as_values_run_on_the_vm() {
    let debug = "../../../spec/conformance/tools-trace/debug-as-a-value-prints-with-its-line.cp";
    for program in ["programs/vm_unit_effects_as_values.cp", debug] {
        for traced in [false, true] {
            assert_eq!(
                walked(&tests_dir().join(program), traced),
                0,
                "{program} traced: {traced}"
            );
        }
    }
}

#[test]
fn plain_swaps_and_local_cell_reads_run_on_the_vm() {
    for program in [
        "programs/vm_scalar_swaps_and_local_reads.cp",
        "programs/vm_swap_tuple_and_optional_parts.cp",
    ] {
        for traced in [false, true] {
            assert_eq!(
                walked(&tests_dir().join(program), traced),
                0,
                "{program} traced: {traced}"
            );
        }
    }
}

/// `m[k].insert(x)` / `.remove(x)` under a user-`Ord` or `@cache` key run on
/// the VM, as do the map literal's entry stores.
#[test]
fn a_set_in_an_entry_found_by_user_order_changes_on_the_vm() {
    let program = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../spec/conformance/core-05/map-entry-set-under-a-user-ord-or-cached-key.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
}
