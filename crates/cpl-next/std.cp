// The core traits: the language gives `==`, `<` and `+` their meaning through them
// (R08.1.5), so they are declared here rather than in a std file; `Ordering` is std's.
use std.algebra.{Ordering}
use std.seq.{IndexGet, IndexMut, IndexedMut, Keyed, Positional, Indexed, Slice, Seq, Sized}
/// The indexed writes and defaults every module names unimported (R12.4.6, R07.12.9).
pub use std.seq.{IndexMut, IndexedMut, IndexSet}
pub use std.default.{Default}
/// The maps and sets every module names unimported (R07.2.4).
pub use std.map.{Map, Set}
use std.list.{List, Array, Str}
/// The lists, arrays and strings every module names unimported (R02.3, R04.8).
pub use std.list.{List, Array, Str, yes}
use std.optional.{Optional}
/// `?T` and `Some(v)` every module names unimported (R02.4).
pub use std.optional.{Optional, some}
use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
/// Tuples every module names unimported (R02.5).
pub use std.tuple.{Tuple1, Tuple2, Tuple3, Tuple4}
use std.raw.{raw_empty, raw_new, raw_get, raw_set, raw_take, raw_cap, check_key, check_range}

/// Traps R0102 unless `ok`: a value its type doesn't have, as an `Int` converted to an enum that
/// is no variant's value (R02.7), or to a cut past a domain's ends (R04.2.17).
@variant_check
pub fn check_value(ok: Bool) = @intrinsic("value_check")

fn read_array<D: Domain>() -> [D]Int:
    var a: [D]Int = 0
    for v in D: a[v] = read()
    a

fn write_list(xs: List<Int>):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

fn write_array<D: Domain>(xs: [D]Int):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

/// `lo..hi by step` as a value: `n` values from `lo`, `step` apart. It walks both ways, and its
/// `len`, `first` and `last` are O(1) (R01.8.4). `D` is the kind of its ends; `lo` and `by`
/// count in ordinals of `D`, which for `Int` are the integers themselves.
pub struct Range<D: Domain>:
    lo: Int
    n: Int
    by: Int

    @get fn len(self) -> Int:
        n

    /// The cut before its first value, of a range walked forwards by one (R04.3.1).
    @get fn start(self) -> Cut<D> = Cut(lo)

    /// The cut after its last value, of a range walked forwards by one (R04.3.1).
    @get fn end(self) -> Cut<D> = Cut(lo + n)

    /// Whether it holds no position (R04.4.12).
    @get fn is_empty(self) -> Bool:
        n == 0

    @get fn first(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo))

    @get fn last(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo + (n - 1) * by))

    /// The same values, last first.
    @get fn rev(self) -> Range<D>:
        Range(lo + (n - 1) * by, n, -by)

    fn each(self) -> yields D:
        for i in 0..n: yield D.nth(lo + i * by)

    /// Without its first `k` values: a range again (R07.4.11).
    fn drop(self, k: Int = 1) -> Range<D>:
        assert k >= 0, "drop: a negative count"
        let c = if k < n: k else: n
        Range(lo + c * by, n - c, by)

    /// Every `k`-th value, from the first: a range again, so `(3..m).step(2).sum()` is O(1).
    fn step(self, k: 1..) -> Range<D>:
        assert k >= 1, "step(k) requires k >= 1"
        Range(lo, (n + k - 1) / k, by * k)

    /// Its first `k` values: a range again (R07.4.11).
    fn take(self, k: Int) -> Range<D>:
        assert k >= 0, "take: a negative count"
        Range(lo, if k < n: k else: n, by)

    /// How `print` writes it: its values, space-separated.
    fn write(self):
        for i in 0..n:
            if i > 0: write(" ")
            write(D.nth(lo + i * by))

/// `lo..hi by step`; empty when `lo` is past `hi` (R04.4.29).
@range_literal
fn range_of<D: Domain>(lo: Int, hi: Int, step: Int) -> Range<D>:
    assert step > 0, "a range's step is positive"
    let n = if hi <= lo: 0 else: (hi - lo + step - 1) / step
    Range(lo, n, step)

/// `a == b` and `a != b` (R08.1.5). Tuples, records, structs and enums whose fields all have
/// it get it field by field unless they write their own (R08.1.6).
pub trait Eq:
    fn eq(self, other: Self) -> Bool

/// `a < b`, and from it `>`, `<=` and `>=` (R08.1.5). An impl writes `lt` or `cmp`, and the
/// other follows from it, given that `a == b` exactly when neither is less. Derived like `Eq`,
/// lexicographic in declaration order (R08.1.6).
pub trait Ord(Eq):
    fn lt(self, other: Self) -> Bool = match self.cmp(other):
        Less => true
        _ => false
    fn cmp(self, other: Self) -> Ordering =
        if self < other: Ordering.Less elif self == other: Ordering.Equal else: Ordering.Greater

/// `a + b` on a type that is not a number (R08.1.5); `-`, `*`, `/`, `%` and `-a` likewise.
@operator("+")
pub trait Add<Rhs = Self>:
    fn add(self, other: Rhs) -> Self
@operator("-")
pub trait Sub<Rhs = Self>:
    fn sub(self, other: Rhs) -> Self
@operator("*")
pub trait Mul<Rhs = Self>:
    fn mul(self, other: Rhs) -> Self
@operator("/")
pub trait Div<Rhs = Self>:
    fn div(self, other: Rhs) -> Self
@operator("%")
pub trait Rem<Rhs = Self>:
    fn rem(self, other: Rhs) -> Self
@operator("-")
pub trait Neg:
    fn neg(self) -> Self
@operator("^")
pub trait BitXor<Rhs = Self>:
    fn bitxor(self, other: Rhs) -> Self
@operator("|")
pub trait BitOr<Rhs = Self>:
    fn bitor(self, other: Rhs) -> Self
@operator("&")
pub trait BitAnd<Rhs = Self>:
    fn bitand(self, other: Rhs) -> Self

/// `x in s`: whether `s` holds `x`, sub-linear; `x not in s` is its negation (R12.4.7). A
/// linear search is `s.any(x)`.
@operator("in")
pub trait Contains<T>:
    fn contains(self, x: T) -> Bool

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


/// The numbers' own operators, so `T: Add` holds of them.
/// `T.zero`: the identity of `T`'s `+`, where a sum starts (R08.1.5).
pub trait Zero:
    @get fn zero() -> Self
impl Zero for Int:
    @get fn zero() -> Int = 0
impl Zero for U64:
    @get fn zero() -> U64 = U64(0)
/// `T.one`: the identity of `T`'s `*`, where a product starts.
pub trait One:
    @get fn one() -> Self
impl One for Int:
    @get fn one() -> Int = 1
impl One for U64:
    @get fn one() -> U64 = U64(1)

impl Add for Int:
    fn add(self, other: Int) -> Int = self + other
impl Sub for Int:
    fn sub(self, other: Int) -> Int = self - other
impl Mul for Int:
    fn mul(self, other: Int) -> Int = self * other
impl Div for Int:
    fn div(self, other: Int) -> Int = self / other
impl Rem for Int:
    fn rem(self, other: Int) -> Int = self % other
impl Neg for Int:
    fn neg(self) -> Int = -self
impl Add for U64:
    fn add(self, other: U64) -> U64 = self + other
impl Sub for U64:
    fn sub(self, other: U64) -> U64 = self - other
impl Mul for U64:
    fn mul(self, other: U64) -> U64 = self * other
impl Div for U64:
    fn div(self, other: U64) -> U64 = self / other
impl Rem for U64:
    fn rem(self, other: U64) -> U64 = self % other
impl BitXor for Int:
    fn bitxor(self, other: Int) -> Int = self ^ other
impl BitOr for Int:
    fn bitor(self, other: Int) -> Int = self | other
impl BitAnd for Int:
    fn bitand(self, other: Int) -> Int = self & other
impl BitXor for U64:
    fn bitxor(self, other: U64) -> U64 = self ^ other
impl BitOr for U64:
    fn bitor(self, other: U64) -> U64 = self | other
impl BitAnd for U64:
    fn bitand(self, other: U64) -> U64 = self & other

/// Positions of one domain compare in its walk order (R04.2).
impl<D: Domain> Eq for D:
    fn eq(a, b) = a.ord == b.ord
impl<D: Domain + Eq> Ord for D:
    fn lt(a, b) = a.ord < b.ord


/// `x.clone()`: the explicit deep copy, O(size) (R05.2.2). Tuples, records and structs whose
/// fields all have it get it field by field unless they write their own (R08.1.6).
@derive
pub trait Clone:
    fn clone(self) -> Self

impl Clone for Int:
    fn clone(a) = a
impl Clone for Char:
    fn clone(a) = a
impl Clone for U64:
    fn clone(a) = a
impl Clone for F64:
    fn clone(a) = a
impl Clone for Bool:
    fn clone(a) = a
impl<D: Domain> Clone for D:
    fn clone(a) = a
impl<D: Domain> Clone for Range<D>:
    fn clone(a) = Range(a.lo, a.n, a.by)

/// The largest of two or more values, the first of equal ones (R11.3.3).
pub fn max<T: Ord, Ts: Alike<T>>(a: T, b: T, ..more: Ts) -> T = more.max_with(if b > a: b else: a)

/// The smallest of two or more values, the first of equal ones (R11.3.3).
pub fn min<T: Ord, Ts: Alike<T>>(a: T, b: T, ..more: Ts) -> T = more.min_with(if b < a: b else: a)

/// A tuple whose fields are all `T`: what `max` and `min` take past their first two values.
pub trait Alike<T>:
    /// The largest of `m` and the fields, the first of equal ones.
    fn max_with(self, m: T) -> T
    /// The smallest of `m` and the fields, the first of equal ones.
    fn min_with(self, m: T) -> T
impl<T: Ord> Alike<T> for ():
    fn max_with(self, m: T) -> T = m
    fn min_with(self, m: T) -> T = m

/// Numbers, characters (by code) and booleans (`false < true`) compare as the machine does.
impl Eq for Int:
    fn eq(a, b) = a == b
impl Ord for Int:
    fn lt(a, b) = a < b
impl Eq for Char:
    fn eq(a, b) = a == b
impl Ord for Char:
    fn lt(a, b) = a < b
impl Eq for U64:
    fn eq(a, b) = a == b
impl Ord for U64:
    fn lt(a, b) = a < b
/// `==` on `F64` is exact; `close(a, b, eps:)` is the tolerant compare (R02.1).
impl Eq for F64:
    fn eq(a, b) = a == b
impl Ord for F64:
    fn lt(a, b) = a < b
impl Eq for Bool:
    fn eq(a, b) = a == b
impl Ord for Bool:
    fn lt(a, b) = not a and b

/// `T(x)` for a core type `T`: `x` as a `T`, by `x`'s impl (R04.2.28).
@convert
pub trait Into<T>:
    fn into(self) -> T

/// A character's code, and the character with a code, which traps R0201 outside `0..=127`.
impl Into<Int> for Char:
    fn into(self) -> Int = @intrinsic("code_of")
impl Into<Char> for Int:
    fn into(self) -> Char = @intrinsic("char_of")
/// An `Int`'s two's-complement bits, and a `U64` as an `Int`, which traps past `Int` (R03.4.1).
impl Into<U64> for Int:
    fn into(self) -> U64 = @intrinsic("u64_of")
impl Into<Int> for U64:
    fn into(self) -> Int = @intrinsic("int_of_u64")
impl Into<Int> for Int:
    fn into(self) -> Int = self
/// `true` is 1 and `false` 0 (R02.2.1).
impl Into<Int> for Bool:
    fn into(self) -> Int = if self: 1 else: 0
impl Into<U64> for U64:
    fn into(self) -> U64 = self
/// The `F64` nearest an `Int` (R08.5.2).
impl Into<F64> for Int:
    fn into(self) -> F64 = @intrinsic("f64_of")
impl Into<F64> for F64:
    fn into(self) -> F64 = self

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

