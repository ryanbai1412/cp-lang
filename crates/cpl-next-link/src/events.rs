//! Contract 8's event points, placed on instance IR after the link step.
use cpl_next_decls::modules::PROGRAM;
use cpl_next_decls::views::Checked;
use cpl_next_ir::ids::SourceId;
use cpl_next_ir::ir::*;
use cpl_next_ir::prog::{CTy, InstKey, Program};
use cpl_next_ir::ty::FnId;
use std::collections::HashMap;

/// Contract 8: event points, placed on instance IR after the link step, only when asked for.
/// They are not in generic IR and are not uses for ownership. Reported here: entering and
/// leaving a fn of the user's file, and each write the user's file makes to one of its
/// top-level bindings. A value an event shows is the text the `trace` role's fn writes for
/// it; a type link made no copy of that fn for shows none.
pub fn add_events(p: &mut Program, checked: &dyn Checked, trace: Option<FnId>) {
    let shows = Shows {
        checked,
        index: &p.index,
        trace,
    };
    for (i, f) in p.insts.iter_mut().enumerate() {
        let func = i as u32;
        let user = match &f.key {
            FuncKey::Decl(id) | FuncKey::Ctx(id, _) => {
                checked.in_program(*id) && f.region.is_none()
            }
            _ => false,
        };
        let body = std::mem::take(&mut f.body);
        let mut placer = Placer {
            shows: &shows,
            sites: &f.sites,
            locals: &mut f.locals,
            user,
            func,
        };
        let mut body = placer.block(body);
        if user {
            let args = (f.params[f.n_captures..].iter())
                .filter_map(|(l, _)| shows.of(&f.locals[*l], Place::Local(*l)))
                .collect();
            body.insert(0, point(EventKind::Enter, func, 0, args));
            if f.ret == CTy::Unit {
                body.push(point(EventKind::Exit, func, 0, vec![]));
            }
        }
        f.body = body;
    }
}

fn point(kind: EventKind, func: u32, site: Site, vals: Vec<Expr<CTy>>) -> Stmt<CTy> {
    Stmt::Event(EventPoint {
        kind,
        func,
        site,
        vals,
    })
}

struct Shows<'a> {
    checked: &'a dyn Checked,
    index: &'a HashMap<InstKey, usize>,
    trace: Option<FnId>,
}

impl Shows<'_> {
    /// The call that shows a copy of the value at `at`, of type `t`, when link made one for `t`.
    fn of(&self, t: &CTy, at: Place) -> Option<Expr<CTy>> {
        let key = InstKey {
            func: FuncKey::Decl(self.trace?),
            targs: vec![t.unlent().clone()],
            sets: vec![],
        };
        let i = *self.index.get(&key)?;
        let x = Expr {
            site: 0,
            kind: EK::Copy(at),
            ty: t.clone(),
        };
        Some(Expr {
            site: 0,
            kind: EK::Call(Target::Inst(i), vec![Arg::Value(x)], 0),
            ty: CTy::Unit,
        })
    }
}

struct Placer<'a> {
    shows: &'a Shows<'a>,
    sites: &'a [SourceId],
    locals: &'a mut Vec<CTy>,
    user: bool,
    func: u32,
}

impl Placer<'_> {
    fn block(&mut self, b: Vec<Stmt<CTy>>) -> Vec<Stmt<CTy>> {
        let func = self.func;
        let mut out = Vec::new();
        for s in b {
            match s {
                // The result is kept in a local first, so the event shows the value returned
                // without computing it twice.
                Stmt::Return(Some(e)) if self.user => {
                    let (site, t, ty) = (e.site, self.locals.len(), e.ty.clone());
                    match self.shows.of(&ty, Place::Local(t)) {
                        Some(v) => {
                            self.locals.push(ty.clone());
                            out.push(Stmt::Let(t, e));
                            out.push(point(EventKind::Exit, func, site, vec![v]));
                            let kind = EK::Copy(Place::Local(t));
                            out.push(Stmt::Return(Some(Expr { site: 0, kind, ty })));
                        }
                        None => {
                            out.push(point(EventKind::Exit, func, site, vec![]));
                            out.push(Stmt::Return(Some(e)));
                        }
                    }
                }
                Stmt::Return(None) if self.user => {
                    out.push(point(EventKind::Exit, func, 0, vec![]));
                    out.push(Stmt::Return(None));
                }
                Stmt::Assign(Place::Global(g), e) => {
                    let global = self.shows.checked.global(cpl_next_ir::whole::GlobalId(g));
                    let (ty, site) = (e.ty.clone(), e.site);
                    out.push(Stmt::Assign(Place::Global(g), e));
                    let mine = site != 0
                        && (self.sites.get(site as usize))
                            .is_some_and(|at| &*at.item.module == PROGRAM);
                    match global {
                        Some(x) if x.top && mine && !x.name.contains('$') => {
                            let kind = EventKind::Write {
                                global: g as u32,
                                shown: x.name.as_str().into(),
                            };
                            let now = self.shows.of(&ty, Place::Global(g));
                            out.push(point(kind, func, site, now.into_iter().collect()));
                        }
                        _ => {}
                    }
                }
                Stmt::If(c, a, b2) => {
                    let (a, b2) = (self.block(a), self.block(b2));
                    out.push(Stmt::If(c, a, b2));
                }
                Stmt::Loop(a, b2, s) => {
                    let (a, b2) = (self.block(a), self.block(b2));
                    out.push(Stmt::Loop(a, b2, s));
                }
                Stmt::Switch(c, arms) => {
                    let arms = arms.into_iter().map(|a| self.block(a)).collect();
                    out.push(Stmt::Switch(c, arms));
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
                    let body = self.block(body);
                    out.push(Stmt::Region {
                        target,
                        args,
                        var,
                        lend,
                        body,
                        site,
                        body_site,
                    });
                }
                other => out.push(other),
            }
        }
        out
    }
}
