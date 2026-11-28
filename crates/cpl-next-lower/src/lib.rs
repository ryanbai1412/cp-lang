//! Contract 4: typed tree -> generic IR. No inference, no solver, no lookup:
//! everything it needs is on the typed tree.
use cpl_diag::Span;
use cpl_next_check::*;
use cpl_next_decls::decls::{Decls, FnDecl, Owner, SelfMode};
use cpl_next_decls::diag::Msg;
use cpl_next_ir::build::{CheckName, Checks};
use cpl_next_ir::ids::{FileId, ItemId, SourceId};
use cpl_next_ir::ir::*;
use cpl_next_ir::ty::*;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_syntax::ops::{BinOp, CmpOp};
use std::collections::{BTreeSet, HashMap};

pub struct Generic {
    pub funcs: HashMap<FuncKey, Func<Ty>>,
    pub globals: Vec<Ty>,
    pub errors: Vec<Msg>,
    /// Each expression the checker could not type (an error node), in the offset space of the loaded modules.
    pub untyped: Vec<SourceId>,
    pub adds: cpl_next_ir::ty::SetAdds,
    /// What link reads of the program beyond its fns.
    pub whole: cpl_next_ir::whole::Whole,
    pub deferred: cpl_next_ir::msg::Deferred,
}

struct Lw<'a> {
    d: &'a Decls,
    typed: &'a Typed,
    /// The `@intrinsic` fns used as values: each is made a fn whose body is its operation.
    intrinsic_values: Vec<FnId>,
    /// The globals' types, a lent mark added where one is bound to a value that lends.
    globals: Vec<Ty>,
    locals: Vec<Ty>,
    /// The current fn's own bindings' types by the checker's numbers, as lowering refines them.
    /// A lambda's body binds these numbers too, even inside another lambda with a table of its own.
    fn_locals: Vec<Ty>,
    /// The names of the current fn's own bindings, empty at top level.
    tnames: Vec<String>,
    /// Which of the current fn's own bindings are `var`s.
    mut_locals: Vec<bool>,
    /// The `var`s among them: a closure writes these through its capture.
    var_locals: Vec<bool>,
    /// Inside a lambda at top level, its params live in the globals table; map them to locals.
    remap: HashMap<VarRef, usize>,
    /// The lambda's captures that hold an address: its body reads them where they point.
    by_addr: BTreeSet<usize>,
    lambdas: Vec<Func<Ty>>,
    /// The bound tests around the code being lowered, each with the branch it is in.
    guards: Vec<Guard>,
    /// The lambdas being lowered, each with its own value as its body makes it: its captures.
    /// A nested fn's siblings are in its group and share its captures.
    selves: Vec<(Vec<LambdaId>, Vec<Expr<Ty>>)>,
    /// What each nested fn of the blocks being lowered reads from around it, its group's together.
    group_free: HashMap<LambdaId, BTreeSet<VarRef>>,
    errors: Vec<Msg>,
    untyped: Vec<SourceId>,
    in_main: bool,
    /// The fn copy, top level or case being lowered: the maker of each lambda written in it.
    key: FuncKey,
    /// The code of one test case, once main's `Cases` is lowered.
    case: Option<Func<Ty>>,
    /// Where the item being lowered starts: spans are relative to it.
    off: ItemId,
    /// Where each line of the source starts, and how many of them are std.
    src: &'a cpl_next_decls::modules::Loaded,
    /// The fn being lowered is one the compiler wrote, a derived impl's: like std's, a check it
    /// makes fails at the line that called it.
    made: bool,
    sites: Vec<SourceId>,
    origins: Vec<u32>,
    /// The checks kept where the code being lowered is (`TStmt::Checks`, R09.4).
    kept: Checks,
}

pub fn lower(d: &Decls, typed: &Typed, src: &cpl_next_decls::modules::Loaded) -> Generic {
    let mut out = Generic {
        funcs: HashMap::new(),
        globals: Vec::new(),
        errors: Vec::new(),
        untyped: Vec::new(),
        adds: cpl_next_ir::ty::SetAdds {
            params: typed.param_adds.clone(),
            calls: typed.call_adds.clone(),
            globals: typed.global_adds.clone(),
        },
        whole: whole(d, typed),
        deferred: Default::default(),
    };
    for (f, msgs) in &typed.deferred {
        out.deferred.add(*f, msgs.iter().cloned());
    }
    let mut lw = Lw {
        d,
        typed,
        intrinsic_values: Vec::new(),
        globals: typed.globals.iter().map(|g| g.ty.clone()).collect(),
        locals: Vec::new(),
        fn_locals: Vec::new(),
        tnames: Vec::new(),
        mut_locals: Vec::new(),
        var_locals: Vec::new(),
        remap: HashMap::new(),
        by_addr: BTreeSet::new(),
        lambdas: Vec::new(),
        guards: Vec::new(),
        selves: Vec::new(),
        group_free: HashMap::new(),
        errors: Vec::new(),
        untyped: Vec::new(),
        in_main: true,
        key: FuncKey::Main,
        case: None,
        off: nowhere().item,
        src,
        made: false,
        sites: fresh_sites().0,
        origins: fresh_sites().1,
        kept: typed.checks,
    };
    let mut body = Vec::new();
    for s in &typed.main {
        lw.stmt(s, &mut body);
    }
    let (sites, origins) = lw.take_sites();
    out.funcs.insert(
        FuncKey::Main,
        Func {
            key: FuncKey::Main,
            name: "main".into(),
            params: vec![],
            n_captures: 0,
            names: lw.names(&[], true),
            vars: lw.vars(&[], true),
            locals: std::mem::take(&mut lw.locals),
            body,
            ret: Ty::Unit,
            region: None,
            line: 0,
            sites,
            origins,
            views: Vec::new(),
            drops: Vec::new(),
            maker: None,
            shown: "top level".into(),
            guards: Vec::new(),
            declared: Vec::new(),
            fast: None,
        },
    );
    if let Some(c) = lw.case.take() {
        out.funcs.insert(FuncKey::Case, c);
    }
    lw.in_main = false;
    let copies = (typed.ctx_fns.iter().enumerate())
        .map(|(k, (id, tf))| (FuncKey::Ctx(*id, k as u32 + 1), id, tf));
    let all: Vec<_> = (typed.fns.iter())
        .map(|(id, tf)| (FuncKey::Decl(*id), id, tf))
        .chain(copies)
        .collect();
    for (key, id, tf) in all {
        let before = lw.errors.len();
        let fd = d.f(*id);
        lw.key = key.clone();
        lw.off = fd.item.clone();
        lw.sites[0] = fd.item.at(Span::default());
        lw.kept = typed.checks;
        lw.made =
            matches!(fd.owner, Some(cpl_next_decls::decls::Owner::Impl(k)) if d.impls[k].derived);
        lw.locals = tf.locals.iter().map(|l| l.ty.clone()).collect();
        lw.fn_locals = lw.locals.clone();
        lw.tnames = tf.locals.iter().map(|l| l.name.clone()).collect();
        // R05.10.1: a closure shares `var`s, local heap values and heap parameters with the fn around it.
        lw.mut_locals = tf
            .locals
            .iter()
            .map(|l| l.mutable || !l.ty.copied(&cpl_next_decls::fields::fields(d)))
            .collect();
        lw.var_locals = tf.locals.iter().map(|l| l.mutable).collect();
        lw.remap.clear();
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
        // A body that ends in `return` or a trap has no value of its own. (The design wants a
        // "never" type here.)
        if ends(&body) {
        } else if tf.ret != Ty::Unit {
            body.push(Stmt::Return(Some(v)));
        } else {
            lw.effect(v, &mut body);
        }
        if typed.quiet.contains(id) && lw.errors.len() > before {
            let es = lw.errors.split_off(before);
            let es = es
                .into_iter()
                .map(|e| e.from_layer(cpl_next_ir::msg::Stage::Lower.name()));
            out.deferred.add(*id, es);
        }
        let (sites, origins) = lw.take_sites();
        out.funcs.insert(
            key.clone(),
            Func {
                key,
                name: fd.name.clone(),
                params,
                n_captures: 0,
                names: lw.names(&lw.tnames, true),
                vars: lw.vars(&lw.var_locals, true),
                locals: std::mem::take(&mut lw.locals),
                body,
                ret: tf.ret.clone(),
                region: tf.region.clone(),
                line: lw.src.program_line(&fd.span),
                sites,
                origins,
                views: Vec::new(),
                drops: Vec::new(),
                maker: None,
                shown: shown_name(d, fd),
                guards: Vec::new(),
                declared: Vec::new(),
                fast: fd.fast.as_deref().map(|n| {
                    Intr::named(n).unwrap_or_else(|| {
                        let name = n.to_string();
                        lw.errors.push(Unbuilt::Intrinsic { name }.into());
                        Intr::RawGet
                    })
                }),
            },
        );
    }
    for id in std::mem::take(&mut lw.intrinsic_values) {
        let fd = d.f(id);
        let Some(op) = fd.intrinsic.and_then(|i| i.op()) else {
            let name = fd.intrinsic.map_or("", |i| i.id()).to_string();
            lw.errors.push(Unbuilt::Intrinsic { name }.into());
            continue;
        };
        let locals: Vec<Ty> = fd.params.iter().map(|p| p.ty.clone()).collect();
        let args = (locals.iter().enumerate())
            .map(|(l, ty)| {
                Arg::Value(Expr {
                    kind: EK::Copy(Place::Local(l)),
                    ty: ty.clone(),
                    site: 0,
                })
            })
            .collect();
        let call = Expr {
            kind: EK::Intr(op, args, 0),
            ty: fd.ret.clone(),
            site: 0,
        };
        let (sites, origins) = fresh_sites();
        let key = FuncKey::Decl(id);
        out.funcs.insert(
            key.clone(),
            Func {
                key,
                name: fd.name.clone(),
                params: (0..locals.len()).map(|l| (l, false)).collect(),
                n_captures: 0,
                names: fd.params.iter().map(|p| p.name.clone()).collect(),
                vars: fd.params.iter().map(|p| p.mutable).collect(),
                locals,
                body: vec![Stmt::Return(Some(call))],
                ret: fd.ret.clone(),
                region: None,
                guards: Vec::new(),
                declared: Vec::new(),
                fast: None,
                line: lw.src.program_line(&fd.span),
                sites,
                origins,
                views: Vec::new(),
                drops: Vec::new(),
                maker: None,
                shown: shown_name(d, fd),
            },
        );
    }
    for f in lw.lambdas.drain(..) {
        out.funcs.insert(f.key.clone(), f);
    }
    out.errors = lw.errors;
    out.untyped = lw.untyped;
    out.globals = lw.globals;
    out
}

/// The traits domain `i` has impl rows of: decls' rows for the domains it declares, the
/// checker's for the rest.
fn dom_impls(d: &Decls, typed: &Typed, i: cpl_next_ir::ty::DomId) -> Vec<cpl_next_ir::ty::TraitId> {
    let impls = cpl_next_decls::decls::Impls {
        items: d,
        rows: &typed.dom_rows,
    };
    let dom = cpl_next_ir::ty::Ty::Dom(i);
    (d.roles.domain.into_iter().chain(d.roles.order))
        .filter(|&tr| impls.trait_impls(tr).any(|(_, im)| im.for_ == dom))
        .collect()
}

fn whole(d: &Decls, typed: &Typed) -> cpl_next_ir::whole::Whole {
    use cpl_next_ir::whole::{Dom, Global, Whole};
    Whole {
        domains: (typed.domains.iter())
            .map(|(&i, x)| {
                (
                    i,
                    Dom {
                        name: x.name.clone(),
                        len: x.len,
                        lo: x.lo,
                        owner: x.owner,
                        counted: x.counted,
                        impls: dom_impls(d, typed, i),
                    },
                )
            })
            .collect(),
        globals: (typed.globals.iter())
            .map(|g| Global {
                name: g.name.clone(),
                top: g.top,
            })
            .collect(),
        program_fns: (0..d.fns.len())
            .map(FnId)
            .filter(|f| d.of_program(d.f(*f).module))
            .collect(),
        ctx_impls: typed.ctx_impls.clone(),
    }
}

impl Lw<'_> {
    /// The receiver type a bound's method is picked by; ownership marks it lent where the
    /// receiver is (`cpl_next_own::lent`).
    fn self_ty(recv: &TArg) -> Ty {
        let (TArg::Value(v) | TArg::Mut(v)) = recv;
        v.ty.clone()
    }
}

fn writes_nothing<T>(s: &Stmt<T>) -> bool {
    fn quiet<T>(e: &Expr<T>) -> bool {
        match &e.kind {
            EK::Intr(i, args, _) => {
                !i.effectful() && args.iter().all(|a| matches!(a, Arg::Value(v) if quiet(v)))
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
    matches!(e.kind, EK::Int(_) | EK::F64(_) | EK::Bool(_) | EK::Unit)
}

impl Lw<'_> {
    /// A new site for the node at `span`, or 0 where the node has no line of the user's file.
    /// A new site for the node at `span`.
    /// The site tables of the fn being lowered, which start again for the next.
    fn take_sites(&mut self) -> (Vec<SourceId>, Vec<u32>) {
        self.swap_sites(fresh_sites())
    }
    fn swap_sites(&mut self, to: (Vec<SourceId>, Vec<u32>)) -> (Vec<SourceId>, Vec<u32>) {
        let origins = std::mem::replace(&mut self.origins, to.1);
        (std::mem::replace(&mut self.sites, to.0), origins)
    }
    /// `a / b` or `a % b` where `div` checks are off: by zero it is 0 (R09.3.6), as IR every
    /// backend runs alike. `b ^ b` is the zero of `b`'s type, and `b` itself the result then.
    fn divided_or_0(
        &mut self,
        op: Intr,
        vs: Vec<Expr<Ty>>,
        site: Site,
        ty: Ty,
        out: &mut Vec<Stmt<Ty>>,
    ) -> Expr<Ty> {
        let mut vs = vs.into_iter();
        let (Some(a), Some(b)) = (vs.next(), vs.next()) else {
            self.errors.push(Unbuilt::Operator.into());
            return Expr {
                site,
                kind: EK::Unit,
                ty,
            };
        };
        let b = self.spill(b, out);
        let ex = |kind, ty: Ty| Expr { site, kind, ty };
        let r = self.temp(ty.clone());
        out.push(Stmt::Let(r, b.clone()));
        let pair = |x: &Expr<Ty>, y: Expr<Ty>| vec![Arg::Value(x.clone()), Arg::Value(y)];
        let zero = ex(EK::Intr(Intr::BitXor, pair(&b, b.clone()), 0), ty.clone());
        let nonzero = ex(EK::Intr(Intr::Ne, pair(&b, zero), 0), Ty::Bool);
        let q = ex(EK::Intr(op, pair(&a, b), site), ty.clone());
        out.push(Stmt::If(
            nonzero,
            vec![Stmt::Assign(Place::Local(r), q)],
            vec![],
        ));
        ex(EK::Copy(Place::Local(r)), ty)
    }
    /// Whether a check of `kind` is made where the code being lowered is (R09.4).
    fn keeps(&self, kind: CheckKind) -> bool {
        kind.name().is_none_or(|n| self.kept.keeps(n))
    }
    fn site(&mut self, span: Span) -> Site {
        let at = self.off.at(span);
        self.origins.push(u32::from(self.user(span)));
        self.sites.push(at);
        (self.sites.len() - 1) as Site
    }
    /// Whether a trap at `span` names it: it is in the user's file and not in a derived impl.
    fn user(&self, span: Span) -> bool {
        !self.made && &*self.off.at(span).item.module == cpl_next_decls::modules::PROGRAM
    }
    fn line(&self, span: Span) -> u32 {
        if self.made {
            return 0;
        }
        self.src.program_line(&self.off.at(span))
    }
    /// What a message calls each local of the fn being lowered: `own` names its first ones, and
    /// with `remap`, a local standing for a binding around it takes that binding's name.
    fn names(&self, own: &[String], remap: bool) -> Vec<String> {
        let mut ns: Vec<String> = (0..self.locals.len())
            .map(|l| own.get(l).cloned().unwrap_or_default())
            .collect();
        for (v, &l) in self.remap.iter().filter(|_| remap) {
            let name = match v {
                VarRef::Local(i) => self.tnames.get(*i),
                VarRef::Global(g) => self.typed.globals.get(*g).map(|g| &g.name),
            };
            if let (Some(n), Some(slot)) = (name, ns.get_mut(l)) {
                n.clone_into(slot);
            }
        }
        ns
    }
    /// Which locals of the fn being lowered are `var`s, as `names` finds their names.
    fn vars(&self, own: &[bool], remap: bool) -> Vec<bool> {
        let mut vs: Vec<bool> = (0..self.locals.len())
            .map(|l| own.get(l).copied().unwrap_or(false))
            .collect();
        for (v, &l) in self.remap.iter().filter(|_| remap) {
            let var = match v {
                VarRef::Local(i) => self.var_locals.get(*i).copied(),
                VarRef::Global(g) => self.typed.globals.get(*g).map(|g| g.mutable),
            };
            if let (Some(var), Some(slot)) = (var, vs.get_mut(l)) {
                *slot = var;
            }
        }
        vs
    }
    fn temp(&mut self, ty: Ty) -> usize {
        self.locals.push(ty);
        self.locals.len() - 1
    }
    fn var(&mut self, v: VarRef) -> Place {
        if let Some(l) = self.remap.get(&v) {
            if self.by_addr.contains(l) {
                return Place::Deref(Box::new(Place::Local(*l)));
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
        let (ty, site) = (e.ty.clone(), e.site);
        out.push(Stmt::Let(t, e));
        Expr {
            site,
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
        self.seq_by(es, out, Self::expr)
    }
    /// `seq`, lowering each with `lower`.
    fn seq_by(
        &mut self,
        es: &[&TExpr],
        out: &mut Vec<Stmt<Ty>>,
        lower: impl Fn(&mut Self, &TExpr, &mut Vec<Stmt<Ty>>) -> Expr<Ty>,
    ) -> Vec<Expr<Ty>> {
        let mut done: Vec<Expr<Ty>> = Vec::new();
        // A binding nothing can write reads the same after any call, so it is not saved.
        let mut fixed: Vec<bool> = Vec::new();
        for e in es {
            let before = out.len();
            let mut tail = Vec::new();
            let v = lower(self, e, &mut tail);
            let v = self.spill(v, &mut tail);
            // Statements that only compute (arithmetic, a check) cannot write what an
            // earlier operand reads, so that operand stays a read of its place and is
            // not copied. A call can, so the operand is saved.
            if !tail.iter().all(writes_nothing) {
                for (d, _) in done.iter_mut().zip(&fixed).filter(|(_, &f)| !f) {
                    // A place that may be viewed stays passed: ownership takes the saved
                    // copy only where the call also writes the place.
                    if let EK::Pass(_, true, _) = d.kind {
                        let t = self.temp(d.ty.clone());
                        out.push(Stmt::Let(t, d.clone()));
                    } else if !constant(d) {
                        let t = self.temp(d.ty.clone());
                        let ty = d.ty.clone();
                        let old = std::mem::replace(
                            d,
                            Expr {
                                site: 0,
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
            fixed.push(matches!(e.kind, TK::Var(VarRef::Local(l))
                if !self.var_locals.get(l).copied().unwrap_or(true)));
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
    fn args(
        &mut self,
        recv: Option<&TArg>,
        args: &[TArg],
        out: &mut Vec<Stmt<Ty>>,
    ) -> Vec<Arg<Ty>> {
        let all: Vec<&TArg> = recv.into_iter().chain(args.iter()).collect();
        let values: Vec<&TExpr> = (all.iter())
            .filter_map(|a| match a {
                TArg::Value(v) => Some(v),
                TArg::Mut(_) => None,
            })
            .collect();
        let mut vals = self.seq_by(&values, out, Self::operand).into_iter();
        let mut lowered = Vec::with_capacity(all.len());
        for a in all {
            match a {
                TArg::Value(_) => {
                    let Some(x) = vals.next() else { continue };
                    lowered.push(Arg::Value(x));
                }
                TArg::Mut(p) => lowered.push(Arg::Lend(self.place(p, out))),
            }
        }
        lowered
    }
    /// A by-value argument. A heap one that is a place is passed (`EK::Pass`): after link,
    /// ownership makes it a view from when the call starts (R11), or a copy in its turn where
    /// the call also writes it (R4). A type parameter could be plain or heap, so it is not viewed.
    fn operand(&mut self, v: &TExpr, out: &mut Vec<Stmt<Ty>>) -> Expr<Ty> {
        let generic = matches!(v.ty, Ty::Param(_));
        let viewable = matches!(v.kind, TK::Var(_) | TK::Field(..) | TK::Payload(..));
        if !(viewable || matches!(v.kind, TK::Slot(..)))
            || !generic && v.ty.copied(&cpl_next_decls::fields::fields(self.d))
        {
            return self.expr(v, out);
        }
        let site = self.site(v.span);
        let p = self.place(v, out);
        Expr {
            site: 0,
            kind: EK::Pass(p, viewable && !generic, site),
            ty: v.ty.clone(),
        }
    }
    /// A struct built from `fs`; `pack` when it is a variadic argument's (`TK::Pack`).
    fn make_struct(
        &mut self,
        e: &TExpr,
        fs: &[TExpr],
        pack: bool,
        out: &mut Vec<Stmt<Ty>>,
    ) -> Expr<Ty> {
        let site = self.site(e.span);
        // A binding nothing can write and a place a pack reads are read where the struct is
        // made, so ownership can store them lent (`cpl_next_own::lent`).
        let held: Vec<bool> = (fs.iter())
            .map(|f| match f.kind {
                TK::Var(VarRef::Local(l)) => {
                    pack || !self.var_locals.get(l).copied().unwrap_or(true)
                }
                _ => pack && is_place(f),
            })
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
                        site,
                        kind: EK::Copy(p),
                        ty: f.ty.clone(),
                    });
                }
            }
        }
        Expr {
            kind: EK::MakeStruct(xs),
            ty: e.ty.clone(),
            site,
        }
    }
    fn expr(&mut self, e: &TExpr, out: &mut Vec<Stmt<Ty>>) -> Expr<Ty> {
        let ty = e.ty.clone();
        let site = self.site(e.span);
        let mk = |kind| Expr {
            kind,
            ty: ty.clone(),
            site,
        };
        match &e.kind {
            TK::Int(i) => mk(EK::Int(*i)),
            TK::Float(b) => mk(EK::F64(*b)),
            TK::Bool(b) => mk(EK::Bool(*b)),
            TK::Unit => mk(EK::Unit),
            // What the checker could not type traps where it runs (contract 4); the checker
            // said why, and a build that reaches it is not clean.
            TK::Error => {
                (self.untyped).push(self.off.at(e.span));
                out.push(Stmt::Check {
                    kind: CheckKind::Untyped,
                    cond: mk(EK::Bool(false)),
                    msg: "a part with a type error ran".into(),
                    site,
                });
                mk(EK::Unit)
            }
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
                // A trait's intrinsic fn was checked into a body (`Domain`'s), called as any fn.
                let inline = fd
                    .intrinsic
                    .filter(|_| !self.typed.fns.contains_key(&f.decl));
                let call = if let Some(n) = inline {
                    let op = n.op().unwrap_or_else(|| {
                        self.errors.push(
                            Unbuilt::Intrinsic {
                                name: n.id().to_string(),
                            }
                            .into(),
                        );
                        Intr::RawGet
                    });
                    mk(EK::Intr(op, a, site))
                } else {
                    Expr {
                        site,
                        kind: EK::Call(Target::Fn(f.clone()), a, site),
                        ty: ty.clone(),
                    }
                };
                self.spill(call, out)
            }
            TK::CallTrait {
                tr,
                method,
                recv,
                self_ty,
                targs,
                sets,
                args,
                ctx,
            } => {
                let a = self.args(recv.as_deref(), args, out);
                let self_ty = match recv {
                    Some(r) => Self::self_ty(r),
                    None => (**self_ty).clone(),
                };
                let call = mk(EK::Call(
                    Target::Trait {
                        tr: tr.tr,
                        args: tr.args.clone(),
                        method: method.clone(),
                        self_ty,
                        targs: targs.to_vec(),
                        sets: sets.to_vec(),
                        ret: e.ty.clone(),
                        ctx: *ctx,
                    },
                    a,
                    site,
                ));
                self.spill(call, out)
            }
            TK::CallValue { callee, args } => {
                let f = TArg::Value((**callee).clone());
                let mut vs = self.args(Some(&f), args, out).into_iter();
                let Some(Arg::Value(c)) = vs.next() else {
                    self.errors
                        .push(Msg::internal("a called function value lowers to no value"));
                    return mk(EK::Unit);
                };
                let call = mk(EK::CallValue(Box::new(c), vs.collect(), site));
                self.spill(call, out)
            }
            TK::Struct(fs) => self.make_struct(e, fs, false, out),
            TK::Pack(x) => match &x.kind {
                TK::Struct(fs) => self.make_struct(x, fs, true, out),
                _ => self.expr(x, out),
            },
            TK::FnItem(f) => {
                let intrinsic = self.d.f(f.decl).intrinsic.is_some();
                if intrinsic
                    && !self.typed.fns.contains_key(&f.decl)
                    && !self.intrinsic_values.contains(&f.decl)
                {
                    self.intrinsic_values.push(f.decl);
                }
                mk(EK::MakeClosure(vec![]))
            }
            TK::Itself(id) => match self.selves.last() {
                Some((group, caps)) if group.contains(id) => mk(EK::MakeClosure(caps.clone())),
                // A nested fn used above its statement: made here from what its group reads, all
                // of it set by now (R06.1.7).
                _ if self.group_free.contains_key(id) => {
                    let vars = self.group_free[id].clone();
                    let caps = self.captured(vars.iter());
                    mk(EK::MakeClosure(Self::capture_args(caps)))
                }
                _ => {
                    (self.errors).push(Unbuilt::LambdaNamesOuter.into());
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
            // A value read before any place is written (R01.6.4): a copy taken here, never a
            // view, since the writes that follow change its place. A call's heap result may
            // lend its arguments, so it is saved as a copy of the temp that holds it.
            TK::Owned(x) => match self.operand(x, out) {
                Expr {
                    kind: EK::Pass(p, true, s),
                    site,
                    ty,
                } => Expr {
                    kind: EK::Pass(p, false, s),
                    site,
                    ty,
                },
                v @ Expr {
                    kind: EK::Call(..) | EK::Copy(_),
                    ..
                } if !x.ty.copied(&cpl_next_decls::fields::fields(self.d)) => {
                    let (ty, site) = (v.ty.clone(), v.site);
                    let p = match v.kind {
                        EK::Copy(p) => p,
                        _ => {
                            let t = self.temp(ty.clone());
                            out.push(Stmt::Let(t, v));
                            Place::Local(t)
                        }
                    };
                    let saved = self.temp(ty.clone());
                    let kind = EK::Pass(p, false, site);
                    let copy = Expr {
                        site: 0,
                        kind,
                        ty: ty.clone(),
                    };
                    out.push(Stmt::Let(saved, copy));
                    Expr {
                        site,
                        kind: EK::Copy(Place::Local(saved)),
                        ty,
                    }
                }
                v => v,
            },
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
                    site,
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
                        self.errors.push(Unbuilt::Operator.into());
                        Intr::Add
                    }
                };
                // Where a check is off, the operation is the one defining its result (R09.3).
                let i = match e.ty == Ty::Int && !self.kept.keeps(CheckName::Overflow) {
                    true => i.wrapping(),
                    false => i,
                };
                let unchecked_div = cpl_next_ir::ops::divides(i)
                    && e.ty != Ty::F64
                    && !self.kept.keeps(CheckName::Div);
                if unchecked_div {
                    self.divided_or_0(i, vs, site, e.ty.clone(), out)
                } else {
                    let v = mk(EK::Intr(i, vs.into_iter().map(Arg::Value).collect(), site));
                    self.spill(v, out)
                }
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
            TK::ToI128(x) => {
                let v = self.expr(x, out);
                mk(EK::Intr(Intr::ToI128, vec![Arg::Value(v)], site))
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
                // A bound test's branches: each lambda in one records which it is in.
                let n = self.guards.len();
                if let TK::HasBound(t, r) = &c.kind {
                    self.guards.push((t.clone(), r.tr, r.args.clone(), true));
                }
                let va = self.expr(a, &mut sa);
                if let Some(g) = self.guards.get_mut(n) {
                    g.3 = false;
                }
                let vb = self.expr(b, &mut sb);
                self.guards.truncate(n);
                if ty == Ty::Unit {
                    self.effect(va, &mut sa);
                    self.effect(vb, &mut sb);
                    out.push(Stmt::If(cv, sa, sb));
                    mk(EK::Unit)
                } else {
                    let t = self.temp(ty.clone());
                    // A branch of no value in an `if` of one leaves, so it has nothing to store.
                    if a.ty != Ty::Unit {
                        sa.push(Stmt::Assign(Place::Local(t), va));
                    }
                    if b.ty != Ty::Unit {
                        sb.push(Stmt::Assign(Place::Local(t), vb));
                    }
                    out.push(Stmt::If(cv, sa, sb));
                    mk(EK::Copy(Place::Local(t)))
                }
            }
            TK::Block(ss, v) => {
                self.group_free
                    .extend(group_free(&self.typed.lambdas, self.in_main, ss));
                let kept = self.kept;
                for s in ss {
                    self.stmt(s, out);
                }
                let v = match v {
                    Some(v) => self.expr(v, out),
                    None => mk(EK::Unit),
                };
                self.kept = kept;
                v
            }
            TK::DomLen(t) => mk(EK::DomLen(t.clone())),
            TK::DomCounted(t) => mk(EK::DomCounted(t.clone())),
            TK::HasBound(t, r) => mk(EK::HasBound(t.clone(), r.tr, r.args.clone())),
            TK::Ordinal(x) => {
                let v = self.expr(x, out);
                let kind = match held(&x.ty) {
                    Some(h) => EK::Intr(Intr::Sub, vec![Arg::Value(v), Arg::Value(h)], 0),
                    None => v.kind,
                };
                Expr {
                    site,
                    kind,
                    ty: Ty::Int,
                }
            }
            // A position is held as its ordinal; its integer is that plus the domain's start.
            TK::DomOrd(x) => {
                let v = self.expr(x, out);
                let int = |kind| Expr {
                    site,
                    kind,
                    ty: Ty::Int,
                };
                if matches!(x.ty, Ty::Int | Ty::Error) {
                    return int(v.kind);
                }
                // A parameter may be a domain that is a struct, as `Cut<D>`: its `ord` says it.
                let v = match (&x.ty, self.d.roles.domain) {
                    (Ty::Param(_), Some(tr)) => {
                        let ord = Target::Trait {
                            tr,
                            args: Vec::new(),
                            method: "ord".to_string(),
                            self_ty: x.ty.clone(),
                            ret: Ty::Int,
                            targs: Vec::new(),
                            sets: Vec::new(),
                            ctx: 0,
                        };
                        self.spill(int(EK::Call(ord, vec![Arg::Value(v)], site)), out)
                    }
                    _ => v,
                };
                let lo = Arg::Value(int(EK::DomLo(x.ty.clone())));
                let sum = int(EK::Intr(Intr::Add, vec![Arg::Value(int(v.kind)), lo], 0));
                self.spill(sum, out)
            }
            TK::DomFrom(t, x, kind) => {
                let (kind, keep) = (*kind, self.keeps(*kind));
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                let int = |kind| Expr {
                    site,
                    kind,
                    ty: Ty::Int,
                };
                let op = |op, a: Expr<Ty>, b: Expr<Ty>, ty: Ty| Expr {
                    site,
                    kind: EK::Intr(op, vec![Arg::Value(a), Arg::Value(b)], 0),
                    ty,
                };
                let check = |cond: Expr<Ty>, out: &mut Vec<Stmt<Ty>>| {
                    if !keep {
                        return;
                    }
                    out.push(Stmt::Check {
                        kind,
                        cond,
                        msg: match kind {
                            CheckKind::Key => "key outside the array",
                            _ => "value outside the domain",
                        }
                        .into(),
                        site,
                    })
                };
                let (lo, len) = (int(EK::DomLo(t.clone())), int(EK::DomLen(t.clone())));
                // With a position in the domain and `v` at or after the first, none of
                // the arithmetic below can overflow. The last position is worked out inside
                // its check, so a check the facts prove takes that arithmetic with it.
                check(op(Intr::Lt, int(EK::Int(0)), len.clone(), Ty::Bool), out);
                check(op(Intr::Ge, v.clone(), lo.clone(), Ty::Bool), out);
                let last = op(Intr::Sub, len, int(EK::Int(1)), Ty::Int);
                let last = op(Intr::Add, lo.clone(), last, Ty::Int);
                check(op(Intr::Le, v.clone(), last, Ty::Bool), out);
                let ord = op(Intr::Sub, v, lo, Ty::Int);
                let ord = match held(t) {
                    Some(h) => op(Intr::Add, ord, h, Ty::Int),
                    None => ord,
                };
                let ord = self.spill(ord, out);
                Expr {
                    site,
                    kind: ord.kind,
                    ty,
                }
            }
            TK::CutOrd(t, x) => {
                let v = self.expr(x, out);
                let int = |kind| Expr {
                    site,
                    kind,
                    ty: Ty::Int,
                };
                let op = |op, a: Expr<Ty>, b: Expr<Ty>, ty: Ty| Expr {
                    site,
                    kind: EK::Intr(op, vec![Arg::Value(a), Arg::Value(b)], 0),
                    ty,
                };
                let ord = op(Intr::Sub, v, int(EK::DomLo(t.clone())), Ty::Int);
                let ord = self.spill(ord, out);
                let within = [
                    op(Intr::Ge, ord.clone(), int(EK::Int(0)), Ty::Bool),
                    op(Intr::Le, ord.clone(), int(EK::DomLen(t.clone())), Ty::Bool),
                ];
                for cond in within.into_iter().filter(|_| self.keeps(CheckKind::Domain)) {
                    out.push(Stmt::Check {
                        kind: CheckKind::Domain,
                        cond,
                        msg: "cut outside the domain".into(),
                        site,
                    });
                }
                ord
            }
            TK::DomHas(t, x) => {
                let v = self.expr(x, out);
                let v = self.spill(v, out);
                let int = |kind| Expr {
                    site,
                    kind,
                    ty: Ty::Int,
                };
                let op = |op, a: Expr<Ty>, b: Expr<Ty>, ty: Ty| Expr {
                    site,
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
                    site,
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
                    site,
                    kind: EK::DomLen(t.clone()),
                    ty: Ty::Int,
                };
                let cond = Expr {
                    site,
                    kind: EK::Intr(Intr::Below, vec![Arg::Value(v.clone()), Arg::Value(len)], 0),
                    ty: Ty::Bool,
                };
                if self.keeps(CheckKind::Domain) {
                    out.push(Stmt::Check {
                        kind: CheckKind::Domain,
                        cond,
                        msg: "a position past the domain's end".into(),
                        site,
                    });
                }
                let kind = match held(t) {
                    Some(h) => EK::Intr(Intr::Add, vec![Arg::Value(v), Arg::Value(h)], 0),
                    None => v.kind,
                };
                Expr { site, kind, ty }
            }
            TK::ReadInt => {
                let v = mk(EK::Intr(Intr::ReadInt, vec![], site));
                self.spill(v, out)
            }
        }
    }
    fn stmt(&mut self, s: &TStmt, out: &mut Vec<Stmt<Ty>>) {
        /// A value of `lo..=hi` stored at `p` is in it, however it was made (R03.1.4).
        fn held(ty: &Ty, p: Place, out: &mut Vec<Stmt<Ty>>) {
            let &Ty::Range(lo, hi) = ty else { return };
            let x = || Expr {
                site: 0,
                kind: EK::Copy(p.clone()),
                ty: Ty::Int,
            };
            let int = |v| Expr {
                site: 0,
                kind: EK::Int(v),
                ty: Ty::Int,
            };
            for (a, b) in [(int(lo), x()), (x(), int(hi))] {
                let le = EK::Intr(Intr::Le, vec![Arg::Value(a), Arg::Value(b)], 0);
                out.push(Stmt::Fact(Expr {
                    site: 0,
                    kind: le,
                    ty: Ty::Bool,
                }));
            }
        }
        match s {
            TStmt::Let(v, e) => {
                let val = self.expr(e, out);
                let place = self.bind(*v);
                match place.clone() {
                    Place::Local(l) => out.push(Stmt::Let(l, val)),
                    p => out.push(Stmt::Assign(p, val)),
                }
                held(&e.ty, place, out);
            }
            TStmt::Assign(p, e) => {
                let val = self.expr(e, out);
                let val = self.spill(val, out);
                let place = self.place(p, out);
                out.push(Stmt::Assign(place.clone(), val));
                held(&e.ty, place, out);
            }
            TStmt::Expr(e) => {
                let v = self.expr(e, out);
                self.effect(v, out);
            }
            TStmt::While(c, b) => {
                let mut body = Vec::new();
                let cv = self.expr(c, &mut body);
                if !matches!(cv.kind, EK::Bool(true)) {
                    body.push(Stmt::If(cv, vec![], vec![Stmt::Break]));
                }
                let v = self.expr(b, &mut body);
                self.effect(v, &mut body);
                let at = self.site(b.span);
                out.push(Stmt::Loop(body, vec![], at));
            }
            TStmt::For(v, lo, hi, b) => {
                // `for i in lo..hi`: both ends are evaluated once, before the loop.
                let lov = self.expr(lo, out);
                let var = self.bind(*v);
                // The start, kept for the fact that the variable is at or after it.
                let start = match lov.kind {
                    EK::Int(_) => lov,
                    _ => {
                        let l = self.temp(Ty::Int);
                        out.push(Stmt::Let(l, lov));
                        Expr {
                            site: 0,
                            kind: EK::Copy(Place::Local(l)),
                            ty: Ty::Int,
                        }
                    }
                };
                out.push(Stmt::Assign(var.clone(), start.clone()));
                let hiv = self.expr(hi, out);
                let h = self.temp(Ty::Int);
                out.push(Stmt::Let(h, hiv));
                let read = |p: Place| Expr {
                    site: 0,
                    kind: EK::Copy(p),
                    ty: Ty::Int,
                };
                let cond = Expr {
                    site: 0,
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
                let at_start = Expr {
                    site: 0,
                    kind: EK::Intr(
                        Intr::Le,
                        vec![Arg::Value(start), Arg::Value(read(var.clone()))],
                        0,
                    ),
                    ty: Ty::Bool,
                };
                let mut body = vec![
                    Stmt::If(cond, vec![], vec![Stmt::Break]),
                    Stmt::Fact(at_start),
                ];
                let bv = self.expr(b, &mut body);
                self.effect(bv, &mut body);
                // The step cannot overflow: the variable is below `hi`.
                let step = Expr {
                    site: 0,
                    kind: EK::Intr(
                        Intr::Add,
                        vec![
                            Arg::Value(read(var.clone())),
                            Arg::Value(Expr {
                                site: 0,
                                kind: EK::Int(1),
                                ty: Ty::Int,
                            }),
                        ],
                        0,
                    ),
                    ty: Ty::Int,
                };
                let at = self.site(b.span);
                out.push(Stmt::Loop(body, vec![Stmt::Assign(var, step)], at));
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
                if self.keeps(*kind) {
                    let site = self.site(*span);
                    out.push(Stmt::Check {
                        kind: *kind,
                        cond: cv,
                        msg: msg.clone(),
                        site,
                    });
                }
            }
            TStmt::Break => out.push(Stmt::Break),
            TStmt::Item(off) => self.off = off.clone(),
            TStmt::Checks(kept) => self.kept = *kept,
            TStmt::Cases(count, body) => {
                let mk = |kind, ty| Expr { site: 0, kind, ty };
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
                            0,
                        ));
                    }
                    None => {
                        let more = mk(EK::Intr(Intr::InputLeft, vec![], 0), Ty::Bool);
                        out.push(Stmt::Loop(
                            vec![Stmt::If(more, vec![], vec![Stmt::Break]), call],
                            vec![],
                            0,
                        ));
                    }
                }
                let (outer, off) = (std::mem::take(&mut self.locals), self.off.clone());
                let outer_sites = self.take_sites();
                let mut b = Vec::new();
                let outer_key = std::mem::replace(&mut self.key, FuncKey::Case);
                body.iter().for_each(|s| self.stmt(s, &mut b));
                self.key = outer_key;
                let (sites, origins) = self.swap_sites(outer_sites);
                self.case = Some(Func {
                    key: FuncKey::Case,
                    name: "case".into(),
                    params: vec![],
                    n_captures: 0,
                    names: self.names(&[], false),
                    vars: self.vars(&[], false),
                    locals: std::mem::replace(&mut self.locals, outer),
                    body: b,
                    ret: Ty::Unit,
                    region: None,
                    line: 0,
                    sites,
                    origins,
                    views: Vec::new(),
                    drops: Vec::new(),
                    maker: None,
                    shown: "case".into(),
                    guards: Vec::new(),
                    declared: Vec::new(),
                    fast: None,
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
                // Inside a lambda it is already one of the lambda's own locals, numbered there.
                let remapped = self.remap.get(var).copied();
                let ty = match (var, remapped) {
                    (_, Some(r)) => self.locals[r].clone(),
                    (VarRef::Local(l), None) if !self.in_main => self.locals[*l].clone(),
                    (VarRef::Local(l), None) => self.locals.get(*l).cloned().unwrap_or(Ty::Error),
                    (VarRef::Global(g), None) => self.globals[*g].clone(),
                };
                let l = match (var, remapped) {
                    (_, Some(r)) => r,
                    (VarRef::Local(l), None) if !self.in_main => *l,
                    _ => {
                        let l = self.temp(ty.clone());
                        self.remap.insert(*var, l);
                        l
                    }
                };
                let body_site = self.site(body.span);
                let mut b = Vec::new();
                let v = self.expr(body, &mut b);
                self.effect(v, &mut b);
                let target = match (callee, recv.as_deref()) {
                    (Callee::Fn(f), _) => Target::Fn(f.clone()),
                    (Callee::Value(c), _) => {
                        let v = self.expr(c, out);
                        Target::Value(Box::new(v))
                    }
                    (Callee::Trait(tr, method), r) => Target::Trait {
                        tr: tr.tr,
                        args: tr.args.clone(),
                        method: method.clone(),
                        self_ty: r.map_or(Ty::Error, Self::self_ty),
                        targs: Vec::new(),
                        sets: Vec::new(),
                        ret: Ty::Unit,
                        ctx: 0,
                    },
                };
                out.push(Stmt::Region {
                    target,
                    args: a,
                    var: l,
                    lend: *lend,
                    body: b,
                    site,
                    body_site,
                });
            }
        }
    }
    /// Lifts a lambda to a function of its own. Returns the locals it captures, by value.
    fn lambda(&mut self, id: LambdaId, params: &[usize], body: &TExpr) -> Vec<Expr<Ty>> {
        let info = &self.typed.lambdas[&id];
        let mut used = BTreeSet::new();
        let mut bound: BTreeSet<VarRef> = BTreeSet::new();
        free_vars(body, self.in_main, &mut used, &mut bound);
        used.extend(self.group_free.get(&id).into_iter().flatten().copied());
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
        let caps = self.captured(
            used.iter()
                .filter(|v| !bound.contains(v) && !param_refs.contains(v)),
        );
        // The lambda's body is lowered with its own local table: captures first, then parameters.
        let outer_locals = std::mem::take(&mut self.locals);
        let outer_by_addr = std::mem::take(&mut self.by_addr);
        let outer_sites = self.take_sites();
        let mut fparams = Vec::new();
        let mut remap = HashMap::new();
        for (key, _, ty, cap) in &caps {
            let l = self.temp(ty.clone());
            if *cap != Cap::Value {
                self.by_addr.insert(l);
            }
            remap.insert(*key, l);
            fparams.push((l, false));
        }
        for (p, pty) in param_refs.iter().zip(&info.params) {
            let (pty, lent) = match pty {
                Ty::Mut(t) => (&**t, true),
                t => (t, false),
            };
            let l = self.temp(pty.clone());
            remap.insert(*p, l);
            fparams.push((l, lent));
        }
        // Locals bound inside the body keep the numbers the checker gave them, in a fn or at top level.
        for b in &bound {
            let ty = match b {
                VarRef::Local(i) => match self.fn_locals.get(*i) {
                    Some(t) => t.clone(),
                    None => {
                        self.errors.push(Unbuilt::LambdaMakesLambda.into());
                        Ty::Error
                    }
                },
                VarRef::Global(i) => match self.typed.globals.get(*i) {
                    Some(g) => g.ty.clone(),
                    None => {
                        self.errors.push(Unbuilt::LambdaMakesLambda.into());
                        Ty::Error
                    }
                },
            };
            let l = self.temp(ty);
            remap.insert(*b, l);
        }
        let itself = (caps.iter())
            .map(|(key, _, ty, _)| Expr {
                site: 0,
                kind: EK::Copy(Place::Local(remap[key])),
                ty: ty.clone(),
            })
            .collect();
        let group = if info.group.is_empty() {
            vec![id]
        } else {
            info.group.clone()
        };
        self.selves.push((group, itself));
        self.remap = remap;
        let was_main = self.in_main;
        let mut stmts = Vec::new();
        let v = self.expr(body, &mut stmts);
        self.selves.pop();
        if ends(&stmts) {
        } else if info.ret != Ty::Unit {
            stmts.push(Stmt::Return(Some(v)));
        } else {
            self.effect(v, &mut stmts);
        }
        self.in_main = was_main;
        let names = self.names(&[], true);
        let vars = self.vars(&[], true);
        let locals = std::mem::replace(&mut self.locals, outer_locals);
        self.by_addr = outer_by_addr;
        self.remap = saved_remap;
        let (sites, origins) = self.swap_sites(outer_sites);
        self.lambdas.push(Func {
            key: FuncKey::Lambda(id),
            name: match id.owner {
                Some(f) => format!("lambda{}_{}_{}", f.0, id.copy, id.n),
                None => format!("lambda_{}_{}", id.copy, id.n),
            },
            params: fparams,
            n_captures: caps.len(),
            names,
            vars,
            locals,
            body: stmts,
            ret: info.ret.clone(),
            region: info.yields.clone().map(|t| (false, t)),
            line: self.line(body.span),
            sites,
            origins,
            views: Vec::new(),
            drops: Vec::new(),
            maker: Some(self.key.clone()),
            shown: "lambda".into(),
            guards: self.guards.clone(),
            declared: Vec::new(),
            fast: None,
        });
        Self::capture_args(caps)
    }
    /// How a closure holds each variable it reads from the fn around it: the place, its type,
    /// and whether it holds the value or its address.
    fn captured<'v>(
        &self,
        vars: impl Iterator<Item = &'v VarRef>,
    ) -> Vec<(VarRef, Place, Ty, Cap)> {
        // What the lambda reads from the fn around it: locals not bound inside it.
        // A binding inside a block is captured by value when the closure is made, at top level too:
        // a loop makes a new one each time round. Only the file's own top-level bindings are shared.
        // A `var` and a top-level binding are shared with the closure (spec R05.10.1): the capture
        // holds its address, and ownership decides whether that is a lend, a reference or a copy.
        let mut_locals = &self.mut_locals;
        let is_var = |v: &VarRef| match v {
            VarRef::Local(i) => mut_locals.get(*i).copied().unwrap_or(false),
            VarRef::Global(_) => true,
        };
        let by_ref = |v: &VarRef, p: Place, ty: Ty| -> (VarRef, Place, Ty, Cap) {
            let cap = if matches!(p, Place::Local(l) if self.by_addr.contains(&l)) {
                Cap::Ptr
            } else if is_var(v) {
                // Ownership decides how it holds it: lent where nothing can reassign it, else
                // by reference (`cpl_next_own::lent`).
                Cap::Addr
            } else {
                Cap::Value
            };
            (*v, p, ty, cap)
        };
        vars.filter_map(|v| match (v, self.remap.get(v)) {
            (_, Some(l)) => Some(by_ref(v, Place::Local(*l), self.locals[*l].clone())),
            (VarRef::Local(l), None) => Some(by_ref(v, Place::Local(*l), self.locals[*l].clone())),
            (VarRef::Global(g), None) if !self.typed.globals[*g].top => Some(by_ref(
                v,
                Place::Global(*g),
                self.typed.globals[*g].ty.clone(),
            )),
            (VarRef::Global(_), None) => None,
        })
        .collect()
    }
    /// The values a closure is made from: each capture read, or its address taken.
    fn capture_args(caps: Vec<(VarRef, Place, Ty, Cap)>) -> Vec<Expr<Ty>> {
        caps.into_iter()
            .map(|(_, p, ty, cap)| Expr {
                site: 0,
                kind: if cap == Cap::Addr {
                    EK::AddrOf(p)
                } else {
                    EK::Copy(p)
                },
                ty,
            })
            .collect()
    }
}

/// How a closure holds a capture: its value, the address of a binding of the fn around it, or
/// the address a capture of that fn already holds.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Cap {
    Value,
    Addr,
    Ptr,
}

/// What a trap's call stack calls a declared fn: its name after its owner's, `Box.at`.
fn shown_name(d: &Decls, fd: &FnDecl) -> String {
    let owner = match &fd.owner {
        Some(Owner::Struct(s)) => Some(&d.structs[s.0].name),
        Some(Owner::Trait(t)) => Some(&d.traits[t.0].name),
        Some(Owner::Impl(i)) => match &d.impls[*i].for_ {
            Ty::Struct(s, ..) => Some(&d.structs[s.0].name),
            _ => None,
        },
        None => None,
    };
    match owner {
        Some(o) => format!("{o}.{}", fd.name),
        None => fd.name.clone(),
    }
}

/// The statements never run past their end: they end in a `return`, a check that always fails,
/// or a branch each of whose ways ends.
fn ends(stmts: &[Stmt<Ty>]) -> bool {
    match stmts.last() {
        Some(Stmt::Return(_)) => true,
        Some(Stmt::Check { cond, .. }) => matches!(cond.kind, EK::Bool(false)),
        Some(Stmt::If(_, a, b)) => ends(a) && ends(b),
        Some(Stmt::Switch(_, arms)) => !arms.is_empty() && arms.iter().all(|a| ends(a)),
        // A loop nothing breaks out of, as `while true` is, only leaves by `return`.
        Some(Stmt::Loop(body, step, _)) => !breaks(body) && !breaks(step),
        _ => false,
    }
}

/// Whether a `break` in `stmts` can leave the loop around them.
fn breaks(stmts: &[Stmt<Ty>]) -> bool {
    stmts.iter().any(|s| match s {
        Stmt::Break => true,
        Stmt::If(_, a, b) => breaks(a) || breaks(b),
        Stmt::Switch(_, arms) => arms.iter().any(|a| breaks(a)),
        Stmt::Region { body, .. } => breaks(body),
        _ => false,
    })
}

/// The place of what no item holds: the program's file as a whole.
fn nowhere() -> SourceId {
    let program = cpl_next_decls::modules::PROGRAM;
    ItemId::file(FileId::new(program), program.into()).at(Span::default())
}

/// A fn's site tables before its first site: site 0, which has no place.
/// Site 0 is the fn as a whole: where its calls enter and leave, and where compiler-made code is.
fn fresh_sites() -> (Vec<SourceId>, Vec<u32>) {
    (vec![nowhere()], vec![0])
}

/// Whether `e` names a place: a variable, or a field of one.
fn is_place(e: &TExpr) -> bool {
    match &e.kind {
        TK::Var(_) => true,
        TK::Field(b, _) => is_place(b),
        _ => false,
    }
}

/// What a position of `t` is held as, less its ordinal, when it may not be 0: a constant range's
/// first integer, or a type parameter's, which link knows (`EK::DomHeld`).
fn held(t: &Ty) -> Option<Expr<Ty>> {
    let kind = match t {
        Ty::Param(_) => EK::DomHeld(t.clone()),
        Ty::Range(lo, _) if *lo != 0 => EK::Int(*lo),
        _ => return None,
    };
    Some(Expr {
        site: Default::default(),
        kind,
        ty: Ty::Int,
    })
}
