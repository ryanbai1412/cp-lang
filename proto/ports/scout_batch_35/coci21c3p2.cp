// COCI 2021/22 #3 Cijanobakterije: a forest on n vertices. Add edges
// (keeping it a forest) to make the longest simple path as long as possible;
// print how many vertices are on it.
//
// The best chain strings the longest path of every tree end to end, so the
// answer is the sum of the trees' diameters, counted in vertices.
// original: corpus/submissions/4196157.cpp
let (n, m): (1..=1e5, 0..n) = read()
domain V = 1..=n
let edges: [0..m](V, V) = read()
let g = Graph.undirected(edges)

var seen: [V]Bool = false
var longest = 0
// vertices on the longest path going down from u; `longest` sees the best path bending at u
fn down(u: V, parent: ?V) -> Int:
    seen[u] = true
    var deep = 0
    for v in g[u] if v != parent:
        let d = down(v, u)
        longest max= deep + d + 1
        deep max= d
    deep + 1

var total = 0
for root in V if not seen[root]:
    longest = 1
    down(root, none)
    total += longest
print(total)
