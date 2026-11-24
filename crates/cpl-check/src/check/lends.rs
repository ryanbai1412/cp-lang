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

mod capture;
mod effects;
mod escape;
mod walk;

pub(crate) use effects::{lambda_effects, own_effects, Effects};

use super::known::{mutating_call, LENGTH_CHANGING_MUTATORS};
use super::moves::{is_compound, nested_moves};
use super::views::Keyed;
use super::*;
use crate::fx::HashSet;
use cpl_diag::msgs;
use cpl_syntax::ast::{self, Arg, BinOp, Block, CmpOp, Cond, Expr, ExprKind, Stmt, StmtKind};
use cpl_syntax::visit::{walk_block, walk_expr, walk_stmt, Visitor};

/// Methods that store their arguments in the receiver.
const STORES: &[&str] = &["push", "insert", "set", "set_at", "add"];

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
    pub(crate) root: String,
    pub(crate) root_mutable: bool,
    steps: Vec<Step>,
    /// The `let`-bound lend the place goes through, if any.
    pub(crate) via: Option<String>,
}

/// What a `let`-bound slice or row stands for.
#[derive(Clone)]
pub(crate) struct Lent {
    pub(super) kind: &'static str,
    root: String,
    root_mutable: bool,
    pub(super) read_only: bool,
    steps: Vec<Step>,
}

/// `let part = a[s]`: `a` stays lent until the last statement that names
/// `part`.
pub(crate) struct Hold {
    pub(super) name: String,
    pub(super) root: String,
    steps: Vec<Step>,
    mutable: bool,
    at: Span,
    until: Span,
    /// `for x in xs`: `xs` is lent to the loop, for reading, until it
    /// ends.
    in_loop: bool,
    /// Closure capture hold: prevents moving `root` (E0607) while the closure
    /// lives, without blocking reads or writes between calls (R05.10.2).
    pub(super) is_capture: bool,
}

/// What a top-level statement leaves behind for the statements after it.
/// Each one is checked on its own, so these are handed to the later ones.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Flow {
    None,
    /// `let part = a[l..=r]`: `a` stays lent while `part` is in use.
    View(String),
    /// `let f = () => ...`: captured variables cannot move while `f` lives.
    Capture(String),
    /// `assert i != j`: holds until one of these names is assigned.
    Assert(Vec<String>),
    /// `let b = a` / `b = a` / `S(a)`: moved if its type is a heap value (§128).
    Move(Vec<String>),
    /// `let k = 7 + 2`, `let f = a.filter(p)`: a fact about `k` that later
    /// items need (its range, R02.10.2; that it is walk-only, R07.1.2).
    Known(String),
    /// `var c = b.cur(0)` / `c = b.next(c)`: a `var` holding a lent value keeps
    /// the loan of the value last stored in it while `c` is in use.
    LentVar(String),
}

/// The arms of `e` that lend something other than what `lends_self` accepts: the lend
/// sources of `if`/`match` arms join, `Some(x)` lends what `x` lends, and `None` lends nothing.
fn foreign_lends<'e>(e: &'e Expr, lends_self: &dyn Fn(&Expr) -> bool, out: &mut Vec<&'e Expr>) {
    let tail = |block: &'e ast::Block, out: &mut Vec<&'e Expr>| {
        if let Some(Stmt {
            kind: StmtKind::Expr(e),
            ..
        }) = block.stmts.last()
        {
            foreign_lends(e, lends_self, out);
        }
    };
    let e = unparen(e);
    match &e.kind {
        ExprKind::None => {}
        ExprKind::Call { callee, args }
            if matches!(&unparen(callee).kind, ExprKind::Name(n) if n == "some" || n == "Some")
                && args.len() == 1 =>
        {
            foreign_lends(&args[0].value, lends_self, out);
        }
        ExprKind::If {
            branches,
            otherwise,
        } => {
            for (_, block) in branches {
                tail(block, out);
            }
            if let Some(block) = otherwise {
                tail(block, out);
            }
        }
        ExprKind::Match { arms, .. } => {
            for arm in arms {
                tail(&arm.body, out);
            }
        }
        ExprKind::Block(block) => tail(block, out),
        _ if lends_self(e) => {}
        _ => out.push(e),
    }
}

/// A call that may return a lent value: any method may be a projection
/// (`b.cur(i)`), and a lowercase fn may be `lends`; the replay asks the types.
/// `let d = o!` or `d = o!`: the name whose optional the statement unwraps.
pub(crate) fn unwrapped_name(stmt: &Stmt) -> Option<&str> {
    let value = match &stmt.kind {
        StmtKind::Let {
            init: Some(value), ..
        }
        | StmtKind::Assign { value, .. } => value,
        _ => return None,
    };
    match &unparen(value).kind {
        ExprKind::Unwrap(inner) => match &unparen(inner).kind {
            ExprKind::Name(src) => Some(src),
            _ => None,
        },
        _ => None,
    }
}

fn call_may_lend(e: &Expr) -> bool {
    match &unparen(e).kind {
        // `b.cur(0)!` and `o!` lend what the optional lends.
        ExprKind::Unwrap(inner) => {
            matches!(unparen(inner).kind, ExprKind::Name(_)) || call_may_lend(inner)
        }
        ExprKind::Call { callee, .. } => match &unparen(callee).kind {
            ExprKind::Field { .. } => true,
            ExprKind::Name(n) => n.chars().next().is_some_and(|c| c.is_ascii_lowercase()),
            _ => false,
        },
        _ => false,
    }
}

/// Constant arithmetic, or a `filter` call: what `Flow::Known` records.
fn known_init(e: &Expr) -> bool {
    use ast::{BinOp, UnOp};
    fn constant(e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Int(_) | ExprKind::Name(_) => true,
            ExprKind::Paren(e)
            | ExprKind::Unary {
                op: UnOp::Neg,
                expr: e,
            } => constant(e),
            ExprKind::Binary {
                op: BinOp::Add | BinOp::Sub | BinOp::Mul,
                lhs,
                rhs,
                ..
            } => constant(lhs) && constant(rhs),
            _ => false,
        }
    }
    let e = unparen(e);
    match &e.kind {
        ExprKind::Call { callee, .. } => {
            matches!(&callee.kind, ExprKind::Field { name, .. } if name.text == "filter")
        }
        ExprKind::Name(_) => false,
        _ => constant(e),
    }
}

fn ctor_move_args(e: &Expr) -> Vec<String> {
    let ExprKind::Call { callee, args } = &unparen(e).kind else {
        return Vec::new();
    };
    let is_type_name = match &unparen(callee).kind {
        ExprKind::Name(n) => n.chars().next().is_some_and(|c| c.is_ascii_uppercase()),
        ExprKind::Generic { base, .. } => {
            matches!(&unparen(base).kind, ExprKind::Name(n) if n.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        }
        _ => false,
    };
    if !is_type_name {
        return Vec::new();
    }
    args.iter()
        .filter_map(|a| match &unparen(&a.value).kind {
            ExprKind::Name(src) if !a.mutable => Some(src.clone()),
            _ => None,
        })
        .collect()
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
            otherwise,
            ty,
            ..
        } => {
            if matches!(unparen(init).kind, ExprKind::Lambda { .. }) {
                return Flow::Capture(name.clone());
            }
            let view = match &unparen(init).kind {
                ExprKind::Mut(_) => true,
                ExprKind::Index { .. } => true,
                // Any getter may lend (`g.rows`); the replay asks the types.
                ExprKind::Field { .. } => true,
                ExprKind::Unwrap(_) => call_may_lend(init),
                ExprKind::Call { callee, .. } => match &unparen(callee).kind {
                    // Any method may be a projection (`b.range(r)`); the
                    // replay asks the types.
                    ExprKind::Field { .. } => true,
                    ExprKind::Name(n) => n.chars().next().is_some_and(|c| c.is_ascii_lowercase()),
                    _ => false,
                },
                _ => false,
            };
            if view {
                Flow::View(name.clone())
            } else if ty.is_none() && otherwise.is_none() && known_init(init) {
                Flow::Known(name.clone())
            } else if otherwise.is_none() {
                match &unparen(init).kind {
                    ExprKind::Name(src) => Flow::Move(vec![src.clone()]),
                    ExprKind::With { base, .. } => match &unparen(base).kind {
                        ExprKind::Name(src) => Flow::Move(vec![src.clone()]),
                        _ => Flow::None,
                    },
                    _ => {
                        let moves = ctor_move_args(init);
                        if !moves.is_empty() {
                            Flow::Move(moves)
                        } else {
                            Flow::None
                        }
                    }
                }
            } else {
                Flow::None
            }
        }
        StmtKind::Let {
            mutable: true,
            pat,
            init: Some(init),
            otherwise: None,
            ..
        } => match &unparen(init).kind {
            ExprKind::Name(src) => Flow::Move(vec![src.clone()]),
            ExprKind::With { base, .. } => match &unparen(base).kind {
                ExprKind::Name(src) => Flow::Move(vec![src.clone()]),
                _ => Flow::None,
            },
            _ => {
                let moves = ctor_move_args(init);
                match &pat.kind {
                    _ if !moves.is_empty() => Flow::Move(moves),
                    ast::PatternKind::Name(name) if call_may_lend(init) => {
                        Flow::LentVar(name.clone())
                    }
                    _ => Flow::None,
                }
            }
        },
        StmtKind::Assign {
            target,
            op: ast::AssignOp::Set,
            value,
        } => {
            // Only plain variable assignment `b = a` moves `a`; slice stores (`dp[0, ..] = lst`) copy cells.
            if let (ExprKind::Name(t), ExprKind::Name(src)) = (&target.kind, &unparen(value).kind) {
                if t != src {
                    Flow::Move(vec![src.clone()])
                } else {
                    Flow::None
                }
            } else {
                let moves = ctor_move_args(value);
                match &target.kind {
                    _ if !moves.is_empty() => Flow::Move(moves),
                    ExprKind::Name(name) if call_may_lend(value) => Flow::LentVar(name.clone()),
                    _ => Flow::None,
                }
            }
        }
        _ if is_compound(stmt) => {
            let mut srcs: Vec<String> = Vec::new();
            for (_, moved) in nested_moves(stmt) {
                for m in moved {
                    if !srcs.contains(&m) {
                        srcs.push(m);
                    }
                }
            }
            if srcs.is_empty() {
                Flow::None
            } else {
                Flow::Move(srcs)
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
    /// How far before the item being checked this one starts.
    pub back: u32,
}

/// A span no statement has: a lend that began in an earlier top-level
/// statement, or one that outlives the one being checked.
const ELSEWHERE: Span = Span {
    start: u32::MAX,
    end: u32::MAX,
};

#[derive(Default)]
pub(crate) struct Lends {
    /// `let`-bound closures that capture only copied values, by where the
    /// name is declared: plain values themselves (decision 471).
    pub(super) plain_closures: HashSet<Span>,
    /// Top-level `let`-bound views from earlier statements, by name.
    pub(super) global_views: crate::names::NameMap<Lent>,
    facts: Vec<Fact>,
    pub(super) holds: Vec<Hold>,
    /// Uses already reported as conflicting with a hold: `xs.push(x)` is
    /// one conflict, not one for `xs` and another for `xs.push`.
    pub(crate) conflicts: Vec<Span>,
    /// Receivers of an enclosing place expression: `a` in `a[i]` is not
    /// also a use of all of `a`.
    covered: HashSet<Span>,
    /// Expressions that made a lent view, with its kind: `slice`, `row`, or `reverse view`.
    pub(super) view_exprs: HashMap<Span, &'static str>,
    /// Expressions whose lent view is strictly read-only (`Str`/`List` slices and `.rev`).
    pub(super) view_readonly: HashSet<Span>,
    /// Calls to projections, by the call's span.
    pub(super) proj_calls: HashMap<Span, ProjCall>,
    /// `let`-bound views, by where the name is declared.
    pub(super) views: HashMap<Span, Lent>,
    /// Locals holding a slice or a run/window/chunk, by where the name is
    /// declared (R07.4.5).
    pub(super) keyed: HashMap<Span, Keyed>,
    /// Locals holding a key of `V.ranges`, by where the name is declared.
    pub(super) range_keys: HashSet<Span>,
    /// The same for top-level `let`s, by name.
    pub(super) global_keyed: crate::names::NameMap<Keyed>,
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
                ast::UnOp::Pos => "+",
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

pub(crate) fn unparen(mut e: &Expr) -> &Expr {
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
pub(super) fn root_name(mut e: &Expr) -> Option<&str> {
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
                if mutating_call(&name.text, args) {
                    self.add(root_name(base));
                }
            }
        }
    }

    fn pattern(&mut self, pat: &ast::Pattern) {
        match &pat.kind {
            ast::PatternKind::Name(name) => self.add(Some(name)),
            ast::PatternKind::Tuple(_)
            | ast::PatternKind::Variant { .. }
            | ast::PatternKind::Record { .. } => pat.parts().for_each(|item| self.pattern(item)),
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
                    "start" => return (**lo).clone(),
                    "end" if *inclusive => {
                        return Expr {
                            kind: ExprKind::Field {
                                base: hi.clone(),
                                name: ast::Name {
                                    text: "r".into(),
                                    span: name.span,
                                },
                            },
                            span: e.span,
                        };
                    }
                    "end" => return (**hi).clone(),
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
                                source: "law disjoint(s, t) = s.end <= t.start or t.end <= s.start"
                                    .into(),
                                shown: "s.end <= t.start or t.end <= s.start".into(),
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
            None => match self.global(name).as_deref() {
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
        if a.name == "slice" && b.name == "slice" && a.args.len() == 1 && b.args.len() == 1 {
            if let (
                ExprKind::Range {
                    lo: lo1,
                    hi: hi1,
                    inclusive: false,
                    ..
                },
                ExprKind::Range {
                    lo: lo2,
                    hi: hi2,
                    inclusive: false,
                    ..
                },
            ) = (&unparen(&a.args[0]).kind, &unparen(&b.args[0]).kind)
            {
                let ab = hi1
                    .as_deref()
                    .zip(lo2.as_deref())
                    .is_some_and(|(x, y)| show(x) == show(y));
                let ba = hi2
                    .as_deref()
                    .zip(lo1.as_deref())
                    .is_some_and(|(x, y)| show(x) == show(y));
                if ab || ba {
                    return true;
                }
            }
        }
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
                    ));
                }
                _ => {
                    return Some(
                        "one is a field and the other a projection of the same value".to_string(),
                    );
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
        let mut lent: Vec<(&Arg, Place, bool)> = Vec::new();
        for arg in args {
            if let Some(place) = self.place(&arg.value) {
                lent.push((arg, place, !self.by_reference(arg)));
            }
        }
        for second in 1..lent.len() {
            for first in 0..second {
                let ((a, p, a_copy), (b, q, b_copy)) = (&lent[first], &lent[second]);
                // A copied read beside a `mut` lend conflicts if it may be the same storage
                // (`f(mut a[i], a[j])`, R9), not as a part read first (`f(mut a, a.len)`, R11).
                let part = (*a_copy || *b_copy) && p.steps.len() != q.steps.len();
                if !(a.mutable || b.mutable) || p.root != q.root || part {
                    continue;
                }
                let (p, q) = (p.steps.clone(), q.steps.clone());
                let (a, b) = (*a, *b);
                let Some(why) = self.overlap(&p, &q) else {
                    continue;
                };
                let (message, help) = super::advice::lent_twice(a, b, &why);
                let first_span = a.value.span;
                if let Some(diag) = self.report_said(b.value.span, message) {
                    diag.labels.push(cpl_diag::Label {
                        span: first_span,
                        message: "also lent here".into(),
                    });
                    diag.help = Some(help);
                }
            }
        }
    }

    // --- lends bound by `let` ------------------------------------------------

    /// Brings the effect of an earlier top-level statement into the item
    /// about to be checked, `current`.
    pub(crate) fn replay(&mut self, replay: &Replay, current: &[Stmt]) {
        let Some(stmt) = replay.item.parsed.stmts.first() else {
            return;
        };
        match (&replay.item.flow, &stmt.kind) {
            (Flow::Assert(_), StmtKind::Assert { cond, .. }) => self.add_facts(cond, false),
            (
                Flow::Known(name),
                StmtKind::Let {
                    init: Some(init), ..
                },
            ) => {
                let (range, walk_only) = self.quietly(|c| {
                    c.infer(init);
                    (c.const_range(init), c.walk_only_expr(init))
                });
                self.lends.view_exprs.clear();
                self.lends.view_readonly.clear();
                self.lends.proj_calls.clear();
                self.lends.covered.clear();
                if let Some(range) = range {
                    self.let_ranges.insert(name.clone(), range);
                }
                if walk_only {
                    self.walk_only.insert(name.clone());
                }
            }
            (Flow::Move(srcs), _) => {
                for (mover, moved) in nested_moves(stmt) {
                    let moved: Vec<&String> = moved.iter().filter(|m| srcs.contains(m)).collect();
                    self.replay_move(&mover, &moved, replay.back);
                }
            }
            (
                Flow::Capture(name),
                StmtKind::Let {
                    init: Some(init), ..
                },
            ) => {
                let until = match current.iter().rev().find(|s| mentions(s, name)) {
                    _ if replay.live_after => ELSEWHERE,
                    Some(last) => last.span,
                    None => return,
                };
                for (root, steps, is_capture) in self.captured_places(init) {
                    self.lends.holds.push(Hold {
                        name: name.clone(),
                        root,
                        steps,
                        mutable: false,
                        at: ELSEWHERE,
                        until,
                        in_loop: false,
                        is_capture,
                    });
                }
            }
            (
                Flow::LentVar(name),
                StmtKind::Let {
                    init: Some(value), ..
                }
                | StmtKind::Assign { value, .. },
            ) => {
                let found: Vec<_> = self.quietly(|c| {
                    c.infer(value);
                    c.extra_holds(value)
                });
                self.lends.view_exprs.clear();
                self.lends.view_readonly.clear();
                self.lends.proj_calls.clear();
                self.lends.covered.clear();
                let until = match current.iter().rev().find(|s| mentions(s, name)) {
                    _ if replay.live_after => ELSEWHERE,
                    Some(last) => last.span,
                    None => return,
                };
                for (_, place, _, force_mut) in found.into_iter().filter(|h| h.0 == "lent view") {
                    self.lends.holds.push(Hold {
                        name: name.clone(),
                        root: place.root,
                        steps: place.steps,
                        mutable: force_mut,
                        at: ELSEWHERE,
                        until,
                        in_loop: false,
                        is_capture: false,
                    });
                }
            }
            (
                Flow::View(name),
                StmtKind::Let {
                    init: Some(init), ..
                },
            ) => {
                // The statement was checked, and reported, as its own item.
                let (found, keyed) = self.quietly(|c| {
                    c.infer(init);
                    let ro = c.is_readonly_view(init);
                    let keyed = c.keyed_or_rev(init);
                    let found = if let Some((k, p)) = c.view_kind(init).zip(c.place(init)) {
                        vec![(k, p, ro, false)]
                    } else {
                        c.extra_holds(init)
                    };
                    (found, keyed)
                });
                match keyed {
                    Some(keyed) => self.lends.global_keyed.insert(name.clone(), keyed),
                    None => self.lends.global_keyed.remove(name),
                };
                // Its spans are positions in another item's text.
                self.lends.view_exprs.clear();
                self.lends.view_readonly.clear();
                self.lends.proj_calls.clear();
                self.lends.covered.clear();
                if found.is_empty() {
                    return;
                }
                if found.len() == 1 {
                    let (kind, ref place, read_only, _) = found[0];
                    self.lends.global_views.insert(
                        name.clone(),
                        Lent {
                            kind,
                            root: place.root.clone(),
                            root_mutable: place.root_mutable,
                            read_only,
                            steps: place.steps.clone(),
                        },
                    );
                }
                let until = match current.iter().rev().find(|s| mentions(s, name)) {
                    _ if replay.live_after => ELSEWHERE,
                    Some(last) => last.span,
                    None => return,
                };
                for (_, place, _, force_mut) in found {
                    self.lends.holds.push(Hold {
                        name: name.clone(),
                        root: place.root,
                        steps: place.steps,
                        mutable: force_mut || replay.written,
                        at: ELSEWHERE,
                        until,
                        in_loop: false,
                        is_capture: false,
                    });
                }
            }
            _ => {}
        }
    }

    /// Generated struct constructors move owned heap arguments (R2, R02.6.3).
    pub(crate) fn move_ctor_args(&mut self, args: &[Arg]) {
        for arg in args {
            if arg.mutable {
                continue;
            }
            if let ExprKind::Name(src) = &unparen(&arg.value).kind {
                if !self.is_owned_movable_var(src) {
                    continue;
                }
                let ty = self.local(src).map(|l| l.ty.clone()).or_else(|| {
                    match self.global(src).as_deref() {
                        Some(Def::Value { ty, .. }) => Some(ty.clone()),
                        _ => None,
                    }
                });
                if let Some(ty) = ty {
                    if self.is_heap_type(&ty) {
                        self.mark_moved(src, arg.value.span);
                    }
                }
            }
        }
    }

    /// True when `name` is an owned variable (not a parameter or lent view)
    /// that can be moved out of.
    pub(crate) fn is_owned_movable_var(&mut self, name: &str) -> bool {
        if let Some(local) = self.local(name) {
            local.keyword != "param" && !self.lends.views.contains_key(&local.span)
        } else if self.in_fn == 0 {
            self.global_is(name, |d| matches!(d, Def::Value { .. }))
                && !self.lends.global_views.contains_key(name)
        } else {
            false
        }
    }

    /// Records `src` moved at `span`; rejects it while captured (E0607) or lent (E0602).
    pub(crate) fn mark_moved(&mut self, src: &str, span: Span) {
        if let Some(hold) = self.lends.holds.iter().find(|h| h.root == src) {
            let lent_to = hold.name.clone();
            let is_capture = hold.is_capture;
            let at = hold.at;
            let until = hold.until;
            if is_capture {
                if let Some(diag) = self.report(
                    span,
                    msgs::check::MOVED_WHILE_CAPTURED_CAN_T_BE_MOVED,
                    &[&src, &lent_to],
                ) {
                    if at != ELSEWHERE {
                        diag.labels.push(cpl_diag::Label {
                            span: at,
                            message: format!("captured by `{lent_to}` here"),
                        });
                    }
                    if until != ELSEWHERE {
                        diag.labels.push(cpl_diag::Label {
                            span: until,
                            message: format!("`{lent_to}` is last used here"),
                        });
                    }
                    diag.help = Some(format!(
                        "clone it (`{src}.clone()`) or finish using `{lent_to}` before moving `{src}`"
                    ));
                    diag.fix(format!("clone `{src}`"), Span::point(span.end), ".clone()");
                }
            } else {
                if let Some(diag) = self.report(
                    span,
                    msgs::check::USED_WHILE_LENT_CAN_T_BE_MOVED,
                    &[&src, &lent_to],
                ) {
                    if at != ELSEWHERE {
                        diag.labels.push(cpl_diag::Label {
                            span: at,
                            message: format!("lent to `{lent_to}` here"),
                        });
                    }
                    if until != ELSEWHERE {
                        diag.labels.push(cpl_diag::Label {
                            span: until,
                            message: format!("`{lent_to}` is last used here"),
                        });
                    }
                    diag.help = Some(format!(
                        "finish with `{lent_to}` first, or clone `{src}` with `{src}.clone()`"
                    ));
                }
            }
        }
        self.moved.insert(src.to_string(), span);
    }

    pub(crate) fn note_view(&mut self, e: &Expr, kind: &'static str) {
        self.lends.view_exprs.insert(e.span, kind);
    }

    pub(crate) fn note_readonly_view(&mut self, e: &Expr, kind: &'static str) {
        self.lends.view_exprs.insert(e.span, kind);
        self.lends.view_readonly.insert(e.span);
    }

    pub(crate) fn note_proj_call(&mut self, e: &Expr, call: ProjCall) {
        self.lends.proj_calls.insert(e.span, call);
    }

    /// `let name = <view>`: the root stays lent until the last statement of `rest` naming `name`.
    /// Additional lends introduced by `value` beyond slice/projection views:
    /// - `mut a[i]` (R05.3.4 / R05.8.4); `rows[0]` of a heap type (R05.3.5)
    /// - `xs.filter(...)` / `.map(...)` / `.take(...)` / `.drop(...)` view adapters (R12.4.9)
    /// - generator calls `evens(a)` / `mark(mut seen)` (R05.7.3 / R05.9.1); `Holder(xs)` (R05.7.2)
    fn extra_holds(&mut self, value: &Expr) -> Vec<(&'static str, Place, bool, bool)> {
        let e = unparen(value);
        match &e.kind {
            // `b.cur(0)!` lends what the optional lends.
            ExprKind::Unwrap(inner) => self.extra_holds(inner),
            // `o!` or `let p = o` for a lent `o` holds `o`'s source.
            ExprKind::Name(name) => {
                let lent = match self.local(name) {
                    Some(l) => self.lends.views.get(&l.span).cloned(),
                    None => self.lends.global_views.get(name).cloned(),
                }
                .filter(|lent| lent.kind == "lent view");
                match (lent, self.place(e)) {
                    (Some(lent), Some(place)) => {
                        vec![("lent view", place, lent.read_only, !lent.read_only)]
                    }
                    _ => Vec::new(),
                }
            }
            ExprKind::Mut(inner) => {
                if let Some(mut place) = self.place(inner) {
                    place.steps.clear();
                    return vec![("mutable reference", place, false, true)];
                }
                Vec::new()
            }
            ExprKind::Index { base, indices }
                if !indices
                    .iter()
                    .any(|i| matches!(unparen(i).kind, ExprKind::Range { .. })) =>
            {
                // An element of a lent view (`g.rows[r]`), not of an element, carries its lend.
                let ty = self.infer_quietly(e);
                let lent = self.extra_holds(base);
                if !ty.is_plain_value() && lent.first().is_some_and(|l| l.0 != "element") {
                    return lent;
                }
                if self.is_heap_type(&ty) {
                    if let Some(place) = self.place(e) {
                        return vec![("element", place, true, false)];
                    }
                }
                Vec::new()
            }
            ExprKind::Field { base, name } if self.lends_getter(name) => self.receiver_hold(base),
            ExprKind::Call { callee, args } => {
                if let ExprKind::Field { base, name } = &unparen(callee).kind {
                    if matches!(name.text.as_str(), "filter" | "take" | "drop") {
                        let base_ty = self.infer_quietly(base);
                        if base_ty.is_seq() {
                            if let Some(mut place) = self.place(base) {
                                place.steps.clear();
                                return vec![("view", place, true, false)];
                            }
                        }
                    }
                }
                // `b.range(r)` on a projection returning a view holds `b`.
                if let ExprKind::Field { base, name } = &unparen(callee).kind {
                    let projection = match self.infer_quietly(base) {
                        Ty::Named {
                            name: type_name, ..
                        } => self
                            .members_of(&type_name)
                            .and_then(|m| m.projections.into_iter().find(|p| p.name == name.text)),
                        _ => None,
                    };
                    let ty = match self.infer_quietly(e) {
                        // `lends ?T` lends what its `Some` holds.
                        Ty::Optional(inner) => *inner,
                        ty => ty,
                    };
                    let is_struct = matches!(&ty, Ty::Named { name, .. }
                        if matches!(self.type_def(name).as_deref(), Some(Def::Struct { .. })));
                    if let Some(projection) =
                        projection.filter(|_| is_struct || self.is_heap_type(&ty))
                    {
                        if let Some(mut place) = self.place(base) {
                            place.steps.clear();
                            let mutable = projection.mutable;
                            let mut out = vec![("lent view", place, !mutable, mutable)];
                            for (arg, source) in args.iter().zip(&projection.sources) {
                                let arg_ty = self.infer_quietly(&arg.value);
                                let lent = arg.mutable || self.is_heap_type(&arg_ty);
                                if let (true, Some(_)) = (lent, source) {
                                    if let Some(mut place) = self.place(&arg.value) {
                                        place.steps.clear();
                                        out.push(("lent view", place, !arg.mutable, arg.mutable));
                                    }
                                }
                            }
                            return out;
                        }
                    }
                }
                // `lends` fn call: holds all `mut` or heap-typed args.
                let is_lends_fn = match &unparen(callee).kind {
                    ExprKind::Name(n) => {
                        self.global_is(n, |d| matches!(d, Def::Fn { lends: true, .. }))
                    }
                    ExprKind::Generic { base, .. } => match &unparen(base).kind {
                        ExprKind::Name(n) => {
                            self.global_is(n, |d| matches!(d, Def::Fn { lends: true, .. }))
                        }
                        _ => false,
                    },
                    _ => false,
                };
                if is_lends_fn {
                    let mut out = Vec::new();
                    for arg in args {
                        let is_mut = arg.mutable;
                        let arg_ty = self.infer_quietly(&arg.value);
                        if is_mut || self.is_heap_type(&arg_ty) {
                            if let Some(mut place) = self.place(&arg.value) {
                                place.steps.clear();
                                out.push(("lent view", place, !is_mut, is_mut));
                            }
                        }
                    }
                    if !out.is_empty() {
                        return out;
                    }
                }
                // Generator call: holds all non-copy or `mut` args.
                let ret_ty = self.infer_quietly(e);
                if matches!(&ret_ty, Ty::Named { name, .. } if name == "Gen") {
                    let mut out = Vec::new();
                    for arg in args {
                        let is_mut = arg.mutable;
                        let arg_ty = self.infer_quietly(&arg.value);
                        if is_mut || self.is_heap_type(&arg_ty) {
                            if let Some(mut place) = self.place(&arg.value) {
                                place.steps.clear();
                                out.push(("generator argument", place, !is_mut, is_mut));
                            }
                        }
                    }
                    return out;
                }
                // Struct constructor given a lent parameter (`Holder(xs)`).
                let is_struct = self.constructs_struct(callee);
                if is_struct {
                    let mut out = Vec::new();
                    for arg in args {
                        if let ExprKind::Name(arg_name) = &unparen(&arg.value).kind {
                            if let Some(local) = self.local(arg_name).cloned() {
                                if local.keyword == "param" && self.is_heap_type(&local.ty) {
                                    if let Some(mut place) = self.place(&arg.value) {
                                        place.steps.clear();
                                        out.push(("lent field", place, true, false));
                                    }
                                }
                            }
                        }
                    }
                    return out;
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// `var c = b.cur(0)` or `c = b.next(c)`: a var holding a lent value holds that
    /// value's source until the var's last use, as a `let` does; each assignment
    /// takes the loan of the value it stores.
    pub(crate) fn begin_var_hold(&mut self, stmt: &Stmt, name: &str, value: &Expr, rest: &[Stmt]) {
        let lent: Vec<_> = self
            .extra_holds(value)
            .into_iter()
            .filter(|hold| hold.0 == "lent view")
            .collect();
        if lent.is_empty() {
            return;
        }
        let Some(last) = rest.iter().rev().find(|s| mentions(s, name)) else {
            return;
        };
        for (_, place, _, force_mut) in lent {
            self.lends.holds.push(Hold {
                name: name.to_string(),
                root: place.root,
                steps: place.steps,
                mutable: force_mut,
                at: stmt.span,
                until: last.span,
                in_loop: false,
                is_capture: false,
            });
        }
    }

    /// `if let d = b.cur(0):` / `while let`: the bound name holds what the value
    /// lends while the body uses it.
    pub(crate) fn begin_cond_hold(&mut self, cond: &Cond, body: &Block) {
        let Cond::Let {
            pat:
                ast::Pattern {
                    kind: ast::PatternKind::Name(name),
                    ..
                },
            value,
            ..
        } = cond
        else {
            return;
        };
        let lent: Vec<_> = self
            .extra_holds(value)
            .into_iter()
            .filter(|hold| hold.0 == "lent view")
            .collect();
        let Some(last) = body.stmts.iter().rev().find(|s| mentions(s, name)) else {
            return;
        };
        for (_, place, _, force_mut) in lent {
            self.lends.holds.push(Hold {
                name: name.clone(),
                root: place.root,
                steps: place.steps,
                mutable: force_mut,
                at: value.span,
                until: last.span,
                in_loop: false,
                is_capture: false,
            });
        }
    }

    pub(crate) fn begin_hold(&mut self, stmt: &Stmt, name: &str, value: &Expr, rest: &[Stmt]) {
        if self.note_closure(name, value) {
            let Some(last) = rest.iter().rev().find(|s| mentions(s, name)) else {
                return;
            };
            for (root, steps, is_capture) in self.captured_places(value) {
                self.lends.holds.push(Hold {
                    name: name.to_string(),
                    root,
                    steps,
                    mutable: false,
                    at: stmt.span,
                    until: last.span,
                    in_loop: false,
                    is_capture,
                });
            }
            return;
        }
        let Some(kind) = self.view_kind(value) else {
            let extras = self.extra_holds(value);
            if extras.is_empty() {
                return;
            }
            let Some(last) = rest.iter().rev().find(|s| mentions(s, name)) else {
                return;
            };
            if extras.len() == 1 {
                if let Some(decl) = self.local(name).map(|local| local.span) {
                    let (kind, ref place, read_only, _) = extras[0];
                    self.lends.views.insert(
                        decl,
                        Lent {
                            kind,
                            root: place.root.clone(),
                            root_mutable: place.root_mutable,
                            read_only,
                            steps: place.steps.clone(),
                        },
                    );
                }
            }
            for (_, place, _, force_mut) in extras {
                self.lends.holds.push(Hold {
                    name: name.to_string(),
                    root: place.root,
                    steps: place.steps,
                    mutable: force_mut,
                    at: stmt.span,
                    until: last.span,
                    in_loop: false,
                    is_capture: false,
                });
            }
            return;
        };
        if let Some(keyed) = self.keyed_or_rev(value) {
            if let Some(decl) = self.local(name).map(|local| local.span) {
                self.lends.keyed.insert(decl, keyed);
            }
        }
        let Some(place) = self.place(value) else {
            return;
        };
        let Some(decl) = self.local(name).map(|local| local.span) else {
            return;
        };
        let read_only = self.is_readonly_view(value);
        self.lends.views.insert(
            decl,
            Lent {
                kind,
                root: place.root.clone(),
                root_mutable: place.root_mutable,
                read_only,
                steps: place.steps.clone(),
            },
        );
        let Some(last) = rest.iter().rev().find(|s| mentions(s, name)) else {
            return;
        };
        let mutable = written_in_stmts(rest).iter().any(|n| n == name);
        if mutable && read_only {
            if let Some(diag) = self.report(
                stmt.span,
                msgs::check::ASSIGN_TO_LET_IS_A_READ_ONLY_3,
                &[&name, &kind, &place.root],
            ) {
                diag.help = Some(
                    "copy it into an owned value first with `.to_str()` or `.to_list()`".into(),
                );
            }
        } else if mutable && !place.root_mutable {
            if let Some(diag) = self.report(
                stmt.span,
                msgs::check::ASSIGN_TO_LET_IS_CHANGED_BELOW_BUT,
                &[&name, &place.root],
            ) {
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
            in_loop: false,
            is_capture: false,
        });
    }

    /// `for x in xs` lends `xs` for reading until the loop ends: growing
    /// or shrinking it would move the storage the loop is walking, and a
    /// loop whose range changes under it is a puzzle even when it is
    /// memory-safe. Looping over a range, a domain or `xs.len` lends
    /// nothing: the loop walks values, not the storage.
    pub(crate) fn hold_iterated(&mut self, stmt: &Stmt, iter: &Expr, iter_ty: &Ty) {
        let mut iter = unparen(iter);
        loop {
            iter = match &iter.kind {
                ExprKind::Field { base, name } if name.text == "rev" => unparen(base),
                ExprKind::Call { callee, args } if args.is_empty() => match &callee.kind {
                    ExprKind::Field { base, name } if name.text == "pairs" => unparen(base),
                    _ => break,
                },
                _ => break,
            };
        }
        self.add_loop_facts(stmt);
        if self.hold_walk(stmt, iter) {
            return;
        }
        if let ExprKind::Field { base, name } = &iter.kind {
            if name.text == "keys" {
                let base_ty = self.infer_quietly(base);
                if matches!(base_ty, Ty::Array { .. }) {
                    return;
                }
            }
        }
        let stored = match iter_ty {
            Ty::Array { .. } => true,
            Ty::Named { name, .. } => matches!(
                name.as_str(),
                "List" | "Set" | "HashSet" | "Map" | "HashMap"
            ),
            _ => false,
        };
        if !stored && self.view_kind(iter).is_none() {
            return;
        }
        let Some(place) = self.place(iter) else {
            return;
        };
        if !place.root_mutable {
            return;
        }
        self.lends.holds.push(Hold {
            name: String::new(),
            root: place.root,
            steps: place.steps,
            mutable: false,
            at: iter.span,
            until: stmt.span,
            in_loop: true,
            is_capture: false,
        });
    }

    /// `a[i] += f(mut a[i])`: the right side runs first and changes the
    /// very place the statement then updates.
    pub(crate) fn assign_alias(&mut self, target: &Expr, op: ast::AssignOp, value: &Expr) {
        // `x = f(mut x)` only overwrites `x` afterwards.
        if op == ast::AssignOp::Set && matches!(unparen(target).kind, ExprKind::Name(_)) {
            return;
        }
        struct Changed(Vec<Expr>, Vec<Expr>);
        impl Visitor for Changed {
            fn expr(&mut self, e: &Expr) {
                let ExprKind::Call { callee, args } = &e.kind else {
                    return;
                };
                self.0
                    .extend(args.iter().filter(|a| a.mutable).map(|a| a.value.clone()));
                if let ExprKind::Field { name, .. } = &callee.kind {
                    if mutating_call(&name.text, args) {
                        self.1.push(e.clone());
                    }
                }
            }
        }
        let mut changed = Changed(Vec::new(), Vec::new());
        walk_expr(value, &mut changed);
        for call in std::mem::take(&mut changed.1) {
            let ExprKind::Call { callee, args } = &call.kind else {
                continue;
            };
            let ExprKind::Field { base, name } = &callee.kind else {
                continue;
            };
            let base_ty = self.infer_quietly(base);
            if self.row_mutator(&base_ty, &name.text, args) {
                changed.0.push((**base).clone());
            }
        }
        if changed.0.is_empty() {
            return;
        }
        let Some(target_place) = self.place(target) else {
            return;
        };
        for place in changed.0 {
            let Some(changed_place) = self.place(&place) else {
                continue;
            };
            if changed_place.root != target_place.root {
                continue;
            }
            let Some(why) = self.overlap(&target_place.steps, &changed_place.steps) else {
                continue;
            };
            let parts = target_place.steps.len().cmp(&changed_place.steps.len());
            self.sink.push(super::advice::same_place(
                &place, target, value, op, parts, why,
            ));
        }
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
            if hold.is_capture {
                continue;
            }
            if hold.root != place.root || place.via.as_deref() == Some(hold.name.as_str()) {
                continue;
            }
            if !(write || hold.mutable) {
                continue;
            }
            let (steps, name, mutable, at, until, in_loop) = (
                hold.steps.clone(),
                hold.name.clone(),
                hold.mutable,
                hold.at,
                hold.until,
                hold.in_loop,
            );
            let Some(why) = self.overlap(&place.steps, &steps) else {
                continue;
            };
            if self
                .lends
                .conflicts
                .iter()
                .any(|s| s.contains(e.span) || e.span.contains(*s))
            {
                return;
            }
            self.lends.conflicts.push(e.span);
            if in_loop {
                let root = place.root;
                if let Some(diag) = self.report(
                    e.span,
                    msgs::check::USED_WHILE_LENT_CAN_T_BE_HERE,
                    &[&show(e), &if write { "changed" } else { "used" }, &root],
                ) {
                    diag.labels.push(cpl_diag::Label {
                        span: at,
                        message: format!("the loop goes over `{root}`"),
                    });
                    diag.help = Some(format!(
                        "loop over positions instead (`for i in 0..{root}.len`), or over a copy (`for x in {root}.clone()`)"
                    ));
                }
                return;
            }
            if let Some(diag) = self.report(
                e.span,
                msgs::check::USED_WHILE_LENT_CAN_T_BE_HERE_2,
                &[
                    &show(e),
                    &if write { "changed" } else { "used" },
                    &place.root,
                    &if mutable { "mutably " } else { "" },
                    &name,
                    &why,
                ],
            ) {
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
                    "finish with `{name}` first, go through `{name}`, or keep the position and re-project later; `.clone()` keeps an owned copy"
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

    // --- writes through projections -----------------------------------------

    // --- projections ---------------------------------------------------------
}
