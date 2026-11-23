// Source: Codeforces 2229F (Load Unbalancing), https://codeforces.com/contest/2229/problem/F
// Problem: n <= 18 numbers a_i <= 1e9 and k <= n. Reorder them, then add each in turn to a
//   minimum of k bins (initially 0); print the largest possible final maximum bin.
//   Multitest, sum of 2^n <= 2^18.
// Solution: put the maximum fin last (an exchange never loses), so the answer is fin plus the
//   largest thresh such that the other m = n - 1 numbers can be ordered to fill every bin to
//   at least thresh; thresh is a subset sum, so binary search (`last_true`) over the sorted
//   `subset_sums`. can_reach(thresh) is a mask DP keeping the lexicographically largest
//   (closed blocks, running remainder), a block closing once it reaches thresh: O(m 2^m) per
//   check, O(m^2 2^m) total.

use std.dp.{subset_sums}
use std.seq.{argmax}

@!multitest
let n: 1..=18 = read()
let k: 1..=n = read()
let a_in: [0..n]1..=10 ** 9 = read()
let best_i = a_in.argmax()!
let fin = a_in[best_i]
let a = [a_in[i] for i in 0..n if i != best_i]
let m = a.len
let total_masks = 1 << m

let sorted_sums = subset_sums(a)
let total_sum = sorted_sums.last!

fn can_reach(thresh: Int) -> Bool:
    if thresh > total_sum / k: return false
    var dp: [0..total_masks](Int, Int) = (0, 0)
    for mask in 1..total_masks:
        for i in 0..m if mask.bit(i):
            let (cnt, rem) = dp[mask - (1 << i)]
            let nrem = rem + a[i]
            dp[mask] max= if nrem >= thresh: (cnt + 1, 0) else: (cnt, nrem)
    dp[total_masks - 1].0 >= k

let ans_idx = (0..total_masks).last_true(idx => can_reach(sorted_sums[idx])) ?? 0
print(sorted_sums[ans_idx] + fin)
