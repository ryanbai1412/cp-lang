//! Integer intervals with open ends.
//!
//! The cheapest layer of the prover: every variable and every product gets
//! a range, and a constraint that a range already decides never reaches
//! the integer solver. An end that leaves `i128` becomes open, which is
//! always sound.

use std::fmt;

/// `lo..=hi`; `None` is an open end.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Interval {
    pub lo: Option<i128>,
    pub hi: Option<i128>,
}

/// An end point: a number or one of the two infinities.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum End {
    NegInf,
    Fin(i128),
    PosInf,
}

impl End {
    fn times(self, other: End) -> End {
        use End::*;
        match (self, other) {
            (Fin(0), _) | (_, Fin(0)) => Fin(0),
            (Fin(a), Fin(b)) => match a.checked_mul(b) {
                Some(p) => Fin(p),
                None if (a < 0) == (b < 0) => PosInf,
                None => NegInf,
            },
            (a, b) => {
                let negative = |e: End| matches!(e, NegInf) || matches!(e, Fin(x) if x < 0);
                if negative(a) == negative(b) {
                    PosInf
                } else {
                    NegInf
                }
            }
        }
    }
}

impl Interval {
    pub const ALL: Interval = Interval { lo: None, hi: None };

    pub fn new(lo: i128, hi: i128) -> Interval {
        Interval {
            lo: Some(lo),
            hi: Some(hi),
        }
    }

    pub fn point(v: i128) -> Interval {
        Interval::new(v, v)
    }

    pub fn is_empty(&self) -> bool {
        matches!((self.lo, self.hi), (Some(lo), Some(hi)) if lo > hi)
    }

    pub fn contains(&self, v: i128) -> bool {
        self.lo.is_none_or(|lo| lo <= v) && self.hi.is_none_or(|hi| v <= hi)
    }

    /// Whether every value of `self` is a value of `other`.
    pub fn within(&self, other: &Interval) -> bool {
        let lo_ok = match (self.lo, other.lo) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => a >= b,
        };
        let hi_ok = match (self.hi, other.hi) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => a <= b,
        };
        lo_ok && hi_ok
    }

    pub fn add(&self, other: &Interval) -> Interval {
        Interval {
            lo: self.lo.zip(other.lo).and_then(|(a, b)| a.checked_add(b)),
            hi: self.hi.zip(other.hi).and_then(|(a, b)| a.checked_add(b)),
        }
    }

    pub fn neg(&self) -> Interval {
        Interval {
            lo: self.hi.and_then(i128::checked_neg),
            hi: self.lo.and_then(i128::checked_neg),
        }
    }

    pub fn scale(&self, c: i128) -> Interval {
        self.mul(&Interval::point(c))
    }

    pub fn mul(&self, other: &Interval) -> Interval {
        let ends = |i: &Interval| {
            (
                i.lo.map_or(End::NegInf, End::Fin),
                i.hi.map_or(End::PosInf, End::Fin),
            )
        };
        let ((a, b), (c, d)) = (ends(self), ends(other));
        let products = [a.times(c), a.times(d), b.times(c), b.times(d)];
        let fin = |e: End| match e {
            End::Fin(v) => Some(v),
            _ => None,
        };
        Interval {
            lo: fin(*products.iter().min().unwrap()),
            hi: fin(*products.iter().max().unwrap()),
        }
    }

    /// The square of a value in `self`: never negative.
    pub fn square(&self) -> Interval {
        let product = self.mul(self);
        let lo = match (self.lo, self.hi) {
            (Some(lo), _) if lo >= 0 => lo.checked_mul(lo),
            (_, Some(hi)) if hi <= 0 => hi.checked_mul(hi),
            _ => Some(0),
        };
        Interval {
            lo: lo.or(Some(0)),
            hi: product.hi,
        }
    }

    /// The smallest interval holding both.
    pub fn hull(&self, other: &Interval) -> Interval {
        Interval {
            lo: self.lo.zip(other.lo).map(|(a, b)| a.min(b)),
            hi: self.hi.zip(other.hi).map(|(a, b)| a.max(b)),
        }
    }

    /// The values in both.
    pub fn meet(&self, other: &Interval) -> Interval {
        let lo = match (self.lo, other.lo) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        };
        let hi = match (self.hi, other.hi) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        Interval { lo, hi }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.lo, self.hi) {
            (Some(lo), Some(hi)) => write!(f, "{lo}..={hi}"),
            (Some(lo), None) => write!(f, "{lo}.."),
            (None, Some(hi)) => write!(f, "..={hi}"),
            (None, None) => write!(f, ".."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn products_take_the_extreme_corners() {
        let i = Interval::new(-3, 4).mul(&Interval::new(-5, 2));
        assert_eq!(i, Interval::new(-20, 15));
        assert_eq!(
            Interval::new(0, 1_000_000_000)
                .mul(&Interval::new(0, 1_000_000_000))
                .hi,
            Some(10i128.pow(18))
        );
    }

    #[test]
    fn open_ends_stay_sound() {
        let nonneg = Interval {
            lo: Some(0),
            hi: None,
        };
        assert_eq!(nonneg.mul(&nonneg), nonneg);
        assert_eq!(
            nonneg.mul(&Interval::new(-2, -1)),
            Interval {
                lo: None,
                hi: Some(0)
            }
        );
        assert_eq!(Interval::ALL.mul(&Interval::point(0)), Interval::point(0));
        let huge = Interval::point(i128::MAX);
        assert_eq!(huge.mul(&huge), Interval { lo: None, hi: None });
    }

    #[test]
    fn squares_are_never_negative() {
        assert_eq!(Interval::new(-3, 2).square(), Interval::new(0, 9));
        assert_eq!(
            Interval::ALL.square(),
            Interval {
                lo: Some(0),
                hi: None
            }
        );
        assert_eq!(Interval::new(2, 5).square(), Interval::new(4, 25));
    }

    #[test]
    fn hull_meet_and_containment() {
        let (a, b) = (Interval::new(0, 5), Interval::new(3, 9));
        assert_eq!(a.hull(&b), Interval::new(0, 9));
        assert_eq!(a.meet(&b), Interval::new(3, 5));
        assert!(Interval::new(6, 5).is_empty());
        assert!(a.meet(&b).within(&a) && !a.within(&b));
        assert_eq!(a.to_string(), "0..=5");
    }
}
