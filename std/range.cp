//! Ranges of positions: boundary cuts, halving, binary search and the cyclic `RingRange`.
//! Nothing here imports another std module, so `seq`, `prefix` and every other module can use it.

/// Left boundary cut of a range (valid even when empty).
///
/// cost: \(O(1)\)
/// empty: the same cut as `r`
/// traps: never
/// example: `(2..5).l.r == Some(2)`
@get pub fn l<D>(self: Range<D>) -> Cut<D> = self.start

/// Right boundary cut of a range (valid even when empty).
///
/// cost: \(O(1)\)
/// empty: the same cut as `l`
/// traps: never
/// example: `(2..5).r.l == Some(4)`
@get pub fn r<D>(self: Range<D>) -> Cut<D> = self.end

/// \(O(1)\). `(r.l..r.mid, r.mid..r.r)`: two non-empty halves, or `None` when `self.len <= 1`, so
/// `let (left, right) = r.halve() else: base` is the divide-and-conquer base case.
///
/// cost: \(O(1)\)
/// empty: `None`, as for one element
/// traps: never
/// example: `(0..5).halve() == Some((0..2, 2..5))`
/// order: left half first
pub fn halve<D>(self: Range<D>) -> ?(Range<D>, Range<D>):
    if self.len <= 1: return None
    let m = self.mid
    Some((self.l..m, m..self.r))

/// Every way to cut the range into two non-empty pieces, as `(left, right)` with
/// `left.r == right.l`.
///
/// There is one pair per cut strictly inside, so `len - 1` of them. Interval DP over
/// `V.ranges` folds over them, and every piece is already done when its range comes up.
///
/// cost: \(O(1)\) to make, \(O(1)\) per pair, nothing allocated
/// empty: yields nothing, as for one element
/// order: by the cut, left to right
/// traps: never
/// example: `(0..4).splits.map((l, r) => (l.len, r.len)).collect() == [(1, 3), (2, 2), (3, 1)]`
@get pub fn splits<D>(self: Range<D>) -> yields (Range<D>, Range<D>):
    for k in 1..self.len:
        let m = (self.nth(k)..self.nth(k)).l
        yield (self.l..m, m..self.r)

/// \(O(1)\). The middle cut: ⌊len/2⌋ elements before it; `r.l` when empty. Its `.r` is the
/// middle element only when `r` is non-empty, so guard with `if r.is_empty` first.
///
/// cost: \(O(1)\)
/// empty: `r.l`
/// traps: never
/// example: `(0..5).mid.r == Some(2)`
@get pub fn mid<D>(self: Range<D>) -> Cut<D>:
    let n = self.len
    if n == 0: return self.r
    (self.nth(n / 2)..self.nth(n / 2)).l

/// \(O(1)\). The positions in both ranges.
///
/// cost: \(O(1)\)
/// empty: an empty range when they do not meet
/// traps: never
/// example: `(0..5).clip(3..9) == 3..5`
/// order: increasing, a sub-range of both
pub fn clip<D>(self: Range<D>, other: Range<D>) -> Range<D>:
    if self.is_empty: return self
    if other.is_empty: return other
    let lo = max(self.l, other.l)
    let hi = min(self.r, other.r)
    if hi < lo: lo..lo else: lo..hi

/// \(O(\log n)\) calls to `ok`. The first element where `ok`, false then true along `r`, holds;
/// `None` when it never does.
///
/// An `ok` that is true at the start and false at the end (C++'s `lower_bound` habit) stops a
/// checked build; it costs one more call of `ok` when the answer is an end of `r`.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the answer is the first element or `None`
/// empty: `None`
/// traps: when `ok` is true at the start and false at the end; any other `ok` that is not
/// false-then-true gives an unspecified answer
/// example: `(0..100).first_true(x => x * x >= 50) == Some(8)`
/// ties: none: a monotone `ok` has one first true position
pub fn first_true<D>(self: Range<D>, ok: D -> Bool) -> ?D:
    let lo = flip_at(self.len, m => ok(self.nth(m)), "`ok` is true at the start and false at the end, but this search wants false then true; for true then false use `last_true`")
    if lo < self.len: Some(self.nth(lo)) else: None

/// \(O(\log n)\) calls to `ok`. The last element where `ok`, true then false along `r`, holds;
/// `None` when it never does.
///
/// An `ok` that is false at the start and true at the end stops a checked build; it costs one
/// more call of `ok` when the answer is an end of `r`.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the answer is the last element or `None`
/// empty: `None`
/// traps: when `ok` is false at the start and true at the end; any other `ok` that is not
/// true-then-false gives an unspecified answer
/// example: `(0..100).last_true(x => x * x <= 50) == Some(7)`
/// ties: none: a monotone `ok` has one last true position
pub fn last_true<D>(self: Range<D>, ok: D -> Bool) -> ?D:
    let lo = flip_at(self.len, m => not (ok(self.nth(m))), "`ok` is false at the start and true at the end, but `last_true` wants true then false; for false then true use `first_true`")
    if lo > 0: Some(self.nth(lo - 1)) else: None

// The first offset in `0..n` where `ok` holds, for `ok` false then true. A wrong-way `ok` (true
// at offset 0, false at `n - 1`) always lands on 0 or `n`, so checking the other end there
// catches every one; the check's call of `ok` happens in every build (R09.2.3).
fn flip_at(n: Int, ok: Int -> Bool, wrong_way: Str) -> Int:
    var lo = 0
    var hi = n
    // The one loop behind `first_true` and its kin; `0 <= lo <= m < hi <= n` keeps it in range.
    unchecked:
        while lo < hi:
            let m = lo + (hi - lo) / 2
            if ok(m): hi = m
            else: lo = m + 1
    if n > 1 and lo == 0:
        let at_end = ok(n - 1)
        assert at_end, wrong_way
    elif n > 1 and lo == n:
        let at_start = ok(0)
        assert not at_start, wrong_way
    lo

/// The cut where `ok` flips: false then true along `r` by default, true then false with
/// `last: true`. \(O(\log n)\) calls to `ok`.
///
/// By default it is `r.l` when `ok` always holds and `r.r` when it never does; with
/// `last: true` it is `r.r` when `ok` always holds. `first_true(ok)` is `.r` of it; `last_true(ok)` is
/// `bsearch(ok, last: true).l`.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the cut is an end of `r`
/// empty: `r.l`
/// ties: none: a monotone `ok` has one flip
/// traps: when `ok` flips the other way (true then false by default, false then true with
/// `last: true`)
/// example: `(0..100).bsearch(x => x * x >= 50).r == 8`
pub fn bsearch<D>(self: Range<D>, ok: D -> Bool, last: Bool = false) -> Cut<D>:
    let n = self.len
    let wrong_way = if last: "`ok` is false at the start and true at the end, but `last: true` wants true then false; drop `last: true` for false then true" else: "`ok` is true at the start and false at the end, but `bsearch` wants false then true; pass `last: true` for true then false"
    let lo = flip_at(n, m => ok(self.nth(m)) != last, wrong_way)
    if lo < n: (self.nth(lo)..self.nth(lo)).l else: self.r

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
///
/// cost: \(O(1)\)
/// empty: `(lo, lo)`
/// traps: when the range is written backwards
/// example: `(2..5).ends == (2, 5)`
@get pub fn ends<D>(self: Range<D>) -> (Int, Int):
    let lo = self.l.ord
    let hi = self.r.ord
    assert lo <= hi, "the range is written backwards"
    (lo, hi)

/// A run of positions of `D` read cyclically, possibly wrapping past the last position, at most
/// `D.count` of them. The keys-only half of a `RingSlice`, so it needs no array.
pub struct RingRange<D>():
    pub at: ?D = None
    n: Int = 0

    /// The number of positions in the run.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).len == 2`
    @get pub fn len(self) -> Int = n

    /// \(O(1)\). The first position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).start == 2`
    @get pub fn start(self) -> D = at!

    /// \(O(1)\). The last position.
    ///
    /// cost: \(O(1)\)
    /// traps: on an empty span
    /// example: `[1, 2, 3].ring.span(2, 2).end == 0`
    @get pub fn end(self) -> D = D.nth((at!.ord + n - 1) % D.count)

    /// \(O(1)\) to make. Its positions in ring order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) per step
    /// order: ring order from `start`
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).keys.collect() == [2, 0]`
    @get pub fn keys(self) -> yields D:
        for k in 0..n: yield D.nth((at!.ord + k) % D.count)

    /// Whether position `i` is in the run, counting the wrap past the last position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).contains(0) == true`
    pub fn contains(self, i: D) -> Bool:
        if n == 0: return false
        (i.ord - at!.ord + D.count) % D.count < n
