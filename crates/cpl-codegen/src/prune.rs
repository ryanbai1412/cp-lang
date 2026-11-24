//! Branches decided at compile time: an `if` whose condition folds to a
//! constant (an `O: Trait` test, `same(op, +)`) keeps only the branch it
//! takes, so the C++ carries neither the other branch nor the fns only it
//! calls. A build without checks drops `assert`s here for the same reason.
//!
//! An optional is also known to be empty where it can only be `None`: a
//! parameter every call passes `None` (a `by: ?F = None` nobody sets), and
//! a struct field every construction fills with `None` or copies from the
//! same field, so `if let f = self.lt` goes when no `Heap` ever gets an `lt`.
//! A list field is known to be empty the same way, so its `len` is 0: a
//! `SegTree` over a domain never makes the `over:` layout's `lc`, and every
//! `if lc.len > 0` branch of the growing tree goes. A field's constructions
//! and stores count only in code that can run once the branches known so far
//! are left out, so a store only those branches make does not keep them.

use crate::uses::effects::{known_with, Touch};
use crate::uses::places::{each_call, each_closure, each_part, each_put, Put};
use crate::uses::split::each_value;
use crate::walk::path_root;
use cpl_ir::{Arg, Block, Builtin, ClosureId, Expr, ExprKind, FnId, Place, PlaceKind};
use cpl_ir::{Program, Slot, Stmt, Ty};
use std::collections::BTreeSet;

/// `asserts` is whether the build checks them; when it does not, they go
/// too, so a fn only an `assert` calls is not emitted.
pub(crate) fn prune(program: &mut Program, asserts: bool) {
    if !asserts {
        keep_stores_of_asserts(program);
    }
    let mut nones = Nones {
        params: none_params(program),
        fields: BTreeSet::new(),
        empty: BTreeSet::new(),
    };
    run(program, asserts, &nones);
    known_fields(program, &mut nones);
    if !nones.fields.is_empty() || !nones.empty.is_empty() {
        run(program, asserts, &nones);
    }
}

/// A skipped `assert` whose condition stores still runs it, so
/// `assert floyd_warshall(mut d)` closes `d` in every build.
fn keep_stores_of_asserts(program: &mut Program) {
    let mut storing = BTreeSet::new();
    let effects: Vec<Touch> = program
        .fns
        .iter()
        .map(|f| Touch::of_block(&f.body))
        .collect();
    loop {
        let before = storing.len();
        for (id, e) in effects.iter().enumerate() {
            if e.writes.iter().any(|w| w.0) || e.calls.iter().any(|c| storing.contains(c)) {
                storing.insert(id);
            }
        }
        if storing.len() == before {
            break;
        }
    }
    let stores = |cond: &Expr| {
        let e = Touch::of_expr(cond);
        !e.writes.is_empty() || e.calls.iter().any(|c| storing.contains(c))
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
    /// List fields that are empty wherever they are read, as `(struct, field)`.
    empty: BTreeSet<(usize, usize)>,
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

    /// The value of an `Int` known without being a literal: the `len` of a
    /// list field known to be empty.
    fn int(&self, e: &Expr) -> Option<i64> {
        let ExprKind::Builtin(Builtin::Len, args) = &e.kind else {
            return None;
        };
        let [list] = &args[..] else { return None };
        let ExprKind::Field(base, i) = &list.kind else {
            return None;
        };
        (matches!(base.ty, Ty::Struct(id) if self.nones.empty.contains(&(id, *i)))
            && path_root(base).is_some())
        .then_some(0)
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
    each_value(body, &mut |e| {
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
    each_closure(body, &mut |cid| {
        let caps = &program.closures[cid].captures;
        out.extend(caps.iter().filter(|c| !c.is_global).map(|c| c.outer));
    });
    out
}

/// The slots `body` stores to, or lends to a closure by reference.
fn written(program: &Program, body: &Block) -> BTreeSet<Slot> {
    let mut out = Touch::of_block(body).stored_locals();
    each_closure(body, &mut |cid| {
        let caps = &program.closures[cid].captures;
        out.extend(
            caps.iter()
                .filter(|c| c.by_ref && !c.is_global)
                .map(|c| c.outer),
        );
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

/// Optional parameters that every call passes a literal `None`, or a
/// parameter of its own fn that is one of them, and the fn never stores to.
/// Starts from every such parameter and drops the ones a call passes
/// something else, until the guess holds.
fn none_params(program: &Program) -> BTreeSet<(FnId, Slot)> {
    enum Passed {
        None,
        Param(FnId, Slot),
        Other,
    }
    let mut passes: Vec<(FnId, Slot, Passed)> = Vec::new();
    for (home, body) in bodies(program) {
        each_call(body, &mut |id, args| {
            for (slot, arg) in args.iter().enumerate() {
                let passed = match (arg, home) {
                    (Arg::Value(v), _) if v.kind == ExprKind::None => Passed::None,
                    (Arg::Value(v), Some(f)) => match v.kind {
                        ExprKind::Local(s) => Passed::Param(f, s),
                        _ => Passed::Other,
                    },
                    _ => Passed::Other,
                };
                passes.push((id, slot, passed));
            }
        });
    }
    let outside: BTreeSet<FnId> = program
        .entry_fns
        .iter()
        .chain(&program.judge_fns)
        .copied()
        .collect();
    let mut known: BTreeSet<(FnId, Slot)> = passes
        .iter()
        .map(|&(id, slot, _)| (id, slot))
        .filter(|&(id, slot)| {
            let f = &program.fns[id];
            !outside.contains(&id)
                && f.params.get(slot).is_some_and(|p| !p.mutable)
                && !written(program, &f.body).contains(&slot)
        })
        .collect();
    loop {
        let before = known.len();
        for (id, slot, passed) in &passes {
            let none = match passed {
                Passed::None => true,
                Passed::Param(f, s) => known.contains(&(*f, *s)),
                Passed::Other => false,
            };
            if !none {
                known.remove(&(*id, *slot));
            }
        }
        if known.len() == before {
            return known;
        }
    }
}

/// The optional fields known `None` and the list fields known empty: every
/// construction that can run fills them with `None` or `[]` or with the same
/// field of another value of the struct, and nothing that can run stores to
/// them. Starts from every such field and drops the ones the code reached
/// under the current guess stores, until the guess holds.
fn known_fields(program: &Program, nones: &mut Nones) {
    for (id, def) in program.structs.iter().enumerate() {
        for (i, (_, ty)) in def.fields.iter().enumerate() {
            match ty {
                Ty::Opt(_) => nones.fields.insert((id, i)),
                Ty::List(_) => nones.empty.insert((id, i)),
                _ => false,
            };
        }
    }
    loop {
        let bad = stored_in(program, &reached(program, nones));
        let before = nones.fields.len() + nones.empty.len();
        nones.fields.retain(|f| !bad.contains(f));
        nones.empty.retain(|f| !bad.contains(f));
        if nones.fields.len() + nones.empty.len() == before {
            return;
        }
    }
}

/// The bodies that can run when the branches `nones` decides are left out:
/// main, setup, the entry, judge and type-hook fns, and what they call.
fn reached<'p>(program: &'p Program, nones: &Nones) -> Vec<&'p Block> {
    let mut fns: BTreeSet<FnId> = program
        .entry_fns
        .iter()
        .chain(&program.judge_fns)
        .copied()
        .collect();
    for s in &program.structs {
        fns.extend(
            [s.eq_fn, s.ord.map(cpl_ir::OrdHook::id), s.show_fn]
                .into_iter()
                .flatten(),
        );
    }
    fns.extend(program.enums.iter().filter_map(|e| e.show_fn));
    let mut fn_work: Vec<FnId> = fns.iter().copied().collect();
    let mut closures: BTreeSet<ClosureId> = BTreeSet::new();
    let mut closure_work: Vec<ClosureId> = Vec::new();
    let mut out = vec![&program.main, &program.setup];
    let mut at = 0;
    loop {
        while at < out.len() {
            let cx = Cx {
                nones,
                home: None,
                written: BTreeSet::new(),
                captured: BTreeSet::new(),
                lets: BTreeSet::new(),
            };
            taken_exprs(out[at], &cx, &mut |e| match e.kind {
                ExprKind::Call(id, _) if fns.insert(id) => fn_work.push(id),
                ExprKind::Closure(id) if closures.insert(id) => closure_work.push(id),
                _ => {}
            });
            at += 1;
        }
        if let Some(id) = fn_work.pop() {
            out.push(&program.fns[id].body);
        } else if let Some(id) = closure_work.pop() {
            out.push(&program.closures[id].body);
        } else {
            return out;
        }
    }
}

/// Every expression of `b` at any depth, skipping the branch of an `if` that `cx` knows is not taken
/// and what follows a statement known to leave the block; whether `b` leaves.
fn taken_exprs<'p>(b: &'p Block, cx: &Cx, f: &mut dyn FnMut(&'p Expr)) -> bool {
    for stmt in &b.stmts {
        cpl_ir::walk::stmt_exprs(stmt, &mut |e| each_part(e, f));
        let left = match stmt {
            Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_) | Stmt::Give(_) => true,
            Stmt::If(cond, then, otherwise) => match known_in(cond, Some(cx)) {
                Some(true) => taken_exprs(then, cx, f),
                Some(false) => taken_exprs(otherwise, cx, f),
                None => taken_exprs(then, cx, f) & taken_exprs(otherwise, cx, f),
            },
            Stmt::While(_, _, body) | Stmt::For(_, _, _, body) => {
                taken_exprs(body, cx, f);
                false
            }
            _ => false,
        };
        if left {
            return true;
        }
    }
    false
}

/// The `(struct, field)`s that `bodies` construct with something other than
/// `None`, `[]` or the same field, store to, or read from input.
fn stored_in(program: &Program, bodies: &[&Block]) -> BTreeSet<(usize, usize)> {
    let mut bad = BTreeSet::new();
    for body in bodies {
        each_value(body, &mut |e| match (&e.kind, &e.ty) {
            (ExprKind::Tuple(items), Ty::Struct(id)) => {
                for (i, item) in items.iter().enumerate() {
                    let kept = match &item.kind {
                        ExprKind::None => true,
                        ExprKind::Tuple(empty) => {
                            empty.is_empty() && matches!(item.ty, Ty::List(_))
                        }
                        ExprKind::Field(base, j) => *j == i && base.ty == Ty::Struct(*id),
                        _ => false,
                    };
                    if !kept {
                        bad.insert((*id, i));
                    }
                }
            }
            (ExprKind::Read(_) | ExprKind::ReadFrom(..), ty) => {
                let mut ids = BTreeSet::new();
                structs_in(program, ty, &mut ids);
                for id in ids {
                    bad.extend((0..program.structs[id].fields.len()).map(|i| (id, i)));
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
                | ExprKind::MutLine(place, ..)
                | ExprKind::Mutate(_, place, _)
                | ExprKind::Sort(place, ..),
                _,
            ) => stored_fields(place, &mut bad),
            _ => {}
        });
        each_put(body, &mut |put, _| {
            if let Put::Store(place, _) = put {
                stored_fields(place, &mut bad);
            }
        });
    }
    bad
}

/// The structs a value of `ty` holds, at any depth.
fn structs_in(program: &Program, ty: &Ty, out: &mut BTreeSet<usize>) {
    match ty {
        Ty::Struct(id) => {
            if out.insert(*id) {
                for (_, t) in &program.structs[*id].fields {
                    structs_in(program, t, out);
                }
            }
        }
        Ty::Enum(id) => {
            for (_, fields) in &program.enums[*id].variants {
                for t in fields {
                    structs_in(program, t, out);
                }
            }
        }
        Ty::Range(t) | Ty::Opt(t) | Ty::List(t) | Ty::Set(t) | Ty::HashSet(t) | Ty::Mut(t) => {
            structs_in(program, t, out)
        }
        Ty::Array { elem, .. } => structs_in(program, elem, out),
        Ty::Map(k, v) | Ty::HashMap(k, v) => {
            structs_in(program, k, out);
            structs_in(program, v, out);
        }
        Ty::Tuple(ts) => ts.iter().for_each(|t| structs_in(program, t, out)),
        Ty::Record(fs) => fs.iter().for_each(|(_, t)| structs_in(program, t, out)),
        _ => {}
    }
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
    known_with(e, &|x| cx.is_some_and(|cx| cx.is_none(x)), &|x| {
        cx.and_then(|cx| cx.int(x))
    })
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
