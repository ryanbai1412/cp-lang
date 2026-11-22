// UCRPC 2021 H, four couriers splitting round trips within 600 (buildable today-form)
// original: corpus/submissions/3950286.cpp
const LIMIT = 300
let n: 1..=100 = read()
let m: 0..=1e4 = read()
domain V = 0..n
var adj: [V]List<(V, Int)> = []
var too_long = false
for _ in 0..m:
    let u: V = read()
    let v: V = read()
    let w: Int = read()
    if w > LIMIT: too_long = true
    adj[u].push((v, w))
    adj[v].push((u, w))

var dist: [V]Int = 1e18
var pq = MinHeap<(Int, V)>()
dist[V.first] = 0
pq.push((0, V.first))
while pq.len > 0:
    let (d, u) = pq.pop()!
    if d == dist[u]:
        for (v, w) in adj[u]:
            if d + w < dist[v]:
                dist[v] = d + w
                pq.push((d + w, v))

var total = 0
var far = 0
for v in V:
    total += dist[v]
    far max= dist[v]

if too_long or far > LIMIT or total > 4 * LIMIT:
    print("Impossible!")
else:
    // split[a, b, c]: three couriers can carry exactly a, b and c; the fourth takes the rest
    let cap = min(LIMIT, total)
    var split: [0..=cap, 0..=cap, 0..=cap]Bool = false
    split[0, 0, 0] = true
    for v in V:
        let x = dist[v]
        if x > 0:
            for a in (0..=cap).rev:
                for b in (0..=cap).rev:
                    for c in (0..=cap).rev:
                        if not split[a, b, c]:
                            if (a >= x and split[a - x, b, c]) or (b >= x and split[a, b - x, c]) or (c >= x and split[a, b, c - x]):
                                split[a, b, c] = true
    var best = 1e18
    for a in 0..=cap:
        for b in 0..=cap:
            for c in 0..=cap:
                if split[a, b, c]: best min= max(max(a, b), max(c, total - a - b - c))
    if best <= LIMIT: print(2 * best)
    else: print("Impossible!")
