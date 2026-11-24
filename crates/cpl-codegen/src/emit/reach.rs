//! What a program reaches: the fns, closures, structs and enums worth
//! writing, so uncalled std fns, unused types and their hooks are left out.

use crate::walk;
use cpl_ir::{Block, ClosureId, ExprKind, OrdHook, Program, Ty};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Reach {
    pub fns: BTreeSet<usize>,
    pub closures: BTreeSet<usize>,
    pub structs: BTreeSet<usize>,
    pub enums: BTreeSet<usize>,
    /// Some compared, sorted or keyed value holds a struct or enum, so the
    /// file needs `cpl_tie`.
    pub needs_tie: bool,
    fn_work: Vec<usize>,
    closure_work: Vec<usize>,
}

impl Reach {
    /// `same[c]` is the closure `c` is written as (`Devirt::same`).
    pub(super) fn of(program: &Program, same: &[ClosureId]) -> Reach {
        let mut r = Reach::default();
        for g in &program.globals {
            r.ty(program, &g.ty);
        }
        for l in &program.main_locals {
            r.ty(program, &l.ty);
        }
        for &entry in &program.entry_fns {
            r.call(entry);
        }
        r.block(program, same, &program.setup);
        r.block(program, same, &program.main);
        while !r.fn_work.is_empty() || !r.closure_work.is_empty() {
            while let Some(fid) = r.fn_work.pop() {
                let f = &program.fns[fid];
                r.ty(program, &f.ret);
                for l in &f.locals {
                    r.ty(program, &l.ty);
                }
                r.block(program, same, &f.body);
            }
            while let Some(cid) = r.closure_work.pop() {
                let c = &program.closures[cid];
                r.ty(program, &c.ret);
                for l in &c.locals {
                    r.ty(program, &l.ty);
                }
                r.block(program, same, &c.body);
            }
        }
        r
    }

    fn call(&mut self, id: usize) {
        if self.fns.insert(id) {
            self.fn_work.push(id);
        }
    }

    fn closure(&mut self, id: usize) {
        if self.closures.insert(id) {
            self.closure_work.push(id);
        }
    }

    /// The structs and enums `ty` holds, and their `==` and order hooks.
    fn ty(&mut self, program: &Program, ty: &Ty) {
        match ty {
            Ty::Struct(sid) => {
                if self.structs.insert(*sid) {
                    let s = &program.structs[*sid];
                    for (_, fty) in &s.fields {
                        self.ty(program, fty);
                    }
                    for hook in [s.eq_fn, s.ord.map(OrdHook::id)].into_iter().flatten() {
                        self.call(hook);
                    }
                }
            }
            Ty::Enum(eid) => {
                if self.enums.insert(*eid) {
                    for (_, fields) in &program.enums[*eid].variants {
                        for fty in fields {
                            self.ty(program, fty);
                        }
                    }
                }
            }
            Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Mut(t) => {
                self.ty(program, t)
            }
            Ty::Array { elem, .. } => self.ty(program, elem),
            Ty::Map(k, v) | Ty::HashMap(k, v) => {
                self.ty(program, k);
                self.ty(program, v);
            }
            Ty::Fn(args, ret) => {
                for a in args {
                    self.ty(program, a);
                }
                self.ty(program, ret);
            }
            Ty::Tuple(ts) => {
                for t in ts {
                    self.ty(program, t);
                }
            }
            Ty::Record(fs) => {
                for (_, t) in fs {
                    self.ty(program, t);
                }
            }
            _ => {}
        }
    }

    /// The `show` fns printing a `ty` calls, each struct and enum once.
    fn show_ty(
        &mut self,
        program: &Program,
        ty: &Ty,
        seen: &mut (BTreeSet<usize>, BTreeSet<usize>),
    ) {
        match ty {
            Ty::Struct(sid) if !seen.0.insert(*sid) => return,
            Ty::Enum(eid) if !seen.1.insert(*eid) => return,
            _ => {
                if let Some(show_fn) = program.show_fn(ty) {
                    self.call(show_fn);
                }
            }
        }
        let parts = match ty {
            Ty::Range(t) | Ty::Mut(t) => vec![(**t).clone()],
            _ => crate::show::parts(program, ty),
        };
        for part in &parts {
            self.show_ty(program, part, seen);
        }
    }

    fn block(&mut self, program: &Program, same: &[ClosureId], b: &Block) {
        walk::deep_exprs(b, &mut |e| {
            self.ty(program, &e.ty);
            if let Ty::Set(k) | Ty::HashSet(k) | Ty::Map(k, _) | Ty::HashMap(k, _) = &e.ty {
                self.needs_tie |= has_adt(k);
            }
            match &e.kind {
                ExprKind::Call(fid, _) => self.call(*fid),
                ExprKind::Closure(cid) => {
                    self.closure(*cid);
                    self.closure(same[*cid]);
                }
                ExprKind::Cmp(_, left, _) => self.needs_tie |= has_adt(&left.ty),
                ExprKind::Sort(place, by, _) => match by {
                    cpl_ir::SortBy::Natural => self.needs_tie |= has_adt(&place.ty),
                    cpl_ir::SortBy::Key(_, k) => self.needs_tie |= has_adt(&k.ty),
                    cpl_ir::SortBy::Fn {
                        func,
                        is_order: false,
                    } => {
                        if let Ty::Fn(_, ret) = &func.ty {
                            self.needs_tie |= has_adt(ret);
                        }
                    }
                    _ => {}
                },
                ExprKind::Builtin(
                    cpl_ir::Builtin::Min
                    | cpl_ir::Builtin::Max
                    | cpl_ir::Builtin::Contains
                    | cpl_ir::Builtin::Sorted
                    | cpl_ir::Builtin::MapGet
                    | cpl_ir::Builtin::SetMin
                    | cpl_ir::Builtin::SetMax
                    | cpl_ir::Builtin::FirstGe
                    | cpl_ir::Builtin::FirstGt
                    | cpl_ir::Builtin::LastLe
                    | cpl_ir::Builtin::LastLt,
                    args,
                ) => self.needs_tie |= args.iter().any(|a| has_adt(&a.ty)),
                ExprKind::Print(args, _) => {
                    let mut seen = Default::default();
                    for a in args {
                        self.show_ty(program, &a.value.ty, &mut seen);
                    }
                }
                ExprKind::Format(pieces) => {
                    let mut seen = Default::default();
                    for p in pieces {
                        if let cpl_ir::FmtPiece::Value(a) = p {
                            self.show_ty(program, &a.value.ty, &mut seen);
                        }
                    }
                }
                ExprKind::Builtin(cpl_ir::Builtin::ShowStr | cpl_ir::Builtin::Join, args) => {
                    let mut seen = Default::default();
                    for a in args {
                        self.show_ty(program, &a.ty, &mut seen);
                    }
                }
                _ => {}
            }
        });
    }
}

/// Whether `ty` holds a struct or enum.
fn has_adt(ty: &Ty) -> bool {
    match ty {
        Ty::Struct(_) | Ty::Enum(_) => true,
        Ty::Range(t)
        | Ty::Opt(t)
        | Ty::List(t)
        | Ty::ListView(t, _)
        | Ty::Keyed(t, _)
        | Ty::MutView(t, _)
        | Ty::Set(t)
        | Ty::HashSet(t)
        | Ty::Mut(t) => has_adt(t),
        Ty::Array { elem, .. } => has_adt(elem),
        Ty::Map(k, v) | Ty::HashMap(k, v) => has_adt(k) || has_adt(v),
        Ty::Tuple(ts) => ts.iter().any(has_adt),
        Ty::Record(fs) => fs.iter().any(|(_, t)| has_adt(t)),
        _ => false,
    }
}
