//! Where a step along a domain is known to stay inside it.
//!
//! `l + 1` is a `?I`: it is `none` when `l` is the last position. Inside
//! `if l < r:`, with `r` an `I` too, it can't be, so there `l + 1` reads
//! as an `I`, and so does `r - 1`. Like [`crate::narrow`], the rule is
//! read off the source alone: a test or a loop orders two names, and the
//! code it covers assigns neither. The checker and the lowering both ask
//! [`Steps`], so they can't disagree. All that is left to them is to see
//! that the two names are positions of one domain.
//!
//! What orders two names:
//!
//! - `if l < r:`, `while l < r:` and `l < r and ...`, for what they cover,
//!   and their opposites for an `else` and for the code after an
//!   `if l >= r: return`;
//! - `l <= r` together with `l != r`, from two tests or from one test and
//!   a loop;
//! - `for m in l..r`, in a loop or a comprehension: `l <= m` and `m < r`.
//!
//! An end of a domain orders a name too: inside `if i > D.first:`,
//! `i - 1` is in `D`, and inside `if i < D.last:` so is `i + 1`.

use std::borrow::Cow;

use crate::ast::*;
use crate::narrow::{kept, unparen, Calls};
use crate::visit::{walk_block, walk_expr, walk_stmt, Visitor};
use cpl_diag::Span;

#[derive(Clone, Copy, PartialEq)]
enum Rel {
    Lt,
    Le,
    Ne,
}

/// `a rel b`, for two names, or a name and an end of a domain
/// (`D.first`, `D.last`).
#[derive(Clone)]
struct Fact<'a> {
    a: Cow<'a, str>,
    rel: Rel,
    b: Cow<'a, str>,
}

/// What `a op b` coming out as `when` says.
fn compared<'a>(a: Cow<'a, str>, op: CmpOp, b: Cow<'a, str>, when: bool, out: &mut Vec<Fact<'a>>) {
    let mut say = |a: &Cow<'a, str>, rel, b: &Cow<'a, str>| {
        out.push(Fact {
            a: a.clone(),
            rel,
            b: b.clone(),
        })
    };
    let (a, b) = (&a, &b);
    match (op, when) {
        (CmpOp::Lt, true) | (CmpOp::Ge, false) => say(a, Rel::Lt, b),
        (CmpOp::Gt, true) | (CmpOp::Le, false) => say(b, Rel::Lt, a),
        (CmpOp::Le, true) | (CmpOp::Gt, false) => say(a, Rel::Le, b),
        (CmpOp::Ge, true) | (CmpOp::Lt, false) => say(b, Rel::Le, a),
        (CmpOp::Eq, true) | (CmpOp::Ne, false) => {
            say(a, Rel::Le, b);
            say(b, Rel::Le, a);
        }
        (CmpOp::Ne, true) | (CmpOp::Eq, false) => say(a, Rel::Ne, b),
        _ => {}
    }
}

fn name(e: &Expr) -> Option<&str> {
    match &unparen(e).kind {
        ExprKind::Name(name) => Some(name),
        _ => None,
    }
}

/// The domain `end` is an end of, when it is `D.first` or `D.last`.
pub fn end_domain(end: &str) -> Option<&str> {
    end.strip_suffix(".first")
        .or_else(|| end.strip_suffix(".last"))
}

/// One side of a comparison that orders: a name, or `D.first` / `D.last`
/// written out.
fn side(e: &Expr) -> Option<Cow<'_, str>> {
    match &unparen(e).kind {
        ExprKind::Name(name) => Some(Cow::Borrowed(name)),
        ExprKind::Field { base, name: end } if matches!(end.text.as_str(), "first" | "last") => {
            name(base).map(|domain| Cow::Owned(format!("{domain}.{}", end.text)))
        }
        _ => None,
    }
}

/// What `cond` coming out as `when` says about the order of names: each
/// comparison of an `and` chain that is true, of an `or` chain that is
/// false.
fn ordered<'a>(cond: &'a Expr, when: bool, out: &mut Vec<Fact<'a>>) {
    match &cond.kind {
        ExprKind::Paren(inner) => ordered(inner, when, out),
        ExprKind::Unary {
            op: UnOp::Not,
            expr,
        } => ordered(expr, !when, out),
        ExprKind::Binary { op, lhs, rhs, .. }
            if *op == if when { BinOp::And } else { BinOp::Or } =>
        {
            ordered(lhs, when, out);
            ordered(rhs, when, out);
        }
        // `a < b <= c` is true when each link is; false says nothing of a
        // chain of more than one.
        ExprKind::Compare { first, rest } if when || rest.len() == 1 => {
            let mut prev = first.as_ref();
            for (op, next) in rest {
                if let (Some(a), Some(b)) = (side(prev), side(next)) {
                    compared(a, *op, b, when, out);
                }
                prev = next;
            }
        }
        _ => {}
    }
}

fn of_cond<'a>(cond: &'a Cond, when: bool, out: &mut Vec<Fact<'a>>) {
    if let Cond::Expr(cond) = cond {
        ordered(cond, when, out);
    }
}

/// `for m in l..r`: `l <= m`, and `m < r` (`m <= r` for `..=`).
fn walked<'a>(pat: &'a Pattern, iter: &'a Expr, out: &mut Vec<Fact<'a>>) {
    let (
        PatternKind::Name(m),
        ExprKind::Range {
            lo,
            hi,
            inclusive,
            step: None,
        },
    ) = (&pat.kind, &unparen(iter).kind)
    else {
        return;
    };
    if let Some(lo) = lo.as_deref().and_then(name) {
        out.push(Fact {
            a: Cow::Borrowed(lo),
            rel: Rel::Le,
            b: Cow::Borrowed(m),
        });
    }
    if let Some(hi) = hi.as_deref().and_then(name) {
        let rel = if *inclusive { Rel::Le } else { Rel::Lt };
        out.push(Fact {
            a: Cow::Borrowed(m),
            rel,
            b: Cow::Borrowed(hi),
        });
    }
}

/// Whether `e` is written as a step: a name, `+` or `-`, and a whole
/// number.
pub fn is_step(e: &Expr) -> bool {
    step(e).is_some()
}

/// The name stepped from, and whether the step is up, for a step of one.
fn step(e: &Expr) -> Option<(&str, bool)> {
    let ExprKind::Binary { op, lhs, rhs, .. } = &unparen(e).kind else {
        return None;
    };
    let up = match op {
        BinOp::Add => true,
        BinOp::Sub => false,
        _ => return None,
    };
    match (name(lhs), &unparen(rhs).kind) {
        (Some(from), ExprKind::Int(1)) => Some((from, up)),
        _ => None,
    }
}

/// Where each order between two names holds, in some code.
#[derive(Default)]
pub struct Steps {
    /// A stretch of the source, and `a rel b` all through it.
    held: Vec<(Span, String, Rel, String)>,
}

struct Finder<'s, 'c> {
    steps: &'s mut Steps,
    by: u32,
    calls: &'c mut Calls<'c>,
}

impl Finder<'_, '_> {
    /// Keeps the facts whose names `covered` never assigns, for `at`.
    fn hold(&mut self, facts: Vec<Fact>, at: Span, covered: impl Fn(&mut dyn Visitor)) {
        for fact in facts {
            // An end of a domain is the domain's; nothing assigns it.
            let root = |side: &str| side.split('.').next().unwrap_or(side).to_string();
            let (a, b) = (root(&fact.a), root(&fact.b));
            let names = kept(vec![a.as_str(), b.as_str()], &mut *self.calls, |writes| {
                covered(writes)
            });
            if names.len() == 2 {
                let at = Span::new(at.start + self.by, at.end + self.by);
                self.steps
                    .held
                    .push((at, fact.a.to_string(), fact.rel, fact.b.to_string()));
            }
        }
    }

    /// The code after an `if` without an `else` whose every branch
    /// leaves: getting past it disproves each test.
    fn sequence(&mut self, stmts: &[Stmt]) {
        for (k, stmt) in stmts.iter().enumerate() {
            let StmtKind::Expr(Expr {
                kind:
                    ExprKind::If {
                        branches,
                        otherwise: None,
                    },
                ..
            }) = &stmt.kind
            else {
                continue;
            };
            let leaves = |body: &Block| {
                matches!(
                    body.stmts.last().map(|s| &s.kind),
                    Some(StmtKind::Return(_) | StmtKind::Break(_) | StmtKind::Continue(_))
                )
            };
            let rest = &stmts[k + 1..];
            let (Some(last), true) = (rest.last(), branches.iter().all(|(_, body)| leaves(body)))
            else {
                continue;
            };
            let mut facts = Vec::new();
            for (cond, _) in branches {
                of_cond(cond, false, &mut facts);
            }
            let at = Span::new(stmt.span.end, last.span.end);
            self.hold(facts, at, |writes| {
                rest.iter().for_each(|stmt| walk_stmt(stmt, writes))
            });
        }
    }
}

impl Visitor for Finder<'_, '_> {
    fn block(&mut self, block: &Block) {
        self.sequence(&block.stmts);
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::While { cond, body } => {
                let mut facts = Vec::new();
                of_cond(cond, true, &mut facts);
                self.hold(facts, body.span, |writes| walk_block(body, writes));
            }
            StmtKind::For {
                pat, iter, body, ..
            } => {
                let mut facts = Vec::new();
                walked(pat, iter, &mut facts);
                self.hold(facts, body.span, |writes| walk_block(body, writes));
            }
            _ => {}
        }
    }

    fn expr(&mut self, e: &Expr) {
        match &e.kind {
            ExprKind::If {
                branches,
                otherwise,
            } => {
                for (at, (cond, body)) in branches.iter().enumerate() {
                    let mut facts = Vec::new();
                    of_cond(cond, true, &mut facts);
                    self.hold(facts, body.span, |writes| walk_block(body, writes));
                    // What the test failing says, for each later branch.
                    let later = branches[at + 1..].iter().map(|(_, body)| body);
                    for body in later.chain(otherwise) {
                        let mut facts = Vec::new();
                        of_cond(cond, false, &mut facts);
                        self.hold(facts, body.span, |writes| walk_block(body, writes));
                    }
                }
            }
            ExprKind::Binary {
                op: op @ (BinOp::And | BinOp::Or),
                lhs,
                rhs,
                ..
            } => {
                let mut facts = Vec::new();
                ordered(lhs, *op == BinOp::And, &mut facts);
                self.hold(facts, rhs.span, |writes| walk_expr(rhs, writes));
            }
            ExprKind::Comprehension { body, clauses } => {
                let mut facts = Vec::new();
                for clause in clauses {
                    walked(&clause.pat, &clause.iter, &mut facts);
                }
                self.hold(facts, body.span, |writes| walk_expr(body, writes));
            }
            _ => {}
        }
    }
}

impl Steps {
    /// Reads `stmts`, the statements of one top-level item that starts
    /// `by` into its file. `calls` says whether calling a fn assigns a
    /// name, as for [`crate::narrow`].
    pub fn read(&mut self, stmts: &[Stmt], by: u32, calls: &mut Calls) {
        let mut finder = Finder {
            steps: self,
            by,
            calls,
        };
        finder.sequence(stmts);
        for stmt in stmts {
            walk_stmt(stmt, &mut finder);
        }
    }

    fn holds(&self, at: Span, a: &str, rel: Rel, b: &str) -> bool {
        self.held.iter().any(|(span, x, r, y)| {
            span.start <= at.start && at.end <= span.end && *r == rel && x == a && y == b
        })
    }

    fn less(&self, at: Span, a: &str, b: &str) -> bool {
        let differ = self.holds(at, a, Rel::Ne, b) || self.holds(at, b, Rel::Ne, a);
        // A position is never before its domain's first or after its last;
        // whoever asks sees that the name is a position of that domain.
        let within = a.ends_with(".first") || b.ends_with(".last");
        self.holds(at, a, Rel::Lt, b) || ((within || self.holds(at, a, Rel::Le, b)) && differ)
    }

    /// The names that show the step `e` stays inside: for `l + 1`, each
    /// `r` with `l < r` here, and for `r - 1` each `l`. The step is in
    /// its domain when one of them is a position of the same domain. `by`
    /// is where the item `e` is in starts in its file.
    pub fn bounds(&self, e: &Expr, by: u32) -> Vec<&str> {
        let Some((from, up)) = step(e) else {
            return Vec::new();
        };
        let at = Span::new(e.span.start + by, e.span.end + by);
        let mut found: Vec<&str> = Vec::new();
        for (_, a, _, b) in &self.held {
            let other = match (a == from, b == from) {
                (true, false) => b,
                (false, true) => a,
                _ => continue,
            };
            let stays = if up {
                self.less(at, from, other)
            } else {
                self.less(at, other, from)
            };
            if stays && !found.contains(&other.as_str()) {
                found.push(other);
            }
        }
        found
    }
}
