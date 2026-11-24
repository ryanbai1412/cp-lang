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
    let program = cpl_ir::lower(&analysis).expect("lowers");
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
    let program = cpl_ir::lower(&analysis).expect("lowers");
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

#[test]
fn vm_lends_variables_held_by_reference_to_a_closure_call() {
    // Nested fns holding their fn's `var`s by reference, as cf2066f's `fit`
    // reads its DP table: a leaf reader in a hot loop, a recursive writer of
    // an Int, an optional and a list, one calling another that reads what
    // the caller holds, and a global.
    let out = vm_runs_alone(
        "
var visits = 0
fn walk(n: Int) -> (Int, Int, Int):
    var seen = List.filled(n, false)
    var depth = 0
    var deepest: ?Int = None
    fn peek(i: Int) -> Bool = seen[i]
    fn mark(i: Int):
        visits += 1
        seen[i] = true
        depth += 1
        deepest = Some(max(deepest ?? 0, depth))
        for j in [2 * i + 1, 2 * i + 2]:
            if j < n and not peek(j): mark(j)
        depth -= 1
    mark(0)
    var hits = 0
    for i in 0..n:
        if peek(i): hits += 1
    (hits, deepest ?? -1, depth)
let (hits, deepest, depth) = walk(read())
print(hits, deepest, depth, visits)
",
        "10\n",
    );
    assert_eq!(out, "10 4 0 10\n");
}

#[test]
fn vm_reads_an_int_cell_of_arrays_nested_any_depth_in_one_op() {
    // cf2062h's `ways[hi][lo][cols]`: three levels of a global, a list of
    // arrays, and a local, each read as one `LoadNestedArrI`.
    let source = "
let n = read<Int>()
var ways: [0..3][0..n][0..4]Int = 0
for a in 0..3:
    for b in 0..n:
        for c in 0..4: ways[a][b][c] = a * 100 + b * 10 + c
let rows: List<[0..4]Int> = [ways[2][1], ways[1][0]]
fn deep(t: [0..3][0..n][0..4]Int) -> Int = t[1][n - 1][3]
var acc = 0
for a in 0..3:
    for c in 0..4: acc += ways[a][n - 1][c]
print(acc, rows[1][2] + rows[0][3], deep(ways))
";
    let (out, trap) = outcome(source, "2\n");
    assert_eq!(trap, None);
    assert_eq!(out, "1338 315 113\n");
    assert_eq!(vm_runs_alone(source, "2\n"), out);
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let nested = |c: &crate::vm::Chunk| {
        (c.ops.iter())
            .filter(|op| matches!(op, crate::vm::Op::LoadNestedArrI { .. }))
            .count()
    };
    assert_eq!(nested(&compiled.main), 3);
    let (_, trap) = outcome(&source.replace("ways[a][n - 1][c]", "ways[a][n][c]"), "2\n");
    assert!(
        trap.as_ref().is_some_and(|t| t.contains("outside")),
        "{trap:?}"
    );
}

#[test]
fn vm_lends_a_leaf_closure_its_variables_without_a_frame() {
    // A nested fn that runs nothing but its own ops (cf2066f's `fit`) is a
    // leaf: it borrows the variables it holds by reference in place. One
    // that calls another fn is not, and still lends them to its frame.
    let source = "
var total = 0
fn run(n: Int) -> (Int, Int):
    var seen = List.filled(n, 0)
    var hits = 0
    var best = (0, 0)
    fn fit(i: Int) -> Bool = seen[i] > 0 or i == 0
    fn note(i: Int):
        if seen[i] > best.0: best = (seen[i], i)
    fn past(i: Int) -> Bool = i > best.1
    fn bump(i: Int):
        seen[i] += 1
        hits += 1
        total += i
    fn twice(i: Int):
        bump(i)
        if fit(i): bump(i)
    for i in 0..n:
        if fit(i) or i % 3 == 1: twice(i)
        if i + 1 < n and fit(i): bump(i + 1)
        note(i)
        if past(i): hits += 0
    (hits, seen.sum() + best.0 * 100 + best.1 * 1000)
let (hits, sum) = run(read())
print(hits, sum, total)
";
    let out = vm_runs_alone(source, "10\n");
    assert_eq!(out, "29 1329 135\n");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let leaves: Vec<bool> = (program.closures.iter().zip(&compiled.closures))
        .filter(|(def, _)| def.captures.iter().any(|cap| cap.by_ref))
        .map(|(_, c)| c.as_ref().is_some_and(|c| c.leaf))
        .collect();
    assert_eq!(leaves.iter().filter(|&&l| l).count(), 2, "{leaves:?}");
    assert!(leaves.contains(&false), "{leaves:?}");
}
