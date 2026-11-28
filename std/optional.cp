//! `?T`, the struct `Optional<T>`: a `T` or `None` (R02.4). Core re-exports it and `Some`, so
//! no module imports them.
//!
//! example: `let a: ?Int = Some(3); a == Some(3)`
use std.mem.{swap}
use std.clone.{Clone}
use std.ops.{Eq}

// `?T`: `T` or `None`. `val` is read only when `some`.
@role(option)
pub struct Optional<T>:
    pub some: Bool
    val: T

    /// `f` of the value, kept `?`: `None` stays `None` (R02.4.1). A member, so it is found before
    /// `Seq`'s `map` (R08.1.4).
    ///
    /// cost: one call of `f` when present
    /// empty: `let a: ?Int = None; a.map(x => x + 1) == None`
    /// traps: never
    /// example: `Some(3).map(x => x * 2) == Some(6)`
    pub fn map<U>(self, f: T -> U) -> ?U = if some: Some(f(val)) else: None

    /// The value, moved out, with `None` left in its place (R05.2).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a: ?Int = Some(3); let b = a.take(); a == None and b == Some(3)`
    pub fn take(mut self) -> ?T:
        var out: ?T = None
        swap(mut out, mut self)
        out

/// `Some(v)`: `v`, present (R01.5.6); `Some(p)` matches it (R01.7.1).
@pattern
pub fn Some<T>(v: T) -> ?T = v

/// Both empty, or both holding equal values; `?T` is not `Ord` (R02.4.5).
impl<T: Eq> Eq for Optional<T>:
    fn eq(a, b) = if not a.some: not b.some elif not b.some: false else: a.val == b.val

impl<T: Clone> Clone for Optional<T>:
    fn clone(a) = if a.some: Optional(true, a.val.clone()) else: None
