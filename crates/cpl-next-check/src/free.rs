//! What a nested fn reads from around it, on the typed tree.
use super::*;
use std::collections::BTreeSet;

/// What each nested fn a block binds reads from around it, together with what its siblings do:
/// a sibling's value is rebuilt from the same captures (R05.10.5).
pub fn group_free(lambdas: &[LambdaInfo], ss: &[TStmt]) -> HashMap<LambdaId, BTreeSet<VarRef>> {
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
            if lambdas[id.0].group.is_empty() {
                continue;
            }
            let (mut used, mut bound) = (BTreeSet::new(), BTreeSet::new());
            free_vars(body, &mut used, &mut bound);
            for p in params {
                bound.insert(VarRef::Local(*p));
                bound.insert(VarRef::Global(*p));
            }
            own.insert(
                *id,
                used.difference(&bound).copied().collect::<BTreeSet<_>>(),
            );
        }
    }
    (own.keys())
        .map(|id| {
            let all = (lambdas[id.0].group.iter())
                .filter_map(|s| own.get(s))
                .flatten()
                .copied()
                .collect();
            (*id, all)
        })
        .collect()
}

pub fn free_vars(e: &TExpr, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>) {
    let arg = |a: &TArg, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>| match a {
        TArg::Value(v) | TArg::Mut(v) => free_vars(v, used, bound),
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
        | TK::DomOrd(b)
        | TK::DomFrom(_, b, _)
        | TK::CutOrd(_, b)
        | TK::DomNth(_, b)
        | TK::DomHas(_, b) => free_vars(b, used, bound),
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
            free_vars(callee, used, bound);
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::Struct(fs) => fs.iter().for_each(|a| free_vars(a, used, bound)),
        TK::Lambda { params, body, .. } => {
            free_vars(body, used, bound);
            for p in params {
                bound.insert(VarRef::Local(*p));
                bound.insert(VarRef::Global(*p));
            }
        }
        TK::Bin(_, a, b, _) | TK::Cmp(_, a, b) | TK::Slot(a, b) => {
            free_vars(a, used, bound);
            free_vars(b, used, bound);
        }
        TK::If(a, b, c) => {
            free_vars(a, used, bound);
            free_vars(b, used, bound);
            free_vars(c, used, bound);
        }
        TK::Block(ss, v) => {
            for s in ss {
                match s {
                    TStmt::Let(v, e) => {
                        bound.insert(*v);
                        free_vars(e, used, bound);
                    }
                    TStmt::Expr(e) | TStmt::Assert(e, ..) => free_vars(e, used, bound),
                    TStmt::Assign(a, b) | TStmt::While(a, b) => {
                        free_vars(a, used, bound);
                        free_vars(b, used, bound);
                    }
                    TStmt::For(v, a, b, c) => {
                        bound.insert(*v);
                        free_vars(a, used, bound);
                        free_vars(b, used, bound);
                        free_vars(c, used, bound);
                    }
                    TStmt::Return(Some(e)) | TStmt::Yield(e) | TStmt::LendOut(e) => {
                        free_vars(e, used, bound)
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
                            free_vars(c, used, bound);
                        }
                        if let Some(r) = recv {
                            arg(r, used, bound);
                        }
                        args.iter().for_each(|a| arg(a, used, bound));
                        free_vars(body, used, bound);
                    }
                    _ => {}
                }
            }
            if let Some(v) = v {
                free_vars(v, used, bound);
            }
        }
        _ => {}
    }
}
