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

    /// A map that gives every position of `D` the value `init`: one run, or None when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: an empty `D` gives zero runs
    /// traps: never
    /// example: `IntervalMap<0..5, Int>(0).runs == 1`
    pub fn new(init: T) -> IntervalMap<D, T>:
        var m = Map<Int, (hi: Int, val: T)>()
        if D.K > 0: m[0] = (hi: D.K, val: init)
        IntervalMap(m)

    /// The map holding `a[i]` at every position `i`, stored as maximal runs of equal values.
    ///
    /// cost: O(|D|)
    /// empty: an empty `a` gives zero runs
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).runs == 2`
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

    /// The number of maximal runs, i.e. one more than the number of adjacent positions whose
    /// values differ (zero for an empty `D`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `IntervalMap.from([1, 2, 2, 1]).runs == 3`
    @get pub fn runs(self) -> Int:
        segs.len

    /// The value at position `i`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).get(2) == 2`
    pub fn get(self, i: D) -> T:
        segs.last_le(i.to_idx)!.1.val

    /// The maximal run containing position `i` and its value.
    ///
    /// cost: O(log runs)
    /// order: the run's positions in ascending order
    /// traps: when `i` is outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).run_at(1).range == 0..2`
    /// example: `IntervalMap.from([1, 1, 2]).run_at(1).val == 1`
    pub fn run_at(self, i: D) -> (range: Range<D>, val: T):
        let (lo, seg) = segs.last_le(i.to_idx)!
        (range: D.nth(lo).l..D.nth(seg.hi - 1).r, val: seg.val)

    /// Give every position of `r` the value `val`, then merge it with equal neighbouring runs so all
    /// runs stay maximal. `on_old` is called with each overwritten piece (an old run overlapping
    /// `r`, clipped to `r`) and its old value; the map is fully repainted when `assign` returns.
    ///
    /// cost: O((1 + overwritten) log runs) amortized
    /// empty: changes nothing and calls `on_old` never
    /// order: `on_old` sees pieces in ascending positions
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var m = IntervalMap<0..5, Int>(0); m.assign(1..3, 7); m.runs == 3`
    /// example: `var m = IntervalMap<0..5, Int>(0); m.assign(1..3, 7); m.assign(1..3, 0); m.runs == 1`
    /// example: `var m = IntervalMap.from([1, 1, 2, 2]); var old: List<Int> = []; m.assign(1..3, 9, on_old: (_, v) => old.push(v)); old == [1, 2]`
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
                    segs[lo] = (hi:, val:)
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
        segs[new_lo] = (hi: new_hi, val:)


    /// Give position `i` the value `val` and return the value it had.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var m = IntervalMap.from([1, 2]); m.assign_point(0, 5) == 1 and m.get(0) == 5`
    pub fn assign_point(mut self, i: D, val: T) -> T:
        var old: ?T = None
        fn record(_: Range<D>, v: T): old = v
        self.assign(i.l..i.r, val, on_old: record)
        old!

    /// Every maximal run with its value.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty `D`
    /// order: ascending positions
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).each().map(p => p.val).collect() == [1, 2]`
    pub fn each(self) -> yields (range: Range<D>, val: T):
        for (lo, seg) in segs.items:
            yield (range: D.nth(lo).l..D.nth(seg.hi - 1).r, val: seg.val)

    /// Give position `i` the value `val`; `assign_point` without the old value, and the
    /// write-through place for `m[i] = val`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var m = IntervalMap.from([1, 1]); m.set(1, 3); m.runs == 2`
    pub fn set(mut self, i: D, val: T):
        self.assign(i.l..i.r, val)

    /// Every run overlapping `r`, clipped to `r`, with its value. Clipping can make the first and
    /// last pieces shorter than their runs; they are not merged with anything.
    ///
    /// cost: O(log runs + k) for `k` pieces
    /// empty: yields nothing
    /// order: ascending positions
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).range(1..3).map(p => p.range).collect() == [1..2, 2..3]`
    pub fn range(self, r: Range<D>) -> yields (range: Range<D>, val: T):
        let (lo, hi) = r.ends
        var pos = lo
        while pos < hi:
            let (sl, seg) = segs.last_le(pos)!
            let b = min(seg.hi, hi)
            yield (range: D.nth(pos).l..D.nth(b - 1).r, val: seg.val)
            pos = b

    /// Every maximal run with its value, last first.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty `D`
    /// order: descending positions
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).each_rev().map(p => p.val).collect() == [2, 1]`
    pub fn each_rev(self) -> yields (range: Range<D>, val: T):
        var pos = D.K
        while pos > 0:
            let (sl, seg) = segs.last_lt(pos)!
            yield (range: D.nth(sl).l..D.nth(seg.hi - 1).r, val: seg.val)
            pos = sl

    /// The first position right of the cut `from` whose value satisfies `ok`, or `None`. Calls
    /// `ok` once per run, not per position.
    ///
    /// cost: O(log runs + skipped) for `skipped` runs that fail `ok`
    /// ties: the first position in key order
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..0).r, v => v == 2) == Some(2)`
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..1).r, v => v == 1) == Some(1)`
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..3).r, v => true) == None`
    pub fn first_where(self, from: Cut<D>, ok: T -> Bool) -> ?D:
        var pos = from.ord
        while pos < D.K:
            let (sl, seg) = segs.last_le(pos)!
            if ok(seg.val): return Some(D.nth(pos))
            pos = seg.hi
        None

/// O(runs) space. Dynamic union of disjoint maximal half-open intervals over domain `D`
/// (KACTL `IntervalContainer`). Maintains total `covered` length in O(1).
pub struct IntervalSet<D>:
    segs: Map<Int, Int>
    total: Int

    /// An empty set of positions of `D`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `IntervalSet<0..10>().covered == 0`
    pub fn new() -> IntervalSet<D>:
        IntervalSet(Map<Int, Int>(), 0)

    /// Add every position of `r`, merging runs that overlap or touch it into one maximal run.
    ///
    /// cost: O((1 + merged) log runs) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(3..5); s.runs == 1 and s.covered == 4`
    pub fn add(mut self, r: Range<D>):
        var (lo, hi) = r.ends
        if lo >= hi: return
        if let (pl, ph) = segs.last_le(lo):
            if ph >= lo:
                lo = pl
                hi = max(hi, ph)
                total -= ph - pl
                let _ = segs.remove(pl)
        while let (nl, nh) = segs.last_le(hi):
            if nl < lo: break
            hi = max(hi, nh)
            total -= nh - nl
            let _ = segs.remove(nl)
        segs[lo] = hi
        total += hi - lo

    /// Remove every position of `r`; a run straddling an end of `r` is cut there.
    ///
    /// cost: O((1 + split) log runs) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(0..5); s.remove(1..2); s.runs == 2 and s.covered == 4`
    pub fn remove(mut self, r: Range<D>):
        let (lo, hi) = r.ends
        if lo >= hi: return
        if let (pl, ph) = segs.last_lt(lo):
            if ph > lo:
                segs[pl] = lo
                total -= ph - lo
                if ph > hi:
                    segs[hi] = ph
                    total += ph - hi
        while let (nl, nh) = segs.last_lt(hi):
            if nl < lo: break
            let _ = segs.remove(nl)
            total -= nh - nl
            if nh > hi:
                segs[hi] = nh
                total += nh - hi

    /// Whether position `i` is in the set.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.contains(3) and not s.contains(4)`
    pub fn contains(self, i: D) -> Bool:
        let x = i.to_idx
        if let (_, h) = segs.last_le(x): return x < h
        false

    /// Whether every position of `r` is in the set, i.e. one run covers `r`.
    ///
    /// cost: O(log runs)
    /// empty: `true`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.add(4..6); s.contains_range(3..6) == true`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.contains_range(3..5) == false`
    pub fn contains_range(self, r: Range<D>) -> Bool:
        let (lo, hi) = r.ends
        if lo >= hi: return true
        if let (_, h) = segs.last_le(lo): return hi <= h
        false

    /// The maximal run containing position `i`, or `None` when `i` is not in the set.
    ///
    /// cost: O(log runs)
    /// order: the run's positions in ascending order
    /// traps: when `i` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.run_at(3) == Some(2..4)`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.run_at(4) == None`
    pub fn run_at(self, i: D) -> ?Range<D>:
        let x = i.to_idx
        if let (l, h) = segs.last_le(x):
            if x < h: return Some(D.nth(l).l..D.nth(h - 1).r)
        None

    /// The first position at or after `from` that is in the set, or `None`.
    ///
    /// cost: O(log runs)
    /// traps: when `from` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..7); s.next_covered(2) == Some(5)`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..7); s.next_covered(7) == None`
    pub fn next_covered(self, from: D) -> ?D:
        let x = from.to_idx
        if self.contains(from): return Some(from)
        if let (l, _) = segs.first_gt(x): return Some(D.nth(l))
        None

    /// The first position at or after `from` that is not in the set (a dynamic mex), or `None`
    /// when every position from `from` to the end of `D` is covered.
    ///
    /// cost: O(log runs)
    /// traps: when `from` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(0..3); s.next_gap(1) == Some(3)`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..10); s.next_gap(6) == None`
    pub fn next_gap(self, from: D) -> ?D:
        let x = from.to_idx
        var g = x
        if let (_, h) = segs.last_le(x):
            if x < h: g = h
        if g < D.K: Some(D.nth(g)) else: None

    /// The number of positions in the set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(6..7); s.covered == 3`
    @get pub fn covered(self) -> Int:
        total

    /// The number of maximal runs.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(6..7); s.runs == 2`
    @get pub fn runs(self) -> Int:
        segs.len

    /// Every maximal run.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty set
    /// order: ascending positions
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(6..7); s.add(1..3); s.each().collect() == [1..3, 6..7]`
    pub fn each(self) -> yields Range<D>:
        for (l, h) in segs.items: yield D.nth(l).l..D.nth(h - 1).r

/// O(runs) space. Dynamic union of disjoint half-open integer intervals `[lo, hi)` over unbounded `Int` coordinates.
pub struct SparseIntervalSet:
    segs: Map<Int, Int>
    total: Int

    /// An empty set of `Int` coordinates, unbounded in both directions.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SparseIntervalSet().covered == 0`
    pub fn new() -> SparseIntervalSet:
        SparseIntervalSet(Map<Int, Int>(), 0)

    /// Add every coordinate of the half-open `r`, merging runs that overlap or touch it.
    ///
    /// cost: O((1 + merged) log runs) amortized
    /// empty: changes nothing
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(-5..0); s.add(0..5); s.runs == 1 and s.covered == 10`
    pub fn add(mut self, r: Range<Int>):
        var lo = r.start
        var hi = r.end
        if lo >= hi: return
        if let (pl, ph) = segs.last_le(lo):
            if ph >= lo:
                lo = pl
                hi = max(hi, ph)
                total -= ph - pl
                let _ = segs.remove(pl)
        while let (nl, nh) = segs.last_le(hi):
            if nl < lo: break
            hi = max(hi, nh)
            total -= nh - nl
            let _ = segs.remove(nl)
        segs[lo] = hi
        total += hi - lo

    /// Remove every coordinate of the half-open `r`; a run straddling an end of `r` is cut there.
    ///
    /// cost: O((1 + split) log runs) amortized
    /// empty: changes nothing
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..10); s.remove(3..4); s.runs == 2 and s.covered == 9`
    pub fn remove(mut self, r: Range<Int>):
        let lo = r.start
        let hi = r.end
        if lo >= hi: return
        if let (pl, ph) = segs.last_lt(lo):
            if ph > lo:
                segs[pl] = lo
                total -= ph - lo
                if ph > hi:
                    segs[hi] = ph
                    total += ph - hi
        while let (nl, nh) = segs.last_lt(hi):
            if nl < lo: break
            let _ = segs.remove(nl)
            total -= nh - nl
            if nh > hi:
                segs[hi] = nh
                total += nh - hi

    /// Whether the coordinate `x` is in the set.
    ///
    /// cost: O(log runs)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(10..20); s.contains(19) and not s.contains(20)`
    pub fn contains(self, x: Int) -> Bool:
        if let (_, h) = segs.last_le(x): return x < h
        false

    /// The least coordinate `>= x` that is not in the set; always exists, since the set is finite.
    ///
    /// cost: O(log runs)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.next_gap(1) == 3`
    /// example: `SparseIntervalSet().next_gap(-7) == -7`
    pub fn next_gap(self, x: Int) -> Int:
        if let (_, h) = segs.last_le(x):
            if x < h: return h
        x

    /// The total number of coordinates in the set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.add(10..12); s.covered == 5`
    @get pub fn covered(self) -> Int:
        total

    /// The number of maximal intervals.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.add(10..12); s.runs == 2`
    @get pub fn runs(self) -> Int:
        segs.len

    /// Every maximal interval.
    ///
    /// cost: O(1) per interval
    /// empty: yields nothing for an empty set
    /// order: ascending coordinates
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(10..12); s.add(0..3); s.each().collect() == [0..3, 10..12]`
    pub fn each(self) -> yields Range<Int>:
        for (l, h) in segs.items: yield l..h

@test
fn interval_map_basic_and_coalescing():
    domain P = 1..=10
    var m = IntervalMap<P, Int>.new(0)
    assert m.runs == 1
    assert m.get(P(1)) == 0 and m.get(P(10)) == 0
    var over1 = List<(range: Range<P>, val: Int)>()
    m.assign(P(3)..=P(7), 5, on_old: (r, v) => over1.push((range: r, val: v)))
    assert over1.len == 1
    assert over1[0].range.len == 5 and over1[0].val == 0
    assert m.runs == 3
    var over2 = List<(range: Range<P>, val: Int)>()
    m.assign(P(1)..=P(2), 5, on_old: (r, v) => over2.push((range: r, val: v)))
    assert over2.len == 1 and over2[0].val == 0
    assert m.runs == 2
    assert m.run_at(P(4)).range.len == 7
    assert m.assign_point(P(4), 9) == 5
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
    let a: [P]Int = [P] i => [1, 1, 2, 2, 2, 1][i.to_idx]
    let m = IntervalMap<P, Int>.from(a)
    assert m.runs == 3
    for i in P: assert m.get(i) == a[i]

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
        var prev_val: ?Int = None
        var covered = 0
        for (rg, cur_v) in m.each():
            if let pv = prev_val: assert pv != cur_v
            prev_val = cur_v
            covered += rg.len
            for i in rg: assert ref_a[i] == cur_v
        assert covered == 25

/// Interval partition map over unbounded `Int` coordinates `[lo, hi)` (ODT without coordinate compression).
/// Maintains maximal constant runs; `assign` is O((1 + removed) log runs) amortized.
pub struct SparseIntervalMap<T>:
    _init: T
    lo: Int
    hi: Int
    segs: Map<Int, (hi: Int, val: T)>

    fn split(mut self, x: Int):
        if let (s, seg) = segs.last_le(x):
            if s < x and x < seg.hi:
                segs[s] = (hi: x, val: seg.val)
                segs[x] = (hi: seg.hi, val: seg.val)

    fn merge_at(mut self, x: Int):
        if let cur = segs.get(x):
            if let (pl, p) = segs.last_lt(x):
                if p.hi == x and p.val == cur.val:
                    let _ = segs.remove(x)
                    segs[pl] = (hi: cur.hi, val: p.val)

    /// A map giving every coordinate of `lo..hi` the value `init`: one run, or None when `lo == hi`.
    ///
    /// cost: O(1)
    /// traps: when `lo > hi`
    /// example: `SparseIntervalMap(0, 1_000_000_000, 0).runs == 1`
    pub fn new(lo: Int, hi: Int, init: T) -> SparseIntervalMap<T>:
        assert lo <= hi, "`SparseIntervalMap` needs `lo <= hi`"
        var m = Map<Int, (hi: Int, val: T)>()
        if lo < hi: m[lo] = (hi: hi, val: init)
        SparseIntervalMap(init, lo, hi, m)

    /// The number of maximal runs of equal values.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.runs == 3`
    @get pub fn runs(self) -> Int:
        segs.len

    /// The value at coordinate `i`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.get(4) == 1`
    pub fn get(self, i: Int) -> T:
        assert lo <= i and i < hi, "`get` coordinate outside the map"
        segs.last_le(i)!.1.val

    /// The maximal run containing coordinate `i` and its value.
    ///
    /// cost: O(log runs)
    /// order: the run's coordinates in ascending order
    /// traps: when `i` is outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.run_at(7).range == 5..10`
    pub fn run_at(self, i: Int) -> (range: Range<Int>, val: T):
        assert lo <= i and i < hi, "`run_at` coordinate outside the map"
        let (s, seg) = segs.last_le(i)!
        (range: s..seg.hi, val: seg.val)

    /// Give every coordinate of `r` the value `val`, then merge with equal neighbouring runs.
    /// `on_old` is called with each old run overlapping `r`, clipped to `r`, and its old value; the
    /// map is fully repainted when `assign` returns.
    ///
    /// cost: O((1 + removed) log runs) amortized
    /// empty: changes nothing and calls `on_old` never
    /// order: `on_old` sees pieces in ascending coordinates
    /// traps: when a non-empty `r` reaches outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.assign(5..7, 1); m.runs == 3`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); var old: List<Int> = []; m.assign(4..6, 2, on_old: (_, v) => old.push(v)); old == [1, 0]`
    pub fn assign(mut self, r: Range<Int>, val: T, on_old: (Range<Int>, T) -> () = (_, _) => ()):
        let (a, b) = (r.start, r.start + r.len)
        if a >= b: return
        assert lo <= a and b <= hi, "`assign` range outside the map"
        self.split(a)
        self.split(b)
        var pos = a
        while pos < b:
            let old = segs.remove(pos)!
            on_old(pos..old.hi, old.val)
            pos = old.hi
        segs[a] = (hi: b, val: val)
        self.merge_at(b)
        self.merge_at(a)


    /// Replace the value `v` of every run overlapping `r`, clipped to `r`, with `f(v)`, then merge
    /// equal neighbours. `f` is called once per piece, in ascending order.
    ///
    /// cost: O((1 + touched) log runs)
    /// empty: changes nothing and calls `f` never
    /// traps: when a non-empty `r` reaches outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.update(0..4, v => v + 1); m.get(0) == 1 and m.get(3) == 2 and m.get(4) == 1`
    pub fn update(mut self, r: Range<Int>, f: T -> T):
        let (a, b) = (r.start, r.start + r.len)
        if a >= b: return
        assert lo <= a and b <= hi, "`update` range outside the map"
        self.split(a)
        self.split(b)
        var starts: List<Int> = []
        var pos = a
        while pos < b:
            let seg = segs.get(pos)!
            segs[pos] = (hi: seg.hi, val: f(seg.val))
            starts.push(pos)
            pos = seg.hi
        self.merge_at(b)
        for k in (0..starts.len).rev: self.merge_at(starts[k])

    /// Every run overlapping `r`, clipped to `r`, with its value.
    ///
    /// cost: O(log runs + k) for `k` pieces
    /// empty: yields nothing
    /// order: ascending coordinates
    /// traps: when a non-empty `r` reaches outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.range(4..6).map(p => p.range).collect() == [4..5, 5..6]`
    pub fn range(self, r: Range<Int>) -> yields (range: Range<Int>, val: T):
        let (a, b) = (r.start, r.start + r.len)
        if a >= b: return
        assert lo <= a and b <= hi, "`range` outside the map"
        var pos = a
        while pos < b:
            let (_, seg) = segs.last_le(pos)!
            let e = min(seg.hi, b)
            yield (range: pos..e, val: seg.val)
            pos = e

    /// Every maximal run with its value.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing when `lo == hi`
    /// order: ascending coordinates
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.each().map(p => p.val).collect() == [0, 1, 0]`
    pub fn each(self) -> yields (range: Range<Int>, val: T):
        for (s, seg) in segs.items: yield (range: s..seg.hi, val: seg.val)

    /// Every maximal run with its value, last first.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing when `lo == hi`
    /// order: descending coordinates
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.each_rev().map(p => p.range).collect() == [5..10, 3..5, 0..3]`
    pub fn each_rev(self) -> yields (range: Range<Int>, val: T):
        let all = [(s, seg) for (s, seg) in segs.items]
        for k in (0..all.len).rev: yield (range: all[k].0..all[k].1.hi, val: all[k].1.val)
