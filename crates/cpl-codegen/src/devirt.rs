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

use crate::uses::effects::{in_domain, pure_path, safe_place};
use std::collections::{BTreeMap, BTreeSet};

use cpl_ir::{
    Arg, AssignOp, Block, ClosureId, Expr, ExprKind, FnId, Place, PlaceKind, Program, Stmt, Ty,
};

use crate::emit::Emitter;
use crate::walk;

mod explain;

mod elide;

mod escape;

pub use explain::explain_dropped_fields;
pub(crate) use explain::{dropped_fields, explain_closures};

/// How a fn-typed slot (struct field, fn parameter, or local closure binding)
/// lowers to C++17.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClosureMode {
    /// Single statically known zero-capture closure: called directly as
    /// `cpl_fnN(...)`, and the `std::function` storage/parameter is elided.
    StaticDirect { closure: ClosureId },
    /// C++ function template functor parameter (`template <class F>`):
    /// lambdas (including capturing ones) are passed by type and inlined.
    StaticTemplate,
    /// Local `auto` closure binding: no `std::function` wrapper or heap
    /// allocation.
    StaticLocalAuto,
    /// A fn field nothing reads: dropped, whatever was stored in it.
    Unread,
    /// A fn field of a struct the program never builds: no value holds
    /// it, so it is an empty `cpl::NoFn` and its calls never run.
    Unbuilt,
    /// Every closure that can reach it captures nothing: a plain function
    /// pointer, no `std::function`.
    FnPointer,
    /// Genuinely dynamic fn value: falls back to `std::function`.
    DynamicStdFunction { reason: String },
}

/// The mode of a fn slot of type `ty` that no closure is known for: a function
/// pointer when its arity only ever holds captureless closures.
fn erased(program: &Program, ty: &Ty, reason: &str) -> ClosureMode {
    let fn_ty = match ty {
        Ty::Opt(inner) => inner.as_ref(),
        t => t,
    };
    match fn_ty {
        Ty::Fn(args, _) if crate::names::fn_pointer(program, args.len()) => ClosureMode::FnPointer,
        _ => ClosureMode::DynamicStdFunction {
            reason: reason.to_string(),
        },
    }
}

/// One fn-typed site reported by `cpl explain <file>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClosureLowering {
    pub site: String,
    pub mode: ClosureMode,
}

/// Somewhere a fn value is kept.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Home {
    Field(usize, usize),
    Param(FnId, usize),
}

/// What a local bound once holds: see [`made_by`].
#[derive(Clone, Copy)]
enum Made {
    /// The struct a call to this fn returned.
    Call(FnId),
    /// Fn field `index` of the struct in local `slot`.
    Field(usize, usize),
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
    elidable: BTreeSet<Home>,
    template_params: BTreeMap<FnId, Vec<usize>>,
    reassigned_params: BTreeSet<(FnId, usize)>,
    recursive_fns: BTreeSet<FnId>,
    unread_fields: BTreeSet<(usize, usize)>,
    unbuilt: BTreeSet<usize>,
    dead_locals: BTreeSet<(Option<FnId>, usize)>,
    /// Locals never read whose one `let` may trap or have an effect:
    /// the value is still computed, as a statement, but not bound.
    unread_lets: BTreeSet<(Option<FnId>, usize)>,
    /// What fn field `(fn, index)` of every struct `fn` returns holds.
    returned: BTreeMap<(FnId, usize), Held>,
    /// Locals bound once and never written again: `let x = f(...)` of a
    /// struct (`x` is what `f` returned), or `let g = x.op` of a fn field.
    made_by: BTreeMap<(FnId, usize), Made>,
    /// Each closure's stand-in: the first closure that only calls the same fn.
    same: Vec<ClosureId>,
    /// Fn-typed globals set once to a closure: each call goes to the
    /// closure, each read names it, and the global is never stored.
    globals: BTreeMap<usize, ClosureId>,
    changed: bool,
}

impl<'p> Devirt<'p> {
    pub fn collect(program: &'p Program) -> Devirt<'p> {
        Self::collect_with(program, same_closures(program))
    }

    /// [`Devirt::collect`] with every closure standing for itself, for a
    /// profiled build, which counts each closure's own line.
    pub fn collect_unmerged(program: &'p Program) -> Devirt<'p> {
        Self::collect_with(program, (0..program.closures.len()).collect())
    }

    fn collect_with(program: &'p Program, same: Vec<ClosureId>) -> Devirt<'p> {
        let mut pass = Devirt {
            program,
            held: BTreeMap::new(),
            elidable: BTreeSet::new(),
            template_params: BTreeMap::new(),
            reassigned_params: BTreeSet::new(),
            recursive_fns: recursive_fns(program),
            unread_fields: BTreeSet::new(),
            unbuilt: crate::built::unbuilt(program, None),
            dead_locals: BTreeSet::new(),
            unread_lets: BTreeSet::new(),
            returned: BTreeMap::new(),
            made_by: made_by(program),
            same,
            globals: BTreeMap::new(),
            changed: true,
        };
        // The judge's grader calls these with values of its own.
        for &id in program.entry_fns.iter().chain(&program.judge_fns) {
            for slot in 0..program.fns[id].params.len() {
                pass.held.insert(Home::Param(id, slot), Held::Many);
            }
        }
        // Fn globals set once to a captureless closure are known from the start.
        pass.globals = pass.called_globals();
        while pass.changed {
            pass.changed = false;
            pass.block(None, &program.setup);
            pass.block(None, &program.main);
            for (id, f) in program.fns.iter().enumerate() {
                pass.block(Some(id), &f.body);
                pass.returns(id, &f.body);
            }
            for closure in &program.closures {
                pass.block(None, &closure.body);
            }
        }
        pass.compute_elidable();
        pass.compute_template_params();
        pass.globals = pass.called_globals();
        pass
    }

    /// See [`Devirt::globals`].
    fn called_globals(&self) -> BTreeMap<usize, ClosureId> {
        let program = self.program;
        let mut set: BTreeMap<usize, Vec<Option<ClosureId>>> = BTreeMap::new();
        let mut blocks = vec![&program.setup, &program.main];
        blocks.extend(program.fns.iter().map(|f| &f.body));
        blocks.extend(program.closures.iter().map(|c| &c.body));
        for block in blocks {
            walk::deep_stmts(block, &mut |stmt| {
                if let Stmt::Assign(place, op, value) = stmt {
                    if let PlaceKind::Global(g) = place.kind {
                        let cid = match value.kind {
                            ExprKind::Closure(cid) if *op == AssignOp::Set => Some(cid),
                            _ => None,
                        };
                        set.entry(g).or_default().push(cid);
                    }
                }
            });
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Call(_, args) = &e.kind {
                    for arg in args {
                        if let Arg::Mut(place) = arg {
                            if let (true, g) = walk::root(place) {
                                set.entry(g).or_default().push(None);
                            }
                        }
                    }
                }
            });
        }
        let captured: BTreeSet<usize> = program
            .closures
            .iter()
            .flat_map(|c| c.captures.iter().filter(|c| c.is_global).map(|c| c.outer))
            .collect();
        set.into_iter()
            .filter(|(g, _)| matches!(program.globals[*g].ty, Ty::Fn(..)) && !captured.contains(g))
            .filter_map(|(g, cids)| match cids.as_slice() {
                [Some(cid)] if self.closure_effectively_captureless(*cid) => {
                    Some((g, self.same[*cid]))
                }
                _ => None,
            })
            .collect()
    }

    /// Global `g` is a fn set once to a closure: it is not stored, and
    /// its calls and reads go to the closure.
    pub fn global_direct(&self, g: usize) -> bool {
        self.globals.contains_key(&g)
    }

    /// The closure global `g` always holds (see [`Devirt::global_direct`]).
    pub fn global_target(&self, g: usize) -> Option<ClosureId> {
        self.globals.get(&g).copied()
    }

    /// The one closure `callee` can be, in fn `home` (`None` for the
    /// file's own lines and for closures).
    pub fn target(&self, home: Option<FnId>, callee: &Expr) -> Option<ClosureId> {
        match self.value(home, callee) {
            Some(Held::One(cid)) => Some(cid),
            _ => None,
        }
    }

    /// The closure emitted in place of `cid` (see [`same_closures`]).
    pub fn same(&self, cid: ClosureId) -> ClosureId {
        self.same[cid]
    }

    /// Parameter `slot` of fn `id` is always the same closure, so its
    /// calls go to the closure and the parameter itself may go unread.
    pub fn param_known(&self, id: FnId, slot: usize) -> bool {
        matches!(self.held.get(&Home::Param(id, slot)), Some(Held::One(_)))
    }

    pub fn param_target(&self, id: FnId, slot: usize) -> Option<ClosureId> {
        match self.held.get(&Home::Param(id, slot)) {
            Some(Held::One(cid)) => Some(*cid),
            _ => None,
        }
    }

    /// Struct field `(struct_id, index)` is a single statically known closure
    /// that never escapes to a dynamic `std::function`, so its `std::function`
    /// storage is replaced by a 1-byte `bool` placeholder.
    pub fn field_elidable(&self, struct_id: usize, index: usize) -> bool {
        self.elidable.contains(&Home::Field(struct_id, index))
    }

    /// Fn parameter `(id, slot)` is a single statically known closure that
    /// never escapes, so its `std::function` parameter is replaced by a
    /// `[[maybe_unused]] bool` placeholder and callers pass `false`.
    pub fn param_elidable(&self, id: FnId, slot: usize) -> bool {
        self.elidable.contains(&Home::Param(id, slot))
    }

    /// Parameter slots of `id` lowered as C++ template functor parameters
    /// (`template <class F0, ...>`).
    pub fn template_params(&self, id: FnId) -> &[usize] {
        self.template_params
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Index into `template_params(id)` if `slot` is a template functor param.
    pub fn template_param_index(&self, id: FnId, slot: usize) -> Option<usize> {
        self.template_params(id).iter().position(|&s| s == slot)
    }

    /// Whether a non-hoisted, non-reassigned local `let name = <closure>` can
    /// be emitted as `auto name = ...;` instead of `std::function<...>`.
    pub fn local_auto(&self, slot: usize, cid: ClosureId) -> bool {
        !self.program.closures[cid]
            .captures
            .iter()
            .any(|c| !c.is_global && c.outer == slot)
    }

    /// Whether `let name = <closure cid>` in `home` (a closure, or the fn when None) binds `auto`.
    pub fn let_auto(&self, home: Option<ClosureId>, slot: usize, cid: ClosureId, n: u32) -> bool {
        let mut stored = home.is_none() && n > 1;
        if let Some(h) = home {
            walk::deep_stmts(&self.program.closures[h].body, &mut |s| {
                stored |= matches!(s, Stmt::Assign(p, _, _) if p.kind == PlaceKind::Local(slot));
            });
        }
        !stored && self.local_auto(slot, cid)
    }

    /// Whether a local `let name = <closure>` that names itself can bind
    /// `auto`: the closure takes itself as a parameter instead of capturing
    /// its own `std::function`.
    pub fn local_rec(&self, slot: usize, cid: ClosureId) -> bool {
        let caps = &self.program.closures[cid].captures;
        caps.iter().any(|c| !c.is_global && c.outer == slot)
            && caps
                .iter()
                .all(|c| c.is_global || c.outer != slot || c.by_ref)
    }

    fn compute_template_params(&mut self) {
        for (id, f) in self.program.fns.iter().enumerate() {
            if self.program.entry_fns.contains(&id)
                || self.program.judge_fns.contains(&id)
                || is_hook_fn(self.program, id)
            {
                continue;
            }
            let slots: Vec<usize> = (0..f.params.len())
                .filter(|&slot| {
                    matches!(f.locals[slot].ty, Ty::Fn(..))
                        && !f.params[slot].mutable
                        && !self.elidable.contains(&Home::Param(id, slot))
                        && !self.reassigned_params.contains(&(id, slot))
                })
                .collect();
            if !slots.is_empty() {
                self.template_params.insert(id, slots);
            }
        }
        self.keep_recursive_templates_finite();
    }

    /// A recursive fn's fn parameter stays a template only when every call
    /// to it from inside its cycle passes a template fn parameter of the
    /// caller straight on, so the recursion instantiates one closure type
    /// and never wraps it in a new one.
    fn keep_recursive_templates_finite(&mut self) {
        let fns = &self.program.fns;
        let n = fns.len();
        let mut calls: Vec<Vec<(FnId, &Vec<Arg>)>> = vec![Vec::new(); n];
        for (g, f) in fns.iter().enumerate() {
            walk::deep_exprs(&f.body, &mut |e| {
                if let ExprKind::Call(callee, args) = &e.kind {
                    calls[g].push((*callee, args));
                }
            });
        }
        let reach: Vec<BTreeSet<FnId>> = (0..n)
            .map(|start| {
                let mut seen = BTreeSet::new();
                let mut stack: Vec<FnId> = calls[start].iter().map(|c| c.0).collect();
                while let Some(u) = stack.pop() {
                    if seen.insert(u) {
                        stack.extend(calls[u].iter().map(|c| c.0));
                    }
                }
                seen
            })
            .collect();
        loop {
            let mut dropped = Vec::new();
            for (&id, slots) in &self.template_params {
                if !self.recursive_fns.contains(&id) {
                    continue;
                }
                for &slot in slots {
                    let forwarded = (0..n)
                        .filter(|&g| reach[id].contains(&g) && reach[g].contains(&id))
                        .all(|g| {
                            calls[g].iter().filter(|c| c.0 == id).all(|(_, args)| {
                                matches!(args.get(slot), Some(Arg::Value(Expr { kind: ExprKind::Local(p), .. }))
                                    if *p < fns[g].params.len()
                                        && self.template_params.get(&g).is_some_and(|s| s.contains(p)))
                            })
                        });
                    if !forwarded {
                        dropped.push((id, slot));
                    }
                }
            }
            if dropped.is_empty() {
                break;
            }
            for (id, slot) in dropped {
                let slots = self.template_params.get_mut(&id).expect("a template fn");
                slots.retain(|&s| s != slot);
                if slots.is_empty() {
                    self.template_params.remove(&id);
                }
            }
        }
    }

    /// `None`: nothing has been stored there yet.
    fn value(&self, home: Option<FnId>, e: &Expr) -> Option<Held> {
        match &e.kind {
            ExprKind::Closure(cid) if self.closure_effectively_captureless(*cid) => {
                Some(Held::One(self.same[*cid]))
            }
            ExprKind::Local(slot) => match home {
                Some(id) if *slot < self.program.fns[id].params.len() => {
                    self.held.get(&Home::Param(id, *slot)).copied()
                }
                Some(id) => match self.made_by.get(&(id, *slot)) {
                    Some(Made::Field(base, index)) => Some(
                        self.made_slot(id, *base, *index)
                            .or_else(|| self.only_closure_of(id, *base, *index))
                            .unwrap_or(Held::Many),
                    ),
                    _ => Some(Held::Many),
                },
                _ => Some(Held::Many),
            },
            ExprKind::Global(g) => Some(self.globals.get(g).map_or(Held::Many, |c| Held::One(*c))),
            ExprKind::Field(base, index) if self.made(home, base, *index).is_some() => {
                self.made(home, base, *index)
            }
            ExprKind::Field(base, index) => match &base.ty {
                Ty::Struct(s) => self.held.get(&Home::Field(*s, *index)).copied(),
                _ => Some(Held::Many),
            },
            _ => Some(Held::Many),
        }
    }

    /// True if closure `cid` has no live captures in emitted C++: either
    /// `captures` is empty, or every captured local is only ever read as the
    /// callee of a `StaticDirect` `CallValue` (which lowers to a direct
    /// `cpl_fnN(...)` call and never touches the captured variable).
    pub(crate) fn closure_effectively_captureless(&self, cid: ClosureId) -> bool {
        let def = &self.program.closures[cid];
        if def.captures.is_empty() {
            return true;
        }
        let cap_slots: BTreeSet<usize> = def.captures.iter().map(|c| c.inner).collect();
        self.block_captures_unused(&def.body, &cap_slots)
    }

    /// Whether evaluating `e` runs nothing and cannot trap, so it can be
    /// skipped when its destination is elided.
    pub(crate) fn skippable_value(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Unit
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Char(_)
            | ExprKind::None
            | ExprKind::DomainLen(_)
            | ExprKind::DomainLo(_) => true,
            ExprKind::Some(inner) => self.skippable_value(inner),
            ExprKind::Index(base, idxs) => in_domain(&base.ty, idxs) && self.skippable_value(base),
            ExprKind::Tuple(items) => items.iter().all(|item| self.skippable_value(item)),
            ExprKind::Closure(cid) => self.closure_effectively_captureless(*cid),
            ExprKind::Fill(dims, value) => {
                dims.iter().all(|d| match d {
                    cpl_ir::DimSpec::Int(n) | cpl_ir::DimSpec::IntFrom(_, n) => {
                        matches!(n.kind, ExprKind::Int(v) if (0..=1 << 20).contains(&v))
                    }
                    _ => true,
                }) && value.as_ref().is_none_or(|v| self.skippable_value(v))
            }
            _ => pure_path(e),
        }
    }

    pub fn unread_field(&self, struct_id: usize, index: usize) -> bool {
        self.unread_fields.contains(&(struct_id, index))
    }

    /// Count only the struct literals in `fns` and `closures`, the code the
    /// file writes: a literal in a fn that is never written never runs.
    pub fn built_only_in(&mut self, fns: &BTreeSet<FnId>, closures: &BTreeSet<ClosureId>) {
        self.unbuilt = crate::built::unbuilt(self.program, Some((fns, closures)));
    }

    /// Fn field `index` of a struct no expression builds, and so no value
    /// holds: see [`crate::built::unbuilt`].
    pub fn unbuilt_field(&self, struct_id: usize, index: usize) -> bool {
        self.unbuilt.contains(&struct_id)
            && matches!(self.program.structs[struct_id].fields[index].1, Ty::Fn(..))
            && !self.unread_field(struct_id, index)
            && !self.field_elidable(struct_id, index)
    }

    /// `xs.push(v)` onto a local list, with `v` running nothing: left out
    /// when the list is a dropped local.
    pub(crate) fn skippable_push(&self, e: &Expr) -> Option<usize> {
        match &e.kind {
            ExprKind::Mutate(cpl_ir::Mutation::Push, place, args) => match place.kind {
                PlaceKind::Local(slot) if args.iter().all(|a| self.skippable_value(a)) => {
                    Some(slot)
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub fn dead_local(&self, home: Option<FnId>, slot: usize) -> bool {
        self.dead_locals.contains(&(home, slot))
    }

    /// Whether `let slot = v` only computes `v`: nothing reads `slot`.
    pub fn unread_let(&self, home: Option<FnId>, slot: usize) -> bool {
        self.unread_lets.contains(&(home, slot))
    }

    /// Whether `place` is part of a dropped field.
    pub fn in_unread_field(&self, place: &cpl_ir::Place) -> bool {
        let mut fields = BTreeSet::new();
        place_fields(place, &mut fields);
        fields.iter().any(|key| self.unread_fields.contains(key))
    }

    /// `place = value` into a dropped destination writes nothing: every
    /// index is a position of the domain its array is over, so the place
    /// cannot trap, and the value is still evaluated for its traps and
    /// effects unless it runs nothing.
    pub(crate) fn skippable_store(&self, place: &cpl_ir::Place, op: AssignOp) -> bool {
        op == AssignOp::Set && safe_place(place)
    }

    /// The one closure field `index` of local `base` holds, when `base`
    /// is a struct a fn made and nothing wrote since.
    fn made(&self, home: Option<FnId>, base: &Expr, index: usize) -> Option<Held> {
        let (ExprKind::Local(slot), Some(id)) = (&base.kind, home) else {
            return None;
        };
        self.made_slot(id, *slot, index)
    }

    /// Fn field `index` of the struct in local `slot` when every store to
    /// that field, in any instance, is the same closure: `let f = mon.op`
    /// calls it directly wherever `mon` came from.
    fn only_closure_of(&self, id: FnId, slot: usize, index: usize) -> Option<Held> {
        let Ty::Struct(s) = self.program.fns[id].locals[slot].ty else {
            return None;
        };
        self.held
            .get(&Home::Field(s, index))
            .copied()
            .filter(|held| matches!(held, Held::One(_)))
    }

    fn made_slot(&self, id: FnId, slot: usize, index: usize) -> Option<Held> {
        let Some(Made::Call(maker)) = self.made_by.get(&(id, slot)) else {
            return None;
        };
        self.returned
            .get(&(*maker, index))
            .copied()
            .filter(|held| matches!(held, Held::One(_)))
    }

    /// Joins the fn fields of each struct fn `id` returns.
    fn returns(&mut self, id: FnId, body: &Block) {
        let mut seen = Vec::new();
        walk::deep_stmts(body, &mut |stmt| {
            if let Stmt::Return(Some(e)) = stmt {
                seen.push(e);
            }
        });
        for e in seen {
            let Ty::Struct(s) = e.ty else { continue };
            for (index, field) in self.program.structs[s].fields.iter().enumerate() {
                if !matches!(field.1, Ty::Fn(..)) {
                    continue;
                }
                let held = match &e.kind {
                    ExprKind::Tuple(items) => self.value(Some(id), &items[index]),
                    _ => Some(Held::Many),
                };
                let Some(held) = held else { continue };
                let joined = match self.returned.get(&(id, index)) {
                    None => held,
                    Some(old) if *old == held || *old == Held::Many => continue,
                    Some(_) => Held::Many,
                };
                self.returned.insert((id, index), joined);
                self.changed = true;
            }
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

/// Each fn's locals bound once, to a call made struct or to a fn field of
/// a local, and never written again, in place, through `mut` or by a closure.
fn made_by(program: &Program) -> BTreeMap<(FnId, usize), Made> {
    let mut out = BTreeMap::new();
    for (id, f) in program.fns.iter().enumerate() {
        let mut lets: BTreeMap<usize, Option<Made>> = BTreeMap::new();
        let mut written = BTreeSet::new();
        walk::deep_stmts(&f.body, &mut |stmt| match stmt {
            Stmt::Let(slot, init) => {
                let maker = match init.as_ref().map(|e| (&e.kind, &e.ty)) {
                    Some((ExprKind::Call(callee, _), Ty::Struct(_))) => Some(Made::Call(*callee)),
                    Some((ExprKind::Field(base, index), Ty::Fn(..))) => match base.kind {
                        ExprKind::Local(b) => Some(Made::Field(b, *index)),
                        _ => None,
                    },
                    _ => None,
                };
                let again = lets.contains_key(slot);
                lets.insert(*slot, maker.filter(|_| !again));
            }
            Stmt::Assign(place, _, _) => {
                written.insert(place_root(place));
            }
            Stmt::For(_, slot, _, _) => {
                written.insert(Some(*slot));
            }
            _ => {}
        });
        walk::deep_exprs(&f.body, &mut |e| match &e.kind {
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        written.insert(place_root(place));
                    }
                }
            }
            ExprKind::Closure(cid) => {
                for cap in &program.closures[*cid].captures {
                    if cap.by_ref && !cap.is_global {
                        written.insert(Some(cap.outer));
                    }
                }
            }
            ExprKind::Mutate(_, place, _) => {
                written.insert(place_root(place));
            }
            _ => {}
        });
        for (slot, maker) in lets {
            if let (Some(maker), false) = (maker, written.contains(&Some(slot))) {
                out.insert((id, slot), maker);
            }
        }
    }
    out
}

/// The local a place writes into, if it is one.
fn place_root(place: &cpl_ir::Place) -> Option<usize> {
    match &place.kind {
        PlaceKind::Local(slot) => Some(*slot),
        PlaceKind::Global(_) => None,
        PlaceKind::Field(base, _) | PlaceKind::Index(base, _) | PlaceKind::Some(base) => {
            place_root(base)
        }
    }
}

fn place_base_ty(place: &cpl_ir::Place) -> &Ty {
    match &place.kind {
        PlaceKind::Field(base, _) => &base.ty,
        _ => &place.ty,
    }
}

/// The struct fields `place` goes through.
fn place_fields(place: &cpl_ir::Place, out: &mut BTreeSet<(usize, usize)>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, idx) => {
            if let Ty::Struct(s) = base.ty {
                out.insert((s, *idx));
            }
            place_fields(base, out);
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => place_fields(base, out),
    }
}

/// Every struct a value of type `ty` holds.
pub(crate) fn ty_structs(ty: &Ty, out: &mut BTreeSet<usize>) {
    match ty {
        Ty::Struct(s) => {
            out.insert(*s);
        }
        Ty::Array { elem, .. }
        | Ty::Opt(elem)
        | Ty::List(elem)
        | Ty::ListView(elem, _)
        | Ty::Keyed(elem, _)
        | Ty::MutView(elem, _)
        | Ty::Set(elem)
        | Ty::HashSet(elem)
        | Ty::Mut(elem)
        | Ty::Range(elem) => ty_structs(elem, out),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            ty_structs(k, out);
            ty_structs(v, out);
        }
        Ty::Tuple(items) => items.iter().for_each(|t| ty_structs(t, out)),
        Ty::Record(items) => items.iter().for_each(|(_, t)| ty_structs(t, out)),
        Ty::Fn(args, ret) => {
            args.iter().for_each(|t| ty_structs(t, out));
            ty_structs(ret, out);
        }
        _ => {}
    }
}

/// Every struct used in a key of a map or set inside `ty`: hashing and
/// ordering keys reads all their fields.
fn key_structs(ty: &Ty, out: &mut BTreeSet<usize>) {
    match ty {
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            ty_structs(k, out);
            key_structs(v, out);
        }
        Ty::Set(k) | Ty::HashSet(k) => ty_structs(k, out),
        Ty::Array { elem, .. }
        | Ty::Opt(elem)
        | Ty::List(elem)
        | Ty::ListView(elem, _)
        | Ty::Keyed(elem, _)
        | Ty::MutView(elem, _)
        | Ty::Mut(elem)
        | Ty::Range(elem) => key_structs(elem, out),
        Ty::Tuple(items) => items.iter().for_each(|t| key_structs(t, out)),
        Ty::Record(items) => items.iter().for_each(|(_, t)| key_structs(t, out)),
        Ty::Fn(args, ret) => {
            args.iter().for_each(|t| key_structs(t, out));
            key_structs(ret, out);
        }
        _ => {}
    }
}

/// Whether `e` only moves the values of its operands around, reading no
/// struct field it does not name.
fn keeps_fields(e: &Expr) -> bool {
    use cpl_ir::Builtin as B;
    match &e.kind {
        ExprKind::Some(_)
        | ExprKind::Unwrap(..)
        | ExprKind::IsSome(_)
        | ExprKind::Field(..)
        | ExprKind::Index(..)
        | ExprKind::Tuple(_)
        | ExprKind::Variant(..)
        | ExprKind::Payload(..)
        | ExprKind::If(..)
        | ExprKind::Call(..)
        | ExprKind::CallExpr(..)
        | ExprKind::Mutate(..)
        | ExprKind::Fill(..) => true,
        ExprKind::Builtin(b, _) => matches!(
            b,
            B::Len
                | B::First
                | B::Last
                | B::FirstVal
                | B::LastVal
                | B::Copy
                | B::ListFilled
                | B::SeqGet
                | B::ArrayToList
                | B::ListToArray
                | B::ListSlice
                | B::Reversed
                | B::ListConcat
        ),
        _ => false,
    }
}

fn is_fn_or_opt_fn(ty: &Ty) -> bool {
    match ty {
        Ty::Fn(..) => true,
        Ty::Opt(inner) => matches!(**inner, Ty::Fn(..)),
        _ => false,
    }
}

/// Whether evaluating `callee` can be skipped when its value is known.
pub(crate) fn skippable(callee: &Expr) -> bool {
    pure_path(callee)
}

/// A known callee called directly: a path, or a field of any base, which is
/// still evaluated (`act!.apply(..)` keeps the `act!` check) before the
/// arguments.
pub(crate) fn direct_callee(callee: &Expr) -> bool {
    pure_path(callee) || matches!(callee.kind, ExprKind::Field(..) | ExprKind::Closure(_))
}

/// The name of closure `cid` as a fn of the emitted program.
/// For each closure, the first one that does the same: a fn named as a value,
/// `op: merge`, becomes a closure calling `merge` at each place it is named,
/// and one field given `merge` twice still holds one fn. Any other closure
/// stands for itself.
pub(crate) fn same_closures(program: &Program) -> Vec<ClosureId> {
    let mut first: BTreeMap<(FnId, usize), ClosureId> = BTreeMap::new();
    (0..program.closures.len())
        .map(|cid| match forwards_to(program, cid) {
            Some(fid) => *first
                .entry((fid, program.closures[cid].params.len()))
                .or_insert(cid),
            None => cid,
        })
        .collect()
}

/// The fn closure `cid` only calls, with its own parameters in order.
fn forwards_to(program: &Program, cid: ClosureId) -> Option<FnId> {
    let c = &program.closures[cid];
    if !c.captures.is_empty() || !c.muts.is_empty() {
        return None;
    }
    let call = match c.body.stmts.as_slice() {
        [Stmt::Return(Some(e))] | [Stmt::Expr(e)] => e,
        _ => return None,
    };
    let ExprKind::Call(fid, args) = &call.kind else {
        return None;
    };
    let own = args.len() == c.params.len()
        && args.iter().zip(&c.params).all(
            |(a, &p)| matches!(a, Arg::Value(Expr { kind: ExprKind::Local(s), .. }) if *s == p),
        );
    own.then_some(*fid)
}

pub(crate) fn name(cid: ClosureId) -> String {
    format!("cpl_fn{cid}")
}

fn is_hook_fn(program: &Program, id: FnId) -> bool {
    program.is_hook(id)
}

fn recursive_fns(program: &Program) -> BTreeSet<FnId> {
    let n = program.fns.len();
    let mut adj: Vec<BTreeSet<FnId>> = vec![BTreeSet::new(); n];
    for (id, f) in program.fns.iter().enumerate() {
        walk::deep_exprs(&f.body, &mut |e| {
            if let ExprKind::Call(callee, _) = e.kind {
                adj[id].insert(callee);
            }
        });
    }
    let mut rec = BTreeSet::new();
    for start in 0..n {
        let mut seen = BTreeSet::new();
        let mut stack: Vec<FnId> = adj[start].iter().copied().collect();
        while let Some(u) = stack.pop() {
            if u == start {
                rec.insert(start);
                break;
            }
            if seen.insert(u) {
                stack.extend(adj[u].iter().copied());
            }
        }
    }
    rec
}

impl Emitter<'_> {
    /// A store into a dropped field or local (decision 178).
    pub(crate) fn dropped_store(&self, place: &Place, op: AssignOp) -> bool {
        let dead_root = match crate::walk::root(place) {
            (false, slot) => self.closure_id.is_none() && self.devirt.dead_local(self.fn_id, slot),
            (true, g) => matches!(place.kind, PlaceKind::Global(_)) && self.devirt.global_direct(g),
        };
        (dead_root || self.devirt.in_unread_field(place)) && self.devirt.skippable_store(place, op)
    }

    pub(crate) fn dropped_value(&mut self, value: &Expr, out: &mut String) {
        if !self.devirt.skippable_value(value) {
            let value = self.flatten(value, out);
            let text = self.expr(value).text;
            self.line(out, &format!("(void)({text});"));
        }
    }
}
