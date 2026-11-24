//! Loop forms (`plan/domains/lint-fmt/design/loop-forms.md`): what a loop
//! does, found once so every loop-shaped lint reads the same answer.

use super::{pushed_onto, var_decl};
use crate::std_fn;
use crate::Calls;
use cpl_syntax::ast::*;

/// `var name = init` followed by a `for` loop: a loop whose job may be to
/// compute `name`.
#[derive(Clone, Copy)]
pub(super) struct AccLoop<'a> {
    pub name: &'a str,
    pub init: &'a Expr,
    pub pat: &'a Pattern,
    pub iter: &'a Expr,
    pub body: &'a Block,
    /// Asks the checker about the loop's calls.
    pub calls: Calls<'a>,
}

/// What an [`AccLoop`] does with its variable.
pub(super) enum Form<'a> {
    /// The body is one assignment to it, maybe under one `if`: a fold (L0504).
    Fold {
        cond: Option<&'a Expr>,
        op: &'a AssignOp,
        value: &'a Expr,
    },
    /// The body is one push onto it, maybe under one `if`: a build (L0507).
    Build {
        cond: Option<&'a Expr>,
        value: &'a Expr,
    },
}

impl<'a> AccLoop<'a> {
    pub(super) fn of(decl: &'a Stmt, next: &'a Stmt, calls: Calls<'a>) -> Option<AccLoop<'a>> {
        let (name, init) = var_decl(decl)?;
        let StmtKind::For {
            label: None,
            pat,
            iter,
            body,
            else_body: None,
        } = &next.kind
        else {
            return None;
        };
        // `repeat n:` is a `for _ in 0..n` whose pattern and range nobody wrote.
        if matches!(pat.kind, PatternKind::Wildcard) && pat.span.end - pat.span.start > 1 {
            return None;
        }
        // `for x in fields(a)` is unrolled at compile time: no sequence to fold.
        if let ExprKind::Call { callee, .. } = &iter.kind {
            if matches!(&callee.kind, ExprKind::Name(n) if n == "fields") {
                return None;
            }
        }
        Some(AccLoop {
            name,
            init,
            pat,
            iter,
            body,
            calls,
        })
    }

    /// Every form the loop has.
    pub(super) fn forms(&self) -> Vec<Form<'a>> {
        self.single_write().into_iter().collect()
    }

    /// A body that is one write to the variable, maybe under one `if`.
    fn single_write(&self) -> Option<Form<'a>> {
        let [only] = &self.body.stmts[..] else {
            return None;
        };
        let (cond, only) = match &only.kind {
            StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }) => match &branches[..] {
                [(Cond::Expr(cond), then)] => match &then.stmts[..] {
                    [only] => (Some(cond), only),
                    _ => return None,
                },
                _ => return None,
            },
            _ => (None, only),
        };
        match &only.kind {
            StmtKind::Assign { target, op, value } if matches!(&target.kind, ExprKind::Name(n) if n == self.name) =>
            {
                let (op, value) = match op {
                    AssignOp::Set => spelled_out(self.name, value, self.calls)?,
                    _ => (op, value),
                };
                Some(Form::Fold { cond, op, value })
            }
            StmtKind::Expr(e) => {
                pushed_onto(e, self.name, self.calls).map(|value| Form::Build { cond, value })
            }
            _ => None,
        }
    }
}

static ADD: AssignOp = AssignOp::Binary(BinOp::Add);
static MAX: AssignOp = AssignOp::Max;
/// `x = f(x, e)`: the fold is the whole call.
static SET: AssignOp = AssignOp::Set;
static MIN: AssignOp = AssignOp::Min;

/// `x = x + v` and `x = max(x, v)`, C++'s spellings of `x += v` and `x max= v`:
/// the operator and `v`.
fn spelled_out<'a>(
    name: &str,
    value: &'a Expr,
    calls: Calls,
) -> Option<(&'static AssignOp, &'a Expr)> {
    let other = |a: &'a Expr, b: &'a Expr| {
        let is_self = |e: &Expr| matches!(&e.kind, ExprKind::Name(n) if n == name);
        match (is_self(a), is_self(b)) {
            (true, false) => Some(b),
            (false, true) => Some(a),
            _ => None,
        }
    };
    match &value.kind {
        ExprKind::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        } => Some((&ADD, other(lhs, rhs)?)),
        ExprKind::Call { callee, args } => {
            let op = match &callee.kind {
                ExprKind::Name(_) if calls.is(value, std_fn::MAX) => &MAX,
                ExprKind::Name(_) if calls.is(value, std_fn::MIN) => &MIN,
                ExprKind::Name(_) => {
                    let is_self = |e: &Expr| matches!(&e.kind, ExprKind::Name(n) if n == name);
                    return match &args[..] {
                        [a, b]
                            if [a, b].iter().all(|x| x.name.is_none() && !x.mutable)
                                && is_self(&a.value)
                                && !is_self(&b.value) =>
                        {
                            Some((&SET, value))
                        }
                        _ => None,
                    };
                }
                _ => return None,
            };
            match &args[..] {
                [a, b] if [a, b].iter().all(|x| x.name.is_none() && !x.mutable) => {
                    Some((op, other(&a.value, &b.value)?))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// `xs.rev.collect()` when a loop over `0..N` pushes `xs[N - 1 - i]`.
pub(super) fn reversed(i: &str, it: &str, v: &str) -> Option<String> {
    let n = it.strip_prefix("0..")?;
    let (xs, rest) = v.split_once('[')?;
    let idx = rest.strip_suffix(']')?;
    let len = format!("{xs}.len");
    let ok = idx == format!("{n} - 1 - {i}") && (n == len || !n.contains('.'));
    (ok && !xs.is_empty() && xs.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .then(|| format!("{xs}.rev.collect()"))
}
