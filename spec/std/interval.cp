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

    /// A map that gives every position of `D` the value `init`: one run, or none when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: an empty `D` gives zero runs
    /// traps: never
    /// example: `IntervalMap<0..5, Int>(0).runs == 1`
    pub fn new(init: T) -> IntervalMap<D, T> = todo

    /// The map holding `a[i]` at every position `i`, stored as maximal runs of equal values.
    ///
    /// cost: O(|D|)
    /// empty: an empty `a` gives zero runs
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).runs == 2`
    pub fn from(a: [D]T) -> IntervalMap<D, T> = todo

    /// The number of maximal runs, i.e. one more than the number of adjacent positions whose
    /// values differ (zero for an empty `D`).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `IntervalMap.from([1, 2, 2, 1]).runs == 3`
    pub @get fn runs(self) -> Int = todo

    /// The value at position `i`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).get(2) == 2`
    pub fn get(self, i: D) -> T = todo

    /// The maximal run containing position `i` and its value.
    ///
    /// cost: O(log runs)
    /// order: the run's positions in ascending order
    /// traps: when `i` is outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).run_at(1).range == 0..2`
    /// example: `IntervalMap.from([1, 1, 2]).run_at(1).val == 1`
    pub fn run_at(self, i: D) -> (range: Range<D>, val: T) = todo

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
    pub fn assign(mut self, r: Range<D>, val: T, on_old: (Range<D>, T) -> () = (_, _) => ()) = todo


    /// Give position `i` the value `val` and return the value it had.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var m = IntervalMap.from([1, 2]); m.assign_point(0, 5) == 1 and m.get(0) == 5`
    pub fn assign_point(mut self, i: D, val: T) -> T = todo

    /// Every maximal run with its value.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty `D`
    /// order: ascending positions
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).each().map(p => p.val).collect() == [1, 2]`
    pub fn each(self) -> yields (range: Range<D>, val: T) = todo

    /// Give position `i` the value `val`; `assign_point` without the old value, and the
    /// write-through place for `m[i] = val`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var m = IntervalMap.from([1, 1]); m.set(1, 3); m.runs == 2`
    pub fn set(mut self, i: D, val: T) = todo

    /// Every run overlapping `r`, clipped to `r`, with its value. Clipping can make the first and
    /// last pieces shorter than their runs; they are not merged with anything.
    ///
    /// cost: O(log runs + k) for `k` pieces
    /// empty: yields nothing
    /// order: ascending positions
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `IntervalMap.from([1, 1, 2]).range(1..3).map(p => p.range).collect() == [1..2, 2..3]`
    pub fn range(self, r: Range<D>) -> yields (range: Range<D>, val: T) = todo

    /// Every maximal run with its value, last first.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty `D`
    /// order: descending positions
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).each_rev().map(p => p.val).collect() == [2, 1]`
    pub fn each_rev(self) -> yields (range: Range<D>, val: T) = todo

    /// The first position right of the cut `from` whose value satisfies `ok`, or `None`. Calls
    /// `ok` once per run, not per position.
    ///
    /// cost: O(log runs + skipped) for `skipped` runs that fail `ok`
    /// ties: the first position in key order
    /// traps: never
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..0).r, v => v == 2) == Some(2)`
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..1).r, v => v == 1) == Some(1)`
    /// example: `IntervalMap.from([1, 1, 2]).first_where((0..3).r, v => true) == None`
    pub fn first_where(self, from: Cut<D>, ok: T -> Bool) -> ?D = todo

/// O(runs) space. Dynamic union of disjoint maximal half-open intervals over domain `D`
/// (KACTL `IntervalContainer`). Maintains total `covered` length in O(1).
pub struct IntervalSet<D>:
    segs: Map<Int, Int>

    /// An empty set of positions of `D`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `IntervalSet<0..10>().covered == 0`
    pub fn new() -> IntervalSet<D> = todo

    /// Add every position of `r`, merging runs that overlap or touch it into one maximal run.
    ///
    /// cost: O((1 + merged) log runs) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(3..5); s.runs == 1 and s.covered == 4`
    pub fn add(mut self, r: Range<D>) = todo

    /// Remove every position of `r`; a run straddling an end of `r` is cut there.
    ///
    /// cost: O((1 + split) log runs) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(0..5); s.remove(1..2); s.runs == 2 and s.covered == 4`
    pub fn remove(mut self, r: Range<D>) = todo

    /// Whether position `i` is in the set.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.contains(3) and not s.contains(4)`
    pub fn contains(self, i: D) -> Bool = todo

    /// Whether every position of `r` is in the set, i.e. one run covers `r`.
    ///
    /// cost: O(log runs)
    /// empty: `true`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.add(4..6); s.contains_range(3..6) == true`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.contains_range(3..5) == false`
    pub fn contains_range(self, r: Range<D>) -> Bool = todo

    /// The maximal run containing position `i`, or `None` when `i` is not in the set.
    ///
    /// cost: O(log runs)
    /// order: the run's positions in ascending order
    /// traps: when `i` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.run_at(3) == Some(2..4)`
    /// example: `var s = IntervalSet<0..10>(); s.add(2..4); s.run_at(4) == None`
    pub fn run_at(self, i: D) -> ?Range<D> = todo

    /// The first position at or after `from` that is in the set, or `None`.
    ///
    /// cost: O(log runs)
    /// traps: when `from` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..7); s.next_covered(2) == Some(5)`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..7); s.next_covered(7) == None`
    pub fn next_covered(self, from: D) -> ?D = todo

    /// The first position at or after `from` that is not in the set (a dynamic mex), or `None`
    /// when every position from `from` to the end of `D` is covered.
    ///
    /// cost: O(log runs)
    /// traps: when `from` is outside `D`
    /// example: `var s = IntervalSet<0..10>(); s.add(0..3); s.next_gap(1) == Some(3)`
    /// example: `var s = IntervalSet<0..10>(); s.add(5..10); s.next_gap(6) == None`
    pub fn next_gap(self, from: D) -> ?D = todo

    /// The number of positions in the set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(6..7); s.covered == 3`
    pub @get fn covered(self) -> Int = todo

    /// The number of maximal runs.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(1..3); s.add(6..7); s.runs == 2`
    pub @get fn runs(self) -> Int = todo

    /// Every maximal run.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing for an empty set
    /// order: ascending positions
    /// traps: never
    /// example: `var s = IntervalSet<0..10>(); s.add(6..7); s.add(1..3); s.each().collect() == [1..3, 6..7]`
    pub fn each(self) -> yields Range<D> = todo

/// O(runs) space. Dynamic union of disjoint half-open integer intervals `[lo, hi)` over unbounded `Int` coordinates.
pub struct SparseIntervalSet:
    segs: Map<Int, Int>

    /// An empty set of `Int` coordinates, unbounded in both directions.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SparseIntervalSet().covered == 0`
    pub fn new() -> SparseIntervalSet = todo

    /// Add every coordinate of the half-open `r`, merging runs that overlap or touch it.
    ///
    /// cost: O((1 + merged) log runs) amortized
    /// empty: changes nothing
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(-5..0); s.add(0..5); s.runs == 1 and s.covered == 10`
    pub fn add(mut self, r: Range<Int>) = todo

    /// Remove every coordinate of the half-open `r`; a run straddling an end of `r` is cut there.
    ///
    /// cost: O((1 + split) log runs) amortized
    /// empty: changes nothing
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..10); s.remove(3..4); s.runs == 2 and s.covered == 9`
    pub fn remove(mut self, r: Range<Int>) = todo

    /// Whether the coordinate `x` is in the set.
    ///
    /// cost: O(log runs)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(10..20); s.contains(19) and not s.contains(20)`
    pub fn contains(self, x: Int) -> Bool = todo

    /// The least coordinate `>= x` that is not in the set; always exists, since the set is finite.
    ///
    /// cost: O(log runs)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.next_gap(1) == 3`
    /// example: `SparseIntervalSet().next_gap(-7) == -7`
    pub fn next_gap(self, x: Int) -> Int = todo

    /// The total number of coordinates in the set.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.add(10..12); s.covered == 5`
    pub @get fn covered(self) -> Int = todo

    /// The number of maximal intervals.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(0..3); s.add(10..12); s.runs == 2`
    pub @get fn runs(self) -> Int = todo

    /// Every maximal interval.
    ///
    /// cost: O(1) per interval
    /// empty: yields nothing for an empty set
    /// order: ascending coordinates
    /// traps: never
    /// example: `var s = SparseIntervalSet(); s.add(10..12); s.add(0..3); s.each().collect() == [0..3, 10..12]`
    pub fn each(self) -> yields Range<Int> = todo

@test
fn interval_map_basic_and_coalescing() = todo

@test
fn interval_map_from_and_empty_domain() = todo

@test
fn interval_map_matches_dense_array() = todo

/// Interval partition map over unbounded `Int` coordinates `[lo, hi)` (ODT without coordinate compression).
/// Maintains maximal constant runs; `assign` is O((1 + removed) log runs) amortized.
pub struct SparseIntervalMap<T>:
    _init: T

    /// A map giving every coordinate of `lo..hi` the value `init`: one run, or none when `lo == hi`.
    ///
    /// cost: O(1)
    /// traps: when `lo > hi`
    /// example: `SparseIntervalMap(0, 1_000_000_000, 0).runs == 1`
    pub fn new(lo: Int, hi: Int, init: T) -> SparseIntervalMap<T> = todo

    /// The number of maximal runs of equal values.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.runs == 3`
    pub @get fn runs(self) -> Int = todo

    /// The value at coordinate `i`.
    ///
    /// cost: O(log runs)
    /// traps: when `i` is outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.get(4) == 1`
    pub fn get(self, i: Int) -> T = todo

    /// The maximal run containing coordinate `i` and its value.
    ///
    /// cost: O(log runs)
    /// order: the run's coordinates in ascending order
    /// traps: when `i` is outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.run_at(7).range == 5..10`
    pub fn run_at(self, i: Int) -> (range: Range<Int>, val: T) = todo

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
    pub fn assign(mut self, r: Range<Int>, val: T, on_old: (Range<Int>, T) -> () = (_, _) => ()) = todo


    /// Replace the value `v` of every run overlapping `r`, clipped to `r`, with `f(v)`, then merge
    /// equal neighbours. `f` is called once per piece, in ascending order.
    ///
    /// cost: O((1 + touched) log runs)
    /// empty: changes nothing and calls `f` never
    /// traps: when a non-empty `r` reaches outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.update(0..4, v => v + 1); m.get(0) == 1 and m.get(3) == 2 and m.get(4) == 1`
    pub fn update(mut self, r: Range<Int>, f: T -> T) = todo

    /// Every run overlapping `r`, clipped to `r`, with its value.
    ///
    /// cost: O(log runs + k) for `k` pieces
    /// empty: yields nothing
    /// order: ascending coordinates
    /// traps: when a non-empty `r` reaches outside `lo..hi`
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.range(4..6).map(p => p.range).collect() == [4..5, 5..6]`
    pub fn range(self, r: Range<Int>) -> yields (range: Range<Int>, val: T) = todo

    /// Every maximal run with its value.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing when `lo == hi`
    /// order: ascending coordinates
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.each().map(p => p.val).collect() == [0, 1, 0]`
    pub fn each(self) -> yields (range: Range<Int>, val: T) = todo

    /// Every maximal run with its value, last first.
    ///
    /// cost: O(1) per run
    /// empty: yields nothing when `lo == hi`
    /// order: descending coordinates
    /// traps: never
    /// example: `var m = SparseIntervalMap(0, 10, 0); m.assign(3..5, 1); m.each_rev().map(p => p.range).collect() == [5..10, 3..5, 0..3]`
    pub fn each_rev(self) -> yields (range: Range<Int>, val: T) = todo
