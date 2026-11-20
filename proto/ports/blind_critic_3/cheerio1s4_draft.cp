// original: 5339437 (cheerio1s4) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
let (n, m): (Int, Int) = read()
let a: [0..n]Int = read()
var prv: [0..n]?Int = none
var prv2: [0..n]?Int = none
var nxt: [0..n]?Int = none
var nxt2: [0..n]?Int = none
var seen: [1..=m + 1]?Int = none
for i in 0..n:
    prv[i] = seen[a[i]]
    seen[a[i]] = i
    prv2[i] = seen[a[i] + 1]
seen = none
for i in (0..n).rev:
    nxt[i] = seen[a[i]]
    seen[a[i]] = i
    nxt2[i] = seen[a[i] + 1]
var dist: [0..n]?Int = none
var pq: MinHeap<(Int, Int)> = []
fn relax(u: Int, d: Int):
    if let old = dist[u]:
        if old <= d: return
    dist[u] = d
    pq.push((d, u))
for i in 0..n:
    if a[i] == 1: relax(i, i + 1)
while not pq.is_empty:
    let (d, u) = pq.pop()!
    if dist[u] != d: continue
    for v in [prv[u], nxt[u], prv2[u], nxt2[u]]:
        if let w = v: relax(w, d + abs(u - w))
var ans: ?Int = none
for i in 0..n:
    if a[i] == m: ans min= dist[i]
print(ans ?? 1e18)
