// Source: Codeforces 2124F2 (Appending Permutations, hard),
//   https://codeforces.com/contest/2124/problem/F2
// Problem: build an array of length n by appending cyclic shifts [r, ..., s, 1, ..., r - 1] of
//   1..=s; m restrictions a_i != x. Count distinct valid arrays mod 998244353. Multitest,
//   sum n <= 5000, sum m <= 5000.
// Solution: dp over cuts c. A block from c is a rising run x0 + 1, x0 + 2, ... on diagonal
//   pos - val = c - x0 up to a cut j, then the wrap 1..=x0 on diagonal j; a run is valid up to
//   its diagonal's first forbidden cell (sorted per diagonal). One difference-array sweep per x0
//   range-adds dp[c] over the allowed j and is read back when the scan reaches j. A 1-starting
//   run of length L followed by a shorter 1-starting run valid on diagonal j is the same array
//   as one cyclic shift, so another sweep subtracts it. O(n^2) time and memory.

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
