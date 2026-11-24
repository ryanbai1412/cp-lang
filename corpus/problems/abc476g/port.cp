// Source: AtCoder ABC476 G (Increasing Popcount),
//   https://atcoder.jp/contests/abc476/tasks/abc476_g
// limits: 2000 ms, 1024 MB
// Problem: given 1 <= L <= R <= 1e18, give each integer i in [L, R] a value A_i so that equal
//   values at i < j need popcount(i) < popcount(j); print the least possible max A.
//   Multitest, T <= 1e4.
// Solution: each value is a chain of strictly rising popcount, so by Dilworth the answer is
//   the longest subsequence of L..R with non-increasing popcount. f[k][a][b]: the longest
//   such subsequence in [0, 2^k) starting at popcount <= a and ending at >= b, built from its
//   two halves in O(60^4) once. [L, R + 1) splits into O(60) aligned blocks (prefix ones o,
//   height k); a DP over them keeps dp[p], the best so far ending at popcount >= p,
//   O(60^3) per block. `@alt(program)` is the direct O(R - L) scan.
// Bounds: l: 1..=1e18; r: l..=1e18

const MAXB = 60
// f[k, a, b]: the longest non-increasing-popcount subsequence of [0, 2^k) that starts at
// popcount <= a and ends at popcount >= b (b <= a <= k); c is where the low half ends.
var f: [0..=MAXB, 0..=MAXB, 0..=MAXB]Int = 0
f[0, 0, 0] = 1
for k in 1..=MAXB:
    for a in 0..=k:
        for b in 0..=a:
            f[k, a, b] = max(
                (if c > k - 1: 0 else: f[k - 1, min(a, k - 1), c])
                    + (if c == 0 or b - 1 > k - 1: 0 else: f[k - 1, min(c - 1, k - 1), max(0, b - 1)])
                for c in b..=a
            )!

@!multitest
let l: 1..=1e18 = read()
let r: l..=1e18 = read()

// [l, r + 1) as aligned blocks, left to right: `ones` set bits above a free low `height`
var blocks: List<(ones: Int, height: Int)> = []
var lo = l
let hi = r + 1
while lo < hi:
    var k = 0
    while k < MAXB and not lo.bit(k) and lo + (2 << k) <= hi:
        k += 1
    blocks.push((ones: popcount(lo >> k), height: k))
    lo += 1 << k

// dp[p]: the longest subsequence so far that ends at popcount >= p
var dp: [0..=MAXB]Int = 0
for (o, k) in blocks:
    var next = dp.clone()
    for b in 0..=min(MAXB, o + k):
        for a in max(b, o)..=min(MAXB, o + k):
            next[b] max= dp[a] + f[k, a - o, max(0, b - o)]
    for b in (0..MAXB).rev:
        next[b] max= next[b + 1]
    dp = next

print(dp[0])

@alt(program)
fn slow():
    var best: [0..=MAXB]Int = 0
    for x in l..=r:
        let p = popcount(x)
        best[p] = max(best[q] for q in p..=MAXB)! + 1
    print(best.max()!)
