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

type Z = Mod<998_244_353>

struct SweepAdd:
    var cur: Z
    diff: List<Z>

    fn advance(mut self, i: Int) -> Z:
        self.cur += self.diff[i]
        self.cur

    fn add(mut self, lo: Int, hi: Int, v: Z):
        if lo <= hi:
            self.diff[lo] += v
            self.diff[hi + 1] -= v

@!multitest
let n: 1..=5000 = read()
let m: 0..=5000 = read()
domain Cut = 0..=n
domain Diag = 1 - n..n

fn diag_of(c: Cut, offset: Int) -> Diag:
    let ci = c.ord
    Diag(ci - offset)

var bads: [Diag]List<Cut> = []
repeat m:
    let (pos, val): (1..=n, 1..=n) = read()
    let c = Cut(pos - 1)
    bads[diag_of(c, val - 1)].push(c)
for d in Diag: bads[d].sort()

fn next_bad(d: Diag, from: Cut) -> Cut:
    if let k = bads[d].first_ge(from): bads[d][k] else: Cut.last!
fn wrap_limit(c: Cut) -> Cut = if c < Cut.last!: next_bad(diag_of(c, 0), from: c) else: Cut.last!

var dp: [Cut]Z = 0
dp[Cut.first!] = 1
var overcount = SweepAdd(Z(0), [Z(0) for _ in 0..n + 2])
var wrap = [0..n] _ => SweepAdd(Z(0), [Z(0) for _ in 0..n + 2])

for c in Cut:
    let i = c.ord
    dp[c] -= overcount.advance(i)
    let wlim = wrap_limit(c)
    for x0 in 0..n:
        let v = wrap[x0].advance(i)
        if let t = Cut.at(i + x0) and t <= wlim: dp[t] += v
    if c < Cut.last!:
        for j in i + 2..=wlim.ord:
            let lim = min(2 * j - i - 1, wrap_limit(Cut(j)).ord)
            overcount.add(j + 1, lim, dp[c])
        for x0 in 0..n:
            let j_lim = min(n - x0, next_bad(diag_of(c, x0), from: c).ord)
            wrap[x0].add(i + 1, j_lim, dp[c])

print(dp[Cut.last!])
