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
        "(some(a) ?? 0) == a",
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
