// Source: Codeforces 2124G (Maximise Sum), https://codeforces.com/contest/2124/problem/G
// limits: 4000 ms, 512 MB
// Problem: an array a of n <= 1e6 values in 0..=n (sum n <= 1e6, multitest). At most once pick
//   i < j, set a[i] += a[j] and a[j] = 0, at cost j - i (0 for no move). For each x in 0..n print
//   the max over moves of cost >= x of the sum of prefix minima.
// Solution: only a prefix-minimum i can gain and only a strict suffix maximum j is worth
//   zeroing. Record each candidate's total at ans[j - i] and print suffix maxima of ans. Per
//   prefix minimum i, its block up to the next one has lifted prefix minima with prefix sums;
//   walk the suffix maxima j > i by rising a[j], getting the gain by binary search and the loss
//   from suffix sums, and stop once a[i] + a[j] reaches the cap pref_min[i - 1]. The walks
//   telescope to O(n) steps, O(n log n) in all.
// Bounds: n: 1..=1_000_000; a: 0..=2_000_000


@!multitest
let n: 1..=1_000_000 = read()
domain Pos = 0..n
let a: [Pos]0..=2_000_000 = read()

let pm = a.scan(min)
let pref_min = [Pos] i => pm.cut(i.ord + 1)
let os = pref_min.rev.scan(+)
let orig_suff = [Pos] p => os[p.l]
let base_ans = orig_suff[Pos.first!]

// Strict suffix maxima in reverse order; `suffix_max_pos.last!` is the global max.
let sm = a.rev.scan(max)
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
    let lifted = [Blk] t => lm.cut(t.ord + 1)
    let ls = lifted.rev.scan(+)
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

let sa = ans.rev.scan(max)
print([sa.cut(i) for i in 0..n])
