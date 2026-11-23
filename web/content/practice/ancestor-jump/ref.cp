let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let es: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(es), V(1))
for _ in 0..q:
    let (v, k): (V, 0..=2e5) = read()
    if let p = t.ancestor(v, k): print(p)
    else: print(-1)
