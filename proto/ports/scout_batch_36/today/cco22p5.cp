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

var ea: List<Edge> = []
for _ in 0..ma:
    let u: Int = read()
    let v: Int = read()
    let w: Int = read()
    ea.push(Edge(w, u - 1, v - 1))

var eb: List<Edge> = []
for _ in 0..mb:
    let u: Int = read()
    let v: Int = read()
    let w: Int = read()
    eb.push(Edge(w, u - 1, v - 1))

ea.sort()
eb.sort()

fn c2(x: Int) -> Int: x * (x - 1) / 2

domain V = 0..n
var rep_a: [V]Int = [V] i => i.idx
var mem_a: [V]List<Int> = []
var rep_b: [V]Int = [V] i => i.idx
var mem_b: [V]List<Int> = []
for i in 0..n:
    mem_a[V(i)].push(i)
    mem_b[V(i)].push(i)

var pairs_a = 0
var pairs_b = 0

// Build B completely, storing per-edge flat slices of (v, old_rep_b).
var hist_pairs: List<Int> = []
var hist_off: List<Int> = [0]
var ch_v: List<Int> = []
var ch_old: List<Int> = []

for ei in 0..mb:
    let e = eb[ei]
    hist_pairs.push(pairs_b)
    var u = rep_b[V(e.u)]
    var v = rep_b[V(e.v)]
    if u != v:
        if mem_b[V(u)].len > mem_b[V(v)].len:
            let tmp = u
            u = v
            v = tmp
        pairs_b -= c2(mem_b[V(u)].len) + c2(mem_b[V(v)].len)
        for idx in 0..mem_b[V(u)].len:
            let x = mem_b[V(u)][idx]
            ch_v.push(x)
            ch_old.push(rep_b[V(x)])
            rep_b[V(x)] = v
            mem_b[V(v)].push(x)
        mem_b[V(u)].clear()
        pairs_b += c2(mem_b[V(v)].len)
    hist_off.push(ch_v.len)

let STRIDE = n + 1
var overlap = 0
var pair_key: [V]Int = [V] i => i.idx * STRIDE + rep_b[i]
var freq = HashMap<Int, Int>()
for i in 0..n:
    let ky = pair_key[V(i)]
    let c = freq.get(ky) ?? 0
    overlap += c
    freq[ky] = c + 1

fn relabel_one(x: Int):
    let old = pair_key[V(x)]
    let co = freq[old] - 1
    freq[old] = co
    overlap -= co
    let nxt = rep_a[V(x)] * STRIDE + rep_b[V(x)]
    pair_key[V(x)] = nxt
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
        for t in lo..hi:
            rep_b[V(ch_v[t])] = ch_old[t]
        for t in lo..hi:
            relabel_one(ch_v[t])
        j -= 1
    if pairs_a + pairs_b - overlap >= k:
        ans min= wa
    if j + 1 < mb:
        ans min= wa + eb[j + 1].w

relax(0)
for ei in 0..ma:
    let e = ea[ei]
    var u = rep_a[V(e.u)]
    var v = rep_a[V(e.v)]
    if u != v:
        if mem_a[V(u)].len > mem_a[V(v)].len:
            let tmp = u
            u = v
            v = tmp
        pairs_a -= c2(mem_a[V(u)].len) + c2(mem_a[V(v)].len)
        for idx in 0..mem_a[V(u)].len:
            let x = mem_a[V(u)][idx]
            rep_a[V(x)] = v
            mem_a[V(v)].push(x)
            relabel_one(x)
        mem_a[V(u)].clear()
        pairs_a += c2(mem_a[V(v)].len)
    relax(e.w)

if let a = ans: print(a) else: print(-1)
