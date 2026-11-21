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
    pub const ALL: [Receiver; 5] =
        [Receiver::List, Receiver::Str, Receiver::Map, Receiver::Set, Receiver::Array];

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
        let ret = if self.ret.is_empty() { String::new() } else { format!(" -> {}", self.ret) };
        match self.kind {
            MemberKind::Get => format!("{}: {}", self.name, self.ret),
            MemberKind::Method => format!("{}({}){ret}", self.name, self.params),
        }
    }
}

/// The members of one built-in type.
pub fn builtin_members(receiver: Receiver) -> impl Iterator<Item = &'static BuiltinMember> {
    BUILTIN_MEMBERS.iter().filter(move |member| member.receiver == receiver)
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
    /// A name `stdlib/prelude.cp` brings in, with the module it is from.
    Std(&'static str),
}

/// Every name a file can use without a `use`: the compiler's own, then the
/// ones `stdlib/prelude.cp` brings in.
pub fn prelude_names() -> Vec<(&'static str, PreludeKind)> {
    let own = prelude::TYPE_NAMES
        .iter()
        .map(|name| (*name, PreludeKind::Type))
        .chain(prelude::TRAIT_NAMES.iter().map(|name| (*name, PreludeKind::Trait)))
        .chain(prelude::VALUE_NAMES.iter().map(|name| (*name, PreludeKind::Value)));
    // `STD_NAMES` is every name of the library; a file gets only the ones
    // the prelude lists.
    let std = prelude::names().iter().filter_map(|(_, name)| {
        let (name, module) = prelude::STD_NAMES.iter().find(|(known, _)| *known == name.text)?;
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
    row!(List, "first", Get, "", "?T", false, "The first element, or `none` when empty.", "xs.first", "?Int"),
    row!(List, "last", Get, "", "?T", false, "The last element, or `none` when empty. `xs.last(k)` is the one `k` back from it.", "xs.last", "?Int"),
    row!(List, "push", Method, "x: T", "", true, "Adds `x` at the end.", "xs.push(4)", ""),
    row!(List, "pop", Method, "", "?T", true, "Takes the last element off, or gives `none` when empty.", "xs.pop()", "?Int"),
    row!(List, "get", Method, "i: Int", "?T", false, "The element at `i`, or `none` when `i` is outside the list.", "xs.get(1)", "?Int"),
    row!(List, "insert", Method, "i: Int, x: T", "", true, "Puts `x` at index `i`, moving the rest up.", "xs.insert(0, 9)", ""),
    row!(List, "remove", Method, "i: Int", "T", true, "Takes out the element at index `i`, moving the rest down.", "xs.remove(0)", "Int"),
    row!(List, "swap", Method, "i: Int, j: Int", "", true, "Exchanges the elements at `i` and `j`.", "xs.swap(0, 1)", ""),
    row!(List, "resize", Method, "n: Int, x: T", "", true, "Makes the length `n`, filling new places with `x`.", "xs.resize(5, 0)", ""),
    row!(List, "truncate", Method, "n: Int", "", true, "Keeps the first `n` elements.", "xs.truncate(2)", ""),
    row!(List, "extend", Method, "ys: List<T>", "", true, "Adds every element of `ys` at the end.", "xs.extend([7, 8])", ""),
    row!(List, "fill", Method, "x: T", "", true, "Sets every element to `x`.", "xs.fill(0)", ""),
    row!(List, "clear", Method, "", "", true, "Removes every element.", "xs.clear()", ""),
    row!(List, "copy", Method, "", "List<T>", false, "A separate list with the same elements.", "xs.copy()", "List<Int>"),
    row!(List, "reverse", Method, "", "", true, "Reverses the list in place.", "xs.reverse()", ""),
    row!(List, "reversed", Method, "", "List<T>", false, "A new list in reverse order.", "xs.reversed()", "List<Int>"),
    row!(List, "rev", Method, "", "List<T>", false, "The elements in reverse order, to loop over or keep.", "xs.rev()", "List<Int>"),
    row!(List, "sort", Method, "", "", true, "Sorts in place; `by:` gives a key and `desc: true` the other order.", "xs.sort()", ""),
    row!(List, "sorted", Method, "", "List<T>", false, "A new sorted list; takes `by:` and `desc:` like `sort`.", "xs.sorted()", "List<Int>"),
    row!(List, "contains", Method, "x: T", "Bool", false, "True when some element equals `x`.", "xs.contains(2)", "Bool"),
    row!(List, "find", Method, "x: T", "?Int", false, "The index of the first element equal to `x`.", "xs.find(2)", "?Int"),
    row!(List, "first_ge", Method, "x: T", "?Int", false, "In a sorted list, the index of the first element `>= x`.", "xs.first_ge(2)", "?Int"),
    row!(List, "last_le", Method, "x: T", "?Int", false, "In a sorted list, the index of the last element `<= x`.", "xs.last_le(2)", "?Int"),
    row!(List, "windows", Method, "k: Int", "List<List<T>>", false, "Every run of `k` neighbouring elements.", "xs.windows(2)", "List<List<Int>>"),

    row!(Str, "len", Get, "", "Int", false, "How many characters.", "s.len", "Int"),
    row!(Str, "is_empty", Get, "", "Bool", false, "True for the empty text.", "s.is_empty", "Bool"),
    row!(Str, "push", Method, "c: Char", "", true, "Adds `c` at the end.", "s.push('!')", ""),
    row!(Str, "copy", Method, "", "Str", false, "A separate text with the same characters.", "s.copy()", "Str"),
    row!(Str, "reversed", Method, "", "Str", false, "The text backwards.", "s.reversed()", "Str"),
    row!(Str, "rev", Method, "", "Str", false, "The characters in reverse order, to loop over or keep.", "s.rev()", "Str"),
    row!(Str, "repeat", Method, "n: Int", "Str", false, "The text `n` times over.", "s.repeat(2)", "Str"),
    row!(Str, "join", Method, "parts: List<Str>", "Str", false, "The parts with this text between them.", "\", \".join([\"a\", \"b\"])", "Str"),
    row!(Str, "find", Method, "p: Str", "?Int", false, "Where `p` first starts in the text.", "s.find(\"ll\")", "?Int"),
    row!(Str, "find_all", Method, "p: Str", "List<Int>", false, "Every place `p` starts in the text.", "s.find_all(\"l\")", "List<Int>"),
    row!(Str, "contains", Method, "c: Char", "Bool", false, "True when the text has the character `c`.", "s.contains('e')", "Bool"),
    row!(Str, "starts_with", Method, "p: Str", "Bool", false, "True when the text begins with `p`.", "s.starts_with(\"he\")", "Bool"),
    row!(Str, "ends_with", Method, "p: Str", "Bool", false, "True when the text ends with `p`.", "s.ends_with(\"lo\")", "Bool"),
    row!(Str, "windows", Method, "k: Int", "List<List<Char>>", false, "Every run of `k` neighbouring characters.", "s.windows(2)", "List<List<Char>>"),

    row!(Map, "len", Get, "", "Int", false, "How many keys.", "m.len", "Int"),
    row!(Map, "is_empty", Get, "", "Bool", false, "True when there is no key.", "m.is_empty", "Bool"),
    row!(Map, "first", Get, "", "?(K, V)", false, "The smallest key with its value.", "m.first", "?(Int, Int)"),
    row!(Map, "last", Get, "", "?(K, V)", false, "The largest key with its value.", "m.last", "?(Int, Int)"),
    row!(Map, "get", Method, "k: K", "?V", false, "The value at `k`, or `none` when `k` is not a key.", "m.get(1)", "?Int"),
    row!(Map, "contains", Method, "k: K", "Bool", false, "True when `k` is a key; `k in m` reads the same.", "m.contains(1)", "Bool"),
    row!(Map, "remove", Method, "k: K", "?V", true, "Takes `k` out and gives the value it had.", "m.remove(1)", "?Int"),
    row!(Map, "clear", Method, "", "", true, "Removes every key.", "m.clear()", ""),
    row!(Map, "copy", Method, "", "Map<K, V>", false, "A separate map with the same entries.", "m.copy()", "Map<Int, Int>"),
    row!(Map, "pop_first", Method, "", "?(K, V)", true, "Takes out the smallest key with its value.", "m.pop_first()", "?(Int, Int)"),
    row!(Map, "pop_last", Method, "", "?(K, V)", true, "Takes out the largest key with its value.", "m.pop_last()", "?(Int, Int)"),
    row!(Map, "first_ge", Method, "k: K", "?(K, V)", false, "The smallest key `>= k` with its value.", "m.first_ge(2)", "?(Int, Int)"),
    row!(Map, "first_gt", Method, "k: K", "?(K, V)", false, "The smallest key `> k` with its value.", "m.first_gt(1)", "?(Int, Int)"),
    row!(Map, "last_le", Method, "k: K", "?(K, V)", false, "The largest key `<= k` with its value.", "m.last_le(4)", "?(Int, Int)"),
    row!(Map, "last_lt", Method, "k: K", "?(K, V)", false, "The largest key `< k` with its value.", "m.last_lt(5)", "?(Int, Int)"),

    row!(Set, "len", Get, "", "Int", false, "How many elements.", "st.len", "Int"),
    row!(Set, "is_empty", Get, "", "Bool", false, "True when there is no element.", "st.is_empty", "Bool"),
    row!(Set, "first", Get, "", "?T", false, "The smallest element.", "st.first", "?Int"),
    row!(Set, "last", Get, "", "?T", false, "The largest element.", "st.last", "?Int"),
    row!(Set, "insert", Method, "x: T", "Bool", true, "Adds `x`; true when it was not there before.", "st.insert(5)", "Bool"),
    row!(Set, "add", Method, "x: T", "Bool", true, "The same as `insert`.", "st.add(5)", "Bool"),
    row!(Set, "remove", Method, "x: T", "Bool", true, "Takes `x` out; true when it was there.", "st.remove(4)", "Bool"),
    row!(Set, "contains", Method, "x: T", "Bool", false, "True when `x` is in the set; `x in st` reads the same.", "st.contains(4)", "Bool"),
    row!(Set, "clear", Method, "", "", true, "Removes every element.", "st.clear()", ""),
    row!(Set, "copy", Method, "", "Set<T>", false, "A separate set with the same elements.", "st.copy()", "Set<Int>"),
    row!(Set, "sorted", Method, "", "List<T>", false, "The elements as a sorted list.", "st.sorted()", "List<Int>"),
    row!(Set, "min", Method, "", "?T", false, "The smallest element.", "st.min()", "?Int"),
    row!(Set, "max", Method, "", "?T", false, "The largest element.", "st.max()", "?Int"),
    row!(Set, "pop_first", Method, "", "?T", true, "Takes out the smallest element.", "st.pop_first()", "?Int"),
    row!(Set, "pop_last", Method, "", "?T", true, "Takes out the largest element.", "st.pop_last()", "?Int"),
    row!(Set, "first_ge", Method, "x: T", "?T", false, "The smallest element `>= x`.", "st.first_ge(5)", "?Int"),
    row!(Set, "first_gt", Method, "x: T", "?T", false, "The smallest element `> x`.", "st.first_gt(4)", "?Int"),
    row!(Set, "last_le", Method, "x: T", "?T", false, "The largest element `<= x`.", "st.last_le(5)", "?Int"),
    row!(Set, "last_lt", Method, "x: T", "?T", false, "The largest element `< x`.", "st.last_lt(9)", "?Int"),

    row!(Array, "len", Get, "", "Int", false, "How many cells.", "a.len", "Int"),
    row!(Array, "first", Get, "", "?T", false, "The first cell, or `none` when the array is empty.", "a.first", "?Int"),
    row!(Array, "last", Get, "", "?T", false, "The last cell, or `none` when the array is empty.", "a.last", "?Int"),
    row!(Array, "swap", Method, "i: D, j: D", "", true, "Exchanges the cells at `i` and `j`.", "a.swap(0, 1)", ""),
    row!(Array, "fill", Method, "x: T", "", true, "Sets every cell to `x`.", "a.fill(2)", ""),
    row!(Array, "copy", Method, "", "[D]T", false, "A separate array with the same cells.", "a.copy()", "[0..3]Int"),
    row!(Array, "sort", Method, "", "", true, "Sorts in place; `by:` gives a key and `desc: true` the other order.", "a.sort()", ""),
    row!(Array, "sorted", Method, "", "List<T>", false, "The cells as a sorted list.", "a.sorted()", "List<Int>"),
    row!(Array, "contains", Method, "x: T", "Bool", false, "True when some cell equals `x`.", "a.contains(7)", "Bool"),
    row!(Array, "find", Method, "x: T", "?Int", false, "The index of the first cell equal to `x`.", "a.find(7)", "?Int"),
    row!(Array, "first_ge", Method, "x: T", "?Int", false, "In a sorted array, the index of the first cell `>= x`.", "a.first_ge(2)", "?Int"),
    row!(Array, "last_le", Method, "x: T", "?Int", false, "In a sorted array, the index of the last cell `<= x`.", "a.last_le(2)", "?Int"),
    row!(Array, "windows", Method, "k: Int", "List<List<T>>", false, "Every run of `k` neighbouring cells.", "a.windows(2)", "List<List<Int>>"),
];
