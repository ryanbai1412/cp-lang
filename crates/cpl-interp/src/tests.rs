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
