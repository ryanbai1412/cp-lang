// CF 1915G Bicycles: Dijkstra over (city, slowness of the bike held); at each city keep the
// better of the held bike and the local one.
@!multitest
domain V = 1..=read<2..=1000>()
let m: 1..=1000 = read()
let roads = read<[m](u: V, v: V, w: 1..=1e5)>()
let s = read<[V]1..=1000>()
let g = Graph.undirected(roads)
var best: [V, V]?Int = None
var done: [V, V]Bool = false
var h = MinHeap<(Int, V, V)>()
best[V.first!, V.first!] = Some(0)
h.push((0, V.first!, V.first!))
while let (t, u, bike) = h.pop():
    if done[u, bike]:
        continue
    done[u, bike] = true
    if u == V.last!:
        print(t)
        return
    let held = if s[u] < s[bike]: u else: bike
    for e in g.out(u):
        let nt = t + e.w * s[held]
        if best[e.to, held] min= nt:
            h.push((nt, e.to, held))
