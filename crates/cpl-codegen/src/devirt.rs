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

use std::collections::{BTreeMap, BTreeSet};

use cpl_ir::{Arg, AssignOp, Block, ClosureId, Expr, ExprKind, FnId, PlaceKind, Program, Stmt, Ty};

use crate::walk;

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
    /// Genuinely dynamic fn value: falls back to `std::function`.
    DynamicStdFunction { reason: String },
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
    changed: bool,
}

impl<'p> Devirt<'p> {
    pub fn collect(program: &'p Program) -> Devirt<'p> {
        let mut pass = Devirt {
            program,
            held: BTreeMap::new(),
            elidable: BTreeSet::new(),
            template_params: BTreeMap::new(),
            reassigned_params: BTreeSet::new(),
            recursive_fns: recursive_fns(program),
            unread_fields: BTreeSet::new(),
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
        pass.compute_elidable();
        pass.compute_template_params();
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
        self.template_params.get(&id).map(Vec::as_slice).unwrap_or(&[])
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

    /// Whether a local `let name = <closure>` that names itself can bind
    /// `auto`: the closure takes itself as a parameter instead of capturing
    /// its own `std::function`.
    pub fn local_rec(&self, slot: usize, cid: ClosureId) -> bool {
        let caps = &self.program.closures[cid].captures;
        caps.iter().any(|c| !c.is_global && c.outer == slot)
            && caps.iter().all(|c| c.is_global || c.outer != slot || c.by_ref)
    }

    fn compute_elidable(&mut self) {
        let mut assigned_fields: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut captured_params: BTreeSet<(FnId, usize)> = BTreeSet::new();
        let mut scan_assigns = |home: Option<FnId>, block: &Block| {
            walk::deep_stmts(block, &mut |stmt| {
                if let Stmt::Assign(place, _, _) = stmt {
                    if let (PlaceKind::Field(base, index), Ty::Struct(s)) = (&place.kind, place_base_ty(place)) {
                        let _ = base;
                        assigned_fields.insert((*s, *index));
                    }
                    if let (PlaceKind::Local(slot), Some(id)) = (&place.kind, home) {
                        if *slot < self.program.fns[id].params.len() {
                            self.reassigned_params.insert((id, *slot));
                        }
                    }
                }
            });
            walk::deep_exprs(block, &mut |e| {
                if let (ExprKind::Closure(cid), Some(id)) = (&e.kind, home) {
                    for cap in &self.program.closures[*cid].captures {
                        if !cap.is_global && cap.outer < self.program.fns[id].params.len() {
                            captured_params.insert((id, cap.outer));
                        }
                    }
                }
                if let ExprKind::Call(_, args) = &e.kind {
                    for arg in args {
                        if let Arg::Mut(place) = arg {
                            if let (PlaceKind::Field(_, index), Ty::Struct(s)) = (&place.kind, place_base_ty(place)) {
                                assigned_fields.insert((*s, *index));
                            }
                            if let (PlaceKind::Local(slot), Some(id)) = (&place.kind, home) {
                                if *slot < self.program.fns[id].params.len() {
                                    self.reassigned_params.insert((id, *slot));
                                }
                            }
                        }
                    }
                }
            });
        };
        scan_assigns(None, &self.program.setup);
        scan_assigns(None, &self.program.main);
        for (id, f) in self.program.fns.iter().enumerate() {
            scan_assigns(Some(id), &f.body);
        }
        for closure in &self.program.closures {
            scan_assigns(None, &closure.body);
        }

        for (&home, &held) in &self.held {
            if !matches!(held, Held::One(_)) {
                continue;
            }
            match home {
                Home::Field(s, idx) => {
                    if !assigned_fields.contains(&(s, idx)) {
                        self.elidable.insert(home);
                    }
                }
                Home::Param(id, slot) => {
                    let f = &self.program.fns[id];
                    let forbidden = self.program.entry_fns.contains(&id)
                        || self.program.judge_fns.contains(&id)
                        || is_hook_fn(self.program, id)
                        || f.params[slot].mutable
                        || self.reassigned_params.contains(&(id, slot))
                        || captured_params.contains(&(id, slot));
                    if !forbidden {
                        self.elidable.insert(home);
                    }
                }
            }
        }

        // Fixed-point shrink: remove any home read in an escaping position.
        loop {
            let mut removed = Vec::new();
            self.find_escapes(None, &self.program.setup, &mut removed);
            self.find_escapes(None, &self.program.main, &mut removed);
            for (id, f) in self.program.fns.iter().enumerate() {
                self.find_escapes(Some(id), &f.body, &mut removed);
            }
            for closure in &self.program.closures {
                self.find_escapes(None, &closure.body, &mut removed);
            }
            let before = self.elidable.len();
            for h in removed {
                self.elidable.remove(&h);
            }
            if self.elidable.len() == before {
                break;
            }
        }
        self.drop_unread_fn_fields();
    }

    /// A fn or `?fn` field nothing reads holds no behaviour, so it is
    /// elided like a known closure, whatever was stored in it.
    fn drop_unread_fn_fields(&mut self) {
        let mut read: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut unskippable: BTreeSet<(usize, usize)> = BTreeSet::new();
        let mut scan = |block: &Block| {
            walk::deep_stmts(block, &mut |stmt| {
                if let Stmt::Assign(place, op, value) = stmt {
                    if let (Some(key), false) = (field_key(place), self.skippable_field_store(place, *op, value)) {
                        unskippable.insert(key);
                    }
                }
            });
            walk::deep_exprs(block, &mut |e| match &e.kind {
                ExprKind::Field(base, idx) => {
                    if let Ty::Struct(s) = base.ty {
                        read.insert((s, *idx));
                    }
                }
                ExprKind::Call(_, args) => {
                    for arg in args {
                        if let Some(key) = match arg {
                            Arg::Mut(place) => field_key(place),
                            _ => None,
                        } {
                            unskippable.insert(key);
                        }
                    }
                }
                ExprKind::Tuple(items) => {
                    if let Ty::Struct(s) = e.ty {
                        for (idx, item) in items.iter().enumerate() {
                            if !self.skippable_value(item) {
                                unskippable.insert((s, idx));
                            }
                        }
                    }
                }
                _ => {}
            });
        };
        scan(&self.program.setup);
        scan(&self.program.main);
        for f in &self.program.fns {
            scan(&f.body);
        }
        for closure in &self.program.closures {
            scan(&closure.body);
        }
        for (sid, def) in self.program.structs.iter().enumerate() {
            for (idx, (_, ty)) in def.fields.iter().enumerate() {
                let key = (sid, idx);
                if is_fn_or_opt_fn(ty) && !read.contains(&key) && !unskippable.contains(&key) {
                    self.elidable.insert(Home::Field(sid, idx));
                    self.unread_fields.insert(key);
                }
            }
        }
    }

    fn find_escapes(&self, home: Option<FnId>, block: &Block, out: &mut Vec<Home>) {
        for stmt in &block.stmts {
            self.find_escapes_stmt(home, stmt, out);
        }
    }

    fn find_escapes_stmt(&self, home: Option<FnId>, stmt: &Stmt, out: &mut Vec<Home>) {
        match stmt {
            Stmt::Let(_, init) => {
                if let Some(e) = init {
                    self.find_escapes_expr(home, e, false, out);
                }
            }
            Stmt::Assign(place, _, value) => {
                walk::place_exprs(place, &mut |e| self.find_escapes_expr(home, e, false, out));
                self.find_escapes_expr(home, value, false, out);
            }
            Stmt::Expr(e) | Stmt::Give(e) | Stmt::DefDomainFromVal { val: e, .. } => {
                self.find_escapes_expr(home, e, false, out);
            }
            Stmt::Return(value) => {
                if let Some(e) = value {
                    self.find_escapes_expr(home, e, false, out);
                }
            }
            Stmt::If(cond, then, otherwise) => {
                self.find_escapes_expr(home, cond, false, out);
                self.find_escapes(home, then, out);
                self.find_escapes(home, otherwise, out);
            }
            Stmt::While(_, cond, body) => {
                self.find_escapes_expr(home, cond, false, out);
                self.find_escapes(home, body, out);
            }
            Stmt::For(_, _, iter, body) => {
                match iter {
                    cpl_ir::Iter::Range { lo, hi, step, .. } => {
                        self.find_escapes_expr(home, lo, false, out);
                        self.find_escapes_expr(home, hi, false, out);
                        if let Some(step) = step {
                            self.find_escapes_expr(home, step, false, out);
                        }
                    }
                    cpl_ir::Iter::RangeVal(e, _)
                    | cpl_ir::Iter::Seq(e, _)
                    | cpl_ir::Iter::Subsets(e)
                    | cpl_ir::Iter::Members(e) => {
                        self.find_escapes_expr(home, e, false, out);
                    }
                    cpl_ir::Iter::TreeRange { container, lo, hi, .. } => {
                        self.find_escapes_expr(home, container, false, out);
                        if let Some((e, _)) = lo {
                            self.find_escapes_expr(home, e, false, out);
                        }
                        if let Some((e, _)) = hi {
                            self.find_escapes_expr(home, e, false, out);
                        }
                    }
                }
                self.find_escapes(home, body, out);
            }
            Stmt::Assert { cond, message, .. } => {
                self.find_escapes_expr(home, cond, false, out);
                if let Some(msg) = message {
                    self.find_escapes_expr(home, msg, false, out);
                }
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.find_escapes_expr(home, lo, false, out);
                self.find_escapes_expr(home, len, false, out);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => {}
        }
    }

    fn find_escapes_expr(&self, home: Option<FnId>, e: &Expr, elided_ctx: bool, out: &mut Vec<Home>) {
        match &e.kind {
            ExprKind::Local(slot) => {
                if !elided_ctx {
                    if let Some(id) = home {
                        if *slot < self.program.fns[id].params.len() {
                            let h = Home::Param(id, *slot);
                            if self.elidable.contains(&h) {
                                out.push(h);
                            }
                        }
                    }
                }
            }
            ExprKind::Field(base, index) => {
                if !elided_ctx {
                    if let Ty::Struct(s) = base.ty {
                        let h = Home::Field(s, *index);
                        if self.elidable.contains(&h) {
                            out.push(h);
                        }
                    }
                }
                self.find_escapes_expr(home, base, false, out);
            }
            ExprKind::CallExpr(callee, args) => {
                let direct = self.target(home, callee).is_some() && skippable(callee);
                self.find_escapes_expr(home, callee, direct, out);
                for arg in args {
                    match arg {
                        Arg::Value(v) => self.find_escapes_expr(home, v, false, out),
                        Arg::Mut(p) => walk::place_exprs(p, &mut |x| self.find_escapes_expr(home, x, false, out)),
                    }
                }
            }
            ExprKind::Call(callee, args) => {
                for (slot, arg) in args.iter().enumerate() {
                    match arg {
                        Arg::Value(v) => {
                            let ok = self.elidable.contains(&Home::Param(*callee, slot))
                                && self.skippable_value(v);
                            self.find_escapes_expr(home, v, ok, out);
                        }
                        Arg::Mut(p) => walk::place_exprs(p, &mut |x| self.find_escapes_expr(home, x, false, out)),
                    }
                }
            }
            ExprKind::Tuple(items) => {
                let s = match e.ty {
                    Ty::Struct(s) => Some(s),
                    _ => None,
                };
                for (i, item) in items.iter().enumerate() {
                    let ok = s.is_some_and(|s| {
                        self.elidable.contains(&Home::Field(s, i))
                            && self.skippable_value(item)
                    });
                    self.find_escapes_expr(home, item, ok, out);
                }
            }
            ExprKind::Do(block) => {
                self.find_escapes(home, block, out);
            }
            _ => {
                walk::children(e, &mut |child| self.find_escapes_expr(home, child, false, out));
            }
        }
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

    /// Whether evaluating a fn-valued argument or field initializer can be skipped
    /// when the destination field or parameter is elided.
    pub(crate) fn skippable_value(&self, e: &Expr) -> bool {
        pure_path(e)
            || matches!(e.kind, ExprKind::None)
            || matches!(&e.kind, ExprKind::Some(inner) if self.skippable_value(inner))
            || matches!(e.kind, ExprKind::Closure(cid) if self.closure_effectively_captureless(cid))
    }

    pub fn unread_field(&self, struct_id: usize, index: usize) -> bool {
        self.unread_fields.contains(&(struct_id, index))
    }

    /// `place = value` into an unread field may be left out: it runs
    /// nothing and cannot trap.
    pub(crate) fn skippable_field_store(&self, place: &cpl_ir::Place, op: AssignOp, value: &Expr) -> bool {
        let PlaceKind::Field(base, _) = &place.kind else { return false };
        op == AssignOp::Set && place_is_path(base) && self.skippable_value(value)
    }

    fn block_captures_unused(&self, block: &Block, cap_slots: &BTreeSet<usize>) -> bool {
        block
            .stmts
            .iter()
            .all(|s| self.stmt_captures_unused(s, cap_slots))
    }

    fn place_captures_unused(&self, place: &cpl_ir::Place, cap_slots: &BTreeSet<usize>) -> bool {
        match &place.kind {
            PlaceKind::Local(slot) => !cap_slots.contains(slot),
            PlaceKind::Global(_) => true,
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => {
                self.place_captures_unused(base, cap_slots)
            }
            PlaceKind::Index(base, idxs) => {
                self.place_captures_unused(base, cap_slots)
                    && idxs
                        .iter()
                        .all(|i| self.expr_captures_unused(i, cap_slots))
            }
        }
    }

    fn stmt_captures_unused(&self, stmt: &Stmt, cap_slots: &BTreeSet<usize>) -> bool {
        match stmt {
            Stmt::Let(_, init) => init
                .as_ref()
                .is_none_or(|e| self.expr_captures_unused(e, cap_slots)),
            Stmt::Assign(place, _, value) => {
                self.place_captures_unused(place, cap_slots)
                    && self.expr_captures_unused(value, cap_slots)
            }
            Stmt::Expr(e) | Stmt::Give(e) | Stmt::DefDomainFromVal { val: e, .. } => {
                self.expr_captures_unused(e, cap_slots)
            }
            Stmt::Return(value) => value
                .as_ref()
                .is_none_or(|e| self.expr_captures_unused(e, cap_slots)),
            Stmt::If(cond, then, otherwise) => {
                self.expr_captures_unused(cond, cap_slots)
                    && self.block_captures_unused(then, cap_slots)
                    && self.block_captures_unused(otherwise, cap_slots)
            }
            Stmt::While(_, cond, body) => {
                self.expr_captures_unused(cond, cap_slots)
                    && self.block_captures_unused(body, cap_slots)
            }
            Stmt::For(_, _, iter, body) => {
                let iter_ok = match iter {
                    cpl_ir::Iter::Range { lo, hi, step, .. } => {
                        self.expr_captures_unused(lo, cap_slots)
                            && self.expr_captures_unused(hi, cap_slots)
                            && step
                                .as_ref()
                                .is_none_or(|s| self.expr_captures_unused(s, cap_slots))
                    }
                    cpl_ir::Iter::RangeVal(e, _)
                    | cpl_ir::Iter::Seq(e, _)
                    | cpl_ir::Iter::Subsets(e)
                    | cpl_ir::Iter::Members(e) => self.expr_captures_unused(e, cap_slots),
                    cpl_ir::Iter::TreeRange { container, lo, hi, .. } => {
                        self.expr_captures_unused(container, cap_slots)
                            && lo.as_ref().is_none_or(|(e, _)| self.expr_captures_unused(e, cap_slots))
                            && hi.as_ref().is_none_or(|(e, _)| self.expr_captures_unused(e, cap_slots))
                    }
                };
                iter_ok && self.block_captures_unused(body, cap_slots)
            }
            Stmt::Assert { cond, message, .. } => {
                self.expr_captures_unused(cond, cap_slots)
                    && message
                        .as_ref()
                        .is_none_or(|m| self.expr_captures_unused(m, cap_slots))
            }
            Stmt::DefDomain { lo, len, .. } => {
                self.expr_captures_unused(lo, cap_slots)
                    && self.expr_captures_unused(len, cap_slots)
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Trap(..) => true,
        }
    }

    fn expr_captures_unused(&self, e: &Expr, cap_slots: &BTreeSet<usize>) -> bool {
        match &e.kind {
            ExprKind::Local(slot) => !cap_slots.contains(slot),
            ExprKind::CallExpr(callee, args) => {
                let callee_ok = self.target(None, callee).is_some_and(|_| skippable(callee))
                    || self.expr_captures_unused(callee, cap_slots);
                callee_ok
                    && args.iter().all(|a| match a {
                        Arg::Value(v) => self.expr_captures_unused(v, cap_slots),
                        Arg::Mut(p) => self.place_captures_unused(p, cap_slots),
                    })
            }
            ExprKind::Closure(inner_cid) => {
                let inner = &self.program.closures[*inner_cid];
                !inner
                    .captures
                    .iter()
                    .any(|c| !c.is_global && cap_slots.contains(&c.outer))
            }
            ExprKind::Do(b) => self.block_captures_unused(b, cap_slots),
            ExprKind::Call(_, args) => args.iter().all(|a| match a {
                Arg::Value(v) => self.expr_captures_unused(v, cap_slots),
                Arg::Mut(p) => self.place_captures_unused(p, cap_slots),
            }),
            ExprKind::Mutate(_, place, _) | ExprKind::Sort(place, _, _)
                if !self.place_captures_unused(place, cap_slots) =>
            {
                false
            }
            _ => {
                let mut ok = true;
                walk::children(e, &mut |child| {
                    if ok && !self.expr_captures_unused(child, cap_slots) {
                        ok = false;
                    }
                });
                ok
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

fn place_base_ty(place: &cpl_ir::Place) -> &Ty {
    match &place.kind {
        PlaceKind::Field(base, _) => &base.ty,
        _ => &place.ty,
    }
}

/// A variable or a field of one: reading it runs nothing.
fn field_key(place: &cpl_ir::Place) -> Option<(usize, usize)> {
    match (&place.kind, place_base_ty(place)) {
        (PlaceKind::Field(_, idx), Ty::Struct(s)) => Some((*s, *idx)),
        _ => None,
    }
}

fn place_is_path(place: &cpl_ir::Place) -> bool {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => true,
        PlaceKind::Field(base, _) => place_is_path(base),
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

fn is_hook_fn(program: &Program, id: FnId) -> bool {
    program
        .structs
        .iter()
        .any(|s| [s.eq_fn, s.lt_fn, s.show_fn].contains(&Some(id)))
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

/// Report how every fn-typed struct field, fn parameter, and local closure
/// binding in `program` lowers to C++17 (static vs dynamic `std::function`).
pub fn explain_closures(program: &Program) -> Vec<ClosureLowering> {
    let devirt = Devirt::collect(program);
    let mut out = Vec::new();

    for (sid, s) in program.structs.iter().enumerate() {
        for (idx, (fname, fty)) in s.fields.iter().enumerate() {
            if !is_fn_or_opt_fn(fty) {
                continue;
            }
            let site = format!("struct {}.{}", s.name, fname);
            let mode = if devirt.unread_fields.contains(&(sid, idx)) {
                ClosureMode::Unread
            } else if matches!(fty, Ty::Opt(_)) {
                ClosureMode::DynamicStdFunction { reason: "optional fn field that is read".to_string() }
            } else if devirt.field_elidable(sid, idx) {
                let Some(Held::One(cid)) = devirt.held.get(&Home::Field(sid, idx)).copied() else {
                    continue;
                };
                ClosureMode::StaticDirect { closure: cid }
            } else {
                let reason = match devirt.held.get(&Home::Field(sid, idx)) {
                    Some(Held::One(_)) => "field escapes or is reassigned".to_string(),
                    Some(Held::Many) => "multiple or capturing closures stored in field".to_string(),
                    None => continue,
                };
                ClosureMode::DynamicStdFunction { reason }
            };
            out.push(ClosureLowering { site, mode });
        }
    }

    for (fid, f) in program.fns.iter().enumerate() {
        for slot in 0..f.params.len() {
            if !matches!(f.locals[slot].ty, Ty::Fn(..)) {
                continue;
            }
            let site = format!("fn {} param {}", f.name, f.locals[slot].name);
            let mode = if devirt.param_elidable(fid, slot) {
                let Some(Held::One(cid)) = devirt.held.get(&Home::Param(fid, slot)).copied() else {
                    continue;
                };
                ClosureMode::StaticDirect { closure: cid }
            } else if devirt.template_param_index(fid, slot).is_some() {
                ClosureMode::StaticTemplate
            } else {
                let reason = if program.entry_fns.contains(&fid) || program.judge_fns.contains(&fid) {
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
                ClosureMode::DynamicStdFunction { reason }
            };
            out.push(ClosureLowering { site, mode });
        }

        explain_locals_in_block(&devirt, &format!("fn {}", f.name), &f.locals, &f.body, &mut out);
    }

    explain_locals_in_block(&devirt, "main", &program.main_locals, &program.main, &mut out);
    out
}

fn explain_locals_in_block(
    devirt: &Devirt<'_>,
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
                mode: ClosureMode::DynamicStdFunction {
                    reason: "fn slot declared before its value (a @memo fn's wrapper)".to_string(),
                },
            });
            return;
        };
        let mode = match init.kind {
            ExprKind::Closure(cid)
                if stores.get(slot).copied().unwrap_or(0) <= 1
                    && (devirt.local_auto(*slot, cid) || devirt.local_rec(*slot, cid)) =>
            {
                ClosureMode::StaticLocalAuto
            }
            ExprKind::Closure(cid) if !devirt.local_auto(*slot, cid) => {
                ClosureMode::DynamicStdFunction {
                    reason: "recursive local closure".to_string(),
                }
            }
            ExprKind::Closure(_) => ClosureMode::DynamicStdFunction {
                reason: "reassigned local closure".to_string(),
            },
            _ => ClosureMode::DynamicStdFunction {
                reason: "dynamic fn value".to_string(),
            },
        };
        out.push(ClosureLowering { site, mode });
    });
}
