//! Calls, closures and `mut` arguments on the VM: what is lent, what is put back.

use super::*;

#[test]
fn vm_handles_mixed_calls_tail_returns_structs_and_strings() {
    let source = "\
struct State(a: Int, b: ?Int, s: Str)

fn step(st: State, i: Int) -> State:
    let ch = st.s[i].ord
    let next_b: ?Int = if let prev = st.b: Some(prev + ch) else: Some(ch)
    State(st.a + 1, next_b, st.s)

fn find_first(s: Str, target: Char, i: Int) -> ?Int:
    if i >= s.len: return None
    if s[i] == target: return i
    find_first(s, target, i + 1)

let s: Str = read()
var st = State(0, None, s.clone())
for i in 0..s.len:
    st = step(st, i)
let idx = find_first(s, s[2], 0) ?? -1
print(st.a, st.b ?? 0, idx)
";
    assert_eq!(outcome(source, "abc\n"), ("3 294 2\n".to_string(), None));
}

#[test]
fn vm_mut_ref_calls_guarded_domains_and_field_writes() {
    let source = "
struct Counter:
    var xs: List<Int>
    var total: Int
    fn add(mut self, x: Int) -> Int:
        self.xs.push(x)
        self.total = self.total + x
        return self.total
    fn bump_at(mut self, i: Int, d: Int):
        self.xs[i] = self.xs[i] + d

fn solve(n: Int) -> Int:
    domain I = 0..n
    var a: [I]Int = [0 for _ in I]
    var c = Counter([], 0)
    for i in I:
        c.add(i.ord + 1)
        a[i] = c.total
    c.bump_at(0, 100)
    return a[I.last!] + c.xs[0]

print(solve(4), solve(3))
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "111 107\n");
}

#[test]
fn vm_closure_calls_with_and_without_captures() {
    let source = "
fn apply2(f: (Int, Int) -> Int, a: Int, b: Int) -> Int:
    return f(a, b)

let k = 10
let add = (x: Int, y: Int) => x + y
let scaled = (x: Int, y: Int) => (x + y) * k
var acc = 0
for i in 0..5:
    acc += apply2(add, i, 1) + apply2(scaled, i, 1)
print(acc)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "165\n");
}

#[test]
fn vm_closure_by_ref_and_mut_callback_parity() {
    let source = "
fn use_with_cb(mut xs: List<Int>, f: () -> Int):
    xs.push(f())

var acc = 0
fn bump(d: Int):
    acc += d

for i in 1..=5:
    bump(i)

var ys: List<Int> = [10, 20]
var zs = [1, 2, 3]
use_with_cb(mut ys, () => zs.len)
print(acc, ys.len, ys[2])
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "15 3 3\n");
}

#[test]
fn vm_runs_closures_that_capture_by_reference_itself() {
    // Nested fns reading and writing their fn's variables, one calling
    // another, as `split_best` keeps its heap; and one writing a global.
    let source = "
var calls = 0
fn heap_order(xs: List<Int>) -> List<Int>:
    var vals: List<Int> = []
    var heap: List<Int> = []
    var swaps = 0
    fn less(a: Int, b: Int) -> Bool:
        calls += 1
        vals[a] < vals[b]
    fn push(x: Int):
        vals.push(x)
        heap.push(vals.len - 1)
        var i = heap.len - 1
        while i > 0 and less(heap[i], heap[(i - 1) >> 1]):
            let p = (i - 1) >> 1
            let t = heap[i]
            heap[i] = heap[p]
            heap[p] = t
            swaps += 1
            i = p
    for x in xs: push(x)
    var out: List<Int> = []
    while heap.len > 0:
        out.push(vals[heap[0]])
        let last = heap.pop()!
        if heap.len > 0:
            heap[0] = last
            var i = 0
            while true:
                var m = i
                for c in [2 * i + 1, 2 * i + 2]:
                    if c < heap.len and less(heap[c], heap[m]): m = c
                if m == i: break
                let t = heap[i]
                heap[i] = heap[m]
                heap[m] = t
                i = m
    out.push(swaps)
    out
print(heap_order([5, 3, 8, 1, 9, 2]), calls)
";
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
    for (cid, def) in program.closures.iter().enumerate() {
        if def.captures.iter().any(|cap| cap.by_ref) {
            assert!(compiled.closures[cid].is_some(), "closure {cid} is walked");
        }
    }
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "1 2 3 5 8 9 4 15\n");
}

#[test]
fn vm_runs_cuts_none_tests_wide_max_and_fn_values_itself() {
    // bench/seq/scan's `cut_after`, bench/ranges/lazy_assign's `act != None`
    // and three-way `max`: none leaves the VM, and a fn passed as a value is
    // called directly.
    let source = "
struct Box2(a: Int, b: Int)
fn pair(x: Int, y: Int) -> Int = x * 10 + y
fn run(f: (Int, Int) -> Int) -> Int = f(3, 4)
domain D = 0..5
let xs: [D]Int = [5, 1, 4, 2, 3]
var cuts = 0
for i in D: cuts = cuts * 10 + Int(xs.cut_after(i))
var rcuts = 0
for i in D: rcuts = rcuts * 10 + Int(xs.rev.cut_before(i))
let b: ?Box2 = Some(Box2(1, 2))
let n: ?Box2 = None
print(cuts, rcuts, yes(b != None), yes(n == None), yes(b == None), max(xs[0], xs[1], 9), min(xs[2], xs[3], xs[4]), run(pair))
";
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
    let ops: Vec<String> = compiled
        .main
        .ops
        .iter()
        .map(|op| format!("{op:?}"))
        .collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    // `xs.rev` (making the view) and the print apply to their values.
    assert_eq!(count("ApplyV"), 2, "{ops:?}");
    assert_eq!(count("CutI"), 2, "{ops:?}");
    assert_eq!(count("IsVariantI"), 3, "{ops:?}");
    assert!(
        compiled.forwards.iter().any(Option::is_some),
        "`pair` is called through a closure"
    );
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "12345 12345 YES YES NO 9 2 34\n");
}

#[test]
fn vm_passes_a_mut_opt_element_to_a_fn() {
    // SegTree's `act.compose_onto(mut lz[p], f)` over `?Bool` tags: a `mut`
    // arg at an indexed `?T` place stays on the VM.
    let source = "
fn put(mut old: ?Bool, new: ?Bool):
    if new != None: old = new
var lz: List<?Bool> = [None, Some(true), None]
for i in 0..3: put(mut lz[i], if i == 2: Some(false) else: None)
for x in lz: print(if x == None: 0 elif x == Some(true): 1 else: 2)
";
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
    let fallback =
        (compiled.main.ops.iter()).any(|op| matches!(op, crate::vm::Op::FallbackStmt { .. }));
    assert!(!fallback, "{:?}", compiled.main.ops);
    let (out, trap) = outcome(source, "");
    assert!(trap.is_none(), "{trap:?}");
    assert_eq!(out, "0\n1\n2\n");
}

#[test]
fn vm_alias_free_mut_closure_runs_without_fallback() {
    let source = "
fn count_and_bump(mut xs: List<Int>, pred: Int -> Bool) -> Int:
    var hits = 0
    for i in 0..xs.len:
        if pred(xs[i]): hits += 1
        xs[i] += 1
    hits

fn local_driver(k: Int) -> Int:
    var loc: List<Int> = [1, 2, 3, 4]
    var tot = 0
    for _ in 0..3:
        tot += count_and_bump(mut loc, v => v > k)
    tot

var glob: List<Int> = [0, 2, 4]
let k = 2
var gtot = 0
for _ in 0..3:
    gtot += count_and_bump(mut glob, v => v > k)
print(gtot, local_driver(2), glob[0])
";
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
    let has_loop_fallback = |chunk: &crate::vm::Chunk<'_>| {
        chunk.ops.iter().any(|op| {
            matches!(
                op,
                crate::vm::Op::FallbackStmt { in_loop: true, .. }
                    | crate::vm::Op::FallbackExprI { in_loop: true, .. }
                    | crate::vm::Op::FallbackExprV { in_loop: true, .. }
            )
        })
    };
    assert!(
        !has_loop_fallback(&compiled.main),
        "alias-free global mut+closure call in main loop fell back"
    );
    for (idx, chunk) in compiled.fns.iter().enumerate() {
        if let Some(ch) = chunk {
            assert!(
                !has_loop_fallback(ch),
                "fn {} fell back inside loop",
                program.fns[idx].name
            );
        }
    }
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "5 9 3\n");
}

#[test]
fn vm_by_ref_closure_passed_to_vm_hof_str_view_and_small_tuple_if() {
    let source = "
fn apply(n: Int, f: (Int) -> Int) -> Int:
    var s = 0
    for i in 0..n:
        s += f(i)
    s

fn run_with_ref() -> Int:
    var vals: List<Int> = [10, 20, 30]
    let f = i => vals[i] + i
    let a = apply(3, f)
    vals.push(40)
    a + apply(4, f)

let s = \"abcdef\"
let v = s.rev
var dp: [0..2](Int, Int) = (0, 0)
for i in 0..2:
    dp[i] max= if i == 0: (5, 2) else: (3, 9)

print(run_with_ref(), v.nth(0), v.nth(5), dp[0].0, dp[1].1)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "169 f a 5 9\n");
}

#[test]
fn vm_fill_slice_and_indexed_mut_call() {
    let src = "\
struct Acc:
    xs: List<Int>
    total: Int

    fn push(mut self, v: Int):
        xs.push(v)
        total += v

var accs = [0..2] _ => Acc([], 0)
for i in 0..6:
    accs[i % 2].push(i + 1)

var grid: [0..2, 0..3]Int = 7
var sum = 0
for r in 0..2:
    for c in 0..3:
        sum += grid[r, c]

let s = \"abcdef\"
let sub = s[1..4]
let ys = [10, 20, 30, 40, 50]
let lv = ys[1..4]
var lsum = 0
for x in lv: lsum += x
print(accs[0].total, accs[1].total, sum, sub, lsum)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "9 12 42 bcd 90\n");
}

#[test]
fn vm_pure_leaf_temp_struct_arg_not_consumed_early() {
    let src = "\
struct P:
    x: Int
    y: Int

fn norm2(p: P) -> Int = p.x * p.x + p.y * p.y
fn cross(a: P, b: P) -> Int = a.x * b.y - a.y * b.x

let ps = [P(x: 3, y: 4), P(x: 5, y: 12)]
print(norm2(ps[0]), norm2(ps[1]), cross(ps[0], ps[1]))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "25 169 16\n");
}

#[test]
fn vm_do_fallback_preserves_outer_call_args() {
    let src = "\
use std.sparse.{SparseTable}

let n: Int = read()
let xs: [0..n]Int = read()
let st = SparseTable(min, xs)
print(st.fold(0..n), st.fold(1..2))
";
    let (out, trap) = outcome(src, "3\n4 2 9\n");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "2 2\n");
}

#[test]
fn vm_mut_arg_lent_from_a_grid_cell_writes_back() {
    let src = "\
struct Acc:
    xs: List<Int>

fn add(mut a: Acc, x: Int):
    a.xs.push(x)

var grid: [3, 3]Acc = [3, 3] (i, j) => Acc(xs: [])
for i in 0..3:
    for j in 0..3:
        add(mut grid[i, j], i * 3 + j)
        add(mut grid[j, i], 9)
print(grid[1, 2].xs, grid[2, 2].xs)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "5 9 8 9\n");
}

#[test]
fn vm_closure_take_fed_yields_range_setminmax_reverse_and_float_ops() {
    let src = "\
fn evens(n: Int) -> yields Int:
    for i in 0..n:
        if i % 2 == 0:
            yield i

var got: List<Int> = []
for x in evens(10):
    got.push(x)
    if x == 6: break
got.reverse()
var moved = got.take()
var s: Set<Int> = Set<Int>()
s.insert(30)
s.insert(10)
s.insert(20)
let r: 1..=10 = 7
let f = sqrt(F64(9))
print(got.len, moved, s.min()!, s.max()!, r, bit_width(13), yes(f > 2.5))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "0 6 4 2 0 10 30 7 4 YES\n");

    let range_trap = "\
let x: Int = read()
let r: 1..=10 = x
print(r)
";
    let (_, trap) = outcome(range_trap, "20\n");
    let msg = trap.expect("expected range trap");
    assert!(msg.contains("is outside"), "got: {msg}");

    let sqrt_trap = "\
print(sqrt(F64(-1)))
";
    let (_, trap) = outcome(sqrt_trap, "");
    let msg = trap.expect("expected sqrt negative trap");
    assert!(msg.contains("sqrt("), "got: {msg}");
}

#[test]
fn only_a_closure_made_off_the_vm_syncs_the_whole_frame_around_calls() {
    // `(has_by_ref_closure, ref_slots)` of fn `name`'s chunk, or main's.
    let syncs = |source: &str, name: Option<&str>| {
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
        let chunk = match name {
            None => &compiled.main,
            Some(name) => {
                let id = program.fns.iter().position(|f| f.name == name).expect("fn");
                compiled.fns[id].as_ref().expect("on the vm")
            }
        };
        (chunk.has_by_ref_closure, chunk.ref_slots.clone())
    };
    // A ranged read is not a closure, so calls in the loop sync nothing.
    let read = "let n: 1..=5 = read()\nfn f(x: Int) -> Int = x + n\nvar s = 0\nfor i in 0..3: s += f(i)\nprint(s)\n";
    assert_eq!(syncs(read, None), (false, Some(vec![])));
    // A closure capturing `t` by reference syncs only `t`.
    let closure = "fn call(g: () -> ()) = g()\nfn run(k: Int) -> Int:\n    var t = k\n    let u = 1\n    call(() => t += u)\n    t\nprint(run(2))\n";
    let (by_ref, slots) = syncs(closure, Some("run"));
    assert!(by_ref);
    assert_eq!(slots.map(|s| s.len()), Some(1));
}

#[test]
fn vm_compiles_keyed_sorts_searches_and_inlined_calls() {
    let src = "\
fn sq(x: Int) -> Int:
    return x * x
fn neg(x: Int) -> Int:
    return -x
var xs = [3, 1, 2]
xs.sort(by: neg)
var s: Set<Int> = []
s.insert(4)
s.insert(9)
var m: Map<Int, Int> = []
m[1] = 5
var t: [0..2](Int, Int) = (0, 0)
t[1] = (2, 3)
t[1] max= (2, 7)
var total = 0
for i in 0..4:
    if i < 2:
        total += sq(i + 1)
print(\" \".join(xs), s.first_ge(5)!, m[1], t[1].1, total)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "3 2 1 9 5 7 5\n");
}

#[test]
fn vm_takes_each_kind_of_mut_arg_and_puts_it_back() {
    let src = "\
fn bump(mut x: Int, mut o: ?Int, mut xs: List<Int>):
    x += 1
    o = Some(x)
    xs.push(x)
fn grow(mut xs: List<Int>):
    xs.push(0)
struct S:
    var xs: List<Int>
var gs = [[1], [2]]
var ga: [0..2]List<Int> = [[], []]
fn main():
    var x = 1
    var o: ?Int = None
    var xs = [5]
    bump(mut x, mut o, mut xs)
    var ls = [[1], [2, 3]]
    var la: [0..2]List<Int> = [[4], []]
    grow(mut ls[1])
    grow(mut la[0])
    grow(mut gs[0])
    grow(mut ga[1])
    var s = S(xs: [])
    grow(mut s.xs)
    print(x, o!, xs, ls[1], la[0], gs[0], ga[1], s.xs)
main()
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "2 2 5 2 2 3 0 4 0 1 0 0 0\n");
}
