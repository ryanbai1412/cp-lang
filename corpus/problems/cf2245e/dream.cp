// Codeforces 2245 E — Tree Edge Gluing
//
// Glue every tree edge `e` to each of its even-degree endpoints (`u`, `v`).
// In `DsuOf`, each edge node carries weight `+1` and each vertex node carries
// weight `-1`, so every connected component's aggregate `diff` equals
// `edges - vertices`. A glued piece with `e` edges and `v` vertices has
// `extra = e - v + 1 = diff + 1` free slots and contributes `choose2(extra)`.

use std.dsu.{DsuOf}
use std.math.{choose2}

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
domain E = 0..n - 1
let edges: [E](u: V, v: V) = read()

var odd: [V]Bool = false
for (u, v) in edges:
    odd[u] = not odd[u]
    odd[v] = not odd[v]

domain X = V | E
var pieces = DsuOf<X, Int>(x => if x is E: 1 else: -1, +)
for (e, (u, v)) in edges.items:
    if not odd[u]: pieces.unite(E(e), V(u))
    if not odd[v]: pieces.unite(E(e), V(v))

print(sum(choose2(diff + 1) for diff in pieces.values()))
