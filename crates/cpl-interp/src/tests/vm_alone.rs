//! Programs the VM runs with no fallback to the tree-walker, and that agree
//! with it.

use super::*;

/// Runs `source` on the VM, which must compile its main and its own fns
/// with no fallback to the tree-walker but on the way to a trap, and on the
/// tree-walker (a profiled run), which must agree.
fn vm_runs_alone(source: &str, input: &str) -> String {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &db.facts_source()).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let own = |id: &usize| {
        let name = program.fns[*id].name.rsplit('.').next().unwrap_or_default();
        source.contains(&format!("fn {name}("))
    };
    let fns = (0..program.fns.len()).filter(own);
    let chunks =
        std::iter::once(&compiled.main).chain(fns.filter_map(|id| compiled.fns[id].as_ref()));
    for chunk in chunks {
        let stmts: Vec<_> = chunk
            .fallback_stmts
            .iter()
            .filter(|s| !matches!(s, cpl_ir::Stmt::Trap(..) | cpl_ir::Stmt::Assert { .. }))
            .collect();
        assert!(
            stmts.is_empty() && chunk.fallback_exprs.is_empty(),
            "the VM fell back on {stmts:?} {:?}",
            chunk.fallback_exprs
        );
    }
    let (out, trap) = outcome(source, input);
    assert_eq!(trap, None);
    let mut tree = Vec::new();
    let io = crate::Io::fixed(input.as_bytes());
    let observed = crate::run_profiled(&program, io, &mut tree);
    assert!(observed.trap.is_none());
    assert_eq!(String::from_utf8(tree).expect("ASCII"), out);
    out
}

#[test]
fn vm_applies_variants_tuples_casts_and_comparisons_to_registers() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_apply_operands.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_apply_operands.out")
    );
}

#[test]
fn vm_applies_builtins_arithmetic_and_negation_to_registers() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_apply_builtins.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_apply_builtins.out")
    );
}

#[test]
fn vm_reads_specs_with_bounds_in_registers() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_read_specs.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_read_specs.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_read_specs.out")
    );
}

#[test]
fn vm_defines_domains_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_def_domain.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_def_domain.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_def_domain.out")
    );
}

#[test]
fn vm_mutates_places_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_mutate_place.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_mutate_place.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_mutate_place.out")
    );
}

#[test]
fn vm_steps_ranges_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_for_step.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_for_step.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_for_step.out")
    );
}

#[test]
fn vm_reads_and_stores_hash_maps_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_hash_map.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_hash_map.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_hash_map.out")
    );
}

#[test]
fn vm_walks_map_entries_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_map_key_place.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_map_key_place.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_map_key_place.out")
    );
}

#[test]
fn vm_walks_sets_maps_and_optionals_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_for_items.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_for_items.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_for_items.out")
    );
}

#[test]
fn vm_sorts_by_keys_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_sort_key.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_sort_key.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_sort_key.out")
    );
}

#[test]
fn vm_walks_range_values_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_range_val.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_range_val.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_range_val.out")
    );
}

#[test]
fn vm_runs_unit_builtin_statements_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_unit_builtins.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_unit_builtins.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_unit_builtins.out")
    );
}

#[test]
fn vm_index_of_a_computed_value_reads_and_traps() {
    let src = "\
fn sq(n: Int) -> List<Int> = [i * i for i in 0..n]

struct Rows:
    m: Map<Int, List<Int>>

let r = Rows(m: Map())
var t = 0
for i in 0..5:
    t += sq(5)[i]
print(t, r.m.len)
var u = 0
for i in 0..4:
    u += sq(3)[i]
print(u)
";
    let (out, trap) = outcome(src, "");
    assert_eq!(out, "30 0\n");
    let msg = format!("{trap:?}");
    assert!(
        msg.contains("out of range") || msg.contains("outside"),
        "{msg}"
    );
}

#[test]
fn vm_swaps_optional_elements_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_swap_opt_elements.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_swap_opt_elements.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_swap_opt_elements.out")
    );
}

/// A traced `swap` of two cells of a packed grid takes and stores each
/// cell on the VM, reporting the taken cell before the store.
#[test]
fn traced_vm_swaps_grid_cells_itself() {
    let source = include_str!("../../../cpl-cli/tests/programs/vm_traced_swap_grid.cp");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &db.facts_source()).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, true, false);
    let writes = (compiled.main.fallback_stmts.iter())
        .filter(|s| matches!(s, cpl_ir::Stmt::Let(..) | cpl_ir::Stmt::Assign(..)))
        .count();
    assert_eq!(writes, 0, "a traced swap fell back");
}

#[test]
fn vm_feeds_loops_from_fns_with_mut_args_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_fed_mut_args.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_fed_mut_args.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_fed_mut_args.out")
    );
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/yields_mut_arg_feeds.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/yields_mut_arg_feeds.out")
    );
}

#[test]
fn vm_feeds_loops_from_fns_with_an_assert_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/zip_rev.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/zip_rev.out")
    );
}

#[test]
fn vm_combines_boxed_values_in_place_itself() {
    let out = vm_runs_alone(
        include_str!("../../../cpl-cli/tests/programs/vm_assign_op_values.cp"),
        include_str!("../../../cpl-cli/tests/programs/vm_assign_op_values.in"),
    );
    assert_eq!(
        out,
        include_str!("../../../cpl-cli/tests/programs/vm_assign_op_values.out")
    );
}
