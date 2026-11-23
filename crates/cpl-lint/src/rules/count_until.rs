//! A value stepped by `*=` or `+=` while a counter counts the steps
//! (`count_until`): the counter is the smallest step count where the loop's
//! condition fails, which `(0..).find(k => ...)!` says directly.

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, COUNT_UNTIL};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::*;

use super::shapes::{compare, int, is_step, name};

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = CountUntil {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    let top: Vec<_> = file.top_level().collect();
    v.scan(&top);
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct CountUntil<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

/// `var x = <int>`: the name and the start.
fn int_var(stmt: &Stmt) -> Option<(&str, i128)> {
    let StmtKind::Let {
        mutable: true,
        pat,
        init: Some(init),
        ..
    } = &stmt.kind
    else {
        return None;
    };
    match &pat.kind {
        PatternKind::Name(n) => Some((n, int(init)?)),
        _ => None,
    }
}

/// Whether `word` occurs in `text` as a whole identifier.
fn mentions(text: &str, word: &str) -> bool {
    spans_of(text, word).next().is_some()
}

fn spans_of<'t>(text: &'t str, word: &'t str) -> impl Iterator<Item = usize> + 't {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(word).filter_map(move |(i, _)| {
        let before = text[..i].chars().next_back().is_some_and(ident);
        let after = text[i + word.len()..].chars().next().is_some_and(ident);
        (!before && !after).then_some(i)
    })
}

fn replace_word(text: &str, word: &str, with: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    for i in spans_of(text, word).collect::<Vec<_>>() {
        out.push_str(&text[last..i]);
        out.push_str(with);
        last = i + word.len();
    }
    out.push_str(&text[last..]);
    out
}

fn negated(op: CmpOp) -> Option<&'static str> {
    Some(match op {
        CmpOp::Lt => ">=",
        CmpOp::Le => ">",
        CmpOp::Gt => "<=",
        CmpOp::Ge => "<",
        CmpOp::Eq => "!=",
        CmpOp::Ne => "==",
        _ => return None,
    })
}

impl CountUntil<'_> {
    fn text(&self, base: u32, span: Span) -> &str {
        span.shift(base).text(self.src)
    }

    /// `p op= c` for `*=` or `+=`, with `c` free of the loop's names.
    fn step<'s>(&self, base: u32, stmt: &'s Stmt, k: &str) -> Option<(&'s str, BinOp, String)> {
        let StmtKind::Assign {
            target,
            op: AssignOp::Binary(op @ (BinOp::Mul | BinOp::Add)),
            value,
        } = &stmt.kind
        else {
            return None;
        };
        let p = name(target)?;
        let c = self.text(base, value.span).to_string();
        (p != k && !mentions(&c, p) && !mentions(&c, k)).then_some((p, *op, c))
    }

    fn scan(&mut self, stmts: &[(u32, &Stmt)]) {
        for (j, &(w_base, w)) in stmts.iter().enumerate() {
            let StmtKind::While {
                cond: Cond::Expr(cond),
                body,
                else_body: None,
            } = &w.kind
            else {
                continue;
            };
            let [a, b] = body.stmts.as_slice() else {
                continue;
            };
            let (counted, step) = if counted_name(b).is_some() {
                (b, a)
            } else {
                (a, b)
            };
            let Some(k) = counted_name(counted) else {
                continue;
            };
            let Some((p, op, c)) = self.step(w_base, step, k) else {
                continue;
            };
            let starts: Vec<_> = stmts[..j].iter().filter_map(|(_, s)| int_var(s)).collect();
            let start = |n: &str| starts.iter().rev().find(|(v, _)| *v == n).map(|(_, x)| *x);
            let (Some(0), Some(p0)) = (start(k), start(p)) else {
                continue;
            };
            let cond_text = self.text(w_base, cond.span).to_string();
            if !mentions(&cond_text, p) || mentions(&cond_text, k) {
                continue;
            }
            let c_atom = if c.chars().all(|ch| ch.is_alphanumeric() || ch == '_') {
                c.clone()
            } else {
                format!("({c})")
            };
            let closed = match (op, p0) {
                (BinOp::Mul, 1) => format!("{c_atom} ** {k}"),
                (BinOp::Mul, _) => format!("{p0} * {c_atom} ** {k}"),
                (_, 0) if c == "1" => k.to_string(),
                (_, 0) => format!("{c_atom} * {k}"),
                (_, _) if c == "1" => format!("{p0} + {k}"),
                (_, _) => format!("{p0} + {c_atom} * {k}"),
            };
            let until = match compare(cond).and_then(|(l, op, r)| Some((l, negated(op)?, r))) {
                Some((l, op, r)) => {
                    let side = |e: &Expr| {
                        let t = self.text(w_base, e.span);
                        if name(e) == Some(p) {
                            closed.clone()
                        } else {
                            replace_word(t, p, &format!("({closed})"))
                        }
                    };
                    format!("{} {op} {}", side(l), side(r))
                }
                None => format!(
                    "not ({})",
                    replace_word(&cond_text, p, &format!("({closed})"))
                ),
            };
            let step_text = self.text(w_base, step.span).to_string();
            let span = Span::new(w.span.start + w_base, cond.span.end + w_base);
            let diag = Diagnostic::warning(
                COUNT_UNTIL.code,
                span,
                format!("`{k}` counts the steps of `{step_text}` until `{cond_text}` fails"),
            )
            .with_help(format!(
                "`{k}` is the smallest step count with `{until}`: write `(0..).find({k} => {until})!`; after `{k}` steps `{p}` is `{closed}`"
            ));
            self.out.push(Finding {
                lint: &COUNT_UNTIL,
                diag,
            });
        }
    }
}

/// `k += 1`: the counter's name.
fn counted_name(stmt: &Stmt) -> Option<&str> {
    let StmtKind::Assign { target, .. } = &stmt.kind else {
        return None;
    };
    let k = name(unparen(target))?;
    is_step(stmt, k).then_some(k)
}

impl Visitor for CountUntil<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let base = self.base;
        let stmts: Vec<_> = stmts.iter().map(|s| (base, s)).collect();
        self.scan(&stmts);
    }
}
