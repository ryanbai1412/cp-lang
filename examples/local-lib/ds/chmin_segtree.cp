//! Segment tree beats: range `min=` with range sums and maxima.
use std.seq.{ends}

/// An array over `D` that supports `chmin(r, x)` (every value in `r` becomes `min(value, x)`)
/// together with range sums and range maxima. Each node keeps its maximum, how many times the
/// maximum occurs and the strict second maximum, so a `chmin` that only lowers the maximum stops
/// early. Amortized O(log^2 n) per operation, O(n) memory.
pub struct ChminSegTree<D>:
    top: List<Int>
    second: List<?Int>
    count: List<Int>
    total: List<Int>

    pub fn new(a: [D]Int) -> ChminSegTree<D>:
        let k = 4 * max(D.K, 1)
        var s = ChminSegTree<D>(
            List.filled(k, 0),
            List.filled(k, none),
            List.filled(k, 0),
            List.filled(k, 0),
        )
        if D.K > 0: s.build(1, 0, D.K, a)
        s

    /// Every value in `r` becomes `min(value, x)`.
    pub fn chmin(mut self, r: Range<D>, x: Int):
        let (lo, hi) = r.ends
        if lo < hi: self.cap(1, 0, D.K, lo, hi, x)

    /// The sum of the values in `r`.
    pub fn sum(self, r: Range<D>) -> Int:
        let (lo, hi) = r.ends
        if lo < hi: self.fold_sum(1, 0, D.K, lo, hi, top[1]) else: 0

    /// The largest value in `r`, or `none` when `r` is empty.
    pub fn max_in(self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo < hi: self.fold_max(1, 0, D.K, lo, hi, top[1]) else: none

    fn build(mut self, p: Int, l: Int, r: Int, a: [D]Int):
        if r - l == 1:
            let v = a[D.nth(l)]
            top[p] = v
            count[p] = 1
            total[p] = v
            return
        let mid = (l + r) / 2
        self.build(2 * p, l, mid, a)
        self.build(2 * p + 1, mid, r, a)
        self.pull(p)

    fn pull(mut self, p: Int):
        let (a, b) = (2 * p, 2 * p + 1)
        total[p] = total[a] + total[b]
        if top[a] == top[b]:
            top[p] = top[a]
            count[p] = count[a] + count[b]
            second[p] = max(second[a], second[b])
        elif top[a] > top[b]:
            top[p] = top[a]
            count[p] = count[a]
            second[p] = max(second[a], top[b])
        else:
            top[p] = top[b]
            count[p] = count[b]
            second[p] = max(top[a], second[b])

    /// Lower the maximum of node `p` to `x`, which must still be above its second maximum.
    fn lower(mut self, p: Int, x: Int):
        if top[p] <= x: return
        total[p] -= (top[p] - x) * count[p]
        top[p] = x

    fn push(mut self, p: Int):
        self.lower(2 * p, top[p])
        self.lower(2 * p + 1, top[p])

    fn cap(mut self, p: Int, l: Int, r: Int, lo: Int, hi: Int, x: Int):
        if hi <= l or r <= lo or top[p] <= x: return
        if lo <= l and r <= hi and (second[p] == none or second[p]! < x):
            self.lower(p, x)
            return
        self.push(p)
        let mid = (l + r) / 2
        self.cap(2 * p, l, mid, lo, hi, x)
        self.cap(2 * p + 1, mid, r, lo, hi, x)
        self.pull(p)

    // A query never pushes: `cap` is the smallest maximum above `p`, and only values equal to
    // `top[p]` can exceed it, so the node's sum and maximum are corrected on the way down.
    fn fold_sum(self, p: Int, l: Int, r: Int, lo: Int, hi: Int, cap: Int) -> Int:
        if hi <= l or r <= lo: return 0
        let t = min(top[p], cap)
        if lo <= l and r <= hi: return total[p] - (top[p] - t) * count[p]
        let mid = (l + r) / 2
        self.fold_sum(2 * p, l, mid, lo, hi, t) + self.fold_sum(2 * p + 1, mid, r, lo, hi, t)

    fn fold_max(self, p: Int, l: Int, r: Int, lo: Int, hi: Int, cap: Int) -> ?Int:
        if hi <= l or r <= lo: return none
        let t = min(top[p], cap)
        if lo <= l and r <= hi: return t
        let mid = (l + r) / 2
        max(self.fold_max(2 * p, l, mid, lo, hi, t), self.fold_max(2 * p + 1, mid, r, lo, hi, t))

@test
fn matches_a_plain_array():
    domain P = 0..9
    for _ in 0..40:
        var plain: [P]Int = [P] _ => rand(-20, 20)
        var st = ChminSegTree<P>(plain)
        for _ in 0..80:
            let i = rand(0, 8)
            let j = rand(i, 8)
            let r = P(i)..=P(j)
            if rand(0, 1) == 0:
                let x = rand(-25, 25)
                st.chmin(r, x)
                for k in i..=j: plain[P(k)] min= x
            assert st.sum(r) == sum(plain[P(k)] for k in i..=j)
            assert st.max_in(r) == (i..=j).map(k => plain[P(k)]).max()
