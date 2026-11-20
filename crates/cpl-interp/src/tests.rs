//! What the interpreter stops a program for. The C++ a judge runs has no
//! such checks, so each of these is a mistake `cpl run` must name: what
//! went wrong, and where.

use crate::run;

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
        "division by zero @ 10 / a"
    );
    assert_eq!(
        failure("let a: Int = read()\nprint(10 % a)\n", "0"),
        "division by zero @ 10 % a"
    );
    let overflow = "integer overflow: the result does not fit in an `Int` (64 bits)";
    assert_eq!(
        failure("let a: Int = read()\nprint(a * a)\n", "4000000000"),
        format!("{overflow} @ a * a")
    );
    assert_eq!(
        failure("let a: Int = read()\nprint(a + a)\n", "9000000000000000000"),
        format!("{overflow} @ a + a")
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
    let message = failure("fn f(n: Int) -> Int:\n    f(n + 1) + 1\nprint(f(0))\n", "");
    assert!(
        message.contains("`f` never stops calling itself?"),
        "{message}"
    );
}
