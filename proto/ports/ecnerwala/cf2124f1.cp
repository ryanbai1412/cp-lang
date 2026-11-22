// Codeforces 2124 F1 — Cyclic-Block Partition DP (Easy Version)
//
// Group forbidden pairs `(i, x)` by diagonal `off = (i - x) + n - 1` (sorted
// descending so `bads[off].last` is the earliest forbidden position `>= i`).
//
// `dp[i]` counts valid partitions of `0..i` into cyclic-shift blocks of
// `1..=k` (each avoiding all forbidden `(pos, val)` pairs). From `i`:
// - Add every valid cyclic block `[i..j + x0)` shifted by `x0` (starting at
//   `x0 + 1` on `[i..j)` and wrapping to `1..=x0` on `[j..j + x0)`).
// - Subtract `dp[i]` for adjacent identity blocks `[i..j)` + `[j..k)` with
//   `k - j < j - i` that would otherwise be double-counted as a single
//   unshifted + shifted split.

@!multitest
type Z = Mod<998_244_353>
let (n, m): (1..=100, 0..=10_000) = read()
var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..m:
    let (i, x): (1..=n, 1..=n) = read()
    bads[i - x + n - 1].push(i - 1)
for off in 0..2 * n - 1: bads[off].sort(desc: true)

var dp: [0..=n]Z = 0
dp[0] = 1
for i in 0..n:
    let j_max0 = bads[i + n - 1].last ?? n
    for j in i + 2..=j_max0:
        var k_max = min(n, 2 * j - i - 1)
        if j < n: k_max min= bads[j + n - 1].last ?? n
        for k in j + 1..=k_max: dp[k] -= dp[i]

    for off in i..=i + n - 1:
        let x0 = i + n - 1 - off
        while let last = bads[off].last and last < i: bads[off].pop()
        let j_max = min(n - x0, bads[off].last ?? n)
        for j in i + 1..=j_max if j == n or (bads[j + n - 1].last ?? n) >= j + x0:
            dp[j + x0] += dp[i]

print(dp[n])
