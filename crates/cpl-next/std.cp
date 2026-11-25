// The core traits: the language gives `==`, `<` and `+` their meaning through them
// (R08.1.5), so they are declared here rather than in a std file; `Ordering` is std's.
use std.algebra.{Ordering}

fn raw_empty<T>() -> Raw<T> = @intrinsic("raw_empty")
fn raw_new<T>(n: Int, fill: T) -> Raw<T> = @intrinsic("raw_new")
fn raw_get<T>(r: Raw<T>, i: Int) -> T = @intrinsic("raw_get")
fn raw_set<T>(mut r: Raw<T>, i: Int, x: T) = @intrinsic("raw_set")

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
        assert 0 <= i and i < n, "index out of range"
        raw_get(buf, i)

    fn set(mut self, i: Int, x: T):
        assert 0 <= i and i < n, "index out of range"
        raw_set(mut buf, i, x)

    fn each(self) -> yields T:
        for i in 0..n: yield raw_get(buf, i)

    fn push(mut self, x: T):
        if n == cap:
            let ncap = if cap == 0: 4 else: cap * 2
            var nb = raw_new(ncap, x)
            for i in 0..n: raw_set(mut nb, i, raw_get(buf, i))
            buf = nb
            cap = ncap
        raw_set(mut buf, n, x)
        n += 1


struct Array<D, T>:
    buf: Raw<T>

    fn filled(x: T) -> Array<D, T>:
        Array(raw_new(D.count, x))

    fn get(self, i: D) -> T:
        raw_get(buf, i.ord)

    fn set(mut self, i: D, x: T):
        raw_set(mut buf, i.ord, x)

    fn each(self) -> yields T:
        for i in 0..D.count: yield raw_get(buf, i)

fn read_array<D>() -> [D]Int:
    var a: [D]Int = 0
    for v in D: a[v] = read()
    a

fn write_list(xs: List<Int>):
    var first = true
    for x in xs:
        if not first: write(" ")
        write(x)
        first = false

fn write_array<D>(xs: [D]Int):
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

    fn set(mut self, i: Int, c: Char):
        chars.set(i, c)

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
/// `len`, `first` and `last` are O(1) (R01.8.4). `D` is the kind of its ends; `lo` and `step`
/// count in ordinals of `D`, which for `Int` are the integers themselves.
struct Range<D>:
    lo: Int
    n: Int
    step: Int

    @get fn len(self) -> Int:
        n

    @get fn first(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo))

    @get fn last(self) -> ?D:
        if n == 0: None else: Some(D.nth(lo + (n - 1) * step))

    /// The same values, last first.
    @get fn rev(self) -> Range<D>:
        Range(lo + (n - 1) * step, n, -step)

    fn each(self) -> yields D:
        for i in 0..n: yield D.nth(lo + i * step)

    /// How `print` writes it: its values, space-separated.
    fn write(self):
        for i in 0..n:
            if i > 0: write(" ")
            write(D.nth(lo + i * step))

/// `lo..hi by step`; empty when `lo` is past `hi` (R04.4.29).
fn range_of<D>(lo: Int, hi: Int, step: Int) -> Range<D>:
    assert step > 0, "a range's step is positive"
    let n = if hi <= lo: 0 else: (hi - lo + step - 1) / step
    Range(lo, n, step)

/// `Some(v)`: `v`, present (R01.5.6).
fn some<T>(v: T) -> ?T = v

struct Tuple2<A, B>:
    _0: A
    _1: B

struct Tuple3<A, B, C>:
    _0: A
    _1: B
    _2: C

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
trait Add<Rhs = Self>:
    fn add(self, other: Rhs) -> Self
trait Sub<Rhs = Self>:
    fn sub(self, other: Rhs) -> Self
trait Mul<Rhs = Self>:
    fn mul(self, other: Rhs) -> Self
trait Div<Rhs = Self>:
    fn div(self, other: Rhs) -> Self
trait Rem<Rhs = Self>:
    fn rem(self, other: Rhs) -> Self
trait Neg:
    fn neg(self) -> Self

/// The positions of a domain (R04.2): `D: Domain` takes any domain, and every domain has it.
@positions
trait Domain

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

/// Numbers, characters (by code) and booleans (`false < true`) compare as the machine does.
impl Eq for Int:
    fn eq(a, b) = a == b
impl Ord for Int:
    fn lt(a, b) = a < b
impl Eq for Char:
    fn eq(a, b) = a == b
impl Ord for Char:
    fn lt(a, b) = a < b
impl Eq for Bool:
    fn eq(a, b) = a == b
impl Ord for Bool:
    fn lt(a, b) = not a and b
