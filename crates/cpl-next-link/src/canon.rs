//! Contract 6: the canonical form link hands every backend.
use super::*;

/// Contract 6, required rewrites: a call through a function value becomes a
/// branch over the closures it can be, and each call a direct call.
/// The canonical form of every instance, or what the link step left inconsistent. It also
/// decides which types share a representation (`Program::shares`).
/// `expand` is ownership's expansion of each call through a fn value into its member calls; it
/// returns how many it could not expand.
pub fn canonicalize(
    p: &mut Program,
    expand: &dyn Fn(&Program, &mut Vec<Stmt<CTy>>) -> usize,
) -> Result<Vec<Func<CTy>>, Vec<Msg>> {
    let mut bad = Vec::new();
    let funcs = p
        .insts
        .iter()
        .map(|f| {
            let mut f = f.clone();
            let body = std::mem::take(&mut f.body);
            f.body = canon_block(p, &mut f.locals, &mut bad, body);
            if expand(p, &mut f.body) > 0 {
                bad.push("a call through a fn value of a member the program lacks".into());
            }
            f
        })
        .collect::<Vec<_>>();
    if bad.is_empty() {
        p.shares = share::shares(p, &funcs);
        Ok(funcs)
    } else {
        Err(bad.into_iter().map(Msg::internal).collect())
    }
}

fn read(p: Place, ty: CTy) -> Expr<CTy> {
    Expr {
        site: 0,
        kind: EK::Copy(p),
        ty,
    }
}
fn canon_block(
    p: &Program,
    locals: &mut Vec<CTy>,
    bad: &mut Vec<String>,
    b: Vec<Stmt<CTy>>,
) -> Vec<Stmt<CTy>> {
    let mut out = Vec::new();
    for s in b {
        match s {
            Stmt::Let(l, e) => canon_store(locals, bad, Place::Local(l), Some(l), e, &mut out),
            Stmt::Assign(pl, e) => canon_store(locals, bad, pl, None, e, &mut out),
            Stmt::Expr(e) => match e.kind {
                EK::CallValue(..) => out.push(Stmt::Expr(canon_call(bad, e))),
                _ => out.push(Stmt::Expr(canon_expr(bad, e))),
            },
            Stmt::If(c, a, b2) => out.push(Stmt::If(
                canon_expr(bad, c),
                canon_block(p, locals, bad, a),
                canon_block(p, locals, bad, b2),
            )),
            Stmt::Loop(a, b2, s) => out.push(Stmt::Loop(
                canon_block(p, locals, bad, a),
                canon_block(p, locals, bad, b2),
                s,
            )),
            Stmt::Switch(c, arms) => out.push(Stmt::Switch(
                canon_expr(bad, c),
                arms.into_iter()
                    .map(|a| canon_block(p, locals, bad, a))
                    .collect(),
            )),
            Stmt::Return(e) => out.push(Stmt::Return(e.map(|e| canon_expr(bad, e)))),
            Stmt::Check {
                kind,
                cond,
                msg,
                site,
            } => out.push(Stmt::Check {
                kind,
                cond: canon_expr(bad, cond),
                msg,
                site,
            }),
            Stmt::Yield(e) => out.push(Stmt::Yield(canon_expr(bad, e))),
            // A walk of a generator value runs the closure it holds: the one of its set it is.
            Stmt::Region {
                target: Target::Value(c),
                args,
                var,
                lend,
                body,
                site,
                body_site,
            } => {
                let cty = c.ty.clone();
                locals.push(cty.clone());
                let tmp = locals.len() - 1;
                out.push(Stmt::Let(tmp, canon_expr(bad, *c)));
                let body = canon_block(p, locals, bad, body);
                let walk = |k: &InstKey, env: Place| {
                    let mut all: Vec<Arg<CTy>> = (p.captures(k).into_iter().enumerate())
                        .map(|(j, t)| Arg::Value(read(Place::Field(Box::new(env.clone()), j), t)))
                        .collect();
                    all.extend(args.iter().cloned());
                    Stmt::Region {
                        target: Target::Inst(p.index[k]),
                        args: all,
                        var,
                        lend,
                        body: body.clone(),
                        site,
                        body_site,
                    }
                };
                match &cty {
                    CTy::Clo(k) => out.push(walk(k, Place::Local(tmp))),
                    CTy::Fn(_, _, ms) => {
                        let arms = (ms.iter().enumerate())
                            .map(|(i, k)| {
                                vec![walk(k, Place::Variant(Box::new(Place::Local(tmp)), i))]
                            })
                            .collect();
                        out.push(Stmt::Switch(
                            Expr {
                                site: 0,
                                kind: EK::Tag(Place::Local(tmp)),
                                ty: CTy::Int,
                            },
                            arms,
                        ));
                    }
                    _ => bad.push("a generator value walked without its closure".into()),
                }
            }
            Stmt::Region {
                target,
                args,
                var,
                lend,
                body,
                site,
                body_site,
            } => {
                let args = args
                    .into_iter()
                    .map(|a| match a {
                        Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                        l => l,
                    })
                    .collect();
                out.push(Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body: canon_block(p, locals, bad, body),
                    site,
                    body_site,
                });
            }
            other => out.push(other),
        }
    }
    out
}
/// A call through a fn value read from a place, with its args canonical; ownership expands it
/// (`canonicalize`'s `expand`).
fn canon_call(bad: &mut Vec<String>, e: Expr<CTy>) -> Expr<CTy> {
    let EK::CallValue(callee, args, at) = e.kind else {
        return e;
    };
    if !matches!(callee.kind, EK::Copy(_)) {
        bad.push("a call through a fn value not read from a place".into());
    }
    let args = (args.into_iter())
        .map(|a| match a {
            Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
            a => a,
        })
        .collect();
    Expr {
        kind: EK::CallValue(callee, args, at),
        ..e
    }
}
fn canon_store(
    locals: &mut Vec<CTy>,
    bad: &mut Vec<String>,
    dst: Place,
    let_local: Option<usize>,
    e: Expr<CTy>,
    out: &mut Vec<Stmt<CTy>>,
) {
    let store = |v: Expr<CTy>| match let_local {
        Some(l) => Stmt::Let(l, v),
        None => Stmt::Assign(dst.clone(), v),
    };
    match e.kind {
        EK::CallValue(..) => out.push(store(canon_call(bad, e))),
        EK::Widen(inner) => {
            let (from, to) = (inner.ty.clone(), e.ty.clone());
            if from == to {
                return out.push(store(canon_expr(bad, *inner)));
            }
            let (CTy::Fn(_, _, a), CTy::Fn(_, _, b)) = (&from, &to) else {
                return;
            };
            locals.push(from.clone());
            let tmp = locals.len() - 1;
            out.push(Stmt::Let(tmp, canon_expr(bad, *inner)));
            let arms = a
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let j = b.iter().position(|x| x == k).unwrap_or_else(|| {
                        bad.push(
                            "internal: a function value is widened to a set that lacks it"
                                .to_string(),
                        );
                        0
                    });
                    let payload = read(
                        Place::Variant(Box::new(Place::Local(tmp)), i),
                        CTy::Clo(Box::new(k.clone())),
                    );
                    vec![Stmt::Assign(
                        dst.clone(),
                        Expr {
                            site: 0,
                            kind: EK::MakeUnion(j, Box::new(payload)),
                            ty: to.clone(),
                        },
                    )]
                })
                .collect();
            out.push(Stmt::Switch(
                Expr {
                    site: 0,
                    kind: EK::Tag(Place::Local(tmp)),
                    ty: CTy::Int,
                },
                arms,
            ));
        }
        _ => out.push(store(canon_expr(bad, e))),
    }
}
fn canon_expr(bad: &mut Vec<String>, e: Expr<CTy>) -> Expr<CTy> {
    let site = e.site;
    let ty = e.ty.clone();
    let kind = match e.kind {
        EK::Inject(x) => {
            let x = canon_expr(bad, *x);
            match (&x.ty, &ty) {
                (CTy::Clo(k), CTy::Fn(_, _, ms)) => {
                    let at = ms.iter().position(|m| m == &**k).unwrap_or_else(|| {
                        bad.push(
                            "internal: a function value is put in a set that lacks it".to_string(),
                        );
                        0
                    });
                    EK::MakeUnion(at, Box::new(x))
                }
                _ => x.kind,
            }
        }
        EK::Widen(x) => {
            // Only reached when both sets are the same.
            canon_expr(bad, *x).kind
        }
        EK::MakeStruct(xs) => EK::MakeStruct(xs.into_iter().map(|e| canon_expr(bad, e)).collect()),
        EK::MakeClosure(xs) => {
            EK::MakeClosure(xs.into_iter().map(|e| canon_expr(bad, e)).collect())
        }
        EK::MakeUnion(i, x) => EK::MakeUnion(i, Box::new(canon_expr(bad, *x))),
        EK::Intr(i, args, at) => EK::Intr(
            i,
            args.into_iter()
                .map(|a| match a {
                    Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                    l => l,
                })
                .collect(),
            at,
        ),
        EK::Call(t, args, at) => EK::Call(
            t,
            args.into_iter()
                .map(|a| match a {
                    Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                    l => l,
                })
                .collect(),
            at,
        ),
        k => k,
    };
    Expr { site, kind, ty }
}
