// Codeforces 2124 G: Maximise Sum.
// For each k in 0..n, maximise sum_t pref_min[t] after at most one operation
// choosing i < pos with cost `pos - i >= k` (setting a[i] += a[pos], a[pos] = 0).
// Recording each operation's total at `ans[pos - i]` and taking a suffix max
// propagates every higher-cost solution down to every smaller k.
//
// Only a prefix-minimum position i benefits: lifting a[i] raises prefix minima
// on the block [i, j) (up to the next prefix minimum j) toward
// `cur_cap = min(cap, a[i] + a[pos])`, where `cap = pref_min[i - 1]` (uncapped
// at `i = 0`). Zeroing a[pos] drops the suffix from pos onward to 0, losing
// `block_loss[pos]` if `pos < j`, or `orig_suff[pos]` otherwise. Only suffix
// maxima (strictly increasing value from the right) can be optimal removals;
// as their value rises, the split in [i, j] where `lifted[t] <= cur_cap`
// sweeps left monotonically.

use std.seq.scan

@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]Int = read()

let pref_min = scan(a, (x, y) => min(x, y))
let orig_pref = prefix(pref_min)
let base_ans = orig_pref.sum(Pos.first..=Pos.last)

var ans: [0..n]Int = base_ans
var cands: List<(val: Int, pos: Pos)> = []
var lifted: [Pos]Int = pref_min
var block_loss: [0..=n]Int = 0
var j = n

for i in Pos.rev:
    let ai = a[i]
    if pref_min[i] == ai:
        if not cands.is_empty:
            let cap = if i == Pos.first: ai + cands.last!.val else: pref_min[(i - 1)!]
            for t in i.ord..j:
                lifted[Pos(t)] = if t == i.ord: cap else: min(lifted[Pos(t - 1)], a[Pos(t)])
            block_loss[j] = if j < n: orig_pref.sum(Pos(j)..=Pos.last) else: 0
            for t in (i.ord..j).rev:
                block_loss[t] = block_loss[t + 1] + lifted[Pos(t)]

            var split = j
            var tail_gain = 0
            for (v_add, pos) in cands:
                let cur_cap = min(cap, ai + v_add)
                while split > i.ord and lifted[Pos(split - 1)] <= cur_cap:
                    split -= 1
                    tail_gain += lifted[Pos(split)] - ai
                let capped_gain = (cur_cap - ai) * (split - i.ord)
                let loss = if pos.ord < j: block_loss[pos.ord] else: orig_pref.sum(pos..=Pos.last)
                ans[pos.ord - i.ord] max= base_ans + capped_gain + tail_gain - loss
                if cur_cap >= cap: break
        j = i.ord
    if cands.is_empty or ai > cands.last!.val:
        cands.push((ai, i))

for d in (1..n).rev: ans[d - 1] max= ans[d]
print(ans)
