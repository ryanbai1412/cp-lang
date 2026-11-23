// Source: The 5th Universal Cup, Stage 3 (Grand Prix of Gorkovskaya), problem A (Adjacent
//   Decrements), qoj.ac/contest/4565/problem/20729
// Problem: an operation picks adjacent a_i, a_{i+1}: if equal, both drop by 1, else the smaller
//   drops by 1, never below 0. Count arrays of length n with 0 <= a_i <= k that can reach all
//   zeros, modulo 998244353 (n, k <= 2.5e5).
// Solution: generating functions: the answer is [x^n] S(x) / A(x) + (n mod 2), with S and A
//   built from binomial transforms T[ds, dw](i) = sum_j C(i, j) ways(j + ds, j + dw), where
//   ways(a, b) counts value sequences over 0..k with a strict and b weak comparisons, a closed
//   binomial. Each transform is one convolution with e^x in EGF form, then one power series
//   inverse: O(n log n).
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
