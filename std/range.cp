//! Ranges of positions: their end cuts, halving, binary search and the cyclic `RingRange`.
//! Nothing here imports another std module, so `seq`, `prefix` and every other module can use it.
//!
//! start: `first_true`, `last_true`, `bsearch`, `ends`, `RingRange`
//! example: `(0..100).first_true(x => x * x >= 50) == Some(8)`

/// `lo..hi` over reals, what `0.0..8.0` is (decision 456). It has no positions to walk, so it is
/// searched by halving (`bsearch(ok, steps:)`) and golden section (`peak`, `valley`).
///
/// example: `let r = RealRange(0.0, 2.0); r.lo == 0.0 and r.hi == 2.0`
pub struct RealRange(pub lo: F64, pub hi: F64)


/// A size: a count, length or capacity. Every `.len` is below `2^31`, so a size is too.
///
/// example: `let n: Len = 5; n == 5`
pub type Len = 0..2_147_483_648

/// Left cut of a range (valid even when empty).
///
/// cost: \(O(1)\)
/// empty: `(2..2).l == (2..2).r`
/// traps: never
/// example: `(2..5).l.r == Some(2)`
@get pub fn l<D>(self: Range<D>) -> Cut<D> = self.start

/// Right cut of a range (valid even when empty).
///
/// cost: \(O(1)\)
/// empty: `(2..2).r == (2..2).l`
/// traps: never
/// example: `(2..5).r.l == Some(4)`
@get pub fn r<D>(self: Range<D>) -> Cut<D> = self.end

/// \(O(1)\). `(r.l..r.mid, r.mid..r.r)`: two non-empty halves, or `None` when `self.len <= 1`, so
/// `let (left, right) = r.halve() else: base` is the divide-and-conquer base case.
///
/// cost: \(O(1)\)
/// empty: `(3..3).halve() == None`
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
/// empty: `(3..3).splits.collect() == []`
/// order: by the cut, left to right
/// traps: never
/// example: `(0..4).splits.map((l, r) => (l.len, r.len)).collect() == [(1, 3), (2, 2), (3, 1)]`
@get pub fn splits<D>(self: Range<D>) -> yields (Range<D>, Range<D>):
    for p in self.drop(1):
        let m = (p..p).l
        yield (self.l..m, m..self.r)

/// \(O(1)\). The middle cut: ⌊len/2⌋ elements before it; `r.l` when empty. Its `.r` is the
/// middle element only when `r` is non-empty, so guard with `if r.is_empty` first.
///
/// cost: \(O(1)\)
/// empty: `(3..3).mid == (3..3).l`
/// traps: never
/// example: `(0..5).mid.r == Some(2)`
@get pub fn mid<D>(self: Range<D>) -> Cut<D>:
    let n = self.len
    if n == 0: return self.r
    (self.nth(n / 2)..self.nth(n / 2)).l

/// \(O(1)\). The positions in both ranges.
///
/// cost: \(O(1)\)
/// empty: `(0..2).clip(5..9).len == 0`
/// traps: never
/// example: `(0..5).clip(3..9) == 3..5`
/// order: increasing, a sub-range of both
pub fn clip<D>(self: Range<D>, other: Range<D>) -> Range<D>:
    if self.is_empty: return self
    if other.is_empty: return other
    let lo = max(self.l, other.l)
    let hi = min(self.r, other.r)
    if hi < lo: lo..lo else: lo..hi

/// The first element where `ok`, false then true along `r`, holds; `None` when it never does. A
/// monotone `ok` has one first true position. \(O(\log n)\) calls to `ok`.
///
/// An `ok` that is true at the start and false at the end (C++'s `lower_bound` habit) stops a
/// checked build; it costs one more call of `ok` when the answer is an end of `r`.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the answer is the first element or `None`
/// empty: `(0..0).first_true(x => true) == None`
/// traps: when `ok` is true at the start and false at the end; any other `ok` that is not
/// false-then-true gives an unspecified answer
/// example: `(0..100).first_true(x => x * x >= 50) == Some(8)`
/// ties: `(0..5).first_true(x => x >= 2) == Some(2)`
@was(bsearch_first)
pub fn first_true<D>(self: Range<D>, ok: D -> Bool) -> ?D:
    let lo = flip_at(self.len, m => ok(self.nth(m)), "`ok` is true at the start and false at the end, but this search wants false then true; for true then false use `last_true`")
    if lo < self.len: Some(self.nth(lo)) else: None

/// The last element where `ok`, true then false along `r`, holds; `None` when it never does. A
/// monotone `ok` has one last true position. \(O(\log n)\) calls to `ok`.
///
/// An `ok` that is false at the start and true at the end stops a checked build; it costs one
/// more call of `ok` when the answer is an end of `r`.
///
/// cost: \(O(\log n)\) calls of `ok`, plus one when the answer is the last element or `None`
/// empty: `(0..0).last_true(x => true) == None`
/// traps: when `ok` is false at the start and true at the end; any other `ok` that is not
/// true-then-false gives an unspecified answer
/// example: `(0..100).last_true(x => x * x <= 50) == Some(7)`
/// ties: `(0..5).last_true(x => x <= 2) == Some(2)`
@was(bsearch_last)
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

/// Where `ok` flips along `self`: false then true by default, true then false with `last: true`.
/// A monotone `ok` has one flip, and the result is its cut: `r.l` when `ok` always holds and
/// `r.r` when it never does (with `last: true`, `r.r` when it always holds). `first_true(ok)` is
/// `.r` of it; `last_true(ok)` is `bsearch(ok, last: true).l`. A `RealRange` has its own
/// `bsearch`, which halves `steps` times.
///
/// cost: \(O(\log n)\) calls of `ok` on \(n\) positions, plus one when the cut is an end
/// empty: `(3..3).bsearch(x => true) == (3..3).l`
/// ties: `(0..5).bsearch(x => x >= 2).r == 2`
/// traps: when `ok` flips the other way (true then false by default, false then true with
/// `last: true`)
/// example: `(0..100).bsearch(x => x * x >= 50).r == 8`
/// example: `domain V = 0..10; (V.first!..=V.last!).bsearch(v => v.ord >= 4).r!.ord == 4`
pub fn bsearch<D>(self: Range<D>, ok: D -> Bool, last: Bool = false) -> Cut<D>:
    let n = self.len
    let wrong_way = if last: "`ok` is false at the start and true at the end, but `last: true` wants true then false; drop `last: true` for false then true" else: "`ok` is true at the start and false at the end, but `bsearch` wants false then true; pass `last: true` for true then false"
    let lo = flip_at(n, m => ok(self.nth(m)) != last, wrong_way)
    if lo < n: (self.nth(lo)..self.nth(lo)).l else: self.r

/// The elements at distances `r` after `self`, clipped to its domain and possibly empty. One
/// definition for positions and cuts, since `V.cuts` is a domain: the chords of length `lo..=hi`
/// from a cut `c` end in `c.shift(lo..=hi)`, and `v.shift(-1..=1)` is `v` and its neighbours.
///
/// cost: \(O(1)\)
/// empty: `domain V = 0..3; V.cuts.last!.shift(1..=5).len == 0`
/// order: increasing
/// traps: never
/// example: `domain V = 0..6; V(0).l.shift(2..=3).len == 2 and V(0).l.shift(2..=3).first!.ord == 2`
/// example: `domain V = 0..6; V(4).shift(-2..=9).len == 4`
pub fn shift<D>(self: D, r: Range<Int>) -> Range<D>:
    let n = D.count
    let lo = min(max(self.ord + r.start, 0), n)
    let hi = min(max(self.ord + r.start + r.len, lo), n)
    if lo == hi: return D.nth(0)..D.nth(0)
    D.nth(lo)..=D.nth(hi - 1)

/// The position `k` steps from `self`, stopping at the domain's first or last: `v.next(k)` that
/// saturates instead of giving `None`. One definition for positions and cuts, like `shift`.
///
/// cost: \(O(1)\)
/// traps: never (a domain with a position is not empty)
/// example: `domain V = 0..6; V(4).shift_saturating(1) == V(5) and V(4).shift_saturating(9) == V(5)`
/// example: `domain V = 0..6; V(1).shift_saturating(-3) == V(0) and V(0).l.shift_saturating(9) == V.cuts.last!`
pub fn shift_saturating<D>(self: D, k: Int) -> D:
    let n = D.count
    let o = self.ord
    D.nth(if k >= n - 1 - o: n - 1 elif k <= -o: 0 else: o + k)

/// The position `k` steps from `self`, or `None` outside its domain: `v.next` for any `k`. One
/// definition for positions and cuts, like `shift`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain V = 1..=6; V(5).offset(1) == Some(V(6)) and V(5).offset(2) == None and V(2).offset(-1) == Some(V(1))`
pub fn offset<D>(self: D, k: Int) -> ?D = D.try_nth(self.ord + k)

/// The position `k` steps after `self`, or `None` past the domain's last: `v.offset(k)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain V = 0..6; V(4).ahead(1) == Some(V(5)) and V(4).ahead(2) == None`
pub fn ahead<D>(self: D, k: Int) -> ?D = self.offset(k)

/// The position `k` steps before `self`, or `None` before the domain's first: `v.offset(-k)`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `domain V = 0..6; V(1).back(1) == Some(V(0)) and V(1).back(2) == None`
pub fn back<D>(self: D, k: Int) -> ?D = self.offset(-k)

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
///
/// cost: \(O(1)\)
/// empty: `(3..3).ends == (3, 3)`
/// traps: when the range is written backwards
/// example: `(2..5).ends == (2, 5)`
@get pub fn ends<D>(self: Range<D>) -> (Int, Int):
    let lo = self.l.ord
    let hi = self.r.ord
    assert lo <= hi, "the range is written backwards"
    (lo, hi)

/// A run of positions of `D` read cyclically, possibly wrapping past the last position, at most
/// the ring's length of them. The keys-only half of a `RingSlice`, so it needs no array.
pub struct RingRange<D: Discrete>():
    pub at: ?D = None
    n: Len = 0
    m: Len = 0

    /// The number of positions in the run.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).len == 2`
    @get pub fn len(self) -> Int = n

    /// Whether the span covers no position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `not [1, 2, 3].ring.span(2, 2).is_empty`
    @get pub fn is_empty(self) -> Bool = n == 0

    /// \(O(\mathit{len})\). Visit the span's positions. Same walk as `keys`.
    ///
    /// cost: \(O(\mathit{len})\)
    /// empty: `[1, 2, 3].ring.span(1, 0).each().collect() == []`
    /// order: ring order from `start`, wrapping past the last position
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).each().collect() == [2, 0]`
    pub fn each(self) -> yields D:
        for k in self.keys: yield k

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
    @get pub fn end(self) -> D = D.nth((at!.ord + n - 1) % m)

    /// \(O(1)\) to make. Its positions in ring order.
    ///
    /// cost: \(O(1)\) to make, \(O(1)\) per step
    /// order: ring order from `start`
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).keys.collect() == [2, 0]`
    @get pub fn keys(self) -> yields D:
        for k in 0..n: yield D.nth((at!.ord + k) % m)

impl<D: Discrete> Contains<D> for RingRange<D>:
    /// Whether position `i` is in the run, counting the wrap past the last position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[1, 2, 3].ring.span(2, 2).contains(0) == true`
    fn contains(self, i: D) -> Bool:
        if n == 0: return false
        (i.ord - at!.ord + m) % m < n

impl Show for RingRange<D: Show>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out
