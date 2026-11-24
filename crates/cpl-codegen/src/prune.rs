//! Branches decided at compile time: an `if` whose condition folds to a
//! constant (an `O: Trait` test, `same(op, +)`) keeps only the branch it
//! takes, so the C++ carries neither the other branch nor the fns only it
//! calls. A build without checks drops `assert`s here for the same reason.
//!
//! An optional is also known to be empty where it can only be `None`: a
//! parameter every call passes `None` (a `by: ?F = None` nobody sets), and
//! a struct field every construction fills with `None` or copies from the
//! same field, so `if let f = self.lt` goes when no `Heap` ever gets an `lt`.

use crate::walk::{deep_exprs, deep_stmts, path_root, Effects};
use cpl_ir::{Arg, Block, Expr, ExprKind, FnId, Place, PlaceKind};
use cpl_ir::{Program, Slot, Stmt, Ty};
use std::collections::{BTreeMap, BTreeSet};

/// `asserts` is whether the build checks them; when it does not, they go
/// too, so a fn only an `assert` calls is not emitted.
pub(crate) fn prune(program: &mut Program, asserts: bool) {
    if !asserts {
        keep_stores_of_asserts(program);
    }
    let mut nones = Nones {
        params: none_params(program),
        fields: BTreeSet::new(),
    };
    run(program, asserts, &nones);
    nones.fields = none_fields(program);
    if !nones.fields.is_empty() {
        run(program, asserts, &nones);
    }
}

/// A skipped `assert` whose condition stores still runs it, so
/// `assert floyd_warshall(mut d)` closes `d` in every build.
fn keep_stores_of_asserts(program: &mut Program) {
    let mut storing = BTreeSet::new();
    let effects: Vec<Effects> = program
        .fns
        .iter()
        .map(|f| Effects::of_block(&f.body))
        .collect();
    loop {
        let before = storing.len();
        for (id, e) in effects.iter().enumerate() {
            if !e.globals.is_empty() || e.calls.iter().any(|c| storing.contains(c)) {
                storing.insert(id);
            }
        }
        if storing.len() == before {
            break;
        }
    }
    let stores = |cond: &Expr| {
        let e = Effects::of_expr(cond);
        !e.locals.is_empty() || !e.globals.is_empty() || e.calls.iter().any(|c| storing.contains(c))
    };
    fn visit(b: &mut Block, stores: &dyn Fn(&Expr) -> bool) {
        for stmt in &mut b.stmts {
            match stmt {
                Stmt::Assert {
                    cond, kept: false, ..
                } if stores(cond) => *stmt = Stmt::Expr(cond.clone()),
                Stmt::If(_, then, otherwise) => {
                    visit(then, stores);
                    visit(otherwise, stores);
                }
                Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => visit(body, stores),
                _ => {}
            }
        }
    }
    for f in &mut program.fns {
        visit(&mut f.body, &stores);
    }
    for c in &mut program.closures {
        visit(&mut c.body, &stores);
    }
    visit(&mut program.main, &stores);
    visit(&mut program.setup, &stores);
}

/// Optionals that hold `None` wherever they are read.
struct Nones {
    params: BTreeSet<(FnId, Slot)>,
    /// `(struct, field)`.
    fields: BTreeSet<(usize, usize)>,
}

/// What one body knows: which of its slots can only be `None`.
struct Cx<'a> {
    nones: &'a Nones,
    home: Option<FnId>,
    /// Slots the body (or a closure, by reference) may store to.
    written: BTreeSet<Slot>,
    captured: BTreeSet<Slot>,
    lets: BTreeSet<Slot>,
}

impl Cx<'_> {
    fn is_none(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::None => true,
            ExprKind::Local(s) => {
                self.lets.contains(s)
                    || self
                        .home
                        .is_some_and(|f| self.nones.params.contains(&(f, *s)))
            }
            ExprKind::Field(base, i) => {
                matches!(base.ty, Ty::Struct(id) if self.nones.fields.contains(&(id, *i)))
                    && path_root(base).is_some()
            }
            _ => false,
        }
    }
}

fn run(program: &mut Program, asserts: bool, nones: &Nones) {
    let cx = |home, body: &Block| Cx {
        nones,
        home,
        written: written(program, body),
        captured: captured(program, body),
        lets: BTreeSet::new(),
    };
    let mut cxs: Vec<Cx> = program
        .fns
        .iter()
        .enumerate()
        .map(|(id, f)| cx(Some(id), &f.body))
        .collect();
    let mut closure_cxs: Vec<Cx> = program.closures.iter().map(|c| cx(None, &c.body)).collect();
    let (mut main_cx, mut setup_cx) = (cx(None, &program.main), cx(None, &program.setup));
    for (f, cx) in program.fns.iter_mut().zip(&mut cxs) {
        block(&mut f.body, asserts, cx);
    }
    for (c, cx) in program.closures.iter_mut().zip(&mut closure_cxs) {
        block(&mut c.body, asserts, cx);
    }
    block(&mut program.main, asserts, &mut main_cx);
    block(&mut program.setup, asserts, &mut setup_cx);
    for (f, cx) in program.fns.iter_mut().zip(&cxs) {
        drop_unread(&mut f.body, &cx.lets, &cx.captured);
    }
    for (c, cx) in program.closures.iter_mut().zip(&closure_cxs) {
        drop_unread(&mut c.body, &cx.lets, &cx.captured);
    }
    drop_unread(&mut program.main, &main_cx.lets, &main_cx.captured);
    drop_unread(&mut program.setup, &setup_cx.lets, &setup_cx.captured);
}

/// Drops the `let`s of `lets` (each binds a `None` read from a variable)
/// that nothing reads now that the branches testing them are gone, so the
/// variable they read is not left named for nothing.
fn drop_unread(body: &mut Block, lets: &BTreeSet<Slot>, captured: &BTreeSet<Slot>) {
    if lets.is_empty() {
        return;
    }
    let mut read = captured.clone();
    deep_exprs(body, &mut |e| {
        if let ExprKind::Local(s) = e.kind {
            read.insert(s);
        }
    });
    let dead: BTreeSet<Slot> = lets.difference(&read).copied().collect();
    fn go(b: &mut Block, dead: &BTreeSet<Slot>) {
        // The `let` still runs where it is written, as a profile counts it.
        for s in &mut b.stmts {
            if let Stmt::Let(slot, Some(init)) = s {
                if dead.contains(slot) {
                    let (ty, span) = (cpl_ir::Ty::Unit, init.span);
                    *s = Stmt::Expr(Expr {
                        kind: ExprKind::Unit,
                        ty,
                        span,
                    });
                }
            }
        }
        for stmt in &mut b.stmts {
            match stmt {
                Stmt::If(_, then, otherwise) => {
                    go(then, dead);
                    go(otherwise, dead);
                }
                Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => go(body, dead),
                _ => {}
            }
        }
    }
    go(body, &dead);
}

/// The slots of `body` a closure built in it captures.
fn captured(program: &Program, body: &Block) -> BTreeSet<Slot> {
    let mut out = BTreeSet::new();
    deep_exprs(body, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            let caps = &program.closures[cid].captures;
            out.extend(caps.iter().filter(|c| !c.is_global).map(|c| c.outer));
        }
    });
    out
}

/// The slots `body` stores to, or lends to a closure by reference.
fn written(program: &Program, body: &Block) -> BTreeSet<Slot> {
    let mut out = Effects::of_block(body).locals;
    deep_exprs(body, &mut |e| {
        if let ExprKind::Closure(cid) = e.kind {
            let caps = &program.closures[cid].captures;
            out.extend(
                caps.iter()
                    .filter(|c| c.by_ref && !c.is_global)
                    .map(|c| c.outer),
            );
        }
    });
    out
}

/// Every block in the program, with the fn it is the body of.
fn bodies(program: &Program) -> impl Iterator<Item = (Option<FnId>, &Block)> {
    let fns = program
        .fns
        .iter()
        .enumerate()
        .map(|(id, f)| (Some(id), &f.body));
    let closures = program.closures.iter().map(|c| (None, &c.body));
    fns.chain(closures)
        .chain([(None, &program.main), (None, &program.setup)])
}

/// Optional parameters that every call passes a literal `None` and the
/// fn never stores to.
fn none_params(program: &Program) -> BTreeSet<(FnId, Slot)> {
    let mut seen: BTreeMap<(FnId, Slot), bool> = BTreeMap::new();
    for (_, body) in bodies(program) {
        deep_exprs(body, &mut |e| {
            if let ExprKind::Call(id, args) = &e.kind {
                for (slot, arg) in args.iter().enumerate() {
                    let none = matches!(arg, Arg::Value(v) if v.kind == ExprKind::None);
                    *seen.entry((*id, slot)).or_insert(true) &= none;
                }
            }
        });
    }
    let outside: BTreeSet<FnId> = program
        .entry_fns
        .iter()
        .chain(&program.judge_fns)
        .copied()
        .collect();
    seen.into_iter()
        .filter(|&((id, slot), none)| {
            let f = &program.fns[id];
            none && !outside.contains(&id)
                && f.params.get(slot).is_some_and(|p| !p.mutable)
                && !written(program, &f.body).contains(&slot)
        })
        .map(|(key, _)| key)
        .collect()
}

/// Optional struct fields that every construction fills with `None` or
/// with the same field of another value of the struct, and nothing stores to.
fn none_fields(program: &Program) -> BTreeSet<(usize, usize)> {
    let mut bad = BTreeSet::new();
    for (_, body) in bodies(program) {
        deep_exprs(body, &mut |e| match (&e.kind, &e.ty) {
            (ExprKind::Tuple(items), Ty::Struct(id)) => {
                for (i, item) in items.iter().enumerate() {
                    let kept = match &item.kind {
                        ExprKind::None => true,
                        ExprKind::Field(base, j) => *j == i && base.ty == Ty::Struct(*id),
                        _ => false,
                    };
                    if !kept {
                        bad.insert((*id, i));
                    }
                }
            }
            (ExprKind::Call(_, args) | ExprKind::CallExpr(_, args), _) => {
                for arg in args {
                    if let Arg::Mut(place) = arg {
                        stored_fields(place, &mut bad);
                    }
                }
            }
            (
                ExprKind::MutSlice(place, ..)
                | ExprKind::Mutate(_, place, _)
                | ExprKind::Sort(place, ..),
                _,
            ) => stored_fields(place, &mut bad),
            _ => {}
        });
        deep_stmts(body, &mut |stmt| {
            if let Stmt::Assign(place, ..) = stmt {
                stored_fields(place, &mut bad);
            }
        });
    }
    let mut out = BTreeSet::new();
    for (id, def) in program.structs.iter().enumerate() {
        for (i, (_, ty)) in def.fields.iter().enumerate() {
            if matches!(ty, Ty::Opt(_)) && !bad.contains(&(id, i)) {
                out.insert((id, i));
            }
        }
    }
    out
}

fn stored_fields(place: &Place, bad: &mut BTreeSet<(usize, usize)>) {
    match &place.kind {
        PlaceKind::Local(_) | PlaceKind::Global(_) => {}
        PlaceKind::Field(base, i) => {
            if let Ty::Struct(id) = base.ty {
                bad.insert((id, *i));
            }
            stored_fields(base, bad);
        }
        PlaceKind::Index(base, _) | PlaceKind::Some(base) => stored_fields(base, bad),
    }
}

/// The value of a condition made only of `true`, `false`, comparisons of
/// two integer literals (`P < 3_037_000_500` once `P` is known), `not`, `and`
/// and `or`; a side that is not known is evaluated only when the known one lets
/// it be skipped.
pub(crate) fn known(e: &Expr) -> Option<bool> {
    known_in(e, None)
}

fn known_in(e: &Expr, cx: Option<&Cx>) -> Option<bool> {
    cpl_ir::effects::known_where(e, &|x| cx.is_some_and(|cx| cx.is_none(x)))
}

fn block(b: &mut Block, asserts: bool, cx: &mut Cx) {
    if !asserts {
        b.stmts
            .retain(|s| !matches!(s, Stmt::Assert { kept: false, .. }));
    }
    for stmt in &mut b.stmts {
        match stmt {
            Stmt::Let(slot, Some(init)) if !cx.written.contains(slot) && cx.is_none(init) => {
                cx.lets.insert(*slot);
            }
            Stmt::If(cond, then, otherwise) => {
                if let Some(taken) = known_in(cond, Some(cx)) {
                    cond.kind = ExprKind::Bool(taken);
                    let dead = if taken { &mut *otherwise } else { &mut *then };
                    dead.stmts.clear();
                }
                block(then, asserts, cx);
                block(otherwise, asserts, cx);
            }
            Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => block(body, asserts, cx),
            _ => {}
        }
    }
    // Nothing after a statement that always leaves the block runs, so an
    // `if same(over.add, +)` folded to its `return` drops the rest of the fn.
    if let Some(end) = b.stmts.iter().position(leaves) {
        b.stmts.truncate(end + 1);
    }
}

fn leaves(stmt: &Stmt) -> bool {
    let any = |b: &Block| b.stmts.iter().any(leaves);
    match stmt {
        Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Give(_) => true,
        Stmt::If(cond, then, otherwise) => match cond.kind {
            ExprKind::Bool(true) => any(then),
            ExprKind::Bool(false) => any(otherwise),
            _ => any(then) && any(otherwise),
        },
        _ => false,
    }
}
