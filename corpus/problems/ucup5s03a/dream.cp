// Universal Cup 5 Stage 3 (Gorkovskaya) A.
//
// `ways(a, b)` counts the value sequences over 0..=k with `a` strict and
// `b` weak comparisons. For an extra strict step `ds` and an extra weak
// step `dw`, `T[ds, dw](i) = sum_j C(i, j) ways(j + ds, j + dw)` is the
// binomial transform, one convolution with e^x in EGF form. The answer is
//   [x^n] S(x) / A(x) + (n mod 2),  where
//   S(x) = 1 - x T[0,0](x^2) + x^2 T[0,1](x^2),
//   A(x) = 1 - (k + 1) x + x^2 T[1,0](x^2) - x^3 T[1,1](x^2).
use std.ntt.{convolve}
use std.poly.{binomial_transform, inv}

type Z = Mod<998_244_353>
let (n, k): (Int, Int) = read()
let comb = Comb<Z>()

fn ways(a: Int, b: Int) -> Z =
    if a > k: 0 else: comb.fact(k + b + 1) * comb.inv_fact(a + b + 1) * comb.inv_fact(k - a)

fn transformed(ds: Int, dw: Int) -> List<Z> = binomial_transform([ways(j + ds, j + dw) for j in 0..=n / 2])

// s += c * x^shift * t(x^2), modulo x^(n + 1)
fn add_spread(mut s: List<Z>, t: List<Z>, shift: Int, c: Z):
    for (i, v) in t.items if 2 * i + shift <= n: s[2 * i + shift] += c * v

var s = [if i == 0: Z(1) else: Z(0) for i in 0..=n]
add_spread(mut s, transformed(0, 0), 1, Z(-1))
add_spread(mut s, transformed(0, 1), 2, Z(1))
var a = [if i == 0: Z(1) elif i == 1: -Z(k + 1) else: Z(0) for i in 0..=n]
add_spread(mut a, transformed(1, 0), 2, Z(1))
add_spread(mut a, transformed(1, 1), 3, Z(-1))
print(convolve(s, inv(a, n + 1))[n] + n % 2)
