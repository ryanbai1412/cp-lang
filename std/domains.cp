//! `Discrete` and `Domain`: the positions of a walk and of a domain (R04.2), `Int` as the first,
//! and `Cut<D>`, the places between them. Core re-exports them, so no module imports them.
//!
//! example: `Int.nth(3) == 3 and 5.next! == 6`
use std.ranges.{Range, range_of}
use std.raw.{check_range}
use std.memo.{MemoTable}
use std.seq.{Seq}

@role(variant_check)
pub fn check_value(ok: Bool) = @intrinsic("value_check")

/// Positions in a walk without, necessarily, an end (R04.2.18): `ord` a value's place in the walk
/// and `nth` the value at a place. `Int` is one, and so is every domain; `D: Discrete` takes
/// both, and `[D]T` over one that isn't a `Domain` is the list (R04.8.11).
@role(discrete)
pub trait Discrete:
    /// `D.nth(k)`: the position `k` steps into its walk; traps past its end (R04.2.9).
    fn nth(k: Int) -> Self = @intrinsic("dom_nth")
    /// `D.try_nth(k)`: the position `k` steps into its walk, or `None` past either end (R04.2.32).
    fn try_nth(k: Int) -> ?Self = @intrinsic("dom_try_nth")
    /// `v.ord`: how many positions come before `v` in the walk (R04.2.15).
    @get fn ord(self) -> Int = @intrinsic("dom_ord")

/// The positions of a domain (R04.2): finitely many, `D.count` of them. `D: Domain` takes any
/// domain, and every domain has it; `Int` has no count, so it is only `Discrete`.
@role(positions)
pub trait Domain(Discrete):
    /// `D.count`: how many positions it has.
    @get fn count() -> Int = @intrinsic("dom_count")
    /// `D.from_count`: whether it is declared by a count alone (`domain E = [m]`), so `io.base`
    /// shifts it on input and output (R10.2.2).
    @get fn from_count() -> Bool = @intrinsic("dom_from_count")
    /// `D.all`: every position, in walk order (R04.2.30). A value's `s.all(p)` is another member.
    @get fn all() -> Range<Self> = range_of(0, Self.count, 1)
    /// `D.range`: the whole range, the same as `..` (R04.2.4).
    @get fn range() -> Range<Self> = Self.all
    /// `D.mask(xs)`: a `[D]Bool` marking the positions `xs` yields (R04.2.1).
    fn mask<S: Seq<Self>>(xs: S) -> [Self]Bool:
        var m: [Self]Bool = false
        for v in xs: m[v] = true
        m

/// The order of a domain's positions (R04.2.31): stepping to a neighbour, the cuts around a
/// position and its integer. Every core domain has it; a library domain, which only counts its
/// values, does not, so a generic body that steps bounds `D: Ordered`.
@role(order)
pub trait Ordered(Discrete):
    /// `D.at(x)`: the position with integer `x`, or `None` outside it (R04.2.8).
    fn at(x: Int) -> ?Self = @intrinsic("dom_at")
    /// `v.next(k)`: the position `k` after `v` in the walk, or `None` past its end (R04.3.1).
    @get fn next(self, k: Int = 1) -> ?Self = Self.try_nth(self.ord + k)
    /// `v.prev(k)`: the position `k` before `v` in the walk, or `None` past its start (R04.3.1).
    @get fn prev(self, k: Int = 1) -> ?Self = self.next(-k)
    /// `v.l`: the cut just before `v`; `v.l(k)`, the left cut of the `k` positions ending at `v`
    /// (R04.3.1).
    @get fn l(self, k: Int = 1) -> Cut<Self> = Cut(self.ord + 1 - k)
    /// `v.r`: the cut just after `v`; `v.r(k)`, the right cut of the `k` positions from `v`
    /// (R04.3.1).
    @get fn r(self, k: Int = 1) -> Cut<Self> = Cut(self.ord + k)
    /// `v.before`: the piece `..v.l` of its domain (R04.2.19).
    @get fn before(self) -> Range<Self> = range_of(0, self.ord, 1)

/// The members of an ordered domain that read its count: wrapping around its walk, and the
/// piece after a position. Every core domain has them; `Int`, which has no count, does not.
pub trait Cyclic: Ordered + Domain:
    /// `D.wrap(k)`: the position `k` steps into its walk, Euclidean mod `D.count` (R04.2.13).
    fn wrap(k: Int) -> Self = Self.nth((k % Self.count + Self.count) % Self.count)
    /// `v.after`: the piece `v.r..` of its domain (R04.2.19).
    @get fn after(self) -> Range<Self> = range_of(self.ord + 1, Self.count, 1)

impl<D: Ordered + Domain> Cyclic for D

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

impl<T: Index> Discrete for T:
    fn nth(k: Int) -> T:
        check_range(0 <= k and k < T.count)
        T.nth(k)
    @get fn ord(self) -> Int:
        let k = self.ord
        check_range(0 <= k and k < T.count)
        k

impl<T: Index + Discrete> Domain for T:
    @get fn count() -> Int = T.count

/// `Int` is `Discrete`: its integers are its positions (R04.2.18), every one of them, so it has no
/// count and is not a `Domain`.
impl Discrete for Int:
    fn try_nth(k: Int) -> ?Int = Some(k)

impl Ordered for Int:
    fn at(x: Int) -> ?Int = x
    @get fn next(self, k: Int = 1) -> ?Int = Some(self + k)
    @get fn prev(self, k: Int = 1) -> ?Int = Some(self - k)

/// A range type is a domain of its own positions, not `Int`'s: `(1..=3).at(9)` is `None`.
impl Discrete for _..=_

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

impl<K: Domain + Finite, V> MemoTable<K, V> for MemoArray<K, V>:
    fn get(self, k: K) -> ?V = if slots.len == 0: None else: slots[k.ord]
    fn set(mut self, k: K, v: V):
        if slots.len == 0: slots = List.filled(K.count, None)
        slots[k.ord] = Some(v)

/// A cut of `D`: the place between two neighbouring positions, or at either end, with `k`
/// positions before it. `D.cuts` names its type and, as a value, the domain of them (R04.2.5).
/// Its one field is `k`, so it prints as that integer (R04.2.16, R10.3.5).
@member(cuts)
@role(cuts)
pub struct Cut<D: Discrete>:
    k: Int

    /// `c.l`: the position just before `c`, or `None` at the start (R04.3.2).
    @get pub fn l(self) -> ?D:
        D.try_nth(k - 1)

    /// `c.r`: the position just after `c`, or `None` at the end (R04.3.2).
    @get pub fn r(self) -> ?D:
        D.try_nth(k)

/// A domain's cuts are a domain, one more than its positions, walked in order; a cut's integer
/// is how many positions come before it, whatever the base (R04.2.5, R04.2.16, R04.2.17).
impl<D: Discrete> Discrete for Cut<D>:
    fn nth(k: Int) -> Cut<D>:
        check_value(has_cut<D>(k))
        Cut(k)
    fn try_nth(k: Int) -> ?Cut<D> = if has_cut<D>(k): Some(Cut(k)) else: None
    @get fn ord(self) -> Int = k

/// Whether `D` has a cut with `k` positions before it: the first, or one just after a position.
fn has_cut<D: Discrete>(k: Int) -> Bool:
    if k == 0: return true
    if let Some(_) = D.try_nth(k - 1): return true
    false

impl<D: Domain> Domain for Cut<D>:
    @get fn count() -> Int = D.count + 1

/// A domain's cuts are in order, whatever its positions are (R04.2.31).
impl<D: Domain> Ordered for Cut<D>:
    fn at(x: Int) -> ?Cut<D> = if 0 <= x and x <= D.count: Some(Cut(x)) else: None

/// `D.cuts(k)`: the cut with `k` positions before it; traps R0102 past the ends (R04.2.17).
impl<D: Discrete> Into<Cut<D>> for Int:
    fn into(self) -> Cut<D>:
        check_value(has_cut<D>(self))
        Cut(self)

/// `Int(c)`: how many positions come before `c` (R04.2.16).
impl<D: Discrete> Into<Int> for Cut<D>:
    fn into(self) -> Int = k

/// A run's ends as cuts: `[r]T` covers the keys from `r.start` to `r.end` (R04b.1.5).
@role(range_bounds)
pub trait RangeEnds<D: Discrete>:
    fn start(self) -> Cut<D>
    fn end(self) -> Cut<D>

impl<D: Discrete> RangeEnds<D> for Range<D>:
    fn start(self) -> Cut<D> = self.start
    fn end(self) -> Cut<D> = self.end
