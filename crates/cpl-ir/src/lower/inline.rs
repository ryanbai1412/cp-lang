//! The one IR inliner, run once after lowering so both engines see its
//! result. A call of a library fn that is small, has one call site, or gets
//! a closure runs as the fn's body at the call; a call of a closure value
//! whose closure is known runs as the closure's body. A library body put
//! into the user's code takes the call's span, so a trap in it shows at the
//! user's call as it did from its own frame. `cpl explain` lists each.

use super::forward;
use crate::walk::{children_mut, deep_exprs, deep_stmts, stmt_blocks_mut, stmt_exprs_mut, Effects};
use crate::{Arg, Block, ClosureDef, Expr, ExprKind, Function, Local, LoopId, Place, PlaceKind};
use crate::{Slot, SortBy, Span, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// A body of at most this many nodes is inlined at every call.
const SMALL: usize = 24;
/// A body that gets a closure is inlined up to this many nodes.
const WITH_CLOSURE: usize = 160;
const ROUNDS: usize = 8;

pub(super) struct Hosts<'a> {
    pub fns: &'a mut [Option<Function>],
    pub main: &'a mut Block,
    pub setup: &'a mut Block,
    pub main_locals: &'a mut Vec<Local>,
    pub closures: &'a mut Vec<ClosureDef>,
}

/// Inlines every call it can; gives the call span, callee and reason of
/// each inline in the user's code.
pub(super) fn run(
    h: Hosts<'_>,
    library: &[bool],
    shown: &[String],
    next_loop: &mut LoopId,
) -> Vec<(Span, String)> {
    forward::inline_forwarders(h.fns, [&mut *h.main, &mut *h.setup], h.closures);
    if std::env::var_os("CPL_NO_INLINE").is_some() {
        return Vec::new();
    }
    let recursive = recursion(h.fns, h.closures);
    let mut sites = vec![0usize; h.fns.len()];
    let mut count = |b: &Block| {
        deep_exprs(b, &mut |e| {
            if let ExprKind::Call(f, _) = e.kind {
                if let Some(n) = sites.get_mut(f) {
                    *n += 1;
                }
            }
        })
    };
    h.fns.iter().flatten().for_each(|f| count(&f.body));
    h.closures.iter().for_each(|c| count(&c.body));
    count(h.main);
    count(h.setup);
    let fn_count = h.fns.len();
    let mut inl = Inliner {
        fns: h.fns,
        closures: h.closures,
        library,
        shown,
        recursive,
        sites,
        next_loop,
        inlined: Vec::new(),
        within: vec![Vec::new(); fn_count],
    };
    for f in 0..inl.fns.len() {
        let Some(mut func) = inl.fns[f].take() else {
            continue;
        };
        inl.host(&mut func.locals, &mut func.body, library.get(f).copied().unwrap_or(false), Some(f));
        inl.fns[f] = Some(func);
    }
    inl.host(h.main_locals, h.main, false, None);
    inl.host(h.main_locals, h.setup, false, None);
    inl.inlined
}

struct Inliner<'a> {
    fns: &'a mut [Option<Function>],
    closures: &'a mut Vec<ClosureDef>,
    library: &'a [bool],
    shown: &'a [String],
    recursive: Vec<bool>,
    sites: Vec<usize>,
    next_loop: &'a mut LoopId,
    inlined: Vec<(Span, String)>,
    /// What was inlined into each library fn, listed with its own inline.
    within: Vec<Vec<String>>,
}

/// The fns that can reach themselves, through calls in their bodies and
/// in the closures they make.
fn recursion(fns: &[Option<Function>], closures: &[ClosureDef]) -> Vec<bool> {
    fn calls(b: &Block, closures: &[ClosureDef], out: &mut BTreeSet<usize>, seen: &mut BTreeSet<usize>) {
        deep_exprs(b, &mut |e| match e.kind {
            ExprKind::Call(f, _) => {
                out.insert(f);
            }
            ExprKind::Closure(c) if seen.insert(c) => {
                if let Some(def) = closures.get(c) {
                    calls(&def.body, closures, out, seen);
                }
            }
            _ => {}
        });
    }
    let edges: Vec<BTreeSet<usize>> = fns
        .iter()
        .map(|f| {
            let mut out = BTreeSet::new();
            if let Some(f) = f {
                calls(&f.body, closures, &mut out, &mut BTreeSet::new());
            }
            out
        })
        .collect();
    (0..fns.len())
        .map(|start| {
            let mut seen = BTreeSet::new();
            let mut todo: Vec<usize> = edges[start].iter().copied().collect();
            while let Some(f) = todo.pop() {
                if f == start {
                    return true;
                }
                if seen.insert(f) {
                    todo.extend(edges.get(f).into_iter().flatten().copied());
                }
            }
            false
        })
        .collect()
}

fn size(b: &Block) -> usize {
    let mut n = 0;
    deep_stmts(b, &mut |_| n += 1);
    deep_exprs(b, &mut |_| n += 1);
    n
}

impl Inliner<'_> {
    fn host(&mut self, locals: &mut Vec<Local>, body: &mut Block, library: bool, f: Option<usize>) {
        let limit = 4 * size(body) + 400;
        for _ in 0..ROUNDS {
            let mut assigned = Effects::of_block(body).locals;
            let mut lets: BTreeMap<Slot, (usize, Option<usize>)> = BTreeMap::new();
            deep_stmts(body, &mut |s| {
                if let Stmt::Let(slot, init) = s {
                    let c = match init.as_ref().map(|e| &e.kind) {
                        Some(ExprKind::Closure(c)) => Some(*c),
                        _ => None,
                    };
                    let entry = lets.entry(*slot).or_insert((0, c));
                    entry.0 += 1;
                }
            });
            deep_exprs(body, &mut |e| {
                if let ExprKind::Closure(c) = e.kind {
                    for cap in self.closures.get(c).into_iter().flat_map(|d| &d.captures) {
                        if cap.by_ref && !cap.is_global {
                            assigned.insert(cap.outer);
                        }
                    }
                }
            });
            let bound = lets
                .into_iter()
                .filter_map(|(s, (n, c))| (n == 1 && !assigned.contains(&s)).then_some((s, c?)))
                .collect();
            let mut cx = Cx { inl: self, locals, assigned, bound, changed: false, user: !library, f };
            cx.block(body);
            if !cx.changed {
                break;
            }
            drop_unused_closures(body, self.closures);
            if size(body) > limit {
                break;
            }
        }
    }
}

struct Cx<'i, 'a> {
    inl: &'i mut Inliner<'a>,
    locals: &'i mut Vec<Local>,
    assigned: BTreeSet<Slot>,
    bound: BTreeMap<Slot, usize>,
    changed: bool,
    user: bool,
    f: Option<usize>,
}

/// What a callee's slot becomes at the call.
#[derive(Clone)]
enum Sub {
    Slot(Slot),
    Expr(Expr),
}

impl Cx<'_, '_> {
    fn block(&mut self, b: &mut Block) {
        for stmt in &mut b.stmts {
            // A walk of a call is the call's own walk (a generator's steps
            // run between the loop's), so the call stays.
            if let Stmt::For(_, _, crate::Iter::Seq(walked @ Expr { kind: ExprKind::Call(..), .. }, _), body) = stmt {
                children_mut(walked, &mut |e| self.expr(e));
                self.block(body);
                continue;
            }
            stmt_exprs_mut(stmt, &mut |e| self.expr(e));
            stmt_blocks_mut(stmt, &mut |inner| self.block(inner));
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        if let ExprKind::Do(inner) = &mut e.kind {
            self.block(inner);
        }
        children_mut(e, &mut |child| self.expr(child));
        match &e.kind {
            ExprKind::Field(base, k) => {
                if let ExprKind::Tuple(items) = &base.kind {
                    if items.iter().all(crate::walk::is_simple) && *k < items.len() {
                        let picked = items[*k].clone();
                        *e = picked;
                        self.changed = true;
                    }
                }
            }
            ExprKind::Call(..) if std::env::var_os("CPL_INL_NO_CALL").is_none() => self.call(e),
            ExprKind::CallExpr(..) if std::env::var_os("CPL_INL_NO_BETA").is_none() => self.beta(e),
            _ => {}
        }
    }

    fn record(&mut self, span: Span, what: String) {
        match (self.user, self.f) {
            (true, _) => self.inl.inlined.push((span, what)),
            (false, Some(f)) => self.inl.within[f].push(what),
            (false, None) => {}
        }
    }

    fn closure_of(&self, e: &Expr) -> Option<usize> {
        match e.kind {
            ExprKind::Closure(c) => Some(c),
            ExprKind::Local(s) => self.bound.get(&s).copied(),
            _ => None,
        }
    }

    fn call(&mut self, e: &mut Expr) {
        let ExprKind::Call(f, args) = &e.kind else {
            return;
        };
        let f = *f;
        if !self.inl.library.get(f).copied().unwrap_or(false) || self.inl.recursive[f] {
            return;
        }
        let Some(func) = self.inl.fns.get(f).and_then(Option::as_ref) else {
            return;
        };
        let takes = |p: &crate::ParamDef| !p.mutable && p.range.is_none() && !p.fills_cache;
        if func.ret != e.ty || func.ret == Ty::Unit || args.len() != func.params.len() {
            return;
        }
        if !func.params.iter().all(takes) {
            return;
        }
        // A check in a library body names its fn's frame, which the user's
        // code doesn't have.
        let mut checks = false;
        deep_stmts(&func.body, &mut |s| checks |= matches!(s, Stmt::Assert { .. } | Stmt::Trap(..)));
        if self.user && checks {
            return;
        }
        // A value the body builds in its own frame is handed out whole;
        // spread into the user's code, the parts would be copied.
        let lone = matches!(
            func.body.stmts.as_slice(),
            [Stmt::Return(Some(Expr { kind: ExprKind::Local(_) | ExprKind::Builtin(..) | ExprKind::Intrinsic(..), .. }))]
        );
        if self.user && !func.ret.is_scalar() && !lone {
            return;
        }
        let closure_arg = args.iter().any(|a| matches!(a, Arg::Value(v) if self.closure_of(v).is_some()));
        let n = size(&func.body);
        let reason = if n <= SMALL {
            "small"
        } else if self.inl.sites[f] == 1 {
            "its one call"
        } else if closure_arg && n <= WITH_CLOSURE {
            "given a closure"
        } else {
            return;
        };
        let (body, callee_locals, nparams) = (func.body.clone(), func.locals.clone(), func.params.len());
        let respan = self.user;
        let map = vec![None; callee_locals.len()];
        let ExprKind::Call(_, args) = &e.kind else {
            return;
        };
        let params: Vec<Slot> = (0..nparams).collect();
        let Some(new) = self.splice(&body, &callee_locals, map, &params, args, e, respan) else {
            return;
        };
        let name = self.inl.shown.get(f).cloned().unwrap_or_default();
        let inner = std::mem::take(&mut self.inl.within[f]);
        let what = match inner.is_empty() {
            true => format!("{name}: {reason}"),
            false => format!("{name}: {reason}, with {}", inner.join(", ")),
        };
        self.inl.within[f] = inner;
        self.record(e.span, what);
        *e = new;
        self.changed = true;
    }

    fn beta(&mut self, e: &mut Expr) {
        let ExprKind::CallExpr(callee, args) = &e.kind else {
            return;
        };
        let Some(c) = self.closure_of(callee) else {
            return;
        };
        let Some(def) = self.inl.closures.get(c) else {
            return;
        };
        if !def.muts.is_empty() || def.ret != e.ty || def.ret == Ty::Unit || def.params.len() != args.len() {
            return;
        }
        if size(&def.body) > WITH_CLOSURE {
            return;
        }
        let mut map = vec![None; def.locals.len()];
        for cap in &def.captures {
            if cap.by_ref || (!cap.is_global && self.assigned.contains(&cap.outer)) {
                return;
            }
            let ty = def.locals.get(cap.inner).map_or(Ty::Error, |l| l.ty.clone());
            let kind = match cap.is_global {
                true => ExprKind::Global(cap.outer),
                false => ExprKind::Local(cap.outer),
            };
            let Some(slot) = map.get_mut(cap.inner) else {
                return;
            };
            *slot = Some(Sub::Expr(Expr { kind, ty, span: e.span }));
        }
        let (body, locals, params) = (def.body.clone(), def.locals.clone(), def.params.clone());
        let Some(new) = self.splice(&body, &locals, map, &params, args, e, false) else {
            return;
        };
        self.record(e.span, "a closure: known at its call".to_string());
        *e = new;
        self.changed = true;
    }

    /// `body` run at the call `at` with `args` for `params`: its locals
    /// appended to the host's, its returns giving the value.
    #[allow(clippy::too_many_arguments)]
    fn splice(
        &mut self,
        body: &Block,
        callee_locals: &[Local],
        mut map: Vec<Option<Sub>>,
        params: &[Slot],
        args: &[Arg],
        at: &Expr,
        respan: bool,
    ) -> Option<Expr> {
        match body.stmts.last() {
            Some(Stmt::Return(Some(_)) | Stmt::Trap(..)) => {}
            _ => return None,
        }
        if top_returns(body) != all_returns(body) {
            return None;
        }
        let base = self.locals.len();
        let mut lets = Vec::new();
        for (&p, a) in params.iter().zip(args) {
            let Arg::Value(v) = a else {
                return None;
            };
            if callee_locals.get(p).map(|l| &l.ty) != Some(&v.ty) {
                return None;
            }
            let reads = || {
                let mut n = 0;
                deep_exprs(body, &mut |e| n += usize::from(e.kind == ExprKind::Local(p)));
                n
            };
            if direct(v) || (matches!(v.kind, ExprKind::Closure(_)) && reads() <= 1) {
                map[p] = Some(Sub::Expr(v.clone()));
            } else if indexes(v) && !scalar(&v.ty) {
                return None;
            } else {
                lets.push(Stmt::Let(base + p, Some(v.clone())));
            }
        }
        let map: Vec<Sub> = map
            .into_iter()
            .enumerate()
            .map(|(s, m)| m.unwrap_or(Sub::Slot(base + s)))
            .collect();
        let mut re = Remap {
            map: &map,
            loops: HashMap::new(),
            next_loop: self.inl.next_loop,
            closures: self.inl.closures,
            span: respan.then_some(at.span),
            ok: true,
        };
        let mut body = body.clone();
        re.block(&mut body);
        if !re.ok {
            return None;
        }
        self.locals.extend(callee_locals.iter().cloned().map(|mut l| {
            if let Some(p) = &mut l.binds {
                re.place(p);
            }
            l
        }));
        if lets.is_empty() {
            if let [Stmt::Give(only)] = body.stmts.as_slice() {
                return Some(only.clone());
            }
        }
        lets.append(&mut body.stmts);
        Some(Expr { kind: ExprKind::Do(Block { stmts: lets }), ty: at.ty.clone(), span: at.span })
    }
}

/// The returns a body makes outside its loops, which can give the value of
/// the block it is inlined as.
fn top_returns(b: &Block) -> usize {
    let one = |s: &Stmt| match s {
        Stmt::Return(_) => 1,
        Stmt::If(_, then, other) => top_returns(then) + top_returns(other),
        _ => 0,
    };
    b.stmts.iter().map(one).sum()
}

fn all_returns(b: &Block) -> usize {
    let mut n = 0;
    deep_stmts(b, &mut |s| n += usize::from(matches!(s, Stmt::Return(_))));
    n
}

/// An argument read in place of its param: it can't change or trap while
/// the body runs.
fn direct(v: &Expr) -> bool {
    match &v.kind {
        ExprKind::Local(_)
        | ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::Char(_)
        | ExprKind::Float(_)
        | ExprKind::Unit
        | ExprKind::None
        | ExprKind::DomainLen(_)
        | ExprKind::DomainLo(_) => true,
        ExprKind::Field(base, _) => direct(base) && matches!(base.kind, ExprKind::Local(_) | ExprKind::Field(..)),
        _ => false,
    }
}

/// A read of a part of a value, which a `let` would copy.
fn indexes(v: &Expr) -> bool {
    matches!(v.kind, ExprKind::Index(..) | ExprKind::Field(..) | ExprKind::Global(_) | ExprKind::Payload(..))
}

fn scalar(ty: &Ty) -> bool {
    matches!(ty, Ty::Int | Ty::Bool | Ty::Char | Ty::Float | Ty::U64)
}

struct Remap<'m> {
    map: &'m [Sub],
    loops: HashMap<LoopId, LoopId>,
    next_loop: &'m mut LoopId,
    closures: &'m mut Vec<ClosureDef>,
    span: Option<Span>,
    ok: bool,
}

impl Remap<'_> {
    fn slot(&mut self, s: &mut Slot) {
        match self.map.get(*s) {
            Some(Sub::Slot(n)) => *s = *n,
            Some(Sub::Expr(Expr { kind: ExprKind::Local(n), .. })) => *s = *n,
            _ => self.ok = false,
        }
    }

    fn loop_id(&mut self, id: &mut LoopId) {
        let next = &mut *self.next_loop;
        *id = *self.loops.entry(*id).or_insert_with(|| {
            *next += 1;
            *next - 1
        });
    }

    fn place(&mut self, p: &mut Place) {
        if let Some(span) = self.span {
            p.span = span;
        }
        match &mut p.kind {
            PlaceKind::Local(s) => self.slot(s),
            PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) => self.place(base),
            PlaceKind::Index(base, ix) => {
                self.place(base);
                ix.iter_mut().for_each(|e| self.expr(e));
            }
        }
    }

    fn block(&mut self, b: &mut Block) {
        for stmt in &mut b.stmts {
            match stmt {
                Stmt::Let(s, _) | Stmt::For(_, s, ..) => self.slot(s),
                Stmt::Assign(p, ..) => {
                    if let Some(span) = self.span {
                        p.span = span;
                    }
                    self.place_root(p);
                }
                Stmt::Return(Some(v)) => *stmt = Stmt::Give(std::mem::replace(v, Expr { kind: ExprKind::Unit, ty: Ty::Unit, span: Span::default() })),
                Stmt::Return(None) => self.ok = false,
                Stmt::Assert { span, .. } | Stmt::Trap(_, span) => {
                    if let Some(at) = self.span {
                        *span = at;
                    }
                }
                _ => {}
            }
            match stmt {
                Stmt::While(id, ..) | Stmt::For(id, ..) => self.loop_id(id),
                Stmt::Break(Some(id)) | Stmt::Continue(Some(id)) => self.loop_id(id),
                _ => {}
            }
            stmt_exprs_mut(stmt, &mut |e| self.expr(e));
            stmt_blocks_mut(stmt, &mut |inner| self.block(inner));
        }
    }

    /// The root slot of an assigned place; its index exprs are walked with
    /// the statement's.
    fn place_root(&mut self, p: &mut Place) {
        match &mut p.kind {
            PlaceKind::Local(s) => self.slot(s),
            PlaceKind::Global(_) => {}
            PlaceKind::Field(base, _) | PlaceKind::Some(base) | PlaceKind::Index(base, _) => {
                self.place_root(base)
            }
        }
    }

    fn arg_places(&mut self, args: &mut [Arg]) {
        for a in args {
            if let Arg::Mut(p) = a {
                self.place_root(p);
            }
        }
    }

    fn expr(&mut self, e: &mut Expr) {
        if let Some(span) = self.span {
            e.span = span;
        }
        match &mut e.kind {
            ExprKind::Local(s) => {
                match self.map.get(*s) {
                    Some(Sub::Slot(n)) => *s = *n,
                    Some(Sub::Expr(v)) => {
                        let span = e.span;
                        *e = v.clone();
                        if self.span.is_some() {
                            e.span = span;
                        }
                    }
                    None => self.ok = false,
                }
                return;
            }
            ExprKind::Do(b) => self.block(b),
            ExprKind::Closure(c) => {
                let Some(mut def) = self.closures.get(*c).cloned() else {
                    self.ok = false;
                    return;
                };
                for cap in def.captures.iter_mut().filter(|cap| !cap.is_global) {
                    match self.map.get(cap.outer) {
                        Some(Sub::Slot(n)) => cap.outer = *n,
                        Some(Sub::Expr(Expr { kind: ExprKind::Local(n), .. })) => cap.outer = *n,
                        Some(Sub::Expr(Expr { kind: ExprKind::Global(g), .. })) => {
                            cap.outer = *g;
                            cap.is_global = true;
                        }
                        _ => self.ok = false,
                    }
                }
                if let Some(span) = self.span {
                    respan(&mut def.body, span);
                }
                *c = self.closures.len();
                self.closures.push(def);
            }
            ExprKind::Call(_, args) | ExprKind::CallExpr(_, args) => self.arg_places(args),
            ExprKind::Mutate(_, p, _) | ExprKind::MutSlice(p, ..) | ExprKind::MutLine(p, ..) => {
                self.place_root(p)
            }
            ExprKind::Sort(p, by, _) => {
                self.place_root(p);
                if let SortBy::Key(s, _) = by {
                    self.slot(s);
                }
            }
            _ => {}
        }
        children_mut(e, &mut |child| self.expr(child));
    }
}

fn respan(b: &mut Block, span: Span) {
    fn expr(e: &mut Expr, span: Span) {
        e.span = span;
        if let ExprKind::Do(inner) = &mut e.kind {
            respan(inner, span);
        }
        children_mut(e, &mut |c| expr(c, span));
    }
    for stmt in &mut b.stmts {
        stmt_exprs_mut(stmt, &mut |e| expr(e, span));
        stmt_blocks_mut(stmt, &mut |inner| respan(inner, span));
    }
}

/// Drops each `let` of a closure nothing reads any more.
fn drop_unused_closures(body: &mut Block, closures: &[ClosureDef]) {
    let mut read: BTreeSet<Slot> = BTreeSet::new();
    deep_exprs(body, &mut |e| match e.kind {
        ExprKind::Local(s) => {
            read.insert(s);
        }
        ExprKind::Closure(c) => {
            for cap in closures.get(c).into_iter().flat_map(|d| &d.captures) {
                if !cap.is_global {
                    read.insert(cap.outer);
                }
            }
        }
        _ => {}
    });
    crate::walk::blocks_mut(body, &mut |b| {
        b.stmts.retain(|s| {
            !matches!(s, Stmt::Let(slot, Some(Expr { kind: ExprKind::Closure(_), .. })) if !read.contains(slot))
        })
    });
}
