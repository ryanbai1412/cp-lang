//! What the interpreter stops a program for. The C++ a judge runs has no
//! such checks, so each of these is a mistake `cpl run` must name: what
//! went wrong, and where.

use crate::{run, run_on_this_thread, Limits};

/// Runs `source` on `input`: what it printed, and the failure if it
/// ended in one.
fn outcome(source: &str, input: &str) -> (String, Option<String>) {
    let mut db = cpl_check::Database::new();
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
        format!("{} @ {at}", trap.message)
    });
    (String::from_utf8(out).expect("ASCII"), failure)
}

fn failure(source: &str, input: &str) -> String {
    outcome(source, input).1.expect("the program fails")
}

#[test]
fn a_program_that_ends_well_has_no_failure() {
    assert_eq!(
        outcome("let n: Int = read()\nprint(n * 2)\n", "21"),
        ("42\n".into(), None)
    );
}

#[test]
fn what_was_printed_before_a_failure_is_kept() {
    let (out, failure) = outcome("print(\"a\")\nassert 1 > 2\nprint(\"b\")\n", "");
    assert_eq!(out, "a\n");
    assert_eq!(
        failure.as_deref(),
        Some("assertion failed: 1 > 2 @ assert 1 > 2")
    );
}

#[test]
fn indices_outside_their_container_stop_the_program() {
    assert_eq!(
        failure("let xs: List<Int> = [1, 2]\nprint(xs[5])\n", ""),
        "index 5 is outside a list of length 2 @ xs[5]"
    );
    assert!(
        failure("let xs: List<Int> = [1, 2]\nprint(xs[0 - 1])\n", "")
            .starts_with("index -1 is outside")
    );
    assert!(failure("let s: Str = read()\nprint(s[3])\n", "abc").starts_with("index 3 is outside"));
    assert!(
        failure("var a: [3]Int = 0\nlet i: Int = read()\na[i] = 1\n", "3")
            .starts_with("index 3 is outside")
    );
    assert!(failure("var m: Map<Int, Int> = []\nprint(m[3])\n", "")
        .starts_with("the key `3` is not in the map"));
}

#[test]
fn arithmetic_that_has_no_answer_stops_the_program() {
    assert_eq!(
        failure("let a: Int = read()\nprint(10 / a)\n", "0"),
        "division by zero: `10 / 0` @ 10 / a"
    );
    assert_eq!(
        failure("let a: Int = read()\nprint(10 % a)\n", "0"),
        "division by zero: `10 % 0` @ 10 % a"
    );
    assert_eq!(
        failure("let a: Int = read()\nprint(a * a)\n", "4000000000"),
        "integer overflow: `4000000000 * 4000000000` does not fit in an `Int` (64 bits) @ a * a"
    );
    assert_eq!(
        failure("let a: Int = read()\nprint(a + a)\n", "9000000000000000000"),
        "integer overflow: `9000000000000000000 + 9000000000000000000` does not fit in an `Int` (64 bits) @ a + a"
    );
}

#[test]
fn division_and_remainder_are_euclidean() {
    let source = "let a: Int = read()\nlet b: Int = read()\nprint(a / b, a % b)\n";
    for (a, b, expected) in [
        (7, 2, "3 1"),
        (-7, 2, "-4 1"),
        (7, -2, "-3 1"),
        (-7, -2, "4 1"),
        (-6, 3, "-2 0"),
        (0, -5, "0 0"),
        // Int is symmetric (decisions.md 1.1): no quotient leaves it.
        (-i64::MAX, -1, "9223372036854775807 0"),
        (i64::MAX, -i64::MAX, "-1 0"),
        (-1, -i64::MAX, "1 9223372036854775806"),
    ] {
        assert_eq!(
            outcome(source, &format!("{a} {b}")),
            (format!("{expected}\n"), None),
            "{a} and {b}"
        );
    }
}

#[test]
fn unwrapping_none_stops_the_program() {
    assert_eq!(
        failure("var m: ?Int = None\nprint(m!)\n", ""),
        "`!` on `None`: there is no value here @ m!"
    );
}

#[test]
fn input_is_held_to_what_the_program_declares() {
    assert!(failure("let x: 1..=5 = read()\nprint(x)\n", "9")
        .starts_with("9 is outside `1..=5` (input line 1, column 1)"));
    assert!(failure("let x: Int = read()\nlet y: Int = read()\n", "1\n")
        .starts_with("the input ended where a number was expected"));
    assert!(failure("let x: Int = read()\n", "seven")
        .starts_with("expected a number, found `seven` (input line 1, column 1)"));
}

#[test]
fn recursion_without_end_is_reported_not_crashed() {
    // On a small stack, so that the test does not fill gigabytes first.
    let mut db = cpl_check::Database::new();
    db.set_file(
        "t.cp",
        "fn f(n: Int) -> Int:\n    f(n + 1) + 1\nprint(f(0))\n",
    );
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let limits = Limits {
        stack_bytes: 48 << 20,
    };
    let trap = std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || run_on_this_thread(&program, b"", &mut Vec::new(), limits))
        .expect("a 64 MB thread starts")
        .join()
        .expect("the run does not crash")
        .expect_err("the run stops");
    let message = trap.message;
    assert!(
        message.contains("`f` never stops calling itself?"),
        "{message}"
    );
}

#[test]
fn recursion_a_million_calls_deep_runs() {
    // Only the stack the calls fill is limited, never how deep they nest.
    let source = "\
let n: 1..=1e6 = read()
domain V = 0..n
var seen: [V]Bool = false
fn dfs(v: V) -> Int:
    seen[v] = true
    if let w = v.next:
        if not seen[w]: return dfs(w) + 1
    1
print(dfs(V.first!))
";
    assert_eq!(outcome(source, "1000000"), ("1000000\n".to_string(), None));
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
fn int_stops_one_above_the_64_bit_minimum() {
    let min = "print(Int.MIN, yes(Int.MIN == -Int.MAX), -Int.MIN, abs(Int.MIN))";
    assert_eq!(
        outcome(min, ""),
        (
            "-9223372036854775807 YES 9223372036854775807 9223372036854775807\n".to_string(),
            None
        )
    );
    for (source, input) in [
        ("let x: Int = read()\nprint(x)", "-9223372036854775808"),
        ("let x: Int = read()\nprint(x - 1)", "-9223372036854775807"),
        ("let x: Int = read()\nprint(x & -2)", "-9223372036854775807"),
        (
            "let x: I128 = read()\nprint(Int(x))",
            "-9223372036854775808",
        ),
        (
            "let x: Int = read()\nprint(bit_ceil(x))",
            "4611686018427387905",
        ),
    ] {
        let (_, error) = outcome(source, input);
        assert!(
            error.is_some_and(|e| e.contains("does not fit in an `Int`")),
            "{source}"
        );
    }
}

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
#[ignore = "spec/conformance/core-04/prefix-and-z-function-over-domain-array-and-range.cp"]
fn seq_and_indexed_traits_unify_sequences_and_reject_non_sequences() {
    let ok_src = "\
use std.seq.{runs, prod, is_sorted}
use std.strings.{prefix_function, z_function}

fn seq_len<S: Seq>(s: S) -> Int: s.len
fn second_elem<S: Indexed>(s: S) -> Int: s.at(1)

let s: Str = \"aabba\"
let xs = [1, 1, 2, 1, 1]
domain V = 1..=5
let arr: [V]Int = [1, 1, 2, 1, 1]
let r = 1..=5

print(seq_len(s), seq_len(xs), seq_len(arr), seq_len(r))
print(second_elem(xs), second_elem(arr), second_elem(r))
print([x.len for x in s.runs()])
print(prefix_function(s))
print(prefix_function(arr))
print(prefix_function(r))
print(z_function(arr))
print(s.first!, s.last!, r[2], sum(r), prod(r), yes(is_sorted(r)))
";
    assert_eq!(
        outcome(ok_src, ""),
        (
            "5 5 5 5\n1 1 2\n2 2 1\n0 1 0 0 1\n0 1 0 1 2\n0 0 0 0 0\n5 1 0 2 1\na a 3 15 120 YES\n"
                .to_string(),
            None
        )
    );

    for (src, want) in [
        (
            "fn seq_len<S: Seq>(s: S) -> Int: s.len\nprint(seq_len(42))\n",
            "`Int` does not implement `Seq`",
        ),
        (
            "fn at0<S: Indexed>(s: S) -> Int: s.at(0)\nprint(at0(42))\n",
            "`Int` does not implement `Indexed`",
        ),
    ] {
        let mut db = cpl_check::Database::new();
        db.set_file("t.cp", src);
        let analysis = db.analyze("t.cp").expect("the file was just set");
        let diags = cpl_ir::lower(&analysis).expect_err("bound violation must fail lowering");
        assert!(
            diags.iter().any(|d| d.message.contains(want)),
            "expected {want}, got: {diags:?}"
        );
    }
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
        suf[P(i.to_idx), P(j.to_idx)] = -falls + suf[P(i.to_idx + 1), P(j.to_idx)]

var order_sum = 0
let ord_arr = [I] i => i.to_idx
for v in ord_arr.rev:
    order_sum = order_sum * 10 + v
print(ways[I.last!].open, ways[I.last!].boxed, rect(0, n, 0, n), order_sum)
";
    let (out, trap) = outcome(source, "4\n");
    assert_eq!(trap, None);
    assert!(out.ends_with(" 3210\n"), "unexpected output: {out}");
}

#[test]
fn vm_from_label_out_of_range_traps_with_domain_info() {
    let source = "
let n = read(1..=10)
domain P = 0..=n
let bad = P(n + 1)
print(bad.to_idx)
";
    let (_, trap) = outcome(source, "3\n");
    let msg = trap.expect("expected out-of-range trap");
    assert!(
        msg.contains("4 is not in `P`, which is 0..=3"),
        "unexpected trap: {msg}"
    );
}

#[test]
fn vm_rolling_array_move_semantics() {
    let source = "
let n = read(1..=20)
domain I = 0..n
var ways: [I]Int = 0
ways[I.first!] = 1
for step in 0..n:
    var next: [I]Int = 0
    for i in I:
        next[i] = ways[i] + (if let p = i.prev: ways[p] else: 0)
    ways = next
    ways[I.first!] += 1
print(ways[I.last!])
";
    let (out, trap) = outcome(source, "5\n");
    assert_eq!(trap, None);
    assert_eq!(out, "6\n");
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
use_with_cb(mut ys, () => ys.len)
print(acc, ys.len, ys[2])
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "15 3 2\n");
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
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false);
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
fn vm_temp_v_register_drop_avoids_cow_cloning_on_global_list() {
    let source = "
var stk: List<(Int, Int)> = []
var acc = 0
for i in 0..2000:
    stk.push((i, i + 1))
    if let (a, b) = stk.last:
        acc += a + b
    stk.pop()
print(acc, stk.len)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "4000000 0\n");
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
fn vm_str_pop_and_wide_range_overflow() {
    let src = "\
var s = \"abc\"
s.pop()
s.push('z')
print(s)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "abz\n");

    let wide = "\
let lo = -6_000_000_000_000_000_000
let hi = 6_000_000_000_000_000_000
let r = lo..=hi
print(r.len)
";
    let (_, trap) = outcome(wide, "");
    let msg = trap.expect("expected wide range span overflow trap");
    assert!(msg.contains("integer overflow: range"), "got: {msg}");
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
buckets[1].sort(desc: true)
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
fn vm_map_set_ops_and_missing_key_trap() {
    let src = "\
var m = Map<Int, Int>()
m[10] = 5
m[10] += 7
m[20] -= 3
let g1 = m.get(10) ?? -1
let g2 = m.get(99) ?? -1
let has10 = 10 in m
let rem = m.remove(20)
var s = Set<Int>()
let ins1 = s.insert(4)
let ins2 = s.insert(4)
let has4 = 4 in s
let r4 = s.remove(4)
print(m[10], g1, g2, yes(has10), rem, yes(ins1), yes(ins2), yes(has4), yes(r4), yes(4 in s))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "12 12 -1 YES -3 YES NO YES YES NO\n");

    let miss = "\
var m = Map<Int, Int>()
print(m[42])
";
    let (_, trap) = outcome(miss, "");
    let msg = trap.expect("expected missing map key trap");
    assert!(msg.contains("is not in the map"), "got: {msg}");

    let ordered = "\
var s = Set<Int>()
for x in [10, 20, 30]: s.insert(x)
let ge = s.first_ge(20)!
let gt = s.first_gt(20)!
let le = s.last_le(20)!
let lt = s.last_lt(20)!
let pf = s.pop_first()!
let pl = s.pop_last()!
var m = Map<Int, Int>()
m[1] = 100
m[2] = 200
m[3] = 300
let (mk, mv) = m.first_gt(1)!
let (pk, pv) = m.pop_last()!
print(ge, gt, le, lt, pf, pl, s.len, mk, mv, pk, pv, m.len)
";
    let (out, trap) = outcome(ordered, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "20 30 20 10 10 30 1 2 200 3 300 2\n");
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
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, true);
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
fn int_literal_coerces_to_domain_position() {
    let src = "\
domain V = 1..=4
let a = [V] v => Int(v) * 2
var pre: [V.cuts]Int = 0
let v = V(2)
print(a[3], pre[0], v + 1)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "6 0 3\n");
}

#[test]
fn builtin_seq_trait_param_monomorphizes() {
    let src = "\
fn total<T>(xs: Seq<T>) -> Int:
  var s = 0
  for x in xs:
    s += x
  s

let a = [2, 3, 5]
print(total(a))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "10\n");
}

#[test]
fn record_spread_prefix_and_row_bound_unify() {
    let src = "\
type Ext<V, R> = (to: V, ..R without (u, v))
fn make<V, R: (u: V, v: V, ..)>(r: R) -> Ext<V, R>:
  (to: r.v, ..r without (u, v))

domain V = 0..3
let a = make((u: V(0), v: V(2), w: 9))
print(a.to, a.w)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "2 9\n");
}

#[test]
fn opt_compare_hints_some_array_literal() {
    let src = "\
domain X = 0..2
let a: ?[X]Bool = Some([false, true])
print(if a == Some([false, true]): 1 else: 0)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "1\n");
}

#[test]
#[ignore = "spec/conformance/core-02/generic-enum-payload-binds-its-type-argument.cp"]
fn generic_enum_qualified_variant() {
    let src = "\
enum Boxed<T> = One(T) | Empty

fn wrap<T>(x: T) -> Boxed<T>:
  Boxed.One(x)

let b = wrap(42)
match b:
  Boxed.One(k) => print(k)
  Boxed.Empty => print(-1)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "42\n");
}

#[test]
fn bare_op_field_on_generic_struct_passes_to_monoid_of() {
    let src = "\
use std.algebra.Monoid

struct Ring<A, M>(add: A, mul: M)

fn combine<R>(a: Int, b: Int, over: R = Ring(min, +)) -> (Int, Int):
  let add = Monoid<Int>.of(over.add)
  let mul = Monoid<Int>.of(over.mul)
  (add.op(a, b), mul.op(a, b))

let (x, y) = combine(3, 5)
print(x, y)
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "3 8\n");
}

const SUB: &str = "
struct Sub(bits: Int)

impl Index for Sub:
    let count = 1 << 4
    @get fn ord(self) -> Int = bits
    fn nth(k: Int) -> Sub = Sub(k)
";

#[test]
fn vm_a_walk_over_a_library_domain_matches_the_same_walk_over_ints() {
    let walk = format!(
        "{SUB}
var dp: [Sub]Int = 0
for k in 0..Sub.count:
    let s = Sub.nth(k)
    dp[s] = s.bits * s.bits % 7
for i in 0..4:
    for k in 0..Sub.count:
        let s = Sub.nth(k)
        if (s.bits >> i) & 1 == 1:
            dp[s] = (dp[s] + dp[Sub.nth(s.bits ^ (1 << i))]) % 1000000007
for s in Sub: print(dp[s])
"
    );
    let ints = "
var dp: [16]Int = 0
for k in 0..16: dp[k] = k * k % 7
for i in 0..4:
    for k in 0..16:
        if (k >> i) & 1 == 1:
            dp[k] = (dp[k] + dp[k ^ (1 << i)]) % 1000000007
for k in 0..16: print(dp[k])
";
    assert_eq!(outcome(&walk, ""), outcome(ints, ""));
}

#[test]
fn vm_a_checked_position_does_not_vouch_for_its_neighbour() {
    let source = format!(
        "{SUB}
var t = 0
for k in 0..Sub.count:
    let s = Sub.nth(k)
    let u = Sub.nth(k + 1)
    t += s.bits + u.bits
print(t)
"
    );
    let msg = failure(&source, "");
    assert!(msg.contains("has no position 16"), "unexpected trap: {msg}");
}

/// Runs `source` on the VM, which must compile its main and its own fns
/// with no fallback to the tree-walker but on the way to a trap, and on the
/// tree-walker (a profiled run), which must agree.
fn vm_runs_alone(source: &str, input: &str) -> String {
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false);
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
        include_str!("../../cpl-cli/tests/programs/vm_apply_operands.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../cpl-cli/tests/programs/vm_apply_operands.out")
    );
}

#[test]
fn vm_applies_builtins_arithmetic_and_negation_to_registers() {
    let out = vm_runs_alone(
        include_str!("../../cpl-cli/tests/programs/vm_apply_builtins.cp"),
        "",
    );
    assert_eq!(
        out,
        include_str!("../../cpl-cli/tests/programs/vm_apply_builtins.out")
    );
}
