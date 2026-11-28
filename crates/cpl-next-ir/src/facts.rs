//! Value facts and the checks they prove (contracts 04 and 06): the one module that computes
//! them. A fact is `0 <= L` for a linear form `L` over Int locals and globals, learned from
//! what the program already decided: an assignment of a linear value, a check or branch
//! condition that held, a `Fact` lowering states (a `for` variable is at or after its start),
//! and the join of an `if`'s arms. A place written loses every fact naming it; a loop or
//! region drops, on entry, the facts on what it writes. `prove` is the step of optimized IR
//! (`opt`) that reads them: it drops a domain or range check only when the facts prove its condition,
//! records each drop, and drops the `Fact`s, which no backend runs.
use crate::ir::*;
use crate::prog::CTy;
use std::collections::{BTreeMap, HashSet};

/// A check optimized IR leaves out, and where it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proved {
    pub func: usize,
    pub kind: CheckKind,
    pub site: Site,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum A {
    L(usize),
    G(usize),
}

/// `c + sum(k * atom)`.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct Lin {
    c: i128,
    t: BTreeMap<A, i128>,
}
impl Lin {
    fn konst(c: i128) -> Lin {
        Lin {
            c,
            t: BTreeMap::new(),
        }
    }
    fn plus(&self, o: &Lin, k: i128) -> Lin {
        let mut r = self.clone();
        r.c += k * o.c;
        for (a, v) in &o.t {
            let e = r.t.entry(*a).or_insert(0);
            *e += k * v;
            if *e == 0 {
                r.t.remove(a);
            }
        }
        r
    }
    fn names(&self, a: A) -> bool {
        self.t.contains_key(&a)
    }
}

/// How far a proof chains facts, each step cancelling one atom, and how many steps it tries.
/// Running out only leaves a check in: it never changes whether a program is accepted or
/// what it does, only that the check runs, so the bounds are the prover's own, not a rule.
const DEPTH: usize = 6;
const STEPS: usize = 4096;

#[derive(Clone, Default)]
struct Known {
    facts: Vec<Lin>,
    /// A place's value as a form over places not in this map.
    eq: BTreeMap<A, Lin>,
}
impl Known {
    fn kill(&mut self, a: A) {
        self.eq.remove(&a);
        self.eq.retain(|_, l| !l.names(a));
        self.facts.retain(|l| !l.names(a));
    }
    fn expand(&self, l: &Lin) -> Lin {
        let mut r = Lin::konst(l.c);
        for (a, k) in &l.t {
            match self.eq.get(a) {
                Some(e) => r = r.plus(e, *k),
                None => r = r.plus(&atom(*a), *k),
            }
        }
        r
    }
    fn add(&mut self, l: Lin) {
        let l = self.expand(&l);
        if !l.t.is_empty() && !self.facts.contains(&l) {
            self.facts.push(l);
        }
    }
    fn set(&mut self, a: A, v: Option<Lin>) {
        let v = v.map(|v| self.expand(&v));
        self.kill(a);
        if let Some(v) = v.filter(|v| !v.names(a)) {
            self.eq.insert(a, v);
        }
    }
    fn proves(&self, l: &Lin) -> bool {
        self.chain(&self.expand(l), DEPTH, &mut { STEPS })
    }
    fn chain(&self, l: &Lin, depth: usize, steps: &mut usize) -> bool {
        let Some((&a, &k)) = l.t.iter().next() else {
            return l.c >= 0;
        };
        depth > 0
            && self.facts.iter().any(|f| match f.t.get(&a) {
                Some(&m) if m.signum() == k.signum() && k % m == 0 && *steps > 0 => {
                    *steps -= 1;
                    self.chain(&l.plus(f, -(k / m)), depth - 1, steps)
                }
                _ => false,
            })
    }
}

fn atom(a: A) -> Lin {
    let mut l = Lin::konst(0);
    l.t.insert(a, 1);
    l
}

struct Pass<'a> {
    func: usize,
    /// Places some `AddrOf` reaches: a write through it names no place, so they are no atoms.
    escaped: &'a HashSet<(Option<usize>, usize)>,
    /// Globals a function other than the program's entry writes: what a call may change.
    foreign: &'a HashSet<usize>,
    proved: Vec<Proved>,
}

fn root(p: &Place) -> Option<&Place> {
    match p {
        Place::Local(_) | Place::Global(_) => Some(p),
        Place::Field(b, _) | Place::Variant(b, _) | Place::Slot(b, _) => root(b),
        Place::Deref(_) | Place::Through(_) => None,
    }
}

/// What a block may write: its places, and whether it calls (a callee writes any global).
#[derive(Default)]
struct Writes {
    places: Vec<Place>,
    calls: bool,
}
impl Writes {
    fn expr(&mut self, e: &Expr<CTy>) {
        match &e.kind {
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| self.expr(x)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => self.expr(x),
            EK::Intr(_, args, _) => self.args(args),
            EK::Call(_, args, _) => {
                self.calls = true;
                self.args(args);
            }
            EK::CallValue(f, xs, _) => {
                self.calls = true;
                self.expr(f);
                self.args(xs);
            }
            _ => {}
        }
    }
    fn args(&mut self, args: &[Arg<CTy>]) {
        for a in args {
            match a {
                Arg::Value(v) => self.expr(v),
                Arg::Lend(p) => self.places.extend(root(p).cloned()),
                Arg::View(_) => {}
            }
        }
    }
    fn block(&mut self, b: &[Stmt<CTy>]) {
        for s in b {
            match s {
                Stmt::Let(l, e) => {
                    self.places.push(Place::Local(*l));
                    self.expr(e);
                }
                Stmt::Assign(p, e) => {
                    self.places.extend(root(p).cloned());
                    self.expr(e);
                }
                Stmt::Expr(e) | Stmt::Yield(e) | Stmt::Fact(e) => {
                    self.calls |= matches!(s, Stmt::Yield(_));
                    self.expr(e);
                }
                Stmt::Return(e) => e.iter().for_each(|e| self.expr(e)),
                Stmt::Check { cond, .. } => self.expr(cond),
                Stmt::If(c, a, b) => {
                    self.expr(c);
                    self.block(a);
                    self.block(b);
                }
                Stmt::Loop(a, b, _) => {
                    self.block(a);
                    self.block(b);
                }
                Stmt::Switch(c, arms) => {
                    self.expr(c);
                    arms.iter().for_each(|a| self.block(a));
                }
                Stmt::Region {
                    args, var, body, ..
                } => {
                    self.calls = true;
                    self.args(args);
                    self.places.push(Place::Local(*var));
                    self.block(body);
                }
                Stmt::LendOut(p) => {
                    self.calls = true;
                    self.places.extend(root(p).cloned());
                }
                Stmt::Drop(p) => self.places.extend(root(p).cloned()),
                Stmt::Event(crate::ir::EventPoint { vals: xs, .. }) => {
                    xs.iter().for_each(|x| self.expr(x))
                }
                Stmt::Break | Stmt::Continue => {}
            }
        }
    }
}

fn ends(b: &[Stmt<CTy>]) -> bool {
    matches!(
        b.last(),
        Some(Stmt::Break | Stmt::Continue | Stmt::Return(_))
    )
}

impl Pass<'_> {
    fn atom(&self, p: &Place) -> Option<A> {
        let a = match p {
            Place::Local(l) => A::L(*l),
            Place::Global(g) => A::G(*g),
            _ => return None,
        };
        let key = match a {
            A::L(l) => (Some(self.func), l),
            A::G(g) => (None, g),
        };
        (!self.escaped.contains(&key)).then_some(a)
    }
    fn lin(&self, e: &Expr<CTy>) -> Option<Lin> {
        if !matches!(e.ty, CTy::Int) {
            return None;
        }
        match &e.kind {
            EK::Int(i) => Some(Lin::konst(*i as i128)),
            EK::Copy(p) => self.atom(p).map(atom),
            EK::Intr(op @ (Intr::Add | Intr::Sub), args, _) => match &args[..] {
                [Arg::Value(a), Arg::Value(b)] => {
                    let k = if *op == Intr::Add { 1 } else { -1 };
                    Some(self.lin(a)?.plus(&self.lin(b)?, k))
                }
                _ => None,
            },
            EK::Intr(Intr::Neg, args, _) => match &args[..] {
                [Arg::Value(a)] => Some(Lin::konst(0).plus(&self.lin(a)?, -1)),
                _ => None,
            },
            _ => None,
        }
    }
    /// The facts `c` being `truth` gives: each `0 <= L`.
    fn holds(&self, c: &Expr<CTy>, truth: bool) -> Vec<Lin> {
        let EK::Intr(op, args, _) = &c.kind else {
            return Vec::new();
        };
        if let (Intr::Not, [Arg::Value(x)]) = (op, &args[..]) {
            return self.holds(x, !truth);
        }
        let (Some(a), Some(b)) = (match &args[..] {
            [Arg::Value(a), Arg::Value(b)] => (self.lin(a), self.lin(b)),
            _ => (None, None),
        }) else {
            return Vec::new();
        };
        let ge = |x: &Lin, y: &Lin, k: i128| vec![x.plus(y, -1).plus(&Lin::konst(k), 1)];
        match (op, truth) {
            (Intr::Le, true) | (Intr::Gt, false) => ge(&b, &a, 0),
            (Intr::Lt, true) | (Intr::Ge, false) => ge(&b, &a, -1),
            (Intr::Ge, true) | (Intr::Lt, false) => ge(&a, &b, 0),
            (Intr::Gt, true) | (Intr::Le, false) => ge(&a, &b, -1),
            (Intr::Eq, true) | (Intr::Ne, false) => [ge(&a, &b, 0), ge(&b, &a, 0)].concat(),
            (Intr::Below, true) => [ge(&a, &Lin::konst(0), 0), ge(&b, &a, -1)].concat(),
            _ => Vec::new(),
        }
    }
    fn drop_writes(&self, k: &mut Known, w: &Writes) {
        for p in &w.places {
            if let Some(a) = self.atom(p) {
                k.kill(a);
            }
        }
        if w.calls {
            let gs: Vec<A> = (k.facts.iter().flat_map(|f| f.t.keys()))
                .chain(k.eq.keys())
                .filter(|a| matches!(a, A::G(g) if self.foreign.contains(g)))
                .copied()
                .collect();
            gs.into_iter().for_each(|g| k.kill(g));
        }
    }
    fn writes_of(&self, e: &Expr<CTy>, k: &mut Known) {
        let mut w = Writes::default();
        w.expr(e);
        self.drop_writes(k, &w);
    }
    /// Optimizes `b` in place from what is known on entry; returns what is known after it.
    fn block(&mut self, b: &mut Vec<Stmt<CTy>>, mut k: Known) -> Known {
        let mut out = Vec::with_capacity(b.len());
        for mut s in std::mem::take(b) {
            match &mut s {
                Stmt::Fact(c) => {
                    for f in self.holds(c, true) {
                        k.add(f);
                    }
                    continue;
                }
                Stmt::Check {
                    kind, cond, site, ..
                } => {
                    let goal = self.holds(cond, true);
                    let removable = matches!(
                        kind,
                        CheckKind::Domain | CheckKind::RangeStore | CheckKind::Key
                    );
                    if removable && !goal.is_empty() && goal.iter().all(|g| k.proves(g)) {
                        let (kind, site) = (*kind, *site);
                        self.proved.push(Proved {
                            func: self.func,
                            kind,
                            site,
                        });
                        continue;
                    }
                    goal.into_iter().for_each(|g| k.add(g));
                }
                Stmt::Let(l, e) => {
                    self.writes_of(e, &mut k);
                    let v = self.lin(e);
                    if let Some(a) = self.atom(&Place::Local(*l)) {
                        k.set(a, v);
                    }
                }
                Stmt::Assign(p, e) => {
                    self.writes_of(e, &mut k);
                    let v = self.lin(e);
                    match root(p).and_then(|r| self.atom(r)) {
                        Some(a) if matches!(p, Place::Local(_) | Place::Global(_)) => k.set(a, v),
                        Some(a) => k.kill(a),
                        None => {}
                    }
                }
                Stmt::If(c, then, other) => {
                    self.writes_of(c, &mut k);
                    let mut kt = k.clone();
                    self.holds(c, true).into_iter().for_each(|f| kt.add(f));
                    let mut ke = k.clone();
                    self.holds(c, false).into_iter().for_each(|f| ke.add(f));
                    let kt = self.block(then, kt);
                    let ke = self.block(other, ke);
                    k = match (ends(then), ends(other)) {
                        (true, false) => ke,
                        (false, true) => kt,
                        _ => self.join(&k, then, other, &kt, &ke),
                    };
                }
                Stmt::Loop(body, step, _) => {
                    let mut w = Writes::default();
                    w.block(body);
                    w.block(step);
                    self.drop_writes(&mut k, &w);
                    self.block(body, k.clone());
                    self.block(step, k.clone());
                }
                Stmt::Switch(c, arms) => {
                    self.writes_of(c, &mut k);
                    let mut w = Writes::default();
                    arms.iter().for_each(|a| w.block(a));
                    for a in arms.iter_mut() {
                        self.block(a, k.clone());
                    }
                    self.drop_writes(&mut k, &w);
                }
                Stmt::Region { .. } => {
                    let mut w = Writes::default();
                    w.block(std::slice::from_ref(&s));
                    self.drop_writes(&mut k, &w);
                    if let Stmt::Region { body, .. } = &mut s {
                        self.block(body, k.clone());
                    }
                }
                Stmt::Break | Stmt::Continue | Stmt::Return(_) => {
                    out.push(s);
                    break;
                }
                _ => {
                    let mut w = Writes::default();
                    w.block(std::slice::from_ref(&s));
                    self.drop_writes(&mut k, &w);
                }
            }
            out.push(s);
        }
        *b = out;
        k
    }
    /// What holds after an `if` whose arms both run on: what held before less what an arm
    /// writes, and each arm's fact on a place it writes that the other arm proves too.
    fn join(
        &self,
        before: &Known,
        then: &[Stmt<CTy>],
        other: &[Stmt<CTy>],
        kt: &Known,
        ke: &Known,
    ) -> Known {
        let mut w = Writes::default();
        w.block(then);
        w.block(other);
        let mut k = before.clone();
        self.drop_writes(&mut k, &w);
        if w.calls {
            return k;
        }
        let written: HashSet<A> = w.places.iter().filter_map(|p| self.atom(p)).collect();
        let mut wanted = Vec::new();
        for arm in [kt, ke] {
            for (&a, v) in &arm.eq {
                if written.contains(&a) {
                    wanted.push(atom(a).plus(v, -1));
                    wanted.push(v.plus(&atom(a), -1));
                }
            }
            wanted.extend(arm.facts.iter().cloned());
        }
        for f in wanted {
            let mut names = f.t.keys().filter(|a| written.contains(a));
            let one = names.next().is_some() && names.next().is_none();
            if one && kt.proves(&f) && ke.proves(&f) && !k.facts.contains(&f) {
                k.facts.push(f);
            }
        }
        k
    }
}

/// Places whose address is taken or that are lent as a value: `(Some(fn), local)` or `(None, global)`.
/// The places an `AddrOf` or lend in instance `f` reaches: its locals, and globals as `None`.
fn escaped(f: usize, func: &Func<CTy>) -> HashSet<(Option<usize>, usize)> {
    fn ex(e: &Expr<CTy>, f: usize, out: &mut HashSet<(Option<usize>, usize)>) {
        match &e.kind {
            EK::AddrOf(p) | EK::Lent(p) => match root(p) {
                Some(Place::Local(l)) => {
                    out.insert((Some(f), *l));
                }
                Some(Place::Global(g)) => {
                    out.insert((None, *g));
                }
                _ => {}
            },
            EK::MakeStruct(xs) | EK::MakeClosure(xs) => xs.iter().for_each(|x| ex(x, f, out)),
            EK::Inject(x) | EK::Widen(x) | EK::MakeUnion(_, x) => ex(x, f, out),
            EK::Intr(_, args, _) | EK::Call(_, args, _) => args.iter().for_each(|a| {
                if let Arg::Value(v) = a {
                    ex(v, f, out)
                }
            }),
            EK::CallValue(g, xs, _) => {
                ex(g, f, out);
                xs.iter().for_each(|a| {
                    if let Arg::Value(v) = a {
                        ex(v, f, out)
                    }
                });
            }
            _ => {}
        }
    }
    fn st(b: &[Stmt<CTy>], f: usize, out: &mut HashSet<(Option<usize>, usize)>) {
        for s in b {
            match s {
                Stmt::Let(_, e)
                | Stmt::Assign(_, e)
                | Stmt::Expr(e)
                | Stmt::Yield(e)
                | Stmt::Fact(e)
                | Stmt::Check { cond: e, .. } => ex(e, f, out),
                Stmt::Return(e) => e.iter().for_each(|e| ex(e, f, out)),
                Stmt::If(c, a, b) => {
                    ex(c, f, out);
                    st(a, f, out);
                    st(b, f, out);
                }
                Stmt::Loop(a, b, _) => {
                    st(a, f, out);
                    st(b, f, out);
                }
                Stmt::Switch(c, arms) => {
                    ex(c, f, out);
                    arms.iter().for_each(|a| st(a, f, out));
                }
                Stmt::Region { args, body, .. } => {
                    args.iter().for_each(|a| {
                        if let Arg::Value(v) = a {
                            ex(v, f, out)
                        }
                    });
                    st(body, f, out);
                }
                Stmt::Event(crate::ir::EventPoint { vals: xs, .. }) => {
                    xs.iter().for_each(|x| ex(x, f, out))
                }
                Stmt::LendOut(_) | Stmt::Drop(_) | Stmt::Break | Stmt::Continue => {}
            }
        }
    }
    let mut out = HashSet::new();
    st(&func.body, f, &mut out);
    out
}

/// What every instance's proofs read of the whole program: the globals some `AddrOf` reaches
/// (a write through one names no place, so they are no atoms) and the globals a fn other than
/// the entry writes (what a call may change). Each instance has its part; the program's is
/// their join.
#[derive(Default, Clone, PartialEq, Debug)]
pub struct Globals {
    pub escaped: HashSet<usize>,
    pub foreign: HashSet<usize>,
}
impl Globals {
    /// Instance `i`'s part, `main` the entry.
    pub fn of(i: usize, f: &Func<CTy>, main: usize) -> Globals {
        let escaped = (escaped(i, f).into_iter())
            .filter_map(|(f, g)| f.is_none().then_some(g))
            .collect();
        let mut w = Writes::default();
        if i != main {
            w.block(&f.body);
        }
        let foreign = (w.places.iter())
            .filter_map(|p| match p {
                Place::Global(g) => Some(*g),
                _ => None,
            })
            .collect();
        Globals { escaped, foreign }
    }
    pub fn add(&mut self, o: &Globals) {
        self.escaped.extend(&o.escaped);
        self.foreign.extend(&o.foreign);
    }
}

/// Instance `i`'s canonical IR less the checks the facts prove, and those checks, from its own
/// body and the program's `g`.
pub fn prove(i: usize, f: &Func<CTy>, g: &Globals) -> (Func<CTy>, Vec<Proved>) {
    let mut escaped = escaped(i, f);
    escaped.extend(g.escaped.iter().map(|&x| (None, x)));
    let mut out = f.clone();
    let mut pass = Pass {
        func: i,
        escaped: &escaped,
        foreign: &g.foreign,
        proved: Vec::new(),
    };
    pass.block(&mut out.body, Known::default());
    (out, pass.proved)
}
