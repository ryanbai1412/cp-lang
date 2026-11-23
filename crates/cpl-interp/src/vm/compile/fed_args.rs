//! Arguments of a fed call of a `yields` fn: values first, then each `mut`
//! argument moved out of its place for the whole loop, stored back when the
//! loop ends however it ends (`CallVmFeed`). The loop body never sees the
//! place (`cpl_ir::pushed::Feeds`), so nothing reads it while it is lent.

use super::*;

type FedArgs = (Vec<u16>, Vec<u16>, Vec<(usize, FeedBack)>);

/// The registers a `mut` argument that is not a register of its own goes in.
enum Lent {
    V(u16),
    I(u16, ScalarKind),
    OptI(u16, u16, ScalarKind),
}

impl<'p, 'f> Compiler<'p, 'f> {
    pub(super) fn compile_fed_args(&mut self, fn_id: FnId, args: &'p [Arg]) -> Option<FedArgs> {
        let f = &self.program.fns[fn_id];
        let has_fn_param =
            (f.locals.iter().take(f.params.len())).any(|l| matches!(l.ty, Ty::Fn(..)));
        let has_mut = f.params.iter().any(|p| p.mutable);
        if has_mut && has_fn_param && !self.mut_fn_args_alias_free(fn_id, args) {
            return None;
        }
        let held = |c: &Self, arg: &Arg| match arg {
            Arg::Mut(place) => match place.kind {
                PlaceKind::Local(slot) => Some(c.slot_loc(slot)),
                _ => None,
            },
            Arg::Value(_) => None,
        };
        for (i, arg) in args.iter().enumerate() {
            let scalar = ScalarKind::of_ty(&f.locals[i].ty, self.program).is_some();
            let opt = ScalarKind::of_opt_ty(&f.locals[i].ty, self.program).is_some();
            let ok = match (arg, held(self, arg)) {
                (Arg::Value(_), _) => !f.params[i].mutable,
                (Arg::Mut(_), loc) => {
                    f.params[i].mutable
                        && match loc {
                            None => true,
                            Some(SlotLoc::I(..)) => scalar,
                            Some(SlotLoc::OptI { .. }) => opt,
                            Some(SlotLoc::V(_)) => !scalar && !opt,
                            Some(SlotLoc::SmallTuple { .. }) => false,
                        }
                }
            };
            if !ok {
                return None;
            }
        }
        let marks = (self.ops.len(), self.spans.len(), self.places.len());
        let (mut args_i, mut args_v, mut muts) = (Vec::new(), Vec::new(), Vec::new());
        let mut lent = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let pty = &f.locals[i].ty;
            let scalar = ScalarKind::of_ty(pty, self.program);
            let opt = ScalarKind::of_opt_ty(pty, self.program);
            match (arg, held(self, arg)) {
                (Arg::Mut(_), Some(SlotLoc::I(r, _))) => {
                    args_i.push(r);
                    muts.push((i, FeedBack::I(r)));
                }
                (Arg::Mut(_), Some(SlotLoc::OptI { val, some, .. })) => {
                    args_i.extend([val, some]);
                    muts.push((i, FeedBack::OptI(val, some)));
                }
                (Arg::Mut(place), _) => {
                    let regs = if let Some(kind) = scalar {
                        let r = self.alloc_i();
                        args_i.push(r);
                        Lent::I(r, kind)
                    } else if let Some(kind) = opt {
                        let (val, some) = (self.alloc_i(), self.alloc_i());
                        args_i.extend([val, some]);
                        Lent::OptI(val, some, kind)
                    } else {
                        let r = self.alloc_v();
                        args_v.push(r);
                        Lent::V(r)
                    };
                    lent.push((i, place, regs));
                }
                (Arg::Value(e), _) if scalar.is_some() => args_i.push(self.compile_expr_i(e)),
                (Arg::Value(e), _) if opt.is_some() => {
                    let (rv, rs) = self.compile_expr_opt_i(e);
                    args_i.extend([rv, rs]);
                }
                (Arg::Value(e), _) => args_v.push(self.compile_expr_v(e)),
            }
        }
        for (i, place, regs) in lent {
            let Some(path) = self.whole_place_path(place) else {
                self.ops.truncate(marks.0);
                self.spans.truncate(marks.1);
                self.places.truncate(marks.2);
                return None;
            };
            let span = place.span;
            let back = match regs {
                Lent::V(dst) => {
                    self.emit(Op::TakePlaceV { dst, path }, span);
                    FeedBack::Place(path)
                }
                Lent::I(dst_i, kind) => {
                    let src_v = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst: src_v, path }, span);
                    self.emit(Op::UnboxI { dst_i, src_v }, span);
                    self.emit(Op::ClearV { reg: src_v }, span);
                    FeedBack::PlaceI(path, kind)
                }
                Lent::OptI(dst_val, dst_some, kind) => {
                    let src_v = self.alloc_v();
                    self.emit(Op::TakePlaceV { dst: src_v, path }, span);
                    let unbox = Op::UnboxOptI {
                        dst_val,
                        dst_some,
                        src_v,
                    };
                    self.emit(unbox, span);
                    self.emit(Op::ClearV { reg: src_v }, span);
                    FeedBack::PlaceOptI(path, kind)
                }
            };
            muts.push((i, back));
        }
        Some((args_i, args_v, muts))
    }

    /// The path of `place`, a whole variable included.
    pub(super) fn whole_place_path(&mut self, place: &'p Place) -> Option<u16> {
        let root = match place.kind {
            PlaceKind::Local(slot) => match self.slot_loc(slot) {
                SlotLoc::V(v) => PlaceRoot::LocalV(v),
                _ => return None,
            },
            PlaceKind::Global(gid) => PlaceRoot::Global(gid as u16),
            _ => return self.compile_place_path_to(place, true),
        };
        let idx = u16::try_from(self.places.len()).ok()?;
        self.places.push(PlacePath {
            root,
            steps: Vec::new(),
            keys: Vec::new(),
        });
        Some(idx)
    }
}
