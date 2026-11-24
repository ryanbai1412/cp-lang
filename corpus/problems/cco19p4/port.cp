// Source: CCO 2019 P4 (Card Scoring), https://dmoj.ca/problem/cco19p4
// limits: 2750 ms, 1024 MB
// Problem: n <= 1e6 cards with values in 1..=n, drawn in order; keep or discard each, the
//   hand holds one value only, and scoring a hand of x cards gives x^(k/2), 2 <= k <= 4.
//   Print the maximum total score (relative error 1e-6).
// Solution: k = 2 scores n (every card alone). Otherwise dp[i] = best from card i on =
//   max over later cards j of the same value of score(#value in i..=j) + dp[j + 1]. Going
//   right to left, each value keeps a Li Chao tree over its occurrence ranks of the curves
//   score(m - x) + dp[j + 1] (score is convex, so two cross once). O(n log n).
// original: corpus/submissions/3617807.cpp
let k: Int = read()
let n: Int = read()

if k == 2:
    print(F64(n))
else:
    domain Card = 0..n
    domain Rank = 0..n
    domain Diff = 0..=n
    let raw: [Card]Int = read()
    let a: [Card]Int = [Card] i => raw[i] - 1

    fn score_of(d: Int) -> F64:
        if k == 4:
            return F64(d) * F64(d)
        let s = sqrt(F64(d))
        s * s * s

    let score: [Diff]F64 = [Diff] d => score_of(d.ord)

    var cnt: [Rank]Int = [Rank] _ => 0
    for i in Card: cnt[Rank(a[i])] += 1

    // Flat per-rank Li Chao trees stored in one offset-indexed arena.
    var off: [Rank]Int = [Rank] _ => 0
    var tsz: [Rank]Int = [Rank] _ => 0
    var tot = 0
    for r in Rank:
        let sz = cnt[r] + 1
        off[r] = tot
        tsz[r] = sz
        tot += 4 * sz + 2

    var has_c: List<Bool> = []
    var cm: List<Int> = []
    var cb: List<F64> = []
    for _ in 0..tot:
        has_c.push(false)
        cm.push(0)
        cb.push(0.0)

    fn eval_c(m: Int, b: F64, x: Int) -> F64:
        if x > m: -1e300 else: score[Diff(m - x)] + b

    fn lc_insert(base: Int, o: Int, l: Int, r: Int, m0: Int, b0: F64):
        let idx = base + o
        if not has_c[idx]:
            has_c[idx] = true
            cm[idx] = m0
            cb[idx] = b0
            return
        var cur_m = cm[idx]
        var cur_b = cb[idx]
        var nm = m0
        var nb = b0
        let mid = (l + r) / 2
        let lv = eval_c(nm, nb, l) > eval_c(cur_m, cur_b, l)
        let mv = eval_c(nm, nb, mid) > eval_c(cur_m, cur_b, mid)
        if mv:
            cm[idx] = nm
            cb[idx] = nb
            nm = cur_m
            nb = cur_b
        if l < r:
            if lv != mv or mid > nm:
                lc_insert(base, o * 2, l, mid, nm, nb)
            else:
                lc_insert(base, o * 2 + 1, mid + 1, r, nm, nb)

    fn lc_query(base: Int, sz: Int, x: Int) -> F64:
        var o = 1
        var l = 0
        var r = sz - 1
        var best = -1e300
        while true:
            let idx = base + o
            if has_c[idx]:
                let v = eval_c(cm[idx], cb[idx], x)
                best max= v
            if l == r: return best
            let mid = (l + r) / 2
            if x <= mid:
                o *= 2
                r = mid
            else:
                o = o * 2 + 1
                l = mid + 1

    var dp: [Diff]F64 = [Diff] _ => 0.0
    var i = n - 1
    while i >= 0:
        let r = Rank(a[Card(i)])
        cnt[r] -= 1
        lc_insert(off[r], 1, 0, tsz[r] - 1, cnt[r] + 1, dp[Diff(i + 1)])
        dp[Diff(i)] = lc_query(off[r], tsz[r], cnt[r])
        i -= 1
    print(dp[Diff(0)])
