// CCO '22 P5 — Phone Plans / Two-Graph Connectivity (today form for proto/cpc)
// original: corpus/submissions/4801885.cpp
let n: 1..=200_000 = read()
let ma: 0..=200_000 = read()
let mb: 0..=200_000 = read()
let k: 0..=20_000_000_000 = read()

struct Edge:
    w: Int
    u: Int
    v: Int

fn read_edges(m: Int) -> List<Edge>:
    var es: List<Edge> = []
    for _ in 0..m:
        let u: Int = read()
        let v: Int = read()
        let w: Int = read()
        es.push(Edge(w, u - 1, v - 1))
    es.sort()
    es

let ea = read_edges(ma)
let eb = read_edges(mb)

fn c2(x: Int) -> Int: x * (x - 1) / 2

domain V = 0..n
var rep_a: [V]V = [V] i => i
var mem_a: [V]List<V> = []
var rep_b: [V]V = [V] i => i
var mem_b: [V]List<V> = []
for i in V:
    mem_a[i].push(i)
    mem_b[i].push(i)

var pairs_a = 0
var pairs_b = 0

var hist_pairs: List<Int> = []
var hist_off: List<Int> = [0]
var ch_v: List<V> = []
var ch_old: List<V> = []

for e in eb:
    hist_pairs.push(pairs_b)
    var u = rep_b[V(e.u)]
    var v = rep_b[V(e.v)]
    if u != v:
        if mem_b[u].len > mem_b[v].len:
            let tmp = u
            u = v
            v = tmp
        pairs_b -= c2(mem_b[u].len) + c2(mem_b[v].len)
        for idx in 0..mem_b[u].len:
            let x = mem_b[u][idx]
            ch_v.push(x)
            ch_old.push(rep_b[x])
            rep_b[x] = v
            mem_b[v].push(x)
        mem_b[u].clear()
        pairs_b += c2(mem_b[v].len)
    hist_off.push(ch_v.len)

let S = n + 1
var overlap = 0
var pkey: [V]Int = [V] i => i.idx * S + rep_b[i].idx
var freq = HashMap<Int, Int>()
for i in V:
    let ky = pkey[i]
    let c = freq.get(ky) ?? 0
    overlap += c
    freq[ky] = c + 1

fn relabel(x: V):
    let old = pkey[x]
    let co = freq[old] - 1
    freq[old] = co
    overlap -= co
    let nxt = rep_a[x].idx * S + rep_b[x].idx
    pkey[x] = nxt
    let cn = freq.get(nxt) ?? 0
    overlap += cn
    freq[nxt] = cn + 1

var j = mb - 1
var ans: ?Int = none

fn relax(wa: Int):
    while j >= 0 and pairs_a + pairs_b - overlap >= k:
        pairs_b = hist_pairs[j]
        let lo = hist_off[j]
        let hi = hist_off[j + 1]
        for t in lo..hi: rep_b[ch_v[t]] = ch_old[t]
        for t in lo..hi: relabel(ch_v[t])
        j -= 1
    if pairs_a + pairs_b - overlap >= k: ans min= wa
    if j + 1 < mb: ans min= wa + eb[j + 1].w

relax(0)
for e in ea:
    var u = rep_a[V(e.u)]
    var v = rep_a[V(e.v)]
    if u != v:
        if mem_a[u].len > mem_a[v].len:
            let tmp = u
            u = v
            v = tmp
        pairs_a -= c2(mem_a[u].len) + c2(mem_a[v].len)
        for idx in 0..mem_a[u].len:
            let x = mem_a[u][idx]
            rep_a[x] = v
            mem_a[v].push(x)
            relabel(x)
        mem_a[u].clear()
        pairs_a += c2(mem_a[v].len)
    relax(e.w)

if let a = ans: print(a) else: print(-1)
