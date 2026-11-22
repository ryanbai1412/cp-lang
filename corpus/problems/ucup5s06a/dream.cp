// Universal Cup 5 Stage 6 (Hyakumanben) A.
//
// For a block series B(x) with B(0) = 0, cyc(B) = [x^n] x B'(x) / (1 - B(x))
// = sum over j of n [x^n] B^j / j counts the ways to cut a cycle of n
// labelled positions into blocks, a block of size i weighted by [x^i] B. With
// B = sum of Catalan(i - 1) x^i over 1 <= i < k, the editorial's answer is
// cyc(B + x) - 2 cyc(B). For k = 1, B is empty and the answer is cyc(x) = 1.
use std.poly.{Series}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let (n, k): (Int, 1..n) = read()
fn cyc(b: Series<Z>) -> Z = (Series.x * b.deriv / (1 - b))[n]
let b = Series<Z>(len: n + 1, i => if i in 1..k: comb.catalan(i - 1) else: 0)
print(cyc(b + Series.x) - 2 * cyc(b))
