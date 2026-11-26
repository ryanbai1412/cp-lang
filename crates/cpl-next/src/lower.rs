//! Contract 4: typed tree -> generic IR. No inference, no solver, no lookup:
//! everything it needs is on the typed tree.
use crate::check::*;
use crate::decls::{Decls, SelfMode};
use crate::diag::Msg;
use crate::ir::*;
use crate::own::Owned;
use crate::ty::*;
use cpl_syntax::ast::{BinOp, CmpOp};
use std::collections::{BTreeSet, HashMap};

pub struct Generic {
    pub funcs: HashMap<FuncKey, Func<Ty>>,
    pub globals: Vec<Ty>,
    pub errors: Vec<Msg>,
}

struct Lw<'a> {
    d: &'a Decls,
    typed: &'a Typed,
    owned: &'a Owned,
    /// The current fn's parameters its result holds (R05.5.3): a struct literal stores them lent.
    lent: Vec<usize>,
    /// The globals' types, a lent mark added where one is bound to a value that lends.
    globals: Vec<Ty>,
    locals: Vec<Ty>,
    /// Which of the current fn's own bindings are `var`s.
    mut_locals: Vec<bool>,
    /// The `var`s among them: a closure writes these through its capture.
    var_locals: Vec<bool>,
    /// Inside a lambda at top level, its params live in the globals table; map them to locals.
    remap: HashMap<VarRef, usize>,
    lambdas: Vec<Func<Ty>>,
    /// The lambdas being lowered, each with its own value as its body makes it: its captures.
    selves: Vec<(LambdaId, Vec<Expr<Ty>>)>,
    errors: Vec<Msg>,
    in_main: bool,
    /// The code of one test case, once main's `Cases` is lowered.
    case: Option<Func<Ty>>,
    /// Where the item being lowered starts: spans are relative to it.
    off: u32,
    /// Where each line of the source starts, and how many of them are std.
    line_starts: Vec<u32>,
    std_lines: u32,
}

pub fn lower(d: &Decls, typed: &Typed, owned: &Owned, src: &str, std_lines: u32) -> Generic {
    let mut out = Generic {
        funcs: HashMap::new(),
        globals: Vec::new(),
        errors: Vec::new(),
    };
    let mut lw = Lw {
        d,
        typed,
        owned,
        lent: Vec::new(),
        globals: typed.globals.iter().map(|g| g.ty.clone()).collect(),
        locals: Vec::new(),
        mut_locals: Vec::new(),
        var_locals: Vec::new(),
        remap: HashMap::new(),
        lambdas: Vec::new(),
        selves: Vec::new(),
        errors: Vec::new(),
        in_main: true,
        case: None,
        off: 0,
        line_starts: std::iter::once(0)
            .chain(src.match_indices('\n').map(|(i, _)| i as u32 + 1))
            .collect(),
        std_lines,
    };
    let mut body = Vec::new();
    for s in &typed.main {
        lw.stmt(s, &mut body);
    }
    out.funcs.insert(
        FuncKey::Main,
        Func {
            key: FuncKey::Main,
            name: "main".into(),
            params: vec![],
            n_captures: 0,
            locals: std::mem::take(&mut lw.locals),
            body,
            ret: Ty::Unit,
            region: None,
            line: 0,
        },
    );
    if let Some(c) = lw.case.take() {
        out.funcs.insert(FuncKey::Case, c);
    }
    lw.in_main = false;
    for (id, tf) in &typed.fns {
        let fd = d.f(*id);
        lw.off = fd.item;
        lw.locals = tf.locals.iter().map(|l| l.ty.clone()).collect();
        // R05.10.1: a closure shares `var`s, local heap values and heap parameters with the fn around it.
        lw.mut_locals = tf
            .locals
            .iter()
            .map(|l| l.mutable || !crate::own::plain(d, &l.ty, 0))
            .collect();
        lw.var_locals = tf.locals.iter().map(|l| l.mutable).collect();
        lw.remap.clear();
        let holds = owned.lends.get(id);
        lw.lent = (lw.param_locals(*id).into_iter())
            .enumerate()
            .filter(|(i, _)| holds.is_some_and(|h| h.contains(i)))
            .map(|(_, l)| l)
            .collect();
        let mut params = Vec::new();
        if let Some(s) = tf.self_local {
            params.push((s, fd.self_mode == SelfMode::Mut || d.writes_cache(*id)));
        }
        for (p, pd) in tf.params.iter().zip(&fd.params) {
            params.push((*p, pd.mutable));
        }
        for (l, m) in &params {
            if *m {
                lw.var_locals[*l] = true;
            }
        }
        let mut body = Vec::new();
        let v = lw.expr(&tf.body, &mut body);
        // A body that ends in `return` has no value of its own. (The design wants a "never" type here.)
        if matches!(body.last(), Some(Stmt::Return(_))) {
        } else if tf.ret != Ty::Unit {
            body.push(Stmt::Return(Some(v)));
        } else {
            lw.effect(v, &mut body);
        }
        out.funcs.insert(
            FuncKey::Decl(*id),
            Func {
                key: FuncKey::Decl(*id),
                name: fd.name.clone(),
                params,
                n_captures: 0,
                locals: std::mem::take(&mut lw.locals),
                body,
                ret: tf.ret.clone(),
                region: fd.region.clone(),
                line: 0,
            },
        );
    }
    for f in lw.lambdas.drain(..) {
        out.funcs.insert(f.key.clone(), f);
    }
    out.errors = lw.errors;
    out.globals = lw.globals;
    out
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

/// The written type: `t` without the lent marks lowering adds.
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

impl Lw<'_> {
    /// The locals of fn `f`'s parameters, receiver first, as call arguments come.
    fn param_locals(&self, f: FnId) -> Vec<usize> {
        let Some(tf) = self.typed.fns.get(&f) else {
            return Vec::new();
        };
        (self.d.f(f).self_mode != SelfMode::None)
            .then_some(tf.self_local)
            .flatten()
            .into_iter()
            .chain(tf.params.iter().copied())
            .collect()
    }
    /// The type a lowered argument has, with any lent mark it took; `None` for a path
    /// into a value, whose type lowering does not track.
    fn arg_ty(&self, a: &Arg<Ty>) -> Option<Ty> {
        match a {
            Arg::Value(e) => Some(e.ty.clone()),
            Arg::View(p) | Arg::Lend(p) => self.place_ty(p),
        }
    }
    /// The type of a place, a lent mark kept where a slot on its path holds one.
    fn place_ty(&self, p: &Place) -> Option<Ty> {
        match p {
            Place::Local(l) => self.locals.get(*l).cloned(),
            Place::Global(g) => self.globals.get(*g).cloned(),
            Place::Field(b, i) => {
                let b = match self.place_ty(b)? {
                    Ty::Lent(t) => *t,
                    t => t,
                };
                let Ty::Struct(d, ts, ss) = &b else {
                    return None;
                };
                let s = self.d.s(*d);
                Some(Subst { tys: ts, sets: ss }.ty(&s.fields.get(*i)?.1))
            }
            _ => None,
        }
    }
    /// The receiver type a bound's method is picked by: lent when the lowered receiver is.
    fn self_ty(&self, recv: &TArg, a: &[Arg<Ty>]) -> Ty {
        let (TArg::Value(v) | TArg::Mut(v)) = recv;
        match a.first().and_then(|x| self.arg_ty(x)) {
            Some(t) if strip_lent(&t) == v.ty => t,
            _ => v.ty.clone(),
        }
    }
    /// `f` with each type argument an argument fills lent taking the lent form, so the
    /// instance reads the slot where it points (one instance per mode).
    fn lent_targs(&self, f: &FnRef, args: &[Arg<Ty>]) -> FnRef {
        let Some(tf) = self.typed.fns.get(&f.decl) else {
            return f.clone();
        };
        let ps = self.param_locals(f.decl);
        let holds = self.owned.lends.get(&f.decl);
        let mut targs = f.targs.clone();
        for (i, (&l, a)) in ps.iter().zip(args).enumerate() {
            let decl = &tf.locals[l].ty;
            if let Some(t) = self.arg_ty(a) {
                rebind(decl, &t, &mut targs);
            }
            match (decl, a) {
                // A held argument that lives in a place is lent; a fresh one is moved in.
                (Ty::Param(k), Arg::View(_) | Arg::Lend(_))
                    if holds.is_some_and(|h| h.contains(&i)) =>
                {
                    if let Some(t) = targs.get_mut(*k).filter(|t| !matches!(t, Ty::Lent(_))) {
                        *t = Ty::Lent(Box::new(t.clone()));
                    }
                }
                _ => {}
            }
        }
        FnRef {
            decl: f.decl,
            targs,
            sets: f.sets.clone(),
        }
    }
}

fn pure<T>(e: &Expr<T>) -> bool {
    match &e.kind {
        EK::Int(_)
        | EK::Bool(_)
        | EK::Text(_)
        | EK::Unit
        | EK::Copy(_)
        | EK::AddrOf(_)
        | EK::Lent(_)
        | EK::Tag(_)
        | EK::DomLen(_)
        | EK::DomLo(_) => true,
        EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().all(pure),
        EK::Inject(x) | EK::MakeUnion(_, x) => pure(x),
        // Widening is a branch once sets are known, so it is a statement of its own.
        EK::Widen(_) => false,
        EK::Intr(i, args, _) => {
            !i.checked() && args.iter().all(|a| matches!(a, Arg::Value(v) if pure(v)))
        }
        EK::Call(..) | EK::CallValue(..) => false,
    }
}
fn writes_nothing<T>(s: &Stmt<T>) -> bool {
    fn quiet<T>(e: &Expr<T>) -> bool {
        match &e.kind {
            EK::Intr(i, args, _) => {
                !matches!(i, Intr::RawSet | Intr::ReadInt)
                    && args.iter().all(|a| matches!(a, Arg::Value(v) if quiet(v)))
            }
            _ => pure(e),
        }
    }
    match s {
        Stmt::Let(_, e) => quiet(e),
        Stmt::Check { .. } => true,
        _ => false,
    }
}
fn constant<T>(e: &Expr<T>) -> bool {
    matches!(e.kind, EK::Int(_) | EK::Bool(_) | EK::Unit)
}

impl Lw<'_> {
    fn site(&self, span: cpl_diag::Span) -> Site {
        let line = self
            .line_starts
            .partition_point(|s| *s <= self.off + span.start) as u32;
        line.saturating_sub(self.std_lines)
    }
    fn temp(&mut self, ty: Ty) -> usize {
        self.locals.push(ty);
        self.locals.len() - 1
    }
    fn var(&mut self, v: VarRef) -> Place {
        if let Some(l) = self.remap.get(&v) {
            match self.locals[*l] {
                Ty::Ref(_) => return Place::Deref(Box::new(Place::Local(*l))),
                Ty::Lent(_) => return Place::Through(Box::new(Place::Local(*l))),
                _ => {}
            }
            return Place::Local(*l);
        }
        match v {
            VarRef::Local(i) => Place::Local(i),
            VarRef::Global(i) => Place::Global(i),
        }
    }
    /// Binds a new variable. At top level a block-local or lambda parameter becomes a local of main.
    fn bind(&mut self, v: VarRef) -> Place {
        self.var(v)
    }
    /// Makes an impure expression a statement of its own and returns a read of its result.
    fn spill(&mut self, e: Expr<Ty>, out: &mut Vec<Stmt<Ty>>) -> Expr<Ty> {
        if pure(&e) {
            return e;
        }
        let t = self.temp(e.ty.clone());
        let ty = e.ty.clone();
        out.push(Stmt::Let(t, e));
        Expr {
            kind: EK::Copy(Place::Local(t)),
            ty,
        }
    }
    fn effect(&mut self, e: Expr<Ty>, out: &mut Vec<Stmt<Ty>>) {
        if !pure(&e) {
            out.push(Stmt::Expr(e));
        }
    }
    /// Lowers operands left to right. If a later one needs statements, earlier
    /// non-constant ones are saved first, so they are read before those statements run.
    fn seq(&mut self, es: &[&TExpr], out: &mut Vec<Stmt<Ty>>) -> Vec<Expr<Ty>> {
        let mut done: Vec<Expr<Ty>> = Vec::new();
        for e in es {
            let before = out.len();
            let mut tail = Vec::new();
            let v = self.expr(e, &mut tail);
            let v = self.spill(v, &mut tail);
            // Statements that only compute (arithmetic, a check) cannot write what an
            // earlier operand reads, so that operand stays a read of its place and is
            // not copied. A call can, so the operand is saved.
            if !tail.iter().all(writes_nothing) {
                for d in done.iter_mut() {
                    if !constant(d) {
                        let t = self.temp(d.ty.clone());
                        let ty = d.ty.clone();
                        let old = std::mem::replace(
                            d,
                            Expr {
                                kind: EK::Copy(Place::Local(t)),
                                ty,
                            },
                        );
                        out.push(Stmt::Let(t, old));
                    }
                }
            }
            let _ = before;
            out.extend(tail);
            done.push(v);
        }
        done
    }
    fn place(&mut self, e: &TExpr, out: &mut Vec<Stmt<Ty>>) -> Place {
        match &e.kind {
            TK::Var(v) => self.var(*v),
            TK::Field(b, i) => Place::Field(Box::new(self.place(b, out)), *i),
            TK::Payload(b, i) => Place::Variant(Box::new(self.place(b, out)), *i),
            TK::Slot(b, i) => {
                let iv = self.expr(i, out);
                let t = self.temp(Ty::Int);
                out.push(Stmt::Let(t, iv));
                Place::Slot(Box::new(self.place(b, out)), t)
            }
            _ => {
                let v = self.expr(e, out);
                let t = self.temp(v.ty.clone());
                out.push(Stmt::Let(t, v));
                Place::Local(t)
            }
        }
    }
    /// The copy ownership asked for at this argument is a statement of its own, so no later
    /// stage can turn it back into a view of the place.
    fn copied(&mut self, arg: &TExpr, x: &mut Expr<Ty>, out: &mut Vec<Stmt<Ty>>) {
        if self.owned.copies.contains(&(self.off + arg.span.start)) {
            let t = self.temp(x.ty.clone());
            let copy = Expr {
                kind: EK::Copy(Place::Local(t)),
                ty: x.ty.clone(),
            };
            out.push(Stmt::Let(t, std::mem::replace(x, copy)));
        }
    }
    fn args(
        &mut self,
        recv: Option<&TArg>,
        args: &[TArg],
        out: &mut Vec<Stmt<Ty>>,
    ) -> Vec<Arg<Ty>> {
        let all: Vec<&TArg> = recv.into_iter().chain(args.iter()).collect();
        // R11: a heap argument that is a place is viewed where it is, from when the call
        // starts, unless ownership found the call also writes it: then it is copied in its
        // turn. A type parameter could be plain or heap and is passed as a value.
        let viewed = |lw: &Self, v: &TExpr| {
            matches!(v.kind, TK::Var(_) | TK::Field(..) | TK::Payload(..))
                && !matches!(v.ty, Ty::Param(_))
                && !crate::own::plain(lw.d, &v.ty, 0)
                && !lw.owned.copies.contains(&(lw.off + v.span.start))
        };
        let values: Vec<&TExpr> = all
            .iter()
            .filter_map(|a| match a {
                TArg::Value(v) if !viewed(self, v) => Some(v),
                _ => None,
            })
            .collect();
        let mut vals = self.seq(&values, out).into_iter();
        let mut lowered = Vec::with_capacity(all.len());
        for a in all {
            match a {
                TArg::Value(v) if viewed(self, v) => lowered.push(Arg::View(self.place(v, out))),
                TArg::Value(v) => {
                    let Some(mut x) = vals.next() else { continue };
                    self.copied(v, &mut x, out);
                    lowered.push(Arg::Value(x));
                }
                TArg::Mut(p) => lowered.push(Arg::Lend(self.place(p, out))),
            }
        }
        lowered
    }
    fn expr(&mut self, e: &TExpr, out: &mut Vec<Stmt<Ty>>) -> Expr<Ty> {
        let ty = e.ty.clone();
        let mk = |kind| Expr {
            kind,
            ty: ty.clone(),
        };
        let site = self.site(e.span);
        match &e.kind {
            TK::Int(i) => mk(EK::Int(*i)),
            TK::Bool(b) => mk(EK::Bool(*b)),
            TK::Unit | TK::Error => mk(EK::Unit),
            TK::Text(t) => mk(EK::Text(t.clone())),
            TK::Unset => mk(EK::Intr(Intr::Unset, vec![], 0)),
            TK::Var(_) | TK::Field(..) | TK::Payload(..) | TK::Slot(..) => {
                let p = self.place(e, out);
                mk(EK::Copy(p))
            }
            TK::Variant(i, x) => {
                let v = self.expr(x, out);
                mk(EK::MakeUnion(*i, Box::new(v)))
            }
            TK::Tag(x) => {
                let p = self.place(x, out);
                mk(EK::Tag(p))
            }
            TK::Call { f, recv, args } => {
                let fd = self.d.f(f.decl);
                let a = self.args(recv.as_deref(), args, out);
                let f = &self.lent_targs(f, &a);
                let call = if let Some(n) = &fd.intrinsic {
                    let op = Intr::named(n).unwrap_or_else(|| {
                        self.errors.push(format!("slice: intrinsic `{n}`").into());
                        Intr::RawGet
                    });
                    mk(EK::Intr(op, a, site))
                } else {
                    let ty = match self.typed.fns.get(&f.decl) {
                        Some(tf) => Subst {
                            tys: &f.targs,
                            sets: &f.sets,
                        }
                        .ty(&tf.ret),
                        None => e.ty.clone(),
                    };
                    // A result held directly is its own storage, not a lent slot.
                    let ty = match mark_lent(&e.ty, &ty) {
                        Ty::Lent(t) => *t,
                        t => t,
                    };
                    Expr {
                        kind: EK::Call(Target::Fn(f.clone()), a, site),
                        ty,
                    }
                };
                self.spill(call, out)
            }
            TK::CallTrait {
                tr,
                method,
                recv,
                args,
            } => {
                let a = self.args(Some(recv), args, out);
                let self_ty = self.self_ty(recv, &a);
                let call = mk(EK::Call(
                    Target::Trait {
                        tr: *tr,
                        method: method.clone(),
                        self_ty,
                    },
                    a,
                    site,
                ));
                self.spill(call, out)
            }
            TK::CallValue { callee, args } => {
                let mut all: Vec<&TExpr> = vec![callee];
                all.extend(args.iter());
                let mut vs = self.seq(&all, out);
                for (v, t) in vs.iter_mut().zip(&all) {
                    self.copied(t, v, out);
                }
                let c = vs.remove(0);
                let call = mk(EK::CallValue(Box::new(c), vs, site));
                self.spill(call, out)
            }
            TK::Struct(fs) => {
                // A held parameter is stored as a pointer to its place, never copied first.
                let held: Vec<bool> = (fs.iter())
                    .map(|f| matches!(f.kind, TK::Var(VarRef::Local(l)) if self.lent.contains(&l)))
                    .collect();
                let rest: Vec<&TExpr> = (fs.iter().zip(&held))
                    .filter(|(_, h)| !**h)
                    .map(|(f, _)| f)
                    .collect();
                let mut rest = self.seq(&rest, out).into_iter();
                let mut xs = Vec::new();
                for (f, h) in fs.iter().zip(held) {
                    match (h, rest.next()) {
                        (false, Some(x)) => xs.push(x),
                        (_, x) => {
                            rest = x.into_iter().chain(rest).collect::<Vec<_>>().into_iter();
                            let p = self.place(f, out);
                            xs.push(Expr {
                                kind: EK::Lent(p),
                                ty: f.ty.clone(),
                            });
                        }
                    }
                }
                mk(EK::MakeStruct(xs))
            }
            TK::FnItem(_) => mk(EK::MakeClosure(vec![])),
            TK::Itself(id) => match self.selves.last() {
                Some((me, caps)) if me == id => mk(EK::MakeClosure(caps.clone())),
                _ => {
                    (self.errors).push("slice: a lambda that names the lambda around it".into());
                    mk(EK::MakeClosure(vec![]))
                }
            },
            TK::Lambda { id, params, body } => {
                let captures = self.lambda(*id, params, body);
                mk(EK::MakeClosure(captures))
            }
            TK::Inject(x) => {
                let v = self.expr(x, out);
                mk(EK::Inject(Box::new(v)))
            }
            TK::Elem(x) => self.expr(x, out),
            TK::Widen(x) => {
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                mk(EK::Widen(Box::new(v)))
            }
            TK::Bin(op, l, r) if matches!(op, BinOp::And | BinOp::Or) => {
                // Lazy: the right side runs only when the left does not decide.
                let t = self.temp(Ty::Bool);
                let lv = self.expr(l, out);
                out.push(Stmt::Let(t, lv));
                let mut rs = Vec::new();
                let rv = self.expr(r, &mut rs);
                rs.push(Stmt::Assign(Place::Local(t), rv));
                let c = Expr {
                    kind: EK::Copy(Place::Local(t)),
                    ty: Ty::Bool,
                };
                if *op == BinOp::And {
                    out.push(Stmt::If(c.clone(), rs, vec![]));
                } else {
                    out.push(Stmt::If(c.clone(), vec![], rs));
                }
                c
            }
            TK::Bin(op, l, r) => {
                let vs = self.seq(&[l, r], out);
                let i = match op {
                    BinOp::Add => Intr::Add,
                    BinOp::Sub => Intr::Sub,
                    BinOp::Mul => Intr::Mul,
                    BinOp::Div => Intr::Div,
                    BinOp::Rem => Intr::Rem,
                    BinOp::BitAnd => Intr::BitAnd,
                    BinOp::BitOr => Intr::BitOr,
                    BinOp::BitXor => Intr::BitXor,
                    BinOp::Pow => Intr::Pow,
                    BinOp::Shl => Intr::Shl,
                    BinOp::Shr => Intr::Shr,
                    _ => {
                        self.errors.push("slice: unsupported operator".into());
                        Intr::Add
                    }
                };
                let v = mk(EK::Intr(i, vs.into_iter().map(Arg::Value).collect(), site));
                self.spill(v, out)
            }
            TK::Cmp(op, l, r) => {
                let vs = self.seq(&[l, r], out);
                let i = match op {
                    CmpOp::Eq => Intr::Eq,
                    CmpOp::Ne => Intr::Ne,
                    CmpOp::Lt => Intr::Lt,
                    CmpOp::Le => Intr::Le,
                    CmpOp::Gt => Intr::Gt,
                    _ => Intr::Ge,
                };
                mk(EK::Intr(i, vs.into_iter().map(Arg::Value).collect(), site))
            }
            TK::CharOf(x) => {
                let v = self.expr(x, out);
                mk(EK::Intr(Intr::CharOf, vec![Arg::Value(v)], site))
            }
            TK::CodeOf(x) => {
                let v = self.expr(x, out);
                mk(EK::Intr(Intr::CodeOf, vec![Arg::Value(v)], site))
            }
            TK::ToU64(x) => {
                let v = self.expr(x, out);
                mk(EK::Intr(Intr::ToU64, vec![Arg::Value(v)], site))
            }
            TK::FromU64(x) => {
                let v = self.expr(x, out);
                let v = mk(EK::Intr(Intr::FromU64, vec![Arg::Value(v)], site));
                self.spill(v, out)
            }
            TK::Not(x) => {
                let v = self.expr(x, out);
                mk(EK::Intr(Intr::Not, vec![Arg::Value(v)], site))
            }
            TK::Neg(x) => {
                let v = self.expr(x, out);
                let v = mk(EK::Intr(Intr::Neg, vec![Arg::Value(v)], site));
                self.spill(v, out)
            }
            TK::If(c, a, b) => {
                let cv = self.expr(c, out);
                let cv = self.spill(cv, out);
                let (mut sa, mut sb) = (Vec::new(), Vec::new());
                let va = self.expr(a, &mut sa);
                let vb = self.expr(b, &mut sb);
                if ty == Ty::Unit {
                    self.effect(va, &mut sa);
                    self.effect(vb, &mut sb);
                    out.push(Stmt::If(cv, sa, sb));
                    mk(EK::Unit)
                } else {
                    let t = self.temp(ty.clone());
                    sa.push(Stmt::Assign(Place::Local(t), va));
                    sb.push(Stmt::Assign(Place::Local(t), vb));
                    out.push(Stmt::If(cv, sa, sb));
                    mk(EK::Copy(Place::Local(t)))
                }
            }
            TK::Block(ss, v) => {
                for s in ss {
                    self.stmt(s, out);
                }
                match v {
                    Some(v) => self.expr(v, out),
                    None => mk(EK::Unit),
                }
            }
            TK::Print(ps) => {
                let vals: Vec<&TExpr> = ps
                    .iter()
                    .filter_map(|p| {
                        if let Piece::Value(v) = p {
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .collect();
                // Checked here and not in the checker: a variable's type may be settled only at the end of the file.
                if vals
                    .iter()
                    .any(|v| !matches!(v.ty, Ty::Int | Ty::Bool | Ty::Char | Ty::U64 | Ty::Error))
                {
                    self.errors.push(
                        format!(
                            "{}: this value cannot be printed",
                            cpl_diag::codes::MISSING_IMPL.0
                        )
                        .into(),
                    );
                }
                let mut vs = self.seq(&vals, out).into_iter();
                let pieces = ps
                    .iter()
                    .filter_map(|p| match p {
                        Piece::Text(t) => Some(PrintPiece::Text(t.clone())),
                        Piece::Value(_) => vs.next().map(PrintPiece::Value),
                    })
                    .collect();
                out.push(Stmt::Print(pieces));
                mk(EK::Unit)
            }
            TK::DomLen(t) => mk(EK::DomLen(t.clone())),
            TK::Ordinal(x) => {
                let v = self.expr(x, out);
                Expr {
                    kind: v.kind,
                    ty: Ty::Int,
                }
            }
            // A position is held as its ordinal; its integer is that plus the domain's start.
            TK::DomOrd(x) => {
                let v = self.expr(x, out);
                let int = |kind| Expr { kind, ty: Ty::Int };
                if matches!(x.ty, Ty::Int | Ty::Error) {
                    return int(v.kind);
                }
                let lo = Arg::Value(int(EK::DomLo(x.ty.clone())));
                let sum = int(EK::Intr(Intr::Add, vec![Arg::Value(int(v.kind)), lo], 0));
                self.spill(sum, out)
            }
            TK::DomFrom(t, x) => {
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                let int = |kind| Expr { kind, ty: Ty::Int };
                let op = |op, a: Expr<Ty>, b: Expr<Ty>, ty: Ty| Expr {
                    kind: EK::Intr(op, vec![Arg::Value(a), Arg::Value(b)], 0),
                    ty,
                };
                let check = |cond: Expr<Ty>, out: &mut Vec<Stmt<Ty>>| {
                    out.push(Stmt::Check {
                        kind: CheckKind::Domain,
                        cond,
                        msg: "value outside the domain".into(),
                        site,
                    })
                };
                let (lo, len) = (int(EK::DomLo(t.clone())), int(EK::DomLen(t.clone())));
                // With a position in the domain and `v` at or after the first, none of
                // the arithmetic below can overflow.
                check(op(Intr::Lt, int(EK::Int(0)), len.clone(), Ty::Bool), out);
                check(op(Intr::Ge, v.clone(), lo.clone(), Ty::Bool), out);
                let last = op(Intr::Sub, len, int(EK::Int(1)), Ty::Int);
                let last = self.spill(last, out);
                let last = op(Intr::Add, lo.clone(), last, Ty::Int);
                let last = self.spill(last, out);
                check(op(Intr::Le, v.clone(), last, Ty::Bool), out);
                let ord = op(Intr::Sub, v, lo, Ty::Int);
                let ord = self.spill(ord, out);
                Expr { kind: ord.kind, ty }
            }
            TK::DomHas(t, x) => {
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                let int = |kind| Expr { kind, ty: Ty::Int };
                let op = |op, a: Expr<Ty>, b: Expr<Ty>, ty: Ty| Expr {
                    kind: EK::Intr(op, vec![Arg::Value(a), Arg::Value(b)], 0),
                    ty,
                };
                let (lo, len) = (int(EK::DomLo(t.clone())), int(EK::DomLen(t.clone())));
                // `v - lo` is read only once `v` is at or after the first, so it cannot overflow.
                let has = self.temp(Ty::Bool);
                out.push(Stmt::Let(
                    has,
                    op(Intr::Ge, v.clone(), lo.clone(), Ty::Bool),
                ));
                let ord = op(Intr::Sub, v, lo, Ty::Int);
                let within = op(Intr::Below, ord, len, Ty::Bool);
                let read = Expr {
                    kind: EK::Copy(Place::Local(has)),
                    ty: Ty::Bool,
                };
                let then = vec![Stmt::Assign(Place::Local(has), within)];
                out.push(Stmt::If(read.clone(), then, vec![]));
                read
            }
            TK::DomNth(t, x) => {
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                let len = Expr {
                    kind: EK::DomLen(t.clone()),
                    ty: Ty::Int,
                };
                let cond = Expr {
                    kind: EK::Intr(Intr::Below, vec![Arg::Value(v.clone()), Arg::Value(len)], 0),
                    ty: Ty::Bool,
                };
                out.push(Stmt::Check {
                    kind: CheckKind::Domain,
                    cond,
                    msg: "a position past the domain's end".into(),
                    site,
                });
                Expr { kind: v.kind, ty }
            }
            TK::ReadInt => {
                let v = mk(EK::Intr(Intr::ReadInt, vec![], site));
                self.spill(v, out)
            }
        }
    }
    fn stmt(&mut self, s: &TStmt, out: &mut Vec<Stmt<Ty>>) {
        match s {
            TStmt::Let(v, e) => {
                let val = self.expr(e, out);
                match self.bind(*v) {
                    Place::Local(l) => {
                        if val.ty != self.locals[l] && strip_lent(&val.ty) == self.locals[l] {
                            self.locals[l] = val.ty.clone();
                        }
                        out.push(Stmt::Let(l, val))
                    }
                    p => {
                        if let (Place::Global(g), VarRef::Global(_)) = (&p, v) {
                            let g = *g;
                            if val.ty != self.globals[g] && strip_lent(&val.ty) == self.globals[g] {
                                self.globals[g] = val.ty.clone();
                            }
                        }
                        out.push(Stmt::Assign(p, val))
                    }
                }
            }
            TStmt::Assign(p, e) => {
                let mut val = self.expr(e, out);
                self.copied(e, &mut val, out);
                let val = self.spill(val, out);
                let place = self.place(p, out);
                out.push(Stmt::Assign(place, val));
            }
            TStmt::Expr(e) => {
                let v = self.expr(e, out);
                self.effect(v, out);
            }
            TStmt::While(c, b) => {
                let mut body = Vec::new();
                let cv = self.expr(c, &mut body);
                body.push(Stmt::If(cv, vec![], vec![Stmt::Break]));
                let v = self.expr(b, &mut body);
                self.effect(v, &mut body);
                out.push(Stmt::Loop(body, vec![]));
            }
            TStmt::For(v, lo, hi, b) => {
                // `for i in lo..hi`: both ends are evaluated once, before the loop.
                let lov = self.expr(lo, out);
                let var = self.bind(*v);
                out.push(Stmt::Assign(var.clone(), lov));
                let hiv = self.expr(hi, out);
                let h = self.temp(Ty::Int);
                out.push(Stmt::Let(h, hiv));
                let read = |p: Place| Expr {
                    kind: EK::Copy(p),
                    ty: Ty::Int,
                };
                let cond = Expr {
                    kind: EK::Intr(
                        Intr::Lt,
                        vec![
                            Arg::Value(read(var.clone())),
                            Arg::Value(read(Place::Local(h))),
                        ],
                        0,
                    ),
                    ty: Ty::Bool,
                };
                let mut body = vec![Stmt::If(cond, vec![], vec![Stmt::Break])];
                let bv = self.expr(b, &mut body);
                self.effect(bv, &mut body);
                // The step cannot overflow: the variable is below `hi`.
                let step = Expr {
                    kind: EK::Intr(
                        Intr::Add,
                        vec![
                            Arg::Value(read(var.clone())),
                            Arg::Value(Expr {
                                kind: EK::Int(1),
                                ty: Ty::Int,
                            }),
                        ],
                        0,
                    ),
                    ty: Ty::Int,
                };
                out.push(Stmt::Loop(body, vec![Stmt::Assign(var, step)]));
            }
            TStmt::Return(e) => {
                let v = e.as_ref().map(|e| {
                    let v = self.expr(e, out);
                    self.spill(v, out)
                });
                out.push(Stmt::Return(v));
            }
            TStmt::Assert(c, msg, span, kind) => {
                let cv = self.expr(c, out);
                let cv = self.spill(cv, out);
                out.push(Stmt::Check {
                    kind: *kind,
                    cond: cv,
                    msg: msg.clone(),
                    site: self.site(*span),
                });
            }
            TStmt::Break => out.push(Stmt::Break),
            TStmt::Item(off) => self.off = *off,
            TStmt::Cases(count, body) => {
                let mk = |kind, ty| Expr { kind, ty };
                let call = Stmt::Expr(mk(EK::Call(Target::Case, vec![], 0), Ty::Unit));
                match count {
                    // The count is read once; the cases left are counted down.
                    Some(c) => {
                        let c = self.expr(c, out);
                        let left = self.temp(Ty::Int);
                        out.push(Stmt::Let(left, c));
                        let read = || mk(EK::Copy(Place::Local(left)), Ty::Int);
                        let int = |i| mk(EK::Int(i), Ty::Int);
                        let more = mk(
                            EK::Intr(Intr::Lt, vec![Arg::Value(int(0)), Arg::Value(read())], 0),
                            Ty::Bool,
                        );
                        // Cannot overflow: `left` is above 0.
                        let step = mk(
                            EK::Intr(Intr::Sub, vec![Arg::Value(read()), Arg::Value(int(1))], 0),
                            Ty::Int,
                        );
                        out.push(Stmt::Loop(
                            vec![Stmt::If(more, vec![], vec![Stmt::Break]), call],
                            vec![Stmt::Assign(Place::Local(left), step)],
                        ));
                    }
                    None => {
                        let more = mk(EK::Intr(Intr::InputLeft, vec![], 0), Ty::Bool);
                        out.push(Stmt::Loop(
                            vec![Stmt::If(more, vec![], vec![Stmt::Break]), call],
                            vec![],
                        ));
                    }
                }
                let (outer, off) = (std::mem::take(&mut self.locals), self.off);
                let mut b = Vec::new();
                body.iter().for_each(|s| self.stmt(s, &mut b));
                self.case = Some(Func {
                    key: FuncKey::Case,
                    name: "case".into(),
                    params: vec![],
                    n_captures: 0,
                    locals: std::mem::replace(&mut self.locals, outer),
                    body: b,
                    ret: Ty::Unit,
                    region: None,
                    line: 0,
                });
                self.off = off;
            }
            TStmt::Continue => out.push(Stmt::Continue),
            TStmt::Yield(e) => {
                let v = self.expr(e, out);
                let v = self.spill(v, out);
                out.push(Stmt::Yield(v));
            }
            TStmt::LendOut(p) => {
                let place = self.place(p, out);
                out.push(Stmt::LendOut(place));
            }
            TStmt::Region {
                callee,
                recv,
                args,
                var,
                lend,
                body,
            } => {
                // A `for` is at the line of what it runs over.
                let first = recv.as_deref().or(args.first());
                let site = match first {
                    Some(TArg::Value(e) | TArg::Mut(e)) => self.site(e.span),
                    None => self.site(body.span),
                };
                let a = self.args(recv.as_deref(), args, out);
                // The bound variable is always a local of this fn: for a lend it stands for the callee's place.
                let ty = match var {
                    VarRef::Local(l) if !self.in_main && !self.remap.contains_key(var) => {
                        self.locals[*l].clone()
                    }
                    VarRef::Local(l) => self.locals.get(*l).cloned().unwrap_or(Ty::Error),
                    VarRef::Global(g) => self.globals[*g].clone(),
                };
                let l = match var {
                    VarRef::Local(l) if !self.in_main && !self.remap.contains_key(var) => *l,
                    _ => {
                        let l = self.temp(ty);
                        self.remap.insert(*var, l);
                        l
                    }
                };
                let mut b = Vec::new();
                let v = self.expr(body, &mut b);
                self.effect(v, &mut b);
                let target = match (callee, recv.as_deref()) {
                    (Callee::Fn(f), _) => Target::Fn(self.lent_targs(f, &a)),
                    (Callee::Trait(tr, method), r) => Target::Trait {
                        tr: *tr,
                        method: method.clone(),
                        self_ty: r.map_or(Ty::Error, |r| self.self_ty(r, &a)),
                    },
                };
                out.push(Stmt::Region {
                    target,
                    args: a,
                    var: l,
                    lend: *lend,
                    body: b,
                    site,
                });
            }
        }
    }
    /// Lifts a lambda to a function of its own. Returns the locals it captures, by value.
    fn lambda(&mut self, id: LambdaId, params: &[usize], body: &TExpr) -> Vec<Expr<Ty>> {
        let info = &self.typed.lambdas[id.0];
        let mut used = BTreeSet::new();
        let mut bound: BTreeSet<VarRef> = BTreeSet::new();
        free_vars(body, &mut used, &mut bound);
        let saved_remap = self.remap.clone();
        let param_refs: Vec<VarRef> = params
            .iter()
            .map(|p| {
                if self.in_main {
                    VarRef::Global(*p)
                } else {
                    VarRef::Local(*p)
                }
            })
            .collect();
        // What the lambda reads from the fn around it: locals not bound inside it.
        // A binding inside a block is captured by value when the closure is made, at top level too:
        // a loop makes a new one each time round. Only the file's own top-level bindings are shared.
        // A `var` is shared with the closure (spec R05.10.1): the capture is a reference to it.
        let (mut_locals, var_locals, globals) =
            (&self.mut_locals, &self.var_locals, &self.typed.globals);
        let is_var = |v: &VarRef| match v {
            VarRef::Local(i) => mut_locals.get(*i).copied().unwrap_or(false),
            VarRef::Global(g) => {
                globals[*g].mutable || !crate::own::plain(self.d, &globals[*g].ty, 0)
            }
        };
        let is_mut = |v: &VarRef| match v {
            VarRef::Local(i) => var_locals.get(*i).copied().unwrap_or(false),
            VarRef::Global(g) => globals[*g].mutable,
        };
        let by_ref = |v: &VarRef, p: Place, ty: Ty| -> (VarRef, Place, Ty, bool) {
            if matches!(ty, Ty::Ref(_) | Ty::Lent(_)) {
                (*v, p, ty, false)
            } else if is_var(v) && !is_mut(v) {
                // A heap value the closure only reads: it points at it without writing (R05.10.1).
                (*v, p, Ty::Lent(Box::new(ty)), true)
            } else if is_var(v) {
                (*v, p, Ty::Ref(Box::new(ty)), true)
            } else {
                (*v, p, ty, false)
            }
        };
        let caps: Vec<(VarRef, Place, Ty, bool)> = used
            .iter()
            .filter(|v| !bound.contains(v) && !param_refs.contains(v))
            .filter_map(|v| match (v, self.remap.get(v)) {
                (_, Some(l)) => Some(by_ref(v, Place::Local(*l), self.locals[*l].clone())),
                (VarRef::Local(l), None) => {
                    Some(by_ref(v, Place::Local(*l), self.locals[*l].clone()))
                }
                (VarRef::Global(g), None) if !self.typed.globals[*g].top => Some(by_ref(
                    v,
                    Place::Global(*g),
                    self.typed.globals[*g].ty.clone(),
                )),
                (VarRef::Global(_), None) => None,
            })
            .collect();
        // The lambda's body is lowered with its own local table: captures first, then parameters.
        let outer_locals = std::mem::take(&mut self.locals);
        let mut fparams = Vec::new();
        let mut remap = HashMap::new();
        for (key, _, ty, _) in &caps {
            let l = self.temp(ty.clone());
            remap.insert(*key, l);
            fparams.push((l, false));
        }
        for (p, pty) in param_refs.iter().zip(&info.params) {
            let l = self.temp(pty.clone());
            remap.insert(*p, l);
            fparams.push((l, false));
        }
        // Locals bound inside the body keep their numbers in a fn; at top level they become locals here.
        let inner_base = if self.in_main {
            None
        } else {
            Some(outer_locals.clone())
        };
        for b in &bound {
            let ty = match b {
                VarRef::Local(i) => match inner_base.as_ref().map(|ls| ls.get(*i).cloned()) {
                    Some(None) => {
                        self.errors.push(
                            "slice: a lambda that makes a lambda with its own parameters".into(),
                        );
                        Ty::Error
                    }
                    Some(Some(t)) => t,
                    None => Ty::Error,
                },
                VarRef::Global(i) => match self.typed.globals.get(*i) {
                    Some(g) => g.ty.clone(),
                    None => {
                        self.errors.push(
                            "slice: a lambda that makes a lambda with its own parameters".into(),
                        );
                        Ty::Error
                    }
                },
            };
            let l = self.temp(ty);
            remap.insert(*b, l);
        }
        let itself = (caps.iter())
            .map(|(key, _, ty, _)| Expr {
                kind: EK::Copy(Place::Local(remap[key])),
                ty: ty.clone(),
            })
            .collect();
        self.selves.push((id, itself));
        self.remap = remap;
        let was_main = self.in_main;
        let mut stmts = Vec::new();
        let v = self.expr(body, &mut stmts);
        self.selves.pop();
        if matches!(stmts.last(), Some(Stmt::Return(_))) {
        } else if info.ret != Ty::Unit {
            stmts.push(Stmt::Return(Some(v)));
        } else {
            self.effect(v, &mut stmts);
        }
        self.in_main = was_main;
        let locals = std::mem::replace(&mut self.locals, outer_locals);
        self.remap = saved_remap;
        self.lambdas.push(Func {
            key: FuncKey::Lambda(id),
            name: format!("lambda{}", id.0),
            params: fparams,
            n_captures: caps.len(),
            locals,
            body: stmts,
            ret: info.ret.clone(),
            region: None,
            line: 0,
        });
        caps.into_iter()
            .map(|(_, p, ty, addr)| Expr {
                kind: match (addr, &ty) {
                    (true, Ty::Lent(_)) => EK::Lent(p),
                    (true, _) => EK::AddrOf(p),
                    _ => EK::Copy(p),
                },
                ty,
            })
            .collect()
    }
}

pub fn free_vars(e: &TExpr, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>) {
    let arg = |a: &TArg, used: &mut BTreeSet<VarRef>, bound: &mut BTreeSet<VarRef>| match a {
        TArg::Value(v) | TArg::Mut(v) => free_vars(v, used, bound),
    };
    match &e.kind {
        TK::Var(v) => {
            used.insert(*v);
        }
        TK::Field(b, _)
        | TK::Payload(b, _)
        | TK::Tag(b)
        | TK::Variant(_, b)
        | TK::Inject(b)
        | TK::Widen(b)
        | TK::Elem(b)
        | TK::Not(b)
        | TK::CharOf(b)
        | TK::CodeOf(b)
        | TK::Neg(b)
        | TK::ToU64(b)
        | TK::FromU64(b)
        | TK::DomOrd(b)
        | TK::DomFrom(_, b)
        | TK::DomNth(_, b)
        | TK::DomHas(_, b) => free_vars(b, used, bound),
        TK::Call { recv, args, .. } => {
            if let Some(r) = recv {
                arg(r, used, bound);
            }
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::CallTrait { recv, args, .. } => {
            arg(recv, used, bound);
            args.iter().for_each(|a| arg(a, used, bound));
        }
        TK::CallValue { callee, args } => {
            free_vars(callee, used, bound);
            args.iter().for_each(|a| free_vars(a, used, bound));
        }
        TK::Struct(fs) => fs.iter().for_each(|a| free_vars(a, used, bound)),
        TK::Lambda { params, body, .. } => {
            free_vars(body, used, bound);
            for p in params {
                bound.insert(VarRef::Local(*p));
                bound.insert(VarRef::Global(*p));
            }
        }
        TK::Bin(_, a, b) | TK::Cmp(_, a, b) | TK::Slot(a, b) => {
            free_vars(a, used, bound);
            free_vars(b, used, bound);
        }
        TK::If(a, b, c) => {
            free_vars(a, used, bound);
            free_vars(b, used, bound);
            free_vars(c, used, bound);
        }
        TK::Block(ss, v) => {
            for s in ss {
                match s {
                    TStmt::Let(v, e) => {
                        bound.insert(*v);
                        free_vars(e, used, bound);
                    }
                    TStmt::Expr(e) | TStmt::Assert(e, ..) => free_vars(e, used, bound),
                    TStmt::Assign(a, b) | TStmt::While(a, b) => {
                        free_vars(a, used, bound);
                        free_vars(b, used, bound);
                    }
                    TStmt::For(v, a, b, c) => {
                        bound.insert(*v);
                        free_vars(a, used, bound);
                        free_vars(b, used, bound);
                        free_vars(c, used, bound);
                    }
                    TStmt::Return(Some(e)) | TStmt::Yield(e) | TStmt::LendOut(e) => {
                        free_vars(e, used, bound)
                    }
                    TStmt::Region {
                        recv,
                        args,
                        var,
                        body,
                        ..
                    } => {
                        bound.insert(*var);
                        if let Some(r) = recv {
                            arg(r, used, bound);
                        }
                        args.iter().for_each(|a| arg(a, used, bound));
                        free_vars(body, used, bound);
                    }
                    _ => {}
                }
            }
            if let Some(v) = v {
                free_vars(v, used, bound);
            }
        }
        TK::Print(ps) => ps.iter().for_each(|p| {
            if let Piece::Value(v) = p {
                free_vars(v, used, bound)
            }
        }),
        _ => {}
    }
}
