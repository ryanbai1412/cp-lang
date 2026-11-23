let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let es: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(es), V(1))
for _ in 0..q:
    let (a, b): (V, V) = read()
    print(t.depth(a) + t.depth(b) - 2 * t.depth(t.lca(a, b)))
