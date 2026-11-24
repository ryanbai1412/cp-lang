// Source: Codeforces 2127D (Root was Built by Love, Broken by Destiny),
//   https://codeforces.com/contest/2127/problem/D
// limits: 2000 ms, 256 MB
// Problem: a connected simple graph with n houses and m bridges. Place the houses on two
//   parallel river banks (order on each bank matters) so every bridge joins opposite banks and
//   no two bridges cross; count arrangements mod 1e9+7. n, m <= 2e5 (sums too); multitest.
// Solution: such a drawing exists iff the graph is a tree (m = n - 1) whose non-leaf vertices
//   (the spine) form a path: no spine vertex has 3 spine neighbours. With a spine edge there
//   are 4 ways to orient the spine and pick the start bank, times (leaves at v)! per spine
//   vertex v; a star gives 2 (n - 1)!. Degree counts and factorials: O(n + m).
// Bounds: n: 2..=200_000; m: 1..=200_000; edges.u, edges.v: 1..=n

const MOD = 1_000_000_007
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let (n, m): (2..=200_000, 1..=200_000) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()
if m > n - 1:
    print(0)
    return

var deg: [V]Int = 0
for (u, v) in edges:
    deg[u] += 1
    deg[v] += 1

var deg2: [V]Int = 0
var spine_edges = 0
for (u, v) in edges if deg[u] > 1 and deg[v] > 1:
    deg2[u] += 1
    deg2[v] += 1
    spine_edges += 1

if V.any(v => deg2[v] >= 3):
    print(0)
    return

if spine_edges >= 1:
    print(Z(4) * fold(*, comb.fact(deg[v] - deg2[v]) for v in V if deg[v] > 1))
else:
    print(Z(2) * comb.fact(n - 1))
