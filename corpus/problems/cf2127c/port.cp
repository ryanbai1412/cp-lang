// Source: Codeforces 2127C (Trip Shopping), https://codeforces.com/contest/2127/problem/C
// limits: 2000 ms, 256 MB
// Problem: arrays a, b of n <= 2e5 values in 1..=1e9 and k <= n rounds; each round Ali picks
//   i < j, then Bahamin rearranges a_i, a_j, b_i, b_j at will. Ali minimises and Bahamin maximises
//   v = sum |a_i - b_i|; print the final v. Multitest, sum of n <= 2e5.
// Solution: index i is the interval [lo, hi] of length hi - lo. Re-pairing two overlapping
//   intervals gains nothing; disjoint ones gain 2 (q.lo - p.hi). Ali picks the cheapest pair once,
//   then re-picks an overlapping pair at no cost, so k does not matter: sort the intervals, and
//   the answer is the total length plus the least max(0, 2 (q.lo - p.hi)) over adjacent pairs.
//   O(n log n).

@!multitest
let (n, _k): (2..=200_000, 1..=200_000) = read()
let a: [0..n]1..=10 ** 9 = read()
let b: [0..n]1..=10 ** 9 = read()
let segs = sorted([(lo: min(a[i], b[i]), hi: max(a[i], b[i])) for i in 0..n])
let tot = sum(p.hi - p.lo for p in segs)
let min_loss = segs.adjacent.map((p, q) => max(0, 2 * (q.lo - p.hi))).min()!
print(tot + min_loss)
