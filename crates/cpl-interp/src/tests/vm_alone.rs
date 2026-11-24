//! Programs the VM runs with no fallback to the tree-walker, and that agree
//! with it.

use super::*;

/// Runs `source` on the VM, which must compile its main and its own fns
/// with no fallback to the tree-walker but on the way to a trap, and on the
/// tree-walker (a profiled run), which must agree, trap or not.
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
    let mut tree = Vec::new();
    let io = crate::Io::fixed(input.as_bytes());
    let observed = crate::run_profiled(&program, io, &mut tree);
    assert_eq!(observed.trap.is_some(), trap.is_some());
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

#[test]
fn vm_matches_payload_variants_in_registers() {
    let source = "\
enum Slot:
    Known(t: Int)
    Pair(a: Int, b: List<Int>)
    Empty

fn species(s: Slot, mask: Int) -> Int:
    match s:
        Known(t) => t
        Pair(a, b) => a + b.len + (mask >> a) & 1
        Empty => -1

let slots = [Known(4), Pair(1, [2, 3]), Empty, Known(-2)]
print([species(s, 6) for s in slots])
";
    assert_eq!(vm_runs_alone(source, ""), "4 0 -1 -2\n");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let id = (0..program.fns.len()).find(|&id| program.fns[id].name.ends_with("species"));
    let chunk = compiled.fns[id.expect("species")]
        .as_ref()
        .expect("on the VM");
    let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    assert_eq!(count("ApplyV"), 0, "{ops:?}");
    assert_eq!(count("IsVariantI"), 3, "{ops:?}");
}

#[test]
fn vm_traps_a_call_to_a_cpp_fn_itself() {
    let source = "fn pc(a: Int) -> Int = @cpp(\"return __builtin_popcountll(a);\")\nprint(1)\nprint(pc(7))\n";
    assert_eq!(vm_runs_alone(source, ""), "1\n");
}

#[test]
fn vm_field_of_a_temporary_frees_it_in_the_same_op() {
    // `FieldI` and `FieldV` drop a temporary they read, with no `ClearV`.
    let source = "\
struct Pair(a: Int, b: List<Int>)
fn mk(i: Int) -> ?Pair = Some(Pair(i, [i, i + 1]))
fn g(i: Int) -> Int:
    let l = mk(i)!.b
    mk(i)!.a + l.len + l[1]
print([g(i) for i in 0..3])
";
    assert_eq!(vm_runs_alone(source, ""), "3 5 7\n");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let id = (0..program.fns.len()).find(|&id| program.fns[id].name.ends_with("g"));
    let chunk = compiled.fns[id.expect("g")].as_ref().expect("on the VM");
    let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    assert_eq!((count("FieldI"), count("FieldV")), (1, 1), "{ops:?}");
    assert_eq!(count("ClearV"), 0, "{ops:?}");
}

#[test]
fn vm_returns_a_small_tuple_it_holds_boxed_as_it_is() {
    // dmopc20c7p2's `equal_range(p, by: e => e.from)`: the key of a boxed
    // nested tuple is already a boxed `Pt`, so it is returned as it is, not
    // unboxed into registers for the caller to box again.
    let source = "\
struct Pt(x: Int, y: Int)
var edges: List<(from: Pt, to: Pt, w: Int)> = []
for i in 0..6:
    edges.push((from: Pt(i / 2, i % 2), to: Pt(i, 0), w: i))
fn from_of(e: (from: Pt, to: Pt, w: Int)) -> Pt = e.from
fn twin(p: Pt) -> Pt = Pt(p.y, p.x)
print([edges[i].w for i in edges.equal_range(Pt(1, 0), by: e => e.from)])
print(from_of(edges[3]).x, from_of(edges[3]).y, twin(from_of(edges[5])).x)
";
    assert_eq!(vm_runs_alone(source, ""), "2\n1 1 1\n");
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let ops = |chunk: &crate::vm::Chunk| -> Vec<String> {
        chunk.ops.iter().map(|op| format!("{op:?}")).collect()
    };
    let count = |ops: &[String], name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    let fn_ops = |name: &str| {
        let id = (0..program.fns.len()).find(|&id| program.fns[id].name.ends_with(name));
        ops(compiled.fns[id.expect(name)].as_ref().expect("on the VM"))
    };
    let from_of = fn_ops("from_of");
    assert_eq!(count(&from_of, "UnboxSmallTuple"), 0, "{from_of:?}");
    assert_eq!(count(&from_of, "FieldV"), 0, "{from_of:?}");
    let twin = fn_ops("twin");
    assert_eq!(count(&twin, "RetSmallTuple"), 1, "{twin:?}");
    let key = compiled
        .closures
        .iter()
        .flatten()
        .map(ops)
        .collect::<Vec<_>>();
    assert!(!key.is_empty());
    for ops in &key {
        assert_eq!(count(ops, "UnboxSmallTuple"), 0, "{ops:?}");
    }
}

#[test]
fn vm_keeps_a_tuple_of_small_structs_in_registers_leaf_by_leaf() {
    // dmopc20c7p2's `(from: Pt, to: Pt, tile)` edges: a tuple of small structs
    // is its scalars in Int registers, read field by field without a box.
    let source = "\
struct Pt(x: Int, y: Int)
var edges: List<(from: Pt, to: Pt, w: Int)> = []
for i in 0..6:
    edges.push((from: Pt(i / 2, i % 2), to: Pt(i, 7 - i), w: i * 10))
fn span(e: (from: Pt, to: Pt, w: Int)) -> Int = e.to.x - e.from.y + e.w
fn flip(e: (from: Pt, to: Pt, w: Int)) -> (from: Pt, to: Pt, w: Int) = (from: e.to, to: e.from, w: e.w + 1)
var total = 0
for i in 0..6:
    let f = flip(edges[i])
    total += span(f) * 100 + f.from.y
    edges[i] = f
print(total, edges[2], edges.equal_range(Pt(3, 4), by: e => e.from))
print(flip(edges[5]).to, [span(e) for e in edges])
";
    assert_eq!(
        vm_runs_alone(source, ""),
        "13527 2 5 1 0 21 3\n5 2 -6 5 17 28 40 51\n"
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let fn_ops = |name: &str| -> Vec<String> {
        let id = (0..program.fns.len()).find(|&id| program.fns[id].name.ends_with(name));
        let chunk = compiled.fns[id.expect(name)].as_ref().expect("on the VM");
        chunk.ops.iter().map(|op| format!("{op:?}")).collect()
    };
    for name in ["span", "flip"] {
        let ops = fn_ops(name);
        let boxed = [
            "FieldV",
            "FieldI",
            "BoxSmallTuple",
            "UnboxSmallTuple",
            "RetV",
        ];
        assert!(
            !ops.iter().any(|op| boxed.iter().any(|b| op.starts_with(b))),
            "{name}: {ops:?}"
        );
    }
}
