// DMOPC '21 Contest 6 P4: colour each road 1 or 2. Roads at c get colour 1,
// roads at d colour 2 (a road joining c and d is impossible). A breadth-first
// search over (city, which of c and d were met) finds the nearest way to
// reach b without meeting both; the other roads take the colour it needs.
// original: corpus/submissions/4342347.cpp
let (n, m): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()
let (a, b, c, d): (V, V, V, V) = read()
let g = Graph.undirected(edges)

type Met = (c: Bool, d: Bool)                 // a product Index: 4 states per city
fn arrive(u: V, met: Met) -> (V, Met): (u, (met.c or u == c, met.d or u == d))

fn solve() -> ?List<Int>:                     // FRICTION X1: no top-level early exit
    if any(c in [e.u, e.v] and d in [e.u, e.v] for e in edges): return none
    let dist = bfs((u, met) => arrive(v, met) for v in g[u], from: [arrive(a, (false, false))])
    let ends = [(dist[b, met]!, met) for met in Met if not (met.c and met.d) and dist[b, met] != none]
    let rest = if min(ends)?.1.d: 2 else: 1
    [if c in [e.u, e.v]: 1 elif d in [e.u, e.v]: 2 else: rest for e in edges]

if let colours = solve():
    print(2)
    for x in colours: print(x)
else:
    print(-1)
