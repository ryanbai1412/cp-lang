// Codeforces 2124 G: Maximise Sum.
// One operation picks i < pos and sets a[i] += a[pos], a[pos] = 0 (cost pos - i).
// Only a prefix-minimum position i benefits: lifting a[i] raises prefix minima
// on [i, j) (up to the next prefix minimum j) toward min(cap, a[i] + v), where
// cap = pref_min[i - 1], while zeroing a[pos] loses lifted[pos..j] + orig[j..n].
// Only suffix-maxima (val, pos) can be optimal removals; as val rises, the split
// k in [i, j] where lifted[t] <= cur_cap sweeps left monotonically.
@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]Int = read()

var pref_min: [Pos]Int = a
for i in Pos if i != Pos.first: pref_min[i] min= pref_min[(i - 1)!]

let base_ans = pref_min.sum()
var ans: [0..n]Int = base_ans
var rem_cands: List<(val: Int, pos: Pos)> = []
var lifted: [Pos]Int = pref_min
var suff_loss: [0..=n]Int = 0
var j = n

for i in Pos.rev:
    let ai = a[i]
    if pref_min[i] == ai:
        let cap = if i == Pos.first: ai + rem_cands.last!.val else: pref_min[(i - 1)!]
        // Lifted prefix minima on [i, j) when a[i] is raised to `cap`, plus suffix loss.
        for t in i.ord..j:
            lifted[Pos(t)] = if t == i.ord: cap else: min(lifted[Pos(t - 1)], a[Pos(t)])
        for t in (i.ord..j).rev:
            suff_loss[t] = suff_loss[t + 1] + lifted[Pos(t)]

        var k = j
        var tail_gain = 0
        for (v_add, pos) in rem_cands:
            let cur_cap = min(cap, ai + v_add)
            while k > i.ord and lifted[Pos(k - 1)] <= cur_cap:
                k -= 1
                tail_gain += lifted[Pos(k)] - ai
            let capped_gain = (cur_cap - ai) * (k - i.ord)
            ans[pos.ord - i.ord] max= base_ans + capped_gain + tail_gain - suff_loss[pos.ord]
            if cur_cap >= cap: break

        // Restore original prefix-min (= ai) suffix sum on [i, j) for earlier blocks.
        for t in (i.ord..j).rev:
            suff_loss[t] = suff_loss[t + 1] + ai
        j = i.ord
    if rem_cands.is_empty or ai > rem_cands.last!.val:
        rem_cands.push((ai, i))

for d in (1..n).rev: ans[d - 1] max= ans[d]
print(ans)
