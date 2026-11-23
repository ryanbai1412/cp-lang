//! Differential tests: the prover against brute force.
//!
//! Random goals over three variables in a small box are decided twice:
//! by the prover, and by evaluating the goal at every point of the box.
//! `Proved` must mean no point fails, and a counterexample must be a
//! point that fails. Nothing here is specific to a law or a plan: this
//! is the check that the solver itself can be trusted.

use cpl_prover::eval::{Env, Trap};
use cpl_prover::{parse, Context, Term, Value, Verdict};

struct Gen(u64);

impl Gen {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }

    fn pick<'a>(&mut self, options: &[&'a str]) -> &'a str {
        options[self.below(options.len() as u64) as usize]
    }

    fn int(&mut self, depth: u32, ops: &[&str]) -> String {
        if depth == 0 || self.below(4) == 0 {
            return match self.below(5) {
                0 => (self.below(7) as i64 - 3).to_string(),
                1 => "x".to_string(),
                2 => "y".to_string(),
                3 => "z".to_string(),
                _ => self.pick(&["x", "y", "z", "0", "1", "2"]).to_string(),
            };
        }
        let (a, b) = (self.int(depth - 1, ops), self.int(depth - 1, ops));
        match self.pick(ops) {
            "neg" => format!("-({a})"),
            "abs" => format!("abs({a})"),
            "min" => format!("min({a}, {b})"),
            "max" => format!("max({a}, {b})"),
            "if" => {
                let cond = self.bool(depth - 1, ops);
                format!("(if {cond}: {a} else: {b})")
            }
            "/k" => format!("({a}) / {}", self.pick(&["2", "3", "-2", "4"])),
            "%k" => format!("({a}) % {}", self.pick(&["2", "3", "-3", "5"])),
            op => format!("({a}) {op} ({b})"),
        }
    }

    fn bool(&mut self, depth: u32, ops: &[&str]) -> String {
        if depth == 0 || self.below(3) == 0 {
            let (a, b) = (self.int(depth.min(2), ops), self.int(depth.min(2), ops));
            return format!(
                "({a}) {} ({b})",
                self.pick(&["==", "!=", "<", "<=", ">", ">="])
            );
        }
        let (a, b) = (self.bool(depth - 1, ops), self.bool(depth - 1, ops));
        match self.below(4) {
            0 => format!("not ({a})"),
            1 => format!("({a}) and ({b})"),
            _ => format!("({a}) or ({b})"),
        }
    }
}

const BOX: std::ops::RangeInclusive<i128> = -3..=3;

fn context() -> Context {
    let mut ctx = Context::new();
    for name in ["x", "y", "z"] {
        ctx.var(name, "-3..=3").unwrap();
    }
    ctx
}

/// What the goal evaluates to at a point: false is a failure, a trap is not.
fn fails_at(goal: &Term, point: [i128; 3]) -> bool {
    let mut env = Env::default();
    for (name, value) in ["x", "y", "z"].iter().zip(point) {
        env.vars.insert(name.to_string(), Value::Int(value));
    }
    match env.eval(goal) {
        Ok(Value::Bool(holds)) => !holds,
        Err(Trap::DivByZero) => false,
        other => panic!("`{goal}` at {point:?} gave {other:?}"),
    }
}

/// Run `rounds` random goals; returns how many the prover left undecided.
fn differential(seed: u64, rounds: usize, depth: u32, ops: &[&str], facts: &[&str]) -> usize {
    let mut gen = Gen(seed);
    let mut undecided = 0;
    for round in 0..rounds {
        let source = gen.bool(depth, ops);
        let goal = parse::term(&source).unwrap();
        let mut ctx = context();
        for fact in facts {
            ctx.assume(fact).unwrap();
        }
        let fact_terms: Vec<Term> = facts
            .iter()
            .map(|fact| parse::term(fact).unwrap())
            .collect();
        let mut failing = None;
        for x in BOX {
            for y in BOX {
                for z in BOX {
                    let point = [x, y, z];
                    if fact_terms.iter().all(|fact| !fails_at(fact, point))
                        && fails_at(&goal, point)
                    {
                        failing.get_or_insert(point);
                    }
                }
            }
        }
        match ctx.prove_term(&goal).unwrap() {
            Verdict::Proved => {
                assert_eq!(
                    failing, None,
                    "round {round}: `{source}` was proved but fails"
                );
            }
            Verdict::Refuted(counterexample) => {
                let point = ["x", "y", "z"].map(|name| counterexample.int(name).unwrap());
                assert!(
                    point.iter().all(|v| BOX.contains(v)),
                    "round {round}: {point:?} is outside the box"
                );
                assert!(
                    fact_terms.iter().all(|fact| !fails_at(fact, point)),
                    "round {round}: a fact fails at {point:?}"
                );
                assert!(
                    fails_at(&goal, point),
                    "round {round}: `{source}` holds at {counterexample}"
                );
            }
            Verdict::Unknown(reason) => {
                undecided += 1;
                assert!(
                    failing.is_none(),
                    "round {round}: `{source}` fails at {failing:?} but: {reason}"
                );
            }
        }
    }
    undecided
}

#[test]
fn linear_goals_are_always_decided() {
    let ops = ["+", "-", "neg", "min", "max", "abs", "if"];
    assert_eq!(differential(0x1234_5678_9ABC_DEF1, 1500, 3, &ops, &[]), 0);
}

#[test]
fn division_and_remainder_by_constants_are_always_decided() {
    let ops = ["+", "-", "/k", "%k", "min", "max", "if"];
    assert_eq!(differential(0x0F1E_2D3C_4B5A_6978, 1200, 3, &ops, &[]), 0);
}

#[test]
fn facts_narrow_what_has_to_hold() {
    let ops = ["+", "-", "max", "min", "%k", "if"];
    assert_eq!(
        differential(
            0x00C0_FFEE_1234_5678,
            600,
            3,
            &ops,
            &["x < y", "y != z or x == 0"]
        ),
        0
    );
    assert_eq!(
        differential(
            0x7777_1111_2222_3333,
            600,
            3,
            &ops,
            &["x + y + z == 1", "x % 2 == 0"]
        ),
        0
    );
}

#[test]
fn products_and_variable_divisors_are_never_decided_wrongly() {
    // Nonlinear goals may stay undecided inside the solver; sampling then
    // finds every failure in a box this small, so "proved" and "fails"
    // are still exact and few goals are left open.
    let ops = ["+", "-", "*", "/", "%", "min", "abs", "if"];
    let undecided = differential(0x0BAD_5EED_0BAD_5EED, 1200, 3, &ops, &[]);
    assert!(
        undecided <= 60,
        "{undecided} of 1200 goals were left undecided"
    );
}

#[test]
fn bit_operations_are_never_decided_wrongly() {
    let ops = ["^", "&", "|", "+", "-", "max"];
    let undecided = differential(0x5151_5151_A0A0_A0A0, 800, 3, &ops, &[]);
    assert!(
        undecided <= 40,
        "{undecided} of 800 goals were left undecided"
    );
}
