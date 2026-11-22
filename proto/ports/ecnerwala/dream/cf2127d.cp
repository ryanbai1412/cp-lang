// Codeforces 2127 D — Root Was Built by Love, Broken by Destiny
//
// Place the n vertices on two parallel banks so no two of the m edges cross.
// A connected graph admits such a 2-bank planar drawing iff it is a tree
// (m == n - 1) whose non-leaf spine (vertices with deg > 1) induces a path
// (every spine vertex has at most 2 spine neighbours, `deg2[v] <= 2`).
//
// Counting valid drawings modulo 10^9 + 7:
// - If the spine has `spine_edges >= 1` edges (at least 2 spine vertices),
//   orienting the spine left-to-right (2 ways) and choosing the starting bank
//   (2 ways) gives 4, times `(deg[v] - deg2[v])!` for ordering each spine
//   vertex's leaf children on the opposite bank.
// - If `spine_edges == 0` (a star, n >= 2), choosing the center's bank (2 ways)
//   and ordering all n - 1 leaves gives `2 * (n - 1)!`.

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

if any(deg2[v] >= 3 for v in V):
    print(0)
    return

if spine_edges >= 1:
    print(Z(4) * fold(*, comb.fact(deg[v] - deg2[v]) for v in V if deg[v] > 1))
else:
    print(Z(2) * comb.fact(n - 1))
