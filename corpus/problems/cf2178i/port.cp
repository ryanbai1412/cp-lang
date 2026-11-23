// Source: Codeforces 2178I (Numbers or Fireworks), https://codeforces.com/contest/2178/problem/I
// Problem: n <= 31 distinct lattice points (cities) and k <= 2e4. For every proper subset T
//   (fireworks), take the product over cities outside T of 1 + (number of cities of T at
//   distance exactly sqrt(k)); print the sum over all T modulo 998244353. Multitest.
// Solution: expand each (f + 1): every outsider picks nothing or one firework neighbour. The
//   sqrt(k)-apart graph is bipartite (`two_color`); enumerate which cities `out` of the smaller
//   side A are outside T, then sweep the other side B with dp over subsets s of out (zeta
//   form) of the outsiders that have picked: b outside T has stay = |adj(b) - out| + 1 choices
//   in A and T; b in T may be picked by each outsider next to b, at most once each. Add dp[out],
//   subtract 1 for T = all cities. O(3^|A| * |B| * deg), |A| <= 15.
use std.graph.{two_color}
type Z = Mod<998_244_353>

@!multitest
let (n, k): (2..=31, 1..=20_000) = read()
domain V = 0..n
let p: [V]Pt = read()
fn near(u: V, v: V) -> Bool = (p[u] - p[v]).norm2 == k
let side = two_color<V>(u => [v for v in V if near(u, v)])!
let left = V.count(v => side[v]) * 2 <= n
let a = [v for v in V if side[v] == left]
let b = [v for v in V if side[v] != left]
domain A = 0..a.len
fn nbr(v: V) -> Subset<A>:
    var m = Subset<A>.empty
    for i in A if near(a[i.ord], v): m = m.with(i)
    return m
let adj = [nbr(v) for v in b]

var dp: [Subset<A>]Z = 0
var skip: [Subset<A>]Z = 0
var ans: Z = 0
for out in Subset<A>.full.subsets:
    for s in out.subsets: dp[s] = 1
    for m in adj:
        let stay = Z((m - out).len + 1)
        for s in out.subsets: skip[s] = dp[s] * stay
        for i in m & out:
            for s in out.without(i).subsets: dp[s.with(i)] += dp[s]
        for s in out.subsets: dp[s] += skip[s]
    ans += dp[out]
print(ans - 1)
