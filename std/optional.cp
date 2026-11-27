//! `?T`, the struct `Optional<T>`: a `T` or `None` (R02.4). Core re-exports it and `Some`, so
//! no module imports them.
//!
//! example: `let a: ?Int = Some(3); a == Some(3)`

// `?T`: `T` or `None`. `val` is read only when `some`.
@optional
pub struct Optional<T>:
    some: Bool
    val: T

/// `Some(v)`: `v`, present (R01.5.6); `Some(p)` matches it (R01.7.1).
@pattern
pub fn Some<T>(v: T) -> ?T = v

/// Both empty, or both holding equal values; `?T` is not `Ord` (R02.4.5).
impl<T: Eq> Eq for Optional<T>:
    fn eq(a, b) = if not a.some: not b.some elif not b.some: false else: a.val == b.val

impl<T: Clone> Clone for Optional<T>:
    fn clone(a) = if a.some: Optional(true, a.val.clone()) else: None
