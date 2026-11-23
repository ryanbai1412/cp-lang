// Codeforces 2124 G: Maximise Sum.
// For each k in 0..n, maximise sum_t pref_min[t] after at most one operation
// choosing i < pos with cost `pos - i >= k` (setting a[i] += a[pos], a[pos] = 0).
// Storing each operation's total at `ans[pos - i]` and taking `suffix_fold(max, ...)`
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
// - Raising `a[i]` lifts prefix minima on the block `[i, next_min)` (up to the
//   next prefix minimum) to `min(lifted, cur_cap)` where `cur_cap = min(cap, a[i] + a[pos])`
//   and `cap = pref_min[i - 1]` (uncapped at `i = 0`). Because `lifted` is
//   non-increasing on the block, `first_uncapped` splits capped elements from
//   the uncapped tail.
// - Complexity proof: scanning `suffix_max_pos` in reverse (decreasing `pos`,
//   strictly increasing `a[pos]`), once `ai + a[pos] >= cap` the first such
//   `pos` has the smallest suffix loss `orig_suff[pos]` and largest distance
//   `pos - i`, dominating all earlier `pos'` — so the inner loop breaks there.
//   At `i = Pos.first` there are at most `n` iterations. For every later prefix
//   minimum `i > Pos.first`, there is at most one breaking iteration plus the
//   non-breaking iterations with `0 <= a[pos] < cap - ai`; because `a[pos]` is
//   strictly increasing along `suffix_max_pos`, at most `cap - ai` suffix
//   maxima satisfy `a[pos] < cap - ai`. Summing `cap - ai = pref_min[i - 1] - pref_min[i]`
//   over all prefix minima telescopes to `a[0] - pref_min[n - 1] <= 2n`, so
//   the inner loop runs O(n) times total (`O(n log n)` with binary search).

use std.prefix.{suffix_fold}

@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]0..=2_000_000 = read()

let pm = a.scan(min)
let pref_min = [Pos] i => pm.prefix(i.ord + 1)
let os = suffix_fold(+, pref_min)
let orig_suff = [Pos] p => os[p.l]
let base_ans = orig_suff[Pos.first!]

// Strict suffix maxima in reverse order; `suffix_max_pos.last!` is the global max.
let sm = suffix_fold(max, a)
let suff_max = [Pos] p => sm[p.l]
let suffix_max_pos = [p for p in Pos.rev if p == Pos.last or a[p] > suff_max[p.next!]]
let top_val = a[suffix_max_pos.last!]

var ans: [0..n]Int = base_ans
var next_min: ?Pos = None

for i in Pos.rev if pref_min[i] == a[i]:
    let ai = a[i]
    // Uncapped at Pos.first: use ai + top_val so no candidate exceeds cap.
    let cap = (i.prev.map(p => pref_min[p])) ?? (ai + top_val)
    let blk_end = next_min.map(j => j.ord) ?? n
    let blk_len = blk_end - i.ord
    let tail_loss = next_min.map(j => orig_suff[j]) ?? 0
    domain Blk = 0..blk_len
    let lm = ([Blk] t => if t == Blk.first: cap else: a[i.offset(t.ord)!]).scan(min)
    let lifted = [Blk] t => lm.prefix(t.ord + 1)
    let ls = suffix_fold(+, lifted)
    let lifted_suff = [Blk] t => ls[t.l]

    for pos in suffix_max_pos if pos > i:
        let cur_cap = min(cap, ai + a[pos])
        let last_capped = Blk.range.last_true(t => lifted[t] > cur_cap)
        let first_uncapped = if let s = last_capped: s.next else: Blk.first
        let capped_cnt = if let t = first_uncapped: t.ord else: blk_len
        let uncapped_gain = if let t = first_uncapped: lifted_suff[t] - ai * (blk_len - t.ord) else: 0
        let gain = (cur_cap - ai) * capped_cnt + uncapped_gain
        let loss = if pos.ord < blk_end: lifted_suff[Blk(pos - i)] + tail_loss else: orig_suff[pos]
        ans[pos - i] max= base_ans + gain - loss
        if ai + a[pos] >= cap: break

    next_min = Some(i)

let sa = suffix_fold(max, ans)
print([sa[i] for i in 0..n])
