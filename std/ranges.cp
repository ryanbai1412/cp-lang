//! `Range<D>`, the value `lo..hi by step` (R01.8.4), and the `[]` of a run of keys: `s[a..b]` is a
//! view of `s`, `r[a..b]` of a range a range again. Core re-exports `Range` and the `[]` fns, so no
//! module imports them. `std.range` has the algorithms over ranges.
//!
//! example: `let r = 2..8 by 3; r.len == 2 and r[5] == 5`
use std.seq.{IndexGet, Keyed, Positional, Indexed, Slice}
use std.raw.{check_key, check_range}
use std.ops.{By}
use std.show.{Fmt}

/// `lo..hi by step` as a value: `n` values from `lo`, `step` apart. It walks both ways, and its
/// `len`, `first` and `last` are O(1) (R01.8.4). `D` is the kind of its ends; `lo` and `by`
/// count in ordinals of `D`, which for `Int` are the integers themselves.
pub struct Range<D: Domain>:
    lo: Int
    n: Int
    by: Int

    @get pub fn len(self) -> Int:
        n

    /// The cut before its first value, of a range walked forwards by one (R04.3.1).
    @get pub fn start(self) -> Cut<D> = Cut(lo)

    /// The cut after its last value, of a range walked forwards by one (R04.3.1).
    @get pub fn end(self) -> Cut<D> = Cut(lo + n)

    /// Whether it holds no position (R04.4.12).
    @get pub fn is_empty(self) -> Bool:
        n == 0

    @get pub fn first(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo))

    @get pub fn last(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo + (n - 1) * by))

    /// The same values, last first.
    @get pub fn rev(self) -> Range<D>:
        Range(lo + (n - 1) * by, n, -by)

    pub fn each(self) -> yields D:
        for i in 0..n: yield D.nth(lo + i * by)

    /// Without its first `k` values: a range again (R07.4.11).
    pub fn drop(self, k: Int = 1) -> Range<D>:
        assert k >= 0, "drop: a negative count"
        let c = if k < n: k else: n
        Range(lo + c * by, n - c, by)

    /// Every `k`-th value, from the first: a range again, so `(3..m).step(2).sum()` is O(1).
    pub fn step(self, k: 1..) -> Range<D>:
        assert k >= 1, "step(k) requires k >= 1"
        Range(lo, (n + k - 1) / k, by * k)

    /// Its first `k` values: a range again (R07.4.11).
    pub fn take(self, k: Int) -> Range<D>:
        assert k >= 0, "take: a negative count"
        Range(lo, if k < n: k else: n, by)

/// `lo..hi by step`; empty when `lo` is past `hi` (R04.4.29).
@role(range_literal)
pub fn range_of<D: Domain>(lo: Int, hi: Int, step: Int) -> Range<D>:
    assert step > 0, "a range's step is positive"
    let n = if hi <= lo: 0 else: (hi - lo + step - 1) / step
    Range(lo, n, step)

/// The type of a range's ends picks what `lo..hi` is (decision 456): `Range<D>` over a domain's
/// positions, `RealRange` over reals.
@role(endpoint)
pub trait Endpoint:
    /// The struct `lo..hi` builds from two ends of this type.
    type Range

impl<D: Domain> Endpoint for D:
    type Range = Range<D>

/// `r by k` is `r.step(k)`: every `k`-th value from its first, so `V by 2` and `r.rev by 3` are
/// ranges again (R01.8.4).
impl<D: Domain> By<Int> for Range<D>:
    fn by(self, k: Int) -> Range<D> = self.step(k)

impl<D: Domain> Clone for Range<D>:
    fn clone(a) = Range(a.lo, a.n, a.by)

/// A range is keyed by its own values, so `(2..5)[3] == 3`, and has no source to lend (R07.2.15).
impl<D: Domain> Keyed<D, D> for Range<D>:
    fn items(self) -> yields (D, D):
        for i in 0..self.n:
            let k = self.lo + i * self.by
            yield (D.nth(k), D.nth(k))
impl<D: Domain> Positional<D> for Range<D>:
    fn nth(self, k: Int) -> D:
        check_key(0 <= k and k < self.n)
        D.nth(self.lo + k * self.by)
impl<D: Domain> IndexGet<D, D> for Range<D>:
    fn at(self, x: D) -> D:
        let d = x.ord - self.lo
        check_key(self.n > 0 and d % self.by == 0 and 0 <= d / self.by and d / self.by < self.n)
        x
impl<D: Domain> Indexed<D, D> for Range<D>:
    @get fn keys(self) -> Range<D> = self
impl<D: Domain> Contains<D> for Range<D>:
    fn contains(self, x: D) -> Bool:
        let d = x.ord - self.lo
        self.n > 0 and d % self.by == 0 and 0 <= d / self.by and d / self.by < self.n
/// `s[r]`: the view of `s` over the keys in `r`, made in O(1); traps R0101 unless `r` is a run of
/// `s`'s keys (R07.4.6). An empty `r` is the empty view at its start.
@index pub fn slice<S: Indexed<D, T>, D: Domain, T>(self: S, r: Range<D>) -> Slice<S>:
    let (i, ok) = run_in(self.keys, r)
    check_key(ok)
    Slice(self, i, i + r.n)
/// `s[lo..]`, `s[..hi]`, `s[..=hi]` and `s[..]`: an end left out is that end of `s`'s keys; traps
/// R0101 unless the ends are keys of `s` in order, or its end (R07.4.6).
@index pub fn slice_ends<S: Indexed<D, T>, D: Domain, T>(self: S, lo: ?D, hi: ?D, through: Bool) -> Slice<S>:
    let (i, j, ok) = cut_ends(self.keys, lo, hi, through)
    check_key(ok)
    Slice(self, i, j)
/// `r[a..b]` of a range is the sub-range `a..b`, itself a range: a range is keyed by its own
/// values and has no source to view (R07.2.15). Traps R0001 unless `a..b` is a run of `r`'s keys.
@index pub fn sub_range<D: Domain>(self: Range<D>, r: Range<D>) -> Range<D>:
    let (i, ok) = run_in(self, r)
    check_range(ok)
    Range(self.lo + i * self.by, r.n, self.by)
/// `r[a..]`, `r[..b]`, `r[..=b]` and `r[..]` of a range: the sub-range between the ends, an end
/// left out being `r`'s. Traps R0001 unless the ends are keys of `r` in order, or its end.
@index pub fn sub_range_ends<D: Domain>(self: Range<D>, lo: ?D, hi: ?D, through: Bool) -> Range<D>:
    let (i, j, ok) = cut_ends(self, lo, hi, through)
    check_range(ok)
    Range(self.lo + i * self.by, j - i, self.by)
/// Where the run `r` starts in the keys `k`, and whether `r` is a run of them (R07.4.6).
fn run_in<D: Domain>(k: Range<D>, r: Range<D>) -> (Int, Bool):
    let d = r.lo - k.lo
    let i = d / k.by
    (i, 0 <= d and d % k.by == 0 and i + r.n <= k.n and (r.n <= 1 or r.by == k.by))
/// The positions `i..j` in the keys `k` of the ends `lo` and `hi` (`hi` in when `through`), an end
/// left out being `k`'s, and whether they are keys of `k` in order, or its end (R07.4.6).
fn cut_ends<D: Domain>(k: Range<D>, lo: ?D, hi: ?D, through: Bool) -> (Int, Int, Bool):
    var ok = true
    var i = 0
    var j = k.n
    if let x = lo:
        let d = x.ord - k.lo
        ok = 0 <= d and d % k.by == 0
        i = d / k.by
    if let y = hi:
        let d = y.ord - k.lo
        ok = ok and 0 <= d and d % k.by == 0
        j = d / k.by + (if through: 1 else: 0)
    (i, j, ok and i <= j and j <= k.n)

/// A range prints its values, one gap apart (R10.3).
impl<D: Show> Show for Range<D>:
    fn write_to(self, mut out: Str, sep: ?Str) = self.write_fmt(mut out, sep, Fmt(None, 0, None))
    fn write_fmt(self, mut out: Str, sep: ?Str, fmt: Fmt):
        var first = true
        for x in self:
            if not first:
                if let s = sep: out.append(s)
                else: x.gap(mut out)
            x.write_fmt(mut out, None, fmt)
            first = false
    fn gap(self, mut out: Str):
        out.push('\n')

/// The same positions in the same order, however they were written (R08.1.5).
impl<D: Domain> Eq for Range<D>:
    fn eq(a, b) = a.len == b.len and (a.len == 0 or a.first == b.first and (a.len == 1 or a.by == b.by))
