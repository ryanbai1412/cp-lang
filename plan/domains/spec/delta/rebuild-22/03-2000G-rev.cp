// cf2000G, revised: there are several calls, given as q intervals (t1, t2) that do not
// overlap; a bus ride may overlap none of them.
@!multitest
let (n, m): (2..=1e5, 1..=1e5) = read()
let (t0, q): (1..=1e9, 1..=1e5) = read()
let calls = read<[q](t1: 1..=1e9, t2: 1..=1e9)>().sorted(by: c => c.t1)
domain V = 1..=n
let g = Graph.undirected(read<[m](u: V, v: V, bus: 1..=1e9, walk: 1..=1e9)>())

// the latest departure no later than `leave` whose ride [dep, dep + len] avoids every call
fn depart(leave: Int, len: Int) -> Int:
    var dep = leave
    for c in calls.rev if c.t1 < dep + len:
        if dep < c.t2: dep = c.t1 - len
    dep

var latest: [V]?Int = None
var heap: MaxHeap<(Int, V)> = MaxHeap()
latest[V.last!] = Some(t0)
heap.push((t0, V.last!))
while let (at, u) = heap.pop():
    if latest[u] != Some(at): continue
    for a in g[u]:
        let bus = depart(at - a.bus, a.bus)
        if latest[a.to] max= max(bus, at - a.walk):
            heap.push((latest[a.to]!, a.to))
let start = latest[V.first!]!
print(if start >= 0: start else: -1)
