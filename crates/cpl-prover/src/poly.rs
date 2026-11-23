//! Polynomials over integer variables with exact `i128` coefficients.
//!
//! Every integer expression is brought to this normal form, so two
//! expressions that are equal as polynomials (`(a * b) * c` and
//! `a * (b * c)`) become the same value and need no further proof. Every
//! operation is checked: `None` means a coefficient left `i128`, and the
//! caller answers "unknown" instead of guessing.

use std::collections::{BTreeMap, BTreeSet};

/// A variable of the core formula.
pub type Var = u32;

/// A product of variables, sorted, with repeats for powers. Empty is 1.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct Mono(pub Vec<Var>);

impl Mono {
    pub fn is_unit(&self) -> bool {
        self.0.is_empty()
    }

    pub fn degree(&self) -> usize {
        self.0.len()
    }

    fn times(&self, other: &Mono) -> Mono {
        let mut vars = self.0.clone();
        vars.extend_from_slice(&other.0);
        vars.sort_unstable();
        Mono(vars)
    }
}

/// A sum of monomials with nonzero coefficients.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct Poly(BTreeMap<Mono, i128>);

impl Poly {
    pub fn zero() -> Poly {
        Poly::default()
    }

    pub fn constant(c: i128) -> Poly {
        let mut terms = BTreeMap::new();
        if c != 0 {
            terms.insert(Mono::default(), c);
        }
        Poly(terms)
    }

    pub fn var(v: Var) -> Poly {
        Poly::mono(Mono(vec![v]), 1)
    }

    pub fn mono(m: Mono, c: i128) -> Poly {
        let mut terms = BTreeMap::new();
        if c != 0 {
            terms.insert(m, c);
        }
        Poly(terms)
    }

    /// The sum of the given terms; `None` when a coefficient overflows.
    pub fn from_terms(terms: impl IntoIterator<Item = (Mono, i128)>) -> Option<Poly> {
        let mut out: BTreeMap<Mono, i128> = BTreeMap::new();
        for (m, c) in terms {
            let slot = out.entry(m).or_insert(0);
            *slot = slot.checked_add(c)?;
        }
        out.retain(|_, c| *c != 0);
        Some(Poly(out))
    }

    pub fn coeff(&self, m: &Mono) -> i128 {
        self.0.get(m).copied().unwrap_or(0)
    }

    /// The degree of the largest monomial; 0 for a constant.
    pub fn degree(&self) -> usize {
        self.0.keys().map(Mono::degree).max().unwrap_or(0)
    }

    /// Whether `v` occurs in any monomial.
    pub fn mentions(&self, v: Var) -> bool {
        self.0.keys().any(|m| m.0.contains(&v))
    }

    pub fn terms(&self) -> impl Iterator<Item = (&Mono, i128)> {
        self.0.iter().map(|(m, c)| (m, *c))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    /// The value, when no variable occurs.
    pub fn as_const(&self) -> Option<i128> {
        match self.0.len() {
            0 => Some(0),
            1 => self.0.get(&Mono::default()).copied(),
            _ => None,
        }
    }

    /// The variable, when the polynomial is exactly one variable.
    pub fn as_var(&self) -> Option<Var> {
        if self.0.len() != 1 {
            return None;
        }
        let (m, c) = self.0.iter().next()?;
        (*c == 1 && m.0.len() == 1).then(|| m.0[0])
    }

    pub fn constant_term(&self) -> i128 {
        self.0.get(&Mono::default()).copied().unwrap_or(0)
    }

    pub fn is_linear(&self) -> bool {
        self.0.keys().all(|m| m.degree() <= 1)
    }

    pub fn vars(&self) -> BTreeSet<Var> {
        self.0.keys().flat_map(|m| m.0.iter().copied()).collect()
    }

    pub fn add(&self, other: &Poly) -> Option<Poly> {
        let mut terms = self.0.clone();
        for (m, c) in &other.0 {
            let sum = terms.get(m).copied().unwrap_or(0).checked_add(*c)?;
            if sum == 0 {
                terms.remove(m);
            } else {
                terms.insert(m.clone(), sum);
            }
        }
        Some(Poly(terms))
    }

    pub fn neg(&self) -> Option<Poly> {
        self.scale(-1)
    }

    pub fn sub(&self, other: &Poly) -> Option<Poly> {
        self.add(&other.neg()?)
    }

    pub fn add_const(&self, c: i128) -> Option<Poly> {
        self.add(&Poly::constant(c))
    }

    pub fn scale(&self, c: i128) -> Option<Poly> {
        if c == 0 {
            return Some(Poly::zero());
        }
        let mut terms = BTreeMap::new();
        for (m, k) in &self.0 {
            terms.insert(m.clone(), k.checked_mul(c)?);
        }
        Some(Poly(terms))
    }

    pub fn mul(&self, other: &Poly) -> Option<Poly> {
        let mut out = Poly::zero();
        for (m1, c1) in &self.0 {
            for (m2, c2) in &other.0 {
                out = out.add(&Poly::mono(m1.times(m2), c1.checked_mul(*c2)?))?;
            }
        }
        Some(out)
    }

    /// Replace every occurrence of `v` by `by`.
    pub fn substitute(&self, v: Var, by: &Poly) -> Option<Poly> {
        let mut out = Poly::zero();
        for (m, c) in &self.0 {
            let mut term = Poly::constant(*c);
            for &x in &m.0 {
                term = if x == v {
                    term.mul(by)?
                } else {
                    term.mul(&Poly::var(x))?
                };
            }
            out = out.add(&term)?;
        }
        Some(out)
    }

    /// The value under an assignment, or `None` when it leaves `i128`.
    pub fn eval(&self, value: &impl Fn(Var) -> i128) -> Option<i128> {
        let mut sum = 0i128;
        for (m, c) in &self.0 {
            let mut term = *c;
            for &v in &m.0 {
                term = term.checked_mul(value(v))?;
            }
            sum = sum.checked_add(term)?;
        }
        Some(sum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn products_are_normalised() {
        let (a, b, c) = (Poly::var(0), Poly::var(1), Poly::var(2));
        let left = a.mul(&b).unwrap().mul(&c).unwrap();
        let right = a.mul(&b.mul(&c).unwrap()).unwrap();
        assert_eq!(left, right);
        let square = a.add(&b).unwrap().mul(&a.add(&b).unwrap()).unwrap();
        let expanded = a
            .mul(&a)
            .unwrap()
            .add(&a.mul(&b).unwrap().scale(2).unwrap())
            .unwrap()
            .add(&b.mul(&b).unwrap())
            .unwrap();
        assert_eq!(square, expanded);
    }

    #[test]
    fn cancelling_terms_disappear() {
        let a = Poly::var(0);
        assert!(a.sub(&a).unwrap().is_zero());
        assert_eq!(a.add_const(3).unwrap().sub(&a).unwrap().as_const(), Some(3));
    }

    #[test]
    fn overflow_is_reported_not_wrapped() {
        let big = Poly::constant(i128::MAX);
        assert!(big.add(&Poly::constant(1)).is_none());
        assert!(big.mul(&Poly::constant(2)).is_none());
    }

    #[test]
    fn substitution_and_evaluation_agree() {
        // (x + 2y) with x := y * y
        let p = Poly::var(0).add(&Poly::var(1).scale(2).unwrap()).unwrap();
        let q = p
            .substitute(0, &Poly::var(1).mul(&Poly::var(1)).unwrap())
            .unwrap();
        assert_eq!(q.eval(&|_| 5), Some(35));
    }
}
