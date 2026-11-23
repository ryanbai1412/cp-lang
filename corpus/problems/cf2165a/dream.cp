// Codeforces 2165 A — Cyclic Merging
//
// Merging two adjacent elements `(x, y)` on the ring removes the smaller one
// and pays `max(x, y)`. Merging elements in non-decreasing order eliminates
// each non-maximum element into the smaller of its two remaining neighbors, so
// every cyclic edge `(x, y)` pays `max(x, y)` exactly once except one edge
// incident to the global maximum `a.max()`, which stays unmerged at the end.

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()
print(sum(max(x, y) for (x, y) in cyclic_pairs(a)) - a.max()!)
