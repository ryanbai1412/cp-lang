// Codeforces 2124 B — Minimise Sum
//
// One operation adds `a[j]` to `a[i]` (`i < j`) and sets `a[j] = 0`, zeroing
// all prefix minima from `j` onwards. Since `a[0]` can never decrease and
// `min(a[0], a[1])` only decreases if `a[1]` is zeroed into `a[0]` (yielding
// `a[0] + a[1] >= a[0] + min(a[0], a[1])`), the prefix-min sum is always at
// least `a[0] + min(a[0], a[1])`. Operating on `(0, 1)` when `a[1] <= a[0]` or
// `(1, 2)` when `a[0] < a[1]` (or doing nothing if `n == 2`) achieves this bound.

@!multitest
let n: 2..=200_000 = read()
let a: [0..n]0..=2 * n = read()
print(a[0] + min(a[0], a[1]))
