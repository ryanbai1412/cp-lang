//! Other languages' names for built-in members and `Str` functions, as
//! CP Lang spells them.

use super::*;

/// Another language's name for a member `receiver` has under its own name:
/// Python's `append`, C++'s `size` or `push_back`, Java's `containsKey`, C++'s `unique`.
pub fn foreign_member(receiver: Receiver, name: &str) -> Option<&'static str> {
    foreign_own(name).iter().copied().find(|own| {
        builtin_members(receiver).any(|m| m.name == *own)
            || crate::prelude::spec_receiver_method_names().contains(*own)
    })
}

/// The CP Lang names another language's (or an older std's) member name stands for, on any
/// type, the first a type has winning: Rust's `append` of a whole list is `splice_back`, of one
/// value `push`.
pub fn foreign_own(name: &str) -> &'static [&'static str] {
    match name {
        "append" => &["splice_back", "push"],
        "push_back" | "add" => &["push"],
        "pop_back" => &["pop"],
        "front" => &["first"],
        "back" => &["last"],
        "empty" | "isEmpty" => &["is_empty"],
        "size" | "length" => &["len"],
        "contains_key" | "containsKey" | "has_key" | "has" | "includes" | "count_of" => {
            &["contains"]
        }
        "erase" | "discard" | "delete" => &["remove"],
        "unique" => &["distinct"],
        "partition_point" => &["bsearch"],
        "flip" => &["toggle", "neg", "add"],
        "insert" => &["add"],
        "pad" | "pad_to" | "grow" => &["resize"],
        _ => &[],
    }
}

/// `x.unwrap()` or `x.expect(..)` on an optional.
pub const OPTIONAL_UNWRAP_HELP: &str =
    "`x!` unwraps an optional, trapping on `None` at this place (R02.4.2)";

/// C++'s `a.begin() + i` or `a.end()` standing for a position in a list.
pub const ITERATOR_POSITION_HELP: &str =
    "CP Lang has no iterators: a position is its index, so `a.begin() + i` is `i` and `a.end()` is `a.len`";

/// `a.pop_front()` on a list: a list changes only at its end, since a front
/// edit would be O(n); both ends are a `Deque`.
pub fn list_end_help(method: &str) -> Option<&'static str> {
    match method {
        "pop_front" | "popleft" => Some(
            "a list pops only at its end; for both ends use `Deque` (`var q = Deque.of(a)`, then `q.pop_front()`)",
        ),
        "push_front" | "appendleft" => Some(
            "a list grows only at its end; for both ends use `Deque` (`var q = Deque.of(a)`, then `q.push_front(x)`)",
        ),
        _ => None,
    }
}

/// Rust's `xs.iter()` or `s.chars()`: a call CP Lang leaves out, since
/// a list or string is iterated as it is.
pub fn foreign_iter_call(name: &str) -> bool {
    matches!(name, "iter" | "into_iter" | "chars")
}

/// Another language's name for a `Str` fn (`upper`, `strip`), as std's
/// `@aka` states it.
pub fn foreign_str_fn(name: &str) -> Option<&'static str> {
    crate::prelude::aka("Str", name)
}
