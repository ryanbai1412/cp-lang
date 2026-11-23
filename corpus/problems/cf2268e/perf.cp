// Source: Codeforces 2268E, Kia Kio and Tree of Life,
//   https://codeforces.com/contest/2268/problem/E
// Problem: multitest, sum n <= 2e5, a_i < 2^18. Over all Catalan(n) binary trees with in-order
//   a, sum f(tree): over the n - 1 edges, the XOR of one side plus the XOR of the other
//   side when the edge is cut. Print it modulo 998244353.
// Solution: bit by bit, with s_i = +-1 the prefix parity: an odd total gives every edge 1; an
//   even one gives 1 - s_i s_j for a subtree covering (i, j], and a gap d is a subtree in
//   C_d C_(n-d) trees. So subtract sum_d C_d C_(n-d) corr(d), the autocorrelation of s by
//   NTT. O(18 n log n).
// Tuned: the reference's one forward NTT per bit. `rev(a)` is `a` read backwards, so its
// transform is `a`'s at `-j`, and coefficient `n` of `w * a * rev(a)` is
// `sum_j W_j A_j A_(-j) / L`: no second transform and no inverse.
use std.ntt.{ntt}

const MOD = 998_244_353
const BITS = 18
const MAX_X = (1 << BITS) - 1
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let xs: [1..=n]0..=MAX_X = read()
var size = 1
while size < 2 * n: size *= 2
var tw = List.filled(size, Z(0))
for d in 1..n: tw[d] = comb.catalan(d) * comb.catalan(n - d)
ntt(mut tw)
let inv_size = Z(size).inv()
var ans: Z = 0
for b in 0..BITS:
    var fa = List.filled(size, Z(0))
    var s: Z = 1
    fa[0] = s
    for i in 1..=n:
        if xs[i].bit(b): s = -s
        fa[i] = s
    var cur = comb.catalan(n) * (n - 1)
    if s == 1:
        ntt(mut fa)
        var dot = tw[0] * fa[0] * fa[0]
        for j in 1..size: dot += tw[j] * fa[j] * fa[size - j]
        cur -= dot * inv_size
    ans += cur * (1 << b)
print(ans)
