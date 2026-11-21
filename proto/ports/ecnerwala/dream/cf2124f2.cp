// Codeforces 2124 F2: Appending Permutations (hard version).
// Each appended block is a cyclic shift of 1..k: a rising run starting at
// x0 + 1 on diagonal (pos - val = c - x0), followed by a wrap run 1..=x0 on
// diagonal (pos - val = j). Both runs stay valid up to the next forbidden cell
// on their diagonal (`wrap_limit`).
//
// Sweeping the cut c in `Cut = 0..=n` accumulates `dp[c]` into online
// range-add sweeps per wrap length `x0`, and subtracts the double-counted
// split where a 1-starting run `[c..j)` of length L = j - c >= 2 is followed
// by a shorter 1-starting run `[j..k)` of length <= L - 1 (so k <= 2j - c - 1),
// which is already counted as a single cyclic shift.

use std.mod.{Mod}
use std.prefix.SweepAdd

type Z = Mod<998_244_353>

@!multitest
let n: 1..=5000 = read()
let m: 0..=5000 = read()
domain Cut = 0..=n
domain Diag = 1 - n..n

var bads: [Diag]List<Cut> = []
repeat m:
    let (pos, val): (1..=n, 1..=n) = read()
    bads[Diag(pos - val)].push(Cut(pos - 1))
for d in Diag: bads[d].sort(desc: true)

fn next_bad(d: Diag) -> ?Cut = bads[d].last
fn wrap_limit(c: Cut) -> Cut = if c < Cut.last: next_bad(Diag(c.ord)) ?? Cut.last else: Cut.last

var dp: [Cut]Z = 0
dp[Cut.first] = 1
var overcount = SweepAdd<Cut, Z>()
var wrap: [0..n]SweepAdd<Cut, Z> = SweepAdd<Cut, Z>()

for c in Cut:
    dp[c] -= overcount.advance(c)
    let wlim = wrap_limit(c)
    for x0 in 0..n:
        let v = wrap[x0].advance(c)
        if c.ord + x0 <= wlim.ord: dp[Cut(c.ord + x0)] += v
    if c < Cut.last:
        for j in c.ord + 2..=wlim.ord:
            let lim = min(2 * j - c.ord - 1, wrap_limit(Cut(j)).ord)
            overcount.add(Cut(j + 1)..=Cut(lim), dp[c])
        for x0 in 0..n:
            let d = Diag(c.ord - x0)
            bads[d].pop_while(p => p < c)
            let j_lim = min(n - x0, (next_bad(d) ?? Cut.last).ord)
            wrap[x0].add(Cut(c.ord + 1)..=Cut(j_lim), dp[c])

print(dp[Cut.last])
