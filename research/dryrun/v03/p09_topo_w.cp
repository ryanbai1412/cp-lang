// p09 with an adjacency list and a min-heap built by hand (rows 8, 23).
let (n, m): (1..=1e5, 1..=2e5) = read()
domain V = 1..=n
domain E = 0..m
let reqs = read<[E](u: V, v: V)>()
var adj: [V]List<V> = []
var indeg: [V]Int = 0
for r in reqs:
    adj[r.u].push(r.v)
    indeg[r.v] += 1
var heap: List<Int> = []
fn push(x: Int):
    heap.push(x)
    var i = heap.len - 1
    while i > 0 and heap[(i - 1) / 2] > heap[i]:
        heap.swap(i, (i - 1) / 2)
        i = (i - 1) / 2
fn pop() -> Int:
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
for v in V if indeg[v] == 0: push(Int(v))
var order: List<V> = []
while heap.len > 0:
    let u = V(pop())
    order.push(u)
    for v in adj[u]:
        indeg[v] -= 1
        if indeg[v] == 0: push(Int(v))
if order.len < n: print("IMPOSSIBLE") else: print(order)
