// CCO '23 P2 — Real Mountains (today form for proto/cpc)
// original: corpus/submissions/5579928.cpp
let n: 1..=1_000_000 = read()
domain V = 0..n
let h: [V]Int = read()

let MOD = 1_000_003

var pref: [V]Int = 0
var mx = 0
for i in V:
    mx max= h[i]
    pref[i] = mx

var suff: [V]Int = 0
mx = 0
var ri = n - 1
while ri >= 0:
    mx max= h[V(ri)]
    suff[V(ri)] = mx
    ri -= 1

var cap: [V]Int = [V] i => min(pref[i], suff[i])

// Coordinate-compress h to support O(log n) first_gt queries via a min-segtree.
var sorted_h: List<Int> = []
for i in V: sorted_h.push(h[i])
sorted_h.sort()
var uniq: List<Int> = []
for x in sorted_h:
    if uniq.len == 0 or uniq[uniq.len - 1] != x:
        uniq.push(x)
let U = uniq.len
var id_of = HashMap<Int, Int>()
for i in 0..U: id_of[uniq[i]] = i

// Group indices by compressed height id (0..U), already in ascending height order.
domain H = 0..U
var by_h: [H]List<Int> = []
for i in 0..n: by_h[H(id_of[h[V(i)]])].push(i)

fn tri(x: Int) -> Int:
    (x % MOD) * ((x + 1) % MOD) % MOD * 500_002 % MOD

fn sum_range(l: Int, r: Int) -> Int:
    (tri(r) - tri(l - 1) + MOD) % MOD

var pq_x: [V]List<Int> = []
var pq_m: [V]List<Int> = []
var sq_x: [V]List<Int> = []
var sq_m: [V]List<Int> = []

var active = Set<Int>()
var ans = 0
var prev_h: ?Int = none

for cid in 0..U:
    let cur_h = uniq[cid]
    if let lo_h = prev_h:
        if active.len > 0:
            let first = active.min()!
            let last = active.max()!
            let span = cur_h - lo_h
            let sz = active.len
            if sz == 1:
                ans = (ans + sum_range(lo_h, cur_h - 1)) % MOD
                pq_x[V(first)].push(lo_h)
                pq_m[V(first)].push(span)
                sq_x[V(first)].push(lo_h)
                sq_m[V(first)].push(span)
            else:
                let t1 = (sz % MOD) * sum_range(lo_h, cur_h - 1) % MOD
                let t2 = ((sz * 2 - 3) % MOD) * sum_range(lo_h + 1, cur_h) % MOD
                ans = (ans + t1 + t2) % MOD
                pq_x[V(first)].push(lo_h)
                pq_m[V(first)].push(span)
                sq_x[V(last)].push(lo_h)
                sq_m[V(last)].push(span)
                pq_x[V(n - 1)].push(lo_h)
                pq_m[V(n - 1)].push(span)
    for idx in by_h[H(cid)]: active.insert(idx)
    while active.len > 0:
        let f = active.min()!
        if cur_h >= cap[V(f)]: active.remove(f)
        else: break
    while active.len > 0:
        let l = active.max()!
        if cur_h >= cap[V(l)]: active.remove(l)
        else: break
    prev_h = cur_h

// Iterative min-segtree over compressed height ids 0..U to answer first_gt(x).
var st_sz = 1
while st_sz < U: st_sz = st_sz * 2
let INF = 2_000_000_000
var seg: List<Int> = []
for _ in 0..2 * st_sz: seg.push(INF)

fn seg_reset():
    for i in 0..2 * st_sz: seg[i] = INF

fn seg_add(pos0: Int, val: Int):
    var p = pos0 + st_sz
    seg[p] = val
    while p > 1:
        p = p / 2
        seg[p] = min(seg[p * 2], seg[p * 2 + 1])

fn seg_min(l0: Int, r0: Int) -> Int:
    var l = l0 + st_sz
    var r = r0 + st_sz
    var res = INF
    while l <= r:
        if l % 2 == 1:
            res min= seg[l]
            l += 1
        if r % 2 == 0:
            res min= seg[r]
            r -= 1
        l = l / 2
        r = r / 2
    res

seg_reset()
for i in 0..n:
    let hv = h[V(i)]
    seg_add(id_of[hv], hv)
    let cnt = pq_x[V(i)].len
    for q in 0..cnt:
        let qx = pq_x[V(i)][q]
        let qm = pq_m[V(i)][q]
        let gt = seg_min(id_of[qx] + 1, U - 1)
        ans = (ans + (qm % MOD) * (gt % MOD)) % MOD

seg_reset()
ri = n - 1
while ri >= 0:
    let hv = h[V(ri)]
    seg_add(id_of[hv], hv)
    let cnt = sq_x[V(ri)].len
    for q in 0..cnt:
        let qx = sq_x[V(ri)][q]
        let qm = sq_m[V(ri)][q]
        let gt = seg_min(id_of[qx] + 1, U - 1)
        ans = (ans + (qm % MOD) * (gt % MOD)) % MOD
    ri -= 1

print(ans)
