//! Contract 5: generic IR -> instances. The link step: it decides which
//! instances exist and what each function-value set holds, from the whole program.
use crate::check::Typed;
use crate::decls::Decls;
use crate::ir::*;
use crate::ty::*;
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum CTy {
    Int,
    Bool,
    Unit,
    Raw(Box<CTy>),
    Ref(Box<CTy>),
    Struct(StructId, Vec<CTy>, Vec<CSet>),
    /// A function value: one of these closures. The order is the tag.
    Set(CSet),
    /// One closure: its captures.
    Clo(Box<InstKey>),
    /// A domain declared in a body. Its values are positions; its size is a run-time value.
    Dom(usize),
    Error,
}
pub type CSet = Vec<InstKey>;
#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct InstKey {
    pub func: FuncKey,
    pub targs: Vec<CTy>,
    pub sets: Vec<CSet>,
}

pub struct Program {
    pub insts: Vec<Func<CTy>>,
    pub index: HashMap<InstKey, usize>,
    pub main: usize,
    pub globals: Vec<CTy>,
    pub structs: HashMap<CTy, Vec<CTy>>,
    pub errors: Vec<String>,
}
impl Program {
    /// The types of what closure `k` captures.
    pub fn captures(&self, k: &InstKey) -> Vec<CTy> {
        let f = &self.insts[self.index[k]];
        f.params[..f.n_captures]
            .iter()
            .map(|(l, _)| f.locals[*l].clone())
            .collect()
    }
}

struct Link<'a> {
    d: &'a Decls,
    typed: &'a Typed,
    generic: &'a HashMap<FuncKey, Func<Ty>>,
    keys: Vec<InstKey>,
    index: HashMap<InstKey, usize>,
    structs: HashMap<CTy, Vec<CTy>>,
    ret_busy: BTreeSet<(InstKey, usize)>,
    named: HashMap<FuncKey, BTreeSet<usize>>,
    errors: Vec<String>,
}
struct Env<'e> {
    targs: &'e [CTy],
    sets: &'e [CSet],
}

const LIMIT: usize = 400;

fn doms_of(t: &CTy, out: &mut Vec<usize>) {
    match t {
        CTy::Dom(d) => out.push(*d),
        CTy::Raw(e) | CTy::Ref(e) => doms_of(e, out),
        CTy::Struct(_, ts, sets) => {
            ts.iter().for_each(|t| doms_of(t, out));
            sets.iter()
                .flatten()
                .for_each(|k| k.targs.iter().for_each(|t| doms_of(t, out)));
        }
        CTy::Set(set) => set
            .iter()
            .for_each(|k| k.targs.iter().for_each(|t| doms_of(t, out))),
        CTy::Clo(k) => k.targs.iter().for_each(|t| doms_of(t, out)),
        _ => {}
    }
}

pub fn link(
    d: &Decls,
    typed: &Typed,
    generic: &HashMap<FuncKey, Func<Ty>>,
    globals: &[Ty],
) -> Program {
    let mut l = Link {
        d,
        typed,
        generic,
        keys: Vec::new(),
        index: HashMap::new(),
        structs: HashMap::new(),
        ret_busy: BTreeSet::new(),
        named: HashMap::new(),
        errors: Vec::new(),
    };
    let main = l.intern(InstKey {
        func: FuncKey::Main,
        targs: vec![],
        sets: vec![],
    });
    let mut insts: Vec<Func<CTy>> = Vec::new();
    let mut next = 0;
    while next < l.keys.len() {
        if l.keys.len() > LIMIT {
            l.errors.push(format!(
                "instantiation does not end: `{}` makes a new copy of itself at every level (a function passed to its own recursive call, wrapped in a new closure)",
                generic[&l.keys[next].func].name
            ));
            break;
        }
        let key = l.keys[next].clone();
        // A fn used at a domain declared in its own body: inside that instance the type
        // argument and the local domain would be one type with two sizes. A run-time value
        // has to be keyed by its binding for this to run; until then it is refused.
        let owner = match &key.func {
            FuncKey::Decl(f) => Some(*f),
            FuncKey::Lambda(id) => typed.lambdas[id.0].owner,
            FuncKey::Main => None,
        };
        let mut doms = Vec::new();
        key.targs.iter().for_each(|t| doms_of(t, &mut doms));
        if let Some(d) = doms
            .iter()
            .find(|d| owner.is_some() && typed.domains[**d].owner == owner)
        {
            l.errors.push(format!(
                "`{}` is called with its own domain `{}` as a type argument, so each call would need a new copy of it; declare the domain outside, or pass its values as `Int`",
                generic[&key.func].name, typed.domains[*d].name
            ));
            break;
        }
        let f = &generic[&key.func];
        let env = Env {
            targs: &key.targs,
            sets: &key.sets,
        };
        let inst = Func {
            key: f.key.clone(),
            name: f.name.clone(),
            params: f.params.clone(),
            n_captures: f.n_captures,
            locals: f.locals.iter().map(|t| l.ty(t, &env)).collect(),
            body: l.block(&f.body, &env),
            ret: l.ty(&f.ret, &env),
            region: f.region.as_ref().map(|(lend, t)| (*lend, l.ty(t, &env))),
            line: f.line,
        };
        insts.push(inst);
        next += 1;
    }
    let env = Env {
        targs: &[],
        sets: &[],
    };
    let globals: Vec<CTy> = globals.iter().map(|t| l.ty(t, &env)).collect();
    // Instances made while typing the globals.
    while insts.len() < l.keys.len() && l.keys.len() <= LIMIT {
        let key = l.keys[insts.len()].clone();
        let f = &generic[&key.func];
        let env = Env {
            targs: &key.targs,
            sets: &key.sets,
        };
        let inst = Func {
            key: f.key.clone(),
            name: f.name.clone(),
            params: f.params.clone(),
            n_captures: f.n_captures,
            locals: f.locals.iter().map(|t| l.ty(t, &env)).collect(),
            body: l.block(&f.body, &env),
            ret: l.ty(&f.ret, &env),
            region: f.region.as_ref().map(|(lend, t)| (*lend, l.ty(t, &env))),
            line: f.line,
        };
        insts.push(inst);
    }
    let mut p = Program {
        insts,
        index: l.index,
        main,
        globals,
        structs: l.structs,
        errors: l.errors,
    };
    if p.errors.is_empty() {
        thread_values(&mut p, typed);
    }
    if p.errors.is_empty() {
        check_finite(&mut p);
        check_escape(&mut p);
    }
    p
}

impl Link<'_> {
    fn intern(&mut self, k: InstKey) -> usize {
        if let Some(i) = self.index.get(&k) {
            return *i;
        }
        self.keys.push(k.clone());
        self.index.insert(k, self.keys.len() - 1);
        self.keys.len() - 1
    }
    fn ty(&mut self, t: &Ty, env: &Env) -> CTy {
        match t {
            Ty::Int | Ty::Range(..) | Ty::RangeOf(_) => CTy::Int,
            Ty::Bool => CTy::Bool,
            Ty::Unit => CTy::Unit,
            Ty::Raw(e) => CTy::Raw(Box::new(self.ty(e, env))),
            Ty::Ref(e) => CTy::Ref(Box::new(self.ty(e, env))),
            Ty::Param(i) => env.targs.get(*i).cloned().unwrap_or(CTy::Error),
            Ty::Dom(d) => CTy::Dom(*d),
            Ty::Struct(d, ts, ss) => {
                let ts: Vec<CTy> = ts.iter().map(|t| self.ty(t, env)).collect();
                let ss: Vec<CSet> = ss.iter().map(|s| self.set(s, env)).collect();
                let out = CTy::Struct(*d, ts.clone(), ss.clone());
                if !self.structs.contains_key(&out) {
                    self.structs.insert(out.clone(), Vec::new());
                    let fields = self.d.s(*d).fields.clone();
                    let inner = Env {
                        targs: &ts,
                        sets: &ss,
                    };
                    let fs = fields.iter().map(|(_, t)| self.ty(t, &inner)).collect();
                    self.structs.insert(out.clone(), fs);
                }
                out
            }
            Ty::Fn(_, _, s) => CTy::Set(self.set(s, env)),
            Ty::Clo(c) => {
                let k = self.clo(c, env);
                self.intern(k.clone());
                CTy::Clo(Box::new(k))
            }
            Ty::Var(_) | Ty::Error => CTy::Error,
        }
    }
    fn fnref(&mut self, f: &FnRef, env: &Env) -> InstKey {
        InstKey {
            func: FuncKey::Decl(f.decl),
            targs: f.targs.iter().map(|t| self.ty(t, env)).collect(),
            sets: f.sets.iter().map(|s| self.set(s, env)).collect(),
        }
    }
    /// The indices of type or set parameters a fn's generic IR names anywhere. An upper bound
    /// (type and set parameters print alike), which is the safe direction.
    fn params_named(&mut self, k: FuncKey) -> BTreeSet<usize> {
        if let Some(u) = self.named.get(&k) {
            return u.clone();
        }
        let text = self
            .generic
            .get(&k)
            .map(|f| format!("{:?} {:?} {:?}", f.locals, f.ret, f.body))
            .unwrap_or_default();
        let mut u = BTreeSet::new();
        for part in text.split("Param(").skip(1) {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(i) = digits.parse() {
                u.insert(i);
            }
        }
        self.named.insert(k, u.clone());
        u
    }
    fn clo(&mut self, c: &CloRef, env: &Env) -> InstKey {
        match c {
            CloRef::Item(f) => self.fnref(f, env),
            CloRef::Lambda(id, ts, ss) => {
                // A lambda is one instance per value of the parameters its body names. A set
                // parameter of the fn around it that it never mentions is left out of its
                // identity, so a closure passed to a recursive call of that fn (the callback
                // of a DFS) is one instance, not one per level.
                let used = self.params_named(FuncKey::Lambda(*id));
                let sets = ss
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        if used.contains(&i) {
                            self.set(s, env)
                        } else {
                            Vec::new()
                        }
                    })
                    .collect();
                InstKey {
                    func: FuncKey::Lambda(*id),
                    targs: ts.iter().map(|t| self.ty(t, env)).collect(),
                    sets,
                }
            }
        }
    }
    /// The closures a set holds, for this instance. Sorted, so the same set is the same type everywhere.
    fn set(&mut self, s: &SetTy, env: &Env) -> CSet {
        let mut out = BTreeSet::new();
        self.set_into(s, env, &mut out);
        let v: CSet = out.into_iter().collect();
        for k in &v {
            self.intern(k.clone());
        }
        v
    }
    fn set_into(&mut self, s: &SetTy, env: &Env, out: &mut BTreeSet<InstKey>) {
        match s {
            SetTy::Param(i) => out.extend(env.sets.get(*i).cloned().unwrap_or_default()),
            SetTy::Closed(ms) => {
                for m in ms {
                    match m {
                        Member::Clo(c) => {
                            out.insert(self.clo(c, env));
                        }
                        Member::Sub(s) => self.set_into(s, env, out),
                    }
                }
            }
            SetTy::RetOf(f, slot) => {
                let k = self.fnref(f, env);
                // A fn that returns what it itself returns adds nothing.
                if self.ret_busy.insert((k.clone(), *slot)) {
                    if let Some(rs) = self.typed.ret_sets.get(&(f.decl, *slot)) {
                        let inner = Env {
                            targs: &k.targs,
                            sets: &k.sets,
                        };
                        self.set_into(&rs.clone(), &inner, out);
                    }
                    self.ret_busy.remove(&(k, *slot));
                }
            }
            SetTy::RetSlot(_) => {}
            SetTy::Var(_) => {}
        }
    }
    fn block(&mut self, b: &[Stmt<Ty>], env: &Env) -> Vec<Stmt<CTy>> {
        b.iter()
            .map(|s| match s {
                Stmt::Let(l, e) => Stmt::Let(*l, self.expr(e, env)),
                Stmt::Assign(p, e) => Stmt::Assign(p.clone(), self.expr(e, env)),
                Stmt::Expr(e) => Stmt::Expr(self.expr(e, env)),
                Stmt::If(c, a, b) => {
                    Stmt::If(self.expr(c, env), self.block(a, env), self.block(b, env))
                }
                Stmt::Loop(a, b) => Stmt::Loop(self.block(a, env), self.block(b, env)),
                Stmt::Switch(c, arms) => Stmt::Switch(
                    self.expr(c, env),
                    arms.iter().map(|a| self.block(a, env)).collect(),
                ),
                Stmt::Break => Stmt::Break,
                Stmt::Continue => Stmt::Continue,
                Stmt::Return(e) => Stmt::Return(e.as_ref().map(|e| self.expr(e, env))),
                Stmt::Check {
                    kind,
                    cond,
                    msg,
                    site,
                } => Stmt::Check {
                    kind: *kind,
                    cond: self.expr(cond, env),
                    msg: msg.clone(),
                    site: *site,
                },
                Stmt::Print(ps) => Stmt::Print(
                    ps.iter()
                        .map(|p| match p {
                            PrintPiece::Text(t) => PrintPiece::Text(t.clone()),
                            PrintPiece::Value(v) => PrintPiece::Value(self.expr(v, env)),
                        })
                        .collect(),
                ),
                Stmt::Yield(e) => Stmt::Yield(self.expr(e, env)),
                Stmt::LendOut(p) => Stmt::LendOut(p.clone()),
                Stmt::Event(l, vs) => {
                    Stmt::Event(l.clone(), vs.iter().map(|v| self.expr(v, env)).collect())
                }
                Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body,
                    site,
                } => {
                    let args = args.iter().map(|a| self.arg(a, env)).collect();
                    let body = self.block(body, env);
                    match self.target(target, env) {
                        Some(i) => Stmt::Region {
                            target: Target::Inst(i),
                            args,
                            var: *var,
                            lend: *lend,
                            body,
                            site: *site,
                        },
                        None => Stmt::Break,
                    }
                }
            })
            .collect()
    }
    /// The instance a call reaches. A method of a bound is resolved here, now that the type is known.
    fn target(&mut self, t: &Target<Ty>, env: &Env) -> Option<usize> {
        let key = match t {
            Target::Fn(f) => Some(self.fnref(f, env)),
            // A method of a bound: now the type is known, so the one impl is too.
            Target::Trait {
                tr,
                method,
                self_ty,
            } => {
                let st = self.ty(self_ty, env);
                let found = match &st {
                    CTy::Struct(d, ts, ss) => self
                        .d
                        .impls
                        .iter()
                        .filter(|im| im.trait_ == *tr && im.for_ == *d)
                        .find_map(|im| im.methods.get(method).copied())
                        .map(|m| InstKey {
                            func: FuncKey::Decl(m),
                            targs: ts.clone(),
                            sets: ss.clone(),
                        }),
                    _ => None,
                };
                if found.is_none() {
                    self.errors
                        .push(format!("no impl provides `{method}` for this type"));
                }
                found
            }
            Target::Inst(i) => Some(self.keys[*i].clone()),
        };
        key.map(|k| self.intern(k))
    }
    fn arg(&mut self, a: &Arg<Ty>, env: &Env) -> Arg<CTy> {
        match a {
            Arg::Value(v) => Arg::Value(self.expr(v, env)),
            Arg::Lend(p) => Arg::Lend(p.clone()),
        }
    }
    fn expr(&mut self, e: &Expr<Ty>, env: &Env) -> Expr<CTy> {
        let ty = self.ty(&e.ty, env);
        let kind = match &e.kind {
            EK::Int(i) => EK::Int(*i),
            EK::Bool(b) => EK::Bool(*b),
            EK::Unit => EK::Unit,
            EK::Copy(p) => EK::Copy(p.clone()),
            EK::AddrOf(p) => EK::AddrOf(p.clone()),
            EK::Tag(p) => EK::Tag(p.clone()),
            EK::DomLen(t) => EK::DomLen(self.ty(t, env)),
            EK::MakeStruct(xs) => EK::MakeStruct(xs.iter().map(|x| self.expr(x, env)).collect()),
            EK::MakeClosure(xs) => EK::MakeClosure(xs.iter().map(|x| self.expr(x, env)).collect()),
            EK::Inject(x) => EK::Inject(Box::new(self.expr(x, env))),
            EK::Widen(x) => EK::Widen(Box::new(self.expr(x, env))),
            EK::MakeUnion(i, x) => EK::MakeUnion(*i, Box::new(self.expr(x, env))),
            EK::Intr(i, args, at) => {
                EK::Intr(*i, args.iter().map(|a| self.arg(a, env)).collect(), *at)
            }
            EK::CallValue(c, args, at) => EK::CallValue(
                Box::new(self.expr(c, env)),
                args.iter().map(|a| self.expr(a, env)).collect(),
                *at,
            ),
            EK::Call(t, args, at) => {
                let args: Vec<Arg<CTy>> = args.iter().map(|a| self.arg(a, env)).collect();
                match self.target(t, env) {
                    Some(i) => EK::Call(Target::Inst(i), args, *at),
                    None => EK::Unit,
                }
            }
        };
        Expr { kind, ty }
    }
}

/// A closure that shares a `var` of the fn around it is lent: it cannot be returned (spec R05.10).
/// Not checked here: storing one through a `mut` parameter.
fn check_escape(p: &mut Program) {
    fn holds_ref(p: &Program, t: &CTy, seen: &mut Vec<CTy>) -> bool {
        if seen.contains(t) {
            return false;
        }
        seen.push(t.clone());
        match t {
            CTy::Ref(_) => true,
            CTy::Raw(e) => holds_ref(p, e, seen),
            CTy::Struct(..) => p.structs[t].iter().any(|f| holds_ref(p, f, seen)),
            CTy::Set(ms) => ms
                .iter()
                .any(|k| holds_ref(p, &CTy::Clo(Box::new(k.clone())), seen)),
            CTy::Clo(k) => p.captures(k).iter().any(|c| holds_ref(p, c, seen)),
            _ => false,
        }
    }
    let bad: Vec<String> = p
        .insts
        .iter()
        .filter(|f| holds_ref(p, &f.ret, &mut Vec::new()))
        .map(|f| f.name.clone())
        .collect();
    for name in bad {
        p.errors.push(format!("`{name}` returns a function that uses a `var` of the fn that made it; the variable is gone by then"));
    }
}

/// A value must have a finite size. A closure that captures a function value
/// whose set contains that closure does not.
fn check_finite(p: &mut Program) {
    fn visit(p: &Program, t: &CTy, stack: &mut Vec<CTy>, bad: &mut bool) {
        if *bad
            || matches!(
                t,
                CTy::Int
                    | CTy::Bool
                    | CTy::Unit
                    | CTy::Raw(_)
                    | CTy::Ref(_)
                    | CTy::Dom(_)
                    | CTy::Error
            )
        {
            return;
        }
        if stack.contains(t) {
            *bad = true;
            return;
        }
        stack.push(t.clone());
        match t {
            CTy::Struct(..) => {
                for f in &p.structs[t] {
                    visit(p, f, stack, bad);
                }
            }
            CTy::Set(ms) => {
                for k in ms {
                    visit(p, &CTy::Clo(Box::new(k.clone())), stack, bad);
                }
            }
            CTy::Clo(k) => {
                for c in p.captures(k) {
                    visit(p, &c, stack, bad);
                }
            }
            _ => {}
        }
        stack.pop();
    }
    let mut bad = false;
    for f in &p.insts {
        for t in &f.locals {
            visit(p, t, &mut Vec::new(), &mut bad);
        }
    }
    for t in &p.globals {
        visit(p, t, &mut Vec::new(), &mut bad);
    }
    if bad {
        p.errors.push("a function value captures a function value that can hold itself, so it has no fixed size; keep the steps in a list and apply them in a loop".to_string());
    }
}

/// Contract 6, required rewrites: a call through a function value becomes a
/// branch over the closures it can be, and each call a direct call.
/// The canonical form of every instance, or what the link step left inconsistent.
pub fn canonicalize(p: &Program) -> Result<Vec<Func<CTy>>, Vec<String>> {
    let mut bad = Vec::new();
    let funcs = p
        .insts
        .iter()
        .map(|f| {
            let mut f = f.clone();
            let body = std::mem::take(&mut f.body);
            f.body = canon_block(p, &mut f.locals, &mut bad, body);
            f
        })
        .collect();
    if bad.is_empty() {
        Ok(funcs)
    } else {
        Err(bad)
    }
}

fn read(p: Place, ty: CTy) -> Expr<CTy> {
    Expr {
        kind: EK::Copy(p),
        ty,
    }
}
fn canon_block(
    p: &Program,
    locals: &mut Vec<CTy>,
    bad: &mut Vec<String>,
    b: Vec<Stmt<CTy>>,
) -> Vec<Stmt<CTy>> {
    let mut out = Vec::new();
    for s in b {
        match s {
            Stmt::Let(l, e) => canon_store(p, locals, bad, Place::Local(l), Some(l), e, &mut out),
            Stmt::Assign(pl, e) => canon_store(p, locals, bad, pl, None, e, &mut out),
            Stmt::Expr(e) => match e.kind {
                EK::CallValue(..) => {
                    locals.push(e.ty.clone());
                    let t = locals.len() - 1;
                    canon_store(p, locals, bad, Place::Local(t), Some(t), e, &mut out)
                }
                _ => out.push(Stmt::Expr(canon_expr(bad, e))),
            },
            Stmt::If(c, a, b2) => out.push(Stmt::If(
                canon_expr(bad, c),
                canon_block(p, locals, bad, a),
                canon_block(p, locals, bad, b2),
            )),
            Stmt::Loop(a, b2) => out.push(Stmt::Loop(
                canon_block(p, locals, bad, a),
                canon_block(p, locals, bad, b2),
            )),
            Stmt::Switch(c, arms) => out.push(Stmt::Switch(
                canon_expr(bad, c),
                arms.into_iter()
                    .map(|a| canon_block(p, locals, bad, a))
                    .collect(),
            )),
            Stmt::Return(e) => out.push(Stmt::Return(e.map(|e| canon_expr(bad, e)))),
            Stmt::Check {
                kind,
                cond,
                msg,
                site,
            } => out.push(Stmt::Check {
                kind,
                cond: canon_expr(bad, cond),
                msg,
                site,
            }),
            Stmt::Print(ps) => out.push(Stmt::Print(
                ps.into_iter()
                    .map(|x| match x {
                        PrintPiece::Value(v) => PrintPiece::Value(canon_expr(bad, v)),
                        t => t,
                    })
                    .collect(),
            )),
            Stmt::Yield(e) => out.push(Stmt::Yield(canon_expr(bad, e))),
            Stmt::Region {
                target,
                args,
                var,
                lend,
                body,
                site,
            } => {
                let args = args
                    .into_iter()
                    .map(|a| match a {
                        Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                        l => l,
                    })
                    .collect();
                out.push(Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body: canon_block(p, locals, bad, body),
                    site,
                });
            }
            other => out.push(other),
        }
    }
    out
}
fn canon_store(
    p: &Program,
    locals: &mut Vec<CTy>,
    bad: &mut Vec<String>,
    dst: Place,
    let_local: Option<usize>,
    e: Expr<CTy>,
    out: &mut Vec<Stmt<CTy>>,
) {
    let store = |v: Expr<CTy>| match let_local {
        Some(l) => Stmt::Let(l, v),
        None => Stmt::Assign(dst.clone(), v),
    };
    match e.kind {
        EK::CallValue(callee, args, at) => {
            let cty = callee.ty.clone();
            locals.push(cty.clone());
            let tmp = locals.len() - 1;
            out.push(Stmt::Let(tmp, canon_expr(bad, *callee)));
            let args: Vec<Expr<CTy>> = args.into_iter().map(|e| canon_expr(bad, e)).collect();
            let call = |k: &InstKey, env: Place| {
                let mut all: Vec<Arg<CTy>> = p
                    .captures(k)
                    .into_iter()
                    .enumerate()
                    .map(|(j, t)| Arg::Value(read(Place::Field(Box::new(env.clone()), j), t)))
                    .collect();
                all.extend(args.iter().cloned().map(Arg::Value));
                Expr {
                    kind: EK::Call(Target::Inst(p.index[k]), all, at),
                    ty: e.ty.clone(),
                }
            };
            match &cty {
                CTy::Clo(k) => out.push(store(call(k, Place::Local(tmp)))),
                CTy::Set(ms) => {
                    let arms = ms
                        .iter()
                        .enumerate()
                        .map(|(i, k)| {
                            vec![Stmt::Assign(
                                dst.clone(),
                                call(k, Place::Variant(Box::new(Place::Local(tmp)), i)),
                            )]
                        })
                        .collect();
                    out.push(Stmt::Switch(
                        Expr {
                            kind: EK::Tag(Place::Local(tmp)),
                            ty: CTy::Int,
                        },
                        arms,
                    ));
                }
                _ => {}
            }
        }
        EK::Widen(inner) => {
            let (from, to) = (inner.ty.clone(), e.ty.clone());
            if from == to {
                return out.push(store(canon_expr(bad, *inner)));
            }
            let (CTy::Set(a), CTy::Set(b)) = (&from, &to) else {
                return;
            };
            locals.push(from.clone());
            let tmp = locals.len() - 1;
            out.push(Stmt::Let(tmp, canon_expr(bad, *inner)));
            let arms = a
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let j = b.iter().position(|x| x == k).unwrap_or_else(|| {
                        bad.push(
                            "internal: a function value is widened to a set that lacks it"
                                .to_string(),
                        );
                        0
                    });
                    let payload = read(
                        Place::Variant(Box::new(Place::Local(tmp)), i),
                        CTy::Clo(Box::new(k.clone())),
                    );
                    vec![Stmt::Assign(
                        dst.clone(),
                        Expr {
                            kind: EK::MakeUnion(j, Box::new(payload)),
                            ty: to.clone(),
                        },
                    )]
                })
                .collect();
            out.push(Stmt::Switch(
                Expr {
                    kind: EK::Tag(Place::Local(tmp)),
                    ty: CTy::Int,
                },
                arms,
            ));
        }
        _ => out.push(store(canon_expr(bad, e))),
    }
}
fn canon_expr(bad: &mut Vec<String>, e: Expr<CTy>) -> Expr<CTy> {
    let ty = e.ty.clone();
    let kind = match e.kind {
        EK::Inject(x) => {
            let x = canon_expr(bad, *x);
            match (&x.ty, &ty) {
                (CTy::Clo(k), CTy::Set(ms)) => {
                    let at = ms.iter().position(|m| m == &**k).unwrap_or_else(|| {
                        bad.push(
                            "internal: a function value is put in a set that lacks it".to_string(),
                        );
                        0
                    });
                    EK::MakeUnion(at, Box::new(x))
                }
                _ => x.kind,
            }
        }
        EK::Widen(x) => {
            // Only reached when both sets are the same.
            canon_expr(bad, *x).kind
        }
        EK::MakeStruct(xs) => EK::MakeStruct(xs.into_iter().map(|e| canon_expr(bad, e)).collect()),
        EK::MakeClosure(xs) => {
            EK::MakeClosure(xs.into_iter().map(|e| canon_expr(bad, e)).collect())
        }
        EK::MakeUnion(i, x) => EK::MakeUnion(i, Box::new(canon_expr(bad, *x))),
        EK::Intr(i, args, at) => EK::Intr(
            i,
            args.into_iter()
                .map(|a| match a {
                    Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                    l => l,
                })
                .collect(),
            at,
        ),
        EK::Call(t, args, at) => EK::Call(
            t,
            args.into_iter()
                .map(|a| match a {
                    Arg::Value(v) => Arg::Value(canon_expr(bad, v)),
                    l => l,
                })
                .collect(),
            at,
        ),
        k => k,
    };
    Expr { kind, ty }
}

/// Run-time values that types carry (the size of a domain) are passed here.
/// A function that needs the size of a domain it did not declare gets a hidden
/// leading parameter for it; a closure captures it when it is made. Which
/// functions need which is a whole-program question, so the link step answers it.
fn thread_values(p: &mut Program, typed: &Typed) {
    use crate::check::VarRef;
    let n = p.insts.len();
    // Where an instance can read a domain's size without being handed it.
    let own = |f: &Func<CTy>, d: usize| -> Option<Place> {
        let info = &typed.domains[d];
        match (info.len, &f.key, info.owner) {
            (VarRef::Global(g), FuncKey::Main, None) => Some(Place::Global(g)),
            // A top-level domain of the file is one shared variable. One inside a block is not: a loop makes a new one.
            (VarRef::Global(g), _, None) if typed.globals[g].top => Some(Place::Global(g)),
            (VarRef::Local(l), FuncKey::Decl(k), Some(o)) if *k == o => Some(Place::Local(l)),
            _ => None,
        }
    };
    fn uses(
        b: &[Stmt<CTy>],
        doms: &mut BTreeSet<usize>,
        calls: &mut BTreeSet<usize>,
        p: &Program,
        bad: &mut bool,
    ) {
        fn ex(
            e: &Expr<CTy>,
            doms: &mut BTreeSet<usize>,
            calls: &mut BTreeSet<usize>,
            p: &Program,
            bad: &mut bool,
        ) {
            match &e.kind {
                EK::DomLen(CTy::Dom(d)) => {
                    doms.insert(*d);
                }
                EK::DomLen(_) => *bad = true,
                EK::MakeStruct(xs) => xs.iter().for_each(|x| ex(x, doms, calls, p, bad)),
                EK::MakeClosure(xs) => {
                    if let CTy::Clo(k) = &e.ty {
                        calls.insert(p.index[&**k]);
                    }
                    xs.iter().for_each(|x| ex(x, doms, calls, p, bad));
                }
                EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => ex(x, doms, calls, p, bad),
                EK::Intr(_, args, _) | EK::Call(_, args, _) => {
                    if let EK::Call(Target::Inst(j), _, _) = &e.kind {
                        calls.insert(*j);
                    }
                    for a in args {
                        if let Arg::Value(v) = a {
                            ex(v, doms, calls, p, bad);
                        }
                    }
                }
                EK::CallValue(c, args, _) => {
                    ex(c, doms, calls, p, bad);
                    args.iter().for_each(|x| ex(x, doms, calls, p, bad));
                }
                _ => {}
            }
        }
        for s in b {
            match s {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) | Stmt::Return(Some(e)) => {
                    ex(e, doms, calls, p, bad)
                }
                Stmt::Check { cond, .. } => ex(cond, doms, calls, p, bad),
                Stmt::If(c, a, b2) => {
                    ex(c, doms, calls, p, bad);
                    uses(a, doms, calls, p, bad);
                    uses(b2, doms, calls, p, bad);
                }
                Stmt::Loop(a, b2) => {
                    uses(a, doms, calls, p, bad);
                    uses(b2, doms, calls, p, bad);
                }
                Stmt::Switch(c, arms) => {
                    ex(c, doms, calls, p, bad);
                    arms.iter().for_each(|a| uses(a, doms, calls, p, bad));
                }
                Stmt::Print(ps) => ps.iter().for_each(|x| {
                    if let PrintPiece::Value(v) = x {
                        ex(v, doms, calls, p, bad)
                    }
                }),
                Stmt::Yield(e) => ex(e, doms, calls, p, bad),
                Stmt::Region {
                    target, args, body, ..
                } => {
                    if let Target::Inst(j) = target {
                        calls.insert(*j);
                    }
                    for a in args {
                        if let Arg::Value(v) = a {
                            ex(v, doms, calls, p, bad);
                        }
                    }
                    uses(body, doms, calls, p, bad);
                }
                _ => {}
            }
        }
    }
    let mut direct = vec![BTreeSet::new(); n];
    let mut calls = vec![BTreeSet::new(); n];
    let mut bad = false;
    for i in 0..n {
        uses(&p.insts[i].body, &mut direct[i], &mut calls[i], p, &mut bad);
    }
    if bad {
        p.errors.push(
            "a type argument is used as a domain (`D.count`, `D(i)`, `for v in D`) but is not one"
                .to_string(),
        );
        return;
    }
    // What each instance must be handed: what it or anything it calls uses, less what it can read itself.
    let mut needs: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); n];
    loop {
        let mut changed = false;
        for i in 0..n {
            let mut want: BTreeSet<usize> = direct[i].clone();
            for j in &calls[i] {
                want.extend(needs[*j].iter().copied());
            }
            want.retain(|d| own(&p.insts[i], *d).is_none());
            if want != needs[i] {
                needs[i] = want;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    if !needs[p.main].is_empty() {
        p.errors
            .push("a domain's size is needed where the domain is not in scope".to_string());
        return;
    }
    // Hidden parameters come first, and count as captures: a closure holds them.
    let mut hidden: Vec<Vec<(usize, usize)>> = Vec::new();
    for i in 0..n {
        let f = &mut p.insts[i];
        let mut hs = Vec::new();
        for (at, d) in needs[i].iter().enumerate() {
            f.locals.push(CTy::Int);
            let l = f.locals.len() - 1;
            f.params.insert(at, (l, false));
            hs.push((*d, l));
        }
        f.n_captures += hs.len();
        hidden.push(hs);
    }
    struct Rw<'a> {
        hidden: &'a [Vec<(usize, usize)>],
        index: &'a HashMap<InstKey, usize>,
        me: usize,
        own: Vec<Option<Place>>,
        /// Set when a size is asked for and neither handed in nor declared here.
        no_source: std::cell::Cell<bool>,
    }
    impl Rw<'_> {
        fn size(&self, d: usize) -> Expr<CTy> {
            let place = match self.hidden[self.me].iter().find(|(x, _)| *x == d) {
                Some((_, l)) => Some(Place::Local(*l)),
                None => self.own[d].clone(),
            };
            let kind = place.map(EK::Copy).unwrap_or_else(|| {
                self.no_source.set(true);
                EK::Int(0)
            });
            Expr { kind, ty: CTy::Int }
        }
        fn handed(&self, j: usize) -> Vec<Expr<CTy>> {
            self.hidden[j].iter().map(|(d, _)| self.size(*d)).collect()
        }
        fn ex(&self, e: &mut Expr<CTy>) {
            let ty = e.ty.clone();
            match &mut e.kind {
                EK::DomLen(CTy::Dom(d)) => *e = self.size(*d),
                EK::MakeStruct(xs) => xs.iter_mut().for_each(|x| self.ex(x)),
                EK::MakeClosure(xs) => {
                    xs.iter_mut().for_each(|x| self.ex(x));
                    if let CTy::Clo(k) = &ty {
                        let mut all = self.handed(self.index[&**k]);
                        all.append(xs);
                        *xs = all;
                    }
                }
                EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.ex(x),
                EK::Intr(_, args, _) => args.iter_mut().for_each(|a| {
                    if let Arg::Value(v) = a {
                        self.ex(v)
                    }
                }),
                EK::Call(t, args, _) => {
                    args.iter_mut().for_each(|a| {
                        if let Arg::Value(v) = a {
                            self.ex(v)
                        }
                    });
                    if let Target::Inst(j) = t {
                        let mut all: Vec<Arg<CTy>> =
                            self.handed(*j).into_iter().map(Arg::Value).collect();
                        all.append(args);
                        *args = all;
                    }
                }
                EK::CallValue(c, args, _) => {
                    self.ex(c);
                    args.iter_mut().for_each(|x| self.ex(x));
                }
                _ => {}
            }
        }
        fn block(&self, b: &mut [Stmt<CTy>]) {
            for s in b {
                match s {
                    Stmt::Let(_, e)
                    | Stmt::Assign(_, e)
                    | Stmt::Expr(e)
                    | Stmt::Return(Some(e)) => self.ex(e),
                    Stmt::Check { cond, .. } => self.ex(cond),
                    Stmt::If(c, a, b2) => {
                        self.ex(c);
                        self.block(a);
                        self.block(b2);
                    }
                    Stmt::Loop(a, b2) => {
                        self.block(a);
                        self.block(b2);
                    }
                    Stmt::Switch(c, arms) => {
                        self.ex(c);
                        arms.iter_mut().for_each(|a| self.block(a));
                    }
                    Stmt::Print(ps) => ps.iter_mut().for_each(|x| {
                        if let PrintPiece::Value(v) = x {
                            self.ex(v)
                        }
                    }),
                    Stmt::Yield(e) => self.ex(e),
                    Stmt::Region {
                        target, args, body, ..
                    } => {
                        args.iter_mut().for_each(|a| {
                            if let Arg::Value(v) = a {
                                self.ex(v)
                            }
                        });
                        if let Target::Inst(j) = target {
                            let mut all: Vec<Arg<CTy>> =
                                self.handed(*j).into_iter().map(Arg::Value).collect();
                            all.append(args);
                            *args = all;
                        }
                        self.block(body);
                    }
                    _ => {}
                }
            }
        }
    }
    for i in 0..n {
        let own_places = (0..typed.domains.len())
            .map(|d| own(&p.insts[i], d))
            .collect();
        let rw = Rw {
            hidden: &hidden,
            index: &p.index,
            me: i,
            own: own_places,
            no_source: std::cell::Cell::new(false),
        };
        let mut body = std::mem::take(&mut p.insts[i].body);
        rw.block(&mut body);
        p.insts[i].body = body;
        if rw.no_source.get() {
            p.errors.push("internal: a domain's size is used in a fn that neither declares it nor is handed it".to_string());
        }
    }
}

/// Contract 8: event points, placed on instance IR after the link step, only when asked for.
/// They are not in generic IR and are not uses for ownership. Reported here: entering and
/// leaving a fn of the user's file, and each write to one of its top-level bindings.
pub fn add_events(p: &mut Program, typed: &Typed, d: &Decls, user_from: u32) {
    fn shown(t: &CTy) -> bool {
        matches!(t, CTy::Int | CTy::Bool | CTy::Dom(_))
    }
    fn block(b: Vec<Stmt<CTy>>, exit: Option<&str>, typed: &Typed) -> Vec<Stmt<CTy>> {
        let mut out = Vec::new();
        for s in b {
            match s {
                Stmt::Return(e) => {
                    if let Some(name) = exit {
                        out.push(Stmt::Event(
                            format!("exit {name}"),
                            e.iter().filter(|e| shown(&e.ty)).cloned().collect(),
                        ));
                    }
                    out.push(Stmt::Return(e));
                }
                Stmt::Assign(Place::Global(g), e) => {
                    let (top, name, ty) = (
                        typed.globals[g].top,
                        typed.globals[g].name.clone(),
                        e.ty.clone(),
                    );
                    out.push(Stmt::Assign(Place::Global(g), e));
                    if top && shown(&ty) && !name.contains('$') {
                        out.push(Stmt::Event(
                            format!("set {name}"),
                            vec![Expr {
                                kind: EK::Copy(Place::Global(g)),
                                ty,
                            }],
                        ));
                    }
                }
                Stmt::If(c, a, b2) => {
                    out.push(Stmt::If(c, block(a, exit, typed), block(b2, exit, typed)))
                }
                Stmt::Loop(a, b2) => {
                    out.push(Stmt::Loop(block(a, exit, typed), block(b2, exit, typed)))
                }
                Stmt::Switch(c, arms) => out.push(Stmt::Switch(
                    c,
                    arms.into_iter().map(|a| block(a, exit, typed)).collect(),
                )),
                Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body,
                    site,
                } => out.push(Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body: block(body, exit, typed),
                    site,
                }),
                other => out.push(other),
            }
        }
        out
    }
    for f in &mut p.insts {
        let user_fn = match &f.key {
            FuncKey::Decl(id) => d.f(*id).item >= user_from && f.region.is_none(),
            _ => false,
        };
        let body = std::mem::take(&mut f.body);
        let name = f.name.clone();
        let mut body = block(body, if user_fn { Some(&name) } else { None }, typed);
        if user_fn {
            let args = f.params[f.n_captures..]
                .iter()
                .filter(|(l, _)| shown(&f.locals[*l]))
                .map(|(l, _)| Expr {
                    kind: EK::Copy(Place::Local(*l)),
                    ty: f.locals[*l].clone(),
                })
                .collect();
            body.insert(0, Stmt::Event(format!("enter {name}"), args));
            if f.ret == CTy::Unit {
                body.push(Stmt::Event(format!("exit {name}"), vec![]));
            }
        }
        f.body = body;
    }
}
