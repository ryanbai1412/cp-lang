// Codeforces 2187 A — Restricted Sorting
//
// Swaps are allowed between values `u, v` with `|u - v| >= k`. Any misplaced
// element `x` can be routed to its sorted slot via the global minimum `lo` or
// global maximum `hi` iff `k <= max(x - lo, hi - x)`, because `lo` and `hi`
// maximize the distance from `x` (so if `x` cannot swap with either extreme,
// it cannot move at all). Thus the maximum feasible `k` is the minimum of
// `max(x - lo, hi - x)` over all misplaced elements (`j != i` in sorted
// `(value, index)` order), or `-1` when `a` is already sorted.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let s = sorted([(a[i], i) for i in 0..n])
let (lo, hi) = (s[0].0, s.last!.0)
print(s.items.filter((i, (x, j)) => j != i).map((i, (x, j)) => max(x - lo, hi - x)).min() ?? -1)
