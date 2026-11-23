// cpl 0.3.0
// Problem: Tree Distances II (CSES 1133): for every vertex, the sum of
// distances to all other vertices.
// Input: n, then n - 1 lines "u v" (1-based). Output: n numbers.
// Written by hand (two DFS passes) because `Rooted.reroot` needs a monoid
// over a record and the spec gives no `+` on records (H-RECORD-MONOID).
// Guesses: H-ARRAY-SCALAR-FILL, H-ROW-PATTERN (`(to: u)` against an arc
// that also carries `e`), H-OPT-COMPARE (`u != from` with `from: ?V`).

let n = read<1..=2e5>()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
var size: [V]Int = 1
var down: [V]Int = 0
var ans: [V]Int = 0

fn gather(v: V, from: ?V):
    for (to: u, ..) in g[v] if u != from:
        gather(u, v)
        size[v] += size[u]
        down[v] += down[u] + size[u]

fn spread(v: V, from: ?V):
    for (to: u, ..) in g[v] if u != from:
        ans[u] = ans[v] - size[u] + (n - size[u])
        spread(u, v)

let root = V.first!
gather(root, None)
ans[root] = down[root]
spread(root, None)
print(ans)
