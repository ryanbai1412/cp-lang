//! What one test case under `@!multitest` may leave for the next (R10.1.1).
use super::free::free_vars;
use super::*;
use std::collections::BTreeSet;

/// An E0805: `name`, declared above `@!multitest`, holds a `dom` the case declares, first used
/// by the case at `span` in `item`.
pub(super) struct Leak {
    pub item: ItemId,
    pub span: Span,
    pub name: String,
    pub dom: String,
}

/// E0805: each name `above` (bound before `@!multitest`, with its type) whose type holds a
/// domain the case declares, one of `doms`, would carry that case's positions into the next
/// (R10.1.1). Each is said where the case `body` first uses it.
pub(super) fn per_test_leaks(
    above: &[(VarRef, String, Ty)],
    doms: &[(DomId, String)],
    body: &[TStmt],
) -> Vec<Leak> {
    let mut out = Vec::new();
    for (v, name, ty) in above {
        let mut held = Vec::new();
        domain::doms_in(ty, &mut held);
        let Some((_, dom)) = doms.iter().find(|(d, _)| held.contains(d)) else {
            continue;
        };
        if let Some((item, span)) = first_use(body, *v) {
            let (name, dom) = (name.clone(), dom.clone());
            out.push(Leak {
                item,
                span,
                name,
                dom,
            });
        }
    }
    out
}

/// The item and span of the first statement of `body` that uses `v`: an assignment's value, else
/// the statement's first expression.
fn first_use(body: &[TStmt], v: VarRef) -> Option<(ItemId, Span)> {
    let mut item = None;
    for s in body {
        let es: Vec<&TExpr> = match s {
            TStmt::Item(i) => {
                item = Some(i.clone());
                continue;
            }
            TStmt::Assign(place, value) => vec![value, place],
            TStmt::Let(_, e)
            | TStmt::Expr(e)
            | TStmt::Assert(e, ..)
            | TStmt::Return(Some(e))
            | TStmt::Yield(e)
            | TStmt::LendOut(e) => vec![e],
            TStmt::While(a, b) => vec![a, b],
            TStmt::For(_, a, b, c) => vec![a, b, c],
            TStmt::Region { body, .. } => vec![body],
            _ => Vec::new(),
        };
        let uses = es.iter().any(|e| {
            let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
            free_vars(e, true, &mut used, &mut bound);
            used.contains(&v)
        });
        if let (true, Some(e)) = (uses, es.first()) {
            return Some((item.clone()?, e.span));
        }
    }
    None
}
