//! C++ spellings that CP Lang names: a bit test (`manual_bit_test`) and the last
//! element by `len - 1` (`last_via_len`).

use crate::facts::Facts;
use crate::is_std;
use crate::std_fn;
use crate::walk::{unparen, Visitor};
use crate::{File, Finding, Lint, LAST_VIA_LEN, MANUAL_BIT_TEST};
use cpl_diag::msgs::{lint_rules, Msg};
use cpl_diag::{Fix, Span};
use cpl_syntax::ast::*;
use std::fmt::Display;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = Spelled {
        src: file.src,
        base: 0,
        facts: file.facts,
        targets: Vec::new(),
        fns: Vec::new(),
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct Spelled<'a> {
    src: &'a str,
    base: u32,
    facts: Option<&'a Facts>,
    /// Places written by an assignment, which `.last` cannot replace.
    targets: Vec<Span>,
    /// Every function's name and body: a fix inside a function must not call it.
    fns: Vec<(String, Span)>,
    out: Vec<Finding>,
}

impl Spelled<'_> {
    fn text(&self, e: &Expr) -> &str {
        e.span.shift(self.base).text(self.src)
    }

    /// `label` is the place whose type `typed.rs` checks.
    fn push(
        &mut self,
        lint: &'static Lint,
        span: Span,
        (msg, args, help): (&'static Msg, &[&dyn Display], String),
        label: Option<(Span, &str)>,
        fix: Option<(Span, String)>,
    ) {
        let at = span.shift(self.base);
        if let Some((_, with)) = &fix {
            // Inside `fn bit` the fix `x.bit(i)` would call itself.
            if self.fns.iter().any(|(name, body)| {
                body.start <= at.start && at.end <= body.end && with.contains(&format!("{name}("))
            }) {
                return;
            }
        }
        let mut diag = msg.warning(at, args).with_help(help);
        if let Some((at, what)) = label {
            diag = diag.with_label(at.shift(self.base), what);
        }
        if let Some((at, with)) = fix {
            diag.fixes.push(Fix {
                title: format!("use `{with}`"),
                edits: vec![(at.shift(self.base), with)],
            });
        }
        self.out.push(Finding { lint, diag });
    }

    /// `x` as a method receiver: bare when it is a place or a call.
    fn receiver(&self, e: &Expr) -> String {
        let text = self.text(e);
        match e.kind {
            ExprKind::Name(_)
            | ExprKind::Field { .. }
            | ExprKind::Index { .. }
            | ExprKind::Call { .. }
            | ExprKind::Paren(_)
            | ExprKind::SelfValue => text.to_string(),
            _ => format!("({text})"),
        }
    }

    fn bit_test(&mut self, expr: &Expr) {
        let ExprKind::Compare { first, rest } = &expr.kind else {
            return;
        };
        let [(op @ (CmpOp::Eq | CmpOp::Ne), rhs)] = rest.as_slice() else {
            return;
        };
        let ExprKind::Int(want) = rhs.kind else {
            return;
        };
        let ExprKind::Binary {
            op: BinOp::BitAnd,
            lhs,
            rhs: mask,
            ..
        } = &unparen(first).kind
        else {
            return;
        };
        let is_one = |e: &Expr| matches!(unparen(e).kind, ExprKind::Int(1));
        let by_shift = if is_one(mask) {
            shifted(lhs)
        } else if is_one(lhs) {
            shifted(mask)
        } else {
            None
        };
        let by_mask = one_shl(mask)
            .map(|k| (&**lhs, k))
            .or_else(|| one_shl(lhs).map(|k| (&**mask, k)));
        let (x, k, set) = match (by_shift, by_mask, want) {
            (Some((x, k)), _, 1) => (x, k, *op == CmpOp::Eq),
            (Some((x, k)), _, 0) | (None, Some((x, k)), 0) => (x, k, *op == CmpOp::Ne),
            _ => return,
        };
        let call = format!("{}.bit({})", self.receiver(unparen(x)), self.text(k));
        let fixed = if set { call } else { format!("not {call}") };
        self.push(
            &MANUAL_BIT_TEST,
            expr.span,
            (
                &lint_rules::MANUAL_BIT_TEST,
                &[&self.text(expr).to_string()],
                format!("write `{fixed}`; `bit` reads bit `i` of an `Int`"),
            ),
            Some((unparen(x).span, "the tested value")),
            Some((expr.span, fixed)),
        );
    }
}

fn shifted(e: &Expr) -> Option<(&Expr, &Expr)> {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Shr,
            lhs,
            rhs,
            ..
        } => Some((lhs, rhs)),
        _ => None,
    }
}

/// `1 << k`, giving `k`.
fn one_shl(e: &Expr) -> Option<&Expr> {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::Shl,
            lhs,
            rhs,
            ..
        } if matches!(lhs.kind, ExprKind::Int(1)) => Some(rhs),
        _ => None,
    }
}

impl Spelled<'_> {
    fn last_via_len(&mut self, expr: &Expr) {
        let ExprKind::Index { base, indices } = &expr.kind else {
            return;
        };
        let [i] = indices.as_slice() else {
            return;
        };
        let ExprKind::Binary {
            op: BinOp::Sub,
            lhs,
            rhs,
            ..
        } = &unparen(i).kind
        else {
            return;
        };
        let len = unparen(lhs);
        let ExprKind::Field { base: of, .. } = &len.kind else {
            return;
        };
        if !is_std(self.facts, len.span.shift(self.base), std_fn::LEN)
            || !matches!(rhs.kind, ExprKind::Int(1))
            || self.text(of) != self.text(base)
            || self.targets.contains(&expr.span.shift(self.base))
        {
            return;
        }
        let fixed = format!("{}.last!", self.receiver(base));
        self.push(
            &LAST_VIA_LEN,
            expr.span,
            (
&lint_rules::LAST_VIA_LEN, &[&self.text(expr).to_string()],
                format!("write `{fixed}`; `.last` is the last element, and `.last(k)` the one `k` before it"),
            ),
            Some((base.span, "the list")),
            Some((expr.span, fixed)),
        );
    }
}

impl Visitor for Spelled<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            self.targets.push(target.span.shift(self.base));
        }
        if let StmtKind::Fn(decl) = &stmt.kind {
            self.fns
                .push((decl.name.text.clone(), stmt.span.shift(self.base)));
        }
    }

    fn expr(&mut self, expr: &Expr) {
        self.bit_test(expr);
        self.last_via_len(expr);
    }
}
