//! What the interpreter stops a program for. The C++ a judge runs has no
//! such checks, so each of these is a mistake `cpl run` must name: what
//! went wrong, and where.

use crate::{run, run_on_this_thread, Limits};

mod vm_alone;

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
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
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
fn unwrapping_cells_names_the_first_gap() {
    let src = "domain V = 1..=4\nvar depth: [V]?Int = None\nfor v in V: if v != V(3): depth[v] = 1\nlet depth = depth!\nprint(depth[V(1)])\n";
    assert_eq!(failure(src, ""), "`!` on `None`: depth[3] is None @ depth!");
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
    let mut db = cpl_check::Session::new();
    db.set_file(
        "t.cp",
        "fn f(n: Int) -> Int:\n    f(n + 1) + 1\nprint(f(0))\n",
    );
    let program =
        cpl_ir::lower(&db.analyze("t.cp").expect("set"), &cpl_check::OwnFacts).expect("lowers");
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
    let message = trap.into_message();
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
fn vm_frames_hold_their_widest_statement_not_every_temporary() {
    // 600 locals and 42,000 statements: more temporaries than a register
    // index addresses, unless each statement's are freed when it ends.
    let mut source = String::from("fn wide(x: Int) -> Int:\n");
    for i in 0..600 {
        source += &format!("    let a{i} = x + {i}\n");
    }
    source += "    var s = 0\n";
    for _ in 0..70 {
        for i in 0..600 {
            source += &format!("    s += a{i} * 1\n");
        }
    }
    source += "    s\nprint(wide(read()))\n";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source.as_str());
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let wide = compiled.fns[0]
        .as_ref()
        .expect("a 600-local fn runs on the VM");
    assert!(wide.num_i < 1000, "{} int registers", wide.num_i);
    assert_eq!(outcome(&source, "3"), ("12705000\n".to_string(), None));
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
            "5 5 5 5\n1 1 2\n2 2 1\n0 1 0 0 1\n0 1 0 1 2\n0 0 0 0 0\n5 1 0 2 1\na a 2 15 120 YES\n"
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
        let mut db = cpl_check::Session::new();
        db.set_file("t.cp", src);
        let analysis = db.analyze("t.cp").expect("the file was just set");
        let diags = cpl_ir::lower(&analysis, &cpl_check::OwnFacts)
            .expect_err("bound violation must fail lowering");
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
fn vm_from_label_out_of_range_traps_with_domain_info() {
    let source = "
let n = read(1..=10)
domain P = 0..=n
let bad = P(n + 1)
print(bad.ord)
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
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
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
fn vm_keeps_a_heap_of_int_pairs_in_int_registers() {
    // dijkstra's `MinHeap<(W, V)>`: entries move between the list and
    // registers, and `less` is inlined into `push` and `sift_down`, so only
    // a `by:` heap's comparator closure takes boxed entries.
    let source = "
fn order(xs: List<(Int, Int)>) -> Int:
    var h = MinHeap<(Int, Int)>()
    for x in xs: h.push(x)
    var acc = 0
    while let (d, v) = h.pop():
        acc = acc * 10 + v + d - d
    acc
fn by_second(xs: List<(Int, Int)>) -> Int:
    var h = Heap<(Int, Int)>.new(by: p => -p.1)
    for x in xs: h.push(x)
    var acc = 0
    while let (d, v) = h.pop():
        acc = acc * 10 + d
    acc
let xs = [(3, 1), (1, 2), (2, 3), (1, 0), (5, 4), (2, 5)]
print(order(xs))
print(by_second(xs))
";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let mut seen = 0;
    for (id, f) in program.fns.iter().enumerate() {
        if !f.name.starts_with("Heap_")
            || ![".push", ".sift_down"].iter().any(|h| f.name.ends_with(h))
        {
            continue;
        }
        seen += 1;
        let chunk = compiled.fns[id].as_ref().expect("heap fns run on the VM");
        let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
        let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
        assert_eq!(count("CallVmMix"), 0, "{} calls `less`: {ops:?}", f.name);
        assert_eq!(
            count("UnboxSmallTuple"),
            0,
            "{} unboxes an entry: {ops:?}",
            f.name
        );
        assert!(count("CmpSmallTuple") > 0, "{ops:?}");
        assert_eq!(
            count("BoxSmallTuple"),
            2 * count("CallClosure"),
            "{} boxes an entry outside the comparator call: {ops:?}",
            f.name
        );
    }
    assert!(seen >= 2, "{seen}");
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "23514\n252131\n");
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
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let ops: Vec<String> = compiled
        .main
        .ops
        .iter()
        .map(|op| format!("{op:?}"))
        .collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    // Only `xs.rev` (making the view) is left to the tree-walker.
    assert_eq!(count("ApplyV"), 1, "{ops:?}");
    assert_eq!(count("CutI"), 2, "{ops:?}");
    assert_eq!(count("IsSomeV"), 3, "{ops:?}");
    assert!(
        compiled.forwards.iter().any(Option::is_some),
        "`pair` is called through a closure"
    );
    let (out, trap) = outcome(source, "");
    assert_eq!(trap, None);
    assert_eq!(out, "12345 12345 YES YES NO 9 2 34\n");
}

#[test]
fn vm_builds_wide_int_structs_from_int_registers() {
    // bench/ranges/lazy_assign's `join`: a 5-field struct of bounded Ints is
    // built straight from Int registers, each field checked against constant bounds.
    let source = "
struct Run(a: 0..=100, b: 0..=100, c: 0..=100, d: 0..=100, e: 0..=100)
fn join(x: Run, y: Run) -> Run = Run(x.a + y.a, x.b + y.b, x.c, y.d, x.e + 1)
let r = join(Run(1, 2, 3, 4, 5), Run(10, 20, 30, 40, 50))
print(r.a, r.b, r.c, r.d, r.e)
let big = join(Run(99, 0, 0, 0, 0), Run(2, 0, 0, 0, 0))
print(big.a)
";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
    let feeds = cpl_ir::pushed::Feeds::of(&program);
    let compiled = crate::vm::CompiledProgram::compile(&program, &feeds, false, false);
    let join = program.fns.iter().position(|f| f.name == "join").unwrap();
    let chunk = compiled.fns[join].as_ref().expect("join is on the VM");
    let ops: Vec<String> = chunk.ops.iter().map(|op| format!("{op:?}")).collect();
    let count = |name: &str| ops.iter().filter(|op| op.starts_with(name)).count();
    assert_eq!(count("MakeTupleI"), 1, "{ops:?}");
    assert_eq!(count("RangeCheckImmI"), 3, "{ops:?}");
    assert_eq!(count("BoxI"), 0, "{ops:?}");
    assert_eq!(count("ConstImmI"), 0, "{ops:?}");
    let (out, trap) = outcome(source, "");
    assert_eq!(out, "11 22 3 40 6\n");
    let msg = trap.expect("expected out-of-range trap");
    assert!(msg.contains("101 is outside"), "unexpected trap: {msg}");
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
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
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
let rem = m.remove(20)!
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
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
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
        let program = cpl_ir::lower(&analysis, &cpl_check::OwnFacts).expect("lowers");
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
fn a_value_is_three_words() {
    // Every list cell, register and stack slot is a Value; at 48 bytes a
    // List<Int> cost six times the C++ vector<long long>.
    assert_eq!(std::mem::size_of::<crate::value::Value>(), 24);
}

#[test]
fn packed_cells_move_like_a_vec_of_their_values() {
    use crate::value::{Cells, OptVal, Value};

    let kinds: [fn(i64) -> Value; 4] = [
        |i| Value::Bool(i % 2 == 0),
        |i| Value::Char(b'a' + i as u8),
        |i| {
            if i % 3 == 0 {
                Value::Opt(OptVal::None)
            } else {
                Value::some(Value::Int(i))
            }
        },
        |i| Value::Int(i * 7),
    ];
    for make in kinds {
        let mut want: Vec<Value> = (0..6).map(make).collect();
        let mut cells = Cells::from(want.clone());
        cells.insert(1, make(9));
        want.insert(1, make(9));
        cells.insert(cells.len(), make(10));
        want.push(make(10));
        assert_eq!(cells.remove(2), want.remove(2));
        assert_eq!(cells.remove(0), want.remove(0));
        cells.reverse();
        want.reverse();
        assert_eq!(cells.to_values(), want);
        assert!(!matches!(cells, Cells::Values(_)));
    }
}

#[test]
fn a_list_of_small_enums_is_packed() {
    use crate::value::{Cells, Value};
    use std::rc::Rc;
    let set = |i, b| Value::Enum(1, Rc::new(vec![Value::Int(i), Value::Bool(b)]));
    let mut asks = Cells::default();
    asks.push(set(3, true));
    asks.push(Value::Enum(0, Rc::new(Vec::new())));
    assert!(matches!(asks, Cells::Rows { stride: 4, .. }));
    assert_eq!(asks.get(0), Some(set(3, true)));
    asks.set(1, set(-4, false));
    assert_eq!(asks.pop(), Some(set(-4, false)));
    // A field that is not a scalar keeps every value, boxed.
    asks.push(Value::Enum(
        2,
        Rc::new(vec![Value::Str(Rc::new(b"x".to_vec()))]),
    ));
    assert!(matches!(asks, Cells::Values(_)));
    assert_eq!(asks.get(0), Some(set(3, true)));
}

#[test]
fn a_live_input_lets_go_of_what_it_read() {
    let lines: Vec<u8> = (0..100_000)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .collect();
    let total = lines.len();
    let mut chunks = lines
        .chunks(4096)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .into_iter();
    let mut input = crate::input::Input::live(Box::new(move || chunks.next()));
    for i in 0..100_000 {
        assert_eq!(input.int().ok().map(|(v, _)| v), Some(i));
    }
    assert!(input.bytes().len() < total / 4, "{}", input.bytes().len());
    let message = input.int().expect_err("ended").message;
    assert!(message.contains("input line 100001, column 1"), "{message}");
    let kept = (0..100)
        .flat_map(|i| format!("{i}\n").into_bytes())
        .collect::<Vec<_>>();
    let mut chunks = kept
        .chunks(7)
        .map(<[u8]>::to_vec)
        .collect::<Vec<_>>()
        .into_iter();
    let mut input = crate::input::Input::live(Box::new(move || chunks.next())).keeping(true);
    while input.int().is_ok() {}
    assert_eq!(input.bytes(), &kept[..]);
}

#[test]
fn a_list_of_scalars_is_packed() {
    use crate::value::{Cells, TupleVal, Value};
    // A List<Int> cell is 8 bytes and a list of (Int, Int) 16, as in the
    // C++, not a 24-byte Value (or a boxed tuple) each.
    let ints = Cells::from((0..4).map(Value::Int).collect::<Vec<_>>());
    assert!(matches!(ints, Cells::Int(_)));
    let mut pairs = Cells::default();
    for i in 0..3 {
        pairs.push(Value::tuple(vec![Value::Int(i), Value::Int(-i)]));
    }
    assert!(matches!(pairs, Cells::Rows { stride: 2, .. }));
    assert_eq!(
        pairs.pop(),
        Some(Value::tuple(vec![Value::Int(2), Value::Int(-2)]))
    );
    // A cell of another kind keeps every value, boxed.
    pairs.push(Value::Tuple(TupleVal::OneInt(7)));
    assert!(matches!(pairs, Cells::Values(_)));
    assert_eq!(pairs.len(), 3);
}

/// What `source` prints run at a judge build's `checks`, and its failure.
fn at_checks(source: &str, checks: cpl_ir::checks::Checks) -> (String, Option<String>) {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program =
        cpl_ir::lower(&db.analyze("t.cp").expect("set"), &cpl_check::OwnFacts).expect("lowers");
    let mut out = Vec::new();
    let io = crate::Io::fixed(b"").at(checks);
    let failure = crate::run_with(&program, io, &mut out)
        .err()
        .map(|t| t.into_message());
    (String::from_utf8(out).expect("ASCII"), failure)
}

#[test]
fn a_release_run_does_what_the_judge_build_does() {
    use cpl_ir::checks::Checks;
    // Opaque to the folder, as in spec/conformance/core-09's release cases.
    let source = concat!(
        "fn id(k: Int) -> Int = if k == 0: 0 else: id(k - 1) + 1\n",
        "let z = id(0)\n",
        "let big = 9223372036854775807 - id(0)\n",
        "var t = big\n",
        "t += id(1)\n",
        "print(7 / z, 7 % z, big + id(1), t)\n",
        "assert z == 1\n",
        "print(\"after\")\n",
    );
    let want = "0 0 -9223372036854775808 -9223372036854775808\nafter\n";
    assert_eq!(at_checks(source, Checks::None), (want.to_string(), None));
    let (_, traps) = at_checks(source, Checks::Traps);
    assert!(traps.is_some_and(|m| m.starts_with("division by zero")));
    let (_, all) = at_checks(source, Checks::LOCAL);
    assert!(all.is_some_and(|m| m.starts_with("integer overflow")));
    let wraps = "let big = 9223372036854775807 - id(0)\nfn id(k: Int) -> Int = k\nprint(big + id(1))\nassert id(0) == 1\n";
    assert_eq!(at_checks(wraps, Checks::Traps).0, "-9223372036854775808\n");
    let (_, asserted) = at_checks(wraps, Checks::Traps);
    assert!(asserted.is_some());
    assert!(at_checks(wraps, Checks::LOCAL)
        .1
        .is_some_and(|m| m.starts_with("integer overflow")));
}

#[test]
fn a_packed_list_sorts_packed() {
    use crate::value::{Cells, Value};
    let ints = Cells::from([3, 1, 2].map(Value::Int).to_vec());
    let up = crate::sort::sorted_packed(&ints, &cpl_ir::Ty::Int, false).unwrap();
    assert!(matches!(&up, Cells::Int(v) if v == &[1, 2, 3]));
    let down = crate::sort::sorted_packed(&ints, &cpl_ir::Ty::Int, true).unwrap();
    assert!(matches!(&down, Cells::Int(v) if v == &[3, 2, 1]));
    // A type with an order of its own sorts through it.
    let opt = cpl_ir::Ty::Opt(Box::new(cpl_ir::Ty::Int));
    assert!(crate::sort::sorted_packed(&ints, &opt, false).is_none());
}

#[test]
fn a_table_of_optional_nested_tuples_is_packed() {
    use crate::value::{Cells, OptVal, TupleVal, Value};

    let tuple = |fs: Vec<Value>| Value::Tuple(TupleVal::from_vec(fs));
    let back = |v: i64, w: i64| {
        Value::some(tuple(vec![
            Value::Int(v),
            tuple(vec![Value::Int(v + 1), Value::Bool(w > 0), Value::Int(w)]),
        ]))
    };
    let mut cells = Cells::filled(4, Value::Opt(OptVal::None));
    cells.set(2, back(5, -7));
    assert!(matches!(cells, Cells::Rows { stride: 5, .. }));
    assert_eq!(cells.get(0), Some(Value::Opt(OptVal::None)));
    assert_eq!(cells.get(2), Some(back(5, -7)));
    cells.push(back(1, 3));
    cells.sort_unstable();
    let want = vec![
        Value::Opt(OptVal::None),
        Value::Opt(OptVal::None),
        Value::Opt(OptVal::None),
        back(1, 3),
        back(5, -7),
    ];
    assert_eq!(cells.to_values(), want);
    cells.set(0, Value::some(Value::Int(1)));
    assert!(matches!(cells, Cells::Values(_)));
    assert_eq!(cells.get(4), Some(back(5, -7)));
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
fn operands_fields_and_variables_read_in_place() {
    let src = "\
let g = [4, 5, 6]
var m: Map<Int, Int> = []
m[2] = 7
let p = (3, 9)
let q = (8,)
print(g[1], p.1, q.0, -p.0, yes(2 in m), yes(5 in m), yes(p.0 < p.1))
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "5 9 8 -3 YES NO YES\n");
}

#[test]
fn one_int_tuples_options_and_formats_change_in_place() {
    let src = "\
var t = (5,)
t.0 += 2
var m: ?Int = 3
m! += 4
let x = 1.5
print(t.0, m!, \"{t.0}:{x}\")
";
    let (out, trap) = outcome(src, "");
    assert!(trap.is_none(), "unexpected trap: {trap:?}");
    assert_eq!(out, "7 7 7:1.5000000000\n");
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

#[test]
fn internal_traps_say_the_shared_row_and_user_traps_their_own() {
    use cpl_diag::msgs::interp;
    let trap = crate::bug::<()>(cpl_diag::Span::default(), "expected list").unwrap_err();
    assert_eq!(trap.message(), "internal: expected list");
    assert_eq!(trap.code(), cpl_diag::codes::RUNTIME);
    for row in interp::ALL {
        if row.text().starts_with("internal") {
            assert_eq!(row.text(), interp::INTERPRETER_BUG.text());
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut dirs = vec![src];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") && !path.ends_with("tests.rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                assert!(
                    !text.contains("\"internal:"),
                    "{}: internal text outside INTERPRETER_BUG",
                    path.display()
                );
            }
        }
    }
    let user = interp::DIV_BY_ZERO_OP.at(&[&"7", &"/"]);
    assert_eq!(user.code(), cpl_diag::codes::DIVISION_BY_ZERO);
    assert_eq!(user.as_str(), "division by zero: `7 / 0`");
}
