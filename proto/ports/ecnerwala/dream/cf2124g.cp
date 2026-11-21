// Codeforces 2124 G: Maximise Sum (dream).
// One operation picks i < pos and sets a[i] += a[pos], a[pos] = 0 (cost pos - i).
// Only a prefix-minimum position i benefits: lifting a[i] raises the prefix
// minima on [i, j) (up to the next prefix minimum j) toward min(cap, a[i] + v),
// where cap = pref_min[i - 1], while zeroing a[pos] drops the suffix from pos
// onward to 0 (losing lifted[pos..j] + orig_pref_min[j..n]). Only suffix-maxima
// (val, pos) can be optimal removal candidates; as val rises, the split k in
// [i, j] where lifted[t] <= cur_cap sweeps left monotonically.
@!multitest
let n: 1..=1e6 = read()
domain Pos = 0..n
let a: [Pos]Int = read()
let pref_min = a.prefix_mins()
let base_ans = pref_min.sum()

var ans: [0..n]Int = base_ans
var rem_cands: List<(val: Int, pos: Pos)> = []    // suffix maxima (strictly increasing val)
var lifted: [Pos]Int = pref_min
var suff_loss: [Pos.bounds]Int = 0
var j = Pos.bounds.last                           // start of the next prefix-min block

for i in Pos.rev:
    let ai = a[i]
    if pref_min[i] == ai:
        let cap = if i == Pos.first: ai + rem_cands.last!.val else: pref_min[i.prev!]
        // Lifted prefix minima on [i, j) when a[i] is raised to `cap`, plus suffix loss.
        lifted[i..j] = a[i..j].with_first(cap).prefix_mins()
        suff_loss[i..j] = lifted[i..j].suffix_sums(from: suff_loss[j])

        var k = j
        var tail_gain = 0
        for (v_add, pos) in rem_cands:
            let cur_cap = min(cap, ai + v_add)
            while k > i.l and lifted[k.prev!] <= cur_cap:
                k = k.prev!
                tail_gain += lifted[k] - ai
            let capped_gain = (cur_cap - ai) * (k - i.l)
            ans[pos - i] max= base_ans + capped_gain + tail_gain - suff_loss[pos.l]
            if cur_cap >= cap: break

        // Restore original prefix-min (= ai) suffix sum on [i, j) for earlier blocks.
        suff_loss[i..j] = ([i..j] _ => ai).suffix_sums(from: suff_loss[j])
        j = i.l
    if rem_cands.is_empty or ai > rem_cands.last!.val:
        rem_cands.push((ai, i))

print(ans.suffix_maxs())
