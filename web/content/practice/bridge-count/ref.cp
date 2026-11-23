use std.graph.{bridges}
let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let es: [E](u: V, v: V) = read()
print(bridges(Graph.undirected(es)).count(b => b))
