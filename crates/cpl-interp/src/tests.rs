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
        failure("var m: ?Int = none\nprint(m!)\n", ""),
        "`!` on `none`: there is no value here @ m!"
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
    if v.ord + 1 < n:
        let w = V.at(v.ord + 1)
        if not seen[w]: return dfs(w) + 1
    1
print(dfs(V.at(0)))
";
    assert_eq!(outcome(source, "1000000"), ("1000000\n".to_string(), None));
}

#[test]
fn vm_handles_optionals_arrays_lists_structs_and_recursion() {
    let source = "\
let n: 1..=100 = read()
domain I = 1..=n
var a: [I]Int = read()
var memo: [I]?Int = none
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
    if let nxt = i + 1:
        res = cur + solve(nxt)
    memo[i] = some(res)
    return res

print(solve(I.first))
";
    assert_eq!(
        outcome(source, "5\n10 20 30 40 50\n"),
        ("150\n".to_string(), None)
    );
}

#[test]
fn int_stops_one_above_the_64_bit_minimum() {
    let min = "print(Int.MIN, Int.MIN == -Int.MAX, -Int.MIN, abs(Int.MIN))";
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
    let next_b: ?Int = if let prev = st.b: some(prev + ch) else: some(ch)
    State(st.a + 1, next_b, st.s)

fn find_first(s: Str, target: Char, i: Int) -> ?Int:
    if i >= s.len: return none
    if s[i] == target: return i
    find_first(s, target, i + 1)

let s: Str = read()
var st = State(0, none, s)
for i in 0..s.len:
    st = step(st, i)
let idx = find_first(s, s[2], 0) ?? -1
print(st.a, st.b ?? 0, idx)
";
    assert_eq!(
        outcome(source, "abc\n"),
        ("3 294 2\n".to_string(), None)
    );
}

#[test]
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
print(s.first!, s.last!, r[2], sum(r), prod(r), is_sorted(r))
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
ways[I.first] = (9, 3, 3, 1)
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
print(ways[I.last].open, ways[I.last].boxed, rect(0, n, 0, n), order_sum)
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
    assert!(msg.contains("4 is not in `P`, which is 0..=3"), "unexpected trap: {msg}");
}

#[test]
fn vm_rolling_array_move_semantics() {
    let source = "
let n = read(1..=20)
domain I = 0..n
var ways: [I]Int = 0
ways[I.first] = 1
for step in 0..n:
    var next: [I]Int = 0
    for i in I:
        next[i] = ways[i] + (if let p = i - 1: ways[p] else: 0)
    ways = next
    ways[I.first] += 1
print(ways[I.last])
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
var opt: ?Pair = none
let p = opt ?? Pair(popped, st.last!)
opt = some(p)
let r = 2..10
print(p.a, p.b, opt!.a, r.lo, r.hi, bit_ceil(9))
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "18 14 18 2 10 16\n");
}

#[test]
fn vm_mut_ref_calls_guarded_domains_and_field_writes() {
    let source = "
struct Counter(xs: List<Int>, total: Int):
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
    return a[I.last] + c.xs[0]

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

let opt1: ?Int = none
let opt2: ?Int = some(42)
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
    a: Int
    b: Int

struct Acc:
    xs: List<Int>

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

print(run_with_ref(), v[0], v[5], dp[0].0, dp[1].1)
";
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "169 f a 5 9\n");
}
