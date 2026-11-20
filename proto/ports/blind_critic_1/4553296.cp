// Corpus submission 4553296: directed-graph fixed-point propagation (SPFA-style queue)
let n: 1..=2e5 = read()
let m: 0..=2e5 = read()

domain V = 1..=n
domain E = 1..=m

let edges: [E](u: V, v: V, r: Int, p: Int) = read()

var preds: [V]Set<V> = Set<V>()
var graph: [V]List<(v: V, r: Int, p: Int)> = []

for e in E:
    let ed = edges[e]
    preds[ed.v].insert(ed.u)
    graph[ed.u].push((ed.v, ed.r, ed.p))

var queued: [V]Bool = false
var q: List<V> = []
var head = 0

fn enqueue(u: V):
    if not queued[u]:
        queued[u] = true
        q.push(u)

var ans: [V]?Int = 0
for v in V:
    enqueue(v)

while head < q.len:
    var u = q[head]
    head += 1
    queued[u] = false
    var cur: ?Int = none
    for ed in graph[u]:
        if let av = ans[ed.v]:
            let cand = max(ed.r, av - ed.p)
            cur min= cand
    if cur != ans[u]:
        ans[u] = cur
        for p in preds[u]:
            enqueue(p)

var out: [V]Int = [V] v => ans[v] ?? -1
print(out)
