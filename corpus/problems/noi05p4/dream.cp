// noi05p4 Cat and Mouse: each turn the cat takes up to two steps toward the mouse (to the closest
// neighbour, smallest id on ties), then the mouse stays or moves to a uniform random neighbour.
// The expected number of turns is a DP over (cat, mouse): the cat's distance strictly falls.
let (n, e) = read<(1..=1_000, 1..=1_000)>()
domain V = 1..=n
let (cat, mouse) = read<(V, V)>()
let g = Graph.undirected(read<[e](u: V, v: V)>())
let dist = [V] t => bfs(g, from: [t]).dist
fn toward(c: V, m: V) -> V:
    if c == m: m else: g.out(c).map(a => a.to).min(by: v => (dist[m][v]!, v))!
@memo(array)
fn turns(c: V, m: V) -> F64:
    if c == m:
        return 0.0
    let c2 = toward(toward(c, m), m)
    if c2 == m:
        return 1.0
    let stay_or_move = turns(c2, m) + g.out(m).map(a => turns(c2, a.to)).sum()
    1.0 + stay_or_move / F64(g.deg(m) + 1)
print(turns(cat, mouse), precision: 3)
