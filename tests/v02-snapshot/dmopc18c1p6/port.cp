// Source: DMOPC 2018 Contest 1 P6 (Sorting), https://dmoj.ca/problem/dmopc18c1p6
// Problem: N <= 1000 binary numbers of exactly B <= 50 bits each, in a list. Flip the fewest
//   bits so that the list is sorted in non-decreasing order; print that count.
// Solution: sorted rows that agree before column c form a block whose bit c is 0s then 1s.
//   Going from the last column to the first, dp[l, r] is the fewest flips making rows
//   l..=r sorted on columns c.. : choose a split k (rows l..=k get 0, the rest 1, cost from
//   column prefix counts) plus the next column's dp on both halves. The best split is
//   monotone (Knuth), giving O(B N^2).
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
        ones[c, r.idx + 1] = ones[c, r.idx] + Int(grid[r, c] == '1')

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
            var best = 1000000000
            var best_k = lo
            for k in lo..=hi:
                var cost = 0
                if k == l - 1:
                    cost = flip_cost(c, l, r, true) + prev_dp[l, r]
                elif k == r:
                    cost = flip_cost(c, l, r, false) + prev_dp[l, r]
                else:
                    cost = prev_dp[l, k] + prev_dp[k + 1, r] + flip_cost(c, l, k, false) + flip_cost(
                        c,
                        k + 1,
                        r,
                        true,
                    )
                if cost < best:
                    best = cost
                    best_k = k
            cur_dp[l, r] = best
            opt[l, r] = best_k
    prev_dp = cur_dp

print(prev_dp[0, n - 1])
