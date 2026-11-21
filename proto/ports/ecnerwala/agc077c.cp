// AtCoder Grand Contest 077 C: decide whether an orientation reachable by
// flipping non-forbidden pairs can give the target out-degree parity profile.
//
// Start from the transitive tournament (i -> j for i < j), where vertex i has
// out-degree parity (n - 1 - i) mod 2, and flip both endpoints for each given
// backward edge (u > v).
// - When k is even, the condition always holds.
// - When k is odd, we need exactly n / 2 vertices with odd out-degree.
// Each allowed flip toggles two endpoints of an edge in the complement graph,
// so inside a complement component of size sz with `odd` initially-odd vertices
// we can achieve any odd count in [odd % 2, sz - (sz - odd) % 2] of the same
// parity as `odd`. Summing these intervals gives [lo, hi] for the whole graph.

use std.graph.undirected_of_pairs

@!multitest
let n: 1..=200_000 = read()
let m: 0..=200_000 = read()
let k: 1..=1_000_000_000 = read()
domain V = 1..=n

let edges: [0..m](V, V) = read()
var odd_out: [V]Bool = [V] v => (n - 1 - v.ord) % 2 == 1
for (u, v) in edges if u > v:
    odd_out[u] = not odd_out[u]
    odd_out[v] = not odd_out[v]

if k % 2 == 0:
    print("Yes")
    return

var forbidden = undirected_of_pairs(List(edges))
for u in V: forbidden[u].sort()
var unvisited = Set(V)

var lo = 0
var hi = 0
for start in V if start in unvisited:
    var sz = 0
    var odd = 0
    fn dfs(cur: V):
        unvisited.remove(cur)
        sz += 1
        if odd_out[cur]: odd += 1
        var cursor: ?V = V.first
        var ai = 0
        while let at = cursor:
            if let nxt = unvisited.first_ge(at):
                while ai < forbidden[cur].len and forbidden[cur][ai] < nxt: ai += 1
                if ai == forbidden[cur].len or forbidden[cur][ai] != nxt: dfs(nxt)
                cursor = nxt.next
            else:
                break
    dfs(start)
    lo += odd % 2
    hi += sz - (sz - odd) % 2

let target = n / 2
print(if lo <= target and target <= hi: "Yes" else: "No")
