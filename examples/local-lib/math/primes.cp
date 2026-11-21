//! Divisors of numbers too large for trial division, from `std.math.factor_big`.
use std.math.{factor_big}

/// Every divisor of `n >= 1`, ascending. O(number of divisors) after `factor_big`.
pub fn divisors_of(n: Int) -> List<Int>:
    var ds = [1]
    for (p, e) in factor_big(n):
        let base = [d for d in ds]
        var scale = 1
        for _ in 0..e:
            scale *= p
            for d in base: ds.push(d * scale)
    sort(mut ds)
    ds

@test
fn agrees_with_trial_division():
    for n in 1..400: assert divisors_of(n) == [d for d in 1..=n if n % d == 0]
    assert divisors_of(2 ** 61).len == 62
    assert divisors_of(999_983 * 999_983 * 1_000_003).len == 6
