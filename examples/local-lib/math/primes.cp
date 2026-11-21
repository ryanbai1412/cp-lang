//! Divisors of large 64-bit numbers, from `std.math.factor`.
use std.math.{factor}

/// Every divisor of `n >= 1`, ascending. O(number of divisors) after `factor`.
pub fn divisors_of(n: Int) -> List<Int>:
    var ds = [1]
    for (p, e) in factor(n):
        let base = [d for d in ds]
        var scale = 1
        for _ in 0..e:
            scale *= p
            for d in base: ds.push(d * scale)
    ds.sort()
    ds

@test
fn agrees_with_trial_division():
    for n in 1..400: assert divisors_of(n) == [d for d in 1..=n if n % d == 0]
    assert divisors_of(2 ** 61).len == 62
    assert divisors_of(999_983 * 999_983 * 1_000_003).len == 6
