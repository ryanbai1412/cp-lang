//! The Omega test: a complete decision procedure for conjunctions of
//! linear constraints over the integers (Presburger arithmetic without
//! quantifiers), after Pugh, "The Omega test" (1991).
//!
//! Equalities are solved exactly and substituted away. Inequalities are
//! eliminated one variable at a time: when the projection is exact
//! (every lower or every upper bound on the variable has coefficient
//! one) this is Fourier-Motzkin; when it is not, the real shadow, the
//! dark shadow and finally the splinters decide the gap between the
//! rationals and the integers. A satisfiable system comes back with a
//! model that prefers values close to zero.

use std::collections::{BTreeMap, HashMap};

pub type Var = u32;

/// Why the solver gave up; the caller reports the query as unknown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stop {
    /// A coefficient left `i128`.
    Overflow,
    /// The work budget ran out.
    Budget,
}

type Res<T> = Result<T, Stop>;

fn mul(a: i128, b: i128) -> Res<i128> {
    a.checked_mul(b).ok_or(Stop::Overflow)
}

fn add(a: i128, b: i128) -> Res<i128> {
    a.checked_add(b).ok_or(Stop::Overflow)
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.unsigned_abs(), b.unsigned_abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a as i128
}

/// `sum(coefficient * variable) + constant`, with the variables in order
/// and no zero coefficient.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Lin {
    terms: Vec<(Var, i128)>,
    constant: i128,
}

impl Lin {
    pub fn new(terms: impl IntoIterator<Item = (Var, i128)>, constant: i128) -> Res<Lin> {
        let mut sorted: Vec<(Var, i128)> = terms.into_iter().collect();
        sorted.sort_unstable_by_key(|(v, _)| *v);
        let mut merged: Vec<(Var, i128)> = Vec::with_capacity(sorted.len());
        for (v, c) in sorted {
            match merged.last_mut() {
                Some((last, sum)) if *last == v => *sum = add(*sum, c)?,
                _ => merged.push((v, c)),
            }
        }
        merged.retain(|(_, c)| *c != 0);
        Ok(Lin {
            terms: merged,
            constant,
        })
    }

    pub fn terms(&self) -> &[(Var, i128)] {
        &self.terms
    }

    pub fn constant(&self) -> i128 {
        self.constant
    }

    fn coeff(&self, v: Var) -> i128 {
        match self.terms.binary_search_by_key(&v, |(var, _)| *var) {
            Ok(i) => self.terms[i].1,
            Err(_) => 0,
        }
    }

    /// `k1 * self + k2 * other`.
    fn combine(&self, k1: i128, other: &Lin, k2: i128) -> Res<Lin> {
        let mut terms = Vec::with_capacity(self.terms.len() + other.terms.len());
        let (mut i, mut j) = (0, 0);
        while i < self.terms.len() || j < other.terms.len() {
            let left = self.terms.get(i);
            let right = other.terms.get(j);
            let (v, c) = match (left, right) {
                (Some((a, x)), Some((b, y))) if a == b => {
                    i += 1;
                    j += 1;
                    (*a, add(mul(*x, k1)?, mul(*y, k2)?)?)
                }
                (Some((a, x)), Some((b, _))) if a < b => {
                    i += 1;
                    (*a, mul(*x, k1)?)
                }
                (Some((a, x)), None) => {
                    i += 1;
                    (*a, mul(*x, k1)?)
                }
                (_, Some((b, y))) => {
                    j += 1;
                    (*b, mul(*y, k2)?)
                }
                (None, None) => unreachable!(),
            };
            if c != 0 {
                terms.push((v, c));
            }
        }
        let constant = add(mul(self.constant, k1)?, mul(other.constant, k2)?)?;
        Ok(Lin { terms, constant })
    }

    fn without(&self, v: Var) -> Lin {
        let terms = self
            .terms
            .iter()
            .copied()
            .filter(|(var, _)| *var != v)
            .collect();
        Lin {
            terms,
            constant: self.constant,
        }
    }

    /// Replace `v` by `by`.
    fn substitute(&self, v: Var, by: &Lin) -> Res<Lin> {
        match self.coeff(v) {
            0 => Ok(self.clone()),
            c => self.without(v).combine(1, by, c),
        }
    }

    fn eval(&self, model: &Model) -> Res<i128> {
        let mut sum = self.constant;
        for (v, c) in &self.terms {
            sum = add(sum, mul(*c, model.get(v).copied().unwrap_or(0))?)?;
        }
        Ok(sum)
    }

    /// Divide by the gcd of the coefficients, rounding the constant down:
    /// the same constraint `>= 0` over the integers, as tight as it gets.
    fn tighten(&mut self) {
        let g = self.terms.iter().fold(0, |g, (_, c)| gcd(g, *c));
        if g > 1 {
            for (_, c) in &mut self.terms {
                *c /= g;
            }
            self.constant = self.constant.div_euclid(g);
        }
    }
}

/// One linear constraint: `lin == 0` or `lin >= 0`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Constraint {
    Eq(Lin),
    Ge(Lin),
}

/// A value for every variable that needs one; the rest are zero.
pub type Model = BTreeMap<Var, i128>;

/// Decide a conjunction of constraints. `Ok(None)` means no integer
/// solution exists. `fresh` is a variable number above every one in use;
/// `budget` bounds the work and is decreased as it is spent.
pub fn solve(constraints: &[Constraint], fresh: Var, budget: &mut u64) -> Res<Option<Model>> {
    // Constraints that share no variable are independent problems.
    let mut parent: HashMap<Var, Var> = HashMap::new();
    fn find(parent: &mut HashMap<Var, Var>, v: Var) -> Var {
        let p = *parent.entry(v).or_insert(v);
        if p == v {
            return v;
        }
        let root = find(parent, p);
        parent.insert(v, root);
        root
    }
    let lin_of = |c: &Constraint| match c {
        Constraint::Eq(lin) | Constraint::Ge(lin) => lin.clone(),
    };
    for c in constraints {
        let lin = lin_of(c);
        if let Some((first, _)) = lin.terms.first() {
            let root = find(&mut parent, *first);
            for (v, _) in &lin.terms[1..] {
                let other = find(&mut parent, *v);
                parent.insert(other, root);
            }
        }
    }
    let mut groups: BTreeMap<Option<Var>, (Vec<Lin>, Vec<Lin>)> = BTreeMap::new();
    for c in constraints {
        let lin = lin_of(c);
        let key = lin.terms.first().map(|(v, _)| find(&mut parent, *v));
        let group = groups.entry(key).or_default();
        match c {
            Constraint::Eq(_) => group.0.push(lin),
            Constraint::Ge(_) => group.1.push(lin),
        }
    }
    let mut solver = Solver { fresh, budget };
    let mut model = Model::new();
    for (_, (eqs, ges)) in groups {
        match solver.solve(eqs, ges)? {
            Some(part) => model.extend(part),
            None => return Ok(None),
        }
    }
    Ok(Some(model))
}

struct Solver<'a> {
    fresh: Var,
    budget: &'a mut u64,
}

/// `a` reduced into `[-m/2, m/2)`.
fn mod_hat(a: i128, m: i128) -> Res<i128> {
    let shifted = add(mul(a, 2)?, m)?;
    Ok(a - mul(m, shifted.div_euclid(mul(m, 2)?))?)
}

/// The value closest to zero in `lo..=hi`, if there is one.
fn closest_to_zero(lo: Option<i128>, hi: Option<i128>) -> Option<i128> {
    match (lo, hi) {
        (Some(lo), Some(hi)) if lo > hi => None,
        (Some(lo), _) if lo > 0 => Some(lo),
        (_, Some(hi)) if hi < 0 => Some(hi),
        _ => Some(0),
    }
}

impl Solver<'_> {
    fn spend(&mut self, work: usize) -> Res<()> {
        let work = work as u64 + 1;
        if *self.budget < work {
            *self.budget = 0;
            return Err(Stop::Budget);
        }
        *self.budget -= work;
        Ok(())
    }

    fn solve(&mut self, mut eqs: Vec<Lin>, mut ges: Vec<Lin>) -> Res<Option<Model>> {
        self.spend(eqs.len() + ges.len())?;
        // Substitutions made so far, oldest first: the variable equals the form.
        let mut solved: Vec<(Var, Lin)> = Vec::new();
        while let Some(mut eq) = eqs.pop() {
            self.spend(eqs.len() + ges.len())?;
            let g = eq.terms.iter().fold(0, |g, (_, c)| gcd(g, *c));
            if g == 0 {
                if eq.constant != 0 {
                    return Ok(None);
                }
                continue;
            }
            if eq.constant % g != 0 {
                return Ok(None);
            }
            for (_, c) in &mut eq.terms {
                *c /= g;
            }
            eq.constant /= g;
            let unit = eq.terms.iter().find(|(_, c)| c.abs() == 1).copied();
            let (var, by) = match unit {
                // c*x + rest == 0 with c = +-1: x = -c * rest.
                Some((x, c)) => (x, eq.without(x).combine(-c, &Lin::default(), 0)?),
                None => {
                    // No unit coefficient: trade the variable with the
                    // smallest one for a fresh variable; the equation
                    // comes back with smaller coefficients.
                    let (x, a) = *eq.terms.iter().min_by_key(|(_, c)| c.abs()).unwrap();
                    let m = add(a.abs(), 1)?;
                    let sign = a.signum();
                    let sigma = self.fresh;
                    self.fresh += 1;
                    let mut terms = vec![(sigma, mul(-sign, m)?)];
                    for (v, c) in &eq.terms {
                        if *v != x {
                            terms.push((*v, mul(sign, mod_hat(*c, m)?)?));
                        }
                    }
                    let by = Lin::new(terms, mul(sign, mod_hat(eq.constant, m)?)?)?;
                    eqs.push(eq.substitute(x, &by)?);
                    (x, by)
                }
            };
            for other in eqs.iter_mut().chain(ges.iter_mut()) {
                *other = other.substitute(var, &by)?;
            }
            solved.push((var, by));
        }

        let model = self.solve_inequalities(ges)?;
        let Some(mut model) = model else {
            return Ok(None);
        };
        for (var, by) in solved.iter().rev() {
            let value = by.eval(&model)?;
            model.insert(*var, value);
        }
        Ok(Some(model))
    }

    fn solve_inequalities(&mut self, ges: Vec<Lin>) -> Res<Option<Model>> {
        // Tighten, and keep only the strongest of the constraints that
        // have the same left-hand side.
        let mut strongest: BTreeMap<Vec<(Var, i128)>, i128> = BTreeMap::new();
        for mut ge in ges {
            ge.tighten();
            if ge.terms.is_empty() {
                if ge.constant < 0 {
                    return Ok(None);
                }
                continue;
            }
            let slot = strongest.entry(ge.terms).or_insert(ge.constant);
            *slot = (*slot).min(ge.constant);
        }
        // `e + c >= 0` and `-e + d >= 0` clash when c + d < 0 and pin `e`
        // when c + d == 0.
        let mut pinned: Option<Lin> = None;
        for (terms, constant) in &strongest {
            if terms[0].1 < 0 {
                continue;
            }
            let negated: Vec<(Var, i128)> = terms.iter().map(|(v, c)| (*v, -c)).collect();
            if let Some(other) = strongest.get(&negated) {
                let slack = add(*constant, *other)?;
                if slack < 0 {
                    return Ok(None);
                }
                if slack == 0 {
                    pinned = Some(Lin {
                        terms: terms.clone(),
                        constant: *constant,
                    });
                    break;
                }
            }
        }
        let mut ges: Vec<Lin> = strongest
            .into_iter()
            .map(|(terms, constant)| Lin { terms, constant })
            .collect();
        if let Some(eq) = pinned {
            let negated: Vec<(Var, i128)> = eq.terms.iter().map(|(v, c)| (*v, -c)).collect();
            ges.retain(|ge| ge.terms != eq.terms && ge.terms != negated);
            return self.solve(vec![eq], ges);
        }
        if ges.is_empty() {
            return Ok(Some(Model::new()));
        }
        self.spend(ges.len())?;

        // Pick the variable that is cheapest to eliminate: one bounded on
        // one side only, else one with an exact projection, else any.
        let mut shape: BTreeMap<Var, (usize, usize, bool, bool)> = BTreeMap::new();
        for ge in &ges {
            for (v, c) in &ge.terms {
                let entry = shape.entry(*v).or_insert((0, 0, true, true));
                if *c > 0 {
                    entry.0 += 1;
                    entry.2 &= *c == 1;
                } else {
                    entry.1 += 1;
                    entry.3 &= *c == -1;
                }
            }
        }
        let rank = |(lowers, uppers, unit_lowers, unit_uppers): (usize, usize, bool, bool)| {
            let one_sided = lowers == 0 || uppers == 0;
            let exact = unit_lowers || unit_uppers;
            let growth = (lowers * uppers) as i64 - (lowers + uppers) as i64;
            (!one_sided, !exact, growth)
        };
        let (x, (lowers, uppers, unit_lowers, unit_uppers)) =
            shape.into_iter().min_by_key(|(_, s)| rank(*s)).unwrap();
        let exact = lowers == 0 || uppers == 0 || unit_lowers || unit_uppers;

        let (mut lower, mut upper, mut rest) = (Vec::new(), Vec::new(), Vec::new());
        for ge in ges.iter().cloned() {
            match ge.coeff(x) {
                0 => rest.push(ge),
                c if c > 0 => lower.push(ge),
                _ => upper.push(ge),
            }
        }
        self.spend(lower.len() * upper.len())?;
        if (rest.len() + lower.len() * upper.len()) > 4000 {
            return Err(Stop::Budget);
        }

        // The shadow of the system on the other variables. With `gap` the
        // dark shadow: it leaves room for an integer between the bounds.
        let shadow = |gap: bool| -> Res<Vec<Lin>> {
            let mut out = rest.clone();
            for lo in &lower {
                for up in &upper {
                    let (a, b) = (lo.coeff(x), -up.coeff(x));
                    let mut combined = lo.combine(b, up, a)?;
                    if gap {
                        combined.constant = add(combined.constant, -mul(a - 1, b - 1)?)?;
                    }
                    out.push(combined);
                }
            }
            Ok(out)
        };
        // The range the constraints leave for `x` once the rest is fixed.
        let room = |model: &Model| -> Res<Option<i128>> {
            let (mut lo, mut hi): (Option<i128>, Option<i128>) = (None, None);
            for ge in &lower {
                // a*x + rest >= 0: x >= ceil(-rest / a).
                let (a, value) = (ge.coeff(x), ge.without(x).eval(model)?);
                let bound = -value.div_euclid(a);
                lo = Some(lo.map_or(bound, |lo| lo.max(bound)));
            }
            for ge in &upper {
                // -b*x + rest >= 0: x <= floor(rest / b).
                let (b, value) = (-ge.coeff(x), ge.without(x).eval(model)?);
                let bound = value.div_euclid(b);
                hi = Some(hi.map_or(bound, |hi| hi.min(bound)));
            }
            Ok(closest_to_zero(lo, hi))
        };

        let Some(mut model) = self.solve_inequalities(shadow(false)?)? else {
            return Ok(None);
        };
        if let Some(value) = room(&model)? {
            model.insert(x, value);
            return Ok(Some(model));
        }
        debug_assert!(!exact, "an exact projection always leaves room");
        if let Some(mut model) = self.solve_inequalities(shadow(true)?)? {
            let value = room(&model)?.expect("the dark shadow leaves room for an integer");
            model.insert(x, value);
            return Ok(Some(model));
        }
        // Splinters: any solution left sits just above one of the lower bounds.
        let widest = upper.iter().map(|up| -up.coeff(x)).max().unwrap();
        for lo in &lower {
            let a = lo.coeff(x);
            let last = (mul(widest, a)? - widest - a) / widest;
            for i in 0..=last {
                let mut eq = lo.clone();
                eq.constant = add(eq.constant, -i)?;
                if let Some(model) = self.solve(vec![eq], ges.clone())? {
                    return Ok(Some(model));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lin(terms: &[(Var, i128)], constant: i128) -> Lin {
        Lin::new(terms.iter().copied(), constant).unwrap()
    }

    fn ge(terms: &[(Var, i128)], constant: i128) -> Constraint {
        Constraint::Ge(lin(terms, constant))
    }

    fn eq(terms: &[(Var, i128)], constant: i128) -> Constraint {
        Constraint::Eq(lin(terms, constant))
    }

    fn run(constraints: &[Constraint]) -> Option<Model> {
        let model = solve(constraints, 1000, &mut 1_000_000).unwrap();
        if let Some(model) = &model {
            assert!(
                holds(constraints, model),
                "the model {model:?} breaks {constraints:?}"
            );
        }
        model
    }

    fn holds(constraints: &[Constraint], model: &Model) -> bool {
        constraints.iter().all(|c| match c {
            Constraint::Eq(lin) => lin.eval(model).unwrap() == 0,
            Constraint::Ge(lin) => lin.eval(model).unwrap() >= 0,
        })
    }

    #[test]
    fn bounds_and_differences() {
        // 0 <= i < n, n <= 5, i >= 5 has no solution.
        assert!(run(&[
            ge(&[(0, 1)], 0),
            ge(&[(1, 1), (0, -1)], -1),
            ge(&[(1, -1)], 5),
            ge(&[(0, 1)], -5)
        ])
        .is_none());
        // Dropping the last constraint leaves i = 0, n = 1.
        let model = run(&[
            ge(&[(0, 1)], 0),
            ge(&[(1, 1), (0, -1)], -1),
            ge(&[(1, -1)], 5),
        ])
        .unwrap();
        assert_eq!((model[&0], model[&1]), (0, 1));
    }

    #[test]
    fn equalities_without_unit_coefficients() {
        // 7x + 12y + 31z == 17, 3x + 5y + 14z == 7 (Pugh's example), boxed.
        let system = [
            eq(&[(0, 7), (1, 12), (2, 31)], -17),
            eq(&[(0, 3), (1, 5), (2, 14)], -7),
            ge(&[(0, 1)], -1),
            ge(&[(0, -1)], 40),
            ge(&[(1, 1)], 50),
            ge(&[(1, -1)], 50),
        ];
        let model = run(&system).unwrap();
        assert_eq!(7 * model[&0] + 12 * model[&1] + 31 * model[&2], 17);
        // 6x + 9y == 4 has no integer solution: 3 does not divide 4.
        assert!(run(&[eq(&[(0, 6), (1, 9)], -4)]).is_none());
        // 6x + 10y + 15z == 1 has one although no coefficient is a unit.
        assert!(run(&[eq(&[(0, 6), (1, 10), (2, 15)], -1)]).is_some());
    }

    #[test]
    fn integer_gaps_between_rational_bounds() {
        // 1 <= 2x <= 1: the rationals allow x = 1/2, the integers nothing.
        assert!(run(&[ge(&[(0, 2)], -1), ge(&[(0, -2)], 1)]).is_none());
        // 27 <= 11x + 13y <= 45, -10 <= 7x - 9y <= 4: Pugh's example of a
        // system with rational solutions and no integer one.
        let system = [
            ge(&[(0, 11), (1, 13)], -27),
            ge(&[(0, -11), (1, -13)], 45),
            ge(&[(0, 7), (1, -9)], 10),
            ge(&[(0, -7), (1, 9)], 4),
        ];
        assert!(run(&system).is_none());
        // 3x <= 2y <= 3x + 1 is satisfied only on a thin integer lattice.
        let thin = [
            ge(&[(1, 2), (0, -3)], 0),
            ge(&[(0, 3), (1, -2)], 1),
            ge(&[(0, 1)], -5),
        ];
        let model = run(&thin).unwrap();
        assert!(model[&0] >= 5);
    }

    #[test]
    fn independent_parts_are_solved_apart() {
        assert!(run(&[ge(&[(0, 1)], -3), ge(&[(5, 1)], 0), ge(&[(5, -1)], -1)]).is_none());
        let model = run(&[ge(&[(0, 1)], -3), ge(&[(5, -1)], -2)]).unwrap();
        assert_eq!((model[&0], model[&5]), (3, -2));
    }

    #[test]
    fn budget_and_overflow_stop_cleanly() {
        let system = [
            ge(&[(0, 3), (1, 5)], -7),
            ge(&[(0, -3), (1, -5)], 9),
            ge(&[(0, 2), (1, -7)], 3),
        ];
        assert_eq!(solve(&system, 10, &mut 2), Err(Stop::Budget));
        let huge = i128::MAX / 2;
        let overflow = [
            ge(&[(0, huge), (1, 3)], 0),
            ge(&[(0, -huge), (1, 5)], 0),
            ge(&[(1, -1)], huge),
        ];
        assert_eq!(solve(&overflow, 10, &mut 1000), Err(Stop::Overflow));
    }

    /// Every small system agrees with a brute-force search of a box.
    #[test]
    fn agrees_with_brute_force() {
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = move |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % n
        };
        for round in 0..3000 {
            let vars = 1 + next(3) as Var;
            let mut system = Vec::new();
            // A box keeps the brute-force search finite.
            for v in 0..vars {
                system.push(ge(&[(v, 1)], 4));
                system.push(ge(&[(v, -1)], 4));
            }
            for _ in 0..1 + next(4) {
                let terms: Vec<(Var, i128)> =
                    (0..vars).map(|v| (v, next(11) as i128 - 5)).collect();
                let constant = next(21) as i128 - 10;
                system.push(if next(4) == 0 {
                    eq(&terms, constant)
                } else {
                    ge(&terms, constant)
                });
            }
            let mut expected = false;
            let side = 9i128;
            for code in 0..side.pow(vars) {
                let model: Model = (0..vars)
                    .map(|v| (v, code / side.pow(v) % side - 4))
                    .collect();
                expected |= holds(&system, &model);
            }
            assert_eq!(
                run(&system).is_some(),
                expected,
                "round {round}: {system:?}"
            );
        }
    }
}
