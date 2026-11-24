// expect: ok, output "0 1 2 3\n3\n4 0\nYES 6\n8 3 2 1"
// rule: std.graph.peel
// The graph free fns bind `V` from a closure graph's parameter with no type arguments, and
// `floyd_warshall`'s default `over:` is the (min, +) semiring over the matrix's weights.
use std.graph.{bfs, peel, scc, components, floyd_warshall, dfs, reach_sets, dominators, count_paths}

domain V = 0..4
let nxt = (u: V) => if u.ord < 3: [V(u.ord + 1)] else: []
print(peel(nxt))
let d = bfs(nxt, [V(0)])
print(d[V(3)])
let (C:, comp:) = scc(nxt)
print(C.count, components(nxt).comp[V(2)].ord)
var w = [V, V] (a, b) => if b.ord == a.ord + 1: Some(2) else: None
print(yes(floyd_warshall(mut w)), w[V(0), V(3)]!)
print(dfs(nxt, [V(0)]).count(), reach_sets(nxt)[V(1)].len, dominators(nxt, V(0))[V(3)]!.ord, count_paths<Int>(nxt, [V(0)])![V(3)])
