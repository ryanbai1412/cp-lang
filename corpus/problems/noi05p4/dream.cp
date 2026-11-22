let (n, e) = read<(1..=1_000, 1..=1_000)>()
domain V = 1..=n
let (cat, mouse) = read<(V, V)>()
let g = Graph.undirected(read<[e](u: V, v: V)>())
let dist = [V] t => bfs(g, from: [t]).dist
fn toward(c: V, m: V) -> V = if c == m: m else: g.next(c).min(by: v => (dist[m][v]!, v))!

@memo(array)
fn turns(c: V, m: V) -> F64:
    if c == m:
        return 0.0
    let c2 = toward(toward(c, m), m)
    if c2 == m:
        return 1.0
    1.0 + chain([m], g.next(m)).mean(of: m2 => turns(c2, m2))

print(turns(cat, mouse), precision: 3)
