//! The search: case splits over `or`, bounds on linear forms, and the
//! Omega test at every node.
//!
//! A node of the search holds what is known so far as an interval for
//! each linear form that occurs (`i`, `j - i`, `r*W + c`): the cheap,
//! octagon-like layer that settles most disjuncts without any solving.
//! Whatever it leaves open is split on, and each node is checked for an
//! integer solution. Products of variables are treated as opaque values
//! with the bounds that follow from multiplying known constraints.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::core::{Formula, Lazy};
use crate::interval::Interval;
use crate::omega::{self, Constraint, Lin};
use crate::poly::{Mono, Poly, Var};

/// A candidate assignment; variables it leaves out are zero.
pub(crate) type Model = BTreeMap<Var, i128>;

pub(crate) enum Outcome {
    /// No assignment satisfies the constraints.
    Unsat,
    /// An assignment exists, and the caller confirmed it.
    Sat,
    /// Neither: the reason, for the report.
    Unknown(&'static str),
}

/// How much work one query may take.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Nodes of the case-split search.
    pub nodes: u64,
    /// Steps of the integer solver, over the whole query.
    pub steps: u64,
    /// Inputs to try when the solver cannot decide.
    pub samples: u32,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            nodes: 20_000,
            steps: 2_000_000,
            samples: 400,
        }
    }
}

/// `p` as `g * form + c` with `form` primitive and its first coefficient positive.
fn normalise(p: &Poly) -> Option<(Poly, i128, i128)> {
    let c = p.constant_term();
    let rest = p.add_const(-c)?;
    let mut g: i128 = 0;
    for (_, k) in rest.terms() {
        let (mut a, mut b) = (g, k.checked_abs()?);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        g = a;
    }
    let (_, first) = rest.terms().next()?;
    if first < 0 {
        g = -g;
    }
    let form = Poly::from_terms(rest.terms().map(|(m, k)| (m.clone(), k / g)))?;
    Some((form, g, c))
}

/// What an atom says about its linear form: the interval it must lie in.
/// `None` when the atom is constant or out of range.
fn bound(atom: &Formula) -> Option<(Poly, Interval)> {
    match atom {
        Formula::Ge(p) => {
            let (form, g, c) = normalise(p)?;
            // g * form + c >= 0.
            let interval = if g > 0 {
                Interval {
                    lo: Some(-c.div_euclid(g)),
                    hi: None,
                }
            } else {
                Interval {
                    lo: None,
                    hi: Some(c.div_euclid(-g)),
                }
            };
            Some((form, interval))
        }
        Formula::Eq(p) => {
            let (form, g, c) = normalise(p)?;
            let interval = if c % g == 0 {
                Interval::point(-c / g)
            } else {
                Interval::new(1, 0)
            };
            Some((form, interval))
        }
        _ => None,
    }
}

/// A disjunction still open: one of the arms holds.
#[derive(Clone)]
struct Choice {
    arms: Rc<Vec<Formula>>,
    /// Split on only when nothing else is left: the negated goal. By
    /// then the definitions it depends on are settled, and most of its
    /// arms are decided without a split.
    last: bool,
}

#[derive(Clone, Default)]
struct Node {
    forms: BTreeMap<Poly, Interval>,
    ors: Vec<Choice>,
    /// Lazy constraints still asleep, by index.
    asleep: Vec<usize>,
    in_use: BTreeSet<Var>,
    /// Equalities solved for a variable, oldest first. The variable is
    /// replaced everywhere else, so that `v == a` turns a later `v > c`
    /// into a statement about `a` that the bounds can decide.
    solved: Vec<(Var, Poly)>,
}

/// A variable an equality `p == 0` can be solved for, and its value: one
/// with coefficient 1 or -1 that occurs in no product. The newest
/// variable wins, since derived variables are made after their inputs.
fn solve_for(p: &Poly) -> Option<(Var, Poly)> {
    let (x, c) = p
        .terms()
        .filter_map(|(m, c)| match m.0.as_slice() {
            [x] if c.abs() == 1 && p.terms().filter(|(m, _)| m.0.contains(x)).count() == 1 => {
                Some((*x, c))
            }
            _ => None,
        })
        .max_by_key(|(x, _)| *x)?;
    // c*x + rest == 0: x = -c * rest.
    let rest = p.sub(&Poly::var(x).scale(c)?)?;
    Some((x, rest.scale(-c)?))
}

impl Node {
    /// The formula with every solved variable replaced. A formula too
    /// large to rewrite becomes `true`: the search then knows less, which
    /// can only cost a proof, never produce a wrong one.
    fn rewrite(&self, formula: &Formula) -> Formula {
        let mut out = formula.clone();
        for (x, by) in &self.solved {
            match out.substitute(*x, by) {
                Some(next) => out = next,
                None => return Formula::True,
            }
        }
        out
    }

    /// Add a constraint; `false` when it contradicts what is known.
    fn assert(&mut self, formula: &Formula) -> bool {
        let formula = if self.solved.is_empty() {
            formula.clone()
        } else {
            self.rewrite(formula)
        };
        self.assert_rewritten(formula)
    }

    fn assert_rewritten(&mut self, formula: Formula) -> bool {
        match formula {
            Formula::True => true,
            Formula::False => false,
            // Each part is rewritten in turn: an earlier one may have
            // solved for a variable the later ones mention.
            Formula::And(parts) => parts.iter().all(|part| self.assert(part)),
            Formula::Or(parts) => {
                self.ors.push(Choice {
                    arms: Rc::new(parts),
                    last: false,
                });
                true
            }
            atom => {
                if let Formula::Eq(p) = &atom {
                    if let Some((x, by)) = solve_for(p) {
                        return self.solve(x, by);
                    }
                }
                let Some((form, interval)) = bound(&atom) else {
                    // Too large to normalise: dropped, as in `rewrite`.
                    return true;
                };
                self.in_use.extend(form.vars());
                let slot = self.forms.entry(form).or_insert(Interval::ALL);
                *slot = slot.meet(&interval);
                !slot.is_empty()
            }
        }
    }

    /// Add the negated goal. Its variables count as in use from the
    /// start, so the definitions they depend on wake up, but a choice it
    /// contains is the last to be split on.
    fn assert_goal(&mut self, formula: &Formula) -> bool {
        match self.rewrite(formula) {
            Formula::Or(parts) => {
                for part in &parts {
                    part.collect_vars(&mut self.in_use);
                }
                self.ors.push(Choice {
                    arms: Rc::new(parts),
                    last: true,
                });
                true
            }
            other => self.assert_rewritten(other),
        }
    }

    /// Record `x == by` and replace `x` in everything known so far.
    fn solve(&mut self, x: Var, by: Poly) -> bool {
        self.in_use.insert(x);
        let affected: Vec<Poly> = self
            .forms
            .keys()
            .filter(|form| form.mentions(x))
            .cloned()
            .collect();
        let mut again = Vec::new();
        for form in affected {
            let interval = self.forms.remove(&form).unwrap();
            let Some(form) = form.substitute(x, &by) else {
                continue;
            };
            match (interval.lo, interval.hi) {
                (Some(lo), Some(hi)) if lo == hi => {
                    again.extend(
                        lo.checked_neg()
                            .and_then(|c| form.add_const(c))
                            .map(Formula::eq),
                    );
                }
                (lo, hi) => {
                    again.extend(
                        lo.and_then(|lo| form.add_const(lo.checked_neg()?))
                            .map(Formula::ge),
                    );
                    again.extend(hi.and_then(|hi| form.neg()?.add_const(hi)).map(Formula::ge));
                }
            }
        }
        for choice in &mut self.ors {
            if choice.arms.iter().any(|arm| arm.mentions(x)) {
                for arm in Rc::make_mut(&mut choice.arms).iter_mut() {
                    *arm = arm.substitute(x, &by).unwrap_or(Formula::True);
                }
            }
        }
        self.solved.push((x, by));
        again.iter().all(|formula| self.assert(formula))
    }

    /// What is known, as (polynomial, is an equality): `p == 0` or
    /// `p >= 0`. `None` on overflow.
    fn atoms(&self) -> Option<Vec<(Poly, bool)>> {
        let mut atoms = Vec::new();
        for (form, interval) in &self.forms {
            let at = |c: i128, sign: i128| {
                form.scale(sign)?
                    .add_const(sign.checked_mul(c)?.checked_neg()?)
            };
            match (interval.lo, interval.hi) {
                (Some(lo), Some(hi)) if lo == hi => atoms.push((at(lo, 1)?, true)),
                (lo, hi) => {
                    if let Some(lo) = lo {
                        atoms.push((at(lo, 1)?, false));
                    }
                    if let Some(hi) = hi {
                        atoms.push((at(hi, -1)?, false));
                    }
                }
            }
        }
        Some(atoms)
    }

    /// Wake every lazy constraint whose variables are now in use.
    fn wake(&mut self, lazies: &[Lazy]) -> bool {
        loop {
            let ready: Vec<usize> = self
                .asleep
                .iter()
                .copied()
                .filter(|i| {
                    lazies[*i]
                        .triggers
                        .iter()
                        .all(|group| group.iter().any(|v| self.in_use.contains(v)))
                })
                .collect();
            if ready.is_empty() {
                return true;
            }
            self.asleep.retain(|i| !ready.contains(i));
            for i in ready {
                if !self.assert(&lazies[i].formula) {
                    return false;
                }
            }
        }
    }

    /// Whether the bounds already decide the formula.
    fn status(&self, formula: &Formula) -> Option<bool> {
        match formula {
            Formula::True => Some(true),
            Formula::False => Some(false),
            Formula::And(parts) => {
                let mut all = true;
                for part in parts {
                    match self.status(part) {
                        Some(false) => return Some(false),
                        Some(true) => {}
                        None => all = false,
                    }
                }
                all.then_some(true)
            }
            Formula::Or(parts) => {
                let mut none = true;
                for part in parts {
                    match self.status(part) {
                        Some(true) => return Some(true),
                        Some(false) => {}
                        None => none = false,
                    }
                }
                none.then_some(false)
            }
            atom => {
                let (form, wanted) = bound(atom)?;
                let known = self.forms.get(&form)?;
                if known.within(&wanted) {
                    Some(true)
                } else if known.meet(&wanted).is_empty() {
                    Some(false)
                } else {
                    None
                }
            }
        }
    }

    /// Drop the choices the bounds decide and take the forced ones.
    fn propagate(&mut self, lazies: &[Lazy]) -> bool {
        if !self.wake(lazies) {
            return false;
        }
        loop {
            let mut forced = None;
            let mut i = 0;
            while i < self.ors.len() {
                let mut live = Vec::new();
                let mut satisfied = false;
                for (k, part) in self.ors[i].arms.iter().enumerate() {
                    match self.status(part) {
                        Some(true) => satisfied = true,
                        Some(false) => {}
                        None => live.push(k),
                    }
                }
                if satisfied {
                    self.ors.swap_remove(i);
                    continue;
                }
                match live.as_slice() {
                    [] => return false,
                    [only] => {
                        forced = Some(self.ors.swap_remove(i).arms[*only].clone());
                        break;
                    }
                    _ => i += 1,
                }
            }
            let Some(formula) = forced else {
                return true;
            };
            if !self.assert(&formula) || !self.wake(lazies) {
                return false;
            }
        }
    }
}

enum Theory {
    Unsat,
    Sat(Model),
    Unknown,
}

pub(crate) struct Search<'a> {
    lazies: &'a [Lazy],
    /// The number of core variables; products are numbered from here.
    vars: usize,
    nodes: u64,
    steps: u64,
    unknown: Option<&'static str>,
    confirm: &'a mut dyn FnMut(&Model) -> bool,
}

impl<'a> Search<'a> {
    /// Look for an assignment that satisfies `root`, the lazy constraints
    /// it wakes, and `confirm`.
    pub fn run(
        root: &[Formula],
        goal: &Formula,
        lazies: &'a [Lazy],
        vars: usize,
        limits: Limits,
        confirm: &'a mut dyn FnMut(&Model) -> bool,
    ) -> Outcome {
        let mut search = Search {
            lazies,
            vars,
            nodes: limits.nodes,
            steps: limits.steps,
            unknown: None,
            confirm,
        };
        let mut node = Node {
            asleep: (0..lazies.len()).collect(),
            ..Node::default()
        };
        if !root.iter().all(|formula| node.assert(formula)) || !node.assert_goal(goal) {
            return Outcome::Unsat;
        }
        match search.explore(node) {
            Some(_) => Outcome::Sat,
            None => match search.unknown {
                Some(reason) => Outcome::Unknown(reason),
                None => Outcome::Unsat,
            },
        }
    }

    fn explore(&mut self, mut node: Node) -> Option<Model> {
        if self.nodes == 0 {
            self.unknown = Some("the search ran out of budget");
            return None;
        }
        self.nodes -= 1;
        if !node.propagate(self.lazies) {
            return None;
        }
        let theory = self.theory(&node);
        if matches!(theory, Theory::Unsat) {
            return None;
        }
        // Split on the choice with the fewest open arms, the goal last.
        let open = |choice: &Choice| {
            (
                choice.last,
                choice
                    .arms
                    .iter()
                    .filter(|arm| node.status(arm) != Some(false))
                    .count(),
            )
        };
        let Some(pick) = (0..node.ors.len()).min_by_key(|i| open(&node.ors[*i])) else {
            match theory {
                Theory::Sat(model) if (self.confirm)(&model) => return Some(model),
                Theory::Sat(_) => self.unknown = Some("the arithmetic is beyond the solver"),
                _ => self.unknown = Some("the integer solver ran out of budget"),
            }
            return None;
        };
        let arms = node.ors.swap_remove(pick).arms;
        for arm in arms.iter() {
            if node.status(arm) == Some(false) {
                continue;
            }
            let mut child = node.clone();
            if child.assert(arm) {
                if let Some(model) = self.explore(child) {
                    return Some(model);
                }
            }
            if self.nodes == 0 {
                return None;
            }
        }
        None
    }

    /// Whether the atoms of a node have an integer solution.
    fn theory(&mut self, node: &Node) -> Theory {
        let Some(mut atoms) = node.atoms() else {
            return Theory::Unknown;
        };
        let solved = &node.solved;

        // Products become variables of their own.
        let mut products: BTreeMap<Mono, Var> = BTreeMap::new();
        for (p, _) in &atoms {
            for (m, _) in p.terms() {
                if m.degree() >= 2 && !products.contains_key(m) {
                    let id = (self.vars + products.len()) as Var;
                    products.insert(m.clone(), id);
                }
            }
        }
        if !products.is_empty() {
            atoms.extend(product_lemmas(&atoms, &products));
        }
        let mut constraints = Vec::with_capacity(atoms.len());
        for (p, eq) in &atoms {
            let terms =
                p.terms()
                    .filter(|(m, _)| !m.is_unit())
                    .map(|(m, c)| match m.0.as_slice() {
                        [x] => (*x, c),
                        _ => (products[m], c),
                    });
            let Ok(lin) = Lin::new(terms, p.constant_term()) else {
                return Theory::Unknown;
            };
            constraints.push(if *eq {
                Constraint::Eq(lin)
            } else {
                Constraint::Ge(lin)
            });
        }
        let fresh = (self.vars + products.len()) as Var;
        let mut model = match omega::solve(&constraints, fresh, &mut self.steps) {
            Ok(Some(model)) => model,
            Ok(None) => return Theory::Unsat,
            Err(_) => return Theory::Unknown,
        };
        model.retain(|v, _| (*v as usize) < self.vars);
        for (x, by) in solved.iter().rev() {
            match by.eval(&|v| model.get(&v).copied().unwrap_or(0)) {
                Some(value) => model.insert(*x, value),
                None => return Theory::Unknown,
            };
        }
        Theory::Sat(model)
    }
}

/// Valid constraints on products: the product of two linear constraints
/// that hold is non-negative. Only products that mention nothing new are
/// kept, so the system does not grow new variables. This is what bounds
/// `r*W + c` below `H*W`, and `a * b` by the ranges of `a` and `b`.
fn product_lemmas(atoms: &[(Poly, bool)], products: &BTreeMap<Mono, Var>) -> Vec<(Poly, bool)> {
    let in_products: BTreeSet<Var> = products.keys().flat_map(|m| m.0.iter().copied()).collect();
    let known = |p: &Poly| {
        p.terms()
            .all(|(m, _)| m.degree() <= 1 || products.contains_key(m))
    };
    let derived = variable_bounds(atoms, &in_products);
    let factors: Vec<&(Poly, bool)> = atoms
        .iter()
        .filter(|(p, _)| p.is_linear() && p.vars().iter().any(|v| in_products.contains(v)))
        .chain(derived.iter().filter(|bound| !atoms.contains(bound)))
        .take(64)
        .collect();
    let mut lemmas = Vec::new();
    // A product whose variables all come in pairs is a square.
    for m in products.keys() {
        if m.0
            .chunks(2)
            .all(|pair| pair.len() == 2 && pair[0] == pair[1])
        {
            lemmas.push((Poly::mono(m.clone(), 1), false));
        }
    }
    for (i, (p, p_eq)) in factors.iter().enumerate() {
        if *p_eq {
            // p == 0, so p * x == 0 for every variable x.
            for x in &in_products {
                if let Some(product) = p.mul(&Poly::var(*x)).filter(|product| known(product)) {
                    lemmas.push((product, true));
                }
            }
            continue;
        }
        for (q, q_eq) in &factors[i..] {
            if *q_eq {
                continue;
            }
            if let Some(product) = p.mul(q).filter(|product| known(product)) {
                lemmas.push((product, false));
            }
        }
        if lemmas.len() > 2000 {
            break;
        }
    }
    lemmas
}

/// Bounds on single variables that follow from the linear atoms by
/// interval propagation (`c >= 0` and `c < W` give `W >= 1`), as atoms.
fn variable_bounds(atoms: &[(Poly, bool)], wanted: &BTreeSet<Var>) -> Vec<(Poly, bool)> {
    let mut out = Vec::new();
    for (v, interval) in variable_intervals(atoms) {
        if !wanted.contains(&v) {
            continue;
        }
        if let Some(p) = interval
            .lo
            .and_then(|lo| Poly::var(v).add_const(lo.checked_neg()?))
        {
            out.push((p, false));
        }
        if let Some(p) = interval.hi.and_then(|hi| Poly::var(v).neg()?.add_const(hi)) {
            out.push((p, false));
        }
    }
    out
}

fn variable_intervals(atoms: &[(Poly, bool)]) -> BTreeMap<Var, Interval> {
    let mut bounds: BTreeMap<Var, Interval> = BTreeMap::new();
    let get =
        |bounds: &BTreeMap<Var, Interval>, v: Var| bounds.get(&v).copied().unwrap_or(Interval::ALL);
    for _ in 0..3 {
        for (p, eq) in atoms.iter().filter(|(p, _)| p.is_linear()) {
            for (m, a) in p.terms() {
                let [x] = m.0.as_slice() else { continue };
                // a*x + rest >= 0 (and <= 0 for an equality).
                let mut rest = Interval::point(p.constant_term());
                for (other, c) in p.terms() {
                    if let [y] = other.0.as_slice() {
                        if y != x {
                            rest = rest.add(&get(&bounds, *y).scale(c));
                        }
                    }
                }
                let mut tighter = Interval::ALL;
                // a*x >= -rest.hi, and for an equality a*x <= -rest.lo.
                let (at_least, at_most) =
                    (rest.hi.map(|hi| -hi), rest.lo.filter(|_| *eq).map(|lo| -lo));
                if a > 0 {
                    tighter.lo = at_least.map(|v| -(-v).div_euclid(a));
                    tighter.hi = at_most.map(|v| v.div_euclid(a));
                } else {
                    tighter.hi = at_least.map(|v| (-v).div_euclid(-a));
                    tighter.lo = at_most.map(|v| -v.div_euclid(-a));
                }
                let slot = bounds.entry(*x).or_insert(Interval::ALL);
                *slot = slot.meet(&tighter);
            }
        }
    }
    bounds
}

/// The range of `target` that follows from `root` by intervals alone:
/// no search, so it is cheap enough to run on every arithmetic node.
pub(crate) fn range(root: &[Formula], lazies: &[Lazy], target: &Poly) -> Interval {
    let mut node = Node {
        asleep: (0..lazies.len()).collect(),
        ..Node::default()
    };
    node.in_use.extend(target.vars());
    if !root.iter().all(|formula| node.assert(formula)) || !node.propagate(lazies) {
        return Interval::new(1, 0);
    }
    let mut rewritten = target.clone();
    for (x, by) in &node.solved {
        match rewritten.substitute(*x, by) {
            Some(next) => rewritten = next,
            None => return Interval::ALL,
        }
    }
    let target = &rewritten;
    let Some(atoms) = node.atoms() else {
        return Interval::ALL;
    };
    let known = variable_intervals(&atoms);
    let of = |v: Var| known.get(&v).copied().unwrap_or(Interval::ALL);
    let mut total = Interval::point(0);
    for (m, c) in target.terms() {
        let mut product = Interval::point(c);
        let mut i = 0;
        while i < m.0.len() {
            // Equal neighbours are a square, which is never negative.
            if i + 1 < m.0.len() && m.0[i] == m.0[i + 1] {
                product = product.mul(&of(m.0[i]).square());
                i += 2;
            } else {
                product = product.mul(&of(m.0[i]));
                i += 1;
            }
        }
        total = total.add(&product);
    }
    // A bound on the whole form, when one is known, can be tighter.
    match bound(&Formula::Ge(target.clone())) {
        Some((form, _)) if form == *target => node
            .forms
            .get(&form)
            .map_or(total, |known| total.meet(known)),
        _ => total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x(v: Var) -> Poly {
        Poly::var(v)
    }

    fn k(c: i128) -> Poly {
        Poly::constant(c)
    }

    fn run(root: &[Formula], lazies: &[Lazy], vars: usize) -> Outcome {
        model_of(root, lazies, vars).0
    }

    /// The outcome, and the model that was confirmed when there is one.
    fn model_of(root: &[Formula], lazies: &[Lazy], vars: usize) -> (Outcome, Model) {
        let mut found = Model::new();
        let mut exact = |model: &Model| {
            let value = |v: Var| model.get(&v).copied().unwrap_or(0);
            let holds = root
                .iter()
                .chain(lazies.iter().map(|l| &l.formula))
                .all(|f| f.eval(&value) == Some(true));
            if holds {
                found = model.clone();
            }
            holds
        };
        let outcome = Search::run(
            root,
            &Formula::True,
            lazies,
            vars,
            Limits::default(),
            &mut exact,
        );
        (outcome, found)
    }

    #[test]
    fn forms_are_normalised() {
        // 2x - 4y + 6 >= 0 is x - 2y >= -3.
        let p =
            Poly::from_terms([(Mono(vec![0]), 2), (Mono(vec![1]), -4), (Mono(vec![]), 6)]).unwrap();
        let (form, interval) = bound(&Formula::Ge(p)).unwrap();
        assert_eq!(form, x(0).sub(&x(1).scale(2).unwrap()).unwrap());
        assert_eq!(
            interval,
            Interval {
                lo: Some(-3),
                hi: None
            }
        );
        // -3x + 7 >= 0 is x <= 2.
        let q = x(0).scale(-3).unwrap().add_const(7).unwrap();
        assert_eq!(
            bound(&Formula::Ge(q)).unwrap().1,
            Interval {
                lo: None,
                hi: Some(2)
            }
        );
        // 2x - 1 == 0 has no integer solution.
        let r = x(0).scale(2).unwrap().add_const(-1).unwrap();
        assert!(bound(&Formula::Eq(r)).unwrap().1.is_empty());
    }

    #[test]
    fn case_splits_find_the_arm_that_works() {
        // (x <= -1 or x >= 5) and 0 <= x <= 9 leaves x = 5.
        let root = [
            Formula::or([
                Formula::le(&x(0), &k(-1)).unwrap(),
                Formula::le(&k(5), &x(0)).unwrap(),
            ]),
            Formula::le(&k(0), &x(0)).unwrap(),
            Formula::le(&x(0), &k(9)).unwrap(),
        ];
        let (outcome, model) = model_of(&root, &[], 1);
        assert!(matches!(outcome, Outcome::Sat));
        assert_eq!(model[&0], 5);
        let mut closed = root.to_vec();
        closed.push(Formula::le(&x(0), &k(4)).unwrap());
        assert!(matches!(run(&closed, &[], 1), Outcome::Unsat));
    }

    #[test]
    fn lazy_definitions_wake_when_their_variable_is_used() {
        // v2 = max(v0, v1), asleep until v2 is mentioned.
        let max = Lazy {
            triggers: vec![vec![2]],
            formula: Formula::and([
                Formula::le(&x(0), &x(2)).unwrap(),
                Formula::le(&x(1), &x(2)).unwrap(),
                Formula::or([
                    Formula::same(&x(2), &x(0)).unwrap(),
                    Formula::same(&x(2), &x(1)).unwrap(),
                ]),
            ]),
        };
        // max(v0, v1) < v0 is impossible.
        assert!(matches!(
            run(
                &[Formula::lt(&x(2), &x(0)).unwrap()],
                std::slice::from_ref(&max),
                3
            ),
            Outcome::Unsat
        ));
        // max(v0, v1) > v0 needs v1 > v0.
        let (outcome, model) = model_of(&[Formula::lt(&x(0), &x(2)).unwrap()], &[max], 3);
        assert!(matches!(outcome, Outcome::Sat));
        let value = |v: Var| model.get(&v).copied().unwrap_or(0);
        assert!(value(1) > value(0) && value(2) == value(1));
    }

    #[test]
    fn products_are_bounded_by_their_factors() {
        // 0 <= r < H, 0 <= c < W: r*W + c < H*W. Variables: r=0, c=1, H=2, W=3.
        let facts = [
            Formula::le(&k(0), &x(0)).unwrap(),
            Formula::lt(&x(0), &x(2)).unwrap(),
            Formula::le(&k(0), &x(1)).unwrap(),
            Formula::lt(&x(1), &x(3)).unwrap(),
        ];
        let flat = x(0).mul(&x(3)).unwrap().add(&x(1)).unwrap();
        let size = x(2).mul(&x(3)).unwrap();
        let mut root = facts.to_vec();
        root.push(Formula::le(&size, &flat).unwrap());
        assert!(matches!(run(&root, &[], 4), Outcome::Unsat));
        // With `c <= W` instead, r = 0, c = W = 1, H = 1 breaks it.
        let mut loose = facts[..3].to_vec();
        loose.push(Formula::le(&x(1), &x(3)).unwrap());
        loose.push(Formula::le(&size, &flat).unwrap());
        assert!(!matches!(run(&loose, &[], 4), Outcome::Unsat));
    }

    #[test]
    fn ranges_follow_from_intervals() {
        // 0 <= a <= 1e9, 0 <= b <= 1e9: a * b is in 0..=1e18 and a - b in -1e9..=1e9.
        let big = 1_000_000_000;
        let root = [
            Formula::le(&k(0), &x(0)).unwrap(),
            Formula::le(&x(0), &k(big)).unwrap(),
            Formula::le(&k(0), &x(1)).unwrap(),
            Formula::le(&x(1), &k(big)).unwrap(),
        ];
        assert_eq!(
            range(&root, &[], &x(0).mul(&x(1)).unwrap()),
            Interval::new(0, big * big)
        );
        assert_eq!(
            range(&root, &[], &x(0).sub(&x(1)).unwrap()),
            Interval::new(-big, big)
        );
        assert_eq!(
            range(&root, &[], &x(2).mul(&x(2)).unwrap()),
            Interval {
                lo: Some(0),
                hi: None
            }
        );
        let mut clash = root.to_vec();
        clash.push(Formula::le(&x(0), &k(-1)).unwrap());
        assert!(range(&clash, &[], &x(0)).is_empty());
    }

    #[test]
    fn the_budget_ends_the_search() {
        let root: Vec<Formula> = (0..12)
            .map(|v| {
                Formula::or([
                    Formula::same(&x(v), &k(0)).unwrap(),
                    Formula::same(&x(v), &k(1)).unwrap(),
                ])
            })
            .chain([Formula::le(
                &k(13),
                &(0..12).fold(k(0), |sum, v| sum.add(&x(v)).unwrap()),
            )
            .unwrap()])
            .collect();
        let mut never = |_: &Model| false;
        let limits = Limits {
            nodes: 1,
            ..Limits::default()
        };
        assert!(matches!(
            Search::run(&root, &Formula::True, &[], 12, limits, &mut never),
            Outcome::Unknown(_)
        ));
        assert!(matches!(run(&root, &[], 12), Outcome::Unsat));
    }
}
