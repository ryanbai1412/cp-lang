//! What the interpreter stops a program for. The C++ a judge runs has no
//! such checks, so each of these is a mistake `cpl run` must name: what
//! went wrong, and where.

use super::*;

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
fn a_call_of_a_cpp_body_stops_the_program() {
    assert!(failure(
        "fn pc(a: Int) -> Int = @cpp(\"return __builtin_popcountll(a);\")\nprint(pc(7))\n",
        ""
    )
    .starts_with("this fn's body is C++ (`@cpp`)"));
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

/// What `source` prints run at a judge build's `checks`, and its failure.
fn at_checks(source: &str, checks: cpl_ir::checks::Checks) -> (String, Option<String>) {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
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
    let tests = src.join("tests");
    let mut dirs = vec![src];
    while let Some(dir) = dirs.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if path != tests {
                    dirs.push(path);
                }
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
