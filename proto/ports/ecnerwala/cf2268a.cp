// Codeforces 2268 A — Middle Block and End Pairs
//
// Positions outside the middle block `[k - 1..n - k + 1)` pair symmetrically
// from both ends `(a[i], a[n - 1 - i])`, each pair contributing `max(...)`.
// The middle block is non-empty iff `k - 1 <= n - k + 1`, i.e. `2 * k - 2 <= n`
// (otherwise `mid = 0` and only the `edge = min(k - 1, n - k + 1, n / 2)`
// end pairs exist).

@!multitest
let (n, k): (1..=200_000, 1..=n) = read()
let a: [0..n]1..=10 ** 9 = read()
let mid = if 2 * k - 2 <= n: a[k - 1..n - k + 1].sum() else: 0
let edge = min(k - 1, n - k + 1, n / 2)
print(mid + sum(max(a[i], a[n - 1 - i]) for i in 0..edge))
