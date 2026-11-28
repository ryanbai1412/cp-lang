//! Which type parameters are domain parameters, and the fresh domains a result names.
use super::*;

/// A result's `domain C` slot holds the fresh domain's positions, a `Range<C>` (R02.9.3).
pub(super) fn fresh_results(d: &mut Decls) {
    // `-> Rooted<V, _, _>`: a `_` in a domain's place is a domain of its own, as a `domain C` is.
    if let Some(domain) = d.roles.domain {
        for f in d.ret_holes.clone() {
            let mut ret = std::mem::replace(&mut d.fns[f.0].ret, Ty::Error);
            fill_holes(d, f, &mut ret, domain);
            d.fns[f.0].ret = ret;
        }
    }
    let range = (d.sugar.range_literal).and_then(|r| match &d.fns[r.0].ret {
        Ty::Struct(r, ..) => Some(*r),
        _ => None,
    });
    let Some(range) = range else {
        return;
    };
    for (f, slots) in std::mem::take(&mut d.fresh_slots) {
        if let Ty::Struct(_, args, _) = &mut d.fns[f.0].ret {
            for k in slots {
                if let Some(a) = args.get_mut(k) {
                    *a = Ty::Struct(range, vec![a.clone()], Vec::new());
                }
            }
        }
    }
}

/// `t` keys by parameter `i`, also through a struct whose own parameter in that place is a
/// domain: `self: Dsu<V>`.
fn keys(d: &Decls, t: &Ty, pos: &[TraitId], i: usize, depth: usize) -> bool {
    let go = |t: &Ty| depth < 8 && keys(d, t, pos, i, depth + 1);
    match t {
        Ty::Struct(s, ts, _) => {
            ts.iter().any(go)
                || (ts.iter().enumerate()).any(|(k, a)| {
                    (pos.iter()
                        .any(|tr| d.structs[s.0].bounds.contains(&(k, *tr)))
                        && domain_at(d, a, pos, i, depth))
                        || (*a == Ty::Param(i)
                            && depth < 8
                            && (d.structs[s.0].fields.iter())
                                .any(|(_, f)| keys(d, f, pos, k, depth + 1)))
                })
        }
        Ty::Raw(e) | Ty::Ref(e) | Ty::Mut(e) | Ty::RangeTo(e, ..) => go(e),
        Ty::Fn(ps, r, _) => ps.iter().chain([&**r]).any(go),
        _ => false,
    }
}
/// `P` in a domain's place is one, and so is a part of it that `P`'s `Domain` impl bounds by
/// `Domain`: `V` in `[(V, V)]T` (R04.2.11).
fn domain_at(d: &Decls, t: &Ty, pos: &[TraitId], i: usize, depth: usize) -> bool {
    let Ty::Struct(s, ts, _) = t else {
        return *t == Ty::Param(i);
    };
    depth < 8
        && (d.impls.iter()).any(|m| {
            let Ty::Struct(f, ps, _) = &m.for_ else {
                return false;
            };
            pos.contains(&m.trait_.tr) && f == s && (ps.iter().zip(ts)).any(|(p, a)| {
            matches!(p, Ty::Param(j) if m.bounds.iter().any(|(b, r)| b == j && pos.contains(&r.tr)))
                && domain_at(d, a, pos, i, depth + 1)
        })
        })
}

/// `s`'s parameter `k` is a domain: a field keys by it, as `at: [Pre]V` does, or it is bounded so.
fn domain_param_of(d: &Decls, s: StructId, k: usize, domain: TraitId) -> bool {
    let pos: Vec<TraitId> = d.roles.discrete.into_iter().chain([domain]).collect();
    pos.iter()
        .any(|tr| d.structs[s.0].bounds.contains(&(k, *tr)))
        || (d.structs[s.0].fields.iter()).any(|(_, f)| keys(d, f, &pos, k, 0))
}

/// The `_`s of `t`, a result, in its structs' domain parameters, each made a fresh domain.
fn fill_holes(d: &mut Decls, f: FnId, t: &mut Ty, domain: TraitId) {
    let Ty::Struct(s, ts, _) = t else {
        return;
    };
    for (k, a) in ts.iter_mut().enumerate() {
        if *a != Ty::Error {
            fill_holes(d, f, a, domain);
        } else if domain_param_of(d, *s, k, domain) {
            let item = d.fns[f.0].item.clone();
            let c = d.new_dom(&item);
            if let Ty::Dom(i) = c {
                let name = format!("{}.{}", d.fns[f.0].name, d.structs[s.0].generics[k]);
                d.fresh_doms.push((i, name));
            }
            let k = d.syms.intern(&format!("${}", d.fresh_doms.len()));
            d.top_doms.insert(k, c.clone());
            *a = c;
        }
    }
}

/// A type parameter in a place whose struct bounds it by `Domain`, in a fn's signature or in its
/// owner's fields, is a domain parameter (`struct Dsu<V>: parent: [V]V`, `Range<D>`): its body
/// may use what every domain has (R04.2). So is one passed where a trait bound's parameter is
/// one (`S: Indexed<D, T>`, whose `keys` is a `Range<D>`), and the `Self` of an `impl for _..=_`,
/// which only range types meet. A caller is not held to `Domain`,
/// since a list keyed by `Int` passes as `[Int]T`.
pub(super) fn domain_params(
    d: &mut Decls,
    from: usize,
    bodies: &HashMap<FnId, cpl_next_syntax::ast::FnBody>,
) {
    let Some(domain) = d.roles.domain else {
        return;
    };
    // A parameter whose body names one of `Domain`'s fns on it (`D.nth(k)`), or indexes a table
    // or an array by it (`[V] v => e`, `var s: [V]Int`), or walks it (`for v in V`), is one (R04.4.30).
    // One that names one of `Ordered`'s steps on it (`D.at(x)`) is ordered too (R04.2.31).
    use cpl_next_syntax::ast::{self as next, AstNode, GenericArg, Type};
    let pos: Vec<TraitId> = d.roles.discrete.into_iter().chain([domain]).collect();
    let dom_fns: Vec<Sym> = (pos.iter())
        .flat_map(|tr| d.traits[tr.0].methods.keys().copied())
        .collect();
    let steps: Vec<Sym> = (d.roles.order.iter())
        .flat_map(|tr| d.traits[tr.0].methods.keys().copied())
        .collect();
    let syms = &d.syms;
    let uses = |f: &FnDecl, body: Option<&next::FnBody>| -> (Vec<usize>, Vec<usize>) {
        let (mut found, mut ordered) = (Vec::new(), Vec::new());
        let Some(body) = body else {
            return (found, ordered);
        };
        let node = match body {
            next::FnBody::Block(b) => b.syntax().clone(),
            next::FnBody::Expr(e) => e.syntax().clone(),
        };
        let generic = |t: &cpl_next_syntax::tree::SyntaxToken| {
            let s = syms.get(t.text())?;
            f.generics.iter().position(|g| *g == s)
        };
        // `[V]`: a parameter that indexes a table or an array written in the body is a domain.
        let index = |dims: Option<next::Dims>, found: &mut Vec<usize>| {
            for a in dims.iter().flat_map(|d| d.args().collect::<Vec<_>>()) {
                let GenericArg::Type(Type::Path(p)) = a else {
                    continue;
                };
                if p.generic_args().is_some_and(|g| g.args().next().is_some()) {
                    continue;
                }
                let segs: Vec<_> = p.segments().collect();
                let [seg] = &segs[..] else {
                    continue;
                };
                if let Some(i) = generic(seg).filter(|i| !found.contains(i)) {
                    found.push(i);
                }
            }
        };
        for n in node.descendants() {
            if let Some(l) = next::LetStmt::cast(n.clone()) {
                if let Some(Type::Array(a)) = l.ty() {
                    index(a.dims(), &mut found);
                }
            } else if let Some(t) = next::TableExpr::cast(n.clone()) {
                index(t.syntax().children().find_map(next::Dims::cast), &mut found);
            } else if let Some(next::Expr::Name(w)) = (next::ForClause::cast(n.clone()))
                .and_then(|c| c.iter())
                .or_else(|| next::CompClause::cast(n.clone()).and_then(|c| c.iter()))
            {
                // `for v in V`: a parameter walked as a value is a domain.
                if let Some(i) = w
                    .ident()
                    .as_ref()
                    .and_then(generic)
                    .filter(|i| !found.contains(i))
                {
                    found.push(i);
                }
            } else if let Some(fe) = next::FieldExpr::cast(n) {
                let (Some(next::Expr::Name(b)), Some(m)) = (fe.base(), fe.name()) else {
                    continue;
                };
                let Some(i) = b.ident().as_ref().and_then(generic) else {
                    continue;
                };
                let m = syms.get(m.text());
                let step = m.is_some_and(|m| steps.contains(&m));
                if (step || m.is_some_and(|m| dom_fns.contains(&m))) && !found.contains(&i) {
                    found.push(i);
                }
                if step && !ordered.contains(&i) {
                    ordered.push(i);
                }
            }
        }
        (found, ordered)
    };
    let (used, ordered): (Vec<Vec<usize>>, Vec<Vec<usize>>) = (from..d.fns.len())
        .map(|k| uses(&d.fns[k], bodies.get(&FnId(k))))
        .unzip();
    // A parameter written with bounds has the order they give it, and no more.
    for (k, mut os) in ordered.into_iter().enumerate() {
        os.retain(|i| !d.fns[from + k].bounds.iter().any(|(p, _)| p == i));
        if !os.is_empty() {
            d.ordered_params.insert(FnId(from + k), os);
        }
    }
    for k in from..d.fns.len() {
        let f = &d.fns[k];
        let fields: &[(Sym, Ty)] = match f.owner {
            Some(Owner::Struct(s)) => &d.structs[s.0].fields,
            _ => &[],
        };
        let added: Vec<usize> = (0..f.generics.len())
            .filter(|&i| {
                (f.params.iter().map(|p| &p.ty))
                    .chain([&f.ret])
                    .chain(&f.self_ty)
                    .chain(fields.iter().map(|(_, t)| t))
                    .any(|t| keys(d, t, &pos, i, 0))
                    || used[k - from].contains(&i)
                    || matches!(f.owner, Some(Owner::Impl(m)) if d.impls[m].for_ == Ty::AnyRange(i))
            })
            .collect();
        if !added.is_empty() {
            d.domain_params.insert(FnId(k), added);
        }
    }
    // A struct's parameter that one of its methods treats as a domain is one in all of them.
    let mut of_struct: HashMap<StructId, Vec<usize>> = HashMap::new();
    for (f, ds) in &d.domain_params {
        if let Some(Owner::Struct(s)) = d.fns[f.0].owner {
            let n = d.structs[s.0].generics.len();
            let js = of_struct.entry(s).or_default();
            js.extend(
                ds.iter()
                    .filter(|&&j| j < n && !js.contains(&j))
                    .collect::<Vec<_>>(),
            );
        }
    }
    for k in from..d.fns.len() {
        let Some(Owner::Struct(s)) = d.fns[k].owner else {
            continue;
        };
        let Some(js) = of_struct.get(&s) else {
            continue;
        };
        let ds = d.domain_params.entry(FnId(k)).or_default();
        for &j in js {
            if !ds.contains(&j) {
                ds.push(j);
            }
        }
    }
    // Through bounds, until no trait's or fn's domain parameters grow.
    loop {
        // A trait's parameter is a domain when one of its methods uses it as one, or when it is
        // passed to a supertrait's domain parameter. A supertrait's args are in the trait's own
        // parameters, without `Self`: `IndexedMut<K, T>: Indexed<K, T>`.
        fn trait_dom(d: &Decls, tr: TraitId, p: usize, depth: usize) -> bool {
            let t = &d.traits[tr.0];
            (t.methods.values())
                .any(|m| (d.domain_params.get(m)).is_some_and(|ds| ds.contains(&(p + 1))))
                || (depth < 8
                    && (t.supers.iter()).any(|sup| {
                        (sup.args.iter().enumerate())
                            .any(|(q, a)| *a == Ty::Param(p) && trait_dom(d, sup.tr, q, depth + 1))
                    }))
        }
        let mut grew = false;
        for k in 0..d.fns.len() {
            let more: Vec<usize> = (d.fns[k].bounds.iter())
                .flat_map(|(_, r)| {
                    (r.args.iter().enumerate()).filter_map(move |(p, a)| match a {
                        Ty::Param(j) => Some((r.tr, p, *j)),
                        _ => None,
                    })
                })
                .filter(|&(tr, p, _)| trait_dom(d, tr, p, 0))
                .map(|(_, _, j)| j)
                .collect();
            let ds = d.domain_params.entry(FnId(k)).or_default();
            for j in more {
                if !ds.contains(&j) {
                    ds.push(j);
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    d.domain_params.retain(|_, ds| !ds.is_empty());
}
