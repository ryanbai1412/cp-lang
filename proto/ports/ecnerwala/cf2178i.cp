// Codeforces 2178 I: Numbers or Fireworks.
// Expand each (f + 1): every city outside T picks nothing or one firework
// neighbour. Sqrt(k)-apart cities form a bipartite graph; enumerate which
// cities of the smaller side O are outside T, then sweep the other side,
// tracking which of O have picked a firework (each picks at most one).
use std.graph.{two_color}
type Z = Mod<998_244_353>

@!multitest
let (n, k): (2..=31, 1..=20_000) = read()
domain V = 0..n
let p: [V]Pt = read()
fn near(u: V, v: V) -> Bool = (p[u] - p[v]).norm2 == k
let side = two_color::<V>(u => [v for v in V if near(u, v)])!
let left = count(side[v] for v in V) * 2 <= n
let a = [v for v in V if side[v] == left]
let b = [v for v in V if side[v] != left]
domain A = 0..a.len
fn nbr(v: V) -> Subset<A>:
    var m = Subset<A>.empty
    for i in A if near(a[i.to_idx], v): m = m.with(i)
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
