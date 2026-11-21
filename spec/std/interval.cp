//! Interval partition map (ODT / Chtholly tree): `IntervalMap<D, T>`.

use std.rand.{Rng}
use std.seq.{ends}

/// A total partition of domain `D` into maximal contiguous half-open runs,
/// each carrying a value of type `T`.
/// Adjacent runs always have distinct values (`==` runs coalesce automatically).
/// `new` is O(1); `from` is O(D.K); `at`, `run_at`, and `assign_point` are
/// O(log runs); `assign` is amortized O((1 + overwritten) log runs);
/// `each` is O(runs); `runs` is O(1).
pub struct IntervalMap<D, T>:
    segs: Map<Int, (hi: Int, val: T)>

    /// Partition all of `D` into one run with value `init` (empty when `D.K == 0`). O(1).
    pub fn new(init: T) -> IntervalMap<D, T>:
        var m = Map<Int, (hi: Int, val: T)>()
        if D.K > 0: m[0] = (hi: D.K, val: init)
        IntervalMap(m)

    /// Build a compressed maximal-run partition from an array `a`. O(D.K).
    pub fn from(a: [D]T) -> IntervalMap<D, T>:
        var m = Map<Int, (hi: Int, val: T)>()
        if D.K == 0: return IntervalMap(m)
        var lo = 0
        var cur = a[D.nth(0)]
        for idx in 1..D.K:
            let v = a[D.nth(idx)]
            if v != cur:
                m[lo] = (hi: idx, val: cur)
                lo = idx
                cur = v
        m[lo] = (hi: D.K, val: cur)
        IntervalMap(m)

    /// Number of maximal runs currently in the partition. O(1).
    pub @get fn runs(self) -> Int = segs.len

    /// The value at position `i`. O(log runs).
    pub fn at(self, i: D) -> T:
        let (_, seg) = segs.last_le(i.to_idx)!
        seg.val

    /// The maximal run containing position `i`, as `(range, val)`. O(log runs).
    pub fn run_at(self, i: D) -> (range: Range<D>, val: T):
        let (lo, seg) = segs.last_le(i.to_idx)!
        (range: D.nth(lo).l..D.nth(seg.hi - 1).r, val: seg.val)

    /// Paint range `r` with `val`, coalescing adjacent equal runs.
    /// Calls `on_old(range, old_val)` for every overwritten run in ascending order
    /// with zero heap allocation. Amortized O((1 + overwritten) log runs).
    pub fn assign(mut self, r: Range<D>, val: T, on_old: (Range<D>, T) -> () = (_, _) => ()):
        let (lo, hi) = r.ends
        if lo >= hi: return
        var new_lo = lo
        var new_hi = hi
        var check_left = true
        var check_right = true
        if let (sl, seg) = segs.last_le(lo):
            if seg.hi >= hi:
                on_old(r, seg.val)
                if seg.val == val: return
                if sl < lo:
                    segs[sl] = (hi: lo, val: seg.val)
                    check_left = false
                else:
                    let _ = segs.remove(lo)
                if seg.hi > hi:
                    segs[hi] = (hi: seg.hi, val: seg.val)
                    check_right = false
                if not check_left and not check_right:
                    segs[lo] = (hi: hi, val: val)
                    return
            else:
                on_old(D.nth(lo).l..D.nth(seg.hi - 1).r, seg.val)
                if sl < lo:
                    if seg.val == val:
                        new_lo = sl
                    else:
                        segs[sl] = (hi: lo, val: seg.val)
                    check_left = false
                else:
                    let _ = segs.remove(lo)
                var pos = seg.hi
                while pos < hi:
                    let old = segs.remove(pos)!
                    if old.hi <= hi:
                        on_old(D.nth(pos).l..D.nth(old.hi - 1).r, old.val)
                        pos = old.hi
                    else:
                        on_old(D.nth(pos).l..D.nth(hi - 1).r, old.val)
                        if old.val == val:
                            new_hi = old.hi
                        else:
                            segs[hi] = (hi: old.hi, val: old.val)
                        check_right = false
                        break
        if check_left:
            if let (pl, pseg) = segs.last_lt(new_lo):
                if pseg.val == val:
                    new_lo = pl
        if check_right:
            if let nseg = segs.get(new_hi):
                if nseg.val == val:
                    let _ = segs.remove(new_hi)
                    new_hi = nseg.hi
        segs[new_lo] = (hi: new_hi, val: val)

    /// Paint the single position `i` with `val` and return its previous value. O(log runs).
    pub fn assign_point(mut self, i: D, val: T) -> T:
        var old: ?T = none
        fn record(_: Range<D>, v: T): old = v
        self.assign(i.l..i.r, val, on_old: record)
        old!

    /// Visit every maximal run `(range, val)` in ascending order. O(runs).
    pub fn each(self) -> yields (range: Range<D>, val: T):
        for (lo, seg) in segs.items:
            yield (range: D.nth(lo).l..D.nth(seg.hi - 1).r, val: seg.val)

    /// O(log runs). Read the value at position `i` (canonical getter alongside `at`).
    pub fn get(self, i: D) -> T = todo

    /// O(log runs). Paint position `i` with `val` (`IndexSet` write-through place).
    pub fn set(mut self, i: D, val: T) = todo

    /// Amortized O((1 + overwritten) log runs). Paint `r` with `val` and yield every overwritten
    /// sub-run `(range, val)` in ascending order (generator form of `assign`).
    pub fn replace(mut self, r: Range<D>, val: T) -> yields (range: Range<D>, val: T) = todo

    /// O(log runs + k). Yield every run overlapping `r`, clipped to `r`, in ascending order.
    pub fn range(self, r: Range<D>) -> yields (range: Range<D>, val: T) = todo

    /// O(runs). Yield every maximal run `(range, val)` in descending order.
    pub fn each_rev(self) -> yields (range: Range<D>, val: T) = todo

    /// O(log runs + skipped). First position at or after `from` whose value satisfies `ok`, or `none`.
    pub fn first_where(self, from: Bounds<D>, ok: T -> Bool) -> ?D = todo

/// O(runs) space. Dynamic union of disjoint maximal half-open intervals over domain `D`
/// (KACTL `IntervalContainer`). Maintains total `covered` length in O(1).
pub struct IntervalSet<D>:
    segs: Map<Int, Int>

    /// O(1). Create an empty interval set over `D`.
    pub fn new() -> IntervalSet<D> = todo

    /// Amortized O((1 + merged) log runs). Add range `r` to the set, coalescing touching runs.
    pub fn add(mut self, r: Range<D>) = todo

    /// Amortized O((1 + split) log runs). Remove range `r` from the set.
    pub fn remove(mut self, r: Range<D>) = todo

    /// O(log runs). Whether position `i` is covered.
    pub fn contains(self, i: D) -> Bool = todo

    /// O(log runs). Whether every position in `r` is covered.
    pub fn contains_range(self, r: Range<D>) -> Bool = todo

    /// O(log runs). The maximal covered run containing `i`, or `none`.
    pub fn run_at(self, i: D) -> ?Range<D> = todo

    /// O(log runs). First covered position at or after `from`, or `none`.
    pub fn next_covered(self, from: D) -> ?D = todo

    /// O(log runs). First uncovered position at or after `from` (dynamic MEX), or `none`.
    pub fn next_gap(self, from: D) -> ?D = todo

    /// O(1). Total number of covered positions across all runs.
    pub @get fn covered(self) -> Int = todo

    /// O(1). Number of disjoint maximal runs.
    pub @get fn runs(self) -> Int = todo

    /// O(runs). Yield every disjoint maximal run in ascending order.
    pub fn each(self) -> yields Range<D> = todo

/// O(runs) space. Dynamic union of disjoint half-open integer intervals `[lo, hi)` over unbounded `Int` coordinates.
pub struct SparseIntervalSet:
    segs: Map<Int, Int>

    /// O(1). Create an empty sparse interval set.
    pub fn new() -> SparseIntervalSet = todo

    /// Amortized O((1 + merged) log runs). Add half-open interval `r`.
    pub fn add(mut self, r: Range<Int>) = todo

    /// Amortized O((1 + split) log runs). Remove half-open interval `r`.
    pub fn remove(mut self, r: Range<Int>) = todo

    /// O(log runs). Whether coordinate `x` is covered.
    pub fn contains(self, x: Int) -> Bool = todo

    /// O(log runs). First uncovered integer `>= x`.
    pub fn next_gap(self, x: Int) -> Int = todo

    /// O(1). Total covered length across all intervals.
    pub @get fn covered(self) -> Int = todo

    /// O(1). Number of disjoint maximal intervals.
    pub @get fn runs(self) -> Int = todo

    /// O(runs). Yield every disjoint maximal interval in ascending order.
    pub fn each(self) -> yields Range<Int> = todo

@test
fn interval_map_basic_and_coalescing():
    domain P = 1..=10
    var m = IntervalMap<P, Int>.new(0)
    assert m.runs == 1
    assert m.at(P(1)) == 0
    assert m.at(P(10)) == 0

    var over1 = List<(range: Range<P>, val: Int)>()
    m.assign(P(3)..=P(7), 5, on_old: (r, v) => over1.push((range: r, val: v)))
    assert over1.len == 1
    assert over1[0].range.len == 5 and over1[0].val == 0
    assert m.runs == 3

    // Painting with the same value coalesces with neighbours.
    var over2 = List<(range: Range<P>, val: Int)>()
    m.assign(P(1)..=P(2), 5, on_old: (r, v) => over2.push((range: r, val: v)))
    assert over2.len == 1 and over2[0].val == 0
    assert m.runs == 2
    assert m.run_at(P(4)).range.len == 7

    // assign_point splits and coalesces back.
    let prev = m.assign_point(P(4), 9)
    assert prev == 5
    assert m.runs == 4
    assert m.assign_point(P(4), 5) == 9
    assert m.runs == 2

@test
fn interval_map_from_and_empty_domain():
    domain E = 0..0
    var em = IntervalMap<E, Int>.new(7)
    assert em.runs == 0
    var seen = 0
    for _ in em.each(): seen += 1
    assert seen == 0

    domain P = 0..6
    let a: [P]Int = [1, 1, 2, 2, 2, 1]
    let m = IntervalMap<P, Int>.from(a)
    assert m.runs == 3
    for i in P: assert m.at(i) == a[i]

@test
fn interval_map_matches_dense_array():
    domain P = 1..=25
    var rng = Rng(42)
    var m = IntervalMap<P, Int>.new(0)
    var ref_a: [P]Int = 0
    for _ in 0..200:
        let l = rng.int(1, 25)
        let r = rng.int(l, 25)
        let v = rng.int(0, 4)
        var pos = l
        var over = List<(range: Range<P>, val: Int)>()
        m.assign(P(l)..=P(r), v, on_old: (rg, old_v) => over.push((range: rg, val: old_v)))
        for (rg, old_v) in over:
            for i in rg:
                assert i.to_idx + 1 == pos
                assert ref_a[i] == old_v
                ref_a[i] = v
                pos += 1
        assert pos == r + 1
        // Verify maximal-run invariant and pointwise agreement.
        var prev_val: ?Int = none
        var covered = 0
        for (rg, cur_v) in m.each():
            if let pv = prev_val: assert pv != cur_v
            prev_val = cur_v
            covered += rg.len
            for i in rg: assert ref_a[i] == cur_v
        assert covered == 25

/// Interval partition map over unbounded `Int` coordinates `[lo, hi)` (ODT without coordinate compression).
/// Maintains maximal constant runs; `assign` / `replace` are O((1 + removed) log runs) amortized.
pub struct SparseIntervalMap<T>:
    _init: T

    /// Create an interval map covering `lo..hi` initialized to `init`. O(1).
    pub fn new(lo: Int, hi: Int, init: T) -> SparseIntervalMap<T> = todo

    /// Number of maximal constant runs. O(1).
    pub @get fn runs(self) -> Int = todo

    /// Value at coordinate `i`. O(log runs).
    pub fn get(self, i: Int) -> T = todo

    /// Maximal run `(range, val)` containing `i`. O(log runs).
    pub fn run_at(self, i: Int) -> (range: Range<Int>, val: T) = todo

    /// Paint `r` with `val` and coalesce neighbours. O((1 + removed) log runs) amortized.
    pub fn assign(mut self, r: Range<Int>, val: T) = todo

    /// Paint `r` with `val`, yielding each overwritten `(range, val)` piece in ascending order. O((1 + removed) log runs) amortized.
    pub fn replace(mut self, r: Range<Int>, val: T) -> yields (range: Range<Int>, val: T) = todo

    /// Transform every run overlapping `r` by `f(old)` and coalesce neighbours. O((1 + touched) log runs).
    pub fn update(mut self, r: Range<Int>, f: T -> T) = todo

    /// Yield maximal sub-runs overlapping `r` (clipped to `r`) in ascending order. O(log runs + k).
    pub fn range(self, r: Range<Int>) -> yields (range: Range<Int>, val: T) = todo

    /// Yield all maximal runs in ascending order. O(runs).
    pub fn each(self) -> yields (range: Range<Int>, val: T) = todo

    /// Yield all maximal runs in descending order. O(runs).
    pub fn each_rev(self) -> yields (range: Range<Int>, val: T) = todo
