//! The core logic: polynomial constraints joined by `and` and `or`.
//!
//! Every surface term is lowered to this form. Negation is pushed to the
//! atoms as the formula is built, so the solver only ever sees `p >= 0`
//! and `p == 0`.

use crate::poly::{Poly, Var};
use crate::term::Error;

pub(crate) type Res<T> = Result<T, Error>;

/// Turn an overflowing polynomial operation into an error the caller
/// reports as "unknown".
pub(crate) fn fit(p: Option<Poly>) -> Res<Poly> {
    p.ok_or_else(|| Error::new("a number in the proof left the 128-bit range"))
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub(crate) enum Formula {
    True,
    False,
    /// `p >= 0`.
    Ge(Poly),
    /// `p == 0`.
    Eq(Poly),
    And(Vec<Formula>),
    Or(Vec<Formula>),
}

impl Formula {
    pub fn ge(p: Poly) -> Formula {
        match p.as_const() {
            Some(c) if c >= 0 => Formula::True,
            Some(_) => Formula::False,
            None => Formula::Ge(p),
        }
    }

    pub fn eq(p: Poly) -> Formula {
        match p.as_const() {
            Some(0) => Formula::True,
            Some(_) => Formula::False,
            None => Formula::Eq(p),
        }
    }

    /// `a <= b`.
    pub fn le(a: &Poly, b: &Poly) -> Res<Formula> {
        Ok(Formula::ge(fit(b.sub(a))?))
    }

    /// `a < b`.
    pub fn lt(a: &Poly, b: &Poly) -> Res<Formula> {
        Ok(Formula::ge(fit(b.sub(a).and_then(|d| d.add_const(-1)))?))
    }

    /// `a == b`.
    pub fn same(a: &Poly, b: &Poly) -> Res<Formula> {
        Ok(Formula::eq(fit(a.sub(b))?))
    }

    pub fn and(parts: impl IntoIterator<Item = Formula>) -> Formula {
        let mut out = Vec::new();
        for part in parts {
            match part {
                Formula::True => {}
                Formula::False => return Formula::False,
                Formula::And(inner) => out.extend(inner),
                other => out.push(other),
            }
        }
        match out.len() {
            0 => Formula::True,
            1 => out.pop().unwrap(),
            _ => Formula::And(out),
        }
    }

    pub fn or(parts: impl IntoIterator<Item = Formula>) -> Formula {
        let mut out = Vec::new();
        for part in parts {
            match part {
                Formula::False => {}
                Formula::True => return Formula::True,
                Formula::Or(inner) => out.extend(inner),
                other => out.push(other),
            }
        }
        match out.len() {
            0 => Formula::False,
            1 => out.pop().unwrap(),
            _ => Formula::Or(out),
        }
    }

    pub fn negate(&self) -> Res<Formula> {
        Ok(match self {
            Formula::True => Formula::False,
            Formula::False => Formula::True,
            // not (p >= 0) is -p - 1 >= 0.
            Formula::Ge(p) => Formula::ge(fit(p.neg().and_then(|n| n.add_const(-1)))?),
            // not (p == 0) is p >= 1 or p <= -1.
            Formula::Eq(p) => Formula::or([
                Formula::ge(fit(p.add_const(-1))?),
                Formula::ge(fit(p.neg().and_then(|n| n.add_const(-1)))?),
            ]),
            Formula::And(parts) => {
                Formula::or(parts.iter().map(Formula::negate).collect::<Res<Vec<_>>>()?)
            }
            Formula::Or(parts) => {
                Formula::and(parts.iter().map(Formula::negate).collect::<Res<Vec<_>>>()?)
            }
        })
    }

    pub fn collect_vars(&self, into: &mut std::collections::BTreeSet<Var>) {
        match self {
            Formula::True | Formula::False => {}
            Formula::Ge(p) | Formula::Eq(p) => into.extend(p.vars()),
            Formula::And(parts) | Formula::Or(parts) => {
                for part in parts {
                    part.collect_vars(into);
                }
            }
        }
    }

    pub fn mentions(&self, v: Var) -> bool {
        match self {
            Formula::True | Formula::False => false,
            Formula::Ge(p) | Formula::Eq(p) => p.mentions(v),
            Formula::And(parts) | Formula::Or(parts) => parts.iter().any(|part| part.mentions(v)),
        }
    }

    /// Replace `v` by `by`; `None` on overflow.
    pub fn substitute(&self, v: Var, by: &Poly) -> Option<Formula> {
        if !self.mentions(v) {
            return Some(self.clone());
        }
        Some(match self {
            Formula::True | Formula::False => self.clone(),
            Formula::Ge(p) => Formula::ge(p.substitute(v, by)?),
            Formula::Eq(p) => Formula::eq(p.substitute(v, by)?),
            Formula::And(parts) => Formula::and(
                parts
                    .iter()
                    .map(|part| part.substitute(v, by))
                    .collect::<Option<Vec<_>>>()?,
            ),
            Formula::Or(parts) => Formula::or(
                parts
                    .iter()
                    .map(|part| part.substitute(v, by))
                    .collect::<Option<Vec<_>>>()?,
            ),
        })
    }

    /// The truth of the formula under an assignment; `None` on overflow.
    pub fn eval(&self, value: &impl Fn(Var) -> i128) -> Option<bool> {
        Some(match self {
            Formula::True => true,
            Formula::False => false,
            Formula::Ge(p) => p.eval(value)? >= 0,
            Formula::Eq(p) => p.eval(value)? == 0,
            Formula::And(parts) => {
                for part in parts {
                    if !part.eval(value)? {
                        return Some(false);
                    }
                }
                true
            }
            Formula::Or(parts) => {
                for part in parts {
                    if part.eval(value)? {
                        return Some(true);
                    }
                }
                false
            }
        })
    }
}

/// A constraint the solver takes up only once it matters: the definition
/// of a derived variable (an `if`, a `max`, a quotient, an array element)
/// is irrelevant until something mentions that variable.
#[derive(Clone, Debug)]
pub(crate) struct Lazy {
    /// The constraint wakes up when every group has a variable in use.
    pub triggers: Vec<Vec<Var>>,
    pub formula: Formula,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_fold_and_connectives_flatten() {
        let x = Poly::var(0);
        assert_eq!(Formula::ge(Poly::constant(0)), Formula::True);
        assert_eq!(Formula::eq(Poly::constant(2)), Formula::False);
        let atom = Formula::ge(x.clone());
        assert_eq!(Formula::and([Formula::True, atom.clone()]), atom);
        assert_eq!(
            Formula::or([Formula::False, atom.clone(), Formula::True]),
            Formula::True
        );
        let nested = Formula::and([atom.clone(), Formula::and([atom.clone(), Formula::eq(x)])]);
        assert!(matches!(nested, Formula::And(parts) if parts.len() == 3));
    }

    #[test]
    fn negation_is_exact_over_the_integers() {
        let x = Poly::var(0);
        let formula = Formula::or([
            Formula::and([
                Formula::ge(x.clone()),
                Formula::lt(&x, &Poly::constant(3)).unwrap(),
            ]),
            Formula::same(&x, &Poly::constant(7)).unwrap(),
        ]);
        let negated = formula.negate().unwrap();
        for v in -5..12 {
            let holds = formula.eval(&|_| v).unwrap();
            assert_eq!(holds, (0..3).contains(&v) || v == 7);
            assert_eq!(negated.eval(&|_| v).unwrap(), !holds);
        }
    }
}
