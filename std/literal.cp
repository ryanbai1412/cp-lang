//! The traits a literal is sugar for: `[a, b]` and `[f(x) for x in s]` build through the
//! `@literal("[]")` trait, and `var a: [D]T = x` fills through `@literal("[;]")` (R01.8.8). Core
//! re-exports both, so a program's own type gets the literals by implementing them.
//!
//! start: `ListLiteral`, `Filled`, `ListLiteral.with_room`
//! example: `let xs: List<Int> = [1, 2]; xs.len == 2`

/// What `[a, b]` builds when its type is `Self`: `with_room(2)`, then a `push` of each (R01.8.8).
@literal("[]")
pub trait ListLiteral<T>:
    /// An empty value with room for `n` elements, before the literal's pushes.
    fn with_room(n: Int) -> Self
    fn push(mut self, x: T)

/// What `var a: [D]T = x` builds when its type is `Self`: every cell `x` (R01.8.8).
@literal("[;]")
pub trait Filled<T>:
    fn filled(x: T) -> Self
