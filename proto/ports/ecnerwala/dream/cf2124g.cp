// Codeforces 2124 G: Maximise Sum.
// For each k in 0..n, maximise sum_t pref_min[t] after at most one operation
// choosing i < pos with cost `pos - i >= k` (setting a[i] += a[pos], a[pos] = 0).
// Recording each operation's total at `ans[pos - i]` and taking `suffix_maxs()`
// propagates every higher-cost solution down to every smaller k.
//
// Only a prefix-minimum position i benefits: lifting a[i] raises prefix minima
// on the block `blk = a[i..end_cut]` (up to the next prefix minimum) toward
// `cur_cap = min(cap, a[i] + a[pos])`, where `cap = pref_min[i - 1]` (uncapped
// at `i = 0`). Zeroing a[pos] drops the suffix from pos onward to 0, losing
// `block_loss[pos - i]` if `pos < end_cut`, or `orig_suff[pos]` otherwise.
// Only suffix maxima (strictly increasing value from the right) can be optimal
// removals. Because `lifted` is non-increasing on `blk`, `split` partitions
// `blk` into `lifted[..split] > cur_cap` (each gaining `cur_cap - a[i]`) and
// `lifted[split..]` (gaining `lifted_suff[split] - a[i] * (blk.len - split)`).

use std.seq.suffix_scan

@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]Int = read()

let pref_min = a.scan(min)
let orig_suff = pref_min.suffix_sums()
let base_ans = orig_suff[Pos.bounds.first]

var ans: [0..n]Int = base_ans
var cands: List<(val: Int, pos: Pos)> = []
var end_cut = Pos.bounds.last

for i in Pos.rev:
    let ai = a[i]
    if pref_min[i] == ai and not cands.is_empty:
        let cap = if i == Pos.first: ai + cands.last!.val else: pref_min[(i - 1)!]
        let blk = a[i.l..end_cut]
        let lifted = ([cap] + blk[1..]).scan(min)
        let lifted_suff = lifted.suffix_sums()
        let block_loss = lifted.suffix_sums(from: orig_suff[end_cut])

        for (v_add, pos) in cands:
            let cur_cap = min(cap, ai + v_add)
            let split = bsearch_last(0..=blk.len, k => k == 0 or lifted[k - 1] > cur_cap)!
            let capped_gain = (cur_cap - ai) * split
            let tail_gain = lifted_suff[split] - ai * (blk.len - split)
            let loss = if pos.l < end_cut: block_loss[pos - i] else: orig_suff[pos.l]
            ans[pos - i] max= base_ans + capped_gain + tail_gain - loss
            if cur_cap >= cap: break
        end_cut = i.l
    if cands.is_empty or ai > cands.last!.val:
        cands.push((ai, i))

print(ans.suffix_scan(max))
