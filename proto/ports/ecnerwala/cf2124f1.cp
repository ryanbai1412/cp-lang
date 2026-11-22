// Codeforces 2124 F1 — Cyclic-Block Partition DP (Easy Version)
//
// Group forbidden pairs `(pos, val)` by diagonal `diag(pos, val) = pos - val + n - 1`
// (sorted descending so `next_bad(off) = bads[off].last ?? n` is the earliest
// forbidden position `>= i`).
//
// `dp[i]` counts valid partitions of `0..i` into cyclic-shift blocks of
// `1..=len` (each avoiding all forbidden `(pos, val)` pairs). From `i`:
// - Add every valid cyclic block `[i..j + x0)` shifted by `x0` (starting at
//   `x0 + 1` on `[i..j)` and wrapping to `1..=x0` on `[j..j + x0)`).
// - Subtract `dp[i]` for adjacent identity blocks `[i..j)` + `[j..end)` with
//   `end - j < j - i` that would otherwise be double-counted as a single
//   unshifted + shifted split.

@!multitest
type Z = Mod<998_244_353>
let (n, m): (1..=100, 0..=10_000) = read()
fn diag(pos: Int, val: Int) -> Int: pos - val + n - 1

var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..m:
    let (i, x): (1..=n, 1..=n) = read()
    bads[diag(i, x)].push(i - 1)
for off in 0..2 * n - 1: bads[off].sort(desc: true)
fn next_bad(off: Int) -> Int: bads[off].last ?? n

var dp: [0..=n]Z = 0
dp[0] = 1
for i in 0..n:
    let j_max0 = next_bad(diag(i, 0))
    for j in i + 2..=j_max0:
        var end_max = min(n, 2 * j - i - 1)
        if j < n: end_max min= next_bad(diag(j, 0))
        for end in j + 1..=end_max: dp[end] -= dp[i]

    for off in i..=diag(i, 0):
        let x0 = diag(i, 0) - off
        while let last = bads[off].last and last < i: bads[off].pop()
        let j_max = min(n - x0, next_bad(off))
        for j in i + 1..=j_max if j == n or next_bad(diag(j, 0)) >= j + x0:
            dp[j + x0] += dp[i]

print(dp[n])
