// Mock CCC '22 Contest 1 S4 (nccc9s4), buildable under proto/cpc today. See ../nccc9s4.cp.
// original: corpus/submissions/4145494.cpp
const B = 270
const HEAVY = 800

let n: 1..=100_000 = read()
let q: 1..=100_000 = read()
domain I = 0..n
domain Q = 0..q
let a: [I]Int = read()
var ql: [Q]Int = 0
var qr: [Q]Int = 0
for qi in Q:
    let l: 1..=100_000 = read()
    let r: 1..=100_000 = read()
    ql[qi] = l - 1
    qr[qi] = r - 1

let (V, vals) = compress(a)
var id: [0..n]Int = 0
for i in 0..n:
    let x = a[I.from_idx(i)!]
    var lo = 0
    var hi = V.K
    while lo < hi:
        let mid = (lo + hi) / 2
        if vals[V.from_idx(mid)!] < x: lo = mid + 1
        else: hi = mid
    id[i] = lo

var ans: [Q]Int = 0

if V.K <= HEAVY:
    var best_c: [Q]Int = 0
    var best_v: [Q]Int = 0
    var pre: [0..=n]Int = 0
    for v in 0..V.K:
        pre[0] = 0
        for i in 0..n:
            pre[i + 1] = pre[i] + (if id[i] == v: 1 else: 0)
        for qi in Q:
            let c = pre[qr[qi] + 1] - pre[ql[qi]]
            if c > best_c[qi] or (c == best_c[qi] and v > best_v[qi]):
                best_c[qi] = c
                best_v[qi] = v
    for qi in Q: ans[qi] = vals[V.from_idx(best_v[qi])!]
else:
    let nb = n / B + 1
    domain Bk = 0..=nb
    var buckets: [Bk]List<(Int, Int, Int)> = []
    var cnt: [0..V.K]Int = 0
    for qi in Q:
        let l = ql[qi]
        let r = qr[qi]
        if r - l + 1 <= B:
            var bc = 0
            var bv = 0
            for j in l..=r:
                let v = id[j]
                cnt[v] += 1
                let c = cnt[v]
                if c > bc or (c == bc and v > bv):
                    bc = c
                    bv = v
            ans[qi] = vals[V.from_idx(bv)!]
            for j in l..=r: cnt[id[j]] -= 1
        else:
            let bi = (l + B - 1) / B
            buckets[Bk.from_idx(bi)!].push((r, l, qi.ord))

    for bi in Bk:
        if buckets[bi].len == 0: continue
        sort(mut buckets[bi])
        for v in 0..V.K: cnt[v] = 0
        var base_c = 0
        var base_v = 0
        let mid = bi.ord * B
        var cr = mid - 1
        for e in buckets[bi]:
            let r = e.0
            let l = e.1
            let qid = e.2
            while cr < r:
                cr += 1
                let v = id[cr]
                cnt[v] += 1
                let c = cnt[v]
                if c > base_c or (c == base_c and v > base_v):
                    base_c = c
                    base_v = v
            var bc = base_c
            var bv = base_v
            for j in l..mid:
                let v = id[j]
                cnt[v] += 1
                let c = cnt[v]
                if c > bc or (c == bc and v > bv):
                    bc = c
                    bv = v
            ans[Q.from_idx(qid)!] = vals[V.from_idx(bv)!]
            for j in l..mid: cnt[id[j]] -= 1

for qi in Q: print(ans[qi])
