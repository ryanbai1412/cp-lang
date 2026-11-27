//! `Into`: `T(x)` converts `x` by its impl (R04.2.28), and the conversions between the core
//! types. Core re-exports it, so no module imports it.
//!
//! example: `Int('a') == 97 and Char(98) == 'b'`

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
