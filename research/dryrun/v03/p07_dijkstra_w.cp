// p07 with Dijkstra and a binary heap by hand (rows 7, 23, 24).
let (n, m): (2..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let flights = read<[E](u: V, v: V, w: 1..=1e9)>()
var adj: [V]List<(V, Int)> = []
for f in flights: adj[f.u].push((f.v, f.w))
var heap: List<(Int, Int)> = []
fn push(x: (Int, Int)):
    heap.push(x)
    var i = heap.len - 1
    while i > 0 and heap[(i - 1) / 2] > heap[i]:
        heap.swap(i, (i - 1) / 2)
        i = (i - 1) / 2
fn pop() -> (Int, Int):
    let top = heap[0]
    heap[0] = heap[heap.len - 1]
    heap.pop()
    var i = 0
    while true:
        var s = i
        for c in [2 * i + 1, 2 * i + 2] if c < heap.len and heap[c] < heap[s]: s = c
        if s == i: break
        heap.swap(i, s)
        i = s
    top
var dist: [V]?Int = None
dist[V.first!] = 0
push((0, Int(V.first!)))
while heap.len > 0:
    let (d, x) = pop()
    let u = V(x)
    if dist[u] != Some(d): continue
    for (v, w) in adj[u]:
        if dist[v] == None or d + w < dist[v]!:
            dist[v] = d + w
            push((d + w, Int(v)))
print([dist[v]! for v in V])
