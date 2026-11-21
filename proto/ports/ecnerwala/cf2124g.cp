// Codeforces 2124 G: Maximise Sum.
// One operation picks i < pos and sets a[i] += a[pos], a[pos] = 0 (cost pos - i).
// Only a prefix-minimum position i benefits: lifting a[i] raises prefix minima
// on the block [i, j) (up to the next prefix minimum j) toward
// cur_cap = min(cap, a[i] + a[pos]), where cap = pref_min[i - 1] (uncapped at
// i = 0). Zeroing a[pos] drops the suffix from pos onward to 0, losing
// lifted_suff[pos..j) + orig_suff[j..n) if pos < j, or orig_suff[pos..n) if
// pos >= j. Only suffix maxima (strictly increasing value from the right) can
// be optimal removals; as their value rises, the split `split` in [i, j] where
// lifted[t] <= cur_cap sweeps left monotonically.

@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]Int = read()

var pref_min: [Pos]Int = a
for i in Pos if i != Pos.first: pref_min[i] min= pref_min[(i - 1)!]

var orig_suff: [0..=n]Int = 0
for i in Pos.rev: orig_suff[i.ord] = orig_suff[i.ord + 1] + pref_min[i]
let base_ans = orig_suff[0]

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
            block_loss[j] = orig_suff[j]
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
                let loss = if pos.ord < j: block_loss[pos.ord] else: orig_suff[pos.ord]
                ans[pos.ord - i.ord] max= base_ans + capped_gain + tail_gain - loss
                if cur_cap >= cap: break
        j = i.ord
    if cands.is_empty or ai > cands.last!.val:
        cands.push((ai, i))

for d in (1..n).rev: ans[d - 1] max= ans[d]
print(ans)
