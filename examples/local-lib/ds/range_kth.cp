//! The k-th smallest value in any range of a fixed array, by a persistent segment tree
//! over the sorted values: O(n log n) to build, O(log n) per query.

/// Answers `kth` and `count_below` on ranges of the array it was built from.
/// Memory is about `n * (log2(n) + 2)` nodes of three `Int`s.
pub struct RangeKth<D>:
    /// The distinct values, increasing; a leaf of the tree is one of them.
    vals: List<Int>
    left: List<Int>
    right: List<Int>
    cnt: List<Int>
    /// `root[i]` is the tree of the first `i` positions.
    root: List<Int>

    pub fn new(a: [D]Int) -> RangeKth<D>:
        let vals = sorted([a[i] for i in D])
        var uniq: List<Int> = []
        for v in vals:
            if uniq.is_empty or uniq.last! != v: uniq.push(v)
        var t = RangeKth<D>(uniq, [0], [0], [0], [0])
        for i in D:
            let rank = t.vals.first_ge(a[i])!
            t.root.push(t.insert(t.root[t.root.len - 1], 0, t.vals.len, rank))
        t

    /// The `k`-th smallest value (from 0) among the positions of `r`, or `none` if the
    /// range holds `k` values or fewer.
    pub fn kth(self, r: Range<D>, k: Int) -> ?Int:
        var (a, b) = (root[r.l.ord], root[r.r.ord])
        if k < 0 or k >= cnt[b] - cnt[a]: return none
        var (lo, hi, left_over) = (0, vals.len, k)
        while hi - lo > 1:
            let mid = (lo + hi) / 2
            let in_left = cnt[left[b]] - cnt[left[a]]
            if left_over < in_left:
                (a, b, hi) = (left[a], left[b], mid)
            else:
                left_over -= in_left
                (a, b, lo) = (right[a], right[b], mid)
        vals[lo]

    /// How many positions of `r` hold a value below `x`.
    pub fn count_below(self, r: Range<D>, x: Int) -> Int:
        // the values below x are the ranks 0..upto
        let upto = vals.first_ge(x) ?? vals.len
        var (a, b) = (root[r.l.ord], root[r.r.ord])
        var (lo, hi, total) = (0, vals.len, 0)
        while hi - lo > 1 and upto > lo:
            if upto >= hi: break
            let mid = (lo + hi) / 2
            if upto <= mid:
                (a, b, hi) = (left[a], left[b], mid)
            else:
                total += cnt[left[b]] - cnt[left[a]]
                (a, b, lo) = (right[a], right[b], mid)
        if upto >= hi: total += cnt[b] - cnt[a]
        total

    /// A copy of the tree `node` (covering ranks `lo..hi`) with one more value of rank `rank`.
    fn insert(mut self, node: Int, lo: Int, hi: Int, rank: Int) -> Int:
        let fresh = cnt.len
        left.push(left[node])
        right.push(right[node])
        cnt.push(cnt[node] + 1)
        if hi - lo > 1:
            let mid = (lo + hi) / 2
            if rank < mid:
                let child = self.insert(left[node], lo, mid, rank)
                left[fresh] = child
            else:
                let child = self.insert(right[node], mid, hi, rank)
                right[fresh] = child
        fresh

@test
fn matches_sorting_each_range():
    domain D = 0..17
    for _ in 0..15:
        let a = [D] i => rand(-6, 6)
        let t = RangeKth<D>(a)
        for _ in 0..40:
            let (x, y) = (D.nth(rand(0, 16)), D.nth(rand(0, 16)))
            let r: Range<D> = if rand(0, 6) == 0: x.l..x.l else: min(x, y)..=max(x, y)
            let part = sorted([a[i] for i in r])
            for k in -1..=part.len:
                assert t.kth(r, k) == (if 0 <= k < part.len: part[k] else: none)
            let bound = rand(-8, 8)
            assert t.count_below(r, bound) == part.count(v => v < bound)

@test
fn median_of_a_window():
    domain D = 1..=7
    let a = [D] i => [5, 1, 9, 3, 7, 2, 8][i.to_idx]
    let t = RangeKth<D>(a)
    assert t.kth(D(2)..=D(6), 2) == 3 and t.kth(D(1)..=D(7), 3) == 5
    assert t.count_below(D(1)..=D(4), 5) == 2
