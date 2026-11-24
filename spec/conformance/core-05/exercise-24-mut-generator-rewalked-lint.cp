// expect: L0604 at 8
// rule: 05.9.4
domain V = 1..=4
domain E = 0..2
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph.undirected(es)
var seen: [V]Bool = false
let walk = dfs(g, from: [V.first!], seen: mut seen)
let n1 = walk.count()
let n2 = walk.count()
print(n1, n2)
