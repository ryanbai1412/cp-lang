// Mock CCC '21 S4 (nccc8s4): m tiles each carry 4 edge labels in -26..=26
// (0 = flat). Non-zero x connects to -x; detect whether an infinite chain
// (directed cycle on non-zero labels) exists.
// original: corpus/submissions/3381215.java
domain Side = 0..=52
let m: 1..=100_000 = read()
var adj: [Side, Side]Bool = false
for _ in 0..m:
    let tile: [0..4](-26..=26) = read()
    for i in 0..4 if tile[i] != 0:
        for j in 0..4 if j != i and tile[j] != 0:
            adj[Side.from_idx(tile[i] + 26)!, Side.from_idx(26 - tile[j])!] = true

var edges: List<(Side, Side)> = []
for u in Side:
    for v in Side if adj[u, v]: edges.push((u, v))
let g = Graph.directed(edges)
print(if topo(g) == none: 1 else: 0)
