// Codeforces 2178 I: Numbers or Fireworks, tuned.
// port.cp's sum, with the dp for each `out` on a dense table over the
// popcount(out) bits of `out` (as orig.cpp keeps it) instead of on submasks of
// `out` scattered across a table over all of A, the neighbour sets as plain
// bit masks, and each bit's sum over subsets walked block by block instead of
// testing the bit of every index.
type Z = Mod<998_244_353>

@!multitest
let (n, k): (2..=31, 1..=20_000) = read()
domain V = 0..n
let p: [V]Pt = read()
fn near(u: V, v: V) -> Bool = (p[u] - p[v]).norm2 == k
let side = two_color<V>(u => [v for v in V if near(u, v)])!
let left = V.count(v => side[v]) * 2 <= n
let a = [v for v in V if side[v] == left]
let b = [v for v in V if side[v] != left]
let s0 = a.len
fn nbr(v: V) -> Int = sum(1 << i for i in 0..s0 if near(a[i], v))
let adj = [nbr(v) for v in b]

var ans: Z = 0
for out in 0..1 << s0:
    let full = 1 << popcount(out)
    var dp: [0..full]Z = 1
    var nd: [0..full]Z = 0
    for m in adj:
        let w = Z(popcount(m & ~out) + 1)
        unchecked:
            for x in 0..full: nd[x] = dp[x] * w
        var bit = 0
        for i in 0..s0 if out >> i & 1 == 1:
            if m >> i & 1 == 1:
                let step = 1 << bit
                var base = 0
                while base < full:
                    unchecked:
                        for x in base..base + step: dp[x + step] += dp[x]
                    base += 2 * step
            bit += 1
        unchecked:
            for x in 0..full: dp[x] += nd[x]
    ans += dp[full - 1]
print(ans - 1)
