// Codeforces 2157 D — Billion Players Game
//
// Maximize `min_{p in [l, r]} sum_i |p - a[i]|` by pairing sorted elements from
// outside in (`a[i]` with `a[n - 1 - i]`), clamping the upper endpoint to
// `max(l, a[n - 1 - i])` and the lower endpoint to `min(r, a[i])`:
// - If `a[i] <= a[n - 1 - i]` straddle `[l, r]`, the `i` term adds
//   `a[n - 1 - i] - a[i]` (their combined distance to any `p` between them) and
//   the mirrored `n - 1 - i` term clamps to `0`.
// - If both lie below `l` (or both above `r`), BOTH orientations contribute
//   their individual distances `l - a[i]` and `l - a[n - 1 - i]` (or
//   `a[n - 1 - i] - r` and `a[i] - r`) to the clamped boundary `l` (or `r`).

@!multitest
let (n, l, r): (1..=200_000, 1..=10 ** 9, 1..=10 ** 9) = read()
let raw: [0..n]1..=10 ** 9 = read()
let a = raw.sorted()
print(sum(max(0, max(l, a[n - 1 - i]) - min(r, a[i])) for i in 0..n))
