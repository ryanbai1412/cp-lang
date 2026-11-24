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
let side = two_color<V>(u => V.filter(v => near(u, v)))!
let small = side.minority
domain A = V if side[v] == small
domain B = V if side[v] != small
let adj: [B]Subset<A> = [b => { a in A if near(a, b) }]

fn ways(out: Subset<A>) -> Z:
    var dp: [Subset<out>]Z = 1
    for b in B:
        let stay = (adj[b] - out).len + 1
        // any of the outsiders next to b may pick it: a zeta over adj[b]
        dp = stay * dp + dp.zeta(over: adj[b])
    return dp[out]
print(Subset<A>.all.map(ways).sum() - 1)
