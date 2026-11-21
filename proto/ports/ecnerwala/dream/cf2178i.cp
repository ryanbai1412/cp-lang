// Codeforces 2178 I: Numbers or Fireworks (dream).
// Expand each (f + 1): every city outside T picks nothing or one firework
// neighbour. Sqrt(k)-apart lattice cities form a bipartite graph (parity);
// enumerate which cities of the smaller side are outside T, then sweep the
// other side, tracking which of those have picked a firework.
type Z = Mod<998_244_353>

@!multitest
let (n, k): (2..=31, 1..=2e4) = read()
domain V = 0..n
let p: [V]Vec2<1..=100> = read()
fn near(u: V, v: V) -> Bool = (p[u] - p[v]).norm2 == k
let side = two_color::<V>(u => V.filter(v => near(u, v)))!
let small = side.minority
domain A = V where side[v] == small
domain B = V where side[v] != small
let adj: [B]Subset<A> = [b => {a in A if near(a, b)}]

var ans: Z = 0
for out in Subset<A>.all:
    var dp: [Subset<out>]Z = 1
    for b in B:
        let stay = (adj[b] - out).len + 1
        dp = [s => dp[s] * stay + (dp[s - {a}] for a in s & adj[b]).sum()]
    ans += dp[out]
print(ans - 1)
