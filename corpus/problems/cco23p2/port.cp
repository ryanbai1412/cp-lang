// Source: CCO 2023 Day 1 P2, Real Mountains, https://dmoj.ca/problem/cco23p2
// Problem: heights h_1..h_N (N <= 1e6, h_i <= 1e9). An edit picks i < j < k with
//   h_i > h_j < h_k, costs h_i + h_j + h_k and increments h_j. Print the minimum total cost
//   to make h unimodal (nondecreasing, then nonincreasing), mod 1e6 + 3.
// Solution: column i must end at cap_i = min(prefix max, suffix max). Sweep the compressed
//   heights upwards, keeping the columns still below their cap in a SortedBag. Lifting them
//   from level x costs x per column plus the least heights above x left of the first and right
//   of the last (one column), or (2sz - 3)(x + 1) plus those and the least above x overall
//   (sz >= 2). Level sums use x(x+1)/2; the least-above-x terms are deferred to a prefix and a
//   suffix sweep of successor queries on a SortedBag. O(N log N).
// original: corpus/submissions/5579928.cpp

type Z = Mod<1_000_003>

let n: 1..=1_000_000 = read()
domain V = 0..n
let h: [V]Int = read()

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

let (domain H, hs) = compress([h[i] for i in V])
var by_h: [H]List<V> = []
for i in V: by_h[hs.at(h[i])].push(i)

fn tri(x: Int) -> Z = Z(x) * Z(x + 1) * Z(500_002)
fn sum_range(l: Int, r: Int) -> Z = tri(r) - tri(l - 1)

var pq: [V]List<(Int, Int)> = []
var sq: [V]List<(Int, Int)> = []
var active = SortedBag<Int>()
var ans: Z = 0
var prev_h: ?Int = None

for cid in H:
    let cur_h = hs[cid]
    if let lo_h = prev_h:
        if not active.is_empty:
            let first = V(active.first!)
            let last = V(active.last!)
            let span = cur_h - lo_h
            let sz = active.len
            if sz == 1:
                ans += sum_range(lo_h, cur_h - 1)
                pq[first].push((lo_h, span))
                sq[first].push((lo_h, span))
            else:
                ans += Z(sz) * sum_range(lo_h, cur_h - 1) + Z(sz * 2 - 3) * sum_range(
                    lo_h + 1,
                    cur_h,
                )
                pq[first].push((lo_h, span))
                sq[last].push((lo_h, span))
                pq[V.last!].push((lo_h, span))
    for v in by_h[cid]: active.add(v.idx)
    while let lo = active.first:
        if cur_h < cap[V(lo)]: break
        active.pop_first()
    while let hi = active.last:
        if cur_h < cap[V(hi)]: break
        active.pop_last()
    prev_h = cur_h

var seen = SortedBag<Int>()
for i in V:
    seen.add(h[i])
    for (x, m) in pq[i]: ans += Z(m) * Z(seen.range(x + 1..).first!)

seen.clear()
for i in V.rev:
    seen.add(h[i])
    for (x, m) in sq[i]: ans += Z(m) * Z(seen.range(x + 1..).first!)

print(ans)
