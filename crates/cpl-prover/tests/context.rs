//! A context's declarations, assignments and sampled inputs.

use cpl_prover::Context;

#[test]
fn havoc_forgets_the_current_version_only() {
    let mut ctx = Context::new();
    ctx.var("n", "1..=10").unwrap();
    ctx.var("j", "0..n").unwrap();
    ctx.assign("j", "j + 1").unwrap();
    assert!(ctx.prove("j <= n").unwrap().is_proved());
    ctx.havoc("j").unwrap();
    assert!(!ctx.prove("j <= n").unwrap().is_proved());
    ctx.havoc("n").unwrap();
    assert!(ctx.prove("n >= 1").unwrap().is_proved());
    assert_eq!(ctx.havoc("k").unwrap_err().message, "`k` is not declared");
}

#[test]
fn undecided_goals_fall_back_to_sampled_inputs() {
    let mut ctx = Context::new();
    ctx.var("a", "0..=20").unwrap();
    ctx.var("b", "0..=20").unwrap();
    let verdict = ctx.prove("a * a * a != b * b + 2 * b + 1 + a").unwrap();
    assert!(
        verdict
            .to_string()
            .contains("400 sampled inputs found no failure"),
        "{verdict:?}"
    );
}
