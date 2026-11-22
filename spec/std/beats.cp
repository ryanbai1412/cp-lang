//! Segment Tree Beats (Ji Driver) for range chmin, chmax, add, sum, min, and max.
//!
//! Maintains exact `min`, second `min`, `max`, second `max`, and `sum` on every node.
//! Range `chmin`, `chmax`, `add`, and `sum`/`min`/`max` queries run in O(log n) amortized
//! (O(log^2 n) amortized when mixing `chmin`/`chmax` with `add`).
use std.seq.{ends}

/// Segment Tree Beats over a domain `D` of `Int` values.
/// Supports `chmin`, `chmax`, `add`, `sum`, `min_val`, `max_val`, and `get`.
pub struct BeatsSegTree<D>:
    s: List<Int>
    mx1: List<Int>
    mx2: List<Int>
    mx_c: List<Int>
    mn1: List<Int>
    mn2: List<Int>
    mn_c: List<Int>
    lz_add: List<Int>
    n: Int

    /// A Segment Tree Beats over `D` holding the values of `a`, or `fill` at every position, or
    /// zero everywhere when neither is given. Every node keeps the exact maximum, second maximum
    /// and its count, the same for the minimum, and the sum.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` builds a tree whose sums are zero and whose extrema are `None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    /// example: `BeatsSegTree<0..3>(fill: 2).all_sum == 6`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> BeatsSegTree<D> = todo

    fn pull(mut self, u: Int) = todo

    fn build(mut self, u: Int, nl: Int, nr: Int, vals: List<Int>) = todo

    fn apply_add(mut self, u: Int, len: Int, x: Int) = todo

    fn apply_chmin(mut self, u: Int, x: Int) = todo

    fn apply_chmax(mut self, u: Int, x: Int) = todo

    fn push(mut self, u: Int, nl: Int, nr: Int) = todo

    fn chmin_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Replace every `a[i]` in `r` with `min(a[i], x)`. Descends only into nodes whose maximum
    /// exceeds `x` and whose second maximum does not, where the update is a tag.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..3, 3); t.all_sum == 7`
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    fn chmax_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Replace every `a[i]` in `r` with `max(a[i], x)`; the mirror of `chmin`.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmax(0..2, 2); t.all_sum == 11`
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    fn add_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Add `x` to every `a[i]` in `r`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D` (overflow wraps in release, traps in debug)
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.add(1..3, 10); t.sum(0..3) == 30`
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    fn sum_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// The sum of the current values in `r`. Takes `mut self` because it pushes pending tags.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..3) == 5`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..1) == 0`
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    fn min_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// The least current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(0..3) == Some(1)`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(2..2) == None`
    pub fn min_val(mut self, r: Range<D>) -> ?Int = todo

    fn max_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// The greatest current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..1, 2); t.max_val(0..3) == Some(4)`
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// The current value at position `i`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).get(2) == 4`
    pub fn get(mut self, i: D) -> Int = todo

    /// Assign `a[i] = x`, replacing whatever earlier updates left there.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.set(1, 9); t.all_max == Some(9)`
    pub fn set(mut self, i: D, x: Int) = todo

    /// Overwrite every position in `r` with `x`.
    ///
    /// cost: O(log |D|) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.assign(0..2, 0); t.all_sum == 4`
    pub fn assign(mut self, r: Range<D>, x: Int) = todo

    /// Replace every `a[i]` in `r` with `a[i] / d`, rounding toward negative infinity. A node whose
    /// values all map to the same shift becomes a single `add` tag, so each value is divided
    /// O(log V) times before it settles, for values of magnitude at most `V`.
    ///
    /// cost: O(log |D| log V) amortized
    /// empty: changes nothing
    /// traps: when `d <= 0`, or a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chdiv(0..3, 2); t.all_sum == 4`
    /// example: `var t = BeatsSegTree([-3]); t.chdiv(0..1, 2); t.get(0) == -2`
    pub fn chdiv(mut self, r: Range<D>, d: Int) = todo

    /// Replace every `a[i]` in `r` with `a[i] mod m`, the remainder in `0..m` (floored, so
    /// negative values map into `0..m` too). Nodes whose maximum is below `m` are skipped.
    ///
    /// cost: O(log |D| log V) amortized
    /// empty: changes nothing
    /// traps: when `m <= 0`, or a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmod(0..3, 3); t.all_sum == 4`
    pub fn chmod(mut self, r: Range<D>, m: Int) = todo

    /// The sum of every current value in `D`.
    ///
    /// cost: O(1)
    /// empty: zero on an empty domain
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    @get pub fn all_sum(self) -> Int = todo

    /// The least current value in `D`, or `None` when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_min == Some(1)`
    @get pub fn all_min(self) -> ?Int = todo

    /// The greatest current value in `D`, or `None` when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_max == Some(5)`
    @get pub fn all_max(self) -> ?Int = todo

/// O(|D|) space. Segment Tree Beats with historic maximum and historic sum tracking under
/// range `chmin`, `chmax`, and `add` (Ji Driver with historic tags).
pub struct HistoricBeatsSegTree<D>:
    n: Int

    /// A Beats tree that also tracks history, holding `a`, or `fill` everywhere, or zeros. Each
    /// position remembers the greatest value it has held and the running sum of its values over
    /// versions; version 0 is the initial array, and every `chmin`, `chmax` or `add` call makes
    /// one new version of the whole array.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` builds a tree whose sums are zero and whose maxima are `None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `HistoricBeatsSegTree([1, 2]).hist_sum(0..2) == 3`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> HistoricBeatsSegTree<D> = todo

    /// Replace every `a[i]` in `r` with `min(a[i], x)`, then record a new version.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmin(0..2, 2); t.sum(0..2) == 3 and t.hist_max(0..1) == Some(5)`
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    /// Replace every `a[i]` in `r` with `max(a[i], x)`, then record a new version.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmax(0..2, 3); t.sum(0..2) == 8`
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    /// Add `x` to every `a[i]` in `r`, then record a new version.
    ///
    /// cost: O(log |D|)
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.sum(0..2) == 6`
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    /// The sum of the current values in `r`.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `HistoricBeatsSegTree([1, 2]).sum(0..2) == 3`
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    /// The greatest current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.max_val(0..1) == Some(-4)`
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// The greatest value any position in `r` has held in any version, the initial one included;
    /// `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.hist_max(0..1) == Some(6)`
    /// example: `HistoricBeatsSegTree([1, 2]).hist_max(1..1) == None`
    pub fn hist_max(mut self, r: Range<D>) -> ?Int = todo

    /// The sum over positions `i` in `r` of every value `a[i]` has had, one term per version:
    /// the initial array plus the array after each update call.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(0..2) == 9`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(1..2) == 4`
    pub fn hist_sum(mut self, r: Range<D>) -> Int = todo

@test
fn beats_segtree_chmin_chmax_add_matches_brute() = todo
