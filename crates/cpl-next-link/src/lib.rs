//! Contract 5: generic IR -> instances. The link step: it decides which
//! instances exist and what each function-value set holds, from the whole program.
use cpl_next_decls::decls::Decls;
use cpl_next_decls::diag::Msg;
use cpl_next_ir::ir::*;
use cpl_next_ir::ty::*;
use cpl_next_ir::unbuilt::Unbuilt;
use cpl_next_ir::whole::{DomId, DomValue, Whole};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub use cpl_next_ir::prog::{parts, CSet, CTy, InstKey, Program};

mod share;

struct Link<'a> {
    /// The type of a slot a value of a type is lent in: ownership's rule (`cpl_next_own`).
    lent: &'a Lend,
    d: &'a Decls,
    whole: &'a Whole,
    generic: &'a HashMap<FuncKey, Func<Ty>>,
    adds: &'a SetAdds,
    keys: Vec<InstKey>,
    index: HashMap<InstKey, usize>,
    structs: HashMap<CTy, Vec<CTy>>,
    sums: HashMap<CTy, Vec<CTy>>,
    add_busy: BTreeSet<usize>,
    /// The globals' sets being read, so one met inside itself adds nothing more.
    global_busy: BTreeSet<(usize, usize)>,
    /// The lambdas whose named sets are being read, so one met inside itself keeps all.
    named_busy: std::collections::HashSet<FuncKey>,
    named: HashMap<FuncKey, BTreeSet<usize>>,
    errors: Vec<Msg>,
    globals: Vec<CTy>,
    /// The locals of the instance being made, to type its places.
    frame: Vec<CTy>,
}
struct Env<'e> {
    targs: &'e [CTy],
    sets: &'e [CSet],
}

const LIMIT: usize = 400;

/// The type parameters of `s` that a field of it, or of one of its variants, holds.
fn held_params(d: &Decls, s: StructId) -> BTreeSet<usize> {
    let sd = d.s(s);
    let mut out = BTreeSet::new();
    let payloads = sd.variants.iter().flat_map(|(_, p)| &d.s(*p).fields);
    (sd.fields.iter().chain(payloads)).for_each(|(_, t)| params_in(t, &mut out));
    out
}

/// A trait's fn as an item with no `Self` (`+` passed uncalled): it has no impl, and nothing runs it.
fn selfless(d: &Decls, key: &InstKey) -> bool {
    let FuncKey::Decl(m) = key.func else {
        return false;
    };
    matches!(d.f(m).body, cpl_syntax::ast::FnBody::None)
        && (key.targs.first()).is_none_or(|t| *t == CTy::Error)
}
/// The key of a generic fn item as a value nothing calls (`type_of(max)`), one for every
/// instance of the fn (R08.1.10): no body runs under it.
fn item_value(d: &Decls, key: &InstKey) -> bool {
    let FuncKey::Decl(m) = key.func else {
        return false;
    };
    key.targs.is_empty() && !d.f(m).generics.is_empty()
}
/// An item value's instance: a value with nothing to run, so no frame and no body.
fn value_only(f: &Func<Ty>) -> Func<CTy> {
    Func {
        key: f.key.clone(),
        name: f.name.clone(),
        params: Vec::new(),
        n_captures: 0,
        names: Vec::new(),
        vars: Vec::new(),
        locals: Vec::new(),
        body: Vec::new(),
        ret: CTy::Unit,
        region: None,
        line: f.line,
        sites: f.sites.clone(),
        lines: f.lines.clone(),
        views: Vec::new(),
        drops: Vec::new(),
        maker: f.maker,
        shown: f.shown.clone(),
        fast: None,
    }
}
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
        CTy::Fn(_, _, set) => set
            .iter()
            .for_each(|k| k.targs.iter().for_each(|t| doms_of(t, out))),
        CTy::Clo(k) => k.targs.iter().for_each(|t| doms_of(t, out)),
        _ => {}
    }
}

/// A fn with one instance: no type parameters, sets or context of its own or its owner's,
/// and no closure in its signature.
fn closed_fn(d: &Decls, f: FnId) -> bool {
    let fd = d.f(f);
    let owner_open =
        matches!(&fd.self_ty, Some(Ty::Struct(_, ts, ss)) if !ts.is_empty() || !ss.is_empty());
    fd.generics.is_empty()
        && fd.n_sets == 0
        && fd.uses.is_empty()
        && fd.intrinsic.is_none()
        && !owner_open
        && !matches!(fd.owner, Some(cpl_next_decls::decls::Owner::Trait(_)))
}

/// Given a type and how to read its parts, the type of a slot a value of it is lent in.
pub type Lend = dyn Fn(CTy, &dyn Fn(&CTy) -> Vec<CTy>) -> CTy;

/// A closure in a signature takes its set from the calls that reach the fn.
pub fn link(
    d: &Decls,
    whole: &Whole,
    deferred: &cpl_next_ir::msg::Deferred,
    generic: &HashMap<FuncKey, Func<Ty>>,
    globals: &[Ty],
    adds: &SetAdds,
    lent: &Lend,
) -> Program {
    let mut l = Link {
        d,
        whole,
        lent,
        generic,
        adds,
        keys: Vec::new(),
        index: HashMap::new(),
        structs: HashMap::new(),
        sums: HashMap::new(),
        add_busy: BTreeSet::new(),
        global_busy: BTreeSet::new(),
        named_busy: std::collections::HashSet::new(),
        named: HashMap::new(),
        errors: Vec::new(),
        globals: Vec::new(),
        frame: Vec::new(),
    };
    let none = Env {
        targs: &[],
        sets: &[],
    };
    l.globals = globals
        .iter()
        .map(|t| l.ty(t, &none).unlent().clone())
        .collect();
    let main = l.intern(InstKey {
        func: FuncKey::Main,
        targs: vec![],
        sets: vec![],
    });
    // Every closed fn of the program is an instance too, called or not, so ownership checks
    // each one once; a backend emits only what main reaches.
    let mut closed: Vec<FuncKey> = (generic.iter())
        .filter(|(k, g)| g.line != 0 && matches!(k, FuncKey::Decl(f) if closed_fn(d, *f)))
        .map(|(k, _)| k.clone())
        .collect();
    closed.sort();
    // Uncalled, a fn's result sets are what its own body puts there.
    for func in closed {
        let FuncKey::Decl(f) = func else { continue };
        let fd = d.f(f);
        let sets = (0..fd.n_sets + fd.n_ret_sets)
            .map(|i| match l.adds(f, i) {
                Some(s) => l.set(&s, &none),
                None => CSet::default(),
            })
            .collect();
        l.intern(InstKey {
            func,
            targs: vec![],
            sets,
        });
    }
    let mut insts: Vec<Func<CTy>> = Vec::new();
    let mut next = 0;
    let mut said = std::collections::HashSet::new();
    while next < l.keys.len() {
        if l.keys.len() > LIMIT {
            l.errors.push(Msg::verdict(cpl_diag::codes::GROWING_INSTANTIATION, format!("instantiation does not end: `{}` makes a new copy of itself at every level (a function passed to its own recursive call, wrapped in a new closure)", generic[&l.keys[next].func].name)));
            break;
        }
        let key = l.keys[next].clone();
        if let Some(f) = key.func.decl() {
            if said.insert(f) {
                l.errors.extend(deferred.of(f).iter().cloned());
            }
        }
        // A fn used at a domain declared in its own body: inside that instance the type
        // argument and the local domain would be one type with two sizes. A run-time value
        // has to be keyed by its binding for this to run; until then it is refused.
        let owner = match &key.func {
            FuncKey::Decl(f) | FuncKey::Ctx(f, _) => Some(*f),
            FuncKey::Lambda(_) => generic[&key.func].maker,
            FuncKey::Main | FuncKey::Case => None,
        };
        let mut doms = Vec::new();
        key.targs.iter().for_each(|t| doms_of(t, &mut doms));
        if let Some(d) = doms
            .iter()
            .find(|d| owner.is_some() && whole.dom(DomId(**d)).owner == owner)
        {
            l.errors.push(
                Unbuilt::DomainTypeArg {
                    func: generic[&key.func].name.clone(),
                    domain: Some(whole.dom(DomId(*d)).name.clone()),
                }
                .into(),
            );
            break;
        }
        let f = &generic[&key.func];
        let env = Env {
            targs: &key.targs,
            sets: &key.sets,
        };
        let mut inst = match item_value(d, &key) {
            true => value_only(f),
            false => l.instance(f, &env),
        };
        if selfless(d, &key) {
            inst.body.clear();
        }
        insts.push(inst);
        next += 1;
    }
    let globals = l.globals.clone();
    // Instances made while typing the globals.
    while insts.len() < l.keys.len() && l.keys.len() <= LIMIT {
        let key = l.keys[insts.len()].clone();
        let f = &generic[&key.func];
        let env = Env {
            targs: &key.targs,
            sets: &key.sets,
        };
        let mut inst = match item_value(d, &key) {
            true => value_only(f),
            false => l.instance(f, &env),
        };
        if selfless(d, &key) {
            inst.body.clear();
        }
        insts.push(inst);
    }
    let mut p = Program {
        insts,
        index: l.index,
        main,
        globals,
        structs: l.structs,
        sums: l.sums,
        shares: HashMap::new(),
        errors: l.errors,
    };
    if p.errors.is_empty() {
        thread_values(&mut p, whole);
    }
    if p.errors.is_empty() {
        check_finite(&mut p);
    }
    p
}

impl Link<'_> {
    fn parts(&self, t: &CTy) -> Vec<CTy> {
        parts(&self.structs, &self.sums, t)
    }
    fn instance(&mut self, f: &Func<Ty>, env: &Env) -> Func<CTy> {
        // A closure's lent captures stay pointers; every other local owns its value.
        let lent_caps: Vec<usize> = f.params[..f.n_captures].iter().map(|(l, _)| *l).collect();
        let locals: Vec<CTy> = (f.locals.iter().enumerate())
            .map(|(i, t)| match self.ty(t, env) {
                t if lent_caps.contains(&i) => t,
                t => t.unlent().clone(),
            })
            .collect();
        self.frame = locals.clone();
        Func {
            key: f.key.clone(),
            name: f.name.clone(),
            params: f.params.clone(),
            n_captures: f.n_captures,
            names: f.names.clone(),
            vars: f.vars.clone(),
            locals,
            body: self.block(&f.body, env),
            ret: self.ty(&f.ret, env).unlent().clone(),
            region: f.region.as_ref().map(|(lend, t)| (*lend, self.ty(t, env))),
            line: f.line,
            sites: f.sites.clone(),
            lines: f.lines.clone(),
            views: Vec::new(),
            drops: Vec::new(),
            maker: f.maker,
            shown: f.shown.clone(),
            fast: f.fast,
        }
    }
    /// `p` read through each lent slot on its path.
    fn place(&self, p: &Place) -> Place {
        let sub = |b: &Place| Box::new(self.place(b));
        match p {
            Place::Field(b, i) => {
                let f = Place::Field(sub(b), *i);
                match self.place_ty(&f) {
                    Some(CTy::Lent(_)) => Place::Through(Box::new(f)),
                    _ => f,
                }
            }
            Place::Variant(b, i) => Place::Variant(sub(b), *i),
            Place::Deref(b) => Place::Deref(sub(b)),
            // A lent capture of a plain value is the value itself.
            Place::Through(b) => {
                let b = self.place(b);
                match self.place_ty(&b) {
                    Some(CTy::Lent(_)) => Place::Through(Box::new(b)),
                    _ => b,
                }
            }
            Place::Slot(b, l) => Place::Slot(sub(b), *l),
            Place::Local(_) | Place::Global(_) => p.clone(),
        }
    }
    fn place_ty(&self, p: &Place) -> Option<CTy> {
        match p {
            Place::Local(l) => self.frame.get(*l).cloned(),
            Place::Global(g) => self.globals.get(*g).cloned(),
            Place::Field(b, i) => self.structs.get(&self.place_ty(b)?)?.get(*i).cloned(),
            Place::Variant(b, i) => self.sums.get(&self.place_ty(b)?)?.get(*i).cloned(),
            Place::Deref(b) | Place::Through(b) | Place::Slot(b, _) => match self.place_ty(b)? {
                CTy::Ref(t) | CTy::Lent(t) | CTy::Raw(t) => Some(*t),
                _ => None,
            },
        }
    }
    fn intern(&mut self, k: InstKey) -> usize {
        if let Some(i) = self.index.get(&k) {
            return *i;
        }
        self.keys.push(k.clone());
        self.index.insert(k, self.keys.len() - 1);
        self.keys.len() - 1
    }
    /// A type argument as written: a constant range keeps its ends, so its instance is its own.
    fn targ(&mut self, t: &Ty, env: &Env) -> CTy {
        match t {
            Ty::Range(lo, hi) => CTy::Range(*lo, *hi),
            Ty::RangeTo(d, _, lo, k) => CTy::RangeTo(Box::new(self.targ(d, env)), *lo, *k),
            Ty::Param(i) => env.targs.get(*i).cloned().unwrap_or(CTy::Error),
            t => self.ty(t, env),
        }
    }
    fn ty(&mut self, t: &Ty, env: &Env) -> CTy {
        match t {
            Ty::Int | Ty::Range(..) | Ty::RangeOf(_) | Ty::RangeTo(..) => CTy::Int,
            Ty::Bool => CTy::Bool,
            Ty::Char => CTy::Char,
            Ty::U64 => CTy::U64,
            Ty::I128 => CTy::I128,
            Ty::F64 => CTy::F64,
            Ty::Unit => CTy::Unit,
            Ty::Raw(e) => CTy::Raw(Box::new(self.ty(e, env))),
            Ty::Ref(e) => CTy::Ref(Box::new(self.ty(e, env))),
            // The place a `mut` parameter is lent has the parameter's own type.
            Ty::Mut(e) => self.ty(e, env),
            Ty::Lent(e) => match self.ty(e, env) {
                CTy::Lent(t) => CTy::Lent(t),
                t => (self.lent)(t, &|s| self.parts(s)),
            },
            Ty::Param(i) | Ty::AnyRange(i) => match env.targs.get(*i) {
                Some(CTy::Range(..) | CTy::RangeTo(..)) => CTy::Int,
                t => t.cloned().unwrap_or(CTy::Error),
            },
            Ty::Dom(d) => CTy::Dom(*d),
            Ty::Struct(d, ts, ss) => {
                // A lend changes a struct only through a field that holds it: `Version<S>`
                // keeps no `S`, so `Version<lent Arr>` is `Version<Arr>`.
                let held = held_params(self.d, *d);
                let ts: Vec<CTy> = (ts.iter().enumerate())
                    .map(|(i, t)| match self.targ(t, env) {
                        t if held.contains(&i) => t,
                        t => t.unlent().clone(),
                    })
                    .collect();
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
                    let own: Vec<Ty> = (0..ts.len()).map(Ty::Param).collect();
                    let payloads: Vec<CTy> = (self.d.s(*d).variants.clone().iter())
                        .map(|(_, p)| self.ty(&Ty::Struct(*p, own.clone(), Vec::new()), &inner))
                        .collect();
                    if !payloads.is_empty() {
                        self.sums.insert(out.clone(), payloads);
                    }
                }
                out
            }
            Ty::Fn(ps, r, s) => CTy::Fn(
                ps.iter()
                    .map(|p| self.ty(p, env).unlent().clone())
                    .collect(),
                Box::new(self.ty(r, env).unlent().clone()),
                self.set(s, env),
            ),
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
            func: match f.ctx {
                0 => FuncKey::Decl(f.decl),
                k => FuncKey::Ctx(f.decl, k),
            },
            targs: f.targs.iter().map(|t| self.targ(t, env)).collect(),
            sets: f.sets.iter().map(|s| self.set(s, env)).collect(),
        }
    }
    /// The set parameters a fn's generic IR names anywhere.
    fn params_named(&mut self, k: FuncKey) -> BTreeSet<usize> {
        if let Some(u) = self.named.get(&k) {
            return u.clone();
        }
        let text = self
            .generic
            .get(&k)
            .map(|f| format!("{:?} {:?} {:?}", f.locals, f.ret, f.body))
            .unwrap_or_default();
        self.named_busy.insert(k.clone());
        let text = self.kept_text(&text);
        self.named_busy.remove(&k);
        let mut u = BTreeSet::new();
        for part in text.split("SetParam(").skip(1) {
            let digits: String = part.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(i) = digits.parse() {
                u.insert(i);
            }
        }
        self.named.insert(k, u.clone());
        u
    }
    /// `text` with each lambda's set arguments kept only where that lambda names them: a set
    /// a nested closure is given but never uses is no set the fn around it names.
    fn kept_text(&mut self, text: &str) -> String {
        const AT: &str = "Lambda(LambdaId(";
        let mut out = String::new();
        let mut rest = text;
        while let Some(p) = rest.find(AT) {
            let (head, tail) = rest.split_at(p + AT.len());
            out.push_str(head);
            let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
            let lists = (tail.find('['))
                .and_then(|a| Some((a, closing(tail, a)?)))
                .and_then(|(a, b)| {
                    let c = b + tail[b..].find('[')?;
                    Some((a, c, closing(tail, c)?))
                });
            let (Ok(id), Some((_, c, d))) = (digits.parse::<usize>(), lists) else {
                rest = tail;
                continue;
            };
            let key = FuncKey::Lambda(LambdaId(id));
            let used = (!self.named_busy.contains(&key)).then(|| self.params_named(key));
            out.push_str(&self.kept_text(&tail[..=c]));
            let kept: Vec<String> = (top_level(&tail[c + 1..d]).into_iter().enumerate())
                .map(
                    |(i, el)| match used.as_ref().is_none_or(|u| u.contains(&i)) {
                        true => self.kept_text(el),
                        false => "_".to_string(),
                    },
                )
                .collect();
            out.push_str(&kept.join(", "));
            rest = &tail[d..];
        }
        out.push_str(rest);
        out
    }
    fn clo(&mut self, c: &CloRef, env: &Env) -> InstKey {
        match c {
            CloRef::Item(f) => {
                let k = self.fnref(f, env);
                // A fn item nothing calls is one value of the item, not of an instance (R08.1.10).
                match k.targs.contains(&CTy::Error) {
                    true => InstKey {
                        func: k.func,
                        targs: Vec::new(),
                        sets: Vec::new(),
                    },
                    false => k,
                }
            }
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
                    targs: ts.iter().map(|t| self.targ(t, env)).collect(),
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
    /// The instance a call of `f` runs: for a trait's fn, the impl's fn the type picks.
    fn callee_key(&mut self, f: &FnRef, env: &Env) -> Option<InstKey> {
        match self.d.f(f.decl).owner {
            Some(cpl_next_decls::decls::Owner::Trait(tr)) => {
                let (st, args) = f.targs.split_first()?;
                let method = self.d.f(f.decl).name.clone();
                let mut k = self.trait_key(&tr, args, &method, st, env)?;
                self.in_copy(&mut k, Some(f.decl), f.ctx);
                let base = self.d.f(f.decl).n_sets;
                let ret = (f.sets.iter().skip(base))
                    .map(|s| self.set(s, env))
                    .collect();
                self.with_ret_sets(&mut k, ret);
                Some(k)
            }
            _ => Some(self.fnref(f, env)),
        }
    }
    /// A trait's fn's context copy runs the picked fn's copy under the same values (R09.1.3).
    fn in_copy(&self, k: &mut InstKey, trait_fn: Option<FnId>, ctx: u32) {
        let FuncKey::Decl(g) = k.func else { return };
        let copy = match Some(g) == trait_fn {
            true => Some(ctx),
            false => self.whole.ctx_impls.get(&(ctx, g)).copied(),
        };
        if let (Some(c), true) = (copy, ctx != 0) {
            k.func = FuncKey::Ctx(g, c);
        }
    }
    /// What `f`'s body adds to its set `slot`.
    fn adds(&self, f: FnId, slot: usize) -> Option<SetTy> {
        self.adds.params.get(&(f, slot)).cloned()
    }
    /// An impl's fn takes the sets its caller passes for its result after the impl's own.
    fn with_ret_sets(&self, k: &mut InstKey, ret: Vec<CSet>) {
        let Some(g) = k.func.decl() else {
            return;
        };
        let fd = self.d.f(g);
        if fd.n_ret_sets == 0 {
            return;
        }
        k.sets.resize(fd.n_sets, CSet::default());
        k.sets.extend(ret.into_iter().take(fd.n_ret_sets));
        k.sets.resize(fd.n_sets + fd.n_ret_sets, CSet::default());
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
            SetTy::AddedAt(c) if !self.adds.live.get(*c).copied().unwrap_or(true) => {}
            SetTy::AddedAt(c) => {
                // A call met again inside its own arguments' sets adds nothing more.
                if !self.add_busy.insert(*c) {
                    return;
                }
                let (f, slot) = self.adds.calls[*c].clone();
                if let Some(k) = self.callee_key(&f, env) {
                    if let Some(decl) = k.func.decl() {
                        // A trait's result slot is the impl's, after the impl's own sets.
                        let base = self.d.f(f.decl).n_sets;
                        let slot = match decl != f.decl && slot >= base {
                            true => self.d.f(decl).n_sets + slot - base,
                            false => slot,
                        };
                        if let Some(s) = self.adds(decl, slot) {
                            let inner = Env {
                                targs: &k.targs,
                                sets: &k.sets,
                            };
                            self.set_into(&s, &inner, out);
                        }
                    }
                }
                self.add_busy.remove(c);
            }
            // A global is the top level's: what the program puts in it names no fn's parameters.
            SetTy::Global(g, k) => {
                if !self.global_busy.insert((*g, *k)) {
                    return;
                }
                if let Some(s) = self.adds.globals.get(&(*g, *k)).cloned() {
                    let top = Env {
                        targs: &[],
                        sets: &[],
                    };
                    self.set_into(&s, &top, out);
                }
                self.global_busy.remove(&(*g, *k));
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
                Stmt::Fact(e) => Stmt::Fact(self.expr(e, env)),
                // A bound test is decided here; the branch it does not take is never linked,
                // so it may call what only the bound gives (R08.2.9).
                Stmt::If(c, a, b) => match &c.kind {
                    EK::HasBound(t, tr, args) => {
                        let v = self.has_bound(t, *tr, args, env);
                        let cond = Expr {
                            kind: EK::Bool(v),
                            ty: self.ty(&c.ty, env),
                            site: c.site,
                        };
                        match v {
                            true => Stmt::If(cond, self.block(a, env), Vec::new()),
                            false => Stmt::If(cond, Vec::new(), self.block(b, env)),
                        }
                    }
                    _ => Stmt::If(self.expr(c, env), self.block(a, env), self.block(b, env)),
                },
                Stmt::Loop(a, b, s) => Stmt::Loop(self.block(a, env), self.block(b, env), *s),
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
                Stmt::Yield(e) => Stmt::Yield(self.expr(e, env)),
                Stmt::LendOut(p) => Stmt::LendOut(p.clone()),
                Stmt::Drop(p) => Stmt::Drop(p.clone()),
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
                    body_site,
                } => {
                    let args = args.iter().map(|a| self.arg(a, env)).collect();
                    let body = self.block(body, env);
                    let target = match target {
                        Target::Value(c) => Some(Target::Value(Box::new(self.expr(c, env)))),
                        t => self.target(t, env).map(Target::Inst),
                    };
                    match target {
                        Some(target) => Stmt::Region {
                            target,
                            args,
                            var: *var,
                            lend: *lend,
                            body,
                            site: *site,
                            body_site: *body_site,
                        },
                        None => Stmt::Break,
                    }
                }
            })
            .collect()
    }
    /// The instance a call reaches. A method of a bound is resolved here, now that the type is known.
    fn target(&mut self, t: &Target<Ty>, env: &Env) -> Option<usize> {
        self.key(t, env).map(|k| self.intern(k))
    }
    fn key(&mut self, t: &Target<Ty>, env: &Env) -> Option<InstKey> {
        match t {
            Target::Fn(f) => Some(self.fnref(f, env)),
            Target::Case => Some(InstKey {
                func: FuncKey::Case,
                targs: vec![],
                sets: vec![],
            }),
            // A method of a bound: now the type is known, so the one impl is too.
            Target::Trait {
                tr,
                args,
                method,
                self_ty,
                targs,
                ret,
                ctx,
            } => {
                let mut k = self.trait_key(tr, args, method, self_ty, env)?;
                k.targs.extend(targs.iter().map(|t| self.targ(t, env)));
                self.in_copy(
                    &mut k,
                    self.d.member(&self.d.traits[tr.0].methods, method),
                    *ctx,
                );
                if let Some(g) = k.func.decl() {
                    let mut out = vec![None; self.d.f(g).n_ret_sets];
                    let c = self.ty(ret, env);
                    ret_sets_in(&self.d.f(g).ret, &c, &mut out);
                    self.with_ret_sets(
                        &mut k,
                        out.into_iter().map(Option::unwrap_or_default).collect(),
                    );
                }
                Some(k)
            }
            Target::Inst(i) => Some(self.keys[*i].clone()),
            Target::Value(_) => None,
        }
    }
    /// The impl's fn a method of a bound runs, now the type is known.
    /// Whether `t` meets trait `tr` at `args` in this instance: an impl fits it, or the type
    /// meets the trait without one (a domain, for `Domain`).
    fn has_bound(&mut self, t: &Ty, tr: TraitId, args: &[Ty], env: &Env) -> bool {
        let st = self.targ(t, env).unlent().clone();
        let want: Vec<CTy> = (self.d.traits[tr.0].inputs(args).iter())
            .map(|a| self.targ(a, env))
            .collect();
        let d = self.d;
        let by_impl = d.impls_of(tr).any(|im| {
            let mut binds = vec![None; im.generics.len()];
            fits_head(d, im, &st, &mut binds)
                && (d.traits[tr.0].inputs(&im.trait_.args).iter().zip(&want))
                    .all(|(p, w)| fit_cty(p, w, &mut binds))
                && blanket_met(d, im, &binds)
        });
        by_impl || (want.is_empty() && cty_meets(d, &st, tr))
    }
    fn trait_key(
        &mut self,
        tr: &TraitId,
        args: &[Ty],
        method: &str,
        self_ty: &Ty,
        env: &Env,
    ) -> Option<InstKey> {
        let st = self.targ(self_ty, env).unlent().clone();
        // `Self` of a trait's fn as an item nothing calls (`+` as a value): nothing runs it.
        if st == CTy::Error {
            return None;
        }
        let want: Vec<CTy> = (self.d.traits[tr.0].inputs(args).iter())
            .map(|a| self.targ(a, env))
            .collect();
        // The impl's sets are its target's: a struct's, or a fn type's own set.
        let sets = match &st {
            CTy::Struct(_, _, ss) => ss.clone(),
            CTy::Fn(_, _, s) => vec![s.clone()],
            _ => Vec::new(),
        };
        let d = self.d;
        // The trait's own fn, with `Self` and the trait's arguments for its parameters:
        // what an impl leaves to the default, and what a type meeting the trait without
        // an impl (a domain, for `Domain`) runs.
        let default = |trait_args: Vec<CTy>| {
            (d.member(&d.traits[tr.0].methods, method))
                .filter(|&tm| !matches!(d.f(tm).body, cpl_syntax::ast::FnBody::None))
                .map(|tm| InstKey {
                    func: FuncKey::Decl(tm),
                    targs: std::iter::once(st.clone()).chain(trait_args).collect(),
                    sets: Vec::new(),
                })
        };
        // The checker let exactly one impl of the most specific rank fit (R08.1.2).
        let (mut found, mut rank) = (None, None);
        for im in d.impls_of(*tr) {
            if rank.is_some_and(|r| d.impl_rank(im) > r) {
                break;
            }
            let mut binds = vec![None; im.generics.len()];
            if !fits_head(d, im, &st, &mut binds)
                || !(d.traits[tr.0].inputs(&im.trait_.args).iter().zip(&want))
                    .all(|(p, w)| fit_cty(p, w, &mut binds))
                || !blanket_met(d, im, &binds)
            {
                continue;
            }
            if rank.is_some() {
                self.errors.push(Msg::internal(format!(
                    "two impls of `{}` give `{method}` for one type",
                    d.traits[tr.0].name
                )));
                break;
            }
            rank = Some(d.impl_rank(im));
            self.bound_only(im, &mut binds);
            let binds: Vec<CTy> = binds.into_iter().map(|b| b.unwrap_or(CTy::Error)).collect();
            found = match self.d.member(&im.methods, method) {
                Some(m) => Some(InstKey {
                    func: FuncKey::Decl(m),
                    targs: binds,
                    sets: sets.clone(),
                }),
                None => {
                    let env = Env {
                        targs: &binds,
                        sets: &sets,
                    };
                    default(im.trait_.args.iter().map(|t| self.ty(t, &env)).collect())
                }
            };
        }
        if found.is_none() && cty_meets(d, &st, *tr) {
            found = default(args.iter().map(|t| self.ty(t, env)).collect());
        }
        if found.is_none() {
            self.errors.push(Msg::internal(format!(
                "no impl provides `{method}` for this type"
            )));
        }
        found
    }
    fn arg(&mut self, a: &Arg<Ty>, env: &Env) -> Arg<CTy> {
        match a {
            Arg::Value(v) => Arg::Value(self.expr(v, env)),
            Arg::View(p) => Arg::View(self.place(p)),
            Arg::Lend(p) => Arg::Lend(p.clone()),
        }
    }
    fn expr(&mut self, e: &Expr<Ty>, env: &Env) -> Expr<CTy> {
        let site = e.site;
        let ty = self.ty(&e.ty, env);
        let kind = match &e.kind {
            EK::Int(i) => EK::Int(*i),
            EK::F64(b) => EK::F64(*b),
            EK::Bool(b) => EK::Bool(*b),
            EK::Text(t) => EK::Text(t.clone()),
            EK::Unit => EK::Unit,
            EK::Copy(p) => EK::Copy(self.place(p)),
            EK::Pass(p, view, s) => EK::Pass(self.place(p), *view, *s),
            // A value in a slot that isn't lent is copied (`Link::lent`).
            EK::Lent(p) if matches!(ty, CTy::Lent(_)) => EK::Lent(self.place(p)),
            EK::Lent(p) => EK::Copy(self.place(p)),
            EK::AddrOf(p) => EK::AddrOf(p.clone()),
            EK::Tag(p) => EK::Tag(self.place(p)),
            // A range with constant ends is the domain of its integers (R04.2.18).
            EK::DomLen(t) => match self.targ(t, env) {
                CTy::Range(lo, hi) => EK::Int((hi - lo + 1).max(0)),
                CTy::RangeTo(d, lo, k) => {
                    let int = |kind| Expr {
                        kind,
                        ty: CTy::Int,
                        site: 0,
                    };
                    let of = match *d {
                        CTy::Range(a, b) => EK::Int((b - a + 1).max(0)),
                        d => EK::DomLen(d),
                    };
                    let more = int(EK::Int(k - lo + 1));
                    EK::Intr(Intr::Add, vec![Arg::Value(int(of)), Arg::Value(more)], 0)
                }
                t => EK::DomLen(t),
            },
            EK::DomLo(t) => match self.targ(t, env) {
                CTy::Range(lo, _) | CTy::RangeTo(_, lo, _) => EK::Int(lo),
                t => EK::DomLo(t),
            },
            EK::DomHeld(t) => match self.targ(t, env) {
                CTy::Range(lo, _) | CTy::RangeTo(_, lo, _) => EK::Int(lo),
                _ => EK::Int(0),
            },
            EK::DomCounted(t) => EK::Bool(match self.targ(t, env) {
                CTy::Dom(d) => self.whole.dom(DomId(d)).counted,
                _ => false,
            }),
            EK::HasBound(t, tr, args) => EK::Bool(self.has_bound(t, *tr, args, env)),
            EK::MakeStruct(xs) => EK::MakeStruct(xs.iter().map(|x| self.expr(x, env)).collect()),
            EK::MakeClosure(xs) => EK::MakeClosure(xs.iter().map(|x| self.expr(x, env)).collect()),
            EK::Inject(x) => EK::Inject(Box::new(self.expr(x, env))),
            EK::Widen(x) => EK::Widen(Box::new(self.expr(x, env))),
            EK::MakeUnion(i, x) => EK::MakeUnion(*i, Box::new(self.expr(x, env))),
            EK::Intr(i, args, at) => {
                let args: Vec<Arg<CTy>> = args.iter().map(|a| self.arg(a, env)).collect();
                match (i, &args[..]) {
                    // A type that is not a domain is its own integers: each is an ordinal of it.
                    (Intr::Below, [_, Arg::Value(n)]) if matches!(&n.kind, EK::DomLen(t) if !matches!(t, CTy::Dom(_))) => {
                        EK::Bool(true)
                    }
                    _ => EK::Intr(*i, args, *at),
                }
            }
            EK::CallValue(c, args, at) => EK::CallValue(
                Box::new(self.expr(c, env)),
                args.iter().map(|a| self.arg(a, env)).collect(),
                *at,
            ),
            EK::Call(t, args, at) => {
                let args: Vec<Arg<CTy>> = args.iter().map(|a| self.arg(a, env)).collect();
                // A method an impl gives as an intrinsic is that operation.
                let key = self.key(t, env);
                let intr = match &key {
                    Some(InstKey {
                        func: FuncKey::Decl(f),
                        ..
                    }) => self.d.f(*f).intrinsic.as_deref().and_then(Intr::named),
                    _ => None,
                };
                match (intr, key) {
                    (Some(op), _) => EK::Intr(op, args, *at),
                    (None, Some(k)) => EK::Call(Target::Inst(self.intern(k)), args, *at),
                    // A call nothing runs (a method of a type nothing fixes) still has its type.
                    (None, None) => EK::Intr(Intr::Unset, Vec::new(), *at),
                }
            }
        };
        // A read of a place that is not a lent slot copies the value (a slot not lent is
        // unlent above), and an instance returns its value unlent: each is of that type.
        let ty = match (&kind, ty) {
            (EK::Copy(p) | EK::Pass(p, ..), CTy::Lent(t)) if matches!(self.place_ty(p), Some(s) if !matches!(s, CTy::Lent(_))) => {
                *t
            }
            (EK::Call(..) | EK::CallValue(..), CTy::Lent(t)) => *t,
            (_, ty) => ty,
        };
        Expr { site, kind, ty }
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
                    | CTy::Char
                    | CTy::U64
                    | CTy::I128
                    | CTy::F64
                    | CTy::Unit
                    | CTy::Raw(_)
                    | CTy::Ref(_)
                    | CTy::Lent(_)
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
                for f in &p.parts(t) {
                    visit(p, f, stack, bad);
                }
            }
            CTy::Fn(_, _, ms) => {
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
        p.errors.push(Msg::internal("a function value captures a function value that can hold itself, so it has no fixed size; keep the steps in a list and apply them in a loop"));
    }
}

/// Contract 6, required rewrites: a call through a function value becomes a
/// branch over the closures it can be, and each call a direct call.
/// The canonical form of every instance, or what the link step left inconsistent. It also
/// decides which types share a representation (`Program::shares`).
pub fn canonicalize(p: &mut Program) -> Result<Vec<Func<CTy>>, Vec<Msg>> {
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
        .collect::<Vec<_>>();
    if bad.is_empty() {
        p.shares = share::shares(p, &funcs);
        Ok(funcs)
    } else {
        Err(bad.into_iter().map(Msg::internal).collect())
    }
}

fn read(p: Place, ty: CTy) -> Expr<CTy> {
    Expr {
        site: 0,
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
            Stmt::Loop(a, b2, s) => out.push(Stmt::Loop(
                canon_block(p, locals, bad, a),
                canon_block(p, locals, bad, b2),
                s,
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
            Stmt::Yield(e) => out.push(Stmt::Yield(canon_expr(bad, e))),
            // A walk of a generator value runs the closure it holds: the one of its set it is.
            Stmt::Region {
                target: Target::Value(c),
                args,
                var,
                lend,
                body,
                site,
                body_site,
            } => {
                let cty = c.ty.clone();
                locals.push(cty.clone());
                let tmp = locals.len() - 1;
                out.push(Stmt::Let(tmp, canon_expr(bad, *c)));
                let body = canon_block(p, locals, bad, body);
                let walk = |k: &InstKey, env: Place| {
                    let mut all: Vec<Arg<CTy>> = (p.captures(k).into_iter().enumerate())
                        .map(|(j, t)| Arg::Value(read(Place::Field(Box::new(env.clone()), j), t)))
                        .collect();
                    all.extend(args.iter().cloned());
                    Stmt::Region {
                        target: Target::Inst(p.index[k]),
                        args: all,
                        var,
                        lend,
                        body: body.clone(),
                        site,
                        body_site,
                    }
                };
                match &cty {
                    CTy::Clo(k) => out.push(walk(k, Place::Local(tmp))),
                    CTy::Fn(_, _, ms) => {
                        let arms = (ms.iter().enumerate())
                            .map(|(i, k)| {
                                vec![walk(k, Place::Variant(Box::new(Place::Local(tmp)), i))]
                            })
                            .collect();
                        out.push(Stmt::Switch(
                            Expr {
                                site: 0,
                                kind: EK::Tag(Place::Local(tmp)),
                                ty: CTy::Int,
                            },
                            arms,
                        ));
                    }
                    _ => bad.push("a generator value walked without its closure".into()),
                }
            }
            Stmt::Region {
                target,
                args,
                var,
                lend,
                body,
                site,
                body_site,
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
                    body_site,
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
            let args: Vec<Arg<CTy>> = (args.into_iter())
                .map(|a| match a {
                    Arg::Value(e) => Arg::Value(canon_expr(bad, e)),
                    a => a,
                })
                .collect();
            // Ownership reads the callee from a place no operand writes (`drops::place`).
            let EK::Copy(env) = callee.kind else {
                return bad.push("a call through a fn value not read from a place".into());
            };
            // A capture the body views is given as a view of the closure's field, as ownership
            // gives one to a viewed param; any other is the callee's copy, which it releases.
            let call = |k: &InstKey, env: Place| {
                let views = &p.insts[p.index[k]].views;
                let mut all: Vec<Arg<CTy>> = p
                    .captures(k)
                    .into_iter()
                    .enumerate()
                    .map(|(j, t)| {
                        let at = Place::Field(Box::new(env.clone()), j);
                        match views.get(j).copied().unwrap_or(false) {
                            true => Arg::View(at),
                            false => Arg::Value(read(at, t)),
                        }
                    })
                    .collect();
                all.extend(args.iter().cloned());
                Expr {
                    site: 0,
                    kind: EK::Call(Target::Inst(p.index[k]), all, at),
                    ty: e.ty.clone(),
                }
            };
            match &cty {
                CTy::Clo(k) => out.push(store(call(k, env.clone()))),
                CTy::Fn(_, _, ms) => {
                    let arms = ms
                        .iter()
                        .enumerate()
                        .map(|(i, k)| {
                            vec![Stmt::Assign(
                                dst.clone(),
                                call(k, Place::Variant(Box::new(env.clone()), i)),
                            )]
                        })
                        .collect();
                    out.push(Stmt::Switch(
                        Expr {
                            site: 0,
                            kind: EK::Tag(env.clone()),
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
            let (CTy::Fn(_, _, a), CTy::Fn(_, _, b)) = (&from, &to) else {
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
                            site: 0,
                            kind: EK::MakeUnion(j, Box::new(payload)),
                            ty: to.clone(),
                        },
                    )]
                })
                .collect();
            out.push(Stmt::Switch(
                Expr {
                    site: 0,
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
    let site = e.site;
    let ty = e.ty.clone();
    let kind = match e.kind {
        EK::Inject(x) => {
            let x = canon_expr(bad, *x);
            match (&x.ty, &ty) {
                (CTy::Clo(k), CTy::Fn(_, _, ms)) => {
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
    Expr { site, kind, ty }
}

/// Run-time values that types carry (the size of a domain) are passed here.
/// A function that needs the size of a domain it did not declare gets a hidden
/// leading parameter for it; a closure captures it when it is made. Which
/// functions need which is a whole-program question, so the link step answers it.
fn thread_values(p: &mut Program, whole: &Whole) {
    use cpl_next_ir::whole::{GlobalId, VarRef};
    let n = p.insts.len();
    // Where an instance can read a domain's value without being handed it.
    let own = |f: &Func<CTy>, v: DomValue| -> Option<Place> {
        let (var, owner) = whole.dom_var(v);
        match (var, &f.key, owner) {
            (VarRef::Global(g), FuncKey::Main | FuncKey::Case, None) => Some(Place::Global(g)),
            // A top-level domain of the file is one shared variable. One inside a block is not: a loop makes a new one.
            (VarRef::Global(g), _, None) if whole.global(GlobalId(g)).is_some_and(|x| x.top) => {
                Some(Place::Global(g))
            }
            (VarRef::Local(l), FuncKey::Decl(k), Some(o)) if *k == o => Some(Place::Local(l)),
            _ => None,
        }
    };
    fn uses(
        b: &[Stmt<CTy>],
        doms: &mut BTreeSet<DomValue>,
        calls: &mut BTreeSet<usize>,
        p: &Program,
        bad: &mut bool,
    ) {
        fn ex(
            e: &Expr<CTy>,
            doms: &mut BTreeSet<DomValue>,
            calls: &mut BTreeSet<usize>,
            p: &Program,
            bad: &mut bool,
        ) {
            match &e.kind {
                EK::DomLen(CTy::Dom(d)) => {
                    doms.insert(DomValue::Len(DomId(*d)));
                }
                EK::DomLo(CTy::Dom(d)) => {
                    doms.insert(DomValue::Lo(DomId(*d)));
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
                    for a in args {
                        if let Arg::Value(v) = a {
                            ex(v, doms, calls, p, bad);
                        }
                    }
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
                Stmt::Loop(a, b2, _) => {
                    uses(a, doms, calls, p, bad);
                    uses(b2, doms, calls, p, bad);
                }
                Stmt::Switch(c, arms) => {
                    ex(c, doms, calls, p, bad);
                    arms.iter().for_each(|a| uses(a, doms, calls, p, bad));
                }
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
    let mut bad = None;
    for i in 0..n {
        let mut b = false;
        uses(&p.insts[i].body, &mut direct[i], &mut calls[i], p, &mut b);
        bad = bad.or(b.then_some(i));
    }
    if let Some(i) = bad {
        let func = p.insts[i].shown.clone();
        p.errors
            .push(Unbuilt::DomainTypeArg { func, domain: None }.into());
        return;
    }
    // What each instance must be handed: what it or anything it calls uses, less what it can read itself.
    let mut needs: Vec<BTreeSet<DomValue>> = vec![BTreeSet::new(); n];
    loop {
        let mut changed = false;
        for i in 0..n {
            let mut want: BTreeSet<DomValue> = direct[i].clone();
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
        p.errors.push(Msg::internal(
            "a domain's size is needed where the domain is not in scope",
        ));
        return;
    }
    // Hidden parameters come first, and count as captures: a closure holds them.
    let mut hidden: Vec<Vec<(DomValue, usize)>> = Vec::new();
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
        hidden: &'a [Vec<(DomValue, usize)>],
        index: &'a HashMap<InstKey, usize>,
        me: usize,
        own: BTreeMap<DomValue, Option<Place>>,
        /// Set when a size is asked for and neither handed in nor declared here.
        no_source: std::cell::Cell<bool>,
    }
    impl Rw<'_> {
        fn size(&self, d: DomValue) -> Expr<CTy> {
            let place = match self.hidden[self.me].iter().find(|(x, _)| *x == d) {
                Some((_, l)) => Some(Place::Local(*l)),
                None => self.own.get(&d).cloned().flatten(),
            };
            let kind = place.map(EK::Copy).unwrap_or_else(|| {
                self.no_source.set(true);
                EK::Int(0)
            });
            Expr {
                site: 0,
                kind,
                ty: CTy::Int,
            }
        }
        fn handed(&self, j: usize) -> Vec<Expr<CTy>> {
            self.hidden[j].iter().map(|(d, _)| self.size(*d)).collect()
        }
        fn ex(&self, e: &mut Expr<CTy>) {
            let ty = e.ty.clone();
            match &mut e.kind {
                EK::DomLen(CTy::Dom(d)) => *e = self.size(DomValue::Len(DomId(*d))),
                EK::DomLo(CTy::Dom(d)) => *e = self.size(DomValue::Lo(DomId(*d))),
                // A type that is not a domain is its own integers.
                EK::DomLo(_) => e.kind = EK::Int(0),
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
                    args.iter_mut().for_each(|a| {
                        if let Arg::Value(v) = a {
                            self.ex(v)
                        }
                    });
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
                    Stmt::Loop(a, b2, _) => {
                        self.block(a);
                        self.block(b2);
                    }
                    Stmt::Switch(c, arms) => {
                        self.ex(c);
                        arms.iter_mut().for_each(|a| self.block(a));
                    }
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
        let own_places = (whole.dom_values())
            .map(|v| (v, own(&p.insts[i], v)))
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
            p.errors.push(Msg::internal(
                "a domain's size is used in a fn that neither declares it nor is handed it",
            ));
        }
    }
}

/// Contract 8: event points, placed on instance IR after the link step, only when asked for.
/// They are not in generic IR and are not uses for ownership. Reported here: entering and
/// leaving a fn of the user's file, and each write to one of its top-level bindings.
pub fn add_events(p: &mut Program, whole: &Whole) {
    fn shown(t: &CTy) -> bool {
        matches!(t, CTy::Int | CTy::Bool | CTy::Char | CTy::U64 | CTy::Dom(_))
    }
    fn block(b: Vec<Stmt<CTy>>, exit: Option<&str>, whole: &Whole) -> Vec<Stmt<CTy>> {
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
                    let global = whole.global(cpl_next_ir::whole::GlobalId(g));
                    let (top, name, ty) = (
                        global.is_some_and(|x| x.top),
                        global.map(|x| x.name.clone()).unwrap_or_default(),
                        e.ty.clone(),
                    );
                    out.push(Stmt::Assign(Place::Global(g), e));
                    if top && shown(&ty) && !name.contains('$') {
                        out.push(Stmt::Event(
                            format!("set {name}"),
                            vec![Expr {
                                site: 0,
                                kind: EK::Copy(Place::Global(g)),
                                ty,
                            }],
                        ));
                    }
                }
                Stmt::If(c, a, b2) => {
                    out.push(Stmt::If(c, block(a, exit, whole), block(b2, exit, whole)))
                }
                Stmt::Loop(a, b2, s) => {
                    out.push(Stmt::Loop(block(a, exit, whole), block(b2, exit, whole), s))
                }
                Stmt::Switch(c, arms) => out.push(Stmt::Switch(
                    c,
                    arms.into_iter().map(|a| block(a, exit, whole)).collect(),
                )),
                Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body,
                    site,
                    body_site,
                } => out.push(Stmt::Region {
                    target,
                    args,
                    var,
                    lend,
                    body: block(body, exit, whole),
                    site,
                    body_site,
                }),
                other => out.push(other),
            }
        }
        out
    }
    for f in &mut p.insts {
        let user_fn = match &f.key {
            FuncKey::Decl(id) | FuncKey::Ctx(id, _) => {
                whole.program_fns.contains(id) && f.region.is_none()
            }
            _ => false,
        };
        let body = std::mem::take(&mut f.body);
        let name = f.name.clone();
        let mut body = block(body, if user_fn { Some(&name) } else { None }, whole);
        if user_fn {
            let args = f.params[f.n_captures..]
                .iter()
                .filter(|(l, _)| shown(&f.locals[*l]))
                .map(|(l, _)| Expr {
                    site: 0,
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

/// Whether a type an impl is written for, in the impl's type parameters, is `t`: binds them.
impl Link<'_> {
    /// Fixes each parameter of `im` only a bound names (`K` of `impl<S: Indexed<K, T>, K, T>`) to
    /// what the impl meeting that bound gives it, as the checker does (R08.1.2).
    fn bound_only(&mut self, im: &cpl_next_decls::decls::ImplDecl, binds: &mut [Option<CTy>]) {
        let d = self.d;
        loop {
            let unbound = binds.iter().filter(|b| b.is_none()).count();
            if unbound == 0 {
                return;
            }
            for (i, r) in &im.bounds {
                let Some(bt) = binds[*i].clone() else {
                    continue;
                };
                // The one impl of the most specific rank that fits fixes them: coherence and
                // the checker leave no second (R08.1.2), so one is a checker bug.
                let (mut one, mut rank) = (None, None);
                for m in d.impls_of(r.tr) {
                    if rank.is_some_and(|k| d.impl_rank(m) > k) {
                        break;
                    }
                    let mut b = vec![None; m.generics.len()];
                    if !fit_cty(&m.for_, &bt, &mut b) || !blanket_met(d, m, &b) {
                        continue;
                    }
                    if rank.is_some() {
                        self.errors.push(Msg::internal(format!(
                            "two impls of `{}` fit one bound at one rank",
                            d.traits[r.tr.0].name
                        )));
                        one = None;
                        break;
                    }
                    rank = Some(d.impl_rank(m));
                    self.bound_only(m, &mut b);
                    let b: Vec<CTy> = b.into_iter().map(|x| x.unwrap_or(CTy::Error)).collect();
                    let env = Env {
                        targs: &b,
                        sets: &[],
                    };
                    one = Some(
                        m.trait_
                            .args
                            .iter()
                            .map(|a| self.ty(a, &env))
                            .collect::<Vec<CTy>>(),
                    );
                }
                for (p, a) in r.args.iter().zip(one.iter().flatten()) {
                    fit_cty(p, a, binds);
                }
            }
            if binds.iter().filter(|b| b.is_none()).count() == unbound {
                return;
            }
        }
    }
}

/// Whether a blanket impl's bounds hold of the types it binds, as the checker asked
/// (`Decls::fits_impl`); an impl for a written type is taken as found.
fn blanket_met(d: &Decls, im: &cpl_next_decls::decls::ImplDecl, binds: &[Option<CTy>]) -> bool {
    !im.blanket()
        || (im.bounds.iter()).all(|(i, r)| binds[*i].as_ref().is_none_or(|t| cty_meets(d, t, r.tr)))
}
fn cty_meets(d: &Decls, t: &CTy, tr: TraitId) -> bool {
    if d.core_domain_trait(tr) && matches!(t, CTy::Dom(_)) {
        return true;
    }
    d.impls_of(tr).any(|im| {
        let mut b = vec![None; im.generics.len()];
        fits_head(d, im, t, &mut b)
            && (im.bounds.iter()).all(|(i, r)| b[*i].as_ref().is_none_or(|a| cty_meets(d, a, r.tr)))
    })
}
/// Whether an impl is for `t`, as `Decls::head_fits` says of a range type and `Int`.
fn fits_head(
    d: &Decls,
    im: &cpl_next_decls::decls::ImplDecl,
    t: &CTy,
    binds: &mut [Option<CTy>],
) -> bool {
    let head = match t {
        CTy::Int => Ty::Int,
        CTy::Range(lo, hi) => Ty::Range(*lo, *hi),
        // Only its kind matters to `head_fits`: a range's.
        CTy::RangeTo(_, lo, _) => Ty::Range(*lo, *lo),
        _ => Ty::Unit,
    };
    d.head_fits(im, &head) && fit_cty(&im.for_, t, binds)
}
fn fit_cty(p: &Ty, t: &CTy, binds: &mut [Option<CTy>]) -> bool {
    match (p, t) {
        (Ty::Param(i), _) => match &binds[*i] {
            // A range's values are `Int`s: `List<0..=9>`'s `T` meets `ListLiteral<Int>`.
            Some(b) => {
                b == t
                    || [b, t]
                        .iter()
                        .all(|x| matches!(x, CTy::Int | CTy::Range(..) | CTy::RangeTo(..)))
            }
            None => {
                binds[*i] = Some(t.clone());
                true
            }
        },
        (
            Ty::Int | Ty::Range(..) | Ty::RangeOf(_) | Ty::RangeTo(..),
            CTy::Int | CTy::Range(..) | CTy::RangeTo(..),
        ) => true,
        (Ty::Bool, CTy::Bool)
        | (Ty::Char, CTy::Char)
        | (Ty::U64, CTy::U64)
        | (Ty::I128, CTy::I128)
        | (Ty::F64, CTy::F64)
        | (Ty::Unit, CTy::Unit) => true,
        (Ty::Raw(a), CTy::Raw(b)) | (Ty::Ref(a), CTy::Ref(b)) => fit_cty(a, b, binds),
        (Ty::Struct(d1, a, _), CTy::Struct(d2, b, _)) => {
            d1 == d2 && a.len() == b.len() && a.iter().zip(b).all(|(x, y)| fit_cty(x, y, binds))
        }
        (Ty::Clo(c), CTy::Clo(k)) => {
            matches!(&**c, CloRef::Item(f) if k.func.decl() == Some(f.decl))
        }
        // A fn value's own types, which the checker gave every closure in its set.
        (Ty::Fn(ps, r, _), CTy::Fn(cps, cr, _)) => {
            ps.len() == cps.len()
                && ps.iter().zip(cps).all(|(p, c)| fit_cty(p, c, binds))
                && fit_cty(r, cr, binds)
        }
        (Ty::AnyRange(i), CTy::Range(..) | CTy::RangeTo(..)) => fit_cty(&Ty::Param(*i), t, binds),
        // A lent slot reads as the value it points at.
        (_, CTy::Lent(t)) => fit_cty(p, t, binds),
        _ => false,
    }
}

/// The sets a value of type `c` holds where `t`, a fn's declared result, has its result slots.
fn ret_sets_in(t: &Ty, c: &CTy, out: &mut [Option<CSet>]) {
    let mut put = |s: &SetTy, cs: &CSet| {
        if let SetTy::RetSlot(k) = s {
            if let Some(o) = out.get_mut(*k) {
                *o = Some(cs.clone());
            }
        }
    };
    match (t, c) {
        (Ty::Fn(_, _, s), CTy::Fn(_, _, cs)) => put(s, cs),
        (Ty::Struct(_, ts, ss), CTy::Struct(_, cs, css)) => {
            ss.iter().zip(css).for_each(|(s, cs)| put(s, cs));
            for (t, c) in ts.iter().zip(cs) {
                ret_sets_in(t, c, out);
            }
        }
        (Ty::Raw(t), CTy::Raw(c)) | (Ty::Ref(t), CTy::Ref(c)) => ret_sets_in(t, c, out),
        (t, CTy::Lent(c)) => ret_sets_in(t, c, out),
        _ => {}
    }
}

/// The index of the bracket closing the one at `open` in a value's debug text.
fn closing(s: &str, open: usize) -> Option<usize> {
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    for (i, b) in s.bytes().enumerate().skip(open) {
        match (quoted, b) {
            (true, _) if escaped => escaped = false,
            (true, b'\\') => escaped = true,
            (_, b'"') => quoted = !quoted,
            (true, _) => {}
            (false, b'[' | b'(' | b'{') => depth += 1,
            (false, b']' | b')' | b'}') => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// The elements of a debug-printed list's inside, split at its top-level commas.
fn top_level(s: &str) -> Vec<&str> {
    let (mut out, mut from) = (Vec::new(), 0);
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    for (i, b) in s.bytes().enumerate() {
        match (quoted, b) {
            (true, _) if escaped => escaped = false,
            (true, b'\\') => escaped = true,
            (_, b'"') => quoted = !quoted,
            (true, _) => {}
            (false, b'[' | b'(' | b'{') => depth += 1,
            (false, b']' | b')' | b'}') => depth = depth.saturating_sub(1),
            (false, b',') if depth == 0 => {
                out.push(&s[from..i]);
                from = i + 1;
            }
            _ => {}
        }
    }
    if !s.trim().is_empty() {
        out.push(&s[from..]);
    }
    out
}
