// etsr23r3p3 (NOI 2018 Return): from v, drive free over roads higher than the water level p,
// then walk; the walk is the least distance home from any vertex the car reaches. Merging roads
// from the highest down builds a Kruskal tree, where "reached above p" is one subtree; answers
// are online, so each subtree keeps its least walk.
@!multitest
let (n, m) = read<(1..=200_000, 1..=400_000)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, len: 1..=10_000, alt: 1..=1_000_000_000)>()
let home = dijkstra(Graph.undirected(roads).weighted(a => a.len), from: [(V.first!, 0)]).dist
let (domain N, kt) = kruskal_tree(roads, w: r => desc(r.alt))
var walk: [N]Int = inf
for v in V:
    walk[kt.leaf[v]] = home[v]!
let least = kt.tree.up(min, finish: (x, u) => min(x, walk[u]))
let (q, k, s) = read<(1..=400_000, 0..=1, 0..=1_000_000_000)>()
var last = 0
for _ in 0..q:
    let (v0, p0) = read<(V, 0..=s)>()
    let v = V((v0 + k * last - 1) % n + 1)
    let p = (p0 + k * last) % (s + 1)
    last = least[kt.reach(v, limit: desc(p + 1))]
    print(last)
