// Codeforces 2237 F — Length-M Block Alignment DP
//
// Maximize the number of kept elements of `a` (so `n - max_kept` is the minimum
// number of edits) that belong to valid `1..=m` length-m blocks:
// - If `a[i]` is kept as the `a[i]`-th element of a block, that block must
//   start at `block_start = i - (a[i] - 1)` in `0..=n - m`.
// - `dp_use[i]` = max kept elements ending at kept index `i`.
// - `dp1[i]` = prefix max of `dp_use[j]` over `j < i` (seeded at `dp1[0] = 0`
//   for starting a first block at any `block_start >= 0`).
// - `dp_m[i]` = prefix max of `dp_use[j]` over blocks finishing at or before
//   `i` (seeded at `dp_m[m] = 0` for skipping an initial length-m region).
// - `prv[st]` = last index `p < i` with the same `block_start == st`, allowing
//   `i` to extend the same block (`dp_use[p] + 1`).
//
// `none` means no elements kept yet (0 kept), so the answer is
// `n - max(dp1[n - m] ?? 0, dp_m[n] ?? 0)`.

@!multitest
let (n, m): (1..=200_000, 1..=200_000) = read()
let a: [0..n]1..=m = read()
var dp1: [0..=n]?Int = none
var dp_m: [0..=n]?Int = none
var dp_use: [0..=n]?Int = none
var prv: [0..=n - m]?Int = none

dp1[0] = 0
dp_m[m] = 0
for i in 0..n:
    dp1[i + 1] max= dp1[i]
    dp_m[i + 1] max= dp_m[i]
    let block_start = i - (a[i] - 1)
    if 0 <= block_start <= n - m:
        if let v = dp1[block_start]: dp_use[i] max= v + 1
        if let v = dp_m[i]: dp_use[i] max= v + 1
        if let p = prv[block_start] and let v = dp_use[p]: dp_use[i] max= v + 1
        prv[block_start] = i
        dp_m[block_start + m] max= dp_use[i]
        dp1[i + 1] max= dp_use[i]
print(n - max(dp1[n - m] ?? 0, dp_m[n] ?? 0))
