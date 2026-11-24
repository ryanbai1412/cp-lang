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

#[test]
fn bool_and_option_flags_are_zero_or_one() {
    let mut ctx = Context::new();
    ctx.var("p", "Bool").unwrap();
    ctx.var("o", "?Int").unwrap();
    assert!(ctx.prove("p or not p").unwrap().is_proved());
    assert!(ctx.prove("o == none or o != none").unwrap().is_proved());
    assert!(!ctx.prove("p").unwrap().is_proved());
}

#[test]
fn solving_an_equation_rewrites_earlier_bounds() {
    let mut ctx = Context::new();
    ctx.var("a", "Int").unwrap();
    ctx.var("b", "Int").unwrap();
    ctx.assume("a >= 5 and a <= 9").unwrap();
    ctx.assume("a == b + 2").unwrap();
    assert!(ctx.prove("b >= 3 and b <= 7").unwrap().is_proved());
    assert!(!ctx.prove("b >= 4").unwrap().is_proved());
}
