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
for i in V.rev:
    mx max= h[i]
    suff[i] = mx

let cap: [V]Int = [V] i => min(pref[i], suff[i])

var sorted_h: List<Int> = []
for i in V: sorted_h.push(h[i])
sorted_h.sort()
var uniq: List<Int> = []
for x in sorted_h:
    if uniq.len == 0 or uniq[uniq.len - 1] != x: uniq.push(x)
let U = uniq.len
var id_of = HashMap<Int, Int>()
for i in 0..U: id_of[uniq[i]] = i

domain H = 0..U
var by_h: [H]List<Int> = []
for i in V: by_h[H(id_of[h[i]])].push(i.idx)

fn tri(x: Int) -> Int: (x % MOD) * ((x + 1) % MOD) % MOD * 500_002 % MOD
fn sum_range(l: Int, r: Int) -> Int: (tri(r) - tri(l - 1) + MOD) % MOD

var pq_x: [V]List<Int> = []
var pq_m: [V]List<Int> = []
var sq_x: [V]List<Int> = []
var sq_m: [V]List<Int> = []

var active = Set<Int>()
var ans = 0
var prev_h: ?Int = none

for cid in H:
    let cur_h = uniq[cid.idx]
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
    for idx in by_h[cid]: active.insert(idx)
    while active.len > 0 and cur_h >= cap[V(active.min()!)]: active.remove(active.min()!)
    while active.len > 0 and cur_h >= cap[V(active.max()!)]: active.remove(active.max()!)
    prev_h = cur_h

var st_sz = 1
while st_sz < U: st_sz *= 2
let INF = 2_000_000_000
var seg: [0..2 * st_sz]Int = INF

fn seg_add(pos0: Int, val: Int):
    var p = pos0 + st_sz
    seg[p] = val
    while p > 1:
        p /= 2
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
        l /= 2
        r /= 2
    res

for i in V:
    let hv = h[i]
    seg_add(id_of[hv], hv)
    for q in 0..pq_x[i].len:
        let gt = seg_min(id_of[pq_x[i][q]] + 1, U - 1)
        ans = (ans + (pq_m[i][q] % MOD) * (gt % MOD)) % MOD

for i in 0..2 * st_sz: seg[i] = INF
for i in V.rev:
    let hv = h[i]
    seg_add(id_of[hv], hv)
    for q in 0..sq_x[i].len:
        let gt = seg_min(id_of[sq_x[i][q]] + 1, U - 1)
        ans = (ans + (sq_m[i][q] % MOD) * (gt % MOD)) % MOD

print(ans)
