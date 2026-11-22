@!multitest
let (n, m) = read<(1..=2e5, 1..=4e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, len: 1..=1e4, alt: 1..=1e9)>()
let home = dijkstra(Graph.undirected(roads).weighted(.len), from: [V.first!]).dist
let (domain N, kt) = kruskal_tree(roads, by: desc(.alt))
let least = kt.tree.up(min, leaf: v => home[v]!)
let (q, k, s) = read<(1..=4e5, 0..=1, 0..=1e9)>()
var last = 0
for (v0, p0) in read<[q](V, 0..=s)>():
    let v = V.ring.step(v0, k * last)
    let p = (0..=s).ring.step(p0, k * last)
    last = least[kt.top(v, ok: road => road.alt > p)]
    print(last)
