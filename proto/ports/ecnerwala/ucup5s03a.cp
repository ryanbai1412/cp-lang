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
use std.poly.{inv}

type Z = Mod<998_244_353>
let (n, k): (Int, Int) = read()
let comb = Comb<Z>()

fn ways(a: Int, b: Int) -> Z =
    if a > k: 0 else: comb.fact(k + b + 1) * comb.inv_fact(a + b + 1) * comb.inv_fact(k - a)

fn binomial_transform(ds: Int, dw: Int) -> List<Z>:
    let h = n / 2
    let egf = convolve(
        [ways(j + ds, j + dw) * comb.inv_fact(j) for j in 0..=h],
        [comb.inv_fact(j) for j in 0..=h],
    )
    [egf[i] * comb.fact(i) for i in 0..=h]

// [x^j] of x^shift f(x^2)
fn spread(f: List<Z>, shift: Int, j: Int) -> Z =
    if j >= shift and (j - shift) % 2 == 0: f[(j - shift) / 2] else: 0

let (t00, t01) = (binomial_transform(0, 0), binomial_transform(0, 1))
let (t10, t11) = (binomial_transform(1, 0), binomial_transform(1, 1))
let s = [Z(if j == 0: 1 else: 0) - spread(t00, 1, j) + spread(t01, 2, j) for j in 0..=n]
let a = [
    Z(
        if j == 0: 1 elif j == 1: -(k + 1) else: 0
    ) + spread(t10, 2, j) - spread(t11, 3, j) for j in 0..=n
]
print(convolve(s, inv(a, n + 1))[n] + n % 2)
