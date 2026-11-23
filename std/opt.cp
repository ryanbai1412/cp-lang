//! Methods of optionals (`?T`, core 2.4). The operators (`??`, `!`,
//! `if let`, `match`) are core; these are ordinary fns over them.

/// Whether there is a value and `p` holds for it; `false` on `None`.
///
/// cost: \(O(1)\) plus one call of `p`
/// traps: only where `p` traps
/// example: `Some(4).is_some_and(v => v > 3) == true`
/// example: `let n: ?Int = None; n.is_some_and(v => v > 3) == false`
pub fn is_some_and<T>(self: ?T, p: T -> Bool) -> Bool:
    if let Some(v) = self: return p(v)
    false

/// The optional `f` gives for the value; `None` on `None`, without calling `f`.
///
/// cost: \(O(1)\) plus one call of `f`
/// traps: only where `f` traps
/// example: `Some(3).and_then(x => if x > 1: Some(x * 2) else: None) == Some(6)`
/// example: `Some(1).and_then(x => if x > 1: Some(x * 2) else: None) == None`
pub fn and_then<T, U>(self: ?T, f: T -> ?U) -> ?U:
    if let Some(v) = self: return f(v)
    None

/// Whether there is no value or `p` holds for it; `true` on `None`.
///
/// cost: \(O(1)\) plus one call of `p`
/// traps: only where `p` traps
/// example: `Some(2).is_none_or(v => v > 3) == false`
/// example: `let n: ?Int = None; n.is_none_or(v => v > 3) == true`
pub fn is_none_or<T>(self: ?T, p: T -> Bool) -> Bool:
    if let Some(v) = self: return p(v)
    true
