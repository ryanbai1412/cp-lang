// Source: Mock CCC 2022 Contest 1 S4 (Berkeley Math Tournament Statistics),
//   https://dmoj.ca/problem/nccc9s4
// Problem: n <= 1e5 scores s_i in 1..=n and q <= 1e5 queries l r: print the most frequent score
//   among students l..=r, the largest one on ties.
// Solution: with at most HEAVY = 800 distinct values, a prefix count per value answers each query
//   in O(1) per value: O(800 (n + q)). Otherwise rollback Mo's: queries shorter than B = 270 are
//   counted directly; longer ones are bucketed by the block of l and sorted by r, the right end
//   grows from the block boundary and the O(B) left part is added and rolled back per query.
//   Ties compare (count, value) pairs. O(q B + n^2 / B).
// original: corpus/submissions/4145494.cpp
// Bounds: n, q: 1..=100_000; a, qs.l, qs.r: 1..=n
const B = 270
const HEAVY = 800

let (n, q): (1..=100_000, 1..=100_000) = read()
let a: [0..n](1..=n) = read()
domain Q = 0..q
let qs: [Q](l: 1..=n, r: 1..=n) = read()

let (domain C, vals) = compress(a)
let id: [0..n]C = [0..n] i => vals.index_of(a[i])
var ans: [Q]Int = 0

if C.count <= HEAVY:
    var best: [Q](Int, C) = (0, C.first!)
    for v in C:
        let pre = prefix([0..n] i => Int(id[i] == v))
        for qi in Q:
            let (l, r) = qs[qi]
            best[qi] max= (pre.fold((l - 1)..r), v)
    for qi in Q: ans[qi] = vals[best[qi].1]
else:
    let nb = n / B + 1
    var buckets: [0..=nb]List<(l: Int, r: Int, qi: Q)> = []
    var cnt: [C]Int = 0
    for qi in Q:
        let (l, r) = (qs[qi].l - 1, qs[qi].r - 1)
        if r - l + 1 <= B:
            var cur = (0, C.first!)
            for j in l..=r:
                let v = id[j]
                cnt[v] += 1
                cur max= (cnt[v], v)
            ans[qi] = vals[cur.1]
            for j in l..=r: cnt[id[j]] -= 1
        else:
            buckets[ceil_div(l, B)].push((l:, r:, qi:))

    for bi in 0..=nb if not buckets[bi].is_empty:
        buckets[bi].sort(by: e => e.r)
        cnt.fill(0)
        var base = (0, C.first!)
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
