// expect: ok, output "2"
// rule: 05.10.1
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph.undirected(es)
let s = V.first!
var depth: [V]Int = 0
fn visit(v: V, from: ?V):
    for (to: u, ..) in g[v]:
        if u != from:
            depth[u] = depth[v] + 1
            visit(u, v)
visit(V.first!, None)
print(depth.max())
visit(V.first!, None)
