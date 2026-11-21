// Codeforces 2124 F2: Appending Permutations (hard version).
// Each appended block is a cyclic shift of 1..k: a rising run starting at
// x0 + 1 on diagonal (pos - val = c - x0), followed by a wrap run 1..=x0
// starting at cut `j`: placing value `t` (1 <= t <= x0) at 1-based position
// `j + t` fixes `pos - val = j`, so the wrap run's validity limit is the first
// forbidden cell on diagonal `j` (`wrap_limit(j)`).
//
// Sweeping the cut c in `Cut = 0..=n` accumulates `dp[c]` into online
// range-add sweeps per wrap length `x0`, and subtracts the double-counted
// split where a 1-starting run `[c..j)` of length L = j - c >= 2 is followed
// by a shorter 1-starting run `[j..k)` of length <= L - 1 (so k <= 2j - c - 1)
// that is itself valid on diagonal `j` (so k <= wrap_limit(j)), which is
// already counted as a single cyclic shift.

use std.mod.{Mod}
use std.prefix.Diff

type Z = Mod<998_244_353>

@!multitest
let n: 1..=5000 = read()
let m: 0..=5000 = read()
domain Cut = 0..=n
domain Diag = 1 - n..n

fn diag_of(c: Cut, offset: Int) -> Diag = Diag(c.ord - offset)

var bads: [Diag]List<Cut> = []
repeat m:
    let (pos, val): (1..=n, 1..=n) = read()
    let c = Cut(pos - 1)
    bads[diag_of(c, val - 1)].push(c)
for d in Diag: bads[d].sort()

fn next_bad(d: Diag, from: Cut) -> Cut = bads[d].first_ge(from) ?? Cut.last
fn wrap_limit(c: Cut) -> Cut = if c < Cut.last: next_bad(diag_of(c, 0), from: c) else: Cut.last

var dp: [Cut]Z = 0
dp[Cut.first] = 1
var overcount = Diff<Cut, Z>()
var wrap: [0..n]Diff<Cut, Z> = Diff<Cut, Z>()

for c in Cut:
    dp[c] -= overcount.advance(c)
    let wlim = wrap_limit(c)
    for x0 in 0..n:
        let v = wrap[x0].advance(c)
        if let t = c + x0, t <= wlim: dp[t] += v
    if c < Cut.last:
        for j in c + 2..=wlim:
            let lim = min(j + (j - c) - 1, wrap_limit(j))
            overcount.add(j + 1..=lim, dp[c])
        for x0 in 0..n:
            let j_lim = min(Cut.last - x0, next_bad(diag_of(c, x0), from: c))
            wrap[x0].add(c + 1..=j_lim, dp[c])

print(dp[Cut.last])
