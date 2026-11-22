// Codeforces 2229 F — Maximum Minimum Block Sum
//
// Exchange argument for the final element: any valid arrangement contributes
// `last + thresh`, where `last` is the final standalone element and `thresh`
// is the minimum block sum across `k` disjoint blocks formed from the other
// `m = n - 1` elements. Swapping the global maximum `fin = a_in[best_i]` into
// the final slot increases `last` by `fin - last` while decreasing at most one
// block's sum by `fin - last`, so `last + thresh` can never decrease.
//
// The optimal threshold `thresh` must be one of the `2^m` subset sums of the
// remaining `a`, computed in sorted order via `std.dp.subset_sums(a)` in
// O(2^m). Binary searching over `sorted_sums` runs at most `m` calls to
// `can_reach` (each O(m * 2^m) bitmask DP `dp[mask] = (cnt, rem)`, pruned
// immediately when `thresh > total_sum / k`), for O(m^2 * 2^m) worst-case time.

use std.dp.{subset_sums}
use std.seq.{argmax}

@!multitest
let (n, k): (1..=21, 1..=21) = read()
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

let ans_idx = bsearch_last(0..total_masks, idx => can_reach(sorted_sums[idx])) ?? 0
print(sorted_sums[ans_idx] + fin)
