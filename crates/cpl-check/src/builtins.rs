//! What the language itself gives a `List`, a `Str`, a `Map`, a `Set` and
//! an array: the one list of their members, for anything that shows them
//! (completion, signature help, docs). `T` is the element type, `K` and
//! `V` a map's key and value.
//!
//! Every row is run by `crates/cpl-cli/tests/builtin_catalog.rs`: its
//! `example` must have the type `example_ty` in `cpl run`, so a row that
//! the compiler does not agree with fails the build.

use crate::prelude;

/// The built-in types that have members.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Receiver {
    List,
    Str,
    Map,
    Set,
    /// `[D]T` and `[0..n]T`.
    Array,
}

impl Receiver {
    pub const ALL: [Receiver; 5] = [
        Receiver::List,
        Receiver::Str,
        Receiver::Map,
        Receiver::Set,
        Receiver::Array,
    ];

    /// The name as written in a type: `List`, and `[D]T` for an array.
    pub fn name(self) -> &'static str {
        match self {
            Receiver::List => "List",
            Receiver::Str => "Str",
            Receiver::Map => "Map",
            Receiver::Set => "Set",
            Receiver::Array => "[D]T",
        }
    }
}

/// How a member is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemberKind {
    /// Read like a field, `xs.len`. Empty parentheses read the same.
    Get,
    /// Called, `xs.push(x)`.
    Method,
}

/// One member of a built-in type.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinMember {
    pub receiver: Receiver,
    pub name: &'static str,
    pub kind: MemberKind,
    /// The parameters after the receiver, `i: Int, j: Int`; empty for a
    /// `Get`.
    pub params: &'static str,
    /// The type it gives; empty when it gives nothing.
    pub ret: &'static str,
    /// True when it changes the receiver, which then must be a `var`.
    pub mutates: bool,
    pub doc: &'static str,
    /// A use of it on `xs = [3, 1, 2]`, `s = "hello"`, `m = {1: 10, 5: 50}`,
    /// `st = {4, 9}` or `a: [0..3]Int = [0, 7, 0]`.
    pub example: &'static str,
    /// The type of `example`; empty when it gives nothing.
    pub example_ty: &'static str,
}

impl BuiltinMember {
    /// `len: Int` for a `Get`, `swap(i: Int, j: Int)` for a method.
    pub fn signature(&self) -> String {
        let ret = if self.ret.is_empty() {
            String::new()
        } else {
            format!(" -> {}", self.ret)
        };
        match self.kind {
            MemberKind::Get => format!("{}: {}", self.name, self.ret),
            MemberKind::Method => format!("{}({}){ret}", self.name, self.params),
        }
    }
}

/// The members of one built-in type.
pub fn builtin_members(receiver: Receiver) -> impl Iterator<Item = &'static BuiltinMember> {
    BUILTIN_MEMBERS
        .iter()
        .filter(move |member| member.receiver == receiver)
}

/// Another language's name for a member `receiver` has under its own name:
/// Python's `append`, C++'s `size` or `push_back`, Java's `containsKey`, C++'s `unique`.
pub fn foreign_member(receiver: Receiver, name: &str) -> Option<&'static str> {
    let own = foreign_own(name)?;
    (builtin_members(receiver).any(|m| m.name == own)
        || crate::prelude::spec_receiver_method_names().contains(own))
    .then_some(own)
}

/// The CP Lang name another language's member name stands for, on any type.
pub fn foreign_own(name: &str) -> Option<&'static str> {
    Some(match name {
        "append" | "push_back" | "add" => "push",
        "pop_back" => "pop",
        "front" => "first",
        "back" => "last",
        "empty" | "isEmpty" => "is_empty",
        "size" | "length" => "len",
        "contains_key" | "containsKey" | "has_key" | "has" | "includes" | "count_of" => "contains",
        "erase" | "discard" | "delete" => "remove",
        "unique" => "distinct",
        _ => return None,
    })
}

/// Rust's `xs.iter()` or `s.chars()`: a call CP Lang leaves out, since
/// a list or string is iterated as it is.
pub fn foreign_iter_call(name: &str) -> bool {
    matches!(name, "iter" | "into_iter" | "chars")
}

/// A one-character string literal's `Char` spelling: `"a"` is `'a'`.
pub fn char_literal(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut chars = inner.chars();
    let c = chars.next()?;
    (chars.next().is_none() && !matches!(c, '\'' | '\\' | '{' | '}' | '"'))
        .then(|| format!("'{c}'"))
}

/// Another language's name for a std `Str` function: Python's `upper`
/// or `strip`, Java's `toUpperCase`.
pub fn foreign_str_fn(name: &str) -> Option<&'static str> {
    match name {
        "upper" | "toUpperCase" | "uppercase" => Some("to_upper"),
        "lower" | "toLowerCase" | "lowercase" => Some("to_lower"),
        "strip" => Some("trim"),
        "lstrip" | "trimStart" | "trimLeft" => Some("trim_start"),
        "rstrip" | "trimEnd" | "trimRight" => Some("trim_end"),
        _ => None,
    }
}

/// The catalog row for `name` on the built-in type written `owner`
/// (`List`, `Str`, `[D]T`).
pub fn builtin_member(owner: &str, name: &str) -> Option<&'static BuiltinMember> {
    BUILTIN_MEMBERS
        .iter()
        .find(|m| m.receiver.name() == owner && m.name == name)
}

/// What kind of thing a name every file can use is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreludeKind {
    /// `Int`, `List`, `Map`.
    Type,
    /// `Ord`, `Monoid`.
    Trait,
    /// `print`, `sum`, `read`.
    Value,
    /// A name `std/prelude.cp` brings in, with the module it is from.
    Std(&'static str),
}

/// Every name a file can use without a `use`: the compiler's own, then the
/// ones `std/prelude.cp` brings in.
pub fn prelude_names() -> Vec<(&'static str, PreludeKind)> {
    let own = prelude::TYPE_NAMES
        .iter()
        .map(|name| (*name, PreludeKind::Type))
        .chain(
            prelude::TRAIT_NAMES
                .iter()
                .map(|name| (*name, PreludeKind::Trait)),
        )
        .chain(
            prelude::VALUE_NAMES
                .iter()
                .map(|name| (*name, PreludeKind::Value)),
        );
    // `STD_NAMES` is every name of the library; a file gets only the ones
    // the prelude lists.
    let std = prelude::names().iter().filter_map(|(_, name)| {
        let (name, module) = prelude::std_names()
            .iter()
            .find(|(known, _)| *known == name.text)?;
        Some((*name, PreludeKind::Std(module)))
    });
    own.chain(std).collect()
}

use MemberKind::{Get, Method};
use Receiver::{Array, List, Map, Set, Str};

macro_rules! row {
    ($receiver:expr, $name:expr, $kind:expr, $params:expr, $ret:expr, $mutates:expr, $doc:expr, $example:expr, $example_ty:expr) => {
        BuiltinMember {
            receiver: $receiver,
            name: $name,
            kind: $kind,
            params: $params,
            ret: $ret,
            mutates: $mutates,
            doc: $doc,
            example: $example,
            example_ty: $example_ty,
        }
    };
}

#[rustfmt::skip]
pub const BUILTIN_MEMBERS: &[BuiltinMember] = &[
    row!(List, "len", Get, "", "Int", false, "How many elements.", "xs.len", "Int"),
    row!(List, "is_empty", Get, "", "Bool", false, "True when there is no element.", "xs.is_empty", "Bool"),
    row!(List, "first", Get, "", "?T", false, "The first element, or `None` when empty. `xs.first(x => x > 1)` is the first one a test holds for.", "xs.first", "?Int"),
    row!(List, "last", Get, "k: Int = 0", "?T", false, "The last element, or `None` when empty. `xs.last(k)` is the one `k` back from it.", "xs.last", "?Int"),
    row!(List, "push", Method, "x: T", "", true, "Adds `x` at the end in O(1) amortized.", "xs.push(4)", ""),
    row!(List, "pop", Method, "", "?T", true, "Takes the last element off in O(1), or gives `None` when empty.", "xs.pop()", "?Int"),
    row!(List, "get", Method, "i: Int", "?T", false, "The element at `i`, or `None` when `i` is outside the list. O(1).", "xs.get(1)", "?Int"),
    row!(List, "insert", Method, "i: Int, x: T", "", true, "Puts `x` at index `i`, moving the rest up. O(n).", "xs.insert(0, 9)", ""),
    row!(List, "remove", Method, "i: Int", "T", true, "Takes out the element at index `i`, moving the rest down. O(n).", "xs.remove(0)", "Int"),
    row!(List, "swap", Method, "i: Int, j: Int", "", true, "Exchanges the elements at `i` and `j`. O(1).", "xs.swap(0, 1)", ""),
    row!(List, "resize", Method, "n: Int, x: T", "", true, "Makes the length `n`, filling new places with `x`.", "xs.resize(5, 0)", ""),
    row!(List, "truncate", Method, "n: Int", "", true, "Keeps the first `n` elements.", "xs.truncate(2)", ""),
    row!(List, "reserve", Method, "n: Int", "", true, "Reserves capacity for at least `n` elements. Amortized O(n).", "xs.reserve(5)", ""),
    row!(List, "extend", Method, "ys: List<T>", "", true, "Adds every element of `ys` at the end.", "xs.extend([7, 8])", ""),
    row!(List, "fill", Method, "x: T", "", true, "Sets every element to `x`. O(n).", "xs.fill(0)", ""),
    row!(List, "clear", Method, "", "", true, "Removes every element.", "xs.clear()", ""),
    row!(List, "clone", Method, "", "List<T>", false, "A separate list with the same elements. O(n).", "xs.clone()", "List<Int>"),
    row!(List, "to_list", Method, "", "List<T>", false, "An owned `List<T>` copy of this list or view. O(n).", "xs[0..2].to_list()", "List<Int>"),
    row!(List, "reverse", Method, "", "", true, "Reverses the list in place. O(n).", "xs.reverse()", ""),
    row!(List, "reversed", Method, "", "List<T>", false, "A new list in reverse order. O(n).", "xs.reversed()", "List<Int>"),
    row!(List, "rev", Get, "", "List<T>", false, "An O(1) read-only reverse view that keeps the keys: `xs.rev.nth(0)` is the last element and `xs.rev[0]` is an error; copy with `.to_list()` to keep.", "xs.rev", "_"),
    row!(List, "sort", Method, "", "", true, "Sorts in place in O(n log n); `by:` gives a key, `cmp:` a strict-weak-order comparator `(a, b) => Bool` (mutually exclusive with `by:`), and `desc: true` reverses the order.", "xs.sort()", ""),
    row!(List, "sorted", Method, "", "List<T>", false, "A new sorted list in O(n log n); takes `by:`, `cmp:`, and `desc:` like `sort`.", "xs.sorted()", "List<Int>"),
    row!(List, "map", Method, "f: T -> U", "List<U>", false, "A new list of `f(x)` for each element `x`. O(n).", "xs.map(x => x * 2)", "List<Int>"),
    row!(List, "filter", Method, "pred: T -> Bool", "List<T>", false, "A new list of elements satisfying `pred`. O(n).", "xs.filter(x => x > 1)", "_"),
    row!(List, "contains", Method, "x: T", "Bool", false, "True when some element equals `x`. O(n).", "xs.contains(2)", "Bool"),
    row!(List, "count", Method, "x: T", "Int", false, "How many elements equal `x`; given a test, `xs.count(x => x > 1)`, how many it holds for. O(n).", "xs.count(2)", "Int"),
    row!(List, "sum", Method, "", "T", false, "The sum of the elements; `0` when empty. O(n).", "xs.sum()", "Int"),
    row!(List, "fold", Method, "op: M", "T", false, "Folds the elements left to right: a monoid (`+`, `*`, `^`, `gcd`, `min`, `max`, `Monoid`) returns its identity when empty; a binary fn `(T, T) -> T` traps when empty, and `xs.fold(f)!` or `xs.fold(f) ?? d` asks for a `?T`. O(n).", "xs.fold(+)", "Int"),
    row!(List, "min", Method, "", "T", false, "The smallest element (optionally keyed by `by:`); traps when empty, and `xs.min()!` or `xs.min() ?? d` asks for a `?T`. O(n).", "xs.min()", "Int"),
    row!(List, "max", Method, "", "T", false, "The largest element (optionally keyed by `by:`); traps when empty, and `xs.max()!` or `xs.max() ?? d` asks for a `?T`. O(n).", "xs.max()", "Int"),
    row!(List, "all", Method, "test: T -> Bool", "Bool", false, "True when the test holds for every element; stops at the first that fails. O(n).", "xs.all(x => x > 0)", "Bool"),
    row!(List, "any", Method, "test: T -> Bool", "Bool", false, "True when the test holds for some element; stops at the first. O(n).", "xs.any(x => x > 0)", "Bool"),
    row!(List, "find", Method, "x: T", "?Int", false, "The index of the first element equal to `x`. The first element a test holds for is `xs.first(x => x > 1)`. O(n).", "xs.find(2)", "?Int"),
    row!(List, "items", Get, "", "List<(Int, T)>", false, "Each element with its index: `for (i, x) in xs.items`.", "xs.items", "List<(Int, Int)>"),
    row!(List, "first_ge", Method, "x: T", "?Int", false, "In a sorted list, the index of the first element `>= x`. O(log n).", "xs.first_ge(2)", "?Int"),
    row!(List, "first_gt", Method, "x: T", "?Int", false, "In a sorted list, the index of the first element `> x`. O(log n).", "xs.first_gt(2)", "?Int"),
    row!(List, "last_le", Method, "x: T", "?Int", false, "In a sorted list, the index of the last element `<= x`. O(log n).", "xs.last_le(2)", "?Int"),
    row!(List, "last_lt", Method, "x: T", "?Int", false, "In a sorted list, the index of the last element `< x`. O(log n).", "xs.last_lt(2)", "?Int"),
    row!(List, "reach", Method, "while: (T, T) -> Bool, cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(xs[i], xs[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; debug builds trap otherwise.", "xs.reach(while: (a, b) => a + b <= 10)", "List<Int>"),
    row!(List, "windows", Method, "k: Int", "List<List<T>>", false, "Every run of `k` neighbouring elements.", "xs.windows(2)", "_"),

    row!(Str, "len", Get, "", "Int", false, "How many characters.", "s.len", "Int"),
    row!(Str, "items", Get, "", "List<(Int, Char)>", false, "Each character with its index: `for (i, c) in s.items`.", "s.items", "List<(Int, Char)>"),
    row!(Str, "is_empty", Get, "", "Bool", false, "True for the empty text.", "s.is_empty", "Bool"),
    row!(Str, "first", Get, "", "?Char", false, "The first character, or `None` when empty. `s.first(c => c > 'e')` is the first one a test holds for.", "s.first", "?Char"),
    row!(Str, "last", Get, "k: Int = 0", "?Char", false, "The last character, or `None` when empty. `s.last(k)` is the one `k` back from it.", "s.last", "?Char"),
    row!(Str, "get", Method, "i: Int", "?Char", false, "The character at `i`, or `None` when `i` is outside the text. O(1).", "s.get(1)", "?Char"),
    row!(Str, "push", Method, "c: Char", "", true, "Adds `c` at the end in O(1) amortized.", "s.push('!')", ""),
    row!(Str, "pop", Method, "", "?Char", true, "Takes the last character off in O(1), or gives `None` when empty.", "s.pop()", "?Char"),
    row!(Str, "swap", Method, "i: Int, j: Int", "", true, "Exchanges the characters at `i` and `j`. O(1).", "s.swap(0, 1)", ""),
    row!(Str, "resize", Method, "n: Int, c: Char", "", true, "Makes the length `n`, filling new places with `c`.", "s.resize(7, '!')", ""),
    row!(Str, "truncate", Method, "n: Int", "", true, "Keeps the first `n` characters.", "s.truncate(2)", ""),
    row!(Str, "fill", Method, "c: Char", "", true, "Sets every character to `c`. O(n).", "s.fill('a')", ""),
    row!(Str, "clear", Method, "", "", true, "Removes every character.", "s.clear()", ""),
    row!(Str, "clone", Method, "", "Str", false, "A separate text with the same characters. O(n).", "s.clone()", "Str"),
    row!(Str, "to_str", Method, "", "Str", false, "An owned `Str` copy of this string or view. O(n).", "s[0..2].to_str()", "Str"),
    row!(Str, "reverse", Method, "", "", true, "Reverses the characters in place. O(n).", "s.reverse()", ""),
    row!(Str, "reversed", Method, "", "Str", false, "The text backwards. O(n).", "s.reversed()", "Str"),
    row!(Str, "rev", Get, "", "Str", false, "An O(1) read-only reverse view that keeps the keys: `s.rev.nth(0)` is the last character and `s.rev[0]` is an error; copy with `.to_str()` to keep.", "s.rev", "_"),
    row!(Str, "sort", Method, "", "", true, "Sorts characters in place in O(n log n); takes `by:`, `cmp:`, and `desc:` like `List.sort`.", "s.sort()", ""),
    row!(Str, "sorted", Method, "", "List<Char>", false, "A new sorted `List<Char>` in O(n log n); takes `by:`, `cmp:`, and `desc:` like `sort`.", "s.sorted()", "List<Char>"),
    row!(Str, "map", Method, "f: Char -> U", "List<U>", false, "A new list of `f(c)` for each character `c`. O(n).", "s.map(c => Int(c == 'l'))", "List<Int>"),
    row!(Str, "filter", Method, "pred: Char -> Bool", "List<Char>", false, "A new list of characters satisfying `pred`. O(n).", "s.filter(c => c > 'e')", "_"),
    row!(Str, "join", Method, "parts: List<Str>", "Str", false, "The parts with this text between them.", "\", \".join([\"a\", \"b\"])", "Str"),
    row!(Str, "find_all", Method, "p: Str", "List<Int>", false, "Every place `p` starts in the text.", "s.find_all(\"l\")", "List<Int>"),
    row!(Str, "contains", Method, "c: Char", "Bool", false, "True when the text has the character `c`. O(n).", "s.contains('e')", "Bool"),
    row!(Str, "count", Method, "c: Char", "Int", false, "How many characters equal `c`; given a test, `s.count(c => c > 'e')`, how many it holds for. O(n).", "s.count('l')", "Int"),
    row!(Str, "fold", Method, "op: M", "Char", false, "Folds characters left to right; a binary fn `(Char, Char) -> Char` traps when empty, and `s.fold(f)!` or `s.fold(f) ?? d` asks for a `?Char`. O(n).", "s.fold(min)", "Char"),
    row!(Str, "min", Method, "", "Char", false, "The smallest character (optionally keyed by `by:`); traps when empty, and `s.min()!` or `s.min() ?? d` asks for a `?Char`. O(n).", "s.min()", "Char"),
    row!(Str, "max", Method, "", "Char", false, "The largest character (optionally keyed by `by:`); traps when empty, and `s.max()!` or `s.max() ?? d` asks for a `?Char`. O(n).", "s.max()", "Char"),
    row!(Str, "all", Method, "test: Char -> Bool", "Bool", false, "True when the test holds for every character; stops at the first that fails. O(n).", "s.all(c => c >= 'a')", "Bool"),
    row!(Str, "any", Method, "test: Char -> Bool", "Bool", false, "True when the test holds for some character; stops at the first. O(n).", "s.any(c => c == 'e')", "Bool"),
    row!(Str, "find", Method, "c: Char", "?Int", false, "The index of the first character equal to `c` (or substring `p: Str`).", "s.find('e')", "?Int"),
    row!(Str, "first_ge", Method, "c: Char", "?Int", false, "In a sorted string, the index of the first character `>= c`. O(log n).", "s.first_ge('e')", "?Int"),
    row!(Str, "first_gt", Method, "c: Char", "?Int", false, "In a sorted string, the index of the first character `> c`. O(log n).", "s.first_gt('e')", "?Int"),
    row!(Str, "last_le", Method, "c: Char", "?Int", false, "In a sorted string, the index of the last character `<= c`. O(log n).", "s.last_le('l')", "?Int"),
    row!(Str, "last_lt", Method, "c: Char", "?Int", false, "In a sorted string, the index of the last character `< c`. O(log n).", "s.last_lt('l')", "?Int"),
    row!(Str, "reach", Method, "while: (Char, Char) -> Bool, cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(s[i], s[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; debug builds trap otherwise.", "s.reach(while: (a, b) => a <= b)", "List<Int>"),
    row!(Str, "windows", Method, "k: Int", "List<List<Char>>", false, "Every run of `k` neighbouring characters.", "s.windows(2)", "_"),

    row!(Map, "len", Get, "", "Int", false, "How many keys.", "m.len", "Int"),
    row!(Map, "items", Get, "", "List<(K, V)>", false, "Each key with its value, smallest key first: `for (k, v) in m.items`.", "m.items", "List<(Int, Int)>"),
    row!(Map, "keys", Get, "", "List<K>", false, "Each key, smallest first (O(1) view in loops and folds): `for k in m.keys`.", "m.keys", "List<Int>"),
    row!(Map, "values", Get, "", "List<V>", false, "Each value, smallest key first (O(1) view in loops and folds): `for v in m.values`.", "m.values", "List<Int>"),
    row!(Map, "is_empty", Get, "", "Bool", false, "True when there is no key.", "m.is_empty", "Bool"),
    row!(Map, "first", Get, "", "?(K, V)", false, "The smallest key with its value.", "m.first", "?(Int, Int)"),
    row!(Map, "last", Get, "", "?(K, V)", false, "The largest key with its value.", "m.last", "?(Int, Int)"),
    row!(Map, "get", Method, "k: K", "?V", false, "The value at `k`, or `None` when `k` is not a key. O(log n).", "m.get(1)", "?Int"),
    row!(Map, "contains", Method, "k: K", "Bool", false, "True when `k` is a key; `k in m` reads the same. O(log n).", "m.contains(1)", "Bool"),
    row!(Map, "remove", Method, "k: K", "?V", true, "Takes `k` out and gives the value it had. O(log n).", "m.remove(1)", "?Int"),
    row!(Map, "clear", Method, "", "", true, "Removes every key.", "m.clear()", ""),
    row!(Map, "clone", Method, "", "Map<K, V>", false, "A separate map with the same entries.", "m.clone()", "Map<Int, Int>"),
    row!(Map, "pop_first", Method, "", "?(K, V)", true, "Takes out the smallest key with its value. O(log n).", "m.pop_first()", "?(Int, Int)"),
    row!(Map, "pop_last", Method, "", "?(K, V)", true, "Takes out the largest key with its value. O(log n).", "m.pop_last()", "?(Int, Int)"),
    row!(Map, "first_ge", Method, "k: K", "?(K, V)", false, "The smallest key `>= k` with its value. O(log n).", "m.first_ge(2)", "?(Int, Int)"),
    row!(Map, "first_gt", Method, "k: K", "?(K, V)", false, "The smallest key `> k` with its value. O(log n).", "m.first_gt(1)", "?(Int, Int)"),
    row!(Map, "last_le", Method, "k: K", "?(K, V)", false, "The largest key `<= k` with its value. O(log n).", "m.last_le(4)", "?(Int, Int)"),
    row!(Map, "last_lt", Method, "k: K", "?(K, V)", false, "The largest key `< k` with its value. O(log n).", "m.last_lt(5)", "?(Int, Int)"),
    row!(Map, "range", Method, "r: Range<K>", "SortedRange<(K, V)>", false, "O(1) key-range view; `.first`, `.last`, `.is_empty` are O(log n) and iteration is O(log n + k). Use `SortedSet`/`SortedBag` for O(log n) `.len`.", "m.range(1..5).first", "?(Int, Int)"),

    row!(Set, "len", Get, "", "Int", false, "How many elements.", "st.len", "Int"),
    row!(Set, "is_empty", Get, "", "Bool", false, "True when there is no element.", "st.is_empty", "Bool"),
    row!(Set, "first", Get, "", "?T", false, "The smallest element.", "st.first", "?Int"),
    row!(Set, "last", Get, "", "?T", false, "The largest element.", "st.last", "?Int"),
    row!(Set, "insert", Method, "x: T", "Bool", true, "Adds `x`; true when it was not there before. O(log n).", "st.insert(5)", "Bool"),
    row!(Set, "add", Method, "x: T", "Bool", true, "The same as `insert`. O(log n).", "st.add(5)", "Bool"),
    row!(Set, "remove", Method, "x: T", "Bool", true, "Takes `x` out; true when it was there. O(log n).", "st.remove(4)", "Bool"),
    row!(Set, "set", Method, "x: T, on: Bool", "", true, "Adds `x` when `on` is true, or removes it otherwise. O(log n) on `Set`, O(1) amortized on `HashSet`.", "st.set(5, on: true)", ""),
    row!(Set, "contains", Method, "x: T", "Bool", false, "True when `x` is in the set; `x in st` reads the same. O(log n).", "st.contains(4)", "Bool"),
    row!(Set, "clear", Method, "", "", true, "Removes every element.", "st.clear()", ""),
    row!(Set, "clone", Method, "", "Set<T>", false, "A separate set with the same elements.", "st.clone()", "Set<Int>"),
    row!(Set, "sorted", Method, "", "List<T>", false, "The elements as a sorted list.", "st.sorted()", "List<Int>"),
    row!(Set, "min", Method, "", "?T", false, "The smallest element. O(log n).", "st.min()", "?Int"),
    row!(Set, "max", Method, "", "?T", false, "The largest element. O(log n).", "st.max()", "?Int"),
    row!(Set, "pop_first", Method, "", "?T", true, "Takes out the smallest element. O(log n).", "st.pop_first()", "?Int"),
    row!(Set, "pop_last", Method, "", "?T", true, "Takes out the largest element. O(log n).", "st.pop_last()", "?Int"),
    row!(Set, "first_ge", Method, "x: T", "?T", false, "The smallest element `>= x`. O(log n).", "st.first_ge(5)", "?Int"),
    row!(Set, "first_gt", Method, "x: T", "?T", false, "The smallest element `> x`. O(log n).", "st.first_gt(4)", "?Int"),
    row!(Set, "last_le", Method, "x: T", "?T", false, "The largest element `<= x`. O(log n).", "st.last_le(5)", "?Int"),
    row!(Set, "last_lt", Method, "x: T", "?T", false, "The largest element `< x`. O(log n).", "st.last_lt(9)", "?Int"),
    row!(Set, "range", Method, "r: Range<T>", "SortedRange<T>", false, "O(1) key-range view; `.first`, `.last`, `.is_empty` are O(log n) and iteration is O(log n + k). Use `SortedSet`/`SortedBag` for O(log n) `.len`.", "st.range(4..9).first", "?Int"),
    row!(Set, "drain", Method, "; after: ?T = none, before: ?T = none, while ok: (T) -> Bool", "List<T>", true, "Removes and returns the maximal run strictly after `after` (ascending), strictly before `before` (descending), or from the smallest element (ascending) while `ok(k)` holds. O(k log n).", "st.drain(after: 4, while: x => x < 9)", "List<Int>"),

    row!(Array, "len", Get, "", "Int", false, "How many cells.", "a.len", "Int"),
    row!(Array, "is_empty", Get, "", "Bool", false, "True when the array has no cells.", "a.is_empty", "Bool"),
    row!(Array, "first", Get, "", "?T", false, "The first cell, or `None` when the array is empty. `a.first(x => x > 1)` is the first one a test holds for.", "a.first", "?Int"),
    row!(Array, "last", Get, "k: Int = 0", "?T", false, "The last cell, or `None` when the array is empty. `a.last(k)` is the one `k` back from it.", "a.last", "?Int"),
    row!(Array, "get", Method, "i: Int", "?T", false, "The cell at `i`, or `None` when `i` is outside the array. O(1).", "a.get(1)", "?Int"),
    row!(Array, "swap", Method, "i: D, j: D", "", true, "Exchanges the cells at `i` and `j`. O(1).", "a.swap(0, 1)", ""),
    row!(Array, "fill", Method, "x: T", "", true, "Sets every cell to `x`. O(n).", "a.fill(2)", ""),
    row!(Array, "clone", Method, "", "[D]T", false, "A separate array with the same cells. O(n).", "a.clone()", "[0..3]Int"),
    row!(Array, "reverse", Method, "", "", true, "Reverses the cells in place. O(n).", "a.reverse()", ""),
    row!(Array, "reversed", Method, "", "List<T>", false, "A new list with the cells in reverse order. O(n).", "a.reversed()", "List<Int>"),
    row!(Array, "sort", Method, "", "", true, "Sorts in place in O(n log n); `by:` gives a key, `cmp:` a strict-weak-order comparator `(a, b) => Bool` (mutually exclusive with `by:`), and `desc: true` reverses the order.", "a.sort()", ""),
    row!(Array, "sorted", Method, "", "List<T>", false, "The cells as a sorted list in O(n log n); takes `by:`, `cmp:`, and `desc:` like `sort`.", "a.sorted()", "List<Int>"),
    row!(Array, "map", Method, "f: T -> U", "[D]U", false, "A new array with `f(x)` in each cell, keeping the index domain(s). O(n).", "a.map(x => x + 1)", "[0..3]Int"),
    row!(Array, "filter", Method, "pred: T -> Bool", "List<T>", false, "A new list of cells satisfying `pred`. O(n).", "a.filter(x => x > 1)", "_"),
    row!(Array, "contains", Method, "x: T", "Bool", false, "True when some cell equals `x`. O(n).", "a.contains(7)", "Bool"),
    row!(Array, "count", Method, "x: T", "Int", false, "How many cells equal `x`; given a test, `a.count(x => x > 1)`, how many it holds for. O(n).", "a.count(7)", "Int"),
    row!(Array, "sum", Method, "", "T", false, "The sum of the cells; `0` when empty. O(n).", "a.sum()", "Int"),
    row!(Array, "fold", Method, "op: M", "T", false, "Folds the cells left to right: a monoid (`+`, `*`, `^`, `gcd`, `min`, `max`, `Monoid`) returns its identity when empty; a binary fn `(T, T) -> T` traps when empty, and `a.fold(f)!` or `a.fold(f) ?? d` asks for a `?T`. O(n).", "a.fold(+)", "Int"),
    row!(Array, "min", Method, "", "T", false, "The smallest cell (optionally keyed by `by:`); traps when empty, and `a.min()!` or `a.min() ?? d` asks for a `?T`. O(n).", "a.min()", "Int"),
    row!(Array, "max", Method, "", "T", false, "The largest cell (optionally keyed by `by:`); traps when empty, and `a.max()!` or `a.max() ?? d` asks for a `?T`. O(n).", "a.max()", "Int"),
    row!(Array, "all", Method, "test: T -> Bool", "Bool", false, "True when the test holds for every cell; stops at the first that fails. O(n).", "a.all(x => x > 0)", "Bool"),
    row!(Array, "any", Method, "test: T -> Bool", "Bool", false, "True when the test holds for some cell; stops at the first. O(n).", "a.any(x => x > 0)", "Bool"),
    row!(Array, "find", Method, "x: T", "?I", false, "The index of the first cell equal to `x`. The first cell a test holds for is `a.first(x => x > 1)`.", "a.find(7)", "?Int"),
    row!(Array, "items", Get, "", "List<(I, T)>", false, "Each cell with its index: `for (i, x) in a.items`.", "a.items", "List<(Int, Int)>"),
    row!(Array, "first_ge", Method, "x: T", "?I", false, "In a sorted array, the index of the first cell `>= x`. O(log n).", "a.first_ge(2)", "?Int"),
    row!(Array, "first_gt", Method, "x: T", "?I", false, "In a sorted array, the index of the first cell `> x`. O(log n).", "a.first_gt(2)", "?Int"),
    row!(Array, "last_le", Method, "x: T", "?I", false, "In a sorted array, the index of the last cell `<= x`. O(log n).", "a.last_le(2)", "?Int"),
    row!(Array, "last_lt", Method, "x: T", "?I", false, "In a sorted array, the index of the last cell `< x`. O(log n).", "a.last_lt(2)", "?Int"),
    row!(Array, "reach", Method, "while: (T, T) -> Bool, cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(a[i], a[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; debug builds trap otherwise.", "a.reach(while: (x, y) => x + y <= 10)", "List<Int>"),
    row!(Array, "windows", Method, "k: Int", "List<List<T>>", false, "Every run of `k` neighbouring cells.", "a.windows(2)", "_"),
];

/// The members `member_ty` gives a domain (`V.first`) and a position
/// (`v.next`), for "did you mean" on a misspelled one.
pub(crate) const DOMAIN_MEMBERS: &[&str] = &[
    "first",
    "last",
    "all",
    "rev",
    "mask",
    "wrap",
    "nth",
    "bounds",
    "cuts",
    "pairs",
    "after",
    "before",
    "partition",
    "filter",
    "sort",
];
pub(crate) const POS_MEMBERS: &[&str] = &[
    "ord", "to_idx", "idx", "label", "prev", "next", "offset", "ahead", "back",
];

#[cfg(test)]
mod tests {
    fn check(src: &str) -> Vec<cpl_diag::Diagnostic> {
        let mut db = crate::Database::new();
        db.set_file("a.cp", src);
        db.diagnostics("a.cp")
    }

    #[test]
    fn a_misspelled_domain_member_suggests_the_domain_member() {
        let d = &check("domain V = 1..=9\nprint(V.mas)\n")[0];
        assert_eq!(d.code.0, "E0203");
        assert_eq!(d.help.as_deref(), Some("did you mean `mask`?"));
        let d = &check("domain V = 1..=9\nfor v in V:\n    print(v.nxt)\n")[0];
        assert_eq!(d.help.as_deref(), Some("did you mean `next`?"));
        // Every listed member is one `member_ty` knows.
        for (owner, members) in [
            ("V", crate::builtins::DOMAIN_MEMBERS),
            ("v", crate::builtins::POS_MEMBERS),
        ] {
            for m in members {
                let src = format!("domain V = 1..=9\nfor v in V:\n    let x = {owner}.{m}\n");
                let found: Vec<_> = check(&src).iter().map(|d| d.code.0).collect();
                assert!(!found.contains(&"E0203"), "{owner}.{m}: {found:?}");
            }
        }
    }
}
