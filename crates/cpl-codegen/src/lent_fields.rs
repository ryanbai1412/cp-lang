//! Struct fields that hold a lent value as a pointer (R05.4.4, R13.4):
//! `fn wrap(xs: List<Int>) -> lends H = H(xs)` stores `&xs`, so making the
//! view is \(O(1)\), not a copy of the list. A field is lent only when every
//! value of its struct dies before what it points to: every literal stores
//! a non-`mut` parameter (or a field path of one), the struct is never part
//! of another value, a global, written or captured, every fn returning it
//! points only into its own parameters, and every call that makes one
//! passes a variable, not a temporary, for those. Any other field owns its
//! value, as before.

use crate::uses::places::{
    each_built, each_closure, each_put, each_tuple_in, each_write, each_write_in, Put, Write,
};
use crate::uses::split::{each_sink, each_value, Sink};
use crate::walk;
use cpl_ir::{
    Arg, Block, Expr, ExprKind, FnId, Function, Place, PlaceKind, Program, ScopeId, Stmt, StructId,
    Ty,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(crate) struct LentFields {
    /// `(struct, field)` stored as `const T*`.
    pub fields: BTreeSet<(StructId, usize)>,
    /// Per fn, the parameters a value it returns may point into.
    pub sources: Vec<BTreeSet<usize>>,
    /// `(fn, param)` a lent field may point to: taken by `const&`, never by value.
    pub by_ref: BTreeSet<(FnId, usize)>,
    /// Variables a lent field may point to: never moved from.
    pub held: BTreeSet<(ScopeId, usize)>,
    /// Globals a lent field may point to: never moved from.
    pub held_globals: BTreeSet<usize>,
}

impl crate::emit::Emitter<'_> {
    /// `&xs` for an item of a struct literal stored in a lent field.
    pub(crate) fn lent_item(
        &mut self,
        s: Option<StructId>,
        field: usize,
        item: &Expr,
    ) -> Option<String> {
        let lent = self.lent_fields.is_lent(s?, field);
        lent.then(|| format!("&{}", self.expr(item).at(crate::emit::PRIMARY)))
    }
}

impl LentFields {
    pub(crate) fn is_lent(&self, s: StructId, field: usize) -> bool {
        self.fields.contains(&(s, field))
    }
}

/// The heap types a lent field may point to.
fn pointable(program: &Program, ty: &Ty) -> bool {
    matches!(
        ty,
        Ty::List(_)
            | Ty::Array { .. }
            | Ty::Str
            | Ty::Map(..)
            | Ty::HashMap(..)
            | Ty::Set(_)
            | Ty::HashSet(_)
            | Ty::Struct(_)
    ) && !crate::names::cached(program, ty)
}

/// The parameter `e` names, directly or through fields.
fn param_root(e: &Expr) -> Option<usize> {
    match &e.kind {
        ExprKind::Local(slot) => Some(*slot),
        ExprKind::Field(base, _) => param_root(base),
        _ => None,
    }
}

/// The struct-literal items that point at a variable nothing changes while
/// the struct lives: a `let` of a statement block (not a `do`, whose result
/// outlives it) that no later statement of the block writes and no closure
/// captures, or a global that no later statement of `main` writes and no
/// fn or closure writes or captures.
fn frozen_items(program: &Program) -> BTreeSet<*const Expr> {
    let mut out = BTreeSet::new();
    let mut shared = BTreeSet::new();
    for (scope, body) in bodies(program) {
        each_closure(body, &mut |id| {
            let caps = &program.closures[id].captures;
            shared.extend(caps.iter().filter(|c| c.is_global).map(|c| c.outer));
        });
        if !matches!(scope, ScopeId::Main) {
            for stmt in &body.stmts {
                let writes = written_roots(stmt);
                shared.extend(writes.into_iter().filter(|w| w.0).map(|w| w.1));
            }
        }
    }
    let writes: Vec<_> = program.main.stmts.iter().map(written_roots).collect();
    for (j, stmt) in program.main.stmts.iter().enumerate() {
        each_tuple_in(stmt, &mut |items| {
            for item in items {
                let Some((true, g)) = var_root(item) else {
                    continue;
                };
                if !shared.contains(&g) && writes[j..].iter().all(|w| !w.contains(&(true, g))) {
                    out.insert(item as *const Expr);
                }
            }
        });
    }
    for (_, body) in bodies(program) {
        let mut captured = BTreeSet::new();
        each_closure(body, &mut |id| {
            let caps = &program.closures[id].captures;
            captured.extend(caps.iter().filter(|c| !c.is_global).map(|c| c.outer));
        });
        plain_blocks(body, &mut |block| frozen_in(block, &captured, &mut out));
    }
    out
}

fn plain_blocks<'p>(block: &'p Block, f: &mut dyn FnMut(&'p Block)) {
    f(block);
    for stmt in &block.stmts {
        walk::stmt_blocks(stmt, &mut |inner| plain_blocks(inner, f));
    }
}

fn frozen_in(block: &Block, captured: &BTreeSet<usize>, out: &mut BTreeSet<*const Expr>) {
    let writes: Vec<_> = block.stmts.iter().map(written_roots).collect();
    for (d, stmt) in block.stmts.iter().enumerate() {
        let Stmt::Let(v, _) = stmt else {
            continue;
        };
        if captured.contains(v) {
            continue;
        }
        let last = writes.iter().rposition(|w| w.contains(&(false, *v)));
        let from = last.map_or(d + 1, |l| (l + 1).max(d + 1));
        for stmt in block.stmts.iter().skip(from) {
            each_tuple_in(stmt, &mut |items| {
                let mine = items.iter().filter(|i| param_root(i) == Some(*v));
                out.extend(mine.map(|i| i as *const Expr));
            });
        }
    }
}

/// The variables `stmt` assigns, mutates or passes `mut`, at any depth,
/// as `(is_global, index)`.
fn written_roots(stmt: &Stmt) -> BTreeSet<(bool, usize)> {
    let mut out = BTreeSet::new();
    each_write_in(stmt, &mut |how, place| {
        if !matches!(how, Write::InPlace) {
            out.insert(walk::root(place));
        }
    });
    out
}

/// The variable `e` names, directly or through fields: `(is_global, index)`.
fn var_root(e: &Expr) -> Option<(bool, usize)> {
    match &e.kind {
        ExprKind::Local(slot) => Some((false, *slot)),
        ExprKind::Global(g) => Some((true, *g)),
        ExprKind::Field(base, _) => var_root(base),
        _ => None,
    }
}

fn bodies(program: &Program) -> Vec<(ScopeId, &Block)> {
    let mut out: Vec<(ScopeId, &Block)> = vec![
        (ScopeId::Main, &program.setup),
        (ScopeId::Main, &program.main),
    ];
    out.extend(
        program
            .fns
            .iter()
            .enumerate()
            .map(|(id, f)| (ScopeId::Fn(id), &f.body)),
    );
    out.extend(
        program
            .closures
            .iter()
            .enumerate()
            .map(|(id, c)| (ScopeId::Closure(id), &c.body)),
    );
    out
}

/// Marks every struct `ty` holds inside another type (`top` is false
/// below the outermost type).
fn nested(ty: &Ty, top: bool, out: &mut BTreeSet<StructId>) {
    match ty {
        Ty::Struct(s) if !top => {
            out.insert(*s);
        }
        Ty::Fn(params, ret) => {
            params.iter().for_each(|p| nested(p, true, out));
            nested(ret, false, out);
        }
        Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Mut(t) => {
            nested(t, false, out)
        }
        Ty::ListView(t, _) | Ty::Keyed(t, _) | Ty::MutView(t, _) => nested(t, false, out),
        Ty::Array { elem, .. } => nested(elem, false, out),
        Ty::Tuple(ts) => ts.iter().for_each(|t| nested(t, false, out)),
        Ty::Record(fs) => fs.iter().for_each(|(_, t)| nested(t, false, out)),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            nested(k, false, out);
            nested(v, false, out);
        }
        _ => {}
    }
}

struct Scan<'p> {
    program: &'p Program,
    fields: BTreeSet<(StructId, usize)>,
    sources: Vec<BTreeSet<usize>>,
    frozen: BTreeSet<*const Expr>,
}

impl Scan<'_> {
    fn has_lent(&self, ty: &Ty) -> Option<StructId> {
        match ty {
            Ty::Struct(s)
                if self
                    .fields
                    .range((*s, 0)..=(*s, usize::MAX))
                    .next()
                    .is_some() =>
            {
                Some(*s)
            }
            _ => None,
        }
    }

    fn drop_struct(&mut self, s: StructId) {
        self.fields.retain(|(t, _)| *t != s);
    }

    /// Drops what a write to `place` may change.
    fn written(&mut self, place: &Place) {
        if let Ty::Struct(s) = place.ty {
            self.drop_struct(s);
        }
        match &place.kind {
            PlaceKind::Field(base, field) => {
                if let Ty::Struct(s) = base.ty {
                    self.fields.remove(&(s, *field));
                }
                self.written(base);
            }
            PlaceKind::Index(base, _) | PlaceKind::Some(base) => self.written(base),
            _ => {}
        }
    }

    /// The parameters of `f` a lent-struct value `e` may point into; `None`
    /// when it may point elsewhere.
    fn points_into(&self, f: &Function, e: &Expr, depth: usize) -> Option<BTreeSet<usize>> {
        let s = self.has_lent(&e.ty)?;
        let params = f.params.len();
        match &e.kind {
            ExprKind::Tuple(items) => Some(
                self.fields
                    .range((s, 0)..=(s, usize::MAX))
                    .filter_map(|(_, i)| param_root(&items[*i]))
                    .collect(),
            ),
            ExprKind::Call(g, args) => {
                let mut out = BTreeSet::new();
                for &k in &self.sources[*g] {
                    match args.get(k) {
                        Some(Arg::Value(v)) => match walk::path_root(v) {
                            Some((false, slot)) if slot < params => {
                                out.insert(slot);
                            }
                            Some((true, _)) => {}
                            _ => return None,
                        },
                        _ => return None,
                    }
                }
                Some(out)
            }
            ExprKind::Local(slot) if *slot < params => Some([*slot].into()),
            ExprKind::Local(slot) if depth < 8 => {
                let mut out = BTreeSet::new();
                let mut bad = false;
                each_put(&f.body, &mut |put, init| {
                    if let Put::Let(l) = put {
                        if l == *slot {
                            match self.points_into(f, init, depth + 1) {
                                Some(set) => out.extend(set),
                                None => bad = true,
                            }
                        }
                    }
                });
                (!bad).then_some(out)
            }
            ExprKind::If(_, a, b) => {
                let mut out = self.points_into(f, a, depth)?;
                out.extend(self.points_into(f, b, depth)?);
                Some(out)
            }
            _ => None,
        }
    }

    /// Whether the lent-struct value `e` points only into globals, so a
    /// global may hold it.
    /// Whether lent-struct value `e` points only at globals; `lets` are the
    /// initial values of `main`'s locals, when `e` is in `main`.
    fn global_only(&self, e: &Expr, lets: Option<&BTreeMap<usize, &Expr>>) -> bool {
        self.has_lent(&e.ty).is_some()
            && match &e.kind {
                ExprKind::Global(_) => true,
                ExprKind::Local(t) => lets
                    .and_then(|lets| lets.get(t))
                    .is_some_and(|init| self.global_only(init, None)),
                ExprKind::Tuple(items) => {
                    let Ty::Struct(s) = e.ty else {
                        return false;
                    };
                    self.fields.range((s, 0)..=(s, usize::MAX)).all(|(_, i)| {
                        self.frozen.contains(&(&items[*i] as *const Expr))
                            && matches!(var_root(&items[*i]), Some((true, _)))
                    })
                }
                ExprKind::Call(g, args) => self.sources[*g].iter().all(|&k| {
                    matches!(args.get(k), Some(Arg::Value(v)) if matches!(walk::path_root(v), Some((true, _))))
                }),
                ExprKind::If(_, a, b) => self.global_only(a, lets) && self.global_only(b, lets),
                ExprKind::Do(block) => match block.stmts.split_last() {
                    Some((Stmt::Give(given), rest)) => {
                        !walk::has_give(rest) && self.global_only(given, lets)
                    }
                    _ => false,
                },
                _ => false,
            }
    }

    /// One pass of every rule but the returns; true when it dropped a field.
    fn check(&mut self) -> bool {
        let program = self.program;
        let before = self.fields.len();
        let mut inside = BTreeSet::new();
        for (sid, def) in program.structs.iter().enumerate() {
            for (i, (_, ty)) in def.fields.iter().enumerate() {
                if !(self.fields.contains(&(sid, i)) && matches!(ty, Ty::Struct(_))) {
                    nested(ty, false, &mut inside);
                }
            }
        }
        for def in &program.enums {
            for (_, fields) in &def.variants {
                fields.iter().for_each(|t| nested(t, false, &mut inside));
            }
        }
        program
            .globals
            .iter()
            .for_each(|g| nested(&g.ty, true, &mut inside));
        for f in &program.fns {
            f.locals
                .iter()
                .for_each(|l| nested(&l.ty, true, &mut inside));
            nested(&f.ret, true, &mut inside);
        }
        program
            .main_locals
            .iter()
            .for_each(|l| nested(&l.ty, true, &mut inside));
        for c in &program.closures {
            c.locals
                .iter()
                .for_each(|l| nested(&l.ty, true, &mut inside));
            nested(&c.ret, false, &mut inside);
            for cap in &c.captures {
                if let Some(Ty::Struct(s)) = c.locals.get(cap.inner).map(|l| &l.ty) {
                    inside.insert(*s);
                }
            }
        }
        let mut writes = Vec::new();
        let mut made = Vec::new();
        let mut main_lets = BTreeMap::new();
        for body in [&program.setup, &program.main] {
            each_sink(body, &mut |sink| {
                if let Sink::Let(t, Some(init)) = sink {
                    main_lets.insert(t, init);
                }
            });
        }
        for (scope, body) in bodies(program) {
            let lets = matches!(scope, ScopeId::Main).then_some(&main_lets);
            each_sink(body, &mut |sink| {
                if let Sink::Store(place, value) = sink {
                    let global = matches!(place.kind, PlaceKind::Global(_));
                    if !(global && self.global_only(value, lets)) {
                        writes.push(place);
                    }
                }
            });
            each_write(body, &mut |how, place| {
                if let Write::Mutate(_) | Write::Sort | Write::Mut = how {
                    writes.push(place);
                }
            });
            each_value(body, &mut |e| {
                nested(&e.ty, true, &mut inside);
                if let Ty::Struct(s) = e.ty {
                    made.push((scope, s, e));
                }
            });
        }
        inside.into_iter().for_each(|s| self.drop_struct(s));
        writes.into_iter().for_each(|p| self.written(p));
        for (scope, s, e) in made {
            match &e.kind {
                ExprKind::Tuple(items) => {
                    let f = match scope {
                        ScopeId::Fn(id) => Some(&program.fns[id]),
                        _ => None,
                    };
                    let lent: Vec<usize> = self
                        .fields
                        .range((s, 0)..=(s, usize::MAX))
                        .map(|(_, i)| *i)
                        .collect();
                    for i in lent {
                        let frozen = self.frozen.contains(&(&items[i] as *const Expr));
                        let ok = frozen
                            || f.is_some_and(|f| {
                                param_root(&items[i])
                                    .is_some_and(|p| p < f.params.len() && !f.params[p].mutable)
                            });
                        if !ok {
                            self.fields.remove(&(s, i));
                        }
                    }
                }
                ExprKind::Call(g, args) => {
                    let temporary = self.sources[*g].iter().any(|&k| {
                        !matches!(args.get(k), Some(Arg::Value(v)) if walk::path_root(v).is_some())
                    });
                    if temporary {
                        self.drop_struct(s);
                    }
                }
                ExprKind::Local(_)
                | ExprKind::Global(_)
                | ExprKind::Field(..)
                | ExprKind::Index(..)
                | ExprKind::Unwrap(..)
                | ExprKind::Payload(..)
                | ExprKind::CallExpr(..)
                | ExprKind::If(..)
                | ExprKind::Do(_) => {}
                _ => self.drop_struct(s),
            }
        }
        self.fields.len() != before
    }

    /// Grows `sources` to a fixed point; true when it dropped a field.
    fn returns(&mut self) -> bool {
        let program = self.program;
        self.sources = vec![BTreeSet::new(); program.fns.len()];
        loop {
            let mut changed = false;
            let mut dropped = Vec::new();
            for (id, f) in program.fns.iter().enumerate() {
                let Some(s) = self.has_lent(&f.ret) else {
                    continue;
                };
                let mut found = BTreeSet::new();
                let mut bad = false;
                each_sink(&f.body, &mut |sink| {
                    if let Sink::Return(v) = sink {
                        match self.points_into(f, v, 0) {
                            Some(set) => found.extend(set),
                            None => bad = true,
                        }
                    }
                });
                if bad {
                    dropped.push(s);
                } else if !found.is_subset(&self.sources[id]) {
                    self.sources[id].extend(found);
                    changed = true;
                }
            }
            if !dropped.is_empty() {
                dropped.into_iter().for_each(|s| self.drop_struct(s));
                return true;
            }
            if !changed {
                return false;
            }
        }
    }
}

/// The lent fields of `program`; `owned(s, i)` names fields kept in a layout
/// of their own (a row, narrowed cells), which stay owned.
pub(crate) fn lent_fields(
    program: &Program,
    owned: &dyn Fn(StructId, usize) -> bool,
) -> LentFields {
    let mut built = BTreeSet::new();
    for (_, body) in bodies(program) {
        each_built(body, &mut |s, _| {
            built.insert(s);
        });
    }
    let fields = program
        .structs
        .iter()
        .enumerate()
        .filter(|(s, def)| built.contains(s) && def.cached_fields.is_empty())
        .flat_map(|(s, def)| {
            def.fields
                .iter()
                .enumerate()
                .filter(move |(i, (_, ty))| {
                    pointable(program, ty) && !def.fixed.contains_key(i) && !owned(s, *i)
                })
                .map(move |(i, _)| (s, i))
        })
        .collect();
    let mut scan = Scan {
        program,
        frozen: frozen_items(program),
        fields,
        sources: vec![BTreeSet::new(); program.fns.len()],
    };
    while !scan.fields.is_empty() && (scan.returns() | scan.check()) {}
    let frozen = std::mem::take(&mut scan.frozen);
    let mut out = LentFields {
        fields: scan.fields,
        sources: scan.sources,
        ..LentFields::default()
    };
    if out.fields.is_empty() {
        out.sources.iter_mut().for_each(BTreeSet::clear);
        return out;
    }
    for (scope, body) in bodies(program) {
        each_value(body, &mut |e| {
            let Ty::Struct(s) = e.ty else {
                return;
            };
            match &e.kind {
                ExprKind::Tuple(items) => {
                    for (_, i) in out.fields.range((s, 0)..=(s, usize::MAX)) {
                        if frozen.contains(&(&items[*i] as *const Expr)) {
                            match var_root(&items[*i]) {
                                Some((false, v)) => {
                                    out.held.insert((scope, v));
                                }
                                Some((true, g)) => {
                                    out.held_globals.insert(g);
                                }
                                None => {}
                            }
                        } else if let (ScopeId::Fn(id), Some(p)) = (scope, param_root(&items[*i])) {
                            out.by_ref.insert((id, p));
                        }
                    }
                }
                ExprKind::Call(g, args) => {
                    for &k in &out.sources[*g] {
                        if let Some(Arg::Value(v)) = args.get(k) {
                            match walk::path_root(v) {
                                Some((false, slot)) => {
                                    out.held.insert((scope, slot));
                                }
                                Some((true, g)) => {
                                    out.held_globals.insert(g);
                                }
                                None => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }
    for (id, src) in out.sources.iter().enumerate() {
        out.by_ref.extend(src.iter().map(|&p| (id, p)));
    }
    out
}
