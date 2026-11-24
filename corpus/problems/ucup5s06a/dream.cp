// Universal Cup 5 Stage 6 (Hyakumanben) A.
//
// For a block series B(x) with B(0) = 0, cyc(B) = [x^n] x B'(x) / (1 - B(x))
// = sum over j of n [x^n] B^j / j counts the ways to cut a cycle of n
// labelled positions into blocks, a block of size i weighted by [x^i] B. With
// B = sum of Catalan(i - 1) x^i over 1 <= i < k, the editorial's answer is
// cyc(B + x) - 2 cyc(B). For k = 1, B is empty and the answer is cyc(x) = 1.
use std.poly.{deriv, inv}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let (n, k): (Int, 1..n) = read()
fn cyc(b: List<Z>) -> Z:
    let one_minus_b = [if i == 0: 1 - b[i] else: -b[i] for i in b.keys]
    convolve(deriv(b), inv(one_minus_b, n))[n - 1]   // [x^n] x B' = [x^(n-1)] B'
let b = [if i in 1..k: comb.catalan(i - 1) else: Z(0) for i in 0..=n]
let b_plus_x = [if i == 1: b[i] + 1 else: b[i] for i in b.keys]
print(cyc(b_plus_x) - 2 * cyc(b))
