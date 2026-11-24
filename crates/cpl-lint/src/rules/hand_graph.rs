//! A monotone deque for each window's minimum or maximum, written out by
//! hand (`sliding_min_by_hand`).

use crate::std_fn;
use crate::walk::{unparen, Visitor};
use crate::{File, Finding, SLIDING_MIN_BY_HAND};
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::{compare, name, squash};
use super::Warn;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandGraph {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandGraph<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

/// `xs.method(args)` as a statement: the receiver, the method, the args.
fn method_call<'e>(stmt: &'e Stmt, method: &str) -> Option<(&'e str, &'e [Arg])> {
    let StmtKind::Expr(e) = &stmt.kind else {
        return None;
    };
    let ExprKind::Call { callee, args } = &e.kind else {
        return None;
    };
    match &callee.kind {
        ExprKind::Field { base, name: m } if m.text == method => Some((name(base)?, args)),
        _ => None,
    }
}

fn conjuncts<'e>(e: &'e Expr, out: &mut Vec<&'e Expr>) {
    match &unparen(e).kind {
        ExprKind::Binary {
            op: BinOp::And,
            lhs,
            rhs,
            ..
        } => {
            conjuncts(lhs, out);
            conjuncts(rhs, out);
        }
        _ => out.push(e),
    }
}

/// The queue a `while` pops, the test it pops under, and whether that test is
/// negated: `while t: q.pop()`, or `while ..:` with `if t: break` then `q.pop()`.
fn pop_test(s: &Stmt) -> Option<(&str, &Expr, bool)> {
    let StmtKind::While {
        cond: Cond::Expr(cond),
        body,
        ..
    } = &s.kind
    else {
        return None;
    };
    match body.stmts.as_slice() {
        [pop] => Some((method_call(pop, std_fn::POP)?.0, cond, false)),
        [stop, pop] => {
            let StmtKind::Expr(e) = &stop.kind else {
                return None;
            };
            let ExprKind::If {
                branches,
                otherwise: None,
            } = &e.kind
            else {
                return None;
            };
            let [(Cond::Expr(test), then)] = branches.as_slice() else {
                return None;
            };
            matches!(then.stmts.as_slice(), [b] if matches!(b.kind, StmtKind::Break(None)))
                .then_some(())?;
            Some((method_call(pop, std_fn::POP)?.0, test, true))
        }
        _ => None,
    }
}

/// The starts of `word` in `text` as a whole identifier.
fn starts_of<'t>(text: &'t str, word: &'t str) -> impl Iterator<Item = usize> + 't {
    text.match_indices(word).map(|(i, _)| i).filter(move |&i| {
        !text[..i]
            .chars()
            .next_back()
            .is_some_and(super::shapes::ident_char)
    })
}

/// Whether `text` has `word` as a whole identifier followed by `next`.
fn has_followed(text: &str, word: &str, next: &str) -> bool {
    starts_of(text, word).any(|i| text[i + word.len()..].trim_start().starts_with(next))
}

impl super::Warn for HandGraph<'_> {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32) {
        (&mut self.out, self.base)
    }
}

impl HandGraph<'_> {
    fn text(&self, span: Span) -> &str {
        span.shift(self.base).text(self.src)
    }

    /// `for i ..: while .. q[q.len - 1] >= a[i]: q.pop()` (or `if a[q.last!] < a[i]: break`
    /// then `q.pop()`) then `q.push(i)`, with the window's start (`i - k`) tested in the
    /// same loop.
    fn deque(&mut self, stmt: &Stmt) {
        let StmtKind::For { pat, body, .. } = &stmt.kind else {
            return;
        };
        let PatternKind::Name(i) = &pat.kind else {
            return;
        };
        let Some((at, (q, test, negated))) =
            (body.stmts.iter().enumerate()).find_map(|(at, s)| Some((at, pop_test(s)?)))
        else {
            return;
        };
        let pushes_i = body
            .stmts
            .get(at + 1)
            .and_then(|s| method_call(s, std_fn::PUSH))
            .is_some_and(|(r, args)| {
                r == q && matches!(args, [a] if name(&a.value) == Some(i.as_str()))
            });
        if !pushes_i {
            return;
        }
        // The test compares the back (or a key of it) with the item pushed (or its key):
        // a turn over two entries, `cross(h.last! - h[h.len - 2], ..)`, is no window's.
        let back = |e: &Expr| {
            let t = squash(self.text(e.span));
            let rest = t
                .replace(&format!("{q}[{q}.len-1]"), "")
                .replace(&format!("{q}.last"), "");
            rest.len() < t.len() && starts_of(&rest, q).next().is_none()
        };
        let item = |e: &Expr| {
            let t = squash(self.text(e.span));
            starts_of(&t, i).next().is_some() && starts_of(&t, q).next().is_none()
        };
        let mut parts = Vec::new();
        match negated {
            false => conjuncts(test, &mut parts),
            true => parts.push(test),
        }
        let Some(keeps_min) = parts.iter().find_map(|c| {
            let (l, op, r) = compare(c)?;
            let drops_larger = match op {
                CmpOp::Ge | CmpOp::Gt => true,
                CmpOp::Le | CmpOp::Lt => false,
                _ => return None,
            };
            if back(l) && item(r) {
                Some(drops_larger != negated)
            } else if back(r) && item(l) {
                Some(drops_larger == negated)
            } else {
                None
            }
        }) else {
            return;
        };
        if !has_followed(self.text(body.span), i, "-") {
            return;
        }
        let m = if keeps_min { std_fn::MIN } else { std_fn::MAX };
        self.warn_at(
            &SLIDING_MIN_BY_HAND,
            stmt.span,
&lint_rules::SLIDING_MIN_BY_HAND, &[&q, &m],
            format!("`var w = Window({m})` (`std.window`) keeps that deque: `w.push(x)` as an item enters, `w.pop()` as the oldest leaves, and `w.all!` is the window's {m}"),
        );
    }
}

impl Visitor for HandGraph<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        self.deque(stmt);
    }
}
