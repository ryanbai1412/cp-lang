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
    /// Returns every overwritten run `(range, old_val)` in ascending order.
    /// Amortized O((1 + overwritten) log runs).
    pub fn assign(mut self, r: Range<D>, val: T) -> List<(range: Range<D>, val: T)>:
        let (lo, hi) = r.ends
        var out = List<(range: Range<D>, val: T)>()
        if lo >= hi: return out
        self.split(lo)
        self.split(hi)
        var pos = lo
        while pos < hi:
            let seg = segs.remove(pos)!
            out.push((range: D.nth(pos).l..D.nth(seg.hi - 1).r, val: seg.val))
            pos = seg.hi
        var new_lo = lo
        var new_hi = hi
        if let (pl, pseg) = segs.last_lt(new_lo):
            if pseg.val == val:
                let _ = segs.remove(pl)
                new_lo = pl
        if let nseg = segs.get(new_hi):
            if nseg.val == val:
                let _ = segs.remove(new_hi)
                new_hi = nseg.hi
        segs[new_lo] = (hi: new_hi, val: val)
        out

    /// Paint the single position `i` with `val` and return its previous value. O(log runs).
    pub fn assign_point(mut self, i: D, val: T) -> T:
        let old = self.assign(i.l..i.r, val)
        old[0].val

    /// Visit every maximal run `(range, val)` in ascending order. O(runs).
    pub fn each(self) -> yields (range: Range<D>, val: T):
        for (lo, seg) in segs.items:
            yield (range: D.nth(lo).l..D.nth(seg.hi - 1).r, val: seg.val)

    fn split(mut self, x: Int):
        if x <= 0 or x >= D.K: return
        if let (sl, seg) = segs.last_le(x):
            if sl < x and seg.hi > x:
                segs[sl] = (hi: x, val: seg.val)
                segs[x] = (hi: seg.hi, val: seg.val)

@test
fn interval_map_basic_and_coalescing():
    domain P = 1..=10
    var m = IntervalMap<P, Int>.new(0)
    assert m.runs == 1
    assert m.at(P(1)) == 0
    assert m.at(P(10)) == 0

    let over1 = m.assign(P(3)..=P(7), 5)
    assert over1.len == 1
    assert over1[0].range.len == 5 and over1[0].val == 0
    assert m.runs == 3

    // Painting with the same value coalesces with neighbours.
    let over2 = m.assign(P(1)..=P(2), 5)
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
        let over = m.assign(P(l)..=P(r), v)
        var pos = l
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
