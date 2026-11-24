// Source: The 5th Universal Cup, Stage 5 (Grand Prix of Chengdu), problem F
// limits: 5000 ms, 1024 MB
// Problem: multitest. A string s of n <= 2e5 lowercase letters. For a distinct substring u let
//   d(u) be the number of distinct substrings of s that start with u. For every k in 1..=n print
//   the sum over distinct substrings u of choose(d(u), k), modulo 998244353.
// Solution: substrings are the points of the suffix tree, and along an edge d runs from
//   top_size + 1 to bottom_size, so its choose(d, k) telescope to choose(bottom_size, k) -
//   choose(top_size, k). Sizes reach n^2, so keep only the power sums p[j] = sum bottom^j - top^j
//   for j <= n (a size of 0 adds nothing for k >= 1), move them onto weights w at the points
//   0..=n with the same power sums (transposed interpolation), and read
//   sum_k answer[k] y^k = sum_m w[m] (1 + y)^m off a Taylor shift; O(n log^2 n).
// Bounds: n: 1..=2e5
use std.poly.{eval_all, power_sums, prod_linear, taylor_shift}
use std.strings.{suffix_tree_edges}

type Z = Mod<998_244_353>
let comb = Comb<Z>()

// sum_s (1 + y)^s mod y^p.len for the multiset whose power sums are p. With q = prod(x - m) over
// the points m in 0..p.len, partial fractions of sum_m w[m] / (x - m) = sum_j p[j] / x^(j + 1)
// give w[m] = num(m) / q'(m), num the polynomial part of q(x) * sum_j p[j] / x^(j + 1).
fn binomial_sums(p: List<Z>) -> List<Z>:
    let deg = p.len - 1
    let points = [Z(m) for m in 0..=deg]
    let q = prod_linear(points)
    let num = convolve(q, p.rev.to_list())[deg + 1..].to_list()
    let at = eval_all(num, points)
    let w = [
        at[m] * (-Z(1)) ** (deg - m) * comb.inv_fact(m) * comb.inv_fact(deg - m) for m in 0..=deg
    ]
    taylor_shift(w, Z(1))

@!multitest
let n: 1..=2e5 = read()
let s: Str of "abcdefghijklmnopqrstuvwxyz" = read()
let edges = suffix_tree_edges(s)
let bottom = power_sums([Z(e.bottom_size) for e in edges], n)
let top = power_sums([Z(e.top_size) for e in edges if e.top_size > 0], n)
let answer = binomial_sums([bottom[j] - top[j] for j in 0..=n])
print(answer[1..=n])
