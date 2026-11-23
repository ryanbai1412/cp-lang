// cf2000G Call During the Journey: the latest moment one may stand at v and still reach
// n by t0, settled from n backwards (largest first). A bus ride [at - bus, at] may not
// overlap the call (t1, t2); then the bus must arrive by t1 instead.
@!multitest
let (n, m): (2..=1e5, 1..=1e5) = read()
let (t0, t1, t2): (1..=1e9, 1..=1e9, 1..=1e9) = read()
domain V = 1..=n
let g = Graph.undirected(read<[m](u: V, v: V, bus: 1..=1e9, walk: 1..=1e9)>())

var latest: [V]?Int = None
var heap: MaxHeap<(Int, V)> = MaxHeap()
latest[V.last!] = Some(t0)
heap.push((t0, V.last!))
while let (at, u) = heap.pop():
    if latest[u] != Some(at): continue
    for a in g[u]:
        let leave = at - a.bus
        let bus = if leave < t2 and at > t1: t1 - a.bus else: leave
        if latest[a.to] max= max(bus, at - a.walk):
            heap.push((latest[a.to]!, a.to))
let start = latest[V.first!]!
print(if start >= 0: start else: -1)
