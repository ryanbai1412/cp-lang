// NOI 2018 Return: walk home from wherever the car can reach above the water level. Dijkstra gives
// every walk home; a Kruskal tree by decreasing altitude turns "reachable by car" into a subtree,
// and folding min up the tree keeps the shortest walk in each.
@!multitest
let (n, m) = read<(1..=2e5, 0..=4e5)>()
domain V = 1..=n
let roads = read<[m](u: V, v: V, w: 1..=1e4, alt: 1..=1e9)>()
let home = dijkstra(Graph.undirected(roads), from: [(V.first!, 0)])
let (domain N, kt) = kruskal_tree(roads, w: r => desc(roads[r].alt))
let least = kt.up(min, leaf: v => home[v]!)
let (q, k, s) = read<(1..=4e5, 0..=1, 0..=1e9)>()
var last = 0
for (v0, p0) in read<[q](V, 0..=s)>():
    let v = V.nth((v0.ord + k * last) % n)
    let p = (p0 + k * last) % (s + 1)
    last = least[kt.reach(v, desc(p + 1))]
    print(last)
