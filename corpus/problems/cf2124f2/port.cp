// Source: Codeforces 2124F2 (Appending Permutations, hard),
//   https://codeforces.com/contest/2124/problem/F2
// limits: 3000 ms, 1024 MB
// Problem: build an array of length n by appending cyclic shifts [r, ..., s, 1, ..., r - 1] of
//   1..=s; m restrictions a_i != x. Count distinct valid arrays mod 998244353. Multitest,
//   sum n <= 5000, sum m <= 5000.
// Solution: dp over cuts c. A block from c is a rising run x0 + 1, x0 + 2, ... on diagonal
//   pos - val = c - x0 up to a cut j, then the wrap 1..=x0 on diagonal j; a run is valid up to
//   its diagonal's first forbidden cell (sorted per diagonal). One difference-array sweep per x0
//   range-adds dp[c] over the allowed j and is read back when the scan reaches j. A 1-starting
//   run [c..j) of length L = j - c followed by a shorter 1-starting run [j..k) (k - j <= L - 1)
//   valid on diagonal j (k <= wrap_limit(j)) is the same array as one cyclic shift, so another
//   sweep subtracts it. O(n^2) time and memory.
// Bounds: n: 1..=5000; m: 0..=5000; pos, val: 1..=n

use std.prefix.SweepAdd

type Z = Mod<998_244_353>

@!multitest
let n: 1..=5000 = read()
let m: 0..=5000 = read()
domain Cut = 0..=n
domain Diag = 1 - n..n

fn diag_of(c: Cut, offset: Int) -> Diag = Diag(c - offset)

var bads: [Diag]List<Cut> = []
repeat m:
    let (pos, val): (1..=n, 1..=n) = read()
    let c = Cut(pos - 1)
    bads[diag_of(c, val - 1)].push(c)
for d in Diag: bads[d].sort(by: c => desc(c))

// The first forbidden cut at or after `from` on `d`; `from` never decreases per diagonal, so
// the passed cuts are popped off the back.
fn next_bad(d: Diag, from: Cut) -> Cut:
    while let b = bads[d].last, b < from: bads[d].pop()
    if let b = bads[d].last: b else: Cut.last!
fn wrap_limit(c: Cut) -> Cut = if c < Cut.last!: next_bad(diag_of(c, 0), from: c) else: Cut.last!

var dp: [Cut]Z = 0
dp[Cut.first!] = 1
var overcount = SweepAdd<Cut, Z>()
var wrap: [0..n]SweepAdd<Cut, Z> = SweepAdd<Cut, Z>()

for c in Cut:
    dp[c] -= overcount.advance(c)
    let wlim = wrap_limit(c)
    for x0 in 0..n:
        let v = wrap[x0].advance(c)
        if let t = c.next(x0), t <= wlim: dp[t] += v
    if c < Cut.last!:
        for j in c.after.clip(..=wlim):
            overcount.add(j.after.take(j - c - 1).clip(..=wrap_limit(j)), dp[c])
        for x0 in 0..n:
            let rise = next_bad(diag_of(c, x0), from: c)
            wrap[x0].add(c.after.clip(..=min(rise, Cut(n - x0))), dp[c])

print(dp[Cut.last!])
