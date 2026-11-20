// ABC476 G — Good Integer Sequence (Popcount Antichain)
// Precomputation above @!multitest + bitwise dyadic DP + @brute(main)

// F[k][a][b] = max weakly-decreasing-popcount subsequence in [0, 2^k)
// starting with popcount allowance <= a and ending with allowance >= b (0 <= b <= a <= k).
const MAXB = 60
var f: [0..=MAXB, 0..=MAXB, 0..=MAXB]Int = 0
f[0, 0, 0] = 1
for k in 1..=MAXB:
    for a in 0..=k:
        for b in 0..=a:
            var best = 0
            for c in b..=a:
                let left = if c > k - 1: 0 else: f[k - 1, min(a, k - 1), c]
                let right = if c == 0 or b - 1 > k - 1: 0 else: f[k - 1, min(c - 1, k - 1), max(0, b - 1)]
                best max= left + right
            f[k, a, b] = best

@!multitest
let l: 1..=1e18 = read()
let r: l..=1e18 = read()

// Decompose [l, r + 1) into aligned dyadic intervals (ones_in_prefix, height) in left-to-right order.
var left_blocks: List<(Int, Int)> = []
var cur_l = l
var cur_r = r + 1
while cur_l < cur_r:
    var k = 0
    while k < MAXB and (cur_l & ((1 << (k + 1)) - 1)) == 0 and cur_l + (1 << (k + 1)) <= cur_r:
        k += 1
    left_blocks.push((popcount(cur_l >> k), k))
    cur_l += 1 << k

// dp[p] = max antichain size ending with popcount allowance >= p
var dp: [0..=MAXB]Int = 0
for (o, k) in left_blocks:
    var ndp = dp.copy()
    let hi_a = min(MAXB, o + k)
    for b in 0..=hi_a:
        let lo_a = max(b, o)
        let fb = max(0, b - o)
        var best = ndp[b]
        for a in lo_a..=hi_a:
            best max= dp[a] + f[k, a - o, fb]
        ndp[b] = best
    // Ensure suffix max (dropping allowance without picking anything)
    for b in (0..MAXB).rev:
        ndp[b] max= ndp[b + 1]
    dp = ndp

print(dp[0])

@brute(main)
fn slow():
    var best: [0..=MAXB]Int = 0
    for x in l..=r:
        let p = popcount(x)
        var v = 0
        for q in p..=MAXB:
            v max= best[q]
        best[p] max= v + 1
    var ans = 0
    for q in 0..=MAXB:
        ans max= best[q]
    print(ans)
