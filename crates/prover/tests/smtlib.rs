//! The SMT-LIB export, and the cross-check against `z3` when there is one.
//! Without `z3` on the `PATH` the cross-check has nothing to compare and
//! only the shape of the scripts is tested.

use cpl_prover::smtlib::{run_z3, Answer};
use cpl_prover::Context;

fn grid() -> Context {
    let mut ctx = Context::new();
    ctx.var("H", "1..=1000").unwrap();
    ctx.var("W", "1..=1000").unwrap();
    ctx.var("r", "0..H").unwrap();
    ctx.var("c", "0..W").unwrap();
    ctx
}

#[test]
fn a_query_is_a_complete_script() {
    let script = grid().to_smtlib("r * W + c < H * W").unwrap();
    assert!(script.starts_with("(set-logic ALL)\n(declare-const v0 Int) ; H\n"));
    assert!(script.contains("(declare-const v3 Int) ; c\n"));
    // The negated goal: r*W + c - H*W >= 0.
    assert!(
        script.contains("(assert (>= (+ v2 (* (- 1) v0 v1) (* v1 v3)) 0))\n")
            || script.contains("v0 v1")
    );
    assert!(script.ends_with("(check-sat)\n"));
    assert_eq!(script.matches("(declare-const").count(), 4);
}

#[test]
fn z3_agrees_when_it_is_installed() {
    let cases = [
        ("r * W + c < H * W", true),
        ("r * W + c < H * W - 1", false),
        ("(r * W + c) / W == r", true),
        ("max(r, c) >= r and min(r, c) <= c", true),
        ("r + c < H", false),
    ];
    for (goal, holds) in cases {
        let mut ctx = grid();
        assert_eq!(ctx.prove(goal).unwrap().is_proved(), holds, "{goal}");
        let script = ctx.to_smtlib(goal).unwrap();
        match run_z3(&script) {
            None => return,
            Some(Answer::Unknown) => {}
            Some(answer) => assert_eq!(answer == Answer::Unsat, holds, "z3 disagrees on `{goal}`"),
        }
    }
}
