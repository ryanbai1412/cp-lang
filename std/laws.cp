//! The values `cpl laws` tries a law's arguments with. Each argument is tried with every value
//! its type's `samples()` lists, so a law over a type of your own runs once it says its samples.
//!
//! start: `Sample`, `Sample.samples`, `samples_of`
//! example: `Bool.samples() == [false, true]`

/// A type `cpl laws` tries a law's argument with: a short fixed list holding the values a law
/// most often breaks at (zero, one, signs, the ends).
pub trait Sample:
    /// The values an argument of this type is tried with, in the order they are tried.
    fn samples() -> List<Self>

/// Small values of each sign, a power of two, and the common modulus and its negation.
impl Sample for Int:
    fn samples() -> List<Int> = [0, 1, -1, 2, -2, 3, 5, -7, 10, 64, 1000, 1000000007, -1000000007]

impl Sample for Bool:
    fn samples() -> List<Bool> = [false, true]

impl Sample for F64:
    fn samples() -> List<F64> = [0.0, 1.0, -1.0, 0.5, 2.0, -3.25, 1000.0]

impl Sample for Char:
    fn samples() -> List<Char> = ['a', 'b', 'z', 'A', '0', ' ']

impl Sample for Str:
    fn samples() -> List<Str> = ["", "a", "b", "ab", "ba", "abc"]

/// A range type of at most 64 values is walked whole. A longer one gives its two first and two
/// last values and `Int`'s samples inside it, in order.
impl Sample for _..=_:
    fn samples() -> List<Self>:
        var out = List<Self>()
        if Self.count <= 64:
            for k in 0..Self.count: out.push(Self.nth(k))
            return out
        for k in [0, 1, Self.count - 2, Self.count - 1]: out.push(Self.nth(k))
        for x in Int.samples():
            if let p = Self.at(x): out.push(p)
        out.sort()
        out.dedup()
        out

/// Every position of a domain, in its walk: an enum's variants, a declared domain's places.
impl<D: Domain> Sample for D:
    fn samples() -> List<D>:
        var out = List<D>()
        for k in 0..D.count: out.push(D.nth(k))
        out

/// `T.samples()` for a type written as an argument: what `cpl laws` walks `x: T` with.
pub fn samples_of<T: Sample>() -> List<T> = T.samples()
