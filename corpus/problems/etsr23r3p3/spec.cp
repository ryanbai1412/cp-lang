// etsr23r3p3 (NOI 2018 Return): from v, drive free over roads higher than the water level p,
// then walk; the walk is the least distance home from any vertex the car reaches. Merging roads
// from the highest down builds a Kruskal tree, where "reached above p" is one subtree; answers
// are online, so each subtree keeps its least walk.
@!multitest
let (n, m) = read<(1..=2e5, 1..=4e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, len: 1..=1e4, alt: 1..=1e9)>()
let home = dijkstra(Graph.undirected(roads).weighted(.len), from: [(V.first!, 0)]).dist
let (domain N, kt) = kruskal_tree(roads, w: r => desc(r.alt))
let least = kt.up(min, leaf: v => home[v]!)
let (q, k, s) = read<(1..=4e5, 0..=1, 0..=1e9)>()
var last = 0
for (v0, p0) in read<[q](V, 0..=s)>():
    let v = V.ring.step(v0, k * last)
    let p = (0..=s).ring.step(p0, k * last)
    last = least[kt.top(v, ok: e => roads[e].alt > p)]
    print(last)
