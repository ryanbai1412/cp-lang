//! Contract 8's event points, placed on instance IR after the link step.
use cpl_next_decls::modules::PROGRAM;
use cpl_next_ir::ids::SourceId;
use cpl_next_ir::ir::*;
use cpl_next_ir::prog::{CTy, Program};
use cpl_next_ir::whole::Whole;

/// Contract 8: event points, placed on instance IR after the link step, only when asked for.
/// They are not in generic IR and are not uses for ownership. Reported here: entering and
/// leaving a fn of the user's file, and each write the user's file makes to one of its
/// top-level bindings.
pub fn add_events(p: &mut Program, whole: &Whole) {
    fn shown(t: &CTy) -> bool {
        matches!(t, CTy::Int | CTy::Bool | CTy::Char | CTy::U64 | CTy::Dom(_))
    }
    fn block(
        b: Vec<Stmt<CTy>>,
        user: bool,
        func: u32,
        cx: (&Whole, &[SourceId]),
    ) -> Vec<Stmt<CTy>> {
        let (whole, sites) = cx;
        let point = |kind, site, vals| {
            Stmt::Event(EventPoint {
                kind,
                func,
                site,
                vals,
            })
        };
        let mut out = Vec::new();
        for s in b {
            match s {
                Stmt::Return(e) => {
                    if user {
                        out.push(point(
                            EventKind::Exit,
                            e.as_ref().map_or(0, |e| e.site),
                            e.iter().filter(|e| shown(&e.ty)).cloned().collect(),
                        ));
                    }
                    out.push(Stmt::Return(e));
                }
                Stmt::Assign(Place::Global(g), e) => {
                    let global = whole.global(cpl_next_ir::whole::GlobalId(g));
                    let (top, name, ty, site) = (
                        global.is_some_and(|x| x.top),
                        global.map(|x| x.name.clone()).unwrap_or_default(),
                        e.ty.clone(),
                        e.site,
                    );
                    out.push(Stmt::Assign(Place::Global(g), e));
                    let mine =
                        (sites.get(site as usize)).is_some_and(|at| &*at.item.module == PROGRAM);
                    if top && mine && shown(&ty) && !name.contains('$') {
                        let kind = EventKind::Write {
                            global: g as u32,
                            shown: name.into(),
                        };
                        let now = Expr {
                            site: 0,
                            kind: EK::Copy(Place::Global(g)),
                            ty,
                        };
                        out.push(point(kind, site, vec![now]));
                    }
                }
                Stmt::If(c, a, b2) => out.push(Stmt::If(
                    c,
                    block(a, user, func, cx),
                    block(b2, user, func, cx),
                )),
                Stmt::Loop(a, b2, s) => out.push(Stmt::Loop(
                    block(a, user, func, cx),
                    block(b2, user, func, cx),
                    s,
                )),
                Stmt::Switch(c, arms) => out.push(Stmt::Switch(
                    c,
                    arms.into_iter().map(|a| block(a, user, func, cx)).collect(),
                )),
                Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body,
                    site,
                    body_site,
                } => out.push(Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body: block(body, user, func, cx),
                    site,
                    body_site,
                }),
                other => out.push(other),
            }
        }
        out
    }
    for (i, f) in p.insts.iter_mut().enumerate() {
        let func = i as u32;
        let user_fn = match &f.key {
            FuncKey::Decl(id) | FuncKey::Ctx(id, _) => {
                whole.program_fns.contains(id) && f.region.is_none()
            }
            _ => false,
        };
        let body = std::mem::take(&mut f.body);
        let mut body = block(body, user_fn, func, (whole, &f.sites));
        if user_fn {
            let args = f.params[f.n_captures..]
                .iter()
                .filter(|(l, _)| shown(&f.locals[*l]))
                .map(|(l, _)| Expr {
                    site: 0,
                    kind: EK::Copy(Place::Local(*l)),
                    ty: f.locals[*l].clone(),
                })
                .collect();
            let point = |kind, vals| {
                Stmt::Event(EventPoint {
                    kind,
                    func,
                    site: 0,
                    vals,
                })
            };
            body.insert(0, point(EventKind::Enter, args));
            if f.ret == CTy::Unit {
                body.push(point(EventKind::Exit, vec![]));
            }
        }
        f.body = body;
    }
}
