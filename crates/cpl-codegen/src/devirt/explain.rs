//! What devirt explains: each closure's lowering, its locals, and the fields it drops.

use super::*;

/// The data fields `program` drops because nothing reads them (decision
/// 142), as `struct S.f`. Dropped fn fields are reported by
/// [`explain_closures`].
pub fn explain_dropped_fields(program: &Program) -> Vec<String> {
    dropped_fields(program, &crate::emit(program, "", None))
}

/// [`explain_dropped_fields`] for the judge build's C++ `cpp`.
pub(crate) fn dropped_fields(program: &Program, cpp: &str) -> Vec<String> {
    let devirt = Devirt::collect(program);
    devirt
        .unread_fields
        .iter()
        .filter(|&&(sid, idx)| !is_fn_or_opt_fn(&program.structs[sid].fields[idx].1))
        .filter(|&&(sid, _)| {
            cpp.contains(&format!(
                "struct {} {{",
                crate::names::ident(&program.structs[sid].name)
            ))
        })
        .map(|&(sid, idx)| {
            format!(
                "struct {}.{}",
                program.structs[sid].name, program.structs[sid].fields[idx].0
            )
        })
        .collect()
}

/// Report how every fn-typed struct field, fn parameter, and local closure
/// binding in `program` lowers to C++17 (static vs dynamic `std::function`),
/// over the structs and fns the file defines (`written`).
pub fn explain_closures(
    program: &Program,
    written: &crate::emit::lowered::Written,
    cpp: &str,
) -> Vec<ClosureLowering> {
    let mut devirt = Devirt::collect(program);
    devirt.built_only_in(&written.fns, &written.closures);
    devirt.bound = written.bound.clone();
    let uses = crate::uses::Uses::of(program, &devirt, true);
    let views = crate::uses::fn_refs::fn_refs(program, &devirt, &uses);
    let mut out = Vec::new();

    for (sid, s) in program.structs.iter().enumerate() {
        if !written.structs.contains(&sid) {
            continue;
        }
        for (idx, (fname, fty)) in s.fields.iter().enumerate() {
            if !is_fn_or_opt_fn(fty) {
                continue;
            }
            let site = format!("struct {}.{}", s.name, fname);
            let mode = if devirt.unread_fields.contains(&(sid, idx)) {
                ClosureMode::Unread
            } else if devirt.unbuilt_field(sid, idx) {
                ClosureMode::Unbuilt
            } else if matches!(fty, Ty::Opt(_)) {
                erased(program, fty, "optional fn field that is read")
            } else if let Some(closures) = devirt.field_functor(sid, idx) {
                ClosureMode::StaticFunctor {
                    closures: closures.to_vec(),
                }
            } else if devirt.field_elidable(sid, idx) {
                let Some(Held::One(cid)) = devirt.held.get(&Home::Field(sid, idx)).copied() else {
                    continue;
                };
                ClosureMode::StaticDirect { closure: cid }
            } else {
                let reason = match devirt.held.get(&Home::Field(sid, idx)) {
                    Some(Held::One(cid)) if devirt.captureless_one(Some(Held::One(*cid))).is_none() => {
                        "one closure, with captures no functor can hold, stored in field".to_string()
                    }
                    Some(Held::One(_)) => "field escapes or is reassigned".to_string(),
                    Some(Held::Many) => {
                        "closures the program can't list, or one capturing nothing, a global or its own struct, stored in field"
                            .to_string()
                    }
                    None => "no closure stored in it is known here".to_string(),
                };
                erased(program, fty, &reason)
            };
            out.push(ClosureLowering { site, mode });
        }
    }

    for (fid, f) in program.fns.iter().enumerate() {
        if !written.fns.contains(&fid) {
            continue;
        }
        let named = crate::scope::named_params(&f.body, f.params.len());
        for slot in 0..f.params.len() {
            if !matches!(f.locals[slot].ty, Ty::Fn(..)) {
                continue;
            }
            let site = format!("fn {} param {}", f.name, f.locals[slot].name);
            let read = named.contains(&slot) && !devirt.dead_local(Some(fid), slot);
            let mode = if devirt.param_elidable(fid, slot) && !read {
                ClosureMode::Unread
            } else if devirt.param_elidable(fid, slot) {
                let Some(Held::One(cid)) = devirt.held.get(&Home::Param(fid, slot)).copied() else {
                    continue;
                };
                ClosureMode::StaticDirect { closure: cid }
            } else if devirt.template_param_index(fid, slot).is_some() {
                ClosureMode::StaticTemplate
            } else if views[fid].contains(&slot) {
                ClosureMode::FnRef
            } else {
                let reason = if program.entry_fns.contains(&fid) || program.judge_fns.contains(&fid)
                {
                    "judge/entry function boundary".to_string()
                } else if devirt.recursive_fns.contains(&fid) {
                    "a call in its recursion passes a different closure".to_string()
                } else if f.params[slot].mutable {
                    "mut fn parameter".to_string()
                } else if devirt.reassigned_params.contains(&(fid, slot)) {
                    "reassigned parameter".to_string()
                } else {
                    "dynamic fn parameter".to_string()
                };
                erased(program, &f.locals[slot].ty, &reason)
            };
            out.push(ClosureLowering { site, mode });
        }

        if matches!(f.ret, Ty::Fn(..)) {
            out.push(ClosureLowering {
                site: format!("fn {} result", f.name),
                mode: if devirt.returns_lambda(fid) {
                    ClosureMode::StaticLocalAuto
                } else {
                    erased(program, &f.ret, "fn returning a closure")
                },
            });
        }

        explain_locals_in_block(
            &devirt,
            Some(fid),
            &format!("fn {}", f.name),
            &f.locals,
            &f.body,
            &views[fid],
            &mut out,
        );
    }

    explain_locals_in_block(
        &devirt,
        None,
        "main",
        &program.main_locals,
        &program.main,
        &BTreeSet::new(),
        &mut out,
    );

    for (id, g) in program.globals.iter().enumerate() {
        if matches!(g.ty, Ty::Fn(..)) {
            let mode = match devirt.global_target(id) {
                Some(closure) => ClosureMode::StaticDirect { closure },
                None if devirt.main_auto_global(id) => ClosureMode::StaticLocalAuto,
                None => erased(program, &g.ty, "top-level fn var"),
            };
            out.push(ClosureLowering {
                site: format!("top-level {}", g.name),
                mode,
            });
        }
    }
    for (cid, c) in program.closures.iter().enumerate() {
        let dynamic = |site: String, ty: &Ty, reason: &str| ClosureLowering {
            site,
            mode: erased(program, ty, reason),
        };
        for &slot in &c.params {
            if matches!(c.locals[slot].ty, Ty::Fn(..)) {
                out.push(dynamic(
                    format!("closure {cid} param {}", c.locals[slot].name),
                    &c.locals[slot].ty,
                    "fn parameter of a closure",
                ));
            }
        }
        if matches!(c.ret, Ty::Fn(..)) {
            out.push(dynamic(
                format!("closure {cid} result"),
                &c.ret,
                "closure returning a closure",
            ));
        }
        let hoisted = crate::scope::hoisted(&c.body, c.params.len() + c.captures.len());
        each_sink(&c.body, &mut |sink| {
            if let Sink::Let(ref slot, init) = sink {
                if matches!(c.locals[*slot].ty, Ty::Fn(..)) {
                    let site = format!("closure {cid} local {}", c.locals[*slot].name);
                    let auto = match init.as_ref() {
                        Some(init) => match &init.kind {
                            ExprKind::Closure(i) => {
                                devirt.let_auto(Some(cid), *slot, *i, 0)
                                    || !hoisted.contains(slot)
                                        && devirt.let_rec(Some(cid), *slot, *i, 0)
                            }
                            ExprKind::Call(f, _) => {
                                !hoisted.contains(slot)
                                    && devirt.let_auto_call(Some(cid), *slot, *f, 0)
                            }
                            _ => {
                                !hoisted.contains(slot)
                                    && devirt.let_auto_join(Some(cid), *slot, init, 0)
                            }
                        },
                        None => false,
                    };
                    out.push(match auto {
                        true => ClosureLowering {
                            site,
                            mode: ClosureMode::StaticLocalAuto,
                        },
                        false => dynamic(site, &c.locals[*slot].ty, "fn local inside a closure"),
                    });
                }
            }
        });
    }

    // A fn inside a tuple, record or container is spelled by its type
    // alone, so its lowering is the one its signature gets everywhere.
    let mut nested = Vec::new();
    for (sid, s) in program.structs.iter().enumerate() {
        if written.structs.contains(&sid) {
            s.fields
                .iter()
                .for_each(|(_, t)| nested_fns(t, true, &mut nested));
        }
    }
    for (fid, f) in program.fns.iter().enumerate() {
        if written.fns.contains(&fid) {
            f.locals
                .iter()
                .for_each(|l| nested_fns(&l.ty, true, &mut nested));
            nested_fns(&f.ret, true, &mut nested);
        }
    }
    for (cid, c) in program.closures.iter().enumerate() {
        if written.closures.contains(&cid) {
            c.locals
                .iter()
                .for_each(|l| nested_fns(&l.ty, true, &mut nested));
            nested_fns(&c.ret, true, &mut nested);
        }
    }
    program
        .main_locals
        .iter()
        .for_each(|l| nested_fns(&l.ty, true, &mut nested));
    program
        .globals
        .iter()
        .for_each(|g| nested_fns(&g.ty, true, &mut nested));
    for ty in nested {
        let mode = erased(program, &ty, "fn inside a tuple, record or container");
        // Read off the C++: a tuple of closures keeps their own types, so an
        // erased fn nested in a type is one the C++ spells.
        if matches!(mode, ClosureMode::DynamicStdFunction { .. }) && !cpp.contains("std::function<")
        {
            continue;
        }
        out.push(ClosureLowering {
            site: format!("fn {ty:?} inside a tuple, record or container"),
            mode,
        });
    }
    out
}

/// Each fn type `ty` holds below its top, once.
fn nested_fns(ty: &Ty, top: bool, out: &mut Vec<Ty>) {
    match ty {
        Ty::Fn(args, ret) => {
            if !top && !out.contains(ty) {
                out.push(ty.clone());
            }
            args.iter().for_each(|a| nested_fns(a, false, out));
            nested_fns(ret, false, out);
        }
        Ty::Opt(t) | Ty::Mut(t) => nested_fns(t, top, out),
        Ty::Tuple(ts) => ts.iter().for_each(|t| nested_fns(t, false, out)),
        Ty::Record(fs) => fs.iter().for_each(|(_, t)| nested_fns(t, false, out)),
        Ty::List(t)
        | Ty::Range(t)
        | Ty::Set(t)
        | Ty::HashSet(t)
        | Ty::ListView(t, _)
        | Ty::Keyed(t, _)
        | Ty::MutView(t, _)
        | Ty::Array { elem: t, .. } => nested_fns(t, false, out),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            nested_fns(k, false, out);
            nested_fns(v, false, out);
        }
        _ => {}
    }
}

pub(super) fn explain_locals_in_block(
    devirt: &Devirt<'_>,
    home: Option<FnId>,
    owner: &str,
    locals: &[cpl_ir::Local],
    block: &Block,
    views: &BTreeSet<usize>,
    out: &mut Vec<ClosureLowering>,
) {
    let mut stores: BTreeMap<usize, usize> = BTreeMap::new();
    each_put(block, &mut |put, _| match put {
        Put::Let(slot) => *stores.entry(slot).or_default() += 1,
        Put::Store(place, _) => {
            if let PlaceKind::Local(slot) = place.kind {
                *stores.entry(slot).or_default() += 1;
            }
        }
    });
    let params = home.map_or(0, |id| devirt.program.fns[id].params.len());
    let hoisted = crate::scope::hoisted(block, params);
    let mut memo_pairs = devirt.memo_pairs(block);
    memo_pairs.retain(|w, (b, ..)| !hoisted.contains(w) && !hoisted.contains(b));
    each_sink(block, &mut |sink| {
        let Sink::Let(ref slot, init) = sink else {
            return;
        };
        if !matches!(locals[*slot].ty, Ty::Fn(..)) {
            return;
        }
        let site = format!("{owner} local {}", locals[*slot].name);
        if views.contains(slot) {
            out.push(ClosureLowering {
                site,
                mode: ClosureMode::FnRef,
            });
            return;
        }
        let Some(init) = init else {
            if memo_pairs.contains_key(slot) {
                out.push(ClosureLowering {
                    site,
                    mode: ClosureMode::StaticLocalAuto,
                });
                return;
            }
            out.push(ClosureLowering {
                site,
                mode: erased(
                    devirt.program,
                    &locals[*slot].ty,
                    "fn slot declared before its value (a @memo fn's wrapper)",
                ),
            });
            return;
        };
        if devirt.dead_local(home, *slot) {
            out.push(ClosureLowering {
                site,
                mode: ClosureMode::Unread,
            });
            return;
        }
        let mode = match init.kind {
            ExprKind::Closure(cid)
                if stores.get(slot).copied().unwrap_or(0) <= 1
                    && (devirt.local_auto(*slot, cid) || devirt.local_rec(*slot, cid)) =>
            {
                if devirt.bound.contains(&cid) {
                    ClosureMode::Bound
                } else {
                    ClosureMode::StaticLocalAuto
                }
            }
            ExprKind::Closure(cid) if !devirt.local_auto(*slot, cid) => {
                erased(devirt.program, &locals[*slot].ty, "recursive local closure")
            }
            ExprKind::Closure(_) => erased(
                devirt.program,
                &locals[*slot].ty,
                "reassigned local closure",
            ),
            ExprKind::Call(f, _)
                if !hoisted.contains(slot)
                    && devirt.let_auto_call(
                        None,
                        *slot,
                        f,
                        stores.get(slot).map_or(0, |&n| n as u32),
                    ) =>
            {
                ClosureMode::StaticLocalAuto
            }
            ExprKind::Local(src)
                if !hoisted.contains(slot)
                    && stores.get(slot).copied().unwrap_or(0) <= 1
                    && home.is_some_and(|id| devirt.template_params(id).contains(&src)) =>
            {
                ClosureMode::StaticLocalAuto
            }
            _ if !hoisted.contains(slot)
                && devirt.let_auto_join(
                    None,
                    *slot,
                    init,
                    stores.get(slot).map_or(0, |&n| n as u32),
                ) =>
            {
                ClosureMode::StaticLocalAuto
            }
            _ => erased(devirt.program, &locals[*slot].ty, "dynamic fn value"),
        };
        out.push(ClosureLowering { site, mode });
    });
}
