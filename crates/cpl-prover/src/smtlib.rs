//! SMT-LIB 2 export, to cross-check a verdict with an external solver.
//!
//! Nothing in the prover needs an external solver. When `z3` happens to
//! be on the `PATH`, [`run_z3`] feeds it a query and a test compares the
//! answers; without it the test passes trivially.

use std::io::Write;
use std::process::{Command, Stdio};

use crate::core::Formula;
use crate::poly::Poly;

fn poly(p: &Poly) -> String {
    let literal = |c: i128| {
        if c < 0 {
            format!("(- {})", c.unsigned_abs())
        } else {
            c.to_string()
        }
    };
    let terms: Vec<String> = p
        .terms()
        .map(|(m, c)| {
            if m.is_unit() {
                return literal(c);
            }
            let vars: Vec<String> = m.0.iter().map(|v| format!("v{v}")).collect();
            match (c, vars.len()) {
                (1, 1) => vars[0].clone(),
                (1, _) => format!("(* {})", vars.join(" ")),
                _ => format!("(* {} {})", literal(c), vars.join(" ")),
            }
        })
        .collect();
    match terms.len() {
        0 => "0".to_string(),
        1 => terms[0].clone(),
        _ => format!("(+ {})", terms.join(" ")),
    }
}

fn formula(f: &Formula) -> String {
    let join = |parts: &[Formula]| parts.iter().map(formula).collect::<Vec<_>>().join(" ");
    match f {
        Formula::True => "true".to_string(),
        Formula::False => "false".to_string(),
        Formula::Ge(p) => format!("(>= {} 0)", poly(p)),
        Formula::Eq(p) => format!("(= {} 0)", poly(p)),
        Formula::And(parts) => format!("(and {})", join(parts)),
        Formula::Or(parts) => format!("(or {})", join(parts)),
    }
}

/// A script that is unsatisfiable exactly when the assertions are.
pub(crate) fn script(names: &[String], asserts: &[Formula]) -> String {
    let mut out = String::from("(set-logic ALL)\n");
    for (i, name) in names.iter().enumerate() {
        out.push_str(&format!("(declare-const v{i} Int) ; {name}\n"));
    }
    for assert in asserts {
        out.push_str(&format!("(assert {})\n", formula(assert)));
    }
    out.push_str("(check-sat)\n");
    out
}

/// What an external solver said about a script.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    Sat,
    Unsat,
    Unknown,
}

/// Run `z3` on a script. `None` when there is no `z3` to run.
pub fn run_z3(script: &str) -> Option<Answer> {
    let mut child = Command::new("z3")
        .args(["-in", "-T:10"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(script.as_bytes()).ok()?;
    let output = child.wait_with_output().ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    Some(match text.lines().next().map(str::trim) {
        Some("sat") => Answer::Sat,
        Some("unsat") => Answer::Unsat,
        _ => Answer::Unknown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poly::Mono;

    #[test]
    fn scripts_are_well_formed() {
        let p = Poly::from_terms([
            (Mono(vec![0, 1]), -2),
            (Mono(vec![0]), 1),
            (Mono(vec![]), -3),
        ])
        .unwrap();
        let names = vec!["i".to_string(), "n".to_string()];
        let text = script(
            &names,
            &[Formula::or([Formula::Ge(p.clone()), Formula::Eq(p)])],
        );
        assert_eq!(
            text,
            "(set-logic ALL)\n(declare-const v0 Int) ; i\n(declare-const v1 Int) ; n\n\
             (assert (or (>= (+ (- 3) v0 (* (- 2) v0 v1)) 0) (= (+ (- 3) v0 (* (- 2) v0 v1)) 0)))\n(check-sat)\n"
        );
    }
}
