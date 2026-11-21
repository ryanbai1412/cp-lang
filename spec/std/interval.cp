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
    pub fn new(init: T) -> IntervalMap<D, T> = todo

    /// Build a compressed maximal-run partition from an array `a`. O(D.K).
    pub fn from(a: [D]T) -> IntervalMap<D, T> = todo

    /// Number of maximal runs currently in the partition. O(1).
    pub @get fn runs(self) -> Int = todo

    /// [rename] The value at position `i` (replaces `at`). O(log runs).
    pub fn get(self, i: D) -> T = todo

    /// The maximal run containing position `i`, as `(range, val)`. O(log runs).
    pub fn run_at(self, i: D) -> (range: Range<D>, val: T) = todo

    /// Paint range `r` with `val`, coalescing adjacent equal runs.
    /// Amortized O((1 + overwritten) log runs).
    pub fn assign(mut self, r: Range<D>, val: T) = todo

    /// [rename] Paint range `r` with `val`, coalescing adjacent equal runs, and yield every
    /// overwritten `(range, val)` piece in ascending order (replaces `assign(..., on_old:)`).
    /// Amortized O((1 + overwritten) log runs).
    pub fn replace(mut self, r: Range<D>, val: T) -> yields (range: Range<D>, val: T) = todo

    /// Paint the single position `i` with `val` and return its previous value. O(log runs).
    pub fn assign_point(mut self, i: D, val: T) -> T = todo

    /// Visit every maximal run `(range, val)` in ascending order. O(runs).
    pub fn each(self) -> yields (range: Range<D>, val: T) = todo

    /// O(log runs). Paint position `i` with `val` (`IndexSet` write-through place).
    pub fn set(mut self, i: D, val: T) = todo

    /// O(log runs + k). Yield every run overlapping `r`, clipped to `r`, in ascending order.
    pub fn range(self, r: Range<D>) -> yields (range: Range<D>, val: T) = todo

    /// O(runs). Yield every maximal run `(range, val)` in descending order.
    pub fn each_rev(self) -> yields (range: Range<D>, val: T) = todo

    /// O(log runs + skipped). First position at or after `from` whose value satisfies `ok`, or `none`.
    pub fn first_where(self, from: Cut<D>, ok: T -> Bool) -> ?D = todo

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
fn interval_map_basic_and_coalescing() = todo

@test
fn interval_map_from_and_empty_domain() = todo

@test
fn interval_map_matches_dense_array() = todo

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
