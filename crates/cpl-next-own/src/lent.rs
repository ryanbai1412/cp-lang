//! Which values are lent rather than moved in (R05.5.3), decided on generic IR between
//! lowering and link: link makes one instance per mode, so it reads these marks to key them.
//!
//! A held argument (`plain::lends`) that lives in a place lends its type argument; a fresh one
//! is moved in. A struct literal stores a held parameter of its fn lent, a variadic pack
//! lends each place it reads for the call, and a closure is lent each binding it holds the
//! address of that nothing can reassign (R05.10.1). The marks then follow the values: a call's result,
//! a struct's type arguments, the locals they are bound to and a closure's captures.
use crate::lends;
use cpl_next_decls::decls::SelfMode;
use cpl_next_decls::views::Signatures;
use cpl_next_ir::ir::{visit_places, Arg, Expr, Func, FuncKey, Place, Stmt, Target, EK};
use cpl_next_ir::ty::*;
use std::collections::{BTreeSet, HashMap};

/// Marks what each generic fn lends, its lambdas through the fn they are written in;
/// `mutable` says which globals are `var`s.
pub(crate) fn mark(
    d: &dyn Signatures,
    funcs: &mut HashMap<FuncKey, Func<Ty>>,
    globals: &mut [Ty],
    mutable: &[bool],
) {
    let sigs: HashMap<FnId, (Vec<Ty>, Ty)> = (funcs.values())
        .filter_map(|f| match f.key {
            FuncKey::Decl(id) | FuncKey::Ctx(id, _) => Some((id, f)),
            _ => None,
        })
        .map(|(id, f)| {
            let ps = f.params.iter().map(|(l, _)| f.locals[*l].clone()).collect();
            (id, (ps, f.ret.clone()))
        })
        .collect();
    let mut keys: Vec<FuncKey> = (funcs.keys())
        .filter(|k| !matches!(k, FuncKey::Lambda(_)))
        .cloned()
        .collect();
    keys.sort();
    let mut copies: HashMap<FnId, Vec<FuncKey>> = HashMap::new();
    for k in &keys {
        if let Some(id) = k.decl() {
            copies.entry(id).or_default().push(k.clone());
        }
    }
    let mut w = Lw {
        d,
        sigs,
        copies,
        funcs,
        globals,
        mutable,
        done: BTreeSet::new(),
        marked: BTreeSet::new(),
    };
    for k in keys {
        w.decl_func(&k);
    }
    // A lambda no fn makes (nothing reaches it) is still marked on its own.
    let mut rest: Vec<FuncKey> = (w.funcs.keys())
        .filter(|k| matches!(k, FuncKey::Lambda(id) if !w.done.contains(id)))
        .cloned()
        .collect();
    rest.sort();
    for k in rest {
        w.func(&k, Vec::new());
    }
}

struct Lw<'a> {
    d: &'a dyn Signatures,
    /// Each fn's declared parameter types, receiver first, and its result type.
    sigs: HashMap<FnId, (Vec<Ty>, Ty)>,
    /// Each declared fn's copies: its body, and one per set of context values.
    copies: HashMap<FnId, Vec<FuncKey>>,
    funcs: &'a mut HashMap<FuncKey, Func<Ty>>,
    globals: &'a mut [Ty],
    mutable: &'a [bool],
    done: BTreeSet<LambdaId>,
    /// The declared fns marked or being marked.
    marked: BTreeSet<FuncKey>,
}

/// One fn being marked: its locals and the parameters its result holds.
struct Fr {
    locals: Vec<Ty>,
    names: Vec<String>,
    vars: Vec<bool>,
    held: Vec<usize>,
    /// The temps a variadic argument is spilled to before its call: their struct is a pack.
    packs: BTreeSet<usize>,
    params: Vec<usize>,
    last: crate::Last,
}

impl Lw<'_> {
    /// Marks a declared fn once, and then its result type in the signature its callers read:
    /// a result that holds a place it lends is lent there (R05.6.1), whatever its type says.
    fn decl_func(&mut self, k: &FuncKey) {
        if !self.marked.insert(k.clone()) {
            return;
        }
        let held = match k {
            FuncKey::Decl(id) | FuncKey::Ctx(id, _) => {
                let holds = lends(self.d, *id);
                let Some(f) = self.funcs.get(k) else { return };
                (f.params.iter().enumerate())
                    .filter(|(i, _)| holds.contains(i))
                    .map(|(_, (l, _))| *l)
                    .collect()
            }
            _ => Vec::new(),
        };
        self.func(k, held);
        if let (Some(id), Some(f)) = (k.decl(), self.funcs.get(k)) {
            if let Some((_, ret)) = self.sigs.get_mut(&id) {
                *ret = mark_lent(ret, &f.ret);
            }
        }
    }
    /// Marks the fn a call names before the call reads its result type.
    fn callee(&mut self, id: FnId) {
        for k in self.copies.get(&id).cloned().unwrap_or_default() {
            self.decl_func(&k);
        }
    }
    fn func(&mut self, k: &FuncKey, held: Vec<usize>) {
        if let FuncKey::Lambda(id) = k {
            if !self.done.insert(*id) {
                return;
            }
        }
        let Some(mut f) = self.funcs.remove(k) else {
            return;
        };
        let mut packs = BTreeSet::new();
        each_call(&f.body, &mut |t, args| {
            if let Some(Arg::Value(Expr {
                kind: EK::Copy(Place::Local(l)),
                ..
            })) = self.pack(t).and_then(|i| args.get(i))
            {
                packs.insert(*l);
            }
        });
        let last = crate::Last::new(&f.body);
        let mut fr = Fr {
            locals: std::mem::take(&mut f.locals),
            names: f.names.clone(),
            vars: f.vars.clone(),
            held,
            packs,
            params: f.params.iter().map(|(l, _)| *l).collect(),
            last,
        };
        self.block(&mut fr, &mut f.body);
        f.locals = fr.locals;
        // The result takes the marks of each value returned: a view of a held param is lent.
        let mut ret = f.ret.clone();
        returns(&f.body, &mut |t| {
            if strip_lent(t) == strip_lent(&ret) {
                ret = mark_lent(&ret, t).unlent().clone();
            }
        });
        f.ret = ret;
        self.funcs.insert(k.clone(), f);
    }
    fn block(&mut self, fr: &mut Fr, b: &mut [Stmt<Ty>]) {
        for s in b {
            self.stmt(fr, s);
        }
    }
    fn stmt(&mut self, fr: &mut Fr, s: &mut Stmt<Ty>) {
        fr.last.stmt(s);
        match s {
            Stmt::Let(l, v) => {
                let pack = fr.packs.contains(l);
                self.expr_in(fr, v, pack);
                if let Some(t) = fr.locals.get_mut(*l) {
                    if v.ty != *t && strip_lent(&v.ty) == *t {
                        *t = v.ty.clone();
                    }
                }
            }
            Stmt::Assign(p, v) => {
                self.expr(fr, v);
                // A top-level binding takes the marks of the value it is bound to.
                if let Place::Global(g) = p {
                    if let Some(t) = self.globals.get_mut(*g) {
                        if v.ty != *t && strip_lent(&v.ty) == *t && strip_lent(t) == *t {
                            *t = v.ty.clone();
                        }
                    }
                }
            }
            Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) | Stmt::Check { cond: e, .. } => {
                self.expr(fr, e)
            }
            Stmt::Return(e) => e.iter_mut().for_each(|e| self.expr(fr, e)),
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: es, .. }) => {
                es.iter_mut().for_each(|e| self.expr(fr, e))
            }
            Stmt::If(c, a, b) => {
                self.expr(fr, c);
                self.block(fr, a);
                self.block(fr, b);
            }
            Stmt::Loop(a, b, _) => {
                fr.last.depth += 1;
                self.block(fr, a);
                self.block(fr, b);
                fr.last.depth -= 1;
            }
            Stmt::Switch(c, arms) => {
                self.expr(fr, c);
                arms.iter_mut().for_each(|a| self.block(fr, a));
            }
            Stmt::Region {
                target, args, body, ..
            } => {
                self.args(fr, target, args);
                self.target(fr, target, args);
                self.block(fr, body);
            }
            Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
        }
    }
    /// Which argument of a call to `target` is its variadic pack.
    fn pack(&self, target: &Target<Ty>) -> Option<usize> {
        let Target::Fn(f) = target else { return None };
        let fd = self.d.f(f.decl);
        let recv = (fd.self_mode != SelfMode::None) as usize;
        fd.params.iter().position(|p| p.variadic).map(|i| i + recv)
    }
    fn args(&mut self, fr: &mut Fr, target: &Target<Ty>, args: &mut [Arg<Ty>]) {
        let pack = self.pack(target);
        // R05.5.3: an owned local the result holds, read for the last time, moves in.
        if let Target::Fn(f) = target {
            let holds = lends(self.d, f.decl);
            for (i, a) in args.iter_mut().enumerate() {
                if holds.contains(&i) && moves_in(fr, a) {
                    if let Arg::Value(Expr {
                        kind: EK::Pass(_, view, _),
                        ..
                    }) = a
                    {
                        *view = false;
                    }
                }
            }
        }
        for (i, a) in args.iter_mut().enumerate() {
            if let Arg::Value(v) = a {
                self.expr_in(fr, v, pack == Some(i));
            }
        }
    }
    /// The callee with each type argument a held argument fills from a place lent, and the
    /// receiver type a bound's method is picked by lent where the receiver is.
    fn target(&mut self, fr: &mut Fr, target: &mut Target<Ty>, args: &[Arg<Ty>]) {
        match target {
            Target::Fn(f) => self.lent_targs(fr, f, args),
            Target::Trait {
                self_ty,
                args: tr_args,
                ..
            } => {
                if let Some(t) = args.first().and_then(|a| self.arg_ty(fr, a)) {
                    if t != *self_ty && strip_lent(&t) == *self_ty {
                        *self_ty = t;
                    }
                }
                // A trait argument an argument fills (`Add<S>`'s `S`) takes that argument's
                // marks, as a fn's type argument does: the impl is picked for the type given.
                let given: Vec<Ty> = (args.iter().skip(1))
                    .filter_map(|a| self.arg_ty(fr, a))
                    .collect();
                for t in tr_args.iter_mut() {
                    if let Some(g) = given.iter().find(|g| *g != t && strip_lent(g) == *t) {
                        *t = g.clone();
                    }
                }
            }
            Target::Value(v) => self.expr(fr, v),
            Target::Case | Target::Inst(_) => {}
        }
    }
    fn lent_targs(&self, fr: &Fr, f: &mut FnRef, args: &[Arg<Ty>]) {
        let Some((ps, _)) = self.sigs.get(&f.decl) else {
            return;
        };
        let holds = lends(self.d, f.decl);
        for (i, (decl, a)) in ps.iter().zip(args).enumerate() {
            if let Some(t) = self.arg_ty(fr, a) {
                rebind(decl, &t, &mut f.targs);
            }
            // A held argument that lives in a place is lent; a fresh one is moved in.
            if let Ty::Param(k) = decl {
                if holds.contains(&i) && placed(a) {
                    if let Some(t) = f.targs.get_mut(*k).filter(|t| !matches!(t, Ty::Lent(_))) {
                        *t = Ty::Lent(Box::new(t.clone()));
                    }
                }
            }
        }
    }
    fn expr(&mut self, fr: &mut Fr, e: &mut Expr<Ty>) {
        self.expr_in(fr, e, false)
    }
    /// `pack` when `e` is a variadic argument: a place it reads is lent for the call.
    fn expr_in(&mut self, fr: &mut Fr, e: &mut Expr<Ty>, pack: bool) {
        match &mut e.kind {
            EK::Copy(p) | EK::Pass(p, ..) => {
                if let Some(t) = self.place_ty(fr, p) {
                    // A whole binding is read as it is held; a lent field, where it points.
                    let whole = matches!(p, Place::Local(_) | Place::Global(_));
                    e.ty = match (&e.ty, t) {
                        (Ty::Ref(_), t @ Ty::Lent(_)) if whole => t,
                        (_, t) if whole => mark_lent(&e.ty, &t),
                        (_, t) => mark_lent(&e.ty, &t).unlent().clone(),
                    };
                }
            }
            EK::MakeStruct(xs) => {
                xs.iter_mut().for_each(|x| self.expr(fr, x));
                self.make_struct(fr, e, pack);
            }
            EK::MakeClosure(xs) => {
                xs.iter_mut().for_each(|x| self.capture(fr, x));
                if let Ty::Clo(c) = &e.ty {
                    if let CloRef::Lambda(id, ..) = **c {
                        self.lambda(fr, id, xs);
                    }
                }
            }
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(fr, x),
            EK::Intr(_, args, _) => (args.iter_mut()).for_each(|a| match a {
                Arg::Value(v) => self.expr(fr, v),
                Arg::View(_) | Arg::Lend(_) => {}
            }),
            EK::Call(target, args, _) => {
                self.args(fr, target, args);
                self.target(fr, target, args);
                if let Target::Fn(f) = target {
                    self.callee(f.decl);
                    if let Some((_, ret)) = self.sigs.get(&f.decl) {
                        let ret = Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        }
                        .ty(ret);
                        // A result held directly is its own storage, not a lent slot.
                        e.ty = mark_lent(&e.ty, &ret).unlent().clone();
                    }
                }
            }
            EK::CallValue(c, xs, _) => {
                self.expr(fr, c);
                (xs.iter_mut()).for_each(|a| match a {
                    Arg::Value(v) => self.expr(fr, v),
                    Arg::View(_) | Arg::Lend(_) => {}
                });
            }
            EK::Int(_)
            | EK::F64(_)
            | EK::Bool(_)
            | EK::Unit
            | EK::Text(_)
            | EK::AddrOf(_)
            | EK::Lent(_)
            | EK::Tag(_)
            | EK::DomLen(_)
            | EK::DomLo(_)
            | EK::DomHeld(_)
            | EK::DomCounted(_)
            | EK::HasBound(..) => {}
        }
    }
    /// A capture keeps the marks of the place it reads, or points at. One that points at a
    /// binding nothing can reassign is lent: the closure only reads it (R05.10.1).
    fn capture(&mut self, fr: &mut Fr, x: &mut Expr<Ty>) {
        // A binding whose address the closure holds is lent where nothing can reassign it, else
        // held by reference (R05.10.1); an address a capture already holds is copied as it is.
        if let EK::AddrOf(p) = &x.kind {
            let var = match p {
                Place::Local(l) => fr.vars.get(*l).copied(),
                Place::Global(g) => self.mutable.get(*g).copied(),
                _ => None,
            };
            let t = Box::new(x.ty.clone());
            if var == Some(false) {
                x.ty = Ty::Lent(t);
                x.kind = EK::Lent(p.clone());
            } else {
                x.ty = Ty::Ref(t);
            }
        } else if let EK::Copy(Place::Local(l)) = &x.kind {
            if let Some(t @ (Ty::Ref(_) | Ty::Lent(_))) = fr.locals.get(*l) {
                x.ty = t.clone();
                return;
            }
        }
        match (&x.kind, &mut x.ty) {
            (EK::AddrOf(p) | EK::Lent(p), Ty::Ref(t) | Ty::Lent(t)) => {
                if let Some(u) = self.place_ty(fr, p) {
                    **t = mark_lent(t, &u);
                }
            }
            _ => self.expr(fr, x),
        }
    }
    /// The lambda `id`, made here: its captures take the marks of what they hold, and the
    /// parameters of the fn around it its result holds stay held in it.
    fn lambda(&mut self, fr: &Fr, id: LambdaId, xs: &[Expr<Ty>]) {
        let k = FuncKey::Lambda(id);
        if self.done.contains(&id) {
            return;
        }
        let Some(f) = self.funcs.get_mut(&k) else {
            return;
        };
        let mut held = Vec::new();
        let mut lent = Vec::new();
        for (x, (c, _)) in xs.iter().zip(&f.params[..f.n_captures]) {
            if let Some(t) = f.locals.get_mut(*c) {
                if let Ty::Lent(_) = x.ty {
                    *t = x.ty.clone();
                    lent.push(*c);
                } else if let Ty::Ref(_) = x.ty {
                    *t = x.ty.clone();
                } else {
                    *t = mark_lent(t, &x.ty);
                }
            }
            if let EK::Copy(Place::Local(l)) | EK::Lent(Place::Local(l)) = x.kind {
                if fr.held.contains(&l) {
                    held.push(*c);
                }
            }
        }
        // The body reads a lent capture where it points, not through a reference.
        if !lent.is_empty() {
            visit_places(&mut f.body, &mut |p| through(p, &lent));
        }
        self.func(&k, held);
    }
    /// A struct built from `e`'s fields: a held parameter of the fn is stored lent, as is a
    /// place a pack reads, and a field filled lent makes its type argument lent too.
    fn make_struct(&self, fr: &Fr, e: &mut Expr<Ty>, pack: bool) {
        let EK::MakeStruct(xs) = &mut e.kind else {
            return;
        };
        for x in xs.iter_mut() {
            let EK::Copy(p) = &x.kind else { continue };
            let temp =
                matches!(p, Place::Local(l) if fr.names.get(*l).is_none_or(String::is_empty));
            let held = matches!(p, Place::Local(l) if fr.held.contains(l));
            if held || pack && !temp {
                x.kind = EK::Lent(p.clone());
                if pack && !matches!(x.ty, Ty::Lent(_)) {
                    x.ty = Ty::Lent(Box::new(x.ty.clone()));
                }
            }
        }
        let Ty::Struct(d, ts, ss) = &e.ty else {
            return;
        };
        let mut ts = ts.clone();
        for (x, (_, f)) in xs.iter_mut().zip(&self.d.s(*d).fields) {
            let t = match &x.kind {
                EK::Copy(p) => self.place_ty(fr, p),
                _ => None,
            };
            // A lent place stays lent: the slot points where the place points.
            if let (EK::Copy(p), Some(lent @ Ty::Lent(_))) = (&x.kind, &t) {
                x.kind = EK::Lent(p.clone());
                x.ty = lent.clone();
            }
            rebind(f, t.as_ref().unwrap_or(&x.ty), &mut ts);
        }
        e.ty = Ty::Struct(*d, ts, ss.clone());
    }
    /// The type a lowered argument has, with any lent mark it took; `None` for a path
    /// into a value, whose type is not tracked.
    fn arg_ty(&self, fr: &Fr, a: &Arg<Ty>) -> Option<Ty> {
        match a {
            Arg::Value(Expr {
                kind: EK::Pass(p, true, _),
                ..
            })
            | Arg::View(p)
            | Arg::Lend(p) => self.place_ty(fr, p),
            Arg::Value(e) => Some(e.ty.clone()),
        }
    }
    /// The type of a place, a lent mark kept where a slot on its path holds one.
    fn place_ty(&self, fr: &Fr, p: &Place) -> Option<Ty> {
        match p {
            Place::Local(l) => fr.locals.get(*l).cloned(),
            Place::Global(g) => self.globals.get(*g).cloned(),
            Place::Field(b, i) => {
                let Ty::Struct(d, ts, ss) = self.place_ty(fr, b)?.unlent().clone() else {
                    return None;
                };
                let s = self.d.s(d);
                Some(
                    Subst {
                        tys: &ts,
                        sets: &ss,
                    }
                    .ty(&s.fields.get(*i)?.1),
                )
            }
            _ => None,
        }
    }
}

/// `p` reading each capture in `lent` through its lent slot rather than a reference.
fn through(p: &mut Place, lent: &[usize]) {
    if let Place::Deref(b) = p {
        if let Place::Local(c) = **b {
            if lent.contains(&c) {
                *p = Place::Through(Box::new(Place::Local(c)));
                return;
            }
        }
    }
    match p {
        Place::Field(b, _)
        | Place::Variant(b, _)
        | Place::Deref(b)
        | Place::Through(b)
        | Place::Slot(b, _) => through(b, lent),
        Place::Local(_) | Place::Global(_) => {}
    }
}

/// A local passed whole where it is read for the last time, owned by this fn.
fn moves_in(fr: &Fr, a: &Arg<Ty>) -> bool {
    let Arg::Value(Expr {
        kind: EK::Pass(Place::Local(l), true, _),
        ..
    }) = a
    else {
        return false;
    };
    fr.last.here.contains(l)
        && !fr.params.contains(l)
        && !fr.packs.contains(l)
        && !matches!(fr.locals.get(*l), Some(Ty::Lent(_)))
}
/// Whether an argument is read where the caller has it, not made fresh for the call.
fn placed<T>(a: &Arg<T>) -> bool {
    match a {
        Arg::View(_) | Arg::Lend(_) => true,
        Arg::Value(v) => matches!(v.kind, EK::Pass(_, true, _)),
    }
}

/// `t` with the lent marks `shape` (the same type, maybe written otherwise) has.
fn mark_lent(t: &Ty, shape: &Ty) -> Ty {
    match (t, shape) {
        (Ty::Lent(_), _) => t.clone(),
        (_, Ty::Lent(s)) => Ty::Lent(Box::new(mark_lent(t, s))),
        (Ty::Struct(d, ts, ss), Ty::Struct(e, us, _)) if d == e && ts.len() == us.len() => {
            Ty::Struct(
                *d,
                ts.iter().zip(us).map(|(t, u)| mark_lent(t, u)).collect(),
                ss.clone(),
            )
        }
        _ => t.clone(),
    }
}

/// The written type: `t` without the lent marks.
fn strip_lent(t: &Ty) -> Ty {
    match t {
        Ty::Lent(e) => strip_lent(e),
        Ty::Struct(d, ts, ss) => Ty::Struct(*d, ts.iter().map(strip_lent).collect(), ss.clone()),
        _ => t.clone(),
    }
}

/// Bind type parameter `k` of `decl` to `arg`'s lent form where `arg` is `targs[k]` lent.
fn rebind(decl: &Ty, arg: &Ty, targs: &mut [Ty]) {
    match (decl, arg) {
        (Ty::Param(k), a) => {
            if let Some(t) = targs.get_mut(*k) {
                if a != t && strip_lent(a) == *t {
                    *t = a.clone();
                }
            }
        }
        (Ty::Struct(d, ts, _), Ty::Struct(e, us, _)) if d == e => {
            for (t, u) in ts.iter().zip(us) {
                rebind(t, u, targs);
            }
        }
        (Ty::Lent(t), Ty::Lent(u)) => rebind(t, u, targs),
        _ => {}
    }
}

/// Calls `f` with each call in `b`, its target and arguments.
/// Calls `f` with the type of each value `b` returns.
fn returns(b: &[Stmt<Ty>], f: &mut impl FnMut(&Ty)) {
    for s in b {
        match s {
            Stmt::Return(Some(e)) => f(&e.ty),
            Stmt::If(_, a, b) | Stmt::Loop(a, b, _) => {
                returns(a, f);
                returns(b, f);
            }
            Stmt::Switch(_, arms) => arms.iter().for_each(|a| returns(a, f)),
            Stmt::Region { body, .. } => returns(body, f),
            _ => {}
        }
    }
}

fn each_call(b: &[Stmt<Ty>], f: &mut impl FnMut(&Target<Ty>, &[Arg<Ty>])) {
    fn ex(e: &Expr<Ty>, f: &mut impl FnMut(&Target<Ty>, &[Arg<Ty>])) {
        match &e.kind {
            EK::Call(t, args, _) => {
                f(t, args);
                if let Target::Value(v) = t {
                    ex(v, f);
                }
                args.iter().for_each(|a| arg(a, f));
            }
            EK::Intr(_, args, _) => args.iter().for_each(|a| arg(a, f)),
            EK::CallValue(c, xs, _) => {
                ex(c, f);
                xs.iter().for_each(|a| arg(a, f));
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| ex(x, f)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => ex(x, f),
            _ => {}
        }
    }
    fn arg(a: &Arg<Ty>, f: &mut impl FnMut(&Target<Ty>, &[Arg<Ty>])) {
        if let Arg::Value(v) = a {
            ex(v, f);
        }
    }
    for s in b {
        match s {
            Stmt::Let(_, e)
            | Stmt::Assign(_, e)
            | Stmt::Expr(e)
            | Stmt::Yield(e)
            | Stmt::Fact(e)
            | Stmt::Check { cond: e, .. } => ex(e, f),
            Stmt::Return(e) => e.iter().for_each(|e| ex(e, f)),
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: es, .. }) => {
                es.iter().for_each(|e| ex(e, f))
            }
            Stmt::If(c, a, b) => {
                ex(c, f);
                each_call(a, f);
                each_call(b, f);
            }
            Stmt::Loop(a, b, _) => {
                each_call(a, f);
                each_call(b, f);
            }
            Stmt::Switch(c, arms) => {
                ex(c, f);
                arms.iter().for_each(|a| each_call(a, f));
            }
            Stmt::Region {
                target, args, body, ..
            } => {
                f(target, args);
                args.iter().for_each(|a| arg(a, f));
                each_call(body, f);
            }
            Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
        }
    }
}
