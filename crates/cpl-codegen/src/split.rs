//! performance/R13. A struct whose fn fields hold known closures is split
//! per closure: a `Monoid<Int>` built with `+` in one place and with `max`
//! in another becomes two structs, each with one closure in `op`, and each
//! fn that takes or returns one is cloned per struct it sees. Then every
//! fn field holds one closure and is called directly, not through a
//! `std::function`. It majorizes: only types are renamed, and the same
//! expressions run in the same order. A struct whose values ever meet in
//! one variable, or live in a list, optional or closure, stays whole.

use crate::devirt::ty_structs;
use crate::walk;
use cpl_ir::{
    Arg, Block, ClosureId, ElemLoc, Expr, ExprKind, FnId, GlobalId, Place, PlaceKind, Program,
    ScopeId, Slot, Stmt, StructId, Ty,
};
use std::collections::{BTreeMap, BTreeSet};

/// What a value of a split struct, or a fn value, is known to hold.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Key {
    Fn(ClosureId),
    /// The struct, and per field what it holds (`None`: not a fn or split struct).
    S(StructId, Vec<Option<Key>>),
    Top,
}

impl Key {
    fn known(&self) -> bool {
        match self {
            Key::Fn(_) => true,
            Key::S(_, fields) => fields.iter().flatten().all(Key::known),
            Key::Top => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Ctx {
    Main,
    Closure(ClosureId),
    /// A fn, with the key of each parameter that has one.
    Fn(FnId, Vec<Option<Key>>),
}

const MAX_CTXS: usize = 4000;

pub(crate) fn split(program: &mut Program) {
    let mut tracked = candidates(program);
    while !tracked.is_empty() {
        let Some(flow) = Flow::run(program, &tracked) else {
            return;
        };
        let failed = flow.failures();
        if failed.is_empty() {
            let plan = flow.plan();
            apply(program, &tracked, plan);
            return;
        }
        let part = components(program, &tracked);
        let dead: BTreeSet<usize> = failed.iter().filter_map(|s| part.get(s).copied()).collect();
        tracked.retain(|s| !dead.contains(&part[s]));
        tracked = settle(program, tracked);
    }
}

fn is_fn(ty: &Ty) -> bool {
    matches!(ty, Ty::Fn(..))
}

/// Structs with a fn field, and the structs that hold them, whose values
/// only ever live in variables, parameters, results and each other's fields.
fn candidates(program: &Program) -> BTreeSet<StructId> {
    let structs = &program.structs;
    let mut out: BTreeSet<StructId> = (0..structs.len())
        .filter(|&s| structs[s].fields.iter().any(|(_, t)| is_fn(t)))
        .collect();
    loop {
        let before = out.len();
        for (s, def) in structs.iter().enumerate() {
            if def
                .fields
                .iter()
                .any(|(_, t)| matches!(t, Ty::Struct(i) if out.contains(i)))
            {
                out.insert(s);
            }
        }
        if out.len() == before {
            break;
        }
    }
    let mut bad = BTreeSet::new();
    let nested = |ty: &Ty, bad: &mut BTreeSet<StructId>| {
        if !matches!(ty, Ty::Struct(_)) {
            ty_structs(ty, bad);
        }
    };
    for (s, def) in structs.iter().enumerate() {
        if def.show_fn.is_some()
            || def.eq_fn.is_some()
            || def.ord.is_some()
            || !def.cached_fields.is_empty()
        {
            bad.insert(s);
        }
        for (_, ty) in &def.fields {
            nested(ty, &mut bad);
        }
    }
    for def in &program.enums {
        for (_, tys) in &def.variants {
            tys.iter().for_each(|t| ty_structs(t, &mut bad));
        }
    }
    for g in &program.globals {
        nested(&g.ty, &mut bad);
    }
    for (id, f) in program.fns.iter().enumerate() {
        nested(&f.ret, &mut bad);
        f.locals.iter().for_each(|l| nested(&l.ty, &mut bad));
        if program.entry_fns.contains(&id) || program.judge_fns.contains(&id) {
            ty_structs(&f.ret, &mut bad);
            for l in &f.locals[..f.params.len()] {
                ty_structs(&l.ty, &mut bad);
            }
        }
    }
    for c in &program.closures {
        ty_structs(&c.ret, &mut bad);
        c.locals.iter().for_each(|l| nested(&l.ty, &mut bad));
        let opaque = c
            .params
            .iter()
            .chain(c.captures.iter().map(|cap| &cap.inner));
        for &slot in opaque {
            ty_structs(&c.locals[slot].ty, &mut bad);
        }
    }
    program
        .main_locals
        .iter()
        .for_each(|l| nested(&l.ty, &mut bad));
    for (_, block) in bodies(program) {
        walk::deep_exprs(block, &mut |e| {
            nested(&e.ty, &mut bad);
            places_in(e, &mut |p| nested(&p.ty, &mut bad));
        });
        walk::deep_stmts(block, &mut |s| match s {
            Stmt::Assign(p, ..) => nested(&p.ty, &mut bad),
            Stmt::For(..) => {}
            _ => {}
        });
    }
    out.retain(|s| !bad.contains(s));
    settle(program, out)
}

/// Drops a struct whose values some struct left whole holds, and one with
/// nothing left to split.
fn settle(program: &Program, mut set: BTreeSet<StructId>) -> BTreeSet<StructId> {
    loop {
        let before = set.len();
        for (s, def) in program.structs.iter().enumerate() {
            for (_, ty) in &def.fields {
                if let Ty::Struct(i) = ty {
                    if !set.contains(&s) && set.contains(i) {
                        set.remove(i);
                    }
                }
            }
        }
        let keep: Vec<StructId> = set
            .iter()
            .copied()
            .filter(|&s| {
                program.structs[s]
                    .fields
                    .iter()
                    .any(|(_, t)| is_fn(t) || matches!(t, Ty::Struct(i) if set.contains(i)))
            })
            .collect();
        set = keep.into_iter().collect();
        if set.len() == before {
            return set;
        }
    }
}

/// Each struct's group: structs that hold one another split together.
fn components(program: &Program, set: &BTreeSet<StructId>) -> BTreeMap<StructId, usize> {
    let mut part: BTreeMap<StructId, usize> = set.iter().map(|&s| (s, s)).collect();
    loop {
        let mut changed = false;
        for &s in set {
            for (_, ty) in &program.structs[s].fields {
                if let Ty::Struct(i) = ty {
                    if let (Some(&a), Some(&b)) = (part.get(&s), part.get(i)) {
                        if a != b {
                            let low = a.min(b);
                            for v in part.values_mut() {
                                if *v == a || *v == b {
                                    *v = low;
                                }
                            }
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            return part;
        }
    }
}

fn bodies(program: &Program) -> Vec<(Ctx, &Block)> {
    let mut out = vec![(Ctx::Main, &program.setup), (Ctx::Main, &program.main)];
    for (id, f) in program.fns.iter().enumerate() {
        out.push((Ctx::Fn(id, Vec::new()), &f.body));
    }
    for (id, c) in program.closures.iter().enumerate() {
        out.push((Ctx::Closure(id), &c.body));
    }
    out
}

fn places_in<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Place)) {
    match &e.kind {
        ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
            for arg in args {
                if let Arg::Mut(p) = arg {
                    f(p);
                }
            }
        }
        ExprKind::MutSlice(p, ..) | ExprKind::Mutate(_, p, _) | ExprKind::Sort(p, ..) => f(p),
        _ => {}
    }
}

struct Flow<'p> {
    program: &'p Program,
    tracked: &'p BTreeSet<StructId>,
    /// Per fn, per parameter: whether its key picks the fn's clone.
    keyed: Vec<Vec<bool>>,
    ctxs: BTreeSet<Ctx>,
    locals: BTreeMap<Ctx, BTreeMap<Slot, Key>>,
    rets: BTreeMap<Ctx, Key>,
    globals: BTreeMap<GlobalId, Key>,
    /// Once nothing changes: what is still unknown is anything.
    settled: bool,
    changed: bool,
}

impl<'p> Flow<'p> {
    fn run(program: &'p Program, tracked: &'p BTreeSet<StructId>) -> Option<Flow<'p>> {
        let is_struct = |t: &Ty| matches!(t, Ty::Struct(s) if tracked.contains(s));
        let mut keyed = Vec::new();
        let mut ctxs = BTreeSet::from([Ctx::Main]);
        for (id, f) in program.fns.iter().enumerate() {
            let involved = is_struct(&f.ret) || f.locals.iter().any(|l| is_struct(&l.ty));
            let mut builds = false;
            walk::deep_exprs(&f.body, &mut |e| builds |= is_struct(&e.ty));
            let row: Vec<bool> = (0..f.params.len())
                .map(|s| {
                    involved && (is_struct(&f.locals[s].ty) || (builds && is_fn(&f.locals[s].ty)))
                })
                .collect();
            if !row.contains(&true) {
                ctxs.insert(Ctx::Fn(id, vec![None; row.len()]));
            }
            keyed.push(row);
        }
        ctxs.extend((0..program.closures.len()).map(Ctx::Closure));
        let mut flow = Flow {
            program,
            tracked,
            keyed,
            ctxs,
            locals: BTreeMap::new(),
            rets: BTreeMap::new(),
            globals: BTreeMap::new(),
            settled: false,
            changed: false,
        };
        for settled in [false, true] {
            flow.settled = settled;
            loop {
                flow.changed = false;
                for ctx in flow.ctxs.clone() {
                    flow.visit(&ctx);
                }
                if flow.ctxs.len() > MAX_CTXS {
                    return None;
                }
                if !flow.changed {
                    break;
                }
            }
        }
        Some(flow)
    }

    fn is_struct(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Struct(s) if self.tracked.contains(s))
    }

    fn is_tracked(&self, ty: &Ty) -> bool {
        self.is_struct(ty) || is_fn(ty)
    }

    fn body(&self, ctx: &Ctx) -> Vec<&'p Block> {
        match ctx {
            Ctx::Main => vec![&self.program.setup, &self.program.main],
            Ctx::Closure(c) => vec![&self.program.closures[*c].body],
            Ctx::Fn(f, _) => vec![&self.program.fns[*f].body],
        }
    }

    fn local_ty(&self, ctx: &Ctx, slot: Slot) -> &'p Ty {
        match ctx {
            Ctx::Main => &self.program.main_locals[slot].ty,
            Ctx::Closure(c) => &self.program.closures[*c].locals[slot].ty,
            Ctx::Fn(f, _) => &self.program.fns[*f].locals[slot].ty,
        }
    }

    /// A slot whose value comes from outside what is tracked.
    fn opaque(&self, ctx: &Ctx, slot: Slot) -> bool {
        match ctx {
            Ctx::Main => false,
            Ctx::Closure(c) => {
                let def = &self.program.closures[*c];
                def.params.contains(&slot) || def.captures.iter().any(|cap| cap.inner == slot)
            }
            Ctx::Fn(_, keys) => keys.get(slot).is_some_and(Option::is_none),
        }
    }

    fn join(old: Option<&Key>, new: Key) -> Key {
        match old {
            None => new,
            Some(old) if *old == new => new,
            Some(_) => Key::Top,
        }
    }

    fn set_local(&mut self, ctx: &Ctx, slot: Slot, key: Key) {
        let map = self.locals.entry(ctx.clone()).or_default();
        let joined = Self::join(map.get(&slot), key);
        if map.get(&slot) != Some(&joined) {
            map.insert(slot, joined);
            self.changed = true;
        }
    }

    fn set_global(&mut self, g: GlobalId, key: Key) {
        let joined = Self::join(self.globals.get(&g), key);
        if self.globals.get(&g) != Some(&joined) {
            self.globals.insert(g, joined);
            self.changed = true;
        }
    }

    /// The keys a call passes the parameters that pick the callee's clone.
    fn call_keys(&self, ctx: &Ctx, callee: FnId, args: &[Arg]) -> Option<Vec<Option<Key>>> {
        let mut keys = Vec::new();
        for (slot, arg) in args.iter().enumerate() {
            if !self.keyed[callee].get(slot).copied().unwrap_or(false) {
                keys.push(None);
                continue;
            }
            let key = match arg {
                Arg::Value(e) => self.eval(ctx, e)?,
                Arg::Mut(p) => self.place_key(ctx, p)?,
            };
            keys.push(Some(key));
        }
        keys.resize(self.keyed[callee].len(), None);
        Some(keys)
    }

    fn unknown(&self) -> Option<Key> {
        self.settled.then_some(Key::Top)
    }

    fn eval(&self, ctx: &Ctx, e: &Expr) -> Option<Key> {
        if !self.is_tracked(&e.ty) {
            return Some(Key::Top);
        }
        match &e.kind {
            ExprKind::Closure(c) => Some(Key::Fn(*c)),
            ExprKind::Local(slot) => {
                if self.opaque(ctx, *slot) {
                    return Some(Key::Top);
                }
                match self.locals.get(ctx).and_then(|m| m.get(slot)) {
                    Some(k) => Some(k.clone()),
                    None => self.unknown(),
                }
            }
            ExprKind::Global(g) => match self.globals.get(g) {
                Some(k) => Some(k.clone()),
                None => self.unknown(),
            },
            ExprKind::Field(base, i) => Some(match self.eval(ctx, base)? {
                Key::S(_, fields) => fields.get(*i).cloned().flatten().unwrap_or(Key::Top),
                _ => Key::Top,
            }),
            ExprKind::Tuple(items) if self.is_struct(&e.ty) => {
                let Ty::Struct(s) = e.ty else { unreachable!() };
                let mut fields = Vec::new();
                for ((_, ty), item) in self.program.structs[s].fields.iter().zip(items) {
                    fields.push(match self.is_tracked(ty) {
                        true => Some(self.eval(ctx, item)?),
                        false => None,
                    });
                }
                Some(Key::S(s, fields))
            }
            ExprKind::Call(f, args) => {
                if !self.is_struct(&self.program.fns[*f].ret) {
                    return Some(Key::Top);
                }
                let keys = self.call_keys(ctx, *f, args)?;
                match self.rets.get(&Ctx::Fn(*f, keys)) {
                    Some(k) => Some(k.clone()),
                    None => self.unknown(),
                }
            }
            ExprKind::If(_, a, b) => {
                let (a, b) = (self.eval(ctx, a)?, self.eval(ctx, b)?);
                Some(if a == b { a } else { Key::Top })
            }
            ExprKind::Do(block) => {
                let mut gives = Vec::new();
                gives_in(block, &mut gives);
                let mut out: Option<Key> = None;
                for g in gives {
                    let k = self.eval(ctx, g)?;
                    out = Some(Self::join(out.as_ref(), k));
                }
                out.or_else(|| self.unknown())
            }
            _ => Some(Key::Top),
        }
    }

    fn place_key(&self, ctx: &Ctx, place: &Place) -> Option<Key> {
        let (root, path) = place_path(place);
        let mut key = match root {
            Some(Root::Local(s)) if self.opaque(ctx, s) => Key::Top,
            Some(Root::Local(s)) => match self.locals.get(ctx).and_then(|m| m.get(&s)) {
                Some(k) => k.clone(),
                None => return self.unknown(),
            },
            Some(Root::Global(g)) => match self.globals.get(&g) {
                Some(k) => k.clone(),
                None => return self.unknown(),
            },
            None => Key::Top,
        };
        for i in path {
            key = match key {
                Key::S(_, fields) => fields.get(i).cloned().flatten().unwrap_or(Key::Top),
                _ => Key::Top,
            };
        }
        Some(key)
    }

    fn visit(&mut self, ctx: &Ctx) {
        if let Ctx::Fn(_, keys) = ctx {
            for (slot, key) in keys.iter().enumerate() {
                if let Some(k) = key {
                    self.set_local(ctx, slot, k.clone());
                }
            }
        }
        for block in self.body(ctx) {
            let mut calls = Vec::new();
            walk::deep_exprs(block, &mut |e| {
                if let ExprKind::Call(f, args) = &e.kind {
                    calls.push((*f, args));
                }
            });
            for (f, args) in calls {
                if !self.keyed[f].contains(&true) {
                    continue;
                }
                if let Some(keys) = self.call_keys(ctx, f, args) {
                    if self.ctxs.insert(Ctx::Fn(f, keys)) {
                        self.changed = true;
                    }
                }
            }
            let mut stmts = Vec::new();
            walk::deep_stmts(block, &mut |s| stmts.push(s));
            for stmt in stmts {
                self.stmt(ctx, stmt);
            }
        }
    }

    fn stmt(&mut self, ctx: &Ctx, stmt: &Stmt) {
        match stmt {
            Stmt::Let(slot, init) if self.is_tracked(self.local_ty(ctx, *slot)) => {
                let key = match init {
                    Some(e) => self.eval(ctx, e),
                    None => Some(Key::Top),
                };
                if let Some(k) = key {
                    self.set_local(ctx, *slot, k);
                }
            }
            Stmt::For(_, slot, ..) if self.is_tracked(self.local_ty(ctx, *slot)) => {
                self.set_local(ctx, *slot, Key::Top);
            }
            Stmt::Return(Some(e)) => {
                if let Ctx::Fn(f, _) = ctx {
                    if self.is_struct(&self.program.fns[*f].ret) {
                        if let Some(k) = self.eval(ctx, e) {
                            let joined = Self::join(self.rets.get(ctx), k);
                            if self.rets.get(ctx) != Some(&joined) {
                                self.rets.insert(ctx.clone(), joined);
                                self.changed = true;
                            }
                        }
                    }
                }
            }
            Stmt::Assign(place, _, value) if self.is_tracked(&place.ty) => {
                let Some(new) = self.eval(ctx, value) else {
                    return;
                };
                let (root, path) = place_path(place);
                let old = match root {
                    Some(Root::Local(s)) => self.locals.get(ctx).and_then(|m| m.get(&s)).cloned(),
                    Some(Root::Global(g)) => self.globals.get(&g).cloned(),
                    None => return,
                };
                let whole = match (old, path.is_empty()) {
                    (_, true) => new,
                    (Some(old), false) => with_path(old, &path, new),
                    (None, false) if self.settled => Key::Top,
                    (None, false) => return,
                };
                match root {
                    Some(Root::Local(s)) => self.set_local(ctx, s, whole),
                    Some(Root::Global(g)) => self.set_global(g, whole),
                    None => {}
                }
            }
            _ => {}
        }
    }

    /// The ctxs the program runs, each call taking its final keys.
    fn live(&self) -> BTreeSet<Ctx> {
        let mut live: BTreeSet<Ctx> = self
            .ctxs
            .iter()
            .filter(|c| match c {
                Ctx::Fn(_, keys) => keys.iter().all(Option::is_none),
                _ => true,
            })
            .cloned()
            .collect();
        let mut work: Vec<Ctx> = live.iter().cloned().collect();
        while let Some(ctx) = work.pop() {
            for block in self.body(&ctx) {
                walk::deep_exprs(block, &mut |e| {
                    if let ExprKind::Call(f, args) = &e.kind {
                        if let Some(keys) = self.call_keys(&ctx, *f, args) {
                            let callee = Ctx::Fn(*f, keys);
                            if self.ctxs.contains(&callee) && live.insert(callee.clone()) {
                                work.push(callee);
                            }
                        }
                    }
                });
            }
        }
        live
    }

    /// Structs some value of which is not known to hold one closure per field.
    fn failures(&self) -> BTreeSet<StructId> {
        let mut out = BTreeSet::new();
        let check = |ty: &Ty, key: Option<&Key>, out: &mut BTreeSet<StructId>| {
            if let Ty::Struct(s) = ty {
                if self.tracked.contains(s) && !key.is_some_and(Key::known) {
                    out.insert(*s);
                }
            }
        };
        for (g, def) in self.program.globals.iter().enumerate() {
            check(&def.ty, self.globals.get(&g), &mut out);
        }
        for ctx in self.live() {
            let n = match &ctx {
                Ctx::Main => self.program.main_locals.len(),
                Ctx::Closure(c) => self.program.closures[*c].locals.len(),
                Ctx::Fn(f, _) => self.program.fns[*f].locals.len(),
            };
            for slot in 0..n {
                let key = self.locals.get(&ctx).and_then(|m| m.get(&slot));
                check(self.local_ty(&ctx, slot), key, &mut out);
            }
            if let Ctx::Fn(f, _) = &ctx {
                check(&self.program.fns[*f].ret, self.rets.get(&ctx), &mut out);
            }
            for block in self.body(&ctx) {
                walk::deep_exprs(block, &mut |e| {
                    if self.is_struct(&e.ty) {
                        let key = self.eval(&ctx, e);
                        check(&e.ty, key.as_ref(), &mut out);
                    }
                    if let ExprKind::Call(f, args) = &e.kind {
                        if self.keyed[*f].contains(&true)
                            && self.call_keys(&ctx, *f, args).is_none()
                        {
                            let def = &self.program.fns[*f];
                            for l in &def.locals[..def.params.len()] {
                                ty_structs(&l.ty, &mut out);
                            }
                            ty_structs(&def.ret, &mut out);
                        }
                    }
                });
            }
        }
        out.retain(|s| self.tracked.contains(s));
        out
    }

    fn plan(&self) -> Plan {
        let live = self.live();
        let mut plan = Plan::default();
        let mut next = self.program.fns.len();
        for ctx in &live {
            if let Ctx::Fn(f, keys) = ctx {
                if keys.iter().any(Option::is_some) {
                    plan.clones.insert(ctx.clone(), next);
                    plan.new_fns.push((*f, next));
                    next += 1;
                }
            }
        }
        for (g, key) in &self.globals {
            if self.is_struct(&self.program.globals[*g].ty) {
                plan.global_keys.insert(*g, key.clone());
            }
        }
        for ctx in &live {
            let mut body = CtxPlan::default();
            let n = match ctx {
                Ctx::Main => self.program.main_locals.len(),
                Ctx::Closure(c) => self.program.closures[*c].locals.len(),
                Ctx::Fn(f, _) => self.program.fns[*f].locals.len(),
            };
            for slot in 0..n {
                if self.is_struct(self.local_ty(ctx, slot)) {
                    if let Some(k) = self.locals.get(ctx).and_then(|m| m.get(&slot)) {
                        body.locals.insert(slot, k.clone());
                    }
                }
            }
            if let Ctx::Fn(f, _) = ctx {
                if self.is_struct(&self.program.fns[*f].ret) {
                    body.ret = self.rets.get(ctx).cloned();
                }
            }
            for block in self.body(ctx) {
                let mut steps = Vec::new();
                exprs_ref(block, &mut |e| {
                    let ty = self.is_struct(&e.ty).then(|| self.eval(ctx, e)).flatten();
                    let callee = match &e.kind {
                        ExprKind::Call(f, args) => self
                            .call_keys(ctx, *f, args)
                            .and_then(|keys| plan.clones.get(&Ctx::Fn(*f, keys)).copied()),
                        _ => None,
                    };
                    steps.push((ty, callee));
                });
                body.blocks.push(steps);
            }
            let target = match ctx {
                Ctx::Main => Target::Main,
                Ctx::Closure(c) => Target::Closure(*c),
                Ctx::Fn(f, _) => match plan.clones.get(ctx) {
                    Some(&new) => Target::Fn(new),
                    None => Target::Fn(*f),
                },
            };
            plan.bodies.push((target, body));
        }
        plan
    }
}

#[derive(Default)]
struct Plan {
    clones: BTreeMap<Ctx, FnId>,
    /// `(original, clone)`, in clone order.
    new_fns: Vec<(FnId, FnId)>,
    global_keys: BTreeMap<GlobalId, Key>,
    bodies: Vec<(Target, CtxPlan)>,
}

#[derive(Clone, Copy)]
enum Target {
    Main,
    Closure(ClosureId),
    Fn(FnId),
}

#[derive(Default)]
struct CtxPlan {
    locals: BTreeMap<Slot, Key>,
    ret: Option<Key>,
    /// Per block, per expression in [`exprs_ref`] order: its key and the
    /// clone its call goes to.
    blocks: Vec<Vec<(Option<Key>, Option<FnId>)>>,
}

enum Root {
    Local(Slot),
    Global(GlobalId),
}

/// The variable a place is in and the fields down to it; `None` when the
/// path goes through an index or an optional.
fn place_path(place: &Place) -> (Option<Root>, Vec<usize>) {
    match &place.kind {
        PlaceKind::Local(s) => (Some(Root::Local(*s)), Vec::new()),
        PlaceKind::Global(g) => (Some(Root::Global(*g)), Vec::new()),
        PlaceKind::Field(base, i) => {
            let (root, mut path) = place_path(base);
            path.push(*i);
            (root, path)
        }
        PlaceKind::Index(..) | PlaceKind::Some(_) => (None, Vec::new()),
    }
}

fn with_path(key: Key, path: &[usize], new: Key) -> Key {
    let Some((&first, rest)) = path.split_first() else {
        return Flow::join(Some(&key), new);
    };
    match key {
        Key::S(s, mut fields) => {
            let old = fields.get(first).cloned().flatten().unwrap_or(Key::Top);
            if let Some(slot) = fields.get_mut(first) {
                *slot = Some(with_path(old, rest, new));
            }
            Key::S(s, fields)
        }
        _ => Key::Top,
    }
}

/// The `Give`s of `block` itself, not of a `Do` nested inside it.
fn gives_in<'a>(block: &'a Block, out: &mut Vec<&'a Expr>) {
    for stmt in &block.stmts {
        if let Stmt::Give(e) = stmt {
            out.push(e);
        }
        walk::stmt_blocks(stmt, &mut |inner| gives_in(inner, out));
    }
}

fn exprs_ref<'a>(block: &'a Block, f: &mut dyn FnMut(&'a Expr)) {
    for stmt in &block.stmts {
        walk::stmt_exprs(stmt, &mut |e| expr_ref(e, f));
        walk::stmt_blocks(stmt, &mut |inner| exprs_ref(inner, f));
    }
}

fn expr_ref<'a>(e: &'a Expr, f: &mut dyn FnMut(&'a Expr)) {
    if let ExprKind::Do(block) = &e.kind {
        exprs_ref(block, f);
    }
    walk::children(e, &mut |child| expr_ref(child, f));
    f(e);
}

fn exprs_mut(block: &mut Block, f: &mut dyn FnMut(&mut Expr)) {
    for stmt in &mut block.stmts {
        walk::stmt_exprs_mut(stmt, &mut |e| expr_mut(e, f));
        walk::stmt_blocks_mut(stmt, &mut |inner| exprs_mut(inner, f));
    }
}

fn expr_mut(e: &mut Expr, f: &mut dyn FnMut(&mut Expr)) {
    if let ExprKind::Do(block) = &mut e.kind {
        exprs_mut(block, f);
    }
    walk::children_mut(e, &mut |child| expr_mut(child, f));
    f(e);
}

fn apply(program: &mut Program, tracked: &BTreeSet<StructId>, plan: Plan) {
    let mut ids: BTreeMap<Key, StructId> = BTreeMap::new();
    let mut count: BTreeMap<StructId, usize> = BTreeMap::new();
    let mut keys: Vec<&Key> = plan.global_keys.values().collect();
    for (_, body) in &plan.bodies {
        keys.extend(body.locals.values());
        keys.extend(body.ret.iter());
        keys.extend(body.blocks.iter().flatten().filter_map(|(k, _)| k.as_ref()));
    }
    for key in keys {
        struct_for(program, key, &mut ids, &mut count);
    }
    let ty_of = |key: &Key| Ty::Struct(ids[key]);
    for (g, key) in &plan.global_keys {
        program.globals[*g].ty = ty_of(key);
    }
    for &(old, _) in &plan.new_fns {
        program.fns.push(program.fns[old].clone());
        program.fn_display.push(program.fn_display[old].clone());
        program.library_fns.push(program.library_fns[old]);
        let copied: Vec<(ElemLoc, cpl_ir::NarrowLayout)> = program
            .narrow_elems
            .iter()
            .filter_map(|(loc, layout)| {
                let new = program.fns.len() - 1;
                match loc {
                    ElemLoc::FnRet(f) if *f == old => Some((ElemLoc::FnRet(new), layout.clone())),
                    ElemLoc::Local(ScopeId::Fn(f), s) if *f == old => {
                        Some((ElemLoc::Local(ScopeId::Fn(new), *s), layout.clone()))
                    }
                    _ => None,
                }
            })
            .collect();
        program.narrow_elems.extend(copied);
    }
    let global_tys: Vec<Ty> = program.globals.iter().map(|g| g.ty.clone()).collect();
    for (target, body) in plan.bodies {
        let structs = program.structs.clone();
        let (locals, blocks, ret): (&mut Vec<cpl_ir::Local>, Vec<&mut Block>, Option<&mut Ty>) =
            match target {
                Target::Main => (
                    &mut program.main_locals,
                    vec![&mut program.setup, &mut program.main],
                    None,
                ),
                Target::Closure(c) => {
                    let c = &mut program.closures[c];
                    (&mut c.locals, vec![&mut c.body], None)
                }
                Target::Fn(f) => {
                    let f = &mut program.fns[f];
                    (&mut f.locals, vec![&mut f.body], Some(&mut f.ret))
                }
            };
        for (slot, key) in &body.locals {
            if key.known() {
                locals[*slot].ty = ty_of(key);
            }
        }
        if let (Some(ret), Some(key)) = (ret, &body.ret) {
            *ret = ty_of(key);
        }
        let local_tys: Vec<Ty> = locals.iter().map(|l| l.ty.clone()).collect();
        let retype = |p: &mut Place| retype_place(p, tracked, &local_tys, &global_tys, &structs);
        for (block, steps) in blocks.into_iter().zip(body.blocks) {
            let mut steps = steps.into_iter();
            exprs_mut(block, &mut |e| {
                let Some((key, callee)) = steps.next() else {
                    return;
                };
                if let Some(key) = key {
                    e.ty = ty_of(&key);
                }
                if let (Some(new), ExprKind::Call(f, _)) = (callee, &mut e.kind) {
                    *f = new;
                }
                match &mut e.kind {
                    ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => {
                        for arg in args {
                            if let Arg::Mut(p) = arg {
                                retype(p);
                            }
                        }
                    }
                    ExprKind::MutSlice(p, ..)
                    | ExprKind::Mutate(_, p, _)
                    | ExprKind::Sort(p, ..) => retype(p),
                    _ => {}
                }
            });
            walk::blocks_mut(block, &mut |b| {
                for stmt in &mut b.stmts {
                    if let Stmt::Assign(p, ..) = stmt {
                        retype(p);
                    }
                }
            });
        }
    }
}

fn struct_for(
    program: &mut Program,
    key: &Key,
    ids: &mut BTreeMap<Key, StructId>,
    count: &mut BTreeMap<StructId, usize>,
) {
    let Key::S(s, fields) = key else {
        return;
    };
    if ids.contains_key(key) || !key.known() {
        return;
    }
    for field in fields.iter().flatten() {
        struct_for(program, field, ids, count);
    }
    let mut def = program.structs[*s].clone();
    let n = count.entry(*s).or_default();
    def.name = format!("{}_k{n}", def.name);
    *n += 1;
    for (field, (_, ty)) in fields.iter().zip(&mut def.fields) {
        if let Some(sub @ Key::S(..)) = field {
            *ty = Ty::Struct(ids[sub]);
        }
    }
    let new = program.structs.len();
    program.structs.push(def);
    if program.struct_shown.len() == new {
        program.struct_shown.push(program.struct_shown[*s].clone());
    }
    let copied: Vec<_> = program
        .narrow_elems
        .iter()
        .filter_map(|(loc, layout)| match loc {
            ElemLoc::Field(sid, i) if sid == s => Some((ElemLoc::Field(new, *i), layout.clone())),
            _ => None,
        })
        .collect();
    program.narrow_elems.extend(copied);
    ids.insert(key.clone(), new);
}

fn retype_place(
    place: &mut Place,
    tracked: &BTreeSet<StructId>,
    locals: &[Ty],
    globals: &[Ty],
    structs: &[cpl_ir::StructDef],
) {
    let old = matches!(place.ty, Ty::Struct(s) if tracked.contains(&s));
    match &mut place.kind {
        PlaceKind::Local(s) if old => place.ty = locals[*s].clone(),
        PlaceKind::Global(g) if old => place.ty = globals[*g].clone(),
        PlaceKind::Field(base, i) => {
            retype_place(base, tracked, locals, globals, structs);
            if let (true, Ty::Struct(sid)) = (old, &base.ty) {
                place.ty = structs[*sid].fields[*i].1.clone();
            }
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => {
            retype_place(base, tracked, locals, globals, structs)
        }
        _ => {}
    }
}
