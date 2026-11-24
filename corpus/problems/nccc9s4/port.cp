// Source: Mock CCC 2022 Contest 1 S4 (Berkeley Math Tournament Statistics),
//   https://dmoj.ca/problem/nccc9s4
// limits: 250 ms, 32 MB
// Problem: n <= 1e5 scores s_i in 1..=n and q <= 1e5 queries l r: print the most frequent score
// Solution: rollback Mo's. Queries shorter than B = 270 are counted directly. Longer ones are
//   bucketed by the block of l and sorted by r: the right part grows from the block's end
//   and is kept, while the O(B) left part is added and rolled back per query. Ties compare
//   (count, value) pairs packed in one Int. O(q B + n^2 / B).
// original: corpus/submissions/4145494.cpp
// Bounds: n, q: 1..=100_000; a, qs.l, qs.r: 1..=n
const B = 270

let (n, q): (1..=100_000, 1..=100_000) = read()
domain V = 1..=n
let a: [0..n]V = read()
domain Q = 0..q
let qs: [Q](l: 1..=n, r: 1..=n) = read()

var cnt: [V]0..=n = 0

// A best is count * n + the score's rank: max on it compares (count, score) pairs.
fn add(j: Int, best: Int) -> Int:
    let v = a[j]
    cnt[v] += 1
    max(best, cnt[v] * n + v.ord)

var ans: [Q]V = V.first!
var wide: [0..=n / B]List<Q> = []
for qi in Q:
    let (l, r) = (qs[qi].l - 1, qs[qi].r - 1)
    if r - l < B:
        var best = 0
        for j in l..=r: best = add(j, best)
        ans[qi] = V.nth(best % n)
        for j in l..=r: cnt[a[j]] -= 1
    else:
        wide[l / B].push(qi)

for b in 0..=n / B if not wide[b].is_empty:
    cnt.fill(0)
    let mid = (b + 1) * B
    var right = 0
    var reach = mid
    wide[b].sort(by: qi => qs[qi].r)
    for qi in wide[b]:
        let (l, r) = (qs[qi].l - 1, qs[qi].r - 1)
        for j in reach..=r: right = add(j, right)
        reach max= r + 1
        var best = right
        for j in l..mid: best = add(j, best)
        ans[qi] = V.nth(best % n)
        for j in l..mid: cnt[a[j]] -= 1

for qi in Q: print(ans[qi])
