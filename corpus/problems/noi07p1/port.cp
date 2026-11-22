// NOI '07 P1 - Social Network (runnable on proto/cp.py lib-run today)
let (n, m): (Int, Int) = read()
let INF = 1000000000000
let stride = n + 1
var dist = List.filled(stride * stride, INF)
var ways = List.filled(stride * stride, 0)

fn cell(i: Int, j: Int) -> Int = i * stride + j

for v in 1..=n:
    dist[cell(v, v)] = 0
    ways[cell(v, v)] = 1

for _ in 0..m:
    let (u, v, w): (Int, Int, Int) = read()
    dist[cell(u, v)] = w
    dist[cell(v, u)] = w
    ways[cell(u, v)] = 1
    ways[cell(v, u)] = 1

for k in 1..=n:
    for i in 1..=n:
        let dik = dist[cell(i, k)]
        if dik == INF: continue
        for j in 1..=n:
            let dkj = dist[cell(k, j)]
            if dkj == INF: continue
            let cand = dik + dkj
            let ij = cell(i, j)
            if cand < dist[ij]:
                dist[ij] = cand
                ways[ij] = ways[cell(i, k)] * ways[cell(k, j)]
            elif cand == dist[ij] and k != i and k != j:
                ways[ij] += ways[cell(i, k)] * ways[cell(k, j)]

for v in 1..=n:
    var total = 0.0
    for s in 1..=n:
        if s == v: continue
        let dsv = dist[cell(s, v)]
        if dsv == INF: continue
        for t in 1..=n:
            if t == v or s == t: continue
            let dvt = dist[cell(v, t)]
            let dst = dist[cell(s, t)]
            if dvt < INF and dsv + dvt == dst:
                total += F64(ways[cell(s, v)] * ways[cell(v, t)]) / F64(ways[cell(s, t)])
    print(total)
