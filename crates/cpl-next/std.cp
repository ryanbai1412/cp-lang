// The core traits: the language gives `==`, `<` and `+` their meaning through them
// (R08.1.5), so they are declared here rather than in a std file; `Ordering` is std's.
use std.algebra.{Ordering}
use std.seq.{Keyed, Positional, Indexed, Slice, Seq, Sized}

fn raw_empty<T>() -> Raw<T> = @intrinsic("raw_empty")
fn raw_new<T>(n: Int, fill: T) -> Raw<T> = @intrinsic("raw_new")
fn raw_get<T>(r: Raw<T>, i: Int) -> T = @intrinsic("raw_get")
/// Traps R0101 unless `ok`: a read at a key the sequence doesn't have (R07.2.17).
fn check_key(ok: Bool) = @intrinsic("key_check")
fn check_range(ok: Bool) = @intrinsic("range_check")
/// Traps R0102 unless `ok`: a value its type doesn't have, as an `Int` converted to an enum that
/// is no variant's value (R02.7), or to a cut past a domain's ends (R04.2.17).
@variant_check
fn check_value(ok: Bool) = @intrinsic("value_check")
fn raw_set<T>(mut r: Raw<T>, i: Int, x: T) = @intrinsic("raw_set")
/// Moves cell `i` out of `r`, leaving it unset until `raw_set` fills it.
fn raw_take<T>(mut r: Raw<T>, i: Int) -> T = @intrinsic("move_out")
/// Storage of `n` unset cells; each is filled by `raw_set` before it is read.
fn raw_cap<T>(n: Int) -> Raw<T> = @intrinsic("unset_cells")

struct List<T>:
    var buf: Raw<T>
    var n: Int
    var cap: Int

    fn new() -> List<T>:
        List(raw_empty(), 0, 0)

    fn filled(n: Int, x: T) -> List<T>:
        List(raw_new(n, x), n, n)

    @get fn len(self) -> Int:
        n

    fn get(self, i: Int) -> T:
        check_key(0 <= i and i < n)
        raw_get(buf, i)

    fn each(self) -> yields T:
        for i in 0..n: yield raw_get(buf, i)

    /// Room for `k` elements without another move of the ones held.
    fn reserve(mut self, k: Int):
        if k > cap:
            var nb = raw_cap(k)
            for i in 0..n: raw_set(mut nb, i, raw_take(mut buf, i))
            buf = nb
            cap = k

    fn push(mut self, x: T):
        if n == cap: self.reserve(if cap == 0: 4 else: cap * 2)
        raw_set(mut buf, n, x)
        n += 1

    fn swap(mut self, i: Int, j: Int):
        check_key(0 <= i and i < n and 0 <= j and j < n)
        if i != j:
            let x = raw_take(mut buf, i)
            raw_set(mut buf, i, raw_take(mut buf, j))
            raw_set(mut buf, j, x)

    /// `x` at position `i`, the elements from `i` on one later.
    fn insert(mut self, i: Int, x: T):
        assert 0 <= i and i <= n, "insert: a position past the end"
        self.push(x)
        var k = n - 1
        while k > i:
            self.swap(k - 1, k)
            k -= 1

    /// The element at `i`, the ones after it one earlier.
    fn remove(mut self, i: Int) -> T:
        check_key(0 <= i and i < n)
        for k in i..n - 1: self.swap(k, k + 1)
        n -= 1
        raw_take(mut buf, n)

    fn extend(mut self, xs: List<T>):
        self.reserve(n + xs.len)
        for x in xs: self.push(x)

    @get fn is_empty(self) -> Bool:
        n == 0

    fn pop(mut self) -> ?T:
        if n == 0: return None
        n -= 1
        Some(raw_get(buf, n))

    fn truncate(mut self, k: Int):
        check_key(0 <= k)
        if k < n: n = k

    fn clear(mut self):
        n = 0


struct Array<D: Domain, T>:
    buf: Raw<T>

    @get fn len(self) -> Int:
        D.count

    fn filled(x: T) -> Array<D, T>:
        Array(raw_new(D.count, x))

    fn get(self, i: D) -> T:
        raw_get(buf, i.ord)

    fn each(self) -> yields T:
        for i in 0..D.count: yield raw_get(buf, i)

/// Cell by cell, in walk order (R08.1.6).
impl<D: Domain, T: Eq> Eq for Array<D, T>:
    fn eq(a, b):
        for i in 0..D.count:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return false
        true

/// A list where `[D]T` is wanted: one element per position, in walk order (R04.8.10).
@coerce
fn array_of_list<D: Domain, T>(xs: List<T>) -> [D]T:
    assert xs.len == D.count, "a list for `[D]T` has one element per position"
    Array(xs.buf)

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

// A flat sequence of `Char` (R02.3).
struct Str:
    var chars: List<Char>

    @get fn len(self) -> Int:
        chars.len

    fn get(self, i: Int) -> Char:
        chars.get(i)

    fn each(self) -> yields Char:
        for c in chars: yield c

    fn push(mut self, c: Char):
        chars.push(c)

    /// How `print` writes it: its characters.
    fn write(self):
        for c in chars: write(c)

// A string literal: its characters and how many there are. The literal's storage is lent,
// so the string gets its own.
fn str_of(text: Raw<Char>, n: Int) -> Str:
    var chars = List<Char>.new()
    for i in 0..n: chars.push(raw_get(text, i))
    Str(chars)

fn str_concat(a: Str, b: Str) -> Str:
    var out = Str(List.new())
    for c in a: out.push(c)
    for c in b: out.push(c)
    out

/// Strings compare character by character, a prefix first (R08.1.5).
impl Eq for Str:
    fn eq(a, b) = str_eq(a, b)
impl Ord for Str:
    fn lt(a, b):
        let n = if a.len < b.len: a.len else: b.len
        for i in 0..n:
            if a.get(i) != b.get(i): return a.get(i) < b.get(i)
        a.len < b.len

fn str_eq(a: Str, b: Str) -> Bool:
    if a.len != b.len: return false
    for i in 0..a.len:
        if a.get(i) != b.get(i): return false
    true

fn yes(b: Bool) -> Str:
    if b: "YES" else: "NO"

// `?T`: `T` or `None`. `val` is read only when `some`.
struct Optional<T>:
    some: Bool
    val: T

/// `lo..hi by step` as a value: `n` values from `lo`, `step` apart. It walks both ways, and its
/// `len`, `first` and `last` are O(1) (R01.8.4). `D` is the kind of its ends; `lo` and `by`
/// count in ordinals of `D`, which for `Int` are the integers themselves.
struct Range<D: Domain>:
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
fn range_of<D: Domain>(lo: Int, hi: Int, step: Int) -> Range<D>:
    assert step > 0, "a range's step is positive"
    let n = if hi <= lo: 0 else: (hi - lo + step - 1) / step
    Range(lo, n, step)

/// `Some(v)`: `v`, present (R01.5.6); `Some(p)` matches it (R01.7.1).
@pattern
fn some<T>(v: T) -> ?T = v

@tuple
struct Tuple2<A, B>:
    _0: A
    _1: B

@tuple
struct Tuple3<A, B, C>:
    _0: A
    _1: B
    _2: C

@tuple
struct Tuple4<A, B, C, E>:
    _0: A
    _1: B
    _2: C
    _3: E

/// `a == b` and `a != b` (R08.1.5). Tuples, records, structs and enums whose fields all have
/// it get it field by field unless they write their own (R08.1.6).
trait Eq:
    fn eq(self, other: Self) -> Bool

/// `a < b`, and from it `>`, `<=` and `>=` (R08.1.5). An impl writes `lt` or `cmp`, and the
/// other follows from it, given that `a == b` exactly when neither is less. Derived like `Eq`,
/// lexicographic in declaration order (R08.1.6).
trait Ord(Eq):
    fn lt(self, other: Self) -> Bool = match self.cmp(other):
        Less => true
        _ => false
    fn cmp(self, other: Self) -> Ordering =
        if self < other: Ordering.Less elif self == other: Ordering.Equal else: Ordering.Greater

/// `a + b` on a type that is not a number (R08.1.5); `-`, `*`, `/`, `%` and `-a` likewise.
@operator("+")
trait Add<Rhs = Self>:
    fn add(self, other: Rhs) -> Self
@operator("-")
trait Sub<Rhs = Self>:
    fn sub(self, other: Rhs) -> Self
@operator("*")
trait Mul<Rhs = Self>:
    fn mul(self, other: Rhs) -> Self
@operator("/")
trait Div<Rhs = Self>:
    fn div(self, other: Rhs) -> Self
@operator("%")
trait Rem<Rhs = Self>:
    fn rem(self, other: Rhs) -> Self
@operator("-")
trait Neg:
    fn neg(self) -> Self

/// The positions of a domain (R04.2): `D: Domain` takes any domain, and every domain has it.
@positions
trait Domain:
    /// `D.count`: how many positions it has.
    @get fn count() -> Int = @intrinsic("dom_count")
    /// `D.all`: every position, in walk order (R04.2.30). A value's `s.all(p)` is another member.
    @get fn all() -> Range<Self> = range_of(0, Self.count, 1)
    /// `D.nth(k)`: the position `k` steps into its walk; traps past its end (R04.2.9).
    fn nth(k: Int) -> Self = @intrinsic("dom_nth")
    /// `D.at(x)`: the position with integer `x`, or `None` outside it (R04.2.8).
    fn at(x: Int) -> ?Self = @intrinsic("dom_at")
    /// `v.ord`: how many positions come before `v` in the walk (R04.2.15).
    @get fn ord(self) -> Int = @intrinsic("dom_ord")
    /// `v.next(k)`: the position `k` after `v` in the walk, or `None` past its end (R04.3.1).
    @get fn next(self, k: Int = 1) -> ?Self:
        let o = self.ord + k
        if 0 <= o and o < Self.count: Some(Self.nth(o)) else: None
    /// `v.prev(k)`: the position `k` before `v` in the walk, or `None` past its start (R04.3.1).
    @get fn prev(self, k: Int = 1) -> ?Self:
        let o = self.ord - k
        if 0 <= o and o < Self.count: Some(Self.nth(o)) else: None
    /// `v.l`: the cut just before `v` (R04.3.1).
    @get fn l(self) -> Cut<Self> = Cut(self.ord)
    /// `v.r`: the cut just after `v` (R04.3.1).
    @get fn r(self) -> Cut<Self> = Cut(self.ord + 1)

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
struct Cut<D: Domain>:
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
trait Zero:
    @get fn zero() -> Self
impl Zero for Int:
    @get fn zero() -> Int = 0
impl Zero for U64:
    @get fn zero() -> U64 = U64(0)

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

/// Positions of one domain compare in its walk order (R04.2).
impl<D: Domain> Eq for D:
    fn eq(a, b) = a.ord == b.ord
impl<D: Domain + Eq> Ord for D:
    fn lt(a, b) = a.ord < b.ord

/// Both empty, or both holding equal values; `?T` is not `Ord` (R02.4.5).
impl<T: Eq> Eq for Optional<T>:
    fn eq(a, b) = if not a.some: not b.some elif not b.some: false else: a.val == b.val

/// The same length and equal values in order.
impl<T: Eq> Eq for List<T>:
    fn eq(a, b):
        if a.n != b.n: return false
        for i in 0..a.n:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return false
        true
/// Lists compare element by element, a prefix first (R08.1.5).
impl<T: Ord> Ord for List<T>:
    fn lt(a, b):
        let n = if a.n < b.n: a.n else: b.n
        for i in 0..n:
            if raw_get(a.buf, i) != raw_get(b.buf, i): return raw_get(a.buf, i) < raw_get(b.buf, i)
        a.n < b.n

/// Tuples compare field by field, lexicographically for `Ord` (R08.1.6).
impl<A: Eq, B: Eq> Eq for Tuple2<A, B>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1
impl<A: Ord, B: Ord> Ord for Tuple2<A, B>:
    fn lt(a, b) = a._0 < b._0 or a._0 == b._0 and a._1 < b._1
impl<A: Eq, B: Eq, C: Eq> Eq for Tuple3<A, B, C>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2
impl<A: Ord, B: Ord, C: Ord> Ord for Tuple3<A, B, C>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and a._2 < b._2)
impl<A: Eq, B: Eq, C: Eq, E: Eq> Eq for Tuple4<A, B, C, E>:
    fn eq(a, b) = a._0 == b._0 and a._1 == b._1 and a._2 == b._2 and a._3 == b._3
impl<A: Ord, B: Ord, C: Ord, E: Ord> Ord for Tuple4<A, B, C, E>:
    fn lt(a, b) =
        a._0 < b._0 or a._0 == b._0 and (a._1 < b._1 or a._1 == b._1 and (a._2 < b._2 or a._2 == b._2 and a._3 < b._3))

/// `x.clone()`: the explicit deep copy, O(size) (R05.2.2). Tuples, records and structs whose
/// fields all have it get it field by field unless they write their own (R08.1.6).
@derive
trait Clone:
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
impl<T: Clone> Clone for Optional<T>:
    fn clone(a) = if a.some: Optional(true, a.val.clone()) else: None
/// A new buffer holding a clone of each element.
impl<T: Clone> Clone for List<T>:
    fn clone(a):
        if a.n == 0: return List.new()
        var out = List.filled(a.n, raw_get(a.buf, 0).clone())
        for i in 1..a.n: raw_set(mut out.buf, i, raw_get(a.buf, i).clone())
        out
impl Clone for Str:
    fn clone(a) = Str(a.chars.clone())
impl<D: Domain, T: Clone> Clone for Array<D, T>:
    fn clone(a):
        if D.count == 0: return Array(raw_empty())
        var buf = raw_new(D.count, raw_get(a.buf, 0).clone())
        for i in 1..D.count: raw_set(mut buf, i, raw_get(a.buf, i).clone())
        Array(buf)
impl<A: Clone, B: Clone> Clone for Tuple2<A, B>:
    fn clone(a) = Tuple2(a._0.clone(), a._1.clone())
impl<A: Clone, B: Clone, C: Clone> Clone for Tuple3<A, B, C>:
    fn clone(a) = Tuple3(a._0.clone(), a._1.clone(), a._2.clone())
impl<A: Clone, B: Clone, C: Clone, E: Clone> Clone for Tuple4<A, B, C, E>:
    fn clone(a) = Tuple4(a._0.clone(), a._1.clone(), a._2.clone(), a._3.clone())

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
trait Into<T>:
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

/// A list, a string and an array read in O(1) at any key, by position from 0 or, for an
/// array, by its domain's positions (R12.4.5).
impl<T> Positional<T> for List<T>:
    fn nth(self, k: Int) -> T = self.get(k)
impl<T> Indexed<Int, T> for List<T>:
    fn at(self, i: Int) -> T = self.get(i)
    @get fn keys(self) -> Range<Int> = 0..self.len
impl Positional<Char> for Str:
    fn nth(self, k: Int) -> Char = self.get(k)
impl Indexed<Int, Char> for Str:
    fn at(self, i: Int) -> Char = self.get(i)
    @get fn keys(self) -> Range<Int> = 0..self.len
/// `s[k]` as a place: a write through it lands in `s`, in place (R12.4.6).
pub trait IndexedMut<K, T>: Indexed<K, T>:
    fn at(mut self, k: K) -> lends mut T
impl<T> IndexedMut<Int, T> for List<T>:
    fn at(mut self, i: Int) -> lends mut T:
        check_key(0 <= i and i < self.n)
        lend mut self.buf[i]
impl IndexedMut<Int, Char> for Str:
    fn at(mut self, i: Int) -> lends mut Char:
        check_key(0 <= i and i < self.chars.n)
        lend mut self.chars.buf[i]
impl<D: Domain, T> Keyed<D, T> for Array<D, T>:
    fn items(self) -> yields (D, T):
        for i in 0..D.count: yield (D.nth(i), raw_get(self.buf, i))
impl<D: Domain, T> Positional<T> for Array<D, T>:
    fn nth(self, k: Int) -> T = self.get(D.nth(k))
impl<D: Domain, T> Indexed<D, T> for Array<D, T>:
    fn at(self, i: D) -> T = self.get(i)
    @get fn keys(self) -> Range<D> = D
/// A range is keyed by its own values, so `(2..5)[3] == 3`, and has no source to lend (R07.2.15).
impl<D: Domain, T> IndexedMut<D, T> for Array<D, T>:
    fn at(mut self, i: D) -> lends mut T:
        lend mut self.buf[i.ord]
impl<D: Domain> Keyed<D, D> for Range<D>:
    fn items(self) -> yields (D, D):
        for i in 0..self.n:
            let k = self.lo + i * self.by
            yield (D.nth(k), D.nth(k))
impl<D: Domain> Positional<D> for Range<D>:
    fn nth(self, k: Int) -> D:
        check_key(0 <= k and k < self.n)
        D.nth(self.lo + k * self.by)
impl<D: Domain> Indexed<D, D> for Range<D>:
    fn at(self, x: D) -> D:
        let d = x.ord - self.lo
        check_key(self.n > 0 and d % self.by == 0 and 0 <= d / self.by and d / self.by < self.n)
        x
    @get fn keys(self) -> Range<D> = self
/// `s[r]`: the view of `s` over the keys in `r`, made in O(1); traps R0101 unless `r` is a run of
/// `s`'s keys (R07.4.6). An empty `r` is the empty view at its start.
@index fn slice<S: Indexed<D, T>, D: Domain, T>(self: S, r: Range<D>) -> Slice<S>:
    let (i, ok) = run_in(self.keys, r)
    check_key(ok)
    Slice(self, i, i + r.n)
/// `s[lo..]`, `s[..hi]`, `s[..=hi]` and `s[..]`: an end left out is that end of `s`'s keys; traps
/// R0101 unless the ends are keys of `s` in order, or its end (R07.4.6).
@index fn slice_ends<S: Indexed<D, T>, D: Domain, T>(self: S, lo: ?D, hi: ?D, through: Bool) -> Slice<S>:
    let (i, j, ok) = cut_ends(self.keys, lo, hi, through)
    check_key(ok)
    Slice(self, i, j)
/// `r[a..b]` of a range is the sub-range `a..b`, itself a range: a range is keyed by its own
/// values and has no source to view (R07.2.15). Traps R0001 unless `a..b` is a run of `r`'s keys.
@index fn sub_range<D: Domain>(self: Range<D>, r: Range<D>) -> Range<D>:
    let (i, ok) = run_in(self, r)
    check_range(ok)
    Range(self.lo + i * self.by, r.n, self.by)
/// `r[a..]`, `r[..b]`, `r[..=b]` and `r[..]` of a range: the sub-range between the ends, an end
/// left out being `r`'s. Traps R0001 unless the ends are keys of `r` in order, or its end.
@index fn sub_range_ends<D: Domain>(self: Range<D>, lo: ?D, hi: ?D, through: Bool) -> Range<D>:
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

/// The balanced search tree under `Set` and `Map`: an AVL tree whose nodes are slots of its
/// lists, so a removed node's slot is reused. Keys are in `<` order, each once.
struct OrdTree<K, V>:
    var keys: List<K> = []
    var vals: List<V> = []
    var left: List<Int> = []
    var right: List<Int> = []
    var height: List<Int> = []
    var free: List<Int> = []
    var root: Int = -1
    var n: Int = 0

impl<K, V> OrdTree<K, V>:
    fn key_at(self, t: Int) -> ?K:
        if t < 0: None else: Some(keys[t])

    /// The node of the least key, or with `last` the greatest; -1 when empty.
    fn edge(self, last: Bool) -> Int:
        var t = root
        while t >= 0:
            let c = if last: right[t] else: left[t]
            if c < 0: return t
            t = c
        t

    /// Walks the nodes in key order.
    fn nodes(self) -> yields Int:
        var stack: List<Int> = []
        var t = root
        while t >= 0 or not stack.is_empty:
            while t >= 0:
                stack.push(t)
                t = left[t]
            let top = stack.pop() ?? -1
            yield top
            t = right[top]

    fn clear(mut self):
        keys.clear()
        vals.clear()
        left.clear()
        right.clear()
        height.clear()
        free.clear()
        root = -1
        n = 0

impl<K: Ord, V> OrdTree<K, V>:
    fn h(self, t: Int) -> Int:
        if t < 0: 0 else: height[t]

    fn fix(mut self, t: Int):
        let a = self.h(left[t])
        let b = self.h(right[t])
        height[t] = if a < b: b + 1 else: a + 1

    fn rot_right(mut self, t: Int) -> Int:
        let l = left[t]
        left[t] = right[l]
        right[l] = t
        self.fix(t)
        self.fix(l)
        l

    fn rot_left(mut self, t: Int) -> Int:
        let r = right[t]
        right[t] = left[r]
        left[r] = t
        self.fix(t)
        self.fix(r)
        r

    fn balance(mut self, t: Int) -> Int:
        self.fix(t)
        let b = self.h(left[t]) - self.h(right[t])
        if b > 1:
            let l = left[t]
            if self.h(left[l]) < self.h(right[l]):
                let c = self.rot_left(l)
                left[t] = c
            return self.rot_right(t)
        if b < -1:
            let r = right[t]
            if self.h(right[r]) < self.h(left[r]):
                let c = self.rot_right(r)
                right[t] = c
            return self.rot_left(t)
        t

    fn node(mut self, k: K, v: V) -> Int:
        n += 1
        if let i = free.pop():
            keys[i] = k
            vals[i] = v
            left[i] = -1
            right[i] = -1
            height[i] = 1
            return i
        keys.push(k)
        vals.push(v)
        left.push(-1)
        right.push(-1)
        height.push(1)
        keys.len - 1

    fn ins(mut self, t: Int, k: K, v: V) -> Int:
        if t < 0: return self.node(k, v)
        if k < keys[t]:
            let c = self.ins(left[t], k, v)
            left[t] = c
        elif keys[t] < k:
            let c = self.ins(right[t], k, v)
            right[t] = c
        else:
            vals[t] = v
            return t
        self.balance(t)

    /// Sets `k`'s value to `v`, adding `k` when it is absent.
    fn put(mut self, k: K, v: V):
        let r = self.ins(root, k, v)
        root = r

    /// The node holding `k`, or -1.
    fn find(self, k: K) -> Int:
        var t = root
        while t >= 0:
            if k < keys[t]: t = left[t]
            elif keys[t] < k: t = right[t]
            else: return t
        -1

    /// The node of the greatest key below `k` (`eq`: or equal to it), or -1.
    fn below(self, k: K, eq: Bool) -> Int:
        var t = root
        var best = -1
        while t >= 0:
            if keys[t] < k or (eq and not (k < keys[t])):
                best = t
                t = right[t]
            else:
                t = left[t]
        best

    /// The node of the least key above `k` (`eq`: or equal to it), or -1.
    fn above(self, k: K, eq: Bool) -> Int:
        var t = root
        var best = -1
        while t >= 0:
            if k < keys[t] or (eq and not (keys[t] < k)):
                best = t
                t = left[t]
            else:
                t = right[t]
        best

    fn cut_min(mut self, t: Int) -> Int:
        if left[t] < 0: return right[t]
        let c = self.cut_min(left[t])
        left[t] = c
        self.balance(t)

    fn cut_max(mut self, t: Int) -> Int:
        if right[t] < 0: return left[t]
        let c = self.cut_max(right[t])
        right[t] = c
        self.balance(t)

    /// Removes the node of the least key, or with `last` the greatest; that node, or -1. Its
    /// key and value stay readable until the next add.
    fn pop_edge(mut self, last: Bool) -> Int:
        let i = self.edge(last)
        if i < 0: return -1
        let r = if last: self.cut_max(root) else: self.cut_min(root)
        root = r
        n -= 1
        free.push(i)
        i

    fn del(mut self, t: Int, k: K) -> Int:
        if t < 0: return t
        if k < keys[t]:
            let c = self.del(left[t], k)
            left[t] = c
        elif keys[t] < k:
            let c = self.del(right[t], k)
            right[t] = c
        else:
            n -= 1
            free.push(t)
            if left[t] < 0: return right[t]
            if right[t] < 0: return left[t]
            var s = right[t]
            while left[s] >= 0: s = left[s]
            let c = self.cut_min(right[t])
            right[s] = c
            left[s] = left[t]
            return self.balance(s)
        self.balance(t)

    /// Removes `k`; whether it was there.
    fn drop(mut self, k: K) -> Bool:
        if self.find(k) < 0: return false
        let r = self.del(root, k)
        root = r
        true

/// An ordered set: its elements in `<` order, each once; `O(log n)` to add, remove and find one
/// (R07.2.4).
pub struct Set<T>:
    var t: OrdTree<T, ()> = OrdTree()

impl<T> Set<T>:
    @get pub fn len(self) -> Int:
        t.n

    @get pub fn is_empty(self) -> Bool:
        t.n == 0

    /// The least element.
    @get pub fn first(self) -> ?T:
        t.key_at(t.edge(false))

    /// The greatest element.
    @get pub fn last(self) -> ?T:
        t.key_at(t.edge(true))

    pub fn min(self) -> ?T:
        t.key_at(t.edge(false))

    pub fn max(self) -> ?T:
        t.key_at(t.edge(true))

    pub fn clear(mut self):
        t.clear()

    /// The elements in order.
    pub fn sorted(self) -> List<T>:
        var out: List<T> = []
        for i in t.nodes(): out.push(t.keys[i])
        out

impl<T: Ord> Set<T>:
    /// Adds `x`; whether it was new.
    pub fn insert(mut self, x: T) -> Bool:
        if t.find(x) >= 0: return false
        t.put(x, ())
        true

    /// Adds `x`; whether it was new.
    pub fn add(mut self, x: T) -> Bool:
        self.insert(x)

    /// Removes `x`; whether it was there.
    pub fn remove(mut self, x: T) -> Bool:
        t.drop(x)

    /// Adds `x` when `on`, else removes it.
    pub fn set(mut self, x: T, on: Bool):
        if on: t.put(x, ()) else: t.drop(x)

    pub fn contains(self, x: T) -> Bool:
        t.find(x) >= 0

    pub fn first_ge(self, x: T) -> ?T:
        t.key_at(t.above(x, true))

    pub fn first_gt(self, x: T) -> ?T:
        t.key_at(t.above(x, false))

    pub fn last_le(self, x: T) -> ?T:
        t.key_at(t.below(x, true))

    pub fn last_lt(self, x: T) -> ?T:
        t.key_at(t.below(x, false))

    /// Removes and returns the least element.
    pub fn pop_first(mut self) -> ?T:
        let i = t.pop_edge(false)
        t.key_at(i)

    /// Removes and returns the greatest element.
    pub fn pop_last(mut self) -> ?T:
        let i = t.pop_edge(true)
        t.key_at(i)

impl<T> Seq<T> for Set<T>:
    fn iter(self) -> yields T:
        for i in t.nodes(): yield t.keys[i]

impl<T> Sized for Set<T>:
    @get fn len(self) -> Int = t.n
