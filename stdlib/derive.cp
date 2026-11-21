//! `use std.derive.{lt_fields}`. The `Eq`, `Ord` and `Show` a struct has when it writes no `impl` of its
//! own, as library code on `fields` (decision 28.7). Each is O(fields)
//! plus what its fields cost. An `impl` can also call one of these, e.g.
//! `impl Ord for E: fn lt(a, b) = lt_fields(b, a)` for the reverse order.
//! `Hash` is a trait the built-in types implement here; `hash_fields`
//! folds it over a struct's fields, so `impl Hash for P: fn hash(self) =
//! hash_fields(self)` is the whole derive.

use std.math.splitmix64

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

/// A 64-bit hash, mixed by `splitmix64`: equal values hash equal, and
/// values close together land far apart. O(1), O(len) for a `Str`.
pub trait Hash:
    fn hash(self) -> U64

impl Hash for Int:
    fn hash(self) -> U64 = splitmix64(U64(self))

impl Hash for U64:
    fn hash(self) -> U64 = splitmix64(self)

impl Hash for Char:
    fn hash(self) -> U64 = splitmix64(U64(Int(self)))

impl Hash for Bool:
    fn hash(self) -> U64 = splitmix64(if self: 1 else: 0)

impl Hash for Str:
    fn hash(self) -> U64:
        var h: U64 = splitmix64(U64(self.len))
        for c in self:
            h = splitmix64(h ^ U64(Int(c)))
        h

/// The `Hash` of `a` from its fields' `Hash`, in declaration order.
pub fn hash_fields<T>(a: T) -> U64:
    var h: U64 = 0
    for f in fields(a):
        h = splitmix64(h ^ f.hash())
    h
