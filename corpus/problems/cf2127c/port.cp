// Codeforces 2127 C — Trip Shopping
//
// Each index `i` represents the interval `(lo: min(a[i], b[i]), hi: max(a[i], b[i]))`
// of initial contribution `hi - lo`. When Ali picks two intervals `p, q`
// (`p.lo <= q.lo`) and Bahamin re-pairs their four endpoints to maximize the
// sum of lengths, overlapping intervals gain `0`, while disjoint intervals
// (`p.hi < q.lo`) merge across the gap and gain `2 * (q.lo - p.hi)`.
//
// Once any two intervals overlap, Ali can re-pick them for 0 gain on every
// remaining turn, so `_k` (the number of rounds >= 1) does not affect the
// answer: Ali simply picks the closest pair once, which must be adjacent after
// sorting `segs` by `(lo, hi)`.

@!multitest
let (n, _k): (2..=200_000, 1..=200_000) = read()
let a: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
let segs = sorted([(lo: min(a[i], b[i]), hi: max(a[i], b[i])) for i in 0..n])
let tot = sum(p.hi - p.lo for p in segs)
let min_loss = segs.adjacent.map((p, q) => max(0, 2 * (q.lo - p.hi))).min()!
print(tot + min_loss)
