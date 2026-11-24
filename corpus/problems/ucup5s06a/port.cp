// Source: The 5th Universal Cup, Stage 6 (Grand Prix of Hyakumanben), problem A (Fugaku
//   NK-kei), qoj.ac/contest/4617/problem/23669
// limits: 2000 ms, 1024 MB
// Problem: n regions on a circle (2 <= n <= 2e5) and a painting for every arc of d <= k < n
//   regions; each arc is collected at the first of its regions visited. Over all visit orders,
//   count the distinct vectors (C_1..C_n) of paintings collected per region, mod 998244353.
//   Multitest, sum of n <= 2e5.
// Solution: per the editorial, with B(x) = sum of Catalan(i - 1) x^i over 1 <= i < k, the answer
//   is cyc(B + x) - 2 cyc(B), where cyc(B) = [x^n] x B'(x) / (1 - B(x)) counts the ways to cut the
//   n-cycle into blocks, a block of size i weighted [x^i] B. cyc inverts 1 - B as a power series
//   and sums i b_i g_(n-i): O(n log n) per test. k = 1 leaves B empty and the answer cyc(x) = 1.
use std.poly.{inv}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

@!multitest
let n: 2..=2e5 = read()
let k: 1..n = read()

fn cyc(b: List<Z>) -> Z:
    let g = inv([Z(Int(i == 0)) - b[i] for i in 0..=n], n + 1)
    sum(Z(i) * b[i] * g[n - i] for i in 1..=n)

let b = [if i in 1..k: comb.catalan(i - 1) else: Z(0) for i in 0..=n]
let b_or_single = [c + Z(Int(i == 1)) for (i, c) in b.items]
print(cyc(b_or_single) - Z(2) * cyc(b))
