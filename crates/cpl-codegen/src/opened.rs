//! An optional the program tests or opens where it is made is not built.
//! A maker is a list's or a string's end (`xs.last`, `xs.first`) or a
//! position of a domain from an ordinal (`D.at(i)`, and so `.map` on one).
//! Its two parts are whether there is a value and the value: `xs.last!` is
//! a check and `xs.back()`, `xs.last ?? d` is `!xs.empty() ? xs.back() : d`,
//! and `if let a = xs.last` is `if (!xs.empty()) { a = xs.back(); ... }`.
//! Only optionals of a scalar: the value is a copy either way.

use crate::emit::{Code, Emitter, AND, COND, PRIMARY, UNARY};
use crate::names::dom_ident;
use cpl_ir::{walk, BinOp, Builtin, Expr, ExprKind, Program, Stmt, Ty};
use std::collections::BTreeSet;

/// An optional's two parts, as C++.
#[derive(Clone)]
pub(crate) struct Parts {
    test: Code,
    value: String,
}

fn is_local(e: &Expr, slot: usize) -> bool {
    matches!(e.kind, ExprKind::Local(s) if s == slot)
}

/// Whether `e` is a maker; `bound` when a statement may come before its
/// parts, to evaluate an operand once.
fn maker(e: &Expr, bound: bool, into: &BTreeSet<usize>) -> bool {
    let Ty::Opt(inner) = &e.ty else {
        return false;
    };
    if let ExprKind::Call(f, _) = e.kind {
        return bound && into.contains(&f);
    }
    if !inner.is_scalar() || **inner == Ty::Unit {
        return false;
    }
    match &e.kind {
        ExprKind::Builtin(Builtin::First | Builtin::Last, args) => {
            matches!(args[0].ty, Ty::List(_) | Ty::Str) && walk::path_root(&args[0]).is_some()
        }
        ExprKind::Builtin(Builtin::PosTryOrd(_), args) => bound || walk::is_simple(&args[0]),
        _ => false,
    }
}

impl Emitter<'_> {
    /// The parts of a maker (see [`maker`]); `prep` takes the statement that
    /// evaluates an operand once.
    fn opt_parts(&mut self, e: &Expr, prep: Option<&mut String>) -> Option<Parts> {
        if let (ExprKind::Call(..), Ty::Opt(inner)) = (&e.kind, &e.ty) {
            let out = prep?;
            let x = self.temp();
            let ty = self.ty(inner);
            self.line(out, &format!("{ty} {x};"));
            self.into_arg = Some(x.clone());
            let test = self.expr(e);
            return Some(Parts { test, value: x });
        }
        let ExprKind::Builtin(b, args) = &e.kind else {
            return None;
        };
        if let Builtin::PosTryOrd(d) = b {
            let len = format!("{}_len", dom_ident(self.program, *d));
            let i = self.expr(&args[0]);
            let at = match prep {
                Some(out) if !walk::is_simple(&args[0]) => {
                    let t = self.temp();
                    self.line(out, &format!("ll {t} = {};", i.text));
                    t
                }
                _ => i.at(PRIMARY),
            };
            return Some(Parts {
                test: Code::new(format!("{at} >= 0 && {at} < {len}"), AND),
                value: format!("(int){at}"),
            });
        }
        let xs = self.expr(&args[0]).at(PRIMARY);
        let end = if matches!(b, Builtin::First) {
            "front"
        } else {
            "back"
        };
        let cast = if args[0].ty == Ty::Str {
            "(cpl::u8)"
        } else {
            ""
        };
        Some(Parts {
            test: Code::new(format!("!{xs}.empty()"), UNARY),
            value: format!("{cast}{xs}.{end}()"),
        })
    }

    /// The parts of `e`: an opened local or a maker needing no statement.
    fn parts_of(&mut self, e: &Expr) -> Option<(Parts, bool)> {
        if let ExprKind::Local(s) = e.kind {
            return self.opened.get(&s).map(|p| (p.clone(), true));
        }
        if !maker(e, false, &self.into) {
            return None;
        }
        self.opt_parts(e, None).map(|p| (p, false))
    }

    /// `e` when it tests, opens or defaults a maker: its parts, no optional.
    pub(crate) fn opened_expr(&mut self, e: &Expr) -> Option<Code> {
        match &e.kind {
            ExprKind::IsSome(x) => {
                let (p, _) = self.parts_of(x)?;
                Some(p.test)
            }
            ExprKind::Unwrap(x, checked) => {
                let (p, tested) = self.parts_of(x)?;
                if tested || !*checked {
                    return Some(Code::primary(p.value));
                }
                self.need("fail");
                Some(Code::primary(format!(
                    "({} ? (void)0 : cpl::fail(\"`!` on `None`: there is no value here\"), {})",
                    p.test.at(COND + 1),
                    p.value
                )))
            }
            ExprKind::Binary(BinOp::Coalesce, l, r) if l.ty != e.ty => {
                let (p, _) = self.parts_of(l)?;
                let d = self.expr(r);
                Some(Code::new(
                    format!("{} ? {} : {}", p.test.at(COND + 1), p.value, d.at(COND)),
                    COND,
                ))
            }
            _ => None,
        }
    }

    /// `let t = maker` tested by the `if` after it (past `let`s of `None`)
    /// and read only by `let`s at the top of that `if`'s branch, where the
    /// test held and nothing ran since: emits through the `if` with `t`
    /// its parts, and gives the statement after it.
    pub(crate) fn open_at(
        &mut self,
        stmts: &[Stmt],
        k: usize,
        out: &mut String,
        each: &dyn Fn(&mut Self, usize, &mut String),
    ) -> Option<usize> {
        let Stmt::Let(t, Some(init)) = &stmts[k] else {
            return None;
        };
        if !maker(init, true, &self.into) {
            return None;
        }
        let j = tested_here(self.program, stmts, k)?;
        let parts = self.opt_parts(init, Some(out))?;
        self.opened.insert(*t, parts);
        for i in k + 1..=j {
            each(self, i, out);
        }
        self.opened.remove(t);
        Some(j + 1)
    }
}

/// Where `stmts[k]`, `let t = ..`, is tested by an `if` after it (past
/// `let`s of `None`) and read only by `let`s at the top of that `if`'s
/// branch: the index of the `if`.
pub(crate) fn tested_here(program: &Program, stmts: &[Stmt], k: usize) -> Option<usize> {
    let Stmt::Let(t, Some(_)) = &stmts[k] else {
        return None;
    };
    let nones = stmts[k + 1..]
        .iter()
        .take_while(|s| matches!(s, Stmt::Let(_, Some(e)) if matches!(e.kind, ExprKind::None)))
        .count();
    let j = k + 1 + nones;
    let Some(Stmt::If(cond, then, otherwise)) = stmts.get(j) else {
        return None;
    };
    if !matches!(&cond.kind, ExprKind::IsSome(x) if is_local(x, *t)) {
        return None;
    }
    let opens = then
        .stmts
        .iter()
        .take_while(|s| {
            matches!(s, Stmt::Let(_, Some(e))
                if matches!(&e.kind, ExprKind::Unwrap(x, _) if is_local(x, *t)))
        })
        .count();
    let unread = |rest: &[Stmt]| crate::uses::live(program, rest, *t).is_empty();
    (unread(&stmts[k + 1..j])
        && unread(&stmts[j + 1..])
        && unread(&then.stmts[opens..])
        && unread(&otherwise.stmts))
    .then_some(j)
}
