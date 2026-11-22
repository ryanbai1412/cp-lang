// Universal Cup 5 Stage 3 (Gorkovskaya) A.
//
// `ways(a, b)` counts the value sequences over 0..=k with `a` strict and
// `b` weak comparisons. For an extra strict step `ds` and an extra weak
// step `dw`, `T[ds, dw](i) = sum_j C(i, j) ways(j + ds, j + dw)` is the
// binomial transform, one convolution with e^x in EGF form. The answer is
//   [x^n] S(x) / A(x) + (n mod 2),  where
//   S(x) = 1 - x T[0,0](x^2) + x^2 T[0,1](x^2),
//   A(x) = 1 - (k + 1) x + x^2 T[1,0](x^2) - x^3 T[1,1](x^2).
use std.poly.{Fps, X}

type Z = Mod<998_244_353>
let (n, k): (Int, Int) = read()
let comb = Comb<Z>()

fn ways(a: Int, b: Int) -> Z =
    if a > k: 0 else: comb.fact(k + b + 1) * comb.inv_fact(a + b + 1) * comb.inv_fact(k - a)

fn binomial_transform(ds: Int, dw: Int) -> Fps<Z>:
    let h = n / 2
    Fps.egf([ways(j + ds, j + dw) for j in 0..=h]).binomial_transform()

let (t00, t01) = (binomial_transform(0, 0), binomial_transform(0, 1))
let (t10, t11) = (binomial_transform(1, 0), binomial_transform(1, 1))
let s = 1 - X * t00.of(X ** 2) + X ** 2 * t01.of(X ** 2)
let a = 1 - (k + 1) * X + X ** 2 * t10.of(X ** 2) - X ** 3 * t11.of(X ** 2)
print((s / a)[n] + n % 2)
