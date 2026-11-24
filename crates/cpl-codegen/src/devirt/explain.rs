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
) -> Vec<ClosureLowering> {
    let mut devirt = Devirt::collect(program);
    devirt.built_only_in(&written.fns, &written.closures);
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
            } else if devirt.field_elidable(sid, idx) {
                let Some(Held::One(cid)) = devirt.held.get(&Home::Field(sid, idx)).copied() else {
                    continue;
                };
                ClosureMode::StaticDirect { closure: cid }
            } else {
                let reason = match devirt.held.get(&Home::Field(sid, idx)) {
                    Some(Held::One(_)) => "field escapes or is reassigned".to_string(),
                    Some(Held::Many) => {
                        "multiple or capturing closures stored in field".to_string()
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
                mode: erased(program, &f.ret, "fn returning a closure"),
            });
        }

        explain_locals_in_block(
            &devirt,
            Some(fid),
            &format!("fn {}", f.name),
            &f.locals,
            &f.body,
            &mut out,
        );
    }

    explain_locals_in_block(
        &devirt,
        None,
        "main",
        &program.main_locals,
        &program.main,
        &mut out,
    );

    for (id, g) in program.globals.iter().enumerate() {
        if matches!(g.ty, Ty::Fn(..)) {
            let mode = match devirt.global_target(id) {
                Some(closure) => ClosureMode::StaticDirect { closure },
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
        walk::deep_stmts(&c.body, &mut |stmt| {
            if let Stmt::Let(slot, init) = stmt {
                if matches!(c.locals[*slot].ty, Ty::Fn(..)) {
                    let site = format!("closure {cid} local {}", c.locals[*slot].name);
                    let auto = matches!(init.as_ref().map(|e| &e.kind),
                        Some(ExprKind::Closure(i)) if devirt.let_auto(Some(cid), *slot, *i, 0));
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
    out
}

pub(super) fn explain_locals_in_block(
    devirt: &Devirt<'_>,
    home: Option<FnId>,
    owner: &str,
    locals: &[cpl_ir::Local],
    block: &Block,
    out: &mut Vec<ClosureLowering>,
) {
    let mut stores: BTreeMap<usize, usize> = BTreeMap::new();
    walk::deep_stmts(block, &mut |stmt| match stmt {
        Stmt::Let(slot, Some(_)) => *stores.entry(*slot).or_default() += 1,
        Stmt::Assign(place, _, _) => {
            if let PlaceKind::Local(slot) = place.kind {
                *stores.entry(slot).or_default() += 1;
            }
        }
        _ => {}
    });
    walk::deep_stmts(block, &mut |stmt| {
        let Stmt::Let(slot, init) = stmt else {
            return;
        };
        if !matches!(locals[*slot].ty, Ty::Fn(..)) {
            return;
        }
        let site = format!("{owner} local {}", locals[*slot].name);
        let Some(init) = init else {
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
                ClosureMode::StaticLocalAuto
            }
            ExprKind::Closure(cid) if !devirt.local_auto(*slot, cid) => {
                erased(devirt.program, &locals[*slot].ty, "recursive local closure")
            }
            ExprKind::Closure(_) => erased(
                devirt.program,
                &locals[*slot].ty,
                "reassigned local closure",
            ),
            _ => erased(devirt.program, &locals[*slot].ty, "dynamic fn value"),
        };
        out.push(ClosureLowering { site, mode });
    });
}
