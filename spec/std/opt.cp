//! Methods of optionals (`?T`, core 2.4). The operators (`??`, `!`,
//! `if let`, `match`) are core; these are ordinary fns over them.

/// Whether there is a value and `p` holds for it; `false` on `None`.
///
/// cost: O(1) plus one call of `p`
/// traps: only where `p` traps
/// example: `Some(4).is_some_and(v => v > 3) == true`
/// example: `(None: ?Int).is_some_and(v => v > 3) == false`
pub fn is_some_and<T>(self: ?T, p: T -> Bool) -> Bool:
    if self is Some(v): return p(v)
    false

/// Whether there is no value or `p` holds for it; `true` on `None`.
///
/// cost: O(1) plus one call of `p`
/// traps: only where `p` traps
/// example: `Some(2).is_none_or(v => v > 3) == false`
/// example: `(None: ?Int).is_none_or(v => v > 3) == true`
pub fn is_none_or<T>(self: ?T, p: T -> Bool) -> Bool:
    if self is Some(v): return p(v)
    true
