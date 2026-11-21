// A topological order of a DAG (Kahn); prints the sum of (i + 1) * order[i].
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let order = topo(Graph.directed(edges))!
var total = 0
for i in 0..order.len: total += (i + 1) * order[i].to_idx
print(total)
