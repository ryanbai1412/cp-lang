//! `Domain`: the positions of a domain (R04.2), `Int` as one, and `Cut<D>`, the places between
//! them. Core re-exports both, so no module imports them.
//!
//! example: `Int.nth(3) == 3 and 5.next! == 6`
use std.ranges.{Range, range_of}
use std.seq.{Seq}

/// The positions of a domain (R04.2): `D: Domain` takes any domain, and every domain has it.
@positions
pub trait Domain:
    /// `D.count`: how many positions it has.
    @get fn count() -> Int = @intrinsic("dom_count")
    /// `D.all`: every position, in walk order (R04.2.30). A value's `s.all(p)` is another member.
    @get fn all() -> Range<Self> = range_of(0, Self.count, 1)
    /// `D.range`: the whole range, the same as `..` (R04.2.4).
    @get fn range() -> Range<Self> = Self.all
    /// `D.nth(k)`: the position `k` steps into its walk; traps past its end (R04.2.9).
    fn nth(k: Int) -> Self = @intrinsic("dom_nth")
    /// `D.at(x)`: the position with integer `x`, or `None` outside it (R04.2.8).
    fn at(x: Int) -> ?Self = @intrinsic("dom_at")
    /// `D.wrap(k)`: the position `k` steps into its walk, Euclidean mod `D.count` (R04.2.13).
    fn wrap(k: Int) -> Self = Self.nth((k % Self.count + Self.count) % Self.count)
    /// `D.mask(xs)`: a `[D]Bool` marking the positions `xs` yields (R04.2.1).
    fn mask<S: Seq<Self>>(xs: S) -> [Self]Bool:
        var m: [Self]Bool = false
        for v in xs: m[v] = true
        m
    /// `v.ord`: how many positions come before `v` in the walk (R04.2.15).
    @get fn ord(self) -> Int = @intrinsic("dom_ord")
    /// `v.next(k)`: the position `k` after `v` in the walk, or `None` past its end (R04.3.1).
    @get fn next(self, k: Int = 1) -> ?Self:
        let o = self.ord + k
        if 0 <= o and o < Self.count: Some(Self.nth(o)) else: None
    /// `v.prev(k)`: the position `k` before `v` in the walk, or `None` past its start (R04.3.1).
    @get fn prev(self, k: Int = 1) -> ?Self = self.next(-k)
    /// `v.l`: the cut just before `v` (R04.3.1).
    @get fn l(self) -> Cut<Self> = Cut(self.ord)
    /// `v.r`: the cut just after `v` (R04.3.1).
    @get fn r(self) -> Cut<Self> = Cut(self.ord + 1)
    /// `v.after`, `v.before`: the pieces `v.r..` and `..v.l` of its domain (R04.2.19).
    @get fn after(self) -> Range<Self> = range_of(self.ord + 1, Self.count, 1)
    @get fn before(self) -> Range<Self> = range_of(0, self.ord, 1)

/// `Int` is a domain: its integers are its positions (R04.2.18).
impl Domain for Int:
    fn nth(k: Int) -> Int = k
    fn at(x: Int) -> ?Int = x
    @get fn ord(self) -> Int = self
    @get fn next(self, k: Int = 1) -> ?Int = Some(self + k)
    @get fn prev(self, k: Int = 1) -> ?Int = Some(self - k)

/// A cut of `D`: the place between two neighbouring positions, or at either end, with `k`
/// positions before it. `D.cuts` names its type and, as a value, the domain of them (R04.2.5).
@member(cuts)
@cuts
pub struct Cut<D: Domain>:
    k: Int

    /// A cut prints as its integer (R04.2.16).
    fn write(self):
        write(k)

    /// `c.l`: the position just before `c`, or `None` at the start (R04.3.2).
    @get fn l(self) -> ?D:
        if k == 0: None else: Some(D.nth(k - 1))

    /// `c.r`: the position just after `c`, or `None` at the end (R04.3.2).
    @get fn r(self) -> ?D:
        if k == D.count: None else: Some(D.nth(k))

/// A domain's cuts are a domain, one more than its positions, walked in order; a cut's integer
/// is how many positions come before it, whatever the base (R04.2.5, R04.2.16, R04.2.17).
impl<D: Domain> Domain for Cut<D>:
    @get fn count() -> Int = D.count + 1
    fn nth(k: Int) -> Cut<D>:
        check_value(0 <= k and k <= D.count)
        Cut(k)
    fn at(x: Int) -> ?Cut<D> = if 0 <= x and x <= D.count: Some(Cut(x)) else: None
    @get fn ord(self) -> Int = k

/// `D.cuts(k)`: the cut with `k` positions before it; traps R0102 past the ends (R04.2.17).
impl<D: Domain> Into<Cut<D>> for Int:
    fn into(self) -> Cut<D>:
        check_value(0 <= self and self <= D.count)
        Cut(self)

/// `Int(c)`: how many positions come before `c` (R04.2.16).
impl<D: Domain> Into<Int> for Cut<D>:
    fn into(self) -> Int = k

/// A run's ends as cuts: `[r]T` covers the keys from `r.start` to `r.end` (R04b.1.5).
@range_bounds
pub trait RangeEnds<D: Domain>:
    fn start(self) -> Cut<D>
    fn end(self) -> Cut<D>

impl<D: Domain> RangeEnds<D> for Range<D>:
    fn start(self) -> Cut<D> = self.start
    fn end(self) -> Cut<D> = self.end
