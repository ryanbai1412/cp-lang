//! Methods of optionals (`?T`, core 2.4). The operators (`??`, `!`,
//! `if let`, `match`) are core; these are ordinary fns over them.
//!
//! start: `is_some_and`, `and_then`, `is_none_or`
//! example: `Some(4).is_some_and(v => v > 3) == true`
use std.optional.{Some}

/// Whether there is a value and `p` holds for it; `false` on `None`.
///
/// cost: \(O(1)\) plus one call of `p`
/// traps: only where `p` traps
/// example: `Some(4).is_some_and(v => v > 3) == true`
/// example: `let n: ?Int = None; n.is_some_and(v => v > 3) == false`
@lint(sentinel_table, from: "var t: [D]Int = -1", advice: "var t: [D]?Int = None", holes: "D, k", also: "t[k] == -1\nt[k] != -1\n-1 == t[k]\n-1 != t[k]")
@lint(sentinel_table, from: "var t: [D, E]Int = -1", advice: "var t: [D, E]?Int = None", holes: "D, E, k, j", also: "t[k, j] == -1\nt[k, j] != -1\n-1 == t[k, j]\n-1 != t[k, j]")
@lint(sentinel_table, from: "var t: List<Int> = [-1 for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, k", also: "t[k] == -1\nt[k] != -1\n-1 == t[k]\n-1 != t[k]")
@lint(sentinel_table, from: "var t = [-1 for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, k", also: "t[k] == -1\nt[k] != -1\n-1 == t[k]\n-1 != t[k]")
@lint(sentinel_table, from: "var t: [D]Int = v", advice: "var t: [D]?Int = None", holes: "D, v, k, x", when: "v >= 1000000000", also: "t[k] == v\nt[k] != v\nv == t[k]\nv != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(sentinel_table, from: "var t: [D, E]Int = v", advice: "var t: [D, E]?Int = None", holes: "D, E, v, k, j, x", when: "v >= 1000000000", also: "t[k, j] == v\nt[k, j] != v\nv == t[k, j]\nv != t[k, j]\nt[k, j] < x\nt[k, j] <= x\nt[k, j] > x\nt[k, j] >= x\nx < t[k, j]\nx <= t[k, j]\nx > t[k, j]\nx >= t[k, j]")
@lint(sentinel_table, from: "var t: List<Int> = [v for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, v, k, x", when: "v >= 1000000000", also: "t[k] == v\nt[k] != v\nv == t[k]\nv != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(sentinel_table, from: "var t = [v for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, v, k, x", when: "v >= 1000000000", also: "t[k] == v\nt[k] != v\nv == t[k]\nv != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(sentinel_table, from: "var t: [D]Int = -v", advice: "var t: [D]?Int = None", holes: "D, v, k, x", when: "v >= 1000000000", also: "t[k] == -v\nt[k] != -v\n-v == t[k]\n-v != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(sentinel_table, from: "var t: [D, E]Int = -v", advice: "var t: [D, E]?Int = None", holes: "D, E, v, k, j, x", when: "v >= 1000000000", also: "t[k, j] == -v\nt[k, j] != -v\n-v == t[k, j]\n-v != t[k, j]\nt[k, j] < x\nt[k, j] <= x\nt[k, j] > x\nt[k, j] >= x\nx < t[k, j]\nx <= t[k, j]\nx > t[k, j]\nx >= t[k, j]")
@lint(sentinel_table, from: "var t: List<Int> = [-v for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, v, k, x", when: "v >= 1000000000", also: "t[k] == -v\nt[k] != -v\n-v == t[k]\n-v != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(sentinel_table, from: "var t = [-v for i in r]", advice: "var t: List<?Int> = [None for i in r]", holes: "r, v, k, x", when: "v >= 1000000000", also: "t[k] == -v\nt[k] != -v\n-v == t[k]\n-v != t[k]\nt[k] < x\nt[k] <= x\nt[k] > x\nt[k] >= x\nx < t[k]\nx <= t[k]\nx > t[k]\nx >= t[k]")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) > y:\n    let w = self!\n    b", to: "while let w = self, w > y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) >= y:\n    let w = self!\n    b", to: "while let w = self, w >= y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) < y:\n    let w = self!\n    b", to: "while let w = self, w < y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) <= y:\n    let w = self!\n    b", to: "while let w = self, w <= y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) == y:\n    let w = self!\n    b", to: "while let w = self, w == y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "while (self ?? c) != y:\n    let w = self!\n    b", to: "while let w = self, w != y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) > y:\n    let w = self!\n    b", to: "if let w = self, w > y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) >= y:\n    let w = self!\n    b", to: "if let w = self, w >= y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) < y:\n    let w = self!\n    b", to: "if let w = self, w < y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) <= y:\n    let w = self!\n    b", to: "if let w = self, w <= y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) == y:\n    let w = self!\n    b", to: "if let w = self, w == y:\n    b", holes: "c, y, b")
@lint(coalesce_test_then_unwrap, from: "if (self ?? c) != y:\n    let w = self!\n    b", to: "if let w = self, w != y:\n    b", holes: "c, y, b")
@lint(none_check_unwrap, from: "self != None and e == self!", to: "self.is_some_and(it => e == it)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! == e", to: "self.is_some_and(it => it == e)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! != e", to: "self.is_some_and(it => it != e)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! < e", to: "self.is_some_and(it => it < e)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! <= e", to: "self.is_some_and(it => it <= e)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! > e", to: "self.is_some_and(it => it > e)", holes: "e")
@lint(none_check_unwrap, from: "self != None and self! >= e", to: "self.is_some_and(it => it >= e)", holes: "e")
pub fn is_some_and<T>(self: ?T, p: T -> Bool) -> Bool = if let Some(v) = self: p(v) else: false

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

/// Whether there is a value.
///
/// cost: \(O(1)\)
/// example: `Some(4).is_some == true`
/// example: `let n: ?Int = None; n.is_some == false`
@get pub fn is_some<T>(self: ?T) -> Bool = self.some

/// Whether there is no value.
///
/// cost: \(O(1)\)
/// example: `Some(4).is_none == false`
/// example: `let n: ?Int = None; n.is_none == true`
@get pub fn is_none<T>(self: ?T) -> Bool = not self.some
