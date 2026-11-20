//! Calls through a fn value that can only be one fn.
//!
//! `Monoid<T>.op`, the `compose` and `apply` of an `Action`, a `by:` or
//! `ok` parameter: each is a `std::function` in the emitted C++, and a
//! call through one is opaque to the C++ compiler. Most of them only
//! ever hold one closure, though: a program with one `SegTree(+, a)`
//! stores `(a, b) => a + b` in every `Monoid<Int>` it makes.
//!
//! This pass finds the struct fields and fn parameters of fn type that
//! are only ever given one closure, and that closure captures nothing.
//! A call through such a field or parameter is emitted as a direct call
//! of the closure, which the C++ compiler then inlines.

use std::collections::BTreeMap;

use cpl_ir::{Arg, Block, ClosureId, Expr, ExprKind, FnId, PlaceKind, Program, Stmt, Ty};

use crate::walk;

/// Somewhere a fn value is kept.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Home {
    Field(usize, usize),
    Param(FnId, usize),
}

/// What has been stored in a [`Home`] so far.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Held {
    One(ClosureId),
    Many,
}

pub(crate) struct Devirt<'p> {
    program: &'p Program,
    held: BTreeMap<Home, Held>,
    changed: bool,
}

impl<'p> Devirt<'p> {
    pub fn collect(program: &'p Program) -> Devirt<'p> {
        let mut pass = Devirt {
            program,
            held: BTreeMap::new(),
            changed: true,
        };
        // The judge's grader calls these with values of its own.
        for &id in program.entry_fns.iter().chain(&program.judge_fns) {
            for slot in 0..program.fns[id].params.len() {
                pass.held.insert(Home::Param(id, slot), Held::Many);
            }
        }
        while pass.changed {
            pass.changed = false;
            pass.block(None, &program.setup);
            pass.block(None, &program.main);
            for (id, f) in program.fns.iter().enumerate() {
                pass.block(Some(id), &f.body);
            }
            for closure in &program.closures {
                pass.block(None, &closure.body);
            }
        }
        pass
    }

    /// The one closure `callee` can be, in fn `home` (`None` for the
    /// file's own lines and for closures).
    pub fn target(&self, home: Option<FnId>, callee: &Expr) -> Option<ClosureId> {
        match self.value(home, callee) {
            Some(Held::One(cid)) => Some(cid),
            _ => None,
        }
    }

    /// Parameter `slot` of fn `id` is always the same closure, so its
    /// calls go to the closure and the parameter itself may go unread.
    pub fn param_known(&self, id: FnId, slot: usize) -> bool {
        matches!(self.held.get(&Home::Param(id, slot)), Some(Held::One(_)))
    }

    /// `None`: nothing has been stored there yet.
    fn value(&self, home: Option<FnId>, e: &Expr) -> Option<Held> {
        match &e.kind {
            ExprKind::Closure(cid) if self.program.closures[*cid].captures.is_empty() => {
                Some(Held::One(*cid))
            }
            ExprKind::Local(slot) => match home {
                Some(id) if *slot < self.program.fns[id].params.len() => {
                    self.held.get(&Home::Param(id, *slot)).copied()
                }
                _ => Some(Held::Many),
            },
            ExprKind::Field(base, index) => match (&base.ty, pure_path(base)) {
                (Ty::Struct(s), true) => self.held.get(&Home::Field(*s, *index)).copied(),
                _ => Some(Held::Many),
            },
            _ => Some(Held::Many),
        }
    }

    fn store(&mut self, to: Home, value: Option<Held>) {
        let Some(value) = value else { return };
        let joined = match self.held.get(&to) {
            None => value,
            Some(old) if *old == value || *old == Held::Many => return,
            Some(_) => Held::Many,
        };
        self.held.insert(to, joined);
        self.changed = true;
    }

    fn block(&mut self, home: Option<FnId>, block: &Block) {
        walk::deep_stmts(block, &mut |stmt| {
            let Stmt::Assign(place, _, value) = stmt else {
                return;
            };
            if !matches!(place.ty, Ty::Fn(..)) {
                return;
            }
            match (&place.kind, home) {
                (PlaceKind::Field(base, index), _) => {
                    if let Ty::Struct(s) = base.ty {
                        self.store(Home::Field(s, *index), self.value(home, value));
                    }
                }
                (PlaceKind::Local(slot), Some(id)) => {
                    self.store(Home::Param(id, *slot), Some(Held::Many));
                }
                _ => {}
            }
        });
        walk::deep_exprs(block, &mut |e| match (&e.kind, &e.ty) {
            (ExprKind::Tuple(fields), Ty::Struct(s)) => {
                for (index, field) in fields.iter().enumerate() {
                    if matches!(field.ty, Ty::Fn(..)) {
                        self.store(Home::Field(*s, index), self.value(home, field));
                    }
                }
            }
            (ExprKind::Call(callee, args), _) => {
                for (slot, arg) in args.iter().enumerate() {
                    match arg {
                        Arg::Value(value) if matches!(value.ty, Ty::Fn(..)) => {
                            self.store(Home::Param(*callee, slot), self.value(home, value));
                        }
                        Arg::Mut(place) if matches!(place.ty, Ty::Fn(..)) => {
                            self.store(Home::Param(*callee, slot), Some(Held::Many));
                            // The callee may store anything in the caller's.
                            if let (PlaceKind::Field(base, index), Ty::Struct(s)) =
                                (&place.kind, place_base_ty(place))
                            {
                                let _ = base;
                                self.store(Home::Field(*s, *index), Some(Held::Many));
                            }
                            if let (PlaceKind::Local(slot), Some(id)) = (&place.kind, home) {
                                self.store(Home::Param(id, *slot), Some(Held::Many));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        });
    }
}

fn place_base_ty(place: &cpl_ir::Place) -> &Ty {
    match &place.kind {
        PlaceKind::Field(base, _) => &base.ty,
        _ => &place.ty,
    }
}

/// A variable or a field of one: reading it runs nothing.
fn pure_path(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Local(_) | ExprKind::Global(_) => true,
        ExprKind::Field(base, _) => pure_path(base),
        _ => false,
    }
}

/// Whether evaluating `callee` can be skipped when its value is known.
pub(crate) fn skippable(callee: &Expr) -> bool {
    pure_path(callee)
}

/// The name of closure `cid` as a fn of the emitted program.
pub(crate) fn name(cid: ClosureId) -> String {
    format!("cpl_fn{cid}")
}
