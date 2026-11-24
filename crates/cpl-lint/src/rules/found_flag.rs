//! `found_flag_pair` (L0596): a `var found = false` that is only ever set to
//! `true` beside an assignment to another `var`, which is set nowhere else.
//! The flag says whether the value holds anything yet; `?T` starting at
//! `None` says it in the value's own type, and the two can't disagree.

use crate::walk::{Scope, Visitor};
use crate::{File, Finding, FOUND_FLAG_PAIR};
use cpl_check::Ty;
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::{AssignOp, Expr, ExprKind, PatternKind, Stmt, StmtKind};

#[derive(Default)]
struct Flags {
    base: u32,
    /// `var f = false`: name, its span, and the statement's start.
    flags: Vec<(String, Span, u32)>,
    /// The statement start and name span of each flag with a written type:
    /// the checker's facts say whether it is `Bool`.
    written: Vec<(u32, Span)>,
    /// Other `var`s started at a placeholder: name, the declared type's
    /// span, whether the start is an integer literal, and the statement's
    /// start.
    vars: Vec<(String, Option<Span>, bool, u32)>,
    /// Every assignment to a name: the name, whether it is `= true`, and
    /// the span of the statement.
    sets: Vec<(String, bool, Span)>,
    /// `x = ...` beside `f = true` in one block: the flag and its set's
    /// span, the value and its set's span.
    beside: Vec<(String, Span, String, Span)>,
    /// How often each name is mentioned at all.
    names: Vec<String>,
    /// The scope each `var` is declared in, by its statement's start.
    scopes: Vec<(u32, usize)>,
    open: Vec<usize>,
    opened: usize,
}

/// A start value that only fills the slot: a literal, or a tuple or record
/// of them.
fn placeholder(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Char(_) | ExprKind::Bool(_) => true,
        ExprKind::Str(_) => true,
        ExprKind::Paren(inner) => placeholder(inner),
        ExprKind::Unary { expr, .. } => placeholder(expr),
        ExprKind::Tuple(items) => items.iter().all(placeholder),
        ExprKind::Record(fields) => fields.iter().all(|(_, e)| placeholder(e)),
        _ => false,
    }
}

fn plain_set(stmt: &Stmt) -> bool {
    matches!(
        &stmt.kind,
        StmtKind::Assign {
            op: AssignOp::Set,
            ..
        }
    )
}

fn set_name(stmt: &Stmt) -> Option<(&str, bool)> {
    let StmtKind::Assign { target, op, value } = &stmt.kind else {
        return None;
    };
    let ExprKind::Name(name) = &target.kind else {
        return None;
    };
    let truth = matches!(op, AssignOp::Set) && matches!(value.kind, ExprKind::Bool(true));
    Some((name, truth))
}

impl Visitor for Flags {
    fn enter(&mut self, _: Scope) {
        self.opened += 1;
        self.open.push(self.opened);
    }

    fn exit(&mut self, _: Scope) {
        self.open.pop();
    }

    fn stmt(&mut self, stmt: &Stmt) {
        if matches!(stmt.kind, StmtKind::Let { mutable: true, .. }) {
            let scope = self.open.last().copied().unwrap_or(0);
            self.scopes.push((stmt.span.start + self.base, scope));
        }
        match &stmt.kind {
            StmtKind::Let {
                mutable: true,
                pat,
                ty,
                init: Some(init),
                ..
            } => {
                let PatternKind::Name(name) = &pat.kind else {
                    return;
                };
                if matches!(init.kind, ExprKind::Bool(false)) {
                    let decl = Span::new(stmt.span.start, init.span.end);
                    if ty.is_some() {
                        self.written
                            .push((stmt.span.start + self.base, pat.span.shift(self.base)));
                    }
                    self.flags.push((
                        name.clone(),
                        decl.shift(self.base),
                        stmt.span.start + self.base,
                    ));
                } else if placeholder(init) {
                    self.vars.push((
                        name.clone(),
                        ty.as_ref().map(|t| t.span.shift(self.base)),
                        matches!(init.kind, ExprKind::Int(_)),
                        stmt.span.start + self.base,
                    ));
                }
            }
            StmtKind::Assign { .. } => {
                let (name, truth) = set_name(stmt).unwrap_or(("", false));
                if !name.is_empty() {
                    self.sets
                        .push((name.to_string(), truth, stmt.span.shift(self.base)));
                }
            }
            _ => {}
        }
    }

    fn stmts(&mut self, stmts: &[Stmt]) {
        for flag in stmts {
            let Some((f, true)) = set_name(flag) else {
                continue;
            };
            for other in stmts {
                if let Some((name, _)) =
                    set_name(other).filter(|(n, _)| *n != f && plain_set(other))
                {
                    self.beside.push((
                        f.to_string(),
                        flag.span.shift(self.base),
                        name.to_string(),
                        other.span.shift(self.base),
                    ));
                }
            }
        }
    }

    fn expr(&mut self, expr: &Expr) {
        if let ExprKind::Name(name) = &expr.kind {
            self.names.push(name.clone());
        }
    }
}

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = Flags::default();
    file.walk(&mut v, |v, base| v.base = base);
    let mut out = Vec::new();
    let scope = |at: u32| v.scopes.iter().find(|(s, _)| *s == at).map(|(_, b)| *b);
    for (flag, span, decl) in &v.flags {
        let written = v.written.iter().find(|(at, _)| at == decl);
        let bool_ty = written.is_none_or(|(_, name)| {
            file.facts
                .is_some_and(|f| matches!(f.place_type(*name), Some(Ty::Bool)))
        });
        if !bool_ty {
            continue;
        }
        let sets: Vec<_> = v.sets.iter().filter(|(n, ..)| n == flag).collect();
        if sets.is_empty() || sets.iter().any(|(_, truth, _)| !truth) {
            continue;
        }
        let mentions = v.names.iter().filter(|n| *n == flag).count();
        if mentions <= sets.len() {
            continue;
        }
        // The values: declared beside the flag, set beside every
        // `flag = true`, and nowhere else.
        let beside: Vec<_> = v.beside.iter().filter(|b| &b.0 == flag).collect();
        let mut values: Vec<(&str, Span, String)> = Vec::new();
        for (_, _, value, at) in &beside {
            let decls: Vec<_> = v.vars.iter().filter(|(n, ..)| n == value).collect();
            let [(_, ty, int, vdecl)] = decls[..] else {
                continue;
            };
            let paired = scope(*vdecl) == scope(*decl)
                && v.flags.iter().all(|(n, ..)| n != value)
                && sets
                    .iter()
                    .all(|(_, _, f)| beside.iter().any(|b| b.1 == *f && &b.2 == value))
                && v.sets
                    .iter()
                    .filter(|(n, ..)| n == value)
                    .all(|(_, _, s)| beside.iter().any(|b| b.3 == *s));
            if paired && values.iter().all(|(n, ..)| n != value) {
                let ty = match (ty, int) {
                    (Some(t), _) => t.text(file.src).to_string(),
                    (None, true) => "Int".to_string(),
                    _ => "T".to_string(),
                };
                values.push((value, *at, ty));
            }
        }
        let Some(&(value, at, ref ty)) = values.first() else {
            continue;
        };
        let help = if values.len() == 1 {
            format!(
                "declare `var {value}: ?{ty} = None` and drop `{flag}`: test `{value} != None` (or `if let`) where `{flag}` is read, so the flag and the value can't disagree"
            )
        } else {
            let tys: Vec<_> = values.iter().map(|(n, _, t)| format!("{n}: {t}")).collect();
            format!(
                "keep them in one `var best: ?({}) = None` and drop `{flag}`: test `best != None` (or `if let`) where `{flag}` is read, so the flag and the values can't disagree",
                tys.join(", ")
            )
        };
        let shown = values
            .iter()
            .map(|(n, ..)| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let diag = Diagnostic::warning(
            FOUND_FLAG_PAIR.code,
            *span,
            format!(
                "`{flag}` only says whether {shown} {} been set",
                if values.len() == 1 { "has" } else { "have" }
            ),
        )
        .with_label(at, "set beside it")
        .with_help(help);
        out.push(Finding {
            lint: &FOUND_FLAG_PAIR,
            diag,
        });
    }
    out
}
