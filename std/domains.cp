//! `Domain`: the positions of a domain (R04.2), `Int` as one, and `Cut<D>`, the places between
//! them. Core re-exports both, so no module imports them.
//!
//! start: `Domain`, `Ordered`, `Domain.count`, `Cut`
//! example: `Int.nth(3) == 3 and 5.next! == 6`
use std.ranges.{Range, range_of}
use std.raw.{check_range}
use std.seq.{Seq}

@role(variant_check)
pub fn check_value(ok: Bool) = @intrinsic("value_check")

/// The positions of a domain (R04.2): `D: Domain` takes any domain, and every domain has it.
@role(positions)
pub trait Domain:
    /// `D.count`: how many positions it has.
    @get fn count() -> Int = @intrinsic("dom_count")
    /// `D.counted`: whether it is declared by a count alone (`domain E = [m]`), so `io.base`
    /// shifts it on input and output (R10.2.2).
    @get fn counted() -> Bool = @intrinsic("dom_counted")
    /// `D.all`: every position, in walk order (R04.2.30). A value's `s.all(p)` is another member.
    @get fn all() -> Range<Self> = range_of(0, Self.count, 1)
    /// `D.range`: the whole range, the same as `..` (R04.2.4).
    @get fn range() -> Range<Self> = Self.all
    /// `D.nth(k)`: the position `k` steps into its walk; traps past its end (R04.2.9).
    fn nth(k: Int) -> Self = @intrinsic("dom_nth")
    /// `D.mask(xs)`: a `[D]Bool` marking the positions `xs` yields (R04.2.1).
    fn mask<S: Seq<Self>>(xs: S) -> [Self]Bool:
        var m: [Self]Bool = false
        for v in xs: m[v] = true
        m
    /// `v.ord`: how many positions come before `v` in the walk (R04.2.15).
    @get fn ord(self) -> Int = @intrinsic("dom_ord")

/// The order of a domain's positions (R04.2.31): stepping to a neighbour, the cuts around a
/// position and its integer. Every core domain has it; a library domain, which only counts its
/// values, does not, so a generic body that steps bounds `D: Ordered`.
@role(order)
pub trait Ordered(Domain):
    /// `D.at(x)`: the position with integer `x`, or `None` outside it (R04.2.8).
    fn at(x: Int) -> ?Self = @intrinsic("dom_at")
    /// `D.wrap(k)`: the position `k` steps into its walk, Euclidean mod `D.count` (R04.2.13).
    fn wrap(k: Int) -> Self = Self.nth((k % Self.count + Self.count) % Self.count)
    /// `v.next(k)`: the position `k` after `v` in the walk, or `None` past its end (R04.3.1).
    @get fn next(self, k: Int = 1) -> ?Self:
        let o = self.ord + k
        if 0 <= o and o < Self.count: Some(Self.nth(o)) else: None
    /// `v.prev(k)`: the position `k` before `v` in the walk, or `None` past its start (R04.3.1).
    @get fn prev(self, k: Int = 1) -> ?Self = self.next(-k)
    /// `v.l`: the cut just before `v`; `v.l(k)`, the left cut of the `k` positions ending at `v`
    /// (R04.3.1).
    @get fn l(self, k: Int = 1) -> Cut<Self> = Cut(self.ord + 1 - k)
    /// `v.r`: the cut just after `v`; `v.r(k)`, the right cut of the `k` positions from `v`
    /// (R04.3.1).
    @get fn r(self, k: Int = 1) -> Cut<Self> = Cut(self.ord + k)
    /// `v.after`, `v.before`: the pieces `v.r..` and `..v.l` of its domain (R04.2.19).
    @get fn after(self) -> Range<Self> = range_of(self.ord + 1, Self.count, 1)
    @get fn before(self) -> Range<Self> = range_of(0, self.ord, 1)

/// A library type is a domain by implementing `Index`: `count` positions, `ord` a value's place in
/// the walk and `nth` the value at a place (R04.2.26). The impl's own code is checked: `nth` sees
/// only `0..count` and every `ord` it gives lands there (R04.2.27).
pub trait Index:
    @was(K)
    @get fn count() -> Int
    @was(to_idx)
    @get fn ord(self) -> Int
    @was(from_idx)
    fn nth(k: Int) -> Self

impl<T: Index> Domain for T:
    @get fn count() -> Int = T.count
    fn nth(k: Int) -> T:
        check_range(0 <= k and k < T.count)
        T.nth(k)
    @get fn ord(self) -> Int:
        let k = self.ord
        check_range(0 <= k and k < T.count)
        k

/// `Int` is a domain: its integers are its positions (R04.2.18).
impl Domain for Int

impl Ordered for Int:
    fn at(x: Int) -> ?Int = x
    @get fn next(self, k: Int = 1) -> ?Int = Some(self + k)
    @get fn prev(self, k: Int = 1) -> ?Int = Some(self - k)

/// A range type is a domain of its own positions, not `Int`'s: `(1..=3).at(9)` is `None`.
impl Domain for _..=_

impl Ordered for _..=_

/// A type with finitely many values, so a dense table over it has an end (R06.4.1): a range with
/// constant ends, a cut of one, or a tuple of them. `Int` is not one.
@on_unimplemented(code: E0311, "a dense table needs finitely many keys, and `{T}` has no end")
pub trait Finite

impl Finite for _..=_

impl<D: Domain + Finite> Finite for Cut<D>

impl<A: Finite, B: Finite> Finite for (A, B)
impl<A: Finite, B: Finite, C: Finite> Finite for (A, B, C)
impl<A: Finite, B: Finite, C: Finite, E: Finite> Finite for (A, B, C, E)

/// The table `@memo(array)` keeps: a slot for each position of `K`, made on the first `set`
/// (R06.4.1).
@memo_table(array)
pub struct MemoArray<K: Domain + Finite, V>:
    /// Each position's result by its ordinal, once one is kept; empty until the first `set`.
    pub var slots: List<?V> = []
    pub fn get(self, k: K) -> ?V = if slots.len == 0: None else: slots[k.ord]
    pub fn set(mut self, k: K, v: V):
        if slots.len == 0: slots = List.filled(K.count, None)
        slots[k.ord] = Some(v)

/// A cut of `D`: the place between two neighbouring positions, or at either end, with `k`
/// positions before it. `D.cuts` names its type and, as a value, the domain of them (R04.2.5).
@member(cuts)
@role(cuts)
pub struct Cut<D: Domain>:
    k: Int

    /// A cut prints as its integer (R04.2.16).
    pub fn write(self):
        write(k)

    /// `c.l`: the position just before `c`, or `None` at the start (R04.3.2).
    @get pub fn l(self) -> ?D:
        if k == 0: None else: Some(D.nth(k - 1))

    /// `c.r`: the position just after `c`, or `None` at the end (R04.3.2).
    @get pub fn r(self) -> ?D:
        if k == D.count: None else: Some(D.nth(k))

/// A domain's cuts are a domain, one more than its positions, walked in order; a cut's integer
/// is how many positions come before it, whatever the base (R04.2.5, R04.2.16, R04.2.17).
impl<D: Domain> Domain for Cut<D>:
    @get fn count() -> Int = D.count + 1
    fn nth(k: Int) -> Cut<D>:
        check_value(0 <= k and k <= D.count)
        Cut(k)
    @get fn ord(self) -> Int = k

/// A domain's cuts are in order, whatever its positions are (R04.2.31).
impl<D: Domain> Ordered for Cut<D>:
    fn at(x: Int) -> ?Cut<D> = if 0 <= x and x <= D.count: Some(Cut(x)) else: None

/// `D.cuts(k)`: the cut with `k` positions before it; traps R0102 past the ends (R04.2.17).
impl<D: Domain> Into<Cut<D>> for Int:
    fn into(self) -> Cut<D>:
        check_value(0 <= self and self <= D.count)
        Cut(self)

/// `Int(c)`: how many positions come before `c` (R04.2.16).
impl<D: Domain> Into<Int> for Cut<D>:
    fn into(self) -> Int = k

/// A run's ends as cuts: `[r]T` covers the keys from `r.start` to `r.end` (R04b.1.5).
@role(range_bounds)
pub trait RangeEnds<D: Domain>:
    fn start(self) -> Cut<D>
    fn end(self) -> Cut<D>

impl<D: Domain> RangeEnds<D> for Range<D>:
    fn start(self) -> Cut<D> = self.start
    fn end(self) -> Cut<D> = self.end
