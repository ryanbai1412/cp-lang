//! Calls of the built-in functions, by arity.

use cpl_prover::Context;

#[test]
fn builtins_take_their_arguments_by_count() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    ctx.var("b", "0..=5").unwrap();
    for goal in [
        "abs(a) >= 0",
        "max(a) == a and min(a, b) <= b",
        "gcd(b) == b",
        "(Some(a) ?? 0) == a",
    ] {
        let verdict = ctx.prove(goal);
        assert!(
            verdict.as_ref().is_ok_and(|v| v.is_proved()),
            "{goal}: {verdict:?}"
        );
    }
    assert_eq!(
        ctx.prove("abs(a, b) == 0").unwrap_err().message,
        "`abs` does not take 2 arguments"
    );
}

#[test]
fn every_integer_operator_lowers_and_evaluates() {
    let mut ctx = Context::new();
    ctx.var("a", "0..=9").unwrap();
    ctx.var("b", "1..=9").unwrap();
    let all = "a + b - a * b + a / b + a % b + a ** 2 + (a << 1) + (a >> 1) + (a & b) + (a | b) + (a ^ b)";
    assert!(ctx.prove(&format!("{all} >= -100")).unwrap().is_proved());
    let refuted = ctx
        .prove(&format!(
            "{all} < 50 or a < b or a <= b or a > b + 8 or a >= 10"
        ))
        .unwrap();
    assert!(refuted.counterexample().is_some(), "{refuted:?}");
}
