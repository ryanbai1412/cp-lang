// CCO '23 P2 — Real Mountains (CP Lang 0.2.2): `compress`, `Mod<1_000_003>`, and
// `SortedBag` (`first`/`last`/`first_gt`) replace manual compression, `% MOD`, and
// a 30-line hand-rolled segment tree with `INF` sentinel.
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

let hs = compress([h[i] for i in V])
type H = hs.D
var by_h: [H]List<V> = []
for i in V: by_h[hs.at(h[i])].push(i)

fn tri(x: Int) -> Z = Z(x) * Z(x + 1) * Z(500_002)
fn sum_range(l: Int, r: Int) -> Z = tri(r) - tri(l - 1)

var pq: [V]List<(Int, Int)> = []
var sq: [V]List<(Int, Int)> = []
var active = SortedBag<Int>()
var ans: Z = 0
var prev_h: ?Int = None

for cid in hs.D:
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
                pq[V.last].push((lo_h, span))
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
