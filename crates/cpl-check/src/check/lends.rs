//! Lexical borrows: what a place lends, and that two live lends of one
//! value never share storage (`plan/19-index-and-lends.md` §3.5 and §3.6).
//!
//! A *place* is a name followed by fields and projections: `a[i]`,
//! `g.row(r)`, `b.cnt`, `b.at(i)`. Evaluating one lends part of its root
//! for as long as the result is in use:
//!
//! - arguments of one call are lent together, so `swap(mut a[i], mut a[j])`
//!   needs `i != j`;
//! - a slice or row bound by `let` keeps its root lent until the last
//!   statement that names it;
//! - a lent slice or row is second-class: it can be passed, bound by `let`
//!   and iterated, never stored, assigned or returned.
//!
//! Whether two lends are disjoint is decided by the projection's `law
//! disjoint` and the comparisons known to hold at that point (the *flow
//! facts*): never by a guess. When it can't be decided, the error says
//! which condition to state.

use super::*;
use cpl_syntax::ast::{self, Arg, BinOp, Block, CmpOp, Cond, Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_block, walk_expr, walk_stmt, Visitor};
use std::collections::HashSet;

/// Methods that change their receiver.
const MUTATORS: &[&str] = &[
    "push", "pop", "sort", "rev", "clear", "insert", "remove", "swap", "set", "add",
];
/// Methods that store their arguments in the receiver.
const STORES: &[&str] = &["push", "insert", "set", "add"];

/// `lhs op rhs` is known to hold here. `names` are the names it mentions:
/// assigning any of them forgets it.
pub(crate) struct Fact {
    lhs: String,
    op: CmpOp,
    rhs: String,
    names: Vec<String>,
}

/// One step from a value to a part of it.
#[derive(Clone)]
pub(crate) enum Step {
    Field(String),
    Proj(Proj),
}

/// A projection applied to arguments: `[i]`, `[l..=r]`, `.row(r)`, `.at(i)`.
#[derive(Clone)]
pub(crate) struct Proj {
    /// What tells two projections of one value apart: `[]`, `slice`, `row`
    /// or the method's name.
    name: String,
    /// For messages: `Array.at`, `Bag.at`.
    who: String,
    law: Law,
    args: Vec<Expr>,
}

#[derive(Clone)]
enum Law {
    /// Elements `a[i, j]` and `a[k, l]` are apart when any index differs.
    Element,
    /// `law disjoint(..) = ..` as source, and as the message shows it.
    Source { source: String, shown: String },
    /// The projection states no law: two of its lends always conflict.
    Missing,
}

/// A projection method of a struct, as a call to it found it.
#[derive(Clone)]
pub(crate) struct ProjCall {
    pub name: String,
    pub who: String,
    pub mutable: bool,
    /// The lent type, as written: for "is declared `lends Int`".
    pub lent: String,
    pub law: Option<String>,
}

/// A place: where a value lives, from a `let` or `var` down.
pub(crate) struct Place {
    root: String,
    root_mutable: bool,
    steps: Vec<Step>,
    /// The `let`-bound lend the place goes through, if any.
    via: Option<String>,
}

/// What a `let`-bound slice or row stands for.
#[derive(Clone)]
pub(crate) struct Lent {
    kind: &'static str,
    root: String,
    root_mutable: bool,
    steps: Vec<Step>,
}

/// `let part = a[s]`: `a` stays lent until the last statement that names
/// `part`.
pub(crate) struct Hold {
    name: String,
    root: String,
    steps: Vec<Step>,
    mutable: bool,
    at: Span,
    until: Span,
}

/// What a top-level statement leaves behind for the statements after it.
/// Each one is checked on its own, so these are handed to the later ones.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Flow {
    None,
    /// `let part = a[l..=r]`: `a` stays lent while `part` is in use.
    View(String),
    /// `assert i != j`: holds until one of these names is assigned.
    Assert(Vec<String>),
}

pub(crate) fn flow_of(stmts: &[Stmt]) -> Flow {
    let [stmt] = stmts else {
        return Flow::None;
    };
    match &stmt.kind {
        StmtKind::Assert { cond, .. } => Flow::Assert(names_in(cond)),
        StmtKind::Let {
            mutable: false,
            pat:
                ast::Pattern {
                    kind: ast::PatternKind::Name(name),
                    ..
                },
            init: Some(init),
            ..
        } => {
            let view = match &unparen(init).kind {
                ExprKind::Index { indices, .. } => indices
                    .iter()
                    .any(|i| matches!(unparen(i).kind, ExprKind::Range { .. })),
                ExprKind::Call { callee, .. } => {
                    matches!(&callee.kind, ExprKind::Field { name, .. } if name.text == "row")
                }
                _ => false,
            };
            if view {
                Flow::View(name.clone())
            } else {
                Flow::None
            }
        }
        _ => Flow::None,
    }
}

/// The names an item mentions, and the ones it may assign.
pub(crate) fn item_names(stmts: &[Stmt]) -> (Vec<String>, Vec<String>) {
    let mut names = Names::default();
    for stmt in stmts {
        walk_stmt(stmt, &mut names);
    }
    (names.0, written_in_stmts(stmts))
}

/// An earlier top-level statement whose effect reaches the item being
/// checked.
pub(crate) struct Replay {
    pub item: std::sync::Arc<crate::db::ItemSyntax>,
    /// The view is named again after the item being checked.
    pub live_after: bool,
    /// The view is changed in or after the item being checked.
    pub written: bool,
}

/// A span no statement has: a lend that began in an earlier top-level
/// statement, or one that outlives the one being checked.
const ELSEWHERE: Span = Span {
    start: u32::MAX,
    end: u32::MAX,
};

#[derive(Default)]
pub(crate) struct Lends {
    /// Top-level `let`-bound views from earlier statements, by name.
    global_views: HashMap<String, Lent>,
    facts: Vec<Fact>,
    holds: Vec<Hold>,
    /// Receivers of an enclosing place expression: `a` in `a[i]` is not
    /// also a use of all of `a`.
    covered: HashSet<Span>,
    /// Expressions that made a lent view, with its kind: `slice` or `row`.
    view_exprs: HashMap<Span, &'static str>,
    /// Calls to projections, by the call's span.
    proj_calls: HashMap<Span, ProjCall>,
    /// `let`-bound views, by where the name is declared.
    views: HashMap<Span, Lent>,
    /// The one statement where `lend` may stand: the `lend` of the
    /// projection being checked.
    lend_at: Option<Span>,
    /// The projection being checked was already told its `lend`s are
    /// wrong.
    lend_reported: bool,
}

/// Source text of an expression, exact enough to compare two for equality.
pub(crate) fn show(e: &Expr) -> String {
    let list = |items: &[Expr]| items.iter().map(show).collect::<Vec<_>>().join(", ");
    match &e.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Char(c) => format!("{c:?}"),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Name(name) => name.clone(),
        ExprKind::SelfValue => "self".into(),
        ExprKind::Paren(inner) => show(inner),
        ExprKind::Binary { op, lhs, rhs, .. } => {
            format!("({} {} {})", show(lhs), op.text(), show(rhs))
        }
        ExprKind::Unary { op, expr } => {
            let op = match op {
                ast::UnOp::Neg => "-",
                ast::UnOp::Not => "not ",
                ast::UnOp::BitNot => "~",
            };
            format!("({op}{})", show(expr))
        }
        ExprKind::Index { base, indices } => format!("{}[{}]", show(base), list(indices)),
        ExprKind::Field { base, name } => format!("{}.{}", show(base), name.text),
        ExprKind::Unwrap(inner) => format!("{}!", show(inner)),
        ExprKind::Call { callee, args } => {
            let args: Vec<String> = args
                .iter()
                .map(|a| match &a.name {
                    Some(name) => format!("{}: {}", name.text, show(&a.value)),
                    None => show(&a.value),
                })
                .collect();
            format!("{}({})", show(callee), args.join(", "))
        }
        ExprKind::Range {
            lo, hi, inclusive, ..
        } => format!(
            "{}{}{}",
            lo.as_deref().map(show).unwrap_or_default(),
            if *inclusive { "..=" } else { ".." },
            hi.as_deref().map(show).unwrap_or_default()
        ),
        ExprKind::Compare { first, rest } => {
            let mut out = format!("({}", show(first));
            for (op, operand) in rest {
                out += &format!(" {} {}", cmp_text(*op), show(operand));
            }
            out + ")"
        }
        ExprKind::Tuple(items) => format!("({})", list(items)),
        // Anything else is equal only to itself.
        _ => format!("<{}..{}>", e.span.start, e.span.end),
    }
}

fn cmp_text(op: CmpOp) -> &'static str {
    match op {
        CmpOp::Eq => "==",
        CmpOp::Ne => "!=",
        CmpOp::Lt => "<",
        CmpOp::Le => "<=",
        CmpOp::Gt => ">",
        CmpOp::Ge => ">=",
        CmpOp::In => "in",
        CmpOp::NotIn => "not in",
        CmpOp::Order => "<=>",
    }
}

/// `b op' a` says the same as `a op b`.
fn flip(op: CmpOp) -> Option<CmpOp> {
    Some(match op {
        CmpOp::Lt => CmpOp::Gt,
        CmpOp::Gt => CmpOp::Lt,
        CmpOp::Le => CmpOp::Ge,
        CmpOp::Ge => CmpOp::Le,
        CmpOp::Eq => CmpOp::Eq,
        CmpOp::Ne => CmpOp::Ne,
        _ => return None,
    })
}

fn negate(op: CmpOp) -> Option<CmpOp> {
    Some(match op {
        CmpOp::Lt => CmpOp::Ge,
        CmpOp::Gt => CmpOp::Le,
        CmpOp::Le => CmpOp::Gt,
        CmpOp::Ge => CmpOp::Lt,
        CmpOp::Eq => CmpOp::Ne,
        CmpOp::Ne => CmpOp::Eq,
        _ => return None,
    })
}

/// The comparisons that, once known, prove `a op b`.
fn implied_by(op: CmpOp) -> &'static [CmpOp] {
    match op {
        CmpOp::Ne => &[CmpOp::Ne, CmpOp::Lt, CmpOp::Gt],
        CmpOp::Lt => &[CmpOp::Lt],
        CmpOp::Gt => &[CmpOp::Gt],
        CmpOp::Le => &[CmpOp::Le, CmpOp::Lt, CmpOp::Eq],
        CmpOp::Ge => &[CmpOp::Ge, CmpOp::Gt, CmpOp::Eq],
        CmpOp::Eq => &[CmpOp::Eq],
        _ => &[],
    }
}

fn unparen(mut e: &Expr) -> &Expr {
    while let ExprKind::Paren(inner) = &e.kind {
        e = inner;
    }
    e
}

/// No calls: evaluating it twice gives the same value while no `var` in
/// it is assigned.
fn is_pure(e: &Expr) -> bool {
    struct Pure(bool);
    impl Visitor for Pure {
        fn expr(&mut self, e: &Expr) {
            self.0 &= matches!(
                e.kind,
                ExprKind::Int(_)
                    | ExprKind::Char(_)
                    | ExprKind::Bool(_)
                    | ExprKind::Name(_)
                    | ExprKind::SelfValue
                    | ExprKind::Paren(_)
                    | ExprKind::Binary { .. }
                    | ExprKind::Unary { .. }
                    | ExprKind::Field { .. }
                    | ExprKind::Index { .. }
                    | ExprKind::Range { .. }
                    | ExprKind::Compare { .. }
                    | ExprKind::Unwrap(_)
            );
        }
    }
    let mut pure = Pure(true);
    walk_expr(e, &mut pure);
    pure.0
}

#[derive(Default)]
struct Names(Vec<String>);

impl Visitor for Names {
    fn expr(&mut self, e: &Expr) {
        if let ExprKind::Name(name) = &e.kind {
            if !self.0.contains(name) {
                self.0.push(name.clone());
            }
        }
    }
}

pub(crate) fn names_in(e: &Expr) -> Vec<String> {
    let mut names = Names::default();
    walk_expr(e, &mut names);
    names.0
}

/// The name a place expression starts from.
fn root_name(mut e: &Expr) -> Option<&str> {
    loop {
        e = match &e.kind {
            ExprKind::Index { base, .. } | ExprKind::Field { base, .. } => base,
            ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Field { base, .. } => base,
                _ => return None,
            },
            ExprKind::Name(name) => return Some(name),
            ExprKind::SelfValue => return Some("self"),
            _ => return None,
        };
    }
}

/// Names a piece of code may assign: assignment targets, `mut` arguments,
/// receivers of mutating methods, and the names its patterns bind.
#[derive(Default)]
struct Written(Vec<String>);

impl Written {
    fn add(&mut self, name: Option<&str>) {
        if let Some(name) = name {
            if !self.0.iter().any(|n| n == name) {
                self.0.push(name.to_string());
            }
        }
    }
}

impl Visitor for Written {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assign { target, .. } = &stmt.kind {
            self.add(root_name(target));
        }
    }

    fn expr(&mut self, e: &Expr) {
        if let ExprKind::Call { callee, args } = &e.kind {
            for arg in args.iter().filter(|a| a.mutable) {
                self.add(root_name(&arg.value));
            }
            if let ExprKind::Field { base, name } = &callee.kind {
                if MUTATORS.contains(&name.text.as_str()) {
                    self.add(root_name(base));
                }
            }
        }
    }

    fn pattern(&mut self, pat: &ast::Pattern) {
        match &pat.kind {
            ast::PatternKind::Name(name) => self.add(Some(name)),
            ast::PatternKind::Tuple(items) | ast::PatternKind::Variant { args: items, .. } => {
                items.iter().for_each(|item| self.pattern(item))
            }
            _ => {}
        }
    }
}

fn written_in_block(block: &Block) -> Vec<String> {
    let mut written = Written::default();
    walk_block(block, &mut written);
    written.0
}

fn written_in_stmts(stmts: &[Stmt]) -> Vec<String> {
    let mut written = Written::default();
    for stmt in stmts {
        walk_stmt(stmt, &mut written);
    }
    written.0
}

fn mentions(stmt: &Stmt, name: &str) -> bool {
    let mut names = Names::default();
    walk_stmt(stmt, &mut names);
    names.0.iter().any(|n| n == name)
}

/// `e` with the law's parameters replaced by the arguments of two lends.
/// `s.lo` and `s.hi` of a range written out are its two ends: `l..=r`
/// ends at the boundary after `r`.
fn subst(e: &Expr, env: &[(&str, &Expr)]) -> Expr {
    let boxed = |inner: &Expr| Box::new(subst(inner, env));
    let kind = match &e.kind {
        ExprKind::Name(name) => match env.iter().find(|(param, _)| param == name) {
            Some((_, arg)) => return (*arg).clone(),
            None => e.kind.clone(),
        },
        ExprKind::Paren(inner) => ExprKind::Paren(boxed(inner)),
        ExprKind::Unwrap(inner) => ExprKind::Unwrap(boxed(inner)),
        ExprKind::Unary { op, expr } => ExprKind::Unary {
            op: *op,
            expr: boxed(expr),
        },
        ExprKind::Binary {
            op,
            op_span,
            lhs,
            rhs,
        } => ExprKind::Binary {
            op: *op,
            op_span: *op_span,
            lhs: boxed(lhs),
            rhs: boxed(rhs),
        },
        ExprKind::Compare { first, rest } => ExprKind::Compare {
            first: boxed(first),
            rest: rest.iter().map(|(op, x)| (*op, subst(x, env))).collect(),
        },
        ExprKind::Index { base, indices } => ExprKind::Index {
            base: boxed(base),
            indices: indices.iter().map(|x| subst(x, env)).collect(),
        },
        ExprKind::Field { base, name } => {
            let base = subst(base, env);
            if let ExprKind::Range {
                lo: Some(lo),
                hi: Some(hi),
                inclusive,
                ..
            } = &unparen(&base).kind
            {
                match name.text.as_str() {
                    "lo" => return (**lo).clone(),
                    "hi" if *inclusive => {
                        return Expr {
                            kind: ExprKind::Field {
                                base: hi.clone(),
                                name: ast::Name {
                                    text: "r".into(),
                                    span: name.span,
                                },
                            },
                            span: e.span,
                        }
                    }
                    "hi" => return (**hi).clone(),
                    _ => {}
                }
            }
            ExprKind::Field {
                base: Box::new(base),
                name: name.clone(),
            }
        }
        other => other.clone(),
    };
    Expr { kind, span: e.span }
}

/// The body and parameter names of a law given as source.
fn parse_law(source: &str) -> Option<(Vec<String>, Expr)> {
    let file = cpl_syntax::parse_file(source);
    let stmt = file.items.into_iter().next()?.1.stmts.into_iter().next()?;
    match stmt.kind {
        StmtKind::Law { params, body, .. } => {
            Some((params.into_iter().map(|p| p.name.text).collect(), body))
        }
        _ => None,
    }
}

fn builtin(name: &str, who: &str, law: Law, args: &[Expr]) -> Step {
    Step::Proj(Proj {
        name: name.into(),
        who: who.into(),
        law,
        args: args.to_vec(),
    })
}

impl<'a> Checker<'a> {
    // --- facts ---------------------------------------------------------------

    /// Records the comparisons `cond` makes true (or, with `negated`, the
    /// ones its being false makes true).
    pub(crate) fn add_facts(&mut self, cond: &Expr, negated: bool) {
        let cond = unparen(cond);
        if let (
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            },
            false,
        ) = (&cond.kind, negated)
        {
            self.add_facts(lhs, false);
            self.add_facts(rhs, false);
            return;
        }
        let ExprKind::Compare { first, rest } = &cond.kind else {
            return;
        };
        if negated && rest.len() != 1 {
            return;
        }
        let mut prev: &Expr = first;
        for (op, next) in rest {
            let op = if negated {
                negate(*op)
            } else {
                flip(*op).map(|_| *op)
            };
            if let (Some(op), true, true) = (op, is_pure(prev), is_pure(next)) {
                let mut names = names_in(prev);
                names.extend(names_in(next));
                self.lends.facts.push(Fact {
                    lhs: show(prev),
                    op,
                    rhs: show(next),
                    names,
                });
            }
            prev = next;
        }
    }

    pub(crate) fn add_cond_facts(&mut self, cond: &Cond, negated: bool) {
        if let Cond::Expr(expr) = cond {
            self.add_facts(expr, negated);
        }
    }

    /// Forgets what was known about names that may have changed.
    pub(crate) fn drop_facts(&mut self, names: &[String]) {
        if !names.is_empty() {
            self.lends
                .facts
                .retain(|fact| !fact.names.iter().any(|n| names.contains(n)));
        }
    }

    /// A loop body runs again after its own assignments: what was known
    /// about a name it assigns holds only on the first trip.
    pub(crate) fn drop_facts_written(&mut self, body: &Block, pat: Option<&ast::Pattern>) {
        let mut written = Written(written_in_block(body));
        if let Some(pat) = pat {
            written.pattern(pat);
        }
        self.drop_facts(&written.0);
    }

    /// Marks where the facts and holds of a block begin.
    pub(crate) fn lend_mark(&self) -> (usize, usize) {
        (self.lends.facts.len(), self.lends.holds.len())
    }

    /// Ends a block: what it learned and lent is over.
    pub(crate) fn lend_reset(&mut self, (facts, holds): (usize, usize)) {
        self.lends.facts.truncate(facts);
        self.lends.holds.truncate(holds);
    }

    /// A fn body knows none of the facts of the place it is defined in.
    pub(crate) fn lend_isolated<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let facts = std::mem::take(&mut self.lends.facts);
        let holds = std::mem::take(&mut self.lends.holds);
        let lend_at = self.lends.lend_at.take();
        let lend_reported = std::mem::take(&mut self.lends.lend_reported);
        let out = f(self);
        self.lends.facts = facts;
        self.lends.holds = holds;
        self.lends.lend_at = lend_at;
        self.lends.lend_reported = lend_reported;
        out
    }

    fn has_fact(&self, a: &str, ops: &[CmpOp], b: &str) -> bool {
        self.lends.facts.iter().any(|fact| {
            (fact.lhs == a && fact.rhs == b && ops.contains(&fact.op))
                || (fact.lhs == b
                    && fact.rhs == a
                    && flip(fact.op).is_some_and(|op| ops.contains(&op)))
        })
    }

    fn literal(e: &Expr) -> Option<i128> {
        match &unparen(e).kind {
            ExprKind::Int(v) => Some(*v),
            ExprKind::Char(c) => Some(*c as i128),
            ExprKind::Unary {
                op: ast::UnOp::Neg,
                expr,
            } => match &unparen(expr).kind {
                ExprKind::Int(v) => Some(-*v),
                _ => None,
            },
            _ => None,
        }
    }

    /// Can the flow facts decide this instance of a law? `false` means
    /// "write the `if` or the `assert`", never "it is false".
    fn prove(&mut self, e: &Expr) -> bool {
        let e = unparen(e);
        match &e.kind {
            ExprKind::Binary {
                op: BinOp::Or,
                lhs,
                rhs,
                ..
            } => self.prove(lhs) || self.prove(rhs),
            ExprKind::Binary {
                op: BinOp::And,
                lhs,
                rhs,
                ..
            } => self.prove(lhs) && self.prove(rhs),
            ExprKind::Compare { first, rest } if rest.len() == 1 => {
                let (op, second) = &rest[0];
                self.prove_cmp(*op, first, second)
            }
            _ => false,
        }
    }

    fn prove_cmp(&mut self, op: CmpOp, a: &Expr, b: &Expr) -> bool {
        if flip(op).is_none() {
            return false;
        }
        let holds = |a: i128, b: i128| match op {
            CmpOp::Ne => a != b,
            CmpOp::Eq => a == b,
            CmpOp::Lt => a < b,
            CmpOp::Le => a <= b,
            CmpOp::Gt => a > b,
            _ => a >= b,
        };
        if let (Some(a), Some(b)) = (Self::literal(a), Self::literal(b)) {
            return holds(a, b);
        }
        let (a_text, b_text) = (show(a), show(b));
        let same = a_text == b_text && is_pure(a);
        if same {
            return matches!(op, CmpOp::Le | CmpOp::Ge | CmpOp::Eq);
        }
        if self.has_fact(&a_text, implied_by(op), &b_text) {
            return true;
        }
        // Ranges of integers that can't meet: `x: 0..5` and `y: 5..9`.
        if op != CmpOp::Eq {
            if let ((Some(a_lo), Some(a_hi)), (Some(b_lo), Some(b_hi))) =
                (self.bounds(a), self.bounds(b))
            {
                return match op {
                    CmpOp::Ne => a_hi < b_lo || b_hi < a_lo,
                    CmpOp::Lt => a_hi < b_lo,
                    CmpOp::Le => a_hi <= b_lo,
                    CmpOp::Gt => a_lo > b_hi,
                    _ => a_lo >= b_hi,
                };
            }
        }
        false
    }

    // --- places --------------------------------------------------------------

    /// The place an expression names, or `None` for a plain value.
    pub(crate) fn place(&mut self, mut e: &Expr) -> Option<Place> {
        let mut steps = Vec::new();
        loop {
            e = match &e.kind {
                ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
                ExprKind::Field { base, name } => {
                    steps.push(Step::Field(name.text.clone()));
                    base
                }
                ExprKind::Index { base, indices } => {
                    let slice = self.lends.view_exprs.contains_key(&e.span)
                        || indices
                            .iter()
                            .any(|i| matches!(unparen(i).kind, ExprKind::Range { .. }));
                    steps.push(if slice {
                        builtin(
                            "slice",
                            "Array.at(Range)",
                            Law::Source {
                                source: "law disjoint(s, t) = s.hi <= t.lo or t.hi <= s.lo".into(),
                                shown: "s.hi <= t.lo or t.hi <= s.lo".into(),
                            },
                            indices,
                        )
                    } else {
                        builtin("[]", "Array.at", Law::Element, indices)
                    });
                    base
                }
                ExprKind::Call { callee, args } => {
                    let ExprKind::Field { base, .. } = &callee.kind else {
                        break;
                    };
                    let args: Vec<Expr> = args.iter().map(|a| a.value.clone()).collect();
                    if self.lends.view_exprs.get(&e.span) == Some(&"row") {
                        steps.push(builtin(
                            "row",
                            "Array.row",
                            Law::Source {
                                source: "law disjoint(r, q) = r != q".into(),
                                shown: "r != q".into(),
                            },
                            &args,
                        ));
                    } else if let Some(call) = self.lends.proj_calls.get(&e.span) {
                        let law = match &call.law {
                            Some(source) => Law::Source {
                                shown: parse_law(source)
                                    .map(|(_, body)| show(&body))
                                    .unwrap_or_default(),
                                source: source.clone(),
                            },
                            None => Law::Missing,
                        };
                        steps.push(Step::Proj(Proj {
                            name: call.name.clone(),
                            who: call.who.clone(),
                            law,
                            args,
                        }));
                    } else {
                        break;
                    }
                    base
                }
                _ => break,
            };
        }
        let name = match &e.kind {
            ExprKind::Name(name) => name.as_str(),
            ExprKind::SelfValue => "self",
            _ => return None,
        };
        steps.reverse();
        let (root_mutable, lent) = match self.local(name) {
            Some(local) => (local.mutable, self.lends.views.get(&local.span).cloned()),
            None => match self.env.global(name).as_deref() {
                Some(Def::Value { mutable, .. }) => {
                    (*mutable, self.lends.global_views.get(name).cloned())
                }
                _ => return None,
            },
        };
        // A `let`-bound lend stands for the place it was lent from.
        if let Some(lent) = lent {
            let mut all = lent.steps;
            all.extend(steps);
            return Some(Place {
                root: lent.root,
                root_mutable: lent.root_mutable,
                steps: all,
                via: Some(name.to_string()),
            });
        }
        Some(Place {
            root: name.to_string(),
            root_mutable,
            steps,
            via: None,
        })
    }

    fn step_disjoint(&mut self, a: &Proj, b: &Proj) -> bool {
        match &a.law {
            Law::Element => {
                a.args.len() == b.args.len()
                    && a.args
                        .iter()
                        .zip(&b.args)
                        .any(|(x, y)| self.prove_cmp(CmpOp::Ne, x, y))
            }
            Law::Source { source, .. } => {
                let Some((params, body)) = parse_law(source) else {
                    return false;
                };
                if params.len() != a.args.len() + b.args.len() {
                    return false;
                }
                let env: Vec<(&str, &Expr)> = params
                    .iter()
                    .map(String::as_str)
                    .zip(a.args.iter().chain(&b.args))
                    .collect();
                let instance = subst(&body, &env);
                self.prove(&instance)
            }
            Law::Missing => false,
        }
    }

    /// `None` when two places of one root can't share storage, else why
    /// they might.
    fn overlap(&mut self, p: &[Step], q: &[Step]) -> Option<String> {
        for (k, (a, b)) in p.iter().zip(q).enumerate() {
            match (a, b) {
                // Different fields are always apart.
                (Step::Field(a), Step::Field(b)) if a != b => return None,
                (Step::Field(_), Step::Field(_)) => {}
                (Step::Proj(a), Step::Proj(b)) if a.name == b.name => {
                    if self.step_disjoint(a, b) {
                        return None;
                    }
                    let law = match &a.law {
                        Law::Element => Some("i != j"),
                        Law::Source { shown, .. } => Some(shown.as_str()),
                        Law::Missing => None,
                    };
                    let same = a.args.iter().map(show).eq(b.args.iter().map(show))
                        && a.args.iter().all(is_pure);
                    if same {
                        // The same element: look one step deeper.
                        if k + 1 < p.len() || k + 1 < q.len() {
                            continue;
                        }
                        return Some(match law {
                            Some(law) => format!(
                                "both are the same part of it; {} lends disjoint parts only when `{law}`",
                                a.who
                            ),
                            None => "both are the same part of it".to_string(),
                        });
                    }
                    return Some(match law {
                        Some(law) => format!(
                            "{} lends disjoint parts only when `{law}`, and that is not known here",
                            a.who
                        ),
                        None => format!(
                            "{} declares no `law disjoint`, so two of its lends always conflict",
                            a.who
                        ),
                    });
                }
                (Step::Proj(a), Step::Proj(b)) => {
                    return Some(format!(
                        "`{}` and `{}` are different projections of the same value, and those always conflict",
                        a.who, b.who
                    ))
                }
                _ => {
                    return Some(
                        "one is a field and the other a projection of the same value".to_string(),
                    )
                }
            }
        }
        Some("one contains the other".to_string())
    }

    /// Arguments of one call are lent together: `f(mut a[i], mut a[j])`
    /// needs `i != j`.
    pub(crate) fn lent_together(&mut self, args: &[Arg]) {
        if args.len() < 2 || !args.iter().any(|a| a.mutable) {
            return;
        }
        let mut lent: Vec<(&Arg, Place)> = Vec::new();
        for arg in args {
            let Some(place) = self.place(&arg.value) else {
                continue;
            };
            // A number or a position read from a place is copied before
            // the call; anything bigger is passed by reference.
            if !arg.mutable {
                let ty = self.quietly(|c| c.infer(&arg.value));
                let by_reference = matches!(
                    ty,
                    Ty::Array { .. } | Ty::Named { .. } | Ty::Str | Ty::Tuple(_) | Ty::Record(_)
                );
                if !by_reference {
                    continue;
                }
            }
            lent.push((arg, place));
        }
        for second in 1..lent.len() {
            for first in 0..second {
                let ((a, p), (b, q)) = (&lent[first], &lent[second]);
                if !(a.mutable || b.mutable) || p.root != q.root {
                    continue;
                }
                let (p, q) = (p.steps.clone(), q.steps.clone());
                let (a, b) = (*a, *b);
                let Some(why) = self.overlap(&p, &q) else {
                    continue;
                };
                let message = format!(
                    "`{}` and `{}` may be the same storage, and {} lent `mut` to this call: {why}",
                    show(&a.value),
                    show(&b.value),
                    if a.mutable && b.mutable {
                        "both are"
                    } else {
                        "one is"
                    }
                );
                let first_span = a.value.span;
                if let Some(diag) = self.error(codes::LENT_TWICE, b.value.span, message) {
                    diag.labels.push(cpl_diag::Label {
                        span: first_span,
                        message: "also lent here".into(),
                    });
                    diag.help = Some(
                        "guard it (`if i != j: ...`), state it (`assert i != j`), or copy the read-only one first"
                            .into(),
                    );
                }
            }
        }
    }

    // --- lends bound by `let` ------------------------------------------------

    /// The kind of lent view an expression is, if it is one.
    pub(crate) fn view_kind(&self, e: &Expr) -> Option<&'static str> {
        let e = unparen(e);
        if let Some(kind) = self.lends.view_exprs.get(&e.span) {
            return Some(kind);
        }
        match &e.kind {
            ExprKind::Name(name) => match self.local(name) {
                Some(local) => self.lends.views.get(&local.span).map(|lent| lent.kind),
                None => self.lends.global_views.get(name).map(|lent| lent.kind),
            },
            _ => None,
        }
    }

    /// Brings the effect of an earlier top-level statement into the item
    /// about to be checked, `current`.
    pub(crate) fn replay(&mut self, replay: &Replay, current: &[Stmt]) {
        let Some(stmt) = replay.item.parsed.stmts.first() else {
            return;
        };
        match (&replay.item.flow, &stmt.kind) {
            (Flow::Assert(_), StmtKind::Assert { cond, .. }) => self.add_facts(cond, false),
            (
                Flow::View(name),
                StmtKind::Let {
                    init: Some(init), ..
                },
            ) => {
                // The statement was checked, and reported, as its own item.
                let found = self.quietly(|c| {
                    c.infer(init);
                    c.view_kind(init).zip(c.place(init))
                });
                // Its spans are positions in another item's text.
                self.lends.view_exprs.clear();
                self.lends.proj_calls.clear();
                self.lends.covered.clear();
                let Some((kind, place)) = found else {
                    return;
                };
                self.lends.global_views.insert(
                    name.clone(),
                    Lent {
                        kind,
                        root: place.root.clone(),
                        root_mutable: place.root_mutable,
                        steps: place.steps.clone(),
                    },
                );
                let until = match current.iter().rev().find(|s| mentions(s, name)) {
                    _ if replay.live_after => ELSEWHERE,
                    Some(last) => last.span,
                    None => return,
                };
                self.lends.holds.push(Hold {
                    name: name.clone(),
                    root: place.root,
                    steps: place.steps,
                    mutable: replay.written,
                    at: ELSEWHERE,
                    until,
                });
            }
            _ => {}
        }
    }

    pub(crate) fn note_view(&mut self, e: &Expr, kind: &'static str) {
        self.lends.view_exprs.insert(e.span, kind);
    }

    pub(crate) fn note_proj_call(&mut self, e: &Expr, call: ProjCall) {
        self.lends.proj_calls.insert(e.span, call);
    }

    /// `let name = <view>`: the root stays lent until the last statement
    /// of `rest` that names `name`.
    pub(crate) fn begin_hold(&mut self, stmt: &Stmt, name: &str, value: &Expr, rest: &[Stmt]) {
        let Some(kind) = self.view_kind(value) else {
            return;
        };
        let Some(place) = self.place(value) else {
            return;
        };
        let Some(decl) = self.local(name).map(|local| local.span) else {
            return;
        };
        self.lends.views.insert(
            decl,
            Lent {
                kind,
                root: place.root.clone(),
                root_mutable: place.root_mutable,
                steps: place.steps.clone(),
            },
        );
        let Some(last) = rest.iter().rev().find(|s| mentions(s, name)) else {
            return;
        };
        let mutable = written_in_stmts(rest).iter().any(|n| n == name);
        if mutable && !place.root_mutable {
            let message = format!(
                "`{name}` is changed below, but `{}` is not a var, so it can only be lent for reading",
                place.root
            );
            if let Some(diag) = self.error(codes::ASSIGN_TO_LET, stmt.span, message) {
                diag.help = Some(format!("declare it with `var {}`", place.root));
            }
        }
        self.lends.holds.push(Hold {
            name: name.to_string(),
            root: place.root,
            steps: place.steps,
            mutable,
            at: stmt.span,
            until: last.span,
        });
    }

    /// A statement is done: the lends last used in it are over.
    pub(crate) fn end_holds(&mut self, stmt: &Stmt) {
        self.lends.holds.retain(|hold| hold.until != stmt.span);
    }

    /// A read or write of a place while part of its root is lent to a
    /// `let`.
    pub(crate) fn check_access(&mut self, e: &Expr, write: bool) {
        if self.lends.holds.is_empty() {
            return;
        }
        let Some(place) = self.place(e) else {
            return;
        };
        for index in 0..self.lends.holds.len() {
            let hold = &self.lends.holds[index];
            if hold.root != place.root || place.via.as_deref() == Some(hold.name.as_str()) {
                continue;
            }
            if !(write || hold.mutable) {
                continue;
            }
            let (steps, name, mutable, at, until) = (
                hold.steps.clone(),
                hold.name.clone(),
                hold.mutable,
                hold.at,
                hold.until,
            );
            let Some(why) = self.overlap(&place.steps, &steps) else {
                continue;
            };
            let message = format!(
                "`{}` can't be {} here: `{}` is {}lent to `{name}` until its last use ({why})",
                show(e),
                if write { "changed" } else { "used" },
                place.root,
                if mutable { "mutably " } else { "" },
            );
            if let Some(diag) = self.error(codes::USED_WHILE_LENT, e.span, message) {
                if at != ELSEWHERE {
                    diag.labels.push(cpl_diag::Label {
                        span: at,
                        message: format!("lent to `{name}` here"),
                    });
                }
                if until != ELSEWHERE {
                    diag.labels.push(cpl_diag::Label {
                        span: until,
                        message: format!("`{name}` is last used here"),
                    });
                }
                diag.help = Some(format!(
                    "finish with `{name}` first, go through `{name}`, or keep the position and re-project later; `.copy()` keeps an owned copy"
                ));
            }
            return;
        }
    }

    /// Called on entry to every expression: an outermost place expression
    /// is a use of its place.
    pub(crate) fn enter_place(&mut self, e: &Expr) -> bool {
        if self.lends.holds.is_empty() {
            return false;
        }
        let base = match &e.kind {
            ExprKind::Name(_) => None,
            ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => Some(base),
            ExprKind::Call { callee, .. } => match &callee.kind {
                ExprKind::Field { base, .. } => Some(base),
                _ => return false,
            },
            _ => return false,
        };
        let outermost = !self.lends.covered.remove(&e.span);
        if let Some(base) = base {
            self.lends.covered.insert(base.span);
        }
        outermost
    }

    // --- lent values are second-class ---------------------------------------

    /// Reports a lent slice or row put where it would outlive its block.
    /// `what` is the thing that would keep it: "`return`", "this
    /// assignment".
    pub(crate) fn no_escape(&mut self, e: &Expr, what: &str) {
        let Some(kind) = self.view_kind(e) else {
            return;
        };
        let src = show(e);
        let message = format!(
            "{what} would keep the lent {kind} `{src}` after this block; a lent value lives only in the block that made it"
        );
        if let Some(diag) = self.error(codes::LENT_ESCAPES, e.span, message) {
            diag.help = Some(format!(
                "keep the range or the row index (a plain value) and re-project where it is used, or keep a copy: `{src}.copy()`"
            ));
        }
    }

    /// The arguments of a call that stores them must not be lent views.
    pub(crate) fn no_escape_args(&mut self, callee: &Expr, callee_ty: &Ty, args: &[Arg]) {
        let what = match (&callee.kind, callee_ty) {
            (ExprKind::Field { name, .. }, _) if STORES.contains(&name.text.as_str()) => {
                format!("`.{}(...)`", name.text)
            }
            (_, Ty::TypeName(name) | Ty::Named { name, .. }) => {
                format!("storing it in a `{name}`")
            }
            _ => return,
        };
        for arg in args {
            self.no_escape(&arg.value, &what);
        }
    }

    // --- writes through projections -----------------------------------------

    /// A write through `target` needs `lends mut` at every projection on
    /// the way, and a `var` under a slice.
    pub(crate) fn writable(&mut self, target: &Expr) {
        let mut e = target;
        loop {
            e = match &e.kind {
                ExprKind::Paren(inner) | ExprKind::Unwrap(inner) => inner,
                ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => base,
                ExprKind::Call { callee, .. } => {
                    let ExprKind::Field { base, .. } = &callee.kind else {
                        return;
                    };
                    if let Some(call) = self.lends.proj_calls.get(&e.span).cloned() {
                        if !call.mutable {
                            let message = format!(
                                "`{}` is lent read-only: `{}` is declared `lends {}`",
                                show(e),
                                call.who,
                                call.lent
                            );
                            if let Some(diag) = self.error(codes::ASSIGN_TO_LET, e.span, message) {
                                diag.help = Some(format!(
                                    "declare it `lends mut {}` to allow writes through it",
                                    call.lent
                                ));
                            }
                            return;
                        }
                    }
                    base
                }
                _ => return,
            };
        }
    }

    /// `a[l..=r].sort()` changes `a`: it must be a `var`.
    pub(crate) fn mutates_view(&mut self, call: &Expr, callee: &Expr) {
        let ExprKind::Field { base, name } = &callee.kind else {
            return;
        };
        if !MUTATORS.contains(&name.text.as_str()) || self.view_kind(base).is_none() {
            return;
        }
        let Some(place) = self.place(base) else {
            return;
        };
        // A `let`-bound lend was checked where it was bound.
        if place.root_mutable || place.via.is_some() {
            return;
        }
        let message = format!(
            "`{}` is not a var, so `{}` can't change it",
            place.root,
            show(call)
        );
        if let Some(diag) = self.error(codes::ASSIGN_TO_LET, base.span, message) {
            diag.help = Some(format!("declare it with `var {}`", place.root));
        }
    }

    // --- projections ---------------------------------------------------------

    /// Checks the shape of `fn name(self, ..) lends [mut] T`: one `lend`
    /// of a part of `self`, as a statement of the body itself. What
    /// follows it is the write-back, which runs when the caller is done
    /// with the place and so needs `mut self`.
    pub(crate) fn projection_shape(&mut self, f: &ast::FnDecl, fields: &[String]) {
        let Some(lends) = &f.lends else {
            return;
        };
        let name = &f.name.text;
        let receiver = f.params.first().filter(|p| p.name.text == "self");
        if receiver.is_none() {
            let message = format!(
                "a projection lends part of its receiver, so `{name}` needs `self` as its first parameter"
            );
            self.error(codes::BAD_PROJECTION, f.name.span, message);
        }
        let stmts: &[Stmt] = match &f.body {
            ast::FnBody::Block(block) => &block.stmts,
            _ => &[],
        };
        let mut found = stmts
            .iter()
            .enumerate()
            .filter_map(|(at, stmt)| match &stmt.kind {
                StmtKind::Lend { mutable, place } => Some((at, stmt, *mutable, place)),
                _ => None,
            });
        let (Some((at, stmt, mutable, place)), None) = (found.next(), found.next()) else {
            let message = format!(
                "`{name}` is declared `lends`, so every path through it must reach one `lend place`"
            );
            if let Some(diag) = self.error(codes::BAD_PROJECTION, f.name.span, message) {
                diag.help = Some(
                    "give the body one `lend <place>`, the part of `self` it lends, as a statement of its own"
                        .into(),
                );
            }
            self.lends.lend_reported = true;
            return;
        };
        if let Some(after) = stmts.get(at + 1) {
            if receiver.is_some_and(|p| !p.mutable) {
                let message =
                    format!("code after the `lend` changes `self`, so `{name}` takes `mut self`");
                self.error(codes::BAD_PROJECTION, after.span, message);
            }
        }
        self.lends.lend_at = Some(stmt.span);
        if mutable && !lends.mutable {
            let message =
                format!("`{name}` is declared `lends` without `mut`, so it can't `lend mut`");
            if let Some(diag) = self.error(codes::BAD_PROJECTION, stmt.span, message) {
                diag.help = Some("declare it `lends mut`".into());
            }
        } else if !mutable && lends.mutable {
            let message = format!(
                "`{name}` is declared `lends mut`, so say `lend mut {}`",
                show(place)
            );
            self.error(codes::BAD_PROJECTION, stmt.span, message);
        }
        let root = root_name(place);
        if !root.is_some_and(|root| root == "self" || fields.iter().any(|f| f == root)) {
            let message = format!(
                "a projection may only lend part of `self`; `{}` is not reached through a field",
                show(place)
            );
            self.error(codes::BAD_PROJECTION, place.span, message);
        }
    }

    /// `lend` anywhere but in the body of a projection, as one of its own
    /// statements.
    pub(crate) fn lend_in_place(&mut self, stmt: &Stmt) {
        if self.lends.lend_at != Some(stmt.span) && !self.lends.lend_reported {
            self.error(
                codes::BAD_PROJECTION,
                stmt.span,
                "`lend` is a statement of a projection's body (`fn name(self, ...) lends T`), and only there"
                    .to_string(),
            );
        }
    }
}
