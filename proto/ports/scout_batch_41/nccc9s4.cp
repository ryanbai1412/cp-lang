// Mock CCC '22 Contest 1 S4 (nccc9s4): answer q range-mode queries on a[1..=n],
// breaking ties by largest value. Uses per-value prefix sums when there are at
// most 800 distinct values and rollback Mo's algorithm otherwise.
// original: corpus/submissions/4145494.cpp
const B = 270
const HEAVY = 800

let (n, q): (1..=100_000, 1..=100_000) = read()
let a: [0..n](1..=n) = read()
domain Q = 0..q
let qs: [Q](l: 1..=n, r: 1..=n) = read()

let vals = compress(a)
let id: [0..n]vals.D = [0..n] i => vals.at(a[i])
var ans: [Q]Int = 0

if vals.D.K <= HEAVY:
    var best: [Q](Int, vals.D) = (0, vals.D.first)
    for v in vals.D:
        let pre = prefix([0..n] i => if id[i] == v: 1 else: 0)
        for qi in Q:
            let (l, r) = qs[qi]
            best[qi] max= (pre.sum((l - 1)..r), v)
    for qi in Q: ans[qi] = vals[best[qi].1]
else:
    let nb = n / B + 1
    var buckets: [0..=nb]List<(l: Int, r: Int, qi: Q)> = []
    var cnt: [vals.D]Int = 0
    for qi in Q:
        let (l, r) = (qs[qi].l - 1, qs[qi].r - 1)
        if r - l + 1 <= B:
            var cur = (0, vals.D.first)
            for j in l..=r:
                let v = id[j]
                cnt[v] += 1
                cur max= (cnt[v], v)
            ans[qi] = vals[cur.1]
            for j in l..=r: cnt[id[j]] -= 1
        else:
            buckets[(l + B - 1) / B].push((l: l, r: r, qi: qi))

    for bi in 0..=nb if not buckets[bi].is_empty:
        sort(mut buckets[bi], by: e => e.r)
        cnt.fill(0)
        var base = (0, vals.D.first)
        let mid = bi * B
        var cr = mid - 1
        for (l, r, qi) in buckets[bi]:
            while cr < r:
                cr += 1
                let v = id[cr]
                cnt[v] += 1
                base max= (cnt[v], v)
            var cur = base
            for j in l..mid:
                let v = id[j]
                cnt[v] += 1
                cur max= (cnt[v], v)
            ans[qi] = vals[cur.1]
            for j in l..mid: cnt[id[j]] -= 1

for qi in Q: print(ans[qi])
