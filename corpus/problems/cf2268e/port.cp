// Source: Codeforces 2268E, Kia Kio and Tree of Life,
//   https://codeforces.com/contest/2268/problem/E
// limits: 3000 ms, 256 MB
// Problem: multitest, sum n <= 2e5, a_i < 2^18. Over all Catalan(n) binary trees with in-order
//   a, sum f(tree): over the n - 1 edges, the XOR of one side plus the XOR of the other
//   side when the edge is cut. Print it modulo 998244353.
// Solution: bit by bit, with s_i = +-1 the prefix parity: an odd total gives every edge 1; an
//   even one gives 1 - s_i s_j for a subtree covering (i, j], and a gap d is a subtree in
//   C_d C_(n-d) trees. So subtract sum_d C_d C_(n-d) corr(d), the autocorrelation of s, as
//   one NTT per bit. O(18 n log n).

use std.ntt.{ntt}

const MOD = 998_244_353
const BITS = 18
const MAX_X = (1 << BITS) - 1
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let xs: [1..=n]0..=MAX_X = read()
// The dot product sum_d weight[d] corr[n - d] in the frequency domain: with A = ntt(a) and
//   W = ntt(weight), it is sum_f A[f] A[-f] W[f] / L, one transform per bit.
let size = bit_ceil(2 * n + 1)
var w = List.filled(size, Z(0))
for d in 1..n: w[d] = comb.catalan(d) * comb.catalan(n - d)
ntt(mut w)
let scale = Z(1) / size
var ans: Z = 0
for b in 0..BITS:
    var a = List.filled(size, Z(0))
    a[0] = 1
    for i in 1..=n: a[i] = if xs[i].bit(b): -a[i - 1] else: a[i - 1]
    var cur = comb.catalan(n) * (n - 1)
    if a[n] == 1:
        ntt(mut a)
        cur -= scale * sum(a[f] * a[(size - f) % size] * w[f] for f in 0..size)
    ans += cur * (1 << b)
print(ans)
