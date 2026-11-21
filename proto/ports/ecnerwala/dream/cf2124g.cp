// Codeforces 2124 G: Maximise Sum.
// For each k in 0..n, maximise sum_t pref_min[t] after at most one operation
// choosing i < pos with cost `pos - i >= k` (setting a[i] += a[pos], a[pos] = 0).
// Storing each operation's total at `ans[pos - i]` and taking `suffix_scan(max)`
// propagates every cost >= k down to k.
//
// Invariants:
// - Only a prefix-minimum position `i` (`a[i] == pref_min[i]`) can gain: any
//   non-prefix-minimum is already capped by an earlier smaller element, so
//   raising `a[i]` changes no prefix minimum.
// - Only a strict suffix maximum `pos` (`a[pos] > max(a[pos + 1..])`) can be
//   an optimal removal: if `pos' > pos` has `a[pos'] >= a[pos]`, removing `pos'`
//   yields a larger distance `pos' - i`, at least as large an addition, and no
//   larger a suffix loss (`orig_suff[pos'] <= orig_suff[pos]`).
// - Raising `a[i]` lifts prefix minima on `blk = a[i..end_cut]` (to the next
//   prefix minimum) to `min(lifted, cur_cap)` where `cur_cap = min(cap, a[i] + a[pos])`
//   and `cap = pref_min[i - 1]` (uncapped at `i = 0`). Because `lifted` is
//   non-increasing on `blk`, its partition cut `split` separates elements capped
//   at `cur_cap` from the uncapped tail.
// - Complexity: scanning `suffix_max_pos` in reverse (decreasing `pos`,
//   increasing `a[pos]`), once `ai + a[pos] >= cap` the first such `pos` has
//   the smallest suffix loss `orig_suff[pos]` and largest distance `pos - i`,
//   dominating all earlier `pos'` — so the inner loop stops there. Since `cap`
//   is strictly increasing as `i` moves left, each `pos` triggers at most one
//   uncapped hit plus the iterations where `a[pos] < cap - ai`, giving O(n log n)
//   total work (amortized with the monotone split).

use std.seq.suffix_scan

@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]Int = read()

let pref_min = a.scan(min)
let orig_suff = pref_min.suffix_sums()
let base_ans = orig_suff[Pos.cuts.first]

// Strict suffix maxima in reverse order; `suffix_max_pos.last!` is the global max.
let suff_max = a.suffix_scan(max)
let suffix_max_pos = [p for p in Pos.rev if p == Pos.last or a[p] > suff_max[p.next!]]
let top_val = a[suffix_max_pos.last!]

var ans: [0..n]Int = base_ans
var end_cut = Pos.cuts.last

for i in Pos.rev if pref_min[i] == a[i]:
    let ai = a[i]
    // Uncapped at Pos.first: use ai + top_val so no candidate exceeds cap.
    let cap = (i.prev.map(p => pref_min[p])) ?? (ai + top_val)
    let blk = a[i.l..end_cut]
    let lifted = ([cap] + blk[1..]).scan(min)
    let lifted_suff = lifted.suffix_sums()
    let block_loss = lifted.suffix_sums(from: orig_suff[end_cut])

    for pos in suffix_max_pos if pos > i:
        let cur_cap = min(cap, ai + a[pos])
        let split = blk.cuts.last(c => lifted[c.prev!] > cur_cap) ?? blk.cuts.first
        let gain = (cur_cap - ai) * split.count_before + lifted_suff[split] - ai * split.count_after
        let loss = if pos.l < end_cut: block_loss[pos] else: orig_suff[pos]
        ans[pos - i] = max(ans[pos - i], base_ans + gain - loss)
        if ai + a[pos] >= cap: break

    end_cut = i.l

print(ans.suffix_scan(max))
