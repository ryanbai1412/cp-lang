//! The traits that give `==`, `<`, `+`, `-`, `*`, `/`, `%`, `-a`, `^`, `|`, `&` and `in` their
//! meaning (R08.1.5), `T.zero` and `T.one`, and the numbers' own impls of them. Core re-exports
//! them, so no module imports them.
//!
//! example: `3 + 4 == 7 and Int.zero == 0 and 2 < 3`
use std.algebra.{Ordering}

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
