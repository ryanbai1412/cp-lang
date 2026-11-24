//! Inputs in their declared ranges for which an arithmetic expression
//! leaves `Int`, found and checked by `cpl-prover`. The playground build
//! has no prover (decision 235): it keeps which names are inputs and finds
//! no witness.

use cpl_syntax::ast::Expr;
#[cfg(not(target_arch = "wasm32"))]
use cpl_syntax::ast::{BinOp, ExprKind, UnOp};

/// A name the prover can be told about: an input of a declared range, or an
/// immutable `let` of arithmetic over such names, as prover text.
#[derive(Clone, Debug)]
enum Def {
    Input {
        #[cfg(not(target_arch = "wasm32"))]
        range: (i128, i128),
    },
    #[cfg(not(target_arch = "wasm32"))]
    Let { text: String, names: Vec<String> },
}

/// How each name the prover can reason about got its value, in scope order.
#[derive(Clone, Debug, Default)]
pub struct Overflow {
    defs: crate::names::NameMap<Def>,
}

impl Overflow {
    /// `name` was rebound to something the prover can't follow.
    pub fn forget(&mut self, name: &str) {
        self.defs.remove(name);
    }

    /// Whether `name` is an input of a declared range, as read.
    pub fn is_input(&self, name: &str) -> bool {
        matches!(self.defs.get(name), Some(Def::Input { .. }))
    }

    /// From here on `name` is an input anywhere in `lo..=hi`.
    pub fn input(&mut self, name: &str, lo: i128, hi: i128) {
        #[cfg(target_arch = "wasm32")]
        let _ = (lo, hi);
        let def = Def::Input {
            #[cfg(not(target_arch = "wasm32"))]
            range: (lo, hi),
        };
        self.defs.insert(name.to_string(), def);
    }

    /// `name` was just bound to the value of `init`, which the prover can
    /// follow when it is arithmetic over names it can follow.
    pub fn define(&mut self, name: &str, init: &Expr) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut names = Vec::new();
            if let Some(text) = prover_text(init, &mut names) {
                if names.iter().all(|n| self.defs.contains_key(n)) {
                    self.defs.insert(name.to_string(), Def::Let { text, names });
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        let _ = (name, init);
    }

    /// Inputs in their declared ranges for which `expr` itself leaves
    /// `Int`, found and checked by the prover, as `n = 55110`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn witness(&self, expr: &Expr) -> Option<String> {
        let mut names = Vec::new();
        let text = prover_text(expr, &mut names)?;
        let mut ctx = cpl_prover::Context::new();
        let mut told = std::collections::HashSet::new();
        for name in &names {
            self.tell(name, &mut ctx, &mut told)?;
        }
        let whole = cpl_prover::parse::term(&text).ok()?.to_string();
        let findings = ctx.check_overflow(&text).ok()?;
        let finding = findings.into_iter().find(|f| f.expr == whole)?;
        finding.verdict.counterexample().map(|c| c.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    pub fn witness(&self, _expr: &Expr) -> Option<String> {
        None
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn tell(
        &self,
        name: &str,
        ctx: &mut cpl_prover::Context,
        told: &mut std::collections::HashSet<String>,
    ) -> Option<()> {
        if !told.insert(name.to_string()) {
            return Some(());
        }
        match self.defs.get(name)? {
            Def::Input { range: (lo, hi) } => ctx.var(name, &format!("{lo}..={hi}")).ok(),
            Def::Let { text, names } => {
                for n in names {
                    self.tell(n, ctx, told)?;
                }
                ctx.define(name, text).ok()
            }
        }
    }
}

/// `expr` as fully parenthesized prover text and the names it reads, when
/// it is only integers, names and arithmetic.
#[cfg(not(target_arch = "wasm32"))]
fn prover_text(expr: &Expr, names: &mut Vec<String>) -> Option<String> {
    Some(match &expr.kind {
        ExprKind::Int(value) => value.to_string(),
        ExprKind::Name(name) => {
            names.push(name.clone());
            name.clone()
        }
        ExprKind::Paren(inner) => prover_text(inner, names)?,
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => format!("(-{})", prover_text(expr, names)?),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            let op = match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem | BinOp::Pow => {
                    op.text()
                }
                _ => return None,
            };
            format!(
                "({} {op} {})",
                prover_text(lhs, names)?,
                prover_text(rhs, names)?
            )
        }
        _ => return None,
    })
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::Overflow;
    use cpl_syntax::ast::{Expr, StmtKind};

    fn expr(src: &str) -> Expr {
        let file = cpl_syntax::parse_file(src);
        match file
            .items
            .into_iter()
            .flat_map(|(_, i)| i.stmts)
            .last()
            .unwrap()
            .kind
        {
            StmtKind::Expr(e) => e,
            other => panic!("not an expression: {other:?}"),
        }
    }

    #[test]
    fn a_witness_names_inputs_through_lets_and_none_when_it_fits() {
        let mut o = Overflow::default();
        o.input("n", 1, 1_000_000_000_000);
        o.define("m", &expr("n * 2\n"));
        assert!(o.is_input("n") && !o.is_input("m"));
        let w = o
            .witness(&expr("m * m * m\n"))
            .expect("n near 1e12 overflows");
        assert!(w.contains("n = "), "{w}");
        assert_eq!(o.witness(&expr("m + 1\n")), None);
        o.forget("n");
        assert_eq!(o.witness(&expr("m * m * m\n")), None);
    }
}
