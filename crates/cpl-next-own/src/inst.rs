//! Ownership on link's instances: what each reads, writes and holds, and which passed places are copies.
//! Each instance is walked once its callees are summarized: what it reads, writes and holds
//! of its parameters and of globals (`Fx`), and which parameters it keeps. A call names its
//! instance (`Target::Inst`), and a call through a fn value calls each closure of its set, so
//! nothing here widens a set or folds a trait's impls.
use crate::lends;
use crate::{overlap, Fx, Pl, Root, Step};
use cpl_diag::{codes, Code};
use cpl_next_decls::decls::{Owner, SelfMode};
use cpl_next_decls::diag::Msg;
use cpl_next_decls::views::Signatures;
use cpl_next_ir::ir::{Arg, Expr, Place, Stmt, Target, EK};
use cpl_next_ir::ir::{Func, FuncKey, Site};
use cpl_next_ir::prog::{CTy, Holds, InstKey, Program};
use cpl_next_ir::ty::{Subst, Ty};
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};

/// What one instance does to its callers' places, by parameter position.
#[derive(Clone, Default, PartialEq)]
pub(crate) struct Summary {
    pub fx: Fx,
    pub keeps: BTreeSet<usize>,
}

pub(crate) struct Walk<'a> {
    pub d: &'a dyn Signatures,
    pub p: &'a Program,
    /// What a message calls each global.
    /// Each global's name, and whether it is a `var`.
    pub globals: &'a [(String, bool)],
    pub sums: HashMap<usize, Summary>,
    /// The instance being walked.
    pub cur: usize,
    /// Each local's provenance: the places its value lends (R6).
    pub lent: HashMap<Root, BTreeSet<Pl>>,
    pub moved: BTreeSet<Pl>,
    /// Block nesting while walking, and each binding's scope path and order.
    pub scopes: Vec<u32>,
    pub next: u32,
    pub decl: HashMap<Root, (Vec<u32>, u32)>,
    /// Bindings whose value the caller lent (heap parameters).
    pub borrowed: BTreeSet<Root>,
    /// Temps bound to an integer literal: a cell lent at one is that element (R05.8.2).
    pub consts: HashMap<usize, i64>,
    /// What each binding of a heap value borrows: written while it is used is E0602 (a fn
    /// value's captures are not frozen, R10).
    pub frozen: HashMap<Root, BTreeSet<Pl>>,
    /// A write to what a binding borrows, said if that binding is used later (E0602).
    pub pending: Vec<(Root, u32, Code, String)>,
    /// Parameters of a type parameter's type the fn stores where they outlive the call.
    pub kept: BTreeSet<usize>,
    /// The locals holding one of this fn's fn-valued parameters, which it can call but not keep.
    pub closure_params: BTreeSet<usize>,
    /// A local bound to a closure: its instance and the places it captures, in order.
    pub caps: HashMap<Root, (usize, Vec<Place>)>,
    /// A temp the lowering spilled a whole binding into, and that binding.
    pub spilled: HashMap<Root, Pl>,
    /// Temps holding an element read out by a call that declares no `lends`.
    pub elems: BTreeSet<Root>,
    /// Temps holding storage a call made, owned by no binding: they die with their statement.
    pub made: BTreeSet<Root>,
    /// The instances that build a literal the program wrote.
    pub literals: BTreeSet<usize>,
    /// The instances that are methods: what their result views is cut from the receiver.
    pub receivers: BTreeSet<usize>,
    /// The instances whose result can hold storage the call made: its type still holds storage
    /// with each type parameter taken to hold nothing. A `Slice<S>` holds only the `S` it is
    /// given, so one cut from a literal is the literal's, not a temporary (R07.5.10).
    pub makers: BTreeSet<usize>,
    pub copies: BTreeSet<(usize, Site)>,
    pub last: crate::Last,
    pub report: bool,
    pub errors: Vec<Msg>,
    /// Each binding's sources where the reporting walk binds it (`cpl-next --dump-own`).
    pub binds: Vec<Bound>,
}

impl Walk<'_> {
    /// The place an IR place names, as own's path: fields and literal keys.
    pub fn pl(&self, p: &Place) -> Option<Pl> {
        match p {
            Place::Local(l) => Some(Pl::whole(Root::Local(*l))),
            Place::Global(g) => Some(Pl::whole(Root::Global(*g))),
            Place::Field(b, i) | Place::Variant(b, i) => {
                let mut q = self.pl(b)?;
                q.path.push(Step::Field(*i));
                Some(q)
            }
            Place::Slot(b, k) => {
                let mut q = self.pl(b)?;
                if let Some(c) = self.consts.get(k) {
                    q.path.push(Step::Key(*c));
                }
                Some(q)
            }
            Place::Deref(b) | Place::Through(b) => self.pl(b),
        }
    }
    /// The instances a call can run: its target, or each closure of a fn value's set.
    pub fn callees(&self, t: &Target<CTy>, f: Option<&Expr<CTy>>) -> Vec<usize> {
        let mut cs = match (t, f.map(|e| &e.ty)) {
            (Target::Inst(i), _) => vec![*i],
            (_, Some(CTy::Fn(_, _, s))) => s
                .iter()
                .filter_map(|k| self.p.index.get(k).copied())
                .collect(),
            (_, Some(CTy::Clo(k))) => self.p.index.get(k).copied().into_iter().collect(),
            _ => vec![],
        };
        // After a link error, an instance may be named but never built: there is nothing to walk.
        cs.retain(|i| *i < self.p.insts.len());
        cs
    }
    /// Instances in callee-first order; a recursive group is walked until its summaries stop
    /// growing (they only grow, so this ends).
    pub fn order(&self) -> Vec<usize> {
        fn visit(w: &Walk, i: usize, seen: &mut Vec<bool>, out: &mut Vec<usize>) {
            if std::mem::replace(&mut seen[i], true) {
                return;
            }
            let mut cs = Vec::new();
            calls_in(&w.p.insts[i].body, &mut |t, f| cs.extend(w.callees(t, f)));
            for c in cs {
                visit(w, c, seen, out);
            }
            out.push(i);
        }
        let (mut seen, mut out) = (vec![false; self.p.insts.len()], Vec::new());
        for i in 0..self.p.insts.len() {
            visit(self, i, &mut seen, &mut out);
        }
        out
    }
}

impl Walk<'_> {
    fn f(&self) -> &cpl_next_ir::ir::Func<CTy> {
        &self.p.insts[self.cur]
    }
    /// R1, with a fn value plain: what it lends are its captures, held to R05.10.2 apart.
    /// A local the lowering made: it lives for one statement, holding what it was given.
    /// Binds `r` to `e`; if `e` is a closure, records its captures and returns the file
    /// body's bindings it reaches as globals (R05.10.2).
    fn closure_bound(&mut self, r: Root, e: &Expr<CTy>) -> BTreeSet<Pl> {
        let Some(c) = self.closure_caps(e) else {
            return BTreeSet::new();
        };
        let mut uses = BTreeSet::new();
        if let (true, Some(sum)) = (self.cur == self.p.main, self.sums.get(&c.0)) {
            let g = |p: &&Pl| matches!(p.root, Root::Global(_));
            uses.extend(sum.fx.reads.iter().chain(&sum.fx.writes).filter(g).cloned());
        }
        self.caps.insert(r, c);
        uses
    }
    fn closure_caps(&self, e: &Expr<CTy>) -> Option<(usize, Vec<Place>)> {
        match &e.kind {
            EK::Inject(x) | EK::Widen(x) => self.closure_caps(x),
            // A generator value is the closure it holds (R07.6.2).
            EK::MakeStruct(xs) if self.walks(&e.ty) => {
                xs.first().and_then(|x| self.closure_caps(x))
            }
            EK::MakeClosure(xs) => {
                let CTy::Clo(k) = &e.ty else { return None };
                let i = *self.p.index.get(k)?;
                let caps = (xs.iter())
                    .map(|x| match &x.kind {
                        EK::AddrOf(p) | EK::Copy(p) | EK::Lent(p) => Some(p.clone()),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some((i, caps))
            }
            _ => None,
        }
    }
    /// A field whose type, in the generic fn this instance comes from, is a type parameter.
    fn generic_field(&self, p: &Place) -> bool {
        let Place::Field(b, i) = p else { return false };
        let Place::Local(l) = &**b else { return false };
        (self.f().declared.get(*l)).is_some_and(|d| d.param_fields.contains(i))
    }
    fn place_root_ty(&self, p: &Place) -> Option<CTy> {
        match p {
            Place::Local(l) => self.root_ty(Root::Local(*l)).cloned(),
            Place::Global(g) => self.root_ty(Root::Global(*g)).cloned(),
            _ => None,
        }
    }
    fn root_ty(&self, r: Root) -> Option<&CTy> {
        match r {
            Root::Local(l) => self.f().locals.get(l),
            Root::Global(g) => self.p.globals.get(g),
        }
    }
    /// What the bindings in `what` borrow in turn.
    fn through(&self, what: &BTreeSet<Pl>) -> BTreeSet<Pl> {
        (what.iter())
            .flat_map(|p| self.lent.get(&p.root).into_iter().flatten().cloned())
            .collect()
    }
    /// A generator value holds what it walks, as a view does (R07.6.2).
    fn walks(&self, t: &CTy) -> bool {
        matches!(t, CTy::Struct(s, ..) if Some(*s) == self.d.roles().recipe)
    }
    fn temp(&self, r: Root) -> bool {
        let name = match r {
            Root::Local(l) => self.f().names.get(l),
            Root::Global(g) => self.globals.get(g).map(|g| &g.0),
        };
        name.is_none_or(|n| n.is_empty() || n.starts_with('$'))
    }
    /// The place an argument lends: a temp holding a spilled binding stands for that binding.
    fn lent_pl(&self, p: &Place) -> Option<Pl> {
        let q = self.pl(p)?;
        Some(match self.spilled.get(&q.root) {
            Some(s) => Pl {
                root: s.root,
                path: [&s.path[..], &q.path[..]].concat(),
            },
            None => q,
        })
    }
    /// An element read out of a container: a `var` made from it is a copy (R05.3.1).
    fn elem_read(&self, e: &Expr<CTy>) -> bool {
        match &e.kind {
            EK::Call(Target::Inst(i), ..) => !self.declares_lends(*i) && !self.holds_lend(&e.ty),
            EK::Copy(p) => self
                .pl(p)
                .is_some_and(|q| q.path.is_empty() && self.elems.contains(&q.root)),
            _ => false,
        }
    }
    fn spill(&mut self, temp: Root, e: &Expr<CTy>) {
        match self.elem_read(e) {
            true => self.elems.insert(temp),
            false => self.elems.remove(&temp),
        };
        // A value given to a parameter the result holds moves in, made storage among them.
        let made_in = |a: &Arg<CTy>| match a {
            Arg::Value(v) => match &v.kind {
                EK::Copy(p) | EK::Pass(p, ..) => {
                    (self.pl(p)).is_some_and(|q| q.path.is_empty() && self.made.contains(&q.root))
                }
                _ => false,
            },
            _ => false,
        };
        let made = match &e.kind {
            EK::Call(Target::Inst(i), args, _) => {
                !self.declares_lends(*i)
                    && !self.literals.contains(i)
                    && (self.makers.contains(i)
                        || (self.lends(*i).iter()).any(|k| args.get(*k).is_some_and(made_in)))
            }
            EK::Copy(p) => {
                (self.pl(p)).is_some_and(|q| q.path.is_empty() && self.made.contains(&q.root))
            }
            _ => false,
        };
        match made && self.p.stores(&e.ty) {
            true => self.made.insert(temp),
            false => self.made.remove(&temp),
        };
        let whole = match &e.kind {
            EK::Copy(p) if !self.p.plain(&e.ty) => self.pl(p).filter(|q| {
                q.path.is_empty() && !self.temp(q.root) && !self.borrowed.contains(&q.root)
            }),
            _ => None,
        };
        match whole {
            Some(q) => self.spilled.insert(temp, q),
            None => self.spilled.remove(&temp),
        };
    }
    /// A call whose result lends its first argument: at a literal key it reaches only that cell.
    fn cell_call(&self, cs: &[usize]) -> bool {
        !cs.is_empty() && cs.iter().all(|c| self.lends(*c).contains(&0))
    }
    fn declares_lends(&self, i: usize) -> bool {
        match self.p.insts.get(i).map(|f| &f.key) {
            Some(FuncKey::Decl(f) | FuncKey::Ctx(f, _)) => self.d.f(*f).lends.is_some(),
            _ => false,
        }
    }
    fn var(&self, r: Root) -> bool {
        match r {
            Root::Local(l) => self.f().vars.get(l).copied().unwrap_or(false),
            Root::Global(g) => self.globals.get(g).is_some_and(|g| g.1),
        }
    }
    /// Whether a value of `t` can hold a lend: a view, a fn value or a generator value. A `var`
    /// made from any other value is its own copy: it borrows only what the fn was lent (R05.3.1).
    fn holds_lend(&self, t: &CTy) -> bool {
        fn go(w: &Walk, t: &CTy, depth: usize) -> bool {
            match t {
                CTy::Lent(_) | CTy::Ref(_) | CTy::Fn(_, _, _) | CTy::Clo(_) => true,
                CTy::Struct(..) if w.walks(t) => true,
                CTy::Struct(..) => depth >= 8 || w.p.parts(t).iter().any(|f| go(w, f, depth + 1)),
                CTy::Raw(t) => go(w, t, depth + 1),
                _ => false,
            }
        }
        go(self, t, 0)
    }
    /// A fn value each of whose members captures nothing holds nothing.
    fn captures_nothing(&self, t: &CTy) -> bool {
        match t {
            CTy::Clo(k) => self.p.captures(k).is_empty(),
            CTy::Fn(_, _, ms) => ms.iter().all(|k| self.p.captures(k).is_empty()),
            _ => false,
        }
    }
    /// Whether a value of `t` has a fn value in it, which holds its captures.
    fn holds_fn(&self, t: &CTy, depth: usize) -> bool {
        match t {
            CTy::Fn(_, _, _) | CTy::Clo(_) => true,
            CTy::Struct(..) if depth < 8 => {
                (self.p.parts(t).iter()).any(|f| self.holds_fn(f, depth + 1))
            }
            _ => false,
        }
    }
    fn root_name(&self, r: Root) -> String {
        match r {
            Root::Local(l) => match self.f().names.get(l) {
                Some(n) if !n.is_empty() => format!("`{n}`"),
                _ => "a local value".into(),
            },
            Root::Global(g) => match self.globals.get(g).map(|g| &g.0) {
                Some(n) => format!("`{n}`"),
                None => "a top-level value".into(),
            },
        }
    }
    fn err(&mut self, site: u32, code: Code, msg: String) {
        if !self.report {
            return;
        }
        let at = self.f().sites[site as usize].clone();
        let e = Msg::at(at, Msg::verdict(code, msg));
        // One message per source node, however many instances of a generic fn reach it.
        if !self.errors.contains(&e) {
            self.errors.push(e);
        }
    }
    /// A new binding of `r` (a loop body's `var` each turn among them) holds nothing yet.
    fn bind(&mut self, r: Root) {
        self.pending.retain(|x| x.0 != r);
        self.frozen.remove(&r);
        self.next += 1;
        self.decl.insert(r, (self.scopes.clone(), self.next));
    }
    /// The site of the first statement of a region's `body` that names `root`: where the lend
    /// it holds is broken.
    fn first_use(&self, body: &[Stmt<CTy>], root: Root) -> Option<u32> {
        body.iter().find_map(|s| {
            let mut places = Vec::new();
            cpl_next_ir::ir::visit_places(&mut [s.clone()], &mut |p| places.push(p.clone()));
            if !(places.iter()).any(|p| self.pl(p).is_some_and(|q| q.root == root)) {
                return None;
            }
            let site = match s {
                Stmt::Let(_, e) | Stmt::Assign(_, e) | Stmt::Expr(e) | Stmt::If(e, ..) => e.site,
                Stmt::Return(Some(e)) | Stmt::Yield(e) | Stmt::Switch(e, _) => e.site,
                Stmt::Region { site, .. } | Stmt::Check { site, .. } | Stmt::Loop(.., site) => {
                    *site
                }
                _ => 0,
            };
            (site != 0).then_some(site)
        })
    }
    fn block(&mut self, b: &[Stmt<CTy>], fx: &mut Fx) {
        self.next += 1;
        self.scopes.push(self.next);
        b.iter().for_each(|s| self.stmt(s, fx));
        self.scopes.pop();
    }
    /// A use of `p`: an error if it was moved out, or if a write froze it while borrowed.
    fn live(&mut self, site: u32, p: &Pl) {
        if let Some(i) = self.pending.iter().position(|x| x.0 == p.root) {
            let (_, at, code, msg) = self.pending.remove(i);
            self.pending.retain(|x| x.0 != p.root);
            self.err(at, code, msg);
        }
        if let Some(m) = self.moved.iter().find(|m| overlap(m, p)).cloned() {
            let n = self.root_name(m.root);
            self.err(
                site,
                codes::USE_AFTER_MOVE,
                format!("{n} was moved into another binding; use `.clone()` there to keep both"),
            );
        }
    }
    /// A write to `p`: an error if a binding that borrows it is used later.
    fn wrote(&mut self, site: u32, p: &Pl) {
        self.wrote_via(site, p, &BTreeSet::new())
    }
    /// A write to `p` made through the fn values `via`: those don't conflict with it.
    fn wrote_via(&mut self, site: u32, p: &Pl, via: &BTreeSet<Root>) {
        let n = self.root_name(p.root);
        let hit: Vec<Root> = (self.frozen.iter())
            .filter(|(b, srcs)| {
                **b != p.root && !via.contains(*b) && srcs.iter().any(|s| overlap(s, p))
            })
            .map(|(b, _)| *b)
            .collect();
        for b in hit {
            let bn = self.root_name(b);
            let msg = format!("{n} is written here while {bn}, which borrows it, is still used");
            self.pending.push((b, site, codes::USED_WHILE_LENT, msg));
        }
    }
    /// An unnamed temporary holding storage a call made, not a lend: it dies at the end of its
    /// statement (R07.5.10).
    /// R8: a lent value is kept only in storage that dies no later than what it borrows.
    fn keep(&mut self, site: u32, storage: Root, what: &BTreeSet<Pl>) {
        if what.is_empty() {
            return;
        }
        if self.temp(storage) {
            self.lent
                .entry(storage)
                .or_default()
                .extend(what.iter().cloned());
            return;
        }
        // R07.5.10: a temporary holding its own value dies at the end of its statement.
        if what
            .iter()
            .any(|p| fresh(&self.made, &self.spilled, &self.lent, p.root))
        {
            let n = self.root_name(storage);
            let m = format!(
                "{n} keeps a view of a temporary, which dies at the end of its statement; \
                 copy it with List(...)"
            );
            self.err(site, codes::LENT_ESCAPES, m);
        }
        let home = self.decl.get(&storage).cloned();
        let roots: BTreeSet<Root> = (what.iter().map(|p| p.root))
            .filter(|r| !fresh(&self.made, &self.spilled, &self.lent, *r))
            .collect();
        for r in roots {
            let ok = match (&home, self.decl.get(&r)) {
                // A parameter's storage is the caller's, and a top-level binding outlives any fn.
                (None, _) => matches!((storage, r), (Root::Global(_), Root::Global(_))),
                (Some(_), None) => true,
                (Some((hs, ho)), Some((ss, so))) => hs.starts_with(ss) && so < ho,
            };
            if !ok {
                let (a, b) = (self.root_name(storage), self.root_name(r));
                self.err(
                    site,
                    codes::LENT_ESCAPES,
                    format!("a value that borrows {b} is stored in {a}, which outlives {b}"),
                );
            }
        }
        self.lent
            .entry(storage)
            .or_default()
            .extend(what.iter().cloned());
    }
    /// Stores `what` into `storage`: a type-parameter parameter is the fn's to keep (R05.2.2),
    /// anything else lent is held to R8. A value lent from `storage` itself is free again.
    fn store(&mut self, site: u32, storage: Root, mut what: BTreeSet<Pl>) {
        what.retain(|p| p.root != storage);
        // Storage with no declaration is a parameter's; a lowering temp is no storage of its own.
        if !self.decl.contains_key(&storage) && !self.temp(storage) {
            let ps: Vec<usize> = self.f().params.iter().map(|p| p.0).collect();
            let own: Vec<usize> = (what.iter())
                .filter_map(|p| match p.root {
                    Root::Local(l) => ps.iter().position(|x| *x == l).filter(|_| self.of_param(l)),
                    Root::Global(_) => None,
                })
                .collect();
            what.retain(
                |p| !matches!(p.root, Root::Local(l) if ps.contains(&l) && self.of_param(l)),
            );
            self.kept.extend(own);
        }
        self.keep(site, storage, &what);
    }
    /// The places the value of `e` lends: a borrowed binding, or what a binding was given.
    /// What `place_copies` makes of an argument: a viewable place not copied in is viewed.
    fn settle_arg<'a>(&self, a: &'a Arg<CTy>) -> Cow<'a, Arg<CTy>> {
        match a {
            Arg::Value(Expr {
                kind: EK::Pass(p, true, s),
                ..
            }) if !self.copies.contains(&(self.cur, *s)) => Cow::Owned(Arg::View(p.clone())),
            a => Cow::Borrowed(a),
        }
    }
    /// A passed place the walk copies in is fresh; any other reads as the place itself.
    fn unpass<'e>(&self, e: &'e Expr<CTy>) -> Option<Cow<'e, Expr<CTy>>> {
        match &e.kind {
            EK::Pass(_, _, s) if self.copies.contains(&(self.cur, *s)) => None,
            EK::Pass(p, ..) => Some(Cow::Owned(Expr {
                kind: EK::Copy(p.clone()),
                ..e.clone()
            })),
            _ => Some(Cow::Borrowed(e)),
        }
    }
    fn sources(&self, e: &Expr<CTy>) -> BTreeSet<Pl> {
        let Some(e) = self.unpass(e) else {
            return BTreeSet::new();
        };
        let e = &*e;
        // A plain value (an `Int` field read among them) holds nothing; fn values hold captures.
        if self.p.plain(&e.ty) && !self.holds_fn(&e.ty, 0) || self.captures_nothing(&e.ty) {
            return BTreeSet::new();
        }
        let mut out = BTreeSet::new();
        // What a receiver's view cuts from a temporary holding its own value (R07.5.10).
        let mut cut = BTreeSet::new();
        if matches!(
            e.ty,
            CTy::Int
                | CTy::Bool
                | CTy::Char
                | CTy::U64
                | CTy::I128
                | CTy::F64
                | CTy::Unit
                | CTy::Dom(_)
        ) {
            return out;
        }
        match &e.kind {
            EK::Copy(p) | EK::Lent(p) | EK::AddrOf(p) | EK::Pass(p, ..) => {
                if let Some(q) = self.pl(p) {
                    out.extend(self.lent.get(&q.root).into_iter().flatten().cloned());
                    // A plain value read out is a copy: it holds only what its binding was given.
                    // So is a field of a type parameter's type: each instance copies it (R05.2.2).
                    // A fn value read out of a borrowed binding still holds what it borrows.
                    let fn_value = matches!(e.ty, CTy::Clo(_) | CTy::Fn(_, _, _));
                    let copy = matches!(e.kind, EK::Copy(_))
                        && (self.p.plain(&e.ty) && !fn_value || self.generic_field(p));
                    if self.temp(q.root) || copy {
                        return out;
                    }
                    if self.borrowed.contains(&q.root)
                        || !q.path.is_empty()
                        || !matches!(e.kind, EK::Copy(_))
                    {
                        out.insert(q);
                    }
                }
            }
            // A field read into a struct moves into it (R05.2.2): it holds what the base holds.
            // A plain field or a type parameter's is a copy, as read anywhere (`sources(x)`).
            EK::MakeStruct(xs) => xs.iter().for_each(|x| match &x.kind {
                EK::Copy(p @ Place::Field(..))
                    if fields_only(p) && !self.p.plain(&x.ty) && !self.generic_field(p) =>
                {
                    if let Some(q) = self.pl(p) {
                        out.extend(self.lent.get(&q.root).into_iter().flatten().cloned());
                        if self.borrowed.contains(&q.root) {
                            out.insert(Pl::whole(q.root));
                        }
                    }
                }
                _ => out.extend(self.sources(x)),
            }),
            EK::MakeClosure(xs) => xs.iter().for_each(|x| out.extend(self.sources(x))),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => out = self.sources(x),
            EK::Call(Target::Inst(i), args, _) => {
                let lends = self.lends(*i);
                for (k, a) in args.iter().enumerate() {
                    if !lends.contains(&k) {
                        continue;
                    }
                    match &*self.settle_arg(a) {
                        Arg::Value(v) => {
                            out.extend(self.sources(v));
                            // A binding passed whole where the result lends it is lent: a heap
                            // one, or any the signature says `lends` (a fixed-size grid).
                            let heap = !self.p.plain(&v.ty) || self.declares_lends(*i);
                            if let (EK::Copy(p), true) = (&v.kind, heap) {
                                // A view cut from a temporary holds it (R07.5.10).
                                let viewed = (k == 0 && self.receivers.contains(i))
                                    || self.declares_lends(*i);
                                let cut_here = (self.pl(p))
                                    .filter(|q| {
                                        fresh(&self.made, &self.spilled, &self.lent, q.root)
                                    })
                                    .filter(|_| viewed && !self.p.copied(&e.ty));
                                // An owned binding read for the last time moves in, with what
                                // it borrows (R05.5.3); a temporary a view is cut from does not.
                                if let (Place::Local(l), None) = (p, &cut_here) {
                                    let r = Root::Local(*l);
                                    if self.last.here.contains(l)
                                        && self.decl.contains_key(&r)
                                        && !self.borrowed.contains(&r)
                                    {
                                        out.extend(
                                            self.lent.get(&r).into_iter().flatten().cloned(),
                                        );
                                        continue;
                                    }
                                }
                                out.extend(self.lent_pl(p));
                                cut.extend(cut_here);
                            }
                        }
                        Arg::View(p) | Arg::Lend(p) => {
                            if let Some(mut q) = self.lent_pl(p) {
                                out.extend(self.lent.get(&q.root).into_iter().flatten().cloned());
                                // R05.8.2: `s[k]` at a literal `k` lends only that element.
                                if let (
                                    0,
                                    [_, Arg::Value(Expr {
                                        kind: EK::Int(k), ..
                                    })],
                                ) = (k, &args[..])
                                {
                                    q.path.push(Step::Key(*k));
                                }
                                out.insert(q);
                            }
                        }
                    }
                }
            }
            EK::Intr(n, args, _) => {
                for k in n.lends() {
                    match args.get(*k).map(|a| self.settle_arg(a)).as_deref() {
                        Some(Arg::Value(v)) => out.extend(self.sources(v)),
                        Some(Arg::View(p) | Arg::Lend(p)) => out.extend(self.lent_pl(p)),
                        None => {}
                    }
                }
            }
            _ => {}
        }
        // A temp is never a source itself, what it holds already is, unless a view is cut from
        // the value it holds: that dies at the end of its statement (R07.5.10). A fresh value
        // passed to any other parameter moves in.
        out.retain(|p| !self.temp(p.root));
        out.extend(cut);
        out
    }
    /// The arguments an instance's result lends, receiver first (R05.5.3).
    fn lends(&self, i: usize) -> BTreeSet<usize> {
        match self.p.insts.get(i).map(|f| &f.key) {
            Some(FuncKey::Decl(f) | FuncKey::Ctx(f, _)) => lends(self.d, *f),
            _ => BTreeSet::new(),
        }
    }
    fn reads(&mut self, e: &Expr<CTy>, fx: &mut Fx) {
        match &e.kind {
            EK::Copy(p) | EK::Lent(p) | EK::AddrOf(p) | EK::Pass(p, ..) | EK::Tag(p) => {
                if let Some(q) = self.pl(p) {
                    self.live(e.site, &q);
                    fx.reads.insert(q);
                }
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| self.reads(x, fx)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.reads(x, fx),
            EK::Call(t, args, _) => {
                let cs = self.callees(t, None);
                self.call(e.site, &cs, &[], self.cell_call(&cs), args, fx);
            }
            EK::Intr(n, args, _) => {
                self.call(e.site, &[], n.keeps(), false, args, fx);
            }
            EK::CallValue(c, args, _) => {
                self.reads(c, fx);
                let mut cs = self.callees(&Target::Case, Some(c));
                let mut args: Vec<Arg<CTy>> = args.to_vec();
                // A known closure's captures are its first parameters: what the body does to them
                // it does to the captured places (R05.10.3).
                if let EK::Copy(p @ (Place::Local(_) | Place::Global(_))) = &c.kind {
                    let r = self.pl(p);
                    if let Some((i, caps)) = r.and_then(|r| self.caps.get(&r.root).cloned()) {
                        cs = vec![i];
                        args = caps.into_iter().map(Arg::View).chain(args).collect();
                    }
                }
                self.call(e.site, &cs, &[], self.cell_call(&cs), &args, fx);
            }
            _ => {}
        }
    }
    /// R2: binding or assigning a heap binding (or a field path under it) moves it.
    fn take(&mut self, e: &Expr<CTy>) {
        let Some(e) = self.unpass(e) else { return };
        let e = &*e;
        // A struct built from `a.xs` (a record update among them) moves the path (R12.11.4).
        if let EK::MakeStruct(xs) = &e.kind {
            return xs.iter().for_each(|x| self.take_path(x));
        }
        // A field read is a lend of it (R05.3.5): `let b = a.xs` moves nothing.
        if let EK::Copy(Place::Local(_) | Place::Global(_)) = &e.kind {
            self.take_path(e)
        }
    }
    fn take_path(&mut self, e: &Expr<CTy>) {
        let EK::Copy(p) = &e.kind else { return };
        if !fields_only(p) {
            return;
        }
        let Some(q) = self.pl(p) else { return };
        // A lowering temp is read where the lowering says; it is no binding to move.
        if self.temp(q.root) {
            return;
        }
        // R05.11.1: a fn reads top-level state lent, as it does a parameter.
        if let (Root::Global(_), false, false, false) = (
            q.root,
            self.cur == self.p.main,
            self.p.plain(&e.ty),
            self.temp(q.root),
        ) {
            let n = self.root_name(q.root);
            return self.err(
                e.site,
                codes::USE_AFTER_MOVE,
                format!("{n} is top-level state this fn reads lent; binding it would move it out, so `.clone()` it"),
            );
        }
        // A value of a type parameter's type is copied by each instance that must (R05.2.2).
        let generic = matches!(q.root, Root::Local(l) if self.of_param(l));
        if self.decl.contains_key(&q.root)
            && !generic
            && !self.borrowed.contains(&q.root)
            && !self.lent.contains_key(&q.root)
            && !self.p.plain(&e.ty)
        {
            // R05.10.2: a value that still holds a lend of it can't outlive the move (E0602).
            let n = self.root_name(q.root);
            let users: Vec<Root> = (self.lent.iter())
                .filter(|(b, srcs)| **b != q.root && srcs.iter().any(|s| s.root == q.root))
                .map(|(b, _)| *b)
                .collect();
            for b in users {
                let bn = self.root_name(b);
                let msg = format!("{n} is moved here while {bn}, which uses it, is still used");
                // A fn value that captures it is E0607 (R05.10.2).
                let code = match self.root_ty(b) {
                    Some(CTy::Clo(_) | CTy::Fn(_, _, _)) => codes::MOVED_WHILE_CAPTURED,
                    _ => codes::USED_WHILE_LENT,
                };
                self.pending.push((b, e.site, code, msg));
            }
            // A binding moved out holds no loans any more.
            if q.path.is_empty() {
                self.pending.retain(|x| x.0 != q.root);
                self.frozen.remove(&q.root);
            }
            self.moved.insert(q);
        }
    }
    /// One call (R10): arguments, and what each instance it can run does by its summary.
    /// Returns what it holds for reading and for writing while it runs.
    fn call(
        &mut self,
        site: u32,
        cs: &[usize],
        kept: &[usize],
        cell: bool,
        args: &[Arg<CTy>],
        fx: &mut Fx,
    ) -> (BTreeSet<Pl>, BTreeSet<Pl>) {
        let mut places: Vec<Option<Pl>> = (args.iter())
            .map(|a| match a {
                Arg::View(p) | Arg::Lend(p) => self.lent_pl(p),
                Arg::Value(v) => match &v.kind {
                    EK::Copy(p) | EK::Pass(p, ..) => self.lent_pl(p),
                    _ => None,
                },
            })
            .collect();
        // R05.8.2: a cell lent at a literal `k` (`s[k]`) reaches only that element.
        if let (
            true,
            [Arg::View(_) | Arg::Lend(_), Arg::Value(Expr {
                kind: EK::Int(k), ..
            })],
            Some(Some(q)),
        ) = (cell, args, places.first_mut())
        {
            q.path.push(Step::Key(*k));
        }
        let mut captured = BTreeSet::new();
        let (mut other, mut keeps) = (
            Fx::default(),
            kept.iter().copied().collect::<BTreeSet<usize>>(),
        );
        for c in cs {
            let Some(sum) = self.sums.get(c) else {
                continue;
            };
            keeps.extend(sum.keeps.iter().copied());
            let ps: Vec<usize> = self.p.insts[*c].params.iter().map(|p| p.0).collect();
            let map = |p: &Pl| match p.root {
                Root::Global(_) => Some(p.clone()),
                Root::Local(l) => {
                    let i = ps.iter().position(|x| *x == l)?;
                    let mut q = places.get(i)?.clone()?;
                    q.path.extend(p.path.iter().copied());
                    Some(q)
                }
            };
            // What it does through its own parameters is the argument's access, not another way in.
            let through = |p: &&Pl| matches!(p.root, Root::Local(_));
            let (tr, ot): (Vec<&Pl>, Vec<&Pl>) = sum.fx.reads.iter().partition(through);
            let (tw, ow): (Vec<&Pl>, Vec<&Pl>) = sum.fx.writes.iter().partition(through);
            other.reads.extend(ot.into_iter().filter_map(map));
            other.writes.extend(ow.into_iter().filter_map(map));
            fx.reads.extend(tr.into_iter().filter_map(map));
            // A write through a viewed parameter is a closure's write to what it captured: the
            // call writes that place (R05.10.3).
            let viewed = |p: &&&Pl| match p.root {
                Root::Local(l) => (ps.iter().position(|x| *x == l))
                    .is_some_and(|i| matches!(args.get(i), Some(Arg::View(_)))),
                Root::Global(_) => false,
            };
            captured.extend(tw.iter().filter(viewed).filter_map(|p| map(p)));
            fx.writes.extend(tw.into_iter().filter_map(map));
            other.held.extend(sum.fx.held.iter().filter_map(map));
        }
        // R05.10.3: a closure passed in may be called there; what it does to its captures the
        // call does another way.
        fn peel(e: &Expr<CTy>) -> &Expr<CTy> {
            match &e.kind {
                EK::Inject(x) | EK::Widen(x) => peel(x),
                _ => e,
            }
        }
        let mut via = BTreeSet::new();
        for a in args {
            let Arg::Value(v) = a else { continue };
            let EK::Copy(p) = &peel(v).kind else { continue };
            let Some((i, caps)) = self.pl(p).and_then(|q| self.caps.get(&q.root).cloned()) else {
                continue;
            };
            via.extend(self.pl(p).map(|q| q.root));
            let Some(sum) = self.sums.get(&i) else {
                continue;
            };
            let Some(callee) = self.p.insts.get(i) else {
                continue;
            };
            let ps: Vec<usize> = callee.params.iter().map(|p| p.0).collect();
            let map = |p: &Pl| match p.root {
                Root::Local(l) => {
                    let at = ps.iter().position(|x| *x == l)?;
                    let mut q = self.pl(caps.get(at)?)?;
                    q.path.extend(p.path.iter().copied());
                    Some(q)
                }
                Root::Global(_) => Some(p.clone()),
            };
            other.reads.extend(sum.fx.reads.iter().filter_map(map));
            other.writes.extend(sum.fx.writes.iter().filter_map(map));
        }
        // R4: the callee keeps the value it was called with, so a passed place the call also
        // writes is copied in; any other is viewed.
        let mut copied = vec![false; args.len()];
        for (i, (a, p)) in args.iter().zip(&places).enumerate() {
            let (
                Arg::Value(Expr {
                    kind: EK::Pass(_, _, s),
                    ty,
                    ..
                }),
                Some(p),
            ) = (a, p)
            else {
                continue;
            };
            if !self.p.plain(ty) && other.writes.iter().any(|w| overlap(w, p)) {
                copied[i] = true;
                self.copies.insert((self.cur, *s));
            }
        }
        let args: Vec<Arg<CTy>> = args
            .iter()
            .map(|a| self.settle_arg(a).into_owned())
            .collect();
        let args = &args[..];
        let viewed = |i: usize| match &args[i] {
            Arg::View(_) => true,
            Arg::Value(v) => matches!(v.kind, EK::Pass(_, true, _)) && !copied[i],
            Arg::Lend(_) => false,
        };
        // R4: one place lent for writing and also passed is two ways in (E0601).
        for (i, (a, p)) in args.iter().zip(&places).enumerate() {
            let (Arg::Lend(_), Some(p)) = (a, p) else {
                continue;
            };
            let twice = (places.iter().enumerate()).any(|(j, q)| {
                // A plain value passed is a copy, not a second way in.
                let copy = matches!(&args[j], Arg::Lend(_))
                    || matches!(&args[j], Arg::Value(v) if self.p.plain(&v.ty))
                    || copied[j];
                j != i && !copy && q.as_ref().is_some_and(|q| overlap(p, q))
            });
            if let (true, Some(c)) = (twice, cs.first()) {
                let (n, who) = (self.root_name(p.root), self.p.insts[*c].name.clone());
                let msg =
                    format!("{n} is passed to `{who}` and lent to it for writing in the same call");
                self.err(site, codes::LENT_TWICE, msg);
            }
        }
        let (mut lent, mut held): (BTreeSet<Pl>, BTreeSet<Pl>) = Default::default();
        for (i, (a, p)) in args.iter().zip(&places).enumerate() {
            if let (true, Some(p)) = (viewed(i), p) {
                held.insert(p.clone());
            }
            match (a, p) {
                (Arg::Lend(_), Some(p)) => {
                    self.live(site, p);
                    if let Some(q) = lent.iter().find(|q| overlap(q, p)) {
                        let n = self.root_name(q.root);
                        self.err(
                            site,
                            codes::LENT_TWICE,
                            format!("{n} is lent for writing twice in the same call"),
                        );
                    }
                    lent.insert(p.clone());
                }
                (Arg::Value(v), _) if !viewed(i) => self.reads(v, fx),
                (_, Some(p)) => {
                    self.live(site, p);
                    fx.reads.insert(p.clone());
                }
                _ => {}
            }
        }
        if let Some(p) = lent.iter().find(|p| {
            other
                .reads
                .iter()
                .chain(&other.writes)
                .any(|q| overlap(p, q))
        }) {
            let n = self.root_name(p.root);
            self.err(
                site,
                codes::USED_WHILE_LENT,
                format!("{n} is lent for writing, and the call also reaches it another way"),
            );
        }
        let mut kept = BTreeSet::new();
        for (i, a) in args.iter().enumerate() {
            if keeps.contains(&i) && !lent.is_empty() && closure_param(&self.closure_params, a) {
                let msg = "a closure argument is a parameter: the callee can call it but not \
                           keep it in storage that outlives the call";
                self.err(site, codes::LENT_ESCAPES, msg.into());
            }
            // A copy holds nothing of the place it was made from.
            if !keeps.contains(&i) || copied[i] {
                continue;
            }
            // A parameter of a type parameter's type that a callee keeps is kept by this fn too,
            // whatever its type is in this instance (R05.2.2).
            let place = match a {
                Arg::Value(Expr {
                    kind: EK::Copy(p) | EK::Pass(p, ..),
                    ..
                })
                | Arg::View(p)
                | Arg::Lend(p) => Some(p),
                Arg::Value(_) => None,
            };
            if let Some(Place::Local(l)) = place {
                let ps: Vec<usize> = self.f().params.iter().map(|p| p.0).collect();
                if let Some(at) = ps.iter().position(|x| x == l).filter(|_| self.of_param(*l)) {
                    self.kept.insert(at);
                }
            }
            match a {
                // A kept view is copied in: it holds what its place holds (R05.2.2).
                Arg::View(p)
                | Arg::Value(Expr {
                    kind: EK::Pass(p, true, _),
                    ..
                }) => {
                    // R2: a whole binding a callee keeps is moved into it.
                    if let (Place::Local(_) | Place::Global(_), Some(ty)) =
                        (p, self.place_root_ty(p))
                    {
                        let e = Expr {
                            kind: EK::Copy(p.clone()),
                            ty,
                            site,
                        };
                        self.take_path(&e);
                    }
                    let Some(q) = self.pl(p) else { continue };
                    kept.extend(self.lent.get(&q.root).into_iter().flatten().cloned());
                    if self.borrowed.contains(&q.root) || !q.path.is_empty() {
                        kept.insert(q);
                    }
                }
                Arg::Value(v) => {
                    self.take(v);
                    kept.extend(self.sources(v));
                }
                Arg::Lend(p) => kept.extend(self.pl(p)),
            }
        }
        for p in lent.clone() {
            self.store(site, p.root, kept.clone());
        }
        for p in (lent.iter().chain(&other.writes).chain(&captured))
            .cloned()
            .collect::<BTreeSet<_>>()
        {
            self.wrote_via(site, &p, &via);
        }
        // R2: a struct built as an argument takes what it is built from, once the call has read it.
        for (i, a) in args.iter().enumerate() {
            if let (
                false,
                Arg::Value(
                    v @ Expr {
                        kind: EK::MakeStruct(_),
                        ..
                    },
                ),
            ) = (keeps.contains(&i), a)
            {
                self.take(v);
            }
        }
        fx.writes.extend(lent.iter().cloned());
        held.extend(other.held.iter().cloned());
        fx.add(&other);
        (held, lent)
    }
    fn stmt(&mut self, s: &Stmt<CTy>, fx: &mut Fx) {
        self.last.stmt(s);
        if let Stmt::Let(l, e) = s {
            if copy_of(e).is_some_and(|m| self.closure_params.contains(&m)) {
                self.closure_params.insert(*l);
            }
        }
        match s {
            // A passed place saved before a later operand runs, or a tuple's part read before
            // its places are written: a copy, or nothing if viewed. It borrows nothing.
            Stmt::Let(_, e) if matches!(e.kind, EK::Pass(..)) => self.reads(e, fx),
            Stmt::Let(l, e) => {
                self.reads(e, fx);
                // A temp the lowering made holds the place it was read from; it moves nothing.
                let temp = self.temp(Root::Local(*l));
                if temp {
                    self.spill(Root::Local(*l), e);
                }
                // A struct built into a temp still takes what it is built from.
                if !temp || matches!(e.kind, EK::MakeStruct(_)) {
                    self.take(e);
                }
                if let EK::Int(k) = e.kind {
                    self.consts.insert(*l, k);
                }
                let uses = self.closure_bound(Root::Local(*l), e);
                self.bind(Root::Local(*l));
                let mut src = self.sources(e);
                if self.var(Root::Local(*l)) && self.elem_read(e) {
                    src.retain(|p| {
                        self.borrowed.contains(&p.root)
                            || fresh(&self.made, &self.spilled, &self.lent, p.root)
                    });
                }
                // A spill of a borrowed binding or of a field path is a lend of it; a spill of a
                // whole owned binding is that value.
                if let (true, EK::Copy(p)) = (temp && !self.p.plain(&e.ty), &e.kind) {
                    src.extend(
                        self.pl(p)
                            .filter(|q| !q.path.is_empty() || self.borrowed.contains(&q.root)),
                    );
                }
                // A fn value holds only its captures' names: they are borrowed per call, so a
                // write between calls is no conflict (R05.10.3).
                let closure = self.closure_caps(e).is_some();
                if (!self.p.plain(&e.ty) || self.walks(&e.ty)) && !src.is_empty() && !closure {
                    self.frozen.insert(Root::Local(*l), src.clone());
                }
                src.extend(uses);
                // A closure holds what its captures borrow, as a view does (R05.7.1).
                if closure {
                    let through = self.through(&src);
                    if !through.is_empty() {
                        self.frozen
                            .entry(Root::Local(*l))
                            .or_default()
                            .extend(through);
                    }
                }
                if !src.is_empty() {
                    self.lent.remove(&Root::Local(*l));
                    self.keep(e.site, Root::Local(*l), &src);
                }
                if self.report {
                    let r = Root::Local(*l);
                    (self.binds).extend(bound(self.cur, r, &self.frozen, &self.lent));
                }
            }
            Stmt::Assign(p, e) => {
                self.reads(e, fx);
                let mut walked = BTreeSet::new();
                if let Some(q) = self.pl(p).filter(|q| q.path.is_empty()) {
                    let uses = self.closure_bound(q.root, e);
                    // A generator value holds the heap state it walks, not plain counters (R07.6.2).
                    if !self.walks(&e.ty) {
                        walked = self.through(&uses);
                    } else {
                        let heap = |p: &&Pl| self.root_ty(p.root).is_some_and(|t| !self.p.plain(t));
                        walked = uses.iter().filter(heap).cloned().collect();
                    }
                    if !uses.is_empty() {
                        self.lent.entry(q.root).or_default().extend(uses);
                    }
                }
                if let Some(q) = self
                    .pl(p)
                    .filter(|q| q.path.is_empty() && self.temp(q.root))
                {
                    self.spill(q.root, e);
                }
                // A branch's value written to the lowering's temp moves nothing, as at its `let`.
                if let (Place::Local(l), true) = (
                    p,
                    matches!(p, Place::Local(l) if self.temp(Root::Local(*l))),
                ) {
                    if let (false, EK::Copy(q)) = (self.p.plain(&e.ty), &e.kind) {
                        let held = self
                            .pl(q)
                            .filter(|q| !q.path.is_empty() || self.borrowed.contains(&q.root));
                        let mut src = self.sources(e);
                        src.extend(held);
                        self.lent.entry(Root::Local(*l)).or_default().extend(src);
                        return;
                    }
                }
                self.take(e);
                if let Some(q) = self.pl(p) {
                    self.wrote(e.site, &q);
                    self.moved
                        .retain(|m| !(m.root == q.root && m.path.starts_with(&q.path)));
                    if q.path.is_empty() {
                        // The file body's `let` binds a global where it is first written.
                        if self.cur == self.p.main && !self.decl.contains_key(&q.root) {
                            self.bind(q.root);
                        }
                    }
                    let mut what = self.sources(e);
                    if q.path.is_empty() && self.var(q.root) && self.elem_read(e) {
                        what.retain(|p| {
                            self.borrowed.contains(&p.root)
                                || fresh(&self.made, &self.spilled, &self.lent, p.root)
                        });
                    }
                    if q.path.is_empty() && !walked.is_empty() {
                        self.frozen
                            .entry(q.root)
                            .or_default()
                            .extend(walked.iter().cloned());
                    }
                    if q.path.is_empty()
                        && (!self.p.plain(&e.ty) || self.walks(&e.ty))
                        && !what.is_empty()
                    {
                        self.frozen
                            .entry(q.root)
                            .or_default()
                            .extend(what.iter().cloned());
                    }
                    self.store(e.site, q.root, what);
                    // The file body's `let`s are globals, bound where first written.
                    if self.report && q.path.is_empty() && matches!(q.root, Root::Global(_)) {
                        let r = q.root;
                        (self.binds).extend(bound(self.cur, r, &self.frozen, &self.lent));
                    }
                    fx.writes.insert(q);
                }
            }
            Stmt::Expr(e) | Stmt::Fact(e) => self.reads(e, fx),
            Stmt::Yield(e) => {
                self.reads(e, fx);
                // R05.10.2: a yielded value over the generator's own locals lives only as long
                // as the walk.
                let ps: BTreeSet<Root> = self.f().params.iter().map(|p| Root::Local(p.0)).collect();
                let own = |p: &Pl| matches!(p.root, Root::Local(_)) && !ps.contains(&p.root);
                let mut v = e;
                while let EK::Inject(x) | EK::Widen(x) = &v.kind {
                    v = x;
                }
                let caps = self.closure_caps(v).map(|(_, ps)| ps).unwrap_or_default();
                fx.yields_own |= self.sources(e).iter().any(own)
                    || caps.iter().filter_map(|p| self.pl(p)).any(|q| own(&q));
            }
            Stmt::Return(e) => e.iter().for_each(|e| {
                self.reads(e, fx);
                self.escapes(e);
            }),
            Stmt::Check { cond, .. } => self.reads(cond, fx),
            Stmt::If(c, a, b) => {
                self.reads(c, fx);
                let before = self.moved.clone();
                self.block(a, fx);
                let after_a = std::mem::replace(&mut self.moved, before);
                self.block(b, fx);
                self.moved.extend(after_a);
            }
            Stmt::Loop(a, b, site) => {
                let (at, before) = (self.next, self.moved.clone());
                self.last.depth += 1;
                self.block(a, fx);
                self.block(b, fx);
                self.last.depth -= 1;
                self.moved_in_loop(*site, at, &before);
            }
            Stmt::Switch(c, arms) => {
                self.reads(c, fx);
                let before = self.moved.clone();
                let mut all = before.clone();
                for a in arms {
                    self.moved = before.clone();
                    self.block(a, fx);
                    all.extend(self.moved.iter().cloned());
                }
                self.moved = all;
            }
            // R12: the callee's lends are live for the walk, and the body runs inside the call.
            Stmt::Region {
                target,
                args,
                var,
                body,
                body_site,
                site,
                lend,
            } => {
                let cs = self.callees(target, None);
                // Only a lender's item is a cell of what it is given (R05.8.2).
                let (mut held, lent) = self.call(*site, &cs, &[], *lend, args, fx);
                // A walk of a generator value uses it, and holds what it holds (R07.6.2).
                if let Target::Value(x) = target {
                    self.reads(x, fx);
                    let root = match &x.kind {
                        EK::Copy(p) => self.pl(p).map(|q| q.root),
                        _ => None,
                    };
                    held.extend(
                        root.and_then(|r| self.lent.get(&r))
                            .into_iter()
                            .flatten()
                            .cloned(),
                    );
                }
                // What the walked value lends is held as long as it is (R05.5.3).
                for a in args {
                    match &*self.settle_arg(a) {
                        Arg::Value(v) => match &v.kind {
                            EK::Copy(p) => {
                                let r = self.pl(p).map(|q| q.root);
                                held.extend(
                                    r.and_then(|r| self.lent.get(&r))
                                        .into_iter()
                                        .flatten()
                                        .cloned(),
                                )
                            }
                            _ => held.extend(self.sources(v)),
                        },
                        Arg::View(p) | Arg::Lend(p) => {
                            let r = self.pl(p).map(|q| q.root);
                            held.extend(
                                r.and_then(|r| self.lent.get(&r))
                                    .into_iter()
                                    .flatten()
                                    .cloned(),
                            )
                        }
                    }
                }
                let who = match cs.first() {
                    Some(c) => format!("`{}`", self.p.insts[*c].name),
                    None => "the generator".to_string(),
                };
                self.bind(Root::Local(*var));
                // The walk lends each item for one turn (R12).
                let frame = cs
                    .iter()
                    .any(|c| self.sums.get(c).is_some_and(|s| s.fx.yields_own));
                if frame || !self.p.plain(&self.f().locals[*var]) && !self.of_param(*var) {
                    self.borrowed.insert(Root::Local(*var));
                }
                let (at, before) = (self.next, self.moved.clone());
                let mut inner = Fx::default();
                self.block(body, &mut inner);
                self.moved_in_loop(*body_site, at, &before);
                let own = Root::Local(*var);
                inner.reads.retain(|p| p.root != own);
                inner.writes.retain(|p| p.root != own);
                let hit = |a: &BTreeSet<Pl>, b: &BTreeSet<Pl>| {
                    a.iter().find(|p| b.iter().any(|q| overlap(p, q))).cloned()
                };
                // A lender lends its cell once, for the body; a generator lends for every turn.
                let (span, user) = match lend {
                    true => ("while its cell is in use", "this"),
                    false => ("for the whole loop", "the loop body"),
                };
                if let Some(p) = hit(&held, &inner.writes) {
                    let n = self.root_name(p.root);
                    let at = match lend {
                        true => self.first_use(body, p.root).unwrap_or(*body_site),
                        false => *body_site,
                    };
                    self.err(
                        at,
                        codes::USED_WHILE_LENT,
                        format!("{n} is lent to {who} {span}, and {user} writes it"),
                    );
                }
                if let Some(p) = hit(&lent, &inner.writes).or_else(|| hit(&lent, &inner.reads)) {
                    let n = self.root_name(p.root);
                    let at = self.first_use(body, p.root).unwrap_or(*body_site);
                    self.err(
                        at,
                        codes::USED_WHILE_LENT,
                        format!("{n} is lent to {who} for writing {span}, and {user} uses it"),
                    );
                }
                fx.add(&inner);
            }
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                xs.iter().for_each(|x| self.reads(x, fx))
            }
            Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
        }
    }
    /// Whether a local's type in the generic fn holds a type parameter.
    fn of_param(&self, l: usize) -> bool {
        (self.f().declared.get(l)).is_some_and(|d| d.of_param)
    }
    /// R6: a fn returns a borrow only of what its signature lends (R05.5.3, R05.6.1).
    /// The `@cache` field of `self` that the lends result `e` would borrow (R05.10.9).
    fn lends_cache(&self, e: &Expr<CTy>) -> Option<String> {
        use cpl_next_decls::decls::{Owner, SelfMode};
        let (FuncKey::Decl(fid) | FuncKey::Ctx(fid, _)) = self.f().key else {
            return None;
        };
        let fd = self.d.f(fid);
        let (Some(Owner::Struct(s)), Some(_), true) =
            (fd.owner, &fd.lends, fd.self_mode != SelfMode::None)
        else {
            return None;
        };
        let cached = |p: &Place| {
            let q = self.pl(p)?;
            let (Some(Step::Field(i)), Some(me)) = (q.path.first(), self.f().params.first()) else {
                return None;
            };
            let sd = self.d.s(s);
            (q.root == Root::Local(me.0) && sd.cached.get(*i) == Some(&true))
                .then(|| self.d.text(sd.fields[*i].0).to_string())
        };
        match &e.kind {
            EK::Copy(p) | EK::Lent(p) | EK::AddrOf(p) | EK::Pass(p, ..) => cached(p),
            EK::Inject(x) | EK::Widen(x) => self.lends_cache(x),
            EK::Call(Target::Inst(i), args, _) => {
                let lends = self.lends(*i);
                (args.iter().enumerate())
                    .filter(|(k, _)| lends.contains(k))
                    .find_map(|(_, a)| match a {
                        Arg::Value(v) => self.lends_cache(v),
                        Arg::View(p) | Arg::Lend(p) => cached(p),
                    })
            }
            _ => None,
        }
    }
    fn escapes(&mut self, e: &Expr<CTy>) {
        if self.cur == self.p.main {
            return;
        }
        if let Some(n) = self.lends_cache(e) {
            let msg = format!(
                "this lends `@cache` field `{n}`, which a later read may change (R05.10.9)"
            );
            return self.err(e.site, codes::BAD_PROJECTION, msg);
        }
        let ps: Vec<usize> = self.f().params.iter().map(|p| p.0).collect();
        let lent = self.lends(self.cur);
        let out: BTreeSet<Root> = (self.sources(e).into_iter())
            .map(|p| p.root)
            .filter(|r| match r {
                Root::Global(_) => false,
                Root::Local(l) => {
                    !ps.iter().position(|x| x == l).is_some_and(|i| lent.contains(&i))
                        // A type-parameter value is a copy where an instance with a lent
                        // argument keeps it (`EK::Lent` in link).
                        && !(ps.contains(l) && self.of_param(*l))
                }
            })
            .collect();
        if let Some(r) = out.into_iter().next() {
            let n = self.root_name(r);
            let why = if self.borrowed.contains(&r) {
                format!("{n} is lent by the caller; return a `.clone()`")
            } else {
                format!("{n} is gone when the fn returns")
            };
            self.err(
                e.site,
                codes::LENT_ESCAPES,
                format!("this returns a value that borrows {n}: {why}"),
            );
        }
    }
    /// A move inside a loop of a binding declared before it leaves nothing for the next turn.
    fn moved_in_loop(&mut self, site: u32, at: u32, before: &BTreeSet<Pl>) {
        let gone: Vec<Pl> = (self.moved.difference(before))
            .filter(|p| self.decl.get(&p.root).is_some_and(|(_, o)| *o <= at))
            .cloned()
            .collect();
        for p in gone {
            let n = self.root_name(p.root);
            self.err(
                site,
                codes::USE_AFTER_MOVE,
                format!("{n} is moved inside a loop, so the next turn would use it after the move"),
            );
        }
    }
}

/// Whether temp `r` holds storage a call made, owned by no binding and lent by nothing: it
/// dies at the end of its statement (R07.5.10).
fn fresh(
    made: &BTreeSet<Root>,
    spilled: &HashMap<Root, Pl>,
    lent: &HashMap<Root, BTreeSet<Pl>>,
    r: Root,
) -> bool {
    made.contains(&r) && !spilled.contains_key(&r) && !lent.contains_key(&r)
}

/// Ownership of every instance link made: callees first, each summarized for its callers.
fn run(d: &dyn Signatures, p: &Program, globals: &[(String, bool)]) -> Owned {
    // A literal the program wrote (`[a, b]` through the `@literal("[]")` impl, `"s"` through
    // `@role(str_literal)`) is the program's value, not a call's temporary (R07.5.10).
    let list = d.literal_trait("[]");
    let literals = (p.insts.iter().enumerate())
        .filter(|(_, f)| match &f.key {
            FuncKey::Decl(f) | FuncKey::Ctx(f, _) => {
                d.sugar().str_literal == Some(*f)
                    || matches!(d.f(*f).owner, Some(Owner::Impl(k)) if Some(d.imp(k).trait_.tr) == list)
            }
            _ => false,
        })
        .map(|(i, _)| i)
        .collect();
    // A method's result is cut from its receiver; a fresh value passed to any other parameter
    // moves in, unless the signature says the result `lends` it.
    let receivers = (p.insts.iter().enumerate())
        .filter(|(_, f)| match &f.key {
            FuncKey::Decl(f) | FuncKey::Ctx(f, _) => d.f(*f).self_mode != SelfMode::None,
            _ => false,
        })
        .map(|(i, _)| i)
        .collect();
    let fields = cpl_next_decls::fields::fields(d);
    let makers = (p.insts.iter().enumerate())
        .filter(|(_, f)| match &f.key {
            FuncKey::Decl(f) | FuncKey::Ctx(f, _) => {
                let fd = d.f(*f);
                let nothing = vec![Ty::Unit; fd.generics.len()];
                let ret = Subst {
                    tys: &nothing,
                    sets: &[],
                }
                .ty(&fd.ret);
                ret.holds(&fields, 0) == Holds::Storage
            }
            _ => true,
        })
        .map(|(i, _)| i)
        .collect();
    let mut w = Walk {
        d,
        p,
        globals,
        sums: HashMap::new(),
        cur: p.main,
        lent: HashMap::new(),
        moved: BTreeSet::new(),
        scopes: vec![],
        next: 0,
        decl: HashMap::new(),
        borrowed: BTreeSet::new(),
        consts: HashMap::new(),
        frozen: HashMap::new(),
        kept: BTreeSet::new(),
        closure_params: BTreeSet::new(),
        caps: HashMap::new(),
        spilled: HashMap::new(),
        elems: BTreeSet::new(),
        made: BTreeSet::new(),
        literals,
        receivers,
        makers,
        copies: BTreeSet::new(),
        last: crate::Last::default(),
        pending: vec![],
        report: false,
        errors: vec![],
        binds: vec![],
    };
    let order = w.order();
    // Summaries only grow, so a recursive group settles; then one reporting walk.
    loop {
        let mut changed = false;
        for &i in &order {
            let s = w.instance(i);
            changed |= w.sums.get(&i) != Some(&s);
            w.sums.insert(i, s);
        }
        if !changed {
            break;
        }
    }
    w.report = true;
    for &i in &order {
        w.instance(i);
    }
    Owned {
        errors: w.errors,
        copies: w.copies,
        binds: w.binds,
    }
}

impl Walk<'_> {
    fn instance(&mut self, i: usize) -> Summary {
        self.cur = i;
        let f = &self.p.insts[i];
        self.last = crate::Last::new(&f.body);
        self.lent.clear();
        self.moved.clear();
        self.decl.clear();
        self.consts.clear();
        self.scopes.clear();
        self.pending.clear();
        self.frozen.clear();
        self.kept.clear();
        self.caps.clear();
        self.spilled.clear();
        self.elems.clear();
        self.made.clear();
        self.closure_params.clear();
        let fns = (f.params.iter().skip(f.n_captures).map(|p| p.0))
            .filter(|l| f.declared.get(*l).is_some_and(|d| d.fn_value));
        self.closure_params.extend(fns);
        // A lambda's own params are its values, as in the fn it is written in; its captures are lent.
        let own = |k: usize| matches!(f.key, FuncKey::Lambda(_)) && k >= f.n_captures;
        self.borrowed = (f.params.iter().enumerate())
            .filter(|(k, (l, _))| !own(*k) && !self.p.plain(&f.locals[*l]))
            .map(|(_, p)| p)
            .map(|(l, _)| Root::Local(*l))
            .collect();
        let mut fx = Fx::default();
        let body = &f.body;
        self.block(body, &mut fx);
        let ps: Vec<usize> = f.params.iter().map(|p| p.0).collect();
        let keep = |p: &Pl| match p.root {
            Root::Global(_) => true,
            Root::Local(l) => ps.contains(&l),
        };
        fx.reads.retain(keep);
        fx.writes.retain(keep);
        fx.held.retain(keep);
        Summary {
            fx,
            // What the result lends is the caller's to keep, not kept by the call.
            keeps: {
                let lends = self.lends(i);
                let mut k = std::mem::take(&mut self.kept);
                k.retain(|p| !lends.contains(p));
                k
            },
        }
    }
}

/// A binding or a field path under one: a place a value moves out of whole.
fn fields_only(p: &Place) -> bool {
    match p {
        Place::Local(_) | Place::Global(_) => true,
        Place::Field(b, _) => fields_only(b),
        _ => false,
    }
}

/// Every call in a body: its target, and the fn value it calls through.
type Calls<'b, T> = dyn FnMut(&'b Target<T>, Option<&'b Expr<T>>) + 'b;
/// Every call in `b`; a call of a fn value is `Target::Case` with the value.
fn calls_in<'b, T>(b: &'b [Stmt<T>], f: &mut Calls<'b, T>) {
    fn args<'b, T>(xs: &'b [Arg<T>], f: &mut Calls<'b, T>) {
        for a in xs {
            if let Arg::Value(v) = a {
                expr(v, f)
            }
        }
    }
    fn expr<'b, T>(e: &'b Expr<T>, f: &mut Calls<'b, T>) {
        match &e.kind {
            EK::Call(t, xs, _) => {
                f(t, None);
                args(xs, f)
            }
            EK::Intr(_, xs, _) => args(xs, f),
            EK::CallValue(c, xs, _) => {
                f(&const { Target::Case }, Some(c));
                expr(c, f);
                args(xs, f)
            }
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| expr(x, f)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => expr(x, f),
            _ => {}
        }
    }
    for s in b {
        match s {
            Stmt::Let(_, e)
            | Stmt::Assign(_, e)
            | Stmt::Expr(e)
            | Stmt::Yield(e)
            | Stmt::Fact(e) => expr(e, f),
            Stmt::Return(e) => e.iter().for_each(|e| expr(e, f)),
            Stmt::Check { cond, .. } => expr(cond, f),
            Stmt::If(c, a, b2) => {
                expr(c, f);
                calls_in(a, f);
                calls_in(b2, f);
            }
            Stmt::Loop(a, b2, _) => {
                calls_in(a, f);
                calls_in(b2, f);
            }
            Stmt::Switch(c, arms) => {
                expr(c, f);
                arms.iter().for_each(|a| calls_in(a, f));
            }
            Stmt::Region {
                target,
                args: xs,
                body,
                ..
            } => {
                f(target, None);
                args(xs, f);
                calls_in(body, f);
            }
            Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
                xs.iter().for_each(|x| expr(x, f))
            }
            Stmt::Break | Stmt::Continue | Stmt::LendOut(_) | Stmt::Drop(_) => {}
        }
    }
}

/// R05.10.12: a method holding a lend into a `@cache` field of `self` calls no fn value, which
/// could reach the struct and write the field under that lend. A rule on the body as written, so
/// it reads generic IR: an uncalled method has no instance.
pub(crate) fn cache_calls(d: &dyn Signatures, generic: &HashMap<FuncKey, Func<Ty>>) -> Vec<Msg> {
    use cpl_next_decls::decls::{Owner, SelfMode};
    let mut keys: Vec<&FuncKey> = generic.keys().collect();
    keys.sort();
    let mut out = Vec::new();
    for k in keys {
        let FuncKey::Decl(fid) = k else { continue };
        let fd = d.f(*fid);
        let (Some(Owner::Struct(s)), true) = (fd.owner, fd.self_mode != SelfMode::None) else {
            continue;
        };
        let cached = &d.s(s).cached;
        let g = &generic[k];
        let Some(me) = g.params.first().map(|p| p.0) else {
            continue;
        };
        let lends_cache = |p: &Place| {
            let mut p = p;
            while let Place::Field(inner, i) = p {
                if matches!(**inner, Place::Local(l) if l == me) {
                    return cached.get(*i) == Some(&true);
                }
                p = inner;
            }
            false
        };
        let mut sites = Vec::new();
        holding(&g.body, &lends_cache, &mut sites);
        for site in sites {
            let msg = "this calls a fn value while a `@cache` field of `self` is lent: it could \
                       write the field under that lend (R05.10.12)";
            let e = Msg::at(
                g.sites[site as usize].clone(),
                Msg::verdict(codes::USED_WHILE_LENT, msg),
            );
            if !out.contains(&e) {
                out.push(e);
            }
        }
    }
    out
}

/// The sites of fn-value calls inside the regions of `b` that lend a place `lent` accepts.
fn holding<T>(b: &[Stmt<T>], lent: &dyn Fn(&Place) -> bool, out: &mut Vec<u32>) {
    for s in b {
        match s {
            Stmt::If(_, a, b2) | Stmt::Loop(a, b2, _) => {
                holding(a, lent, out);
                holding(b2, lent, out);
            }
            Stmt::Switch(_, arms) => arms.iter().for_each(|a| holding(a, lent, out)),
            Stmt::Region { args, body, .. } => {
                let held = args.iter().any(|a| match a {
                    Arg::Lend(p) | Arg::View(p) => lent(p),
                    Arg::Value(Expr {
                        kind: EK::Pass(p, ..),
                        ..
                    }) => lent(p),
                    Arg::Value(_) => false,
                });
                if held {
                    calls_in(body, &mut |t, c| match (t, c) {
                        (_, Some(c)) => out.push(c.site),
                        (Target::Value(v), None) => out.push(v.site),
                        _ => {}
                    });
                } else {
                    holding(body, lent, out);
                }
            }
            _ => {}
        }
    }
}

/// What ownership decides: its errors, and which passed places (`EK::Pass`, by instance and
/// site) are copies (`place_copies`).
#[derive(Default)]
pub struct Owned {
    pub errors: Vec<Msg>,
    pub copies: BTreeSet<(usize, Site)>,
    pub binds: Vec<Bound>,
}

/// `r`'s binding in instance `inst`, if it freezes or lends any place.
fn bound(
    inst: usize,
    r: Root,
    frozen: &HashMap<Root, BTreeSet<Pl>>,
    lent: &HashMap<Root, BTreeSet<Pl>>,
) -> Option<Bound> {
    let (frozen, lent) = (frozen.get(&r), lent.get(&r));
    if frozen.is_none() && lent.is_none() {
        return None;
    }
    let all = |ps: Option<&BTreeSet<Pl>>| ps.into_iter().flatten().cloned().collect();
    Some(Bound {
        inst,
        local: r,
        frozen: all(frozen),
        lent: all(lent),
    })
}

/// A local where it is bound, in instance `inst`: the places a write to which conflicts with it
/// while it is used (`frozen`, E0602) and the places it lends (`lent`, R8), as own holds them.
/// The facade names them for `--dump-own`.
#[derive(Debug)]
pub struct Bound {
    pub inst: usize,
    pub local: Root,
    pub frozen: Vec<Pl>,
    pub lent: Vec<Pl>,
}

/// Ownership after link (contract 4): checks the instances link made, reporting what
/// [`crate::before`] found on the generic fns too, then settles their passed places.
pub fn after(
    d: &dyn Signatures,
    p: &mut Program,
    globals: &[(String, bool)],
    before: crate::Before,
) -> Owned {
    use cpl_next_ir::verify::{all, callees, instances, Stage};
    // A program link rejected can be partial (it stops at an instantiation that does not
    // end), so ownership has nothing sound to walk; link's error is the answer.
    let linked = p.errors.is_empty();
    p.errors.extend(before.generic);
    if !linked {
        return Owned {
            errors: before.cache,
            ..Owned::default()
        };
    }
    let fast = callees(&p.insts);
    p.errors
        .extend(all(&p.insts, Stage::Instances, &fast, p.globals.len()));
    let mut owned = run(d, p, globals);
    owned.errors.extend(before.cache);
    owned.errors.extend(lenders_lend(p));
    if p.errors.is_empty() {
        owned.errors.extend(escape(d, p));
    }
    crate::place_copies(p, &owned.copies);
    let raw = instances(&p.insts, Stage::Raw, p.globals.len());
    p.errors.extend(raw);
    owned
}

/// A lender lends one place on every path through it: its one expression, or a `lend mut`
/// (R05.6.3). A path that leaves without one would lose the caller's write.
fn lenders_lend(p: &Program) -> Vec<Msg> {
    /// Whether a path through `b` reaches its end, and whether one leaves the fn, unlent.
    fn gap(b: &[Stmt<CTy>]) -> (bool, bool) {
        let (mut on, mut left) = (true, false);
        for s in b {
            if !on {
                break;
            }
            match s {
                Stmt::LendOut(_) | Stmt::Break | Stmt::Continue => on = false,
                Stmt::Return(_) => (on, left) = (false, true),
                Stmt::If(_, a, b) => {
                    let ((fa, la), (fb, lb)) = (gap(a), gap(b));
                    (on, left) = (fa || fb, left || la || lb);
                }
                Stmt::Switch(_, arms) if !arms.is_empty() => {
                    let gs: Vec<_> = arms.iter().map(|a| gap(a)).collect();
                    on = gs.iter().any(|g| g.0);
                    left |= gs.iter().any(|g| g.1);
                }
                // A lend's body runs once; a loop's or a walk's may not run, and lends nothing.
                Stmt::Region {
                    lend: true, body, ..
                } => {
                    let (f, l) = gap(body);
                    (on, left) = (f, left || l);
                }
                Stmt::Loop(body, step, _) => left |= gap(body).1 || gap(step).1,
                Stmt::Region { body, .. } => left |= gap(body).1,
                _ => {}
            }
        }
        (on, left)
    }
    let mut bad = Vec::new();
    for f in &p.insts {
        if !matches!(f.region, Some((true, _))) || gap(&f.body) == (false, false) {
            continue;
        }
        let last = f.body.last().map(std::slice::from_ref).unwrap_or_default();
        let site = site_of(last, &|_| true).or_else(|| site_of(&f.body, &|_| true));
        let m = Msg::verdict(
            codes::UNSUPPORTED,
            format!(
                "`{}` is declared `lends mut`, so every path through it ends in `lend mut place`; \
                 a body that leaves without one would lose the caller's write (R05.6.3)",
                f.shown
            ),
        );
        let m = match site.and_then(|i| f.sites.get(i)) {
            Some(s) => Msg::at(s.clone(), m),
            None => m,
        };
        if !bad.contains(&m) {
            bad.push(m);
        }
    }
    bad
}

/// A closure that shares a `var` of the fn around it is lent: it cannot be returned (spec R05.10).
/// Nor can it be stored through a `mut` parameter.
fn escape(d: &dyn Signatures, p: &Program) -> Vec<Msg> {
    // `made` keeps the closures the returning fn itself makes: one passed in through a
    // container's `new` or `at` captures what its maker's frame holds, which outlives them.
    fn holds_ref(
        p: &Program,
        t: &CTy,
        seen: &mut Vec<CTy>,
        made: &dyn Fn(&InstKey) -> bool,
    ) -> bool {
        if seen.contains(t) {
            return false;
        }
        seen.push(t.clone());
        match t {
            CTy::Ref(_) => true,
            CTy::Raw(e) => holds_ref(p, e, seen, made),
            CTy::Struct(..) => p.parts(t).iter().any(|f| holds_ref(p, f, seen, made)),
            CTy::Fn(_, _, ms) => ms
                .iter()
                .any(|k| holds_ref(p, &CTy::Clo(Box::new(k.clone())), seen, made)),
            CTy::Clo(k) if made(k) => p.captures(k).iter().any(|c| holds_ref(p, c, seen, made)),
            _ => false,
        }
    }
    let mut bad = Vec::new();
    for (me, f) in p.insts.iter().enumerate() {
        let owner = match f.key {
            FuncKey::Decl(g) | FuncKey::Ctx(g, _) => Some(g),
            _ => None,
        };
        // A generator's frame ends with its walk, so a closure made in one outlives it anywhere.
        let made = |k: &InstKey| match (owner, &k.func) {
            (Some(g), FuncKey::Lambda(_)) => {
                let o = (p.index.get(k)).and_then(|&i| p.insts.get(i)?.maker.as_ref()?.decl());
                o == Some(g) || o.is_some_and(|o| matches!(d.f(o).region, Some((false, _))))
            }
            (Some(_), _) => false,
            (None, _) => true,
        };
        // Said where the fn makes the closure, or else where it is called.
        let at = |bad: &mut Vec<Msg>, m: String| {
            let made_here = |e: &Expr<CTy>| {
                matches!((&e.kind, &e.ty), (EK::MakeClosure(_), CTy::Clo(k))
                    if made(k) && p.captures(k).iter().any(|c| matches!(c, CTy::Ref(_))))
            };
            let calls_me =
                |e: &Expr<CTy>| matches!(&e.kind, EK::Call(Target::Inst(j), ..) if *j == me);
            let site = (site_of(&f.body, &made_here).and_then(|i| f.sites.get(i))).or_else(|| {
                (p.insts.iter())
                    .find_map(|g| site_of(&g.body, &calls_me).and_then(|i| g.sites.get(i)))
            });
            let m = Msg::verdict(codes::LENT_ESCAPES, m);
            bad.push(match site {
                Some(s) => Msg::at(s.clone(), m),
                None => m,
            });
        };
        if holds_ref(p, &f.ret, &mut Vec::new(), &made) {
            // A lambda's lowered name is no name the program wrote.
            let who = match f.key {
                FuncKey::Lambda(_) => "this lambda".to_string(),
                _ => format!("`{}`", f.name),
            };
            at(&mut bad, format!("{who} returns a function that uses a `var` of the fn that made it; the variable is gone by then"));
        }
        // What a `mut` parameter holds is the caller's, so a closure stored there outlives this fn too.
        let stored = (f.params.iter().filter(|(_, m)| *m)).any(|(l, _)| {
            let t = match &f.locals[*l] {
                CTy::Ref(t) => t,
                t => t,
            };
            owner.is_some() && holds_ref(p, t, &mut Vec::new(), &made)
        });
        if stored {
            at(&mut bad, format!("`{}` stores a function that uses one of its `var`s through a `mut` parameter; the variable is gone when it returns", f.name));
        }
    }
    bad
}

/// The site of the first expression of `b` that `hit` takes, outermost first.
fn site_of(b: &[Stmt<CTy>], hit: &dyn Fn(&Expr<CTy>) -> bool) -> Option<usize> {
    fn ex(e: &Expr<CTy>, hit: &dyn Fn(&Expr<CTy>) -> bool) -> Option<usize> {
        if hit(e) {
            return Some(e.site as usize);
        }
        let args = |xs: &[Arg<CTy>]| {
            xs.iter().find_map(|a| match a {
                Arg::Value(v) => ex(v, hit),
                Arg::View(_) | Arg::Lend(_) => None,
            })
        };
        match &e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().find_map(|x| ex(x, hit)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => ex(x, hit),
            EK::Intr(_, xs, _) | EK::Call(_, xs, _) => args(xs),
            EK::CallValue(g, xs, _) => ex(g, hit).or_else(|| args(xs)),
            EK::Copy(_)
            | EK::Pass(..)
            | EK::AddrOf(_)
            | EK::Lent(_)
            | EK::Tag(_)
            | EK::Int(_)
            | EK::F64(_)
            | EK::Bool(_)
            | EK::Unit
            | EK::Text(_)
            | EK::DomLen(_)
            | EK::DomLo(_)
            | EK::DomHeld(_)
            | EK::DomCounted(_)
            | EK::TypeValue(_)
            | EK::HasBound(..) => None,
        }
    }
    b.iter().find_map(|s| match s {
        Stmt::Let(_, e) | Stmt::Assign(_, e) => ex(e, hit),
        Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) | Stmt::Check { cond: e, .. } => ex(e, hit),
        Stmt::Return(e) => e.as_ref().and_then(|e| ex(e, hit)),
        Stmt::If(c, x, y) => ex(c, hit)
            .or_else(|| site_of(x, hit))
            .or_else(|| site_of(y, hit)),
        Stmt::Loop(x, y, _) => site_of(x, hit).or_else(|| site_of(y, hit)),
        Stmt::Switch(c, arms) => ex(c, hit).or_else(|| arms.iter().find_map(|a| site_of(a, hit))),
        Stmt::Region { args: xs, body, .. } => (xs.iter())
            .find_map(|a| match a {
                Arg::Value(v) => ex(v, hit),
                Arg::View(_) | Arg::Lend(_) => None,
            })
            .or_else(|| site_of(body, hit)),
        Stmt::Event(cpl_next_ir::ir::EventPoint { vals: xs, .. }) => {
            xs.iter().find_map(|x| ex(x, hit))
        }
        Stmt::LendOut(_) | Stmt::Drop(_) | Stmt::Break | Stmt::Continue => None,
    })
}

/// The local a value is a copy of, through widening a fn value's closure set.
fn copy_of(e: &Expr<CTy>) -> Option<usize> {
    match &e.kind {
        EK::Widen(x) => copy_of(x),
        EK::Copy(Place::Local(l)) | EK::Pass(Place::Local(l), ..) => Some(*l),
        _ => None,
    }
}

/// Whether an argument is one of `params`, the locals holding the fn's own fn-valued
/// parameters (R05.10.4).
fn closure_param(params: &BTreeSet<usize>, a: &Arg<CTy>) -> bool {
    match a {
        Arg::Value(e) => copy_of(e).is_some_and(|l| params.contains(&l)),
        Arg::View(Place::Local(l)) | Arg::Lend(Place::Local(l)) => params.contains(l),
        _ => false,
    }
}
