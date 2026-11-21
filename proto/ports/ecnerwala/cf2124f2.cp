// Codeforces 2124 F2: Appending Permutations (hard version).
// Each appended block is a cyclic shift of 1..k: a rising run starting at
// x0 + 1 on diagonal (pos - val = i - x0), followed by a wrap run 1..=x0 on
// diagonal (pos - val = j). Both runs stay valid up to the next forbidden cell
// on their diagonal. Sweeping the cut j accumulates dp[i] into range-add
// sweeps per wrap length x0, and subtracts the double-counted split where a
// 1-starting run of length >= 2 is followed by a shorter 1-starting run.
type Z = Mod<998_244_353>

@!multitest
let n: 1..=5000 = read()
let m: 0..=5000 = read()
domain Diag = 1 - n..n

var bads: [Diag]List<Int> = []
repeat m:
    let (i, x): (1..=n, 1..=n) = read()
    bads[Diag(i - x)].push(i - 1)
for d in Diag:
    bads[d].sort(by: p => -p)

fn next_bad(d: Diag) -> Int = bads[d].last ?? n

var dp: [0..=n]Z = 0
dp[0] = 1
var over_cur: Z = 0
var over_diff: [0..n + 2]Z = 0
var wrap_cur: [0..n]Z = 0
var wrap_diff: [0..n, 0..n + 2]Z = 0

fn add_over(lo: Int, hi: Int, v: Z):
    if lo <= hi:
        over_diff[lo] += v
        over_diff[hi + 1] -= v

fn add_wrap(x0: Int, lo: Int, hi: Int, v: Z):
    if lo <= hi:
        wrap_diff[x0, lo] += v
        wrap_diff[x0, hi + 1] -= v

for i in 0..=n:
    over_cur += over_diff[i]
    dp[i] -= over_cur
    let wrap_lim = if i < n: next_bad(Diag(i)) else: n
    for x0 in 0..n:
        wrap_cur[x0] += wrap_diff[x0, i]
        if i + x0 <= wrap_lim: dp[i + x0] += wrap_cur[x0]
    if i == n: break

    for j in i + 2..=wrap_lim:
        let lim = min(2 * j - i - 1, if j < n: next_bad(Diag(j)) else: n)
        add_over(j + 1, lim, dp[i])

    for x0 in 0..n:
        let d = Diag(i - x0)
        while let last = bads[d].last, last < i: bads[d].pop()
        let j_lim = min(n - x0, next_bad(d))
        add_wrap(x0, i + 1, j_lim, dp[i])

print(dp[n])
