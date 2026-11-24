// Source: Codeforces 2268E, Kia Kio and Tree of Life,
//   https://codeforces.com/contest/2268/problem/E
// Problem: multitest, sum n <= 2e5, a_i < 2^18. Over all Catalan(n) binary trees with in-order
//   a, sum f(tree): over the n - 1 edges, the XOR of one side plus the XOR of the other
//   side when the edge is cut. Print it modulo 998244353.
// Solution: bit by bit, with s_i = +-1 the prefix parity: an odd total gives every edge 1; an
//   even one gives 1 - s_i s_j for a subtree covering (i, j], and a gap d is a subtree in
//   C_d C_(n-d) trees. So subtract sum_d C_d C_(n-d) corr(d), the autocorrelation of s by
//   NTT. O(18 n log n).

const MOD = 998_244_353
const BITS = 18
const MAX_X = (1 << BITS) - 1
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let xs: [1..=n]0..=MAX_X = read()
let weight = [0..n] d => if d == 0: Z(0) else: comb.catalan(d) * comb.catalan(n - d)
var ans: Z = 0
for b in 0..BITS:
    var a: [0..=n]Z = 1
    for i in 1..=n: a[i] = if xs[i].bit(b): -a[i - 1] else: a[i - 1]
    var cur = comb.catalan(n) * (n - 1)
    if a[n] == 1:
        let corr = convolve(a, [a[n - i] for i in 0..=n])
        for d in 1..n: cur -= weight[d] * corr[n - d]
    ans += cur * (1 << b)
print(ans)
