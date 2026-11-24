//! Interval arithmetic on inclusive `(lo, hi)` integer ranges: the one
//! table lint's bounds and codegen's range facts both combine with.

use cpl_syntax::ast::BinOp;

/// The values `a op b` can take for `a`, `b` in their ranges, as the
/// interpreter computes them (`/` and `%` Euclidean, by a positive
/// divisor; `**` by a constant); `None` when this cannot say.
pub fn combine(op: BinOp, a: (i128, i128), b: (i128, i128)) -> Option<(i128, i128)> {
    let hull = |c: [i128; 4]| Some((*c.iter().min()?, *c.iter().max()?));
    match op {
        BinOp::Add => Some((a.0.checked_add(b.0)?, a.1.checked_add(b.1)?)),
        BinOp::Sub => Some((a.0.checked_sub(b.1)?, a.1.checked_sub(b.0)?)),
        BinOp::Mul => hull([
            a.0.checked_mul(b.0)?,
            a.0.checked_mul(b.1)?,
            a.1.checked_mul(b.0)?,
            a.1.checked_mul(b.1)?,
        ]),
        BinOp::Div if b.0 > 0 => hull([
            a.0.checked_div_euclid(b.0)?,
            a.0.checked_div_euclid(b.1)?,
            a.1.checked_div_euclid(b.0)?,
            a.1.checked_div_euclid(b.1)?,
        ]),
        BinOp::Rem if a.0 == a.1 && b.0 == b.1 && b.0 != 0 => {
            Some((a.0.rem_euclid(b.0), a.0.rem_euclid(b.0)))
        }
        BinOp::Rem if b.0 > 0 => Some(if a.0 >= 0 && a.1 < b.0 {
            a
        } else {
            (0, if a.0 >= 0 { a.1.min(b.1 - 1) } else { b.1 - 1 })
        }),
        BinOp::Pow if b.0 == b.1 && b.0 >= 0 => {
            let k = u32::try_from(b.0).ok()?;
            let (lo, hi) = (a.0.checked_pow(k)?, a.1.checked_pow(k)?);
            Some(if k % 2 == 1 || a.0 >= 0 {
                (lo, hi)
            } else if a.1 <= 0 {
                (hi, lo)
            } else {
                (0, lo.max(hi))
            })
        }
        _ => None,
    }
}

/// Whether every value in `lo..=hi` is an `Int`: `Int.MIN` is the niche of
/// `?Int`, so arithmetic that reaches it traps like any overflow (R0202).
pub fn in_int((lo, hi): (i128, i128)) -> bool {
    lo > i128::from(i64::MIN) && hi <= i128::from(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_by_corners_and_keeps_int_min_out_of_int() {
        assert_eq!(combine(BinOp::Mul, (-3, 2), (-5, 4)), Some((-12, 15)));
        assert_eq!(combine(BinOp::Sub, (0, 10), (1, 3)), Some((-3, 9)));
        assert_eq!(combine(BinOp::Rem, (-7, 7), (1, 4)), Some((0, 3)));
        assert_eq!(combine(BinOp::Rem, (-7, -7), (-3, -3)), Some((2, 2)));
        assert_eq!(combine(BinOp::Div, (0, 7), (0, 4)), None);
        let max = i128::from(i64::MAX);
        assert!(in_int((-max, max)));
        assert!(!in_int((-max - 1, 0)));
        assert!(!in_int((0, max + 1)));
    }
}
