// Codeforces 2178 I: Numbers or Fireworks (dream).
// Expand each (f + 1): every city outside T picks nothing or one firework
// neighbour. Sqrt(k)-apart lattice cities form a bipartite graph (parity);
// enumerate which cities of the smaller side are outside T, then sweep the
// other side, tracking which of those have picked a firework.
// A city b of the other side outside T picks nothing or one neighbour in T:
// `stay` = |adj(b) - out| + 1 ways. With b in T, any outsider next to b may
// pick it, each outsider at most once: dp[s] counts the ways in which exactly
// the outsiders in s have picked, and the zeta over adj(b) & out adds b's picks.
type Z = Mod<998_244_353>

@!multitest
let (n, k): (2..=31, 1..=2e4) = read()
domain V = 0..n
let p: [V]Pt = read()
fn near(u: V, v: V) -> Bool = (p[u] - p[v]).norm2 == k
let side = two_color<V>(u => [v for v in V if near(u, v)])!
let small = V.count(v => side[v]) * 2 <= n
let a = [v for v in V if side[v] == small]
let b = [v for v in V if side[v] != small]
domain A = 0..a.len
fn nbr(v: V) -> Subset<A>:
    var m = Subset<A>.empty
    for i in A if near(a[i.ord], v): m = m.with(i)
    m
let adj = [nbr(v) for v in b]

// Fix the outsiders `out`; each b in turn stays put or is picked by one outsider next to it.
fn ways(out: Subset<A>) -> Z:
    var dp: [Subset<A>]Z = 0
    for s in out.subsets: dp[s] = 1
    for m in adj:
        let stay = Z((m - out).len + 1)
        let skip = [Subset<A>] s => dp[s] * stay
        for i in m & out:
            for s in out.without(i).subsets: dp[s.with(i)] += dp[s]
        for s in out.subsets: dp[s] += skip[s]
    dp[out]
print(sum(ways(out) for out in Subset<A>.full.subsets) - 1)
