//! Whole programs run by the interpreter: loops, sequences and builtins here;
//! traps, calls, value layout and typing each in their own file below.

use crate::{run, run_on_this_thread, Limits};

mod calls;
mod layout;
mod scaling;
mod traps;
mod types;
mod vm_alone;
mod vm_ops;

/// Runs `source` on `input`: what it printed, and the failure if it
/// ended in one.
fn outcome(source: &str, input: &str) -> (String, Option<String>) {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let mut out = Vec::new();
    let failure = run(&program, input.as_bytes(), &mut out).err().map(|trap| {
        let at = &source[trap.span.start as usize..trap.span.end as usize];
        format!("{} @ {at}", trap.message())
    });
    (String::from_utf8(out).expect("ASCII"), failure)
}

fn failure(source: &str, input: &str) -> String {
    outcome(source, input).1.expect("the program fails")
}

#[test]
fn vm_handles_optionals_arrays_lists_structs_and_recursion() {
    let source = "\
let n: 1..=100 = read()
domain I = 1..=n
var a: [I]Int = read()
var memo: [I]?Int = None
var list: List<Int> = []
for i in I:
    list.push(a[i])

struct Wrap(v: Int)
fn add_wrap(x: Wrap, y: Wrap) -> Wrap:
    return Wrap(x.v + y.v)

fn solve(i: I) -> Int:
    if let ans = memo[i]:
        return ans
    let cur = add_wrap(Wrap(list[i.ord]), Wrap(0)).v
    var res = cur
    if let nxt = i.next:
        res = cur + solve(nxt)
    memo[i] = Some(res)
    return res

print(solve(I.first!))
";
    assert_eq!(
        outcome(source, "5\n10 20 30 40 50\n"),
        ("150\n".to_string(), None)
    );
}

#[test]
fn vm_small_tuple_arrays_mod_superops_and_rev_iter() {
    let source = "
type Z = Mod<1_000_000_007>
type Ways = (open: Z, walled: Z, shut: Z, boxed: Z)

let n = read(1..=100)
domain I = 0..n
domain P = 0..=n
var ways: [I]Ways = (0, 0, 0, 0)
ways[I.first!] = (9, 3, 3, 1)
var suf: [P, P]Z = 0
fn rect(x0: Int, x1: Int, y0: Int, y1: Int) -> Z =
    suf[P(x0), P(y0)] - suf[P(x1), P(y0)] - suf[P(x0), P(y1)] + suf[P(x1), P(y1)]

for i in I:
    var side: Ways = (0, 0, 0, 0)
    for j in I:
        let w = ways[j]
        let (open, walled, shut) = (w.open + side.open, w.walled + side.walled, w.shut + side.shut)
        let falls = open + walled
        ways[j] = (6 * falls, 3 * falls, 2 * falls, falls)
        let slides = open + 2 * shut
        side = (3 * slides, slides, 3 * shut, shut)
        suf[P(i.ord), P(j.ord)] = -falls + suf[P(i.ord + 1), P(j.ord)]

var order_sum = 0
let ord_arr = [I] i => i.ord
for v in ord_arr.rev:
    order_sum = order_sum * 10 + v
print(ways[I.last!].open, ways[I.last!].boxed, rect(0, n, 0, n), order_sum)
";
    let (out, trap) = outcome(source, "4\n");
    assert_eq!(trap, None);
    assert!(out.ends_with(" 3210\n"), "unexpected output: {out}");
}

#[test]
fn vm_optionals_builtins_assert_and_list_mutate() {
    let source = "
struct Pair(a: Int, b: Int)
var st: List<Int> = []
for i in 0..10:
    st.push(i * 2)
    assert(st.len == i + 1)
assert(st.first! == 0)
assert(st.last! == 18)
let popped = st.pop()!
st.pop()
var opt: ?Pair = None
let p = opt ?? Pair(popped, st.last!)
opt = Some(p)
let r = 2..10
print(p.a, p.b, opt!.a, r.start, r.end, bit_ceil(9))
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "18 14 18 2 10 16\n");
}

#[test]
fn vm_general_seq_loops_general_index_and_scalar_coalesce() {
    let source = "
var adj: [0..3]List<(Int, Int)> = []
adj[0].push((1, 10))
adj[0].push((2, 20))
adj[1].push((2, 30))

var sum = 0
for u in 0..3:
    for (v, w) in adj[u]:
        sum += u * 100 + v * 10 + w

let s = \"abcd\"
var ch_sum = 0
for c in s.rev:
    if c == 'a': ch_sum += 1
    if c == 'd': ch_sum += 4

let opt1: ?Int = None
let opt2: ?Int = Some(42)
let c1 = opt1 ?? 7
let c2 = opt2 ?? 7
print(sum, ch_sum, c1, c2)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "210 5 7 42\n");
}

#[test]
fn vm_compound_list_field_and_indexed_struct_field_assign() {
    let source = "
struct Node:
    var a: Int
    var b: Int

struct Acc:
    var xs: List<Int>

fn bump(mut acc: Acc, i: Int, d: Int):
    acc.xs[i] += d

var xs: List<Int> = [1, 2, 3]
var nodes: [0..3]Node = Node(a: 0, b: 1)
for i in 0..3:
    xs[i] += i * 10
    nodes[i].a += i + 5
    nodes[i].b = nodes[i].a * 2

var acc = Acc(xs: [10, 20])
bump(mut acc, 1, 7)
print(xs[2], nodes[2].a, nodes[2].b, acc.xs[1])
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "23 7 14 27\n");
}

#[test]
fn vm_bit_builtins_small_tuple_minmax_and_arr_list_push() {
    let source = "
var dp: [0..4](Int, Int) = (0, 0)
for i in 0..4:
    dp[i] max= (i, i * 10)
    dp[i] max= (i, i * 5)
    dp[i] min= (i, i * 7)

var adj: [0..3]List<Int> = []
for i in 0..3:
    adj[i].push(i + 1)
    adj[i].push(i + 2)

let x = 24
print(dp[3].0, dp[3].1, adj[2].len, popcount(x), ctz(x))
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "3 21 2 2 3\n");
}

#[test]
fn vm_opt_list_index_range_field_compound_and_field_pop() {
    let source = "
struct Boxed:
    var xs: List<?Int>
    var total: Int

fn step(mut b: Boxed, r: Range<Int>) -> ?Int:
    b.total += r.len
    b.total -= 1
    let last = b.xs.pop()
    let first = b.xs[0]
    first ?? last

var b = Boxed(xs: [None, 42, 99], total: 10)
let a = step(mut b, 2..7)
print(b.total, a!)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "14 99\n");
}

#[test]
fn vm_do_blocks_give_and_opt_i_comparisons() {
    let source = "
fn best_opt(a: ?Int, b: ?Int) -> ?Int:
    if a == None: b
    elif b == None: a
    elif a! < b!: b
    else: a

let x: ?Int = 3
let y: ?Int = 7
let z: ?Int = None
let r1 = best_opt(x, y)
let r2 = best_opt(z, x)
let r3 = if let v = r1: v * 10 else: -1
print(yes(r1 == y), yes(r2 == x), yes(z != x), r3)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "YES YES YES 70\n");
}

#[test]
fn vm_arr2_v_float_ops_and_if_v() {
    let src = "\
var dp: [0..2, 0..3]F64 = 0.0
dp[0, 0] = 1.0
for i in 0..2:
    for j in 0..3:
        if i == 0 and j == 0: continue
        let up = if i > 0: dp[i - 1, j] else: 0.0
        let left = if j > 0: dp[i, j - 1] else: 0.0
        dp[i, j] = (up + left) * 0.5 + max(up, left) / 2.0
print(round(dp[1, 2] * 100.0))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "200\n");
}

#[test]
fn vm_list_literals_filled_to_list_and_structural_cmp_v() {
    let source = "
var xs: List<Int> = []
let ys = [1, 2, 3]
let zs = List.filled(3, 5)
var arr: [0..3]Int = 7
let ls = ys.rev.to_list()
let p1 = (1, 2)
let p2 = (1, 3)
print(xs.len, ys[2], zs[1], ls[0], yes(p1 < p2), yes(p1 == p1), yes(ys < zs))
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "0 3 5 3 YES YES YES\n");
}

#[test]
fn vm_yields_infinite_generator_stays_lazy() {
    let src = "\
fn nat() -> yields Int:
    var i = 0
    while true:
        yield i
        i += 1

var sum = 0
for x in nat():
    if x > 4: break
    sum += x
print(sum)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "10\n");
}

#[test]
fn vm_fed_loop_inside_a_do_gives_after_the_loop() {
    // `.filter(..).collect()` lowers to a `do` whose loop `upto` feeds.
    let src = "\
fn upto(n: Int) -> yields Int:
    for i in 0..n: yield i

fn evens(n: Int) -> List<Int> = upto(n).filter(q => q % 2 == 0).collect()
print(evens(7))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "0 2 4 6\n");
}

#[test]
fn vm_fed_loop_inside_a_closure_stays_lazy() {
    let src = "\
fn nat() -> yields Int:
    var i = 0
    while true:
        yield i
        i += 1

let below = (n: Int) => do:
    var s = 0
    for x in nat():
        if x >= n: break
        s += x
    s
print(below(5), below(3))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "10 3\n");
}

#[test]
fn vm_one_int_struct_custom_ord_cmp() {
    let src = "\
use std.heap.{MaxHeap}

let xs = [3, 1, 4, 1, 5, 9, 2, 6]
var h = MaxHeap<Int>.of(xs)
var out: List<Int> = []
while let x = h.pop(): out.push(x)
print(out)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "9 6 5 4 3 2 1 1\n");
}

#[test]
fn vm_u64_ops_nested_place_path_and_sort() {
    let src = "\
struct Node:
    var cnt: Int
    var kid: [0..2]?Int
var nodes = [Node(1, [0..2] _ => None), Node(2, [0..2] _ => None)]
nodes[0].kid[1] = 1
nodes[0].cnt += 5
var buckets: [0..2]List<Int> = []
buckets[1].push(3)
buckets[1].push(1)
buckets[1].push(4)
buckets[1].sort(by: x => desc(x))
buckets[1].pop()
var u: U64 = U64(5)
u = u * U64(7) + U64(1)
u -= U64(6)
let big = U64(-1)
print(nodes[0].cnt, nodes[0].kid[1]!, buckets[1], Int(u), yes(big > U64(10)))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "6 1 4 3 30 YES\n");

    let trap_src = "\
let big = U64(-1)
print(Int(big))
";
    let (_, trap) = outcome(trap_src, "");
    let msg = trap.expect("expected U64ToInt overflow trap");
    assert!(msg.contains("does not fit in an `Int`"), "got: {msg}");
}

#[test]
fn vm_opt_minmax_nested_arr_list_tuple_and_idx_field_idx() {
    let src = "\
struct Node(kid: [0..2]?Int)
var dp: [4][3]?Int = None
dp[0][0] = 5
let transitions = [(0, 1, 7), (0, 1, 3), (1, 2, 4)]
for step in 0..3:
    for (u, v, w) in transitions:
        dp[step + 1][v] min= dp[step][u] + w
var best: [3, 3]?Int = None
best[0, 0] = 1
best[0, 1] max= best[0, 0] + 9
best[0, 1] max= best[0, 0] + 2
var nodes = List<Node>()
nodes.push(Node([0..2] b => if b == 1: 42 else: None))
let k0 = nodes[0].kid[0] ?? -1
let k1 = nodes[0].kid[1] ?? -1
print(dp[1][1]!, dp[2][2]!, best[0, 1]!, k0, k1)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "8 12 10 -1 42\n");
}

/// A traced run keeps writes to scalars and array cells on the VM: each
/// is followed by a `Trace` instead of falling back to the tree-walker.
#[test]
fn traced_vm_runs_writes_itself() {
    let source = "\
var x = 0
var a: [0..1000]Int = 0
for i in 0..100:
    x += i % 7
    a[i % 1000] += 1
print(x, a[0])
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
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, true, false);
    let in_loop =
        |op: &crate::vm::Op| matches!(op, crate::vm::Op::FallbackStmt { in_loop: true, .. });
    assert!(
        !compiled.main.ops.iter().any(in_loop),
        "a traced write in the loop fell back"
    );
    let traces = compiled
        .main
        .ops
        .iter()
        .filter(|op| matches!(op, crate::vm::Op::Trace { .. }))
        .count();
    assert!(
        traces >= 3,
        "the loop variable and both writes are reported: {traces}"
    );
}

#[test]
fn traced_vm_runs_writes_through_calls_itself() {
    let source = "\
var hits: [0..4]Int = 0
var rows: List<List<Int>> = [[], [], [], []]
var stacks: List<List<Int>> = [[5, 3], [4]]
fn pick(i: Int) -> Int: i % 4
fn evens(k: Int) -> yields Int:
    for x in 0..k:
        if x % 2 == 0: yield x
fn chained(n: Int) -> yields Int:
    for x in evens(stacks[pick(n)].pop()!):
        yield x + n
for i in 0..6:
    hits[pick(i)] += i
    rows[pick(i)].push(i)
for y in chained(0): print(y)
print(hits, rows)
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
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, true, false);
    let chunks = std::iter::once(&compiled.main).chain(compiled.fns.iter().flatten());
    for chunk in chunks {
        assert!(
            !(chunk.ops.iter()).any(|op| matches!(op, crate::vm::Op::FallbackStmt { .. })),
            "a traced statement fell back"
        );
    }
}

#[test]
fn slices_walk_print_and_join_in_order() {
    let prog = "let a = [1, 2, 3, 4]\nlet s = a[1..]\nfor x in s: print(x)\nprint(s)\nprint(a + s)\nlet t = \"abcd\"\nlet u = t[1..]\nfor c in u: print(c)\nprint(u)\nprint(t + u)\nprint(yes(u < t))\n";
    let want = "2\n3\n4\n2 3 4\n1 2 3 4 2 3 4\nb\nc\nd\nbcd\nabcdbcd\nNO\n";
    assert_eq!(outcome(prog, ""), (want.into(), None));
}

#[test]
fn vm_nested_loop_and_do_exits_land_on_their_own_targets() {
    let src = "\
fn walk(n: Int) -> Int:
    var s = 0
    'outer: for a in 0..n:
        for b in 0..n:
            if b == a: continue 'outer
            if a + b > 6: break 'outer
            if b == 2: continue
            s += a * 10 + b
    return s

fn pick(n: Int) -> Int:
    let x = do:
        var t = 0
        for i in 0..n:
            if i == 3: break
            t += i
        t
    let name = do:
        var w = \"\"
        for i in 0..n:
            if i % 2 == 0: continue
            w = w + \"o\"
        w
    return x * 100 + name.len

print(walk(5), pick(6), pick(2))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "193 303 101\n");
}

#[test]
fn vm_for_walks_each_kind_of_sequence() {
    let src = "\
let g = [5, 6, 7]
fn walk(xs: List<Int>, s: Str) -> Int:
    var t = 0
    for x in g: t += x
    for x in xs: t += x * 10
    for c in s: t += c.ord
    var seen: Set<Int> = []
    seen.insert(3)
    seen.insert(1)
    for x in seen: t += x * 1000
    return t

print(walk([1, 2], \"ab\"))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "4243\n");
}

#[test]
fn vm_feeds_values_and_writes_nested_places() {
    let src = "\
fn words(mut n: Int) -> yields Str:
    while n > 0:
        n -= 1
        yield \"w\"
struct P:
    var xs: List<Int>
    var best: ?Int
var ps = [P(xs: [1, 2], best: None), P(xs: [3], best: Some(4))]
ps[0].xs[1] = 7
ps[1].xs[0] += 5
ps[0].best = Some(2)
var k = 3
var s = \"\"
for w in words(mut k):
    s += w
let add = (a: Int) => a + ps[0].xs[1]
print(ps[0].xs, ps[1].xs, ps[0].best!, s, k, add(1))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "1 7 8 2 www 0 8\n");
}
