//! What a nested fn reads from around it, on the typed tree.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// What each nested fn a block binds reads from around it, together with what its siblings do:
/// a sibling's value is rebuilt from the same captures (R05.10.5).
pub fn group_free(
    lambdas: &BTreeMap<LambdaId, LambdaInfo>,
    top: bool,
    ss: &[TStmt],
) -> HashMap<LambdaId, BTreeSet<VarRef>> {
    let mut own = HashMap::new();
    for s in ss {
        if let TStmt::Let(
            _,
            TExpr {
                kind: TK::Lambda { id, params, body },
                ..
            },
        ) = s
        {
            if lambdas[id].group.is_empty() {
                continue;
            }
            let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
            free_vars(body, top, &mut used, &mut bound);
            bound.extend(params.iter().map(|&p| param_ref(top, p)));
            own.insert(
                *id,
                used.difference(&bound).copied().collect::<BTreeSet<_>>(),
            );
        }
    }
    (own.keys())
        .map(|id| {
            let all = (lambdas[id].group.iter())
                .filter_map(|s| own.get(s))
                .flatten()
                .copied()
                .collect();
            (*id, all)
        })
        .collect()
}

/// `top`: the code is top-level, where every binding, a lambda's parameters too, is a global.
pub fn free_vars(e: &TExpr, top: bool, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>) {
    let arg = |a: &TArg, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>| match a {
        TArg::Value(v) | TArg::Mut(v) => free_vars(v, top, used, bound),
    };
    match &e.kind {
        TK::Var(v) => {
            used.insert(*v);
        }
        TK::Field(b, _)
        | TK::Payload(b, _)
        | TK::Tag(b)
        | TK::Variant(_, b)
        | TK::Inject(b)
        | TK::Widen(b)
        | TK::Owned(b)
        | TK::Pack(b)
        | TK::Elem(b)
        | TK::Not(b)
        | TK::CharOf(b)
        | TK::CodeOf(b)
        | TK::Neg(b)
        | TK::ToU64(b)
        | TK::ToI128(b)
        | TK::DomOrd(b)
        | TK::DomFrom(_, b, _)
        | TK::CutOrd(_, b)
        | TK::DomNth(_, b)
        | TK::DomHas(_, b) => free_vars(b, top, used, bound),
        TK::Call { recv, args, .. } => {
            if let Some(r) = recv {
                arg(r, used, bound);
            }
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::CallTrait { recv, args, .. } => {
            if let Some(r) = recv {
                arg(r, used, bound);
            }
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::CallValue { callee, args } => {
            free_vars(callee, top, used, bound);
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::Struct(fs) => fs.iter().for_each(|a| free_vars(a, top, used, bound)),
        TK::Lambda { params, body, .. } => {
            free_vars(body, top, used, bound);
            bound.extend(params.iter().map(|&p| param_ref(top, p)));
        }
        TK::Bin(_, a, b) | TK::Cmp(_, a, b) | TK::Slot(a, b) => {
            free_vars(a, top, used, bound);
            free_vars(b, top, used, bound);
        }
        TK::If(a, b, c) => {
            free_vars(a, top, used, bound);
            free_vars(b, top, used, bound);
            free_vars(c, top, used, bound);
        }
        TK::Block(ss, v) => {
            for s in ss {
                match s {
                    TStmt::Let(v, e) => {
                        bound.insert(*v);
                        free_vars(e, top, used, bound);
                    }
                    TStmt::Expr(e) | TStmt::Assert(e, ..) => free_vars(e, top, used, bound),
                    TStmt::Assign(a, b) | TStmt::While(a, b) => {
                        free_vars(a, top, used, bound);
                        free_vars(b, top, used, bound);
                    }
                    TStmt::For(v, a, b, c) => {
                        bound.insert(*v);
                        free_vars(a, top, used, bound);
                        free_vars(b, top, used, bound);
                        free_vars(c, top, used, bound);
                    }
                    TStmt::Return(Some(e)) | TStmt::Yield(e) | TStmt::LendOut(e) => {
                        free_vars(e, top, used, bound)
                    }
                    TStmt::Region {
                        callee,
                        recv,
                        args,
                        var,
                        body,
                        ..
                    } => {
                        bound.insert(*var);
                        if let Callee::Value(c) = callee {
                            free_vars(c, top, used, bound);
                        }
                        if let Some(r) = recv {
                            arg(r, used, bound);
                        }
                        args.iter().for_each(|a| arg(a, used, bound));
                        free_vars(body, top, used, bound);
                    }
                    _ => {}
                }
            }
            if let Some(v) = v {
                free_vars(v, top, used, bound);
            }
        }
        _ => {}
    }
}

/// A lambda parameter's variable: a global at top level, a local in a fn.
fn param_ref(top: bool, p: usize) -> VarRef {
    match top {
        true => VarRef::Global(p),
        false => VarRef::Local(p),
    }
}
