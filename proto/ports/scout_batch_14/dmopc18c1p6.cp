// DMOPC 2018 C1 P6 Sorting: flip the fewest bits in n binary strings of length
// m so that the strings are in non-decreasing order. Knuth-optimized interval DP.
// original: corpus/submissions/4155900.cpp
let n: 1..=1000 = read()
let m: 1..=50 = read()
domain Row = 0..n
domain Col = 0..m
let grid: [Row, Col]Char = read()

// Column prefix sums of '1' bits: ones[c, r] = count of '1' in rows 0..r of col c.
var ones: [Col, 0..=n]Int = 0
for c in Col:
    for r in Row:
        ones[c, r.idx + 1] = ones[c, r.idx] + (if grid[r, c] == '1': 1 else: 0)

fn flip_cost(c: Col, l: Int, r: Int, want_one: Bool) -> Int:
    let cnt = ones[c, r + 1] - ones[c, l]
    if want_one: (r - l + 1) - cnt else: cnt

// dp[l, r]: min flips for rows l..=r on columns col..m; opt[l, r] is the split
// k in (l-1)..=r where rows l..=k get '0' and k+1..=r get '1'.
type Cost = [0..n, 0..n]Int
var prev_dp: Cost = 0
var opt: Cost = 0

for c in Col.rev:
    var cur_dp: Cost = 0
    for span in (1..n).rev:
        for l in 0..n - span:
            let r = l + span
            let lo = if l > 0: max(l - 1, opt[l - 1, r]) else: l - 1
            let hi = if r + 1 < n: min(r, opt[l, r + 1]) else: r
            let (best, best_k) = min_by(lo..=hi, k => (           // WISH P13
                if k == l - 1: flip_cost(c, l, r, true) + prev_dp[l, r]
                elif k == r: flip_cost(c, l, r, false) + prev_dp[l, r]     // WISH P9
                else: prev_dp[l, k] + prev_dp[k + 1, r]
                    + flip_cost(c, l, k, false) + flip_cost(c, k + 1, r, true)
            ))
            cur_dp[l, r] = best
            opt[l, r] = best_k
    prev_dp = cur_dp

print(prev_dp[0, n - 1])
