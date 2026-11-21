//! `use std.derive.{lt_fields}`. The `Eq`, `Ord` and `Show` a struct has when it writes no `impl` of its
//! own, as library code on `fields` (decision 28.7). Each is O(fields)
//! plus what its fields cost. An `impl` can also call one of these, e.g.
//! `impl Ord for E: fn lt(a, b) = lt_fields(b, a)` for the reverse order.

/// `a == b` field by field, in declaration order.
pub fn eq_fields<T>(a: T, b: T) -> Bool:
    for (x, y) in fields(a, b):
        if x != y:
            return false
    true

/// `a < b` by the first field that differs, in declaration order.
pub fn lt_fields<T>(a: T, b: T) -> Bool:
    for (x, y) in fields(a, b):
        if x != y:
            return x < y
    false

/// The fields of `a` in declaration order, one space apart.
pub fn show_fields<T>(a: T) -> Str:
    var s = ""
    var first = true
    for f in fields(a):
        s = if first: "{f}" else: "{s} {f}"
        first = false
    s
