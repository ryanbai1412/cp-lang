// Lowest common ancestors on a tree rooted at 1 (heavy-light); prints the
// sum of the answers to q queries.
let (n, q): (2..=1e6, 1..=1e6) = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(edges), V(1))
var total = 0
for _ in 0..q:
    let (a, b): (V, V) = read()
    total += t.lca(a, b).to_idx
print(total)
