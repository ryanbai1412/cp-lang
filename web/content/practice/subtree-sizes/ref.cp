let n: 1..=2e5 = read()
domain V = 1..=n
let es: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(es), V(1))
print(t.subtree_sizes)
