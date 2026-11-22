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

    /// Build from `a: [D]Int` or a uniform `fill:` value (default 0). O(n).
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> BeatsSegTree<D> = todo

    fn pull(mut self, u: Int) = todo

    fn build(mut self, u: Int, nl: Int, nr: Int, vals: List<Int>) = todo

    fn apply_add(mut self, u: Int, len: Int, x: Int) = todo

    fn apply_chmin(mut self, u: Int, x: Int) = todo

    fn apply_chmax(mut self, u: Int, x: Int) = todo

    fn push(mut self, u: Int, nl: Int, nr: Int) = todo

    fn chmin_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Replace every `a[i]` in `r` with `min(a[i], x)`. O(log n) amortized.
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    fn chmax_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Replace every `a[i]` in `r` with `max(a[i], x)`. O(log n) amortized.
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    fn add_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int) = todo

    /// Add `x` to every `a[i]` in `r`. O(log n).
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    fn sum_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// Sum of elements in `r`. O(log n).
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    fn min_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// Minimum element in `r`, or `None` if `r` is empty. O(log n).
    pub fn min_val(mut self, r: Range<D>) -> ?Int = todo

    fn max_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int = todo

    /// Maximum element in `r`, or `None` if `r` is empty. O(log n).
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// Value at a single position `i`. O(log n).
    pub fn get(mut self, i: D) -> Int = todo

    /// O(log |D|). Assign position `i` to `x`.
    pub fn set(mut self, i: D, x: Int) = todo

    /// O(log |D|) amortized. Overwrite every position in `r` with `x`.
    pub fn assign(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D| * log V) amortized. Replace every `a[i]` in `r` with `a[i] / d` (floor division, `d > 0`).
    pub fn chdiv(mut self, r: Range<D>, d: Int) = todo

    /// O(log |D| * log V) amortized. Replace every `a[i]` in `r` with `a[i] % m` (`m > 0`).
    pub fn chmod(mut self, r: Range<D>, m: Int) = todo

    /// O(1). Sum of the entire domain.
    pub @get fn all_sum(self) -> Int = todo

    /// O(1). Minimum of the entire domain, or `None` when empty.
    pub @get fn all_min(self) -> ?Int = todo

    /// O(1). Maximum of the entire domain, or `None` when empty.
    pub @get fn all_max(self) -> ?Int = todo

/// O(|D|) space. Segment Tree Beats with historic maximum and historic sum tracking under
/// range `chmin`, `chmax`, and `add` (Ji Driver with historic tags).
pub struct HistoricBeatsSegTree<D>:
    n: Int

    /// O(|D|). Build from `a: [D]Int` or uniform `fill:` (default 0).
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> HistoricBeatsSegTree<D> = todo

    /// O(log |D|) amortized. Replace every `a[i]` in `r` with `min(a[i], x)`.
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|) amortized. Replace every `a[i]` in `r` with `max(a[i], x)`.
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|). Add `x` to every `a[i]` in `r`.
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|). Current sum of elements in `r`.
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    /// O(log |D|). Current maximum in `r`, or `None` when empty.
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// O(log |D|). Historic maximum ever attained at any position in `r`, or `None` when empty.
    pub fn hist_max(mut self, r: Range<D>) -> ?Int = todo

    /// O(log |D|). Historic sum of all versions across `r`.
    pub fn hist_sum(mut self, r: Range<D>) -> Int = todo

@test
fn beats_segtree_chmin_chmax_add_matches_brute() = todo
