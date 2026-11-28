//! The traits that give `==`, `<`, `+`, `-`, `*`, `/`, `%`, `-a`, `^`, `|`, `&`, `in` and `by` their
//! meaning (R08.1.5), `T.zero` and `T.one`, and the numbers' own impls of them. Core re-exports
//! them, so no module imports them.
//!
//! start: `Eq`, `Ord`, `Add`, `Zero`, `min`
//! example: `3 + 4 == 7 and Int.zero == 0 and 2 < 3`
use std.algebra.{Ordering}

/// `a == b` and `a != b` (R08.1.5). Tuples, records, structs and enums whose fields all have
/// it get it field by field unless they write their own (R08.1.6).
@operator("==")
pub trait Eq:
    fn eq(self, other: Self) -> Bool

/// `a < b`, and from it `>`, `<=` and `>=` (R08.1.5). An impl writes `lt` or `cmp`, and the
/// other follows from it, given that `a == b` exactly when neither is less. Derived like `Eq`,
/// lexicographic in declaration order (R08.1.6).
@operator("<")
pub trait Ord(Eq):
    fn lt(self, other: Self) -> Bool = match self.cmp(other):
        Less => true
        _ => false
    fn cmp(self, other: Self) -> Ordering =
        if self < other: Ordering.Less elif self == other: Ordering.Equal else: Ordering.Greater

/// `a + b` on a type that is not a number (R08.1.5); `-`, `*`, `/`, `%`, `**` and `-a` likewise.
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
/// `a ** e`: `a` to the power `e`, an `Int` unless the impl says otherwise.
@operator("**")
pub trait Pow<Rhs = Int>:
    fn pow(self, e: Rhs) -> Self
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

/// `w by k`: every `k`-th value of a walk that is not a range literal, from its first, as a walk
/// of the same type (R01.8.4); `(0..10).rev by 3` is `9, 6, 3, 0`.
@operator("by")
pub trait By<Rhs = Int>:
    fn by(self, k: Rhs) -> Self

/// The numbers' own operators, so `T: Add` holds of them.
/// `T.zero`: the identity of `T`'s `+`, where a sum starts (R08.1.5).
pub trait Zero:
    @get fn zero() -> Self
impl Zero for Int:
    @get fn zero() -> Int = 0
impl Zero for U64:
    @get fn zero() -> U64 = U64(0)
impl Zero for F64:
    @get fn zero() -> F64 = 0.0
/// `T.one`: the identity of `T`'s `*`, where a product starts.
pub trait One:
    @get fn one() -> Self
impl One for Int:
    @get fn one() -> Int = 1
impl One for U64:
    @get fn one() -> U64 = U64(1)
impl One for F64:
    @get fn one() -> F64 = 1.0
/// `T.MIN`, `T.MAX`: the least and greatest values of `T`, so `max` and `min` over it have an
/// identity (R02.2.1, decision 580).
pub trait Bounded:
    @get fn MIN() -> Self
    @get fn MAX() -> Self
/// `Int.MIN` is one above `-2^63`, which is `?Int`'s `None` (R02.1).
impl Bounded for Int:
    @get fn MIN() -> Int = -9223372036854775807
    @get fn MAX() -> Int = 9223372036854775807
impl Bounded for Char:
    @get fn MIN() -> Char = Char(0)
    @get fn MAX() -> Char = Char(127)
impl Bounded for Bool:
    @get fn MIN() -> Bool = false
    @get fn MAX() -> Bool = true

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
impl Zero for I128:
    @get fn zero() -> I128 = I128(0)
impl One for I128:
    @get fn one() -> I128 = I128(1)

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
impl Pow for Int:
    fn pow(self, e: Int) -> Int = self ** e
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
impl Pow for U64:
    fn pow(self, e: Int) -> U64 = self ** e
impl Add for I128:
    fn add(self, other: I128) -> I128 = self + other
impl Sub for I128:
    fn sub(self, other: I128) -> I128 = self - other
impl Mul for I128:
    fn mul(self, other: I128) -> I128 = self * other
impl Div for I128:
    fn div(self, other: I128) -> I128 = self / other
impl Rem for I128:
    fn rem(self, other: I128) -> I128 = self % other
impl Pow for I128:
    fn pow(self, e: Int) -> I128 = self ** e
impl Neg for I128:
    fn neg(self) -> I128 = -self
impl Add for F64:
    fn add(self, other: F64) -> F64 = self + other
impl Sub for F64:
    fn sub(self, other: F64) -> F64 = self - other
impl Mul for F64:
    fn mul(self, other: F64) -> F64 = self * other
impl Div for F64:
    fn div(self, other: F64) -> F64 = self / other
impl Neg for F64:
    fn neg(self) -> F64 = -self
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
impl Eq for I128:
    fn eq(a, b) = a == b
impl Ord for I128:
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

/// A range type's ends are its first and last positions (R02.2.1).
impl Bounded for _..=_:
    @get fn MIN() -> Self = Self.nth(0)
    @get fn MAX() -> Self = Self.nth(Self.count - 1)
