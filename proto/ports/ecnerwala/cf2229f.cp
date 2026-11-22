// Codeforces 2229 F — Maximum Minimum Block Sum
//
// Remove the largest element `fin = a_in[best_i]` (which is always optimal to
// place as the final standalone gain), leaving `m = n - 1` elements `a`.
//
// The optimal threshold `thresh` for partitioning `a` into `>= k` disjoint
// blocks each of sum `>= thresh` must equal one of the `2^m` subset sums of
// `a`. Precompute all subset sums in sorted order and binary search for the
// largest feasible `thresh`, verified by bitmask DP `dp[mask] = (cnt, rem)`
// maximizing completed blocks `cnt` and then current partial-block sum `rem`.

use std.seq.{argmax}

@!multitest
let (n, k): (1..=21, 1..=21) = read()
let a_in: [0..n]1..=10 ** 9 = read()
let best_i = a_in.argmax()!
let fin = a_in[best_i]
let a = [a_in[i] for i in 0..n if i != best_i]
let m = a.len
let total_masks = 1 << m

var sums: [0..total_masks]Int = 0
for i in 0..m:
    let bit = 1 << i
    for j in 0..bit: sums[bit + j] = sums[j] + a[i]
let sorted_sums = sums.sorted()
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

let ans_idx = bsearch_last(0..total_masks, idx => can_reach(sorted_sums[idx])) ?? 0
print(sorted_sums[ans_idx] + fin)
