//! Closed forms that stand for a loop: each gives the loop's value, or
//! `None` where the loop's checked `Int` arithmetic would overflow.

/// `sum` of `lo, lo + by, ...` below `hi`, exact in 128 bits.
pub(crate) fn range_sum(lo: i64, hi: i64, by: i64) -> Option<i64> {
    if hi <= lo {
        return Some(0);
    }
    let span = (hi as u64).wrapping_sub(lo as u64).wrapping_sub(1);
    let n = i128::from(span / (by as u64)) + 1;
    let tri = if n & 1 == 0 {
        n / 2 * (n - 1)
    } else {
        (n - 1) / 2 * n
    };
    tri.checked_mul(i128::from(by))
        .and_then(|s| s.checked_add(n * i128::from(lo)))
        .and_then(|s| i64::try_from(s).ok())
        .filter(|s| *s != i64::MIN)
}
