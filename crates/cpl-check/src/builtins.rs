//! What the language itself gives a `List`, a `Str`, a `Map`, a `Set` and
//! an array: the one list of their members, for anything that shows them
//! (completion, signature help, docs). `T` is the element type, `K` and
//! `V` a map's key and value.
//!
//! Every row is run by `crates/cpl-cli/tests/builtin_catalog.rs`: its
//! `example` must have the type `example_ty` in `cpl run`, so a row that
//! the compiler does not agree with fails the build.

use crate::prelude;

mod foreign;
pub use foreign::*;

/// The built-in types that have members.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Receiver {
    List,
    Str,
    Map,
    Set,
    /// `[D]T` and `[0..n]T`.
    Array,
    /// A `Map` with no order: the members that need none.
    HashMap,
    /// A `Set` with no order.
    HashSet,
}

impl Receiver {
    pub const ALL: [Receiver; 7] = [
        Receiver::List,
        Receiver::Str,
        Receiver::Map,
        Receiver::Set,
        Receiver::Array,
        Receiver::HashMap,
        Receiver::HashSet,
    ];

    /// The name as written in a type: `List`, and `[D]T` for an array.
    pub fn name(self) -> &'static str {
        match self {
            Receiver::List => "List",
            Receiver::Str => "Str",
            Receiver::Map => "Map",
            Receiver::Set => "Set",
            Receiver::Array => "[D]T",
            Receiver::HashMap => "HashMap",
            Receiver::HashSet => "HashSet",
        }
    }

    /// The ordered type a hash receiver is the unordered form of: `Map` for `HashMap`.
    pub fn ordered(self) -> Option<Receiver> {
        match self {
            Receiver::HashMap => Some(Receiver::Map),
            Receiver::HashSet => Some(Receiver::Set),
            _ => None,
        }
    }
}

/// `h.first_ge(k)` on a `HashMap`: a member only the ordered form has, and what to write.
pub fn needs_order(type_name: &str, member: &str) -> Option<cpl_diag::msgs::Said> {
    let hash = *Receiver::ALL.iter().find(|r| r.name() == type_name)?;
    let ordered = hash.ordered()?;
    let on = |r: Receiver| builtin_members(r).any(|m| m.name == member);
    (on(ordered) && !on(hash))
        .then(|| cpl_diag::msgs::check::NO_ORDER_USE.at(&[&hash.name(), &ordered.name()]))
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
    /// True when a call must pass an argument: some parameter has no
    /// `= default`. `xs.sort()` needs none; `xs.swap(i, j)` does.
    pub fn needs_arguments(&self) -> bool {
        split_params(self.params).iter().any(|p| !p.contains(" = "))
    }

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

/// A type the language itself declares: what `cpl doc` shows for it.
#[derive(Clone, Copy, Debug)]
pub struct BuiltinType {
    pub name: &'static str,
    /// How it is written in a type, `HashMap<K, V>`.
    pub written: &'static str,
    /// What it is, as Markdown, with its C++, Python and Rust names so a
    /// search for `unordered_map` finds `HashMap`.
    pub doc: &'static str,
    /// Whose catalog rows it has; `None` for a scalar.
    pub members: Option<Receiver>,
}

/// Every built-in type `cpl doc` documents, in the order a reader meets
/// them (spec/core/02-values-types.md).
pub const BUILTIN_TYPES: &[BuiltinType] = &[
    BuiltinType { name: "Int", written: "Int", members: None, doc: "The default integer, `-(2^63 - 1) ..= 2^63 - 1`: C++ `long long`, Python `int`, Rust `i64`. `Int.MIN` is one above `-2^63`, which is `?Int`'s `None`; narrow storage is a range type such as `0..=255`, never another integer type." },
    BuiltinType { name: "I128", written: "I128", members: None, doc: "A signed 128-bit integer for products past `Int`: C++ `__int128`, Rust `i128`." },
    BuiltinType { name: "U64", written: "U64", members: None, doc: "An unsigned 64-bit integer that wraps, for hashes, masks and bit tricks: C++ `unsigned long long`, Rust `u64`." },
    BuiltinType { name: "F64", written: "F64", members: None, doc: "An IEEE double: C++ `double`, Python `float`, Rust `f64`. `==` is exact; `close(a, b, eps:)` is the tolerant compare." },
    BuiltinType { name: "Bool", written: "Bool", members: None, doc: "`true` or `false`, one byte: C++ `bool`. `Int(b)` is 1 or 0." },
    BuiltinType { name: "Char", written: "Char", members: None, doc: "An ASCII character `0..=127`, one byte, ordered by code: C++ `char`. `Int(c)` and `Char(i)` convert; `Char(i)` traps outside `0..=127`." },
    BuiltinType { name: "Str", written: "Str", members: Some(Receiver::Str), doc: "A flat sequence of `Char` indexed by `0..len`: C++ `string`, Python `str`, Rust `String`. `+` makes a new string; `s.push(c)` and `s[i] = c` change a `var` one in place." },
    BuiltinType { name: "List", written: "List<T>", members: Some(Receiver::List), doc: "A growable sequence indexed by `0..len`: C++ `vector`, Python `list`, Rust `Vec`. A counted array `[n]T` passes where a `List<T>` is read." },
    BuiltinType { name: "Array", written: "[D]T", members: Some(Receiver::Array), doc: "A fixed-size array indexed by the positions of domain `D`, never a raw `Int`: C++ `array` or a sized `vector`. `[n]T` is `[0..n]T`; `[R, C]T` is a dense row-major 2-D array indexed `g[r, c]`. Large arrays never live on the C++ stack." },
    BuiltinType { name: "Map", written: "Map<K, V>", members: Some(Receiver::Map), doc: "The lookup table: C++ `map`, Python `dict`, Rust `BTreeMap`. `for (k, v) in m` walks it in key order; when nothing observes the order, the C++ is a salted hash table." },
    BuiltinType { name: "HashMap", written: "HashMap<K, V>", members: Some(Receiver::HashMap), doc: "A hash table whatever the program observes: C++ `unordered_map`, Python `dict`, Rust `HashMap`. Salted with `splitmix64` against anti-hash tests. Walking one has no defined order, so it is not `Show`; `Map<K, V>` is the usual spelling." },
    BuiltinType { name: "Set", written: "Set<T>", members: Some(Receiver::Set), doc: "A set of distinct values: C++ `set`, Python `set`, Rust `BTreeSet`. It walks in order; when nothing observes the order, the C++ is a salted hash set." },
    BuiltinType { name: "HashSet", written: "HashSet<T>", members: Some(Receiver::HashSet), doc: "A hash set whatever the program observes: C++ `unordered_set`, Python `set`, Rust `HashSet`. Salted with `splitmix64`. Walking one has no defined order, so it is not `Show`; `Set<T>` is the usual spelling." },
];

/// The members of one built-in type.
pub fn builtin_members(receiver: Receiver) -> impl Iterator<Item = &'static BuiltinMember> {
    BUILTIN_MEMBERS
        .iter()
        .filter(move |member| member.receiver == receiver)
}

/// A one-character string literal's `Char` spelling: `"a"` is `'a'`.
pub fn char_literal(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut chars = inner.chars();
    let c = chars.next()?;
    (chars.next().is_none() && !matches!(c, '\'' | '\\' | '{' | '}' | '"'))
        .then(|| format!("'{c}'"))
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
use Receiver::{Array, HashMap, HashSet, List, Map, Set, Str};

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
    row!(List, "first", Get, "", "?T", false, "\\(O(1)\\). The first element, `None` when empty. For the first element satisfying a predicate,\nuse `find(p)`; for its key, `keys.find(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.first == None`\nties: `[(1, 'a'), (1, 'b')].first == Some((1, 'a'))`\ntraps: never\nexample: `[4, 5].first == Some(4)`", "xs.first", "?Int"),
    row!(List, "last", Get, "", "?T", false, "\\(O(1)\\). The last element, `None` when empty. For the last element satisfying a predicate,\nuse `rfind(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.last == None`\nties: `[(1, 'a'), (1, 'b')].last == Some((1, 'b'))`\ntraps: never\nexample: `[4, 5].last == Some(5)`", "xs.last", "?Int"),
    row!(List, "push", Method, "x: T", "", true, "Adds `x` at the end in O(1) amortized.\n\ncost: \\(O(1)\\) amortized", "xs.push(4)", ""),
    row!(List, "push_id", Method, "x: T", "Int", true, "Adds `x` at the end and gives the index it lands on, the list's old length, in O(1) amortized: an arena of nodes linked by index.\n\ncost: \\(O(1)\\) amortized", "xs.push_id(4)", "Int"),
    row!(List, "pop", Method, "", "?T", true, "Takes the last element off in O(1), or gives `None` when empty.\n\ncost: \\(O(1)\\)", "xs.pop()", "?Int"),
    row!(List, "get", Method, "i: Int", "?T", false, "The element at `i`, or `None` when `i` is outside the list. O(1).\n\ncost: \\(O(1)\\)", "xs.get(1)", "?Int"),
    row!(List, "insert", Method, "i: Int, x: T", "", true, "Puts `x` at index `i`, moving the rest up. O(n).\n\ncost: \\(O(n)\\)", "xs.insert(0, 9)", ""),
    row!(List, "remove", Method, "i: Int", "T", true, "Takes out the element at index `i`, moving the rest down. O(n).\n\ncost: \\(O(n)\\)", "xs.remove(0)", "Int"),
    row!(List, "swap", Method, "i: Int, j: Int", "", true, "Exchanges the elements at `i` and `j`. O(1).\n\ncost: \\(O(1)\\)", "xs.swap(0, 1)", ""),
    row!(List, "resize", Method, "n: Int, x: T", "", true, "Makes the length `n`, filling new places with `x`.", "xs.resize(5, 0)", ""),
    row!(List, "truncate", Method, "n: Int", "", true, "Keeps the first `n` elements.", "xs.truncate(2)", ""),
    row!(List, "cut_after", Method, "i: Int", "Int", false, "The cut after index `i` in walk order: `i + 1`, or `i` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "xs.cut_after(1)", "Int"),
    row!(List, "cut_before", Method, "i: Int", "Int", false, "The cut before index `i` in walk order: `i`, or `i + 1` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "xs.cut_before(1)", "Int"),
    row!(List, "reserve", Method, "n: Int", "", true, "Reserves capacity for at least `n` elements. Amortized O(n).\n\ncost: \\(O(n)\\)", "xs.reserve(5)", ""),
    row!(List, "extend", Method, "ys: List<T>", "", true, "Adds every element of `ys` at the end.", "xs.extend([7, 8])", ""),
    row!(List, "fill", Method, "x: T", "", true, "Sets every element to `x`. O(n).\n\ncost: \\(O(n)\\)", "xs.fill(0)", ""),
    row!(List, "clear", Method, "", "", true, "Removes every element.", "xs.clear()", ""),
    row!(List, "clone", Method, "", "List<T>", false, "\\(O(n)\\), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.\n\ncost: \\(O(n)\\) for a value of size \\(n\\), allocates for heap values\ntraps: never\nexample: `let a = [1]; var b = a.clone(); b.push(2); a == [1]`", "xs.clone()", "List<Int>"),
    row!(List, "to_list", Method, "", "List<T>", false, "\\(O(n)\\). An owned copy of the slice's elements, in order.\n\ncost: \\(O(n)\\)\ntraps: never\nexample: `[5, 6, 7][1..].to_list() == [6, 7]`", "xs[0..2].to_list()", "List<Int>"),
    row!(List, "reverse", Method, "range: ?Range<Int> = None", "", true, "\\(O(n)\\). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).\nWorks on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).\n\ncost: \\(O(n)\\), \\(O(1)\\) extra space\nempty: `var a: List<Int> = []; a.reverse(); a == []`\ntraps: when `range` falls outside the keys\nexample: `var a = [1, 2, 3]; a.reverse(); a == [3, 2, 1]`", "xs.reverse()", ""),
    row!(List, "reversed", Method, "", "List<T>", false, "\\(O(n)\\). A new list of the elements in reverse order; `self.rev` walks them without copying.\n\ncost: \\(O(n)\\)\nempty: `let e: List<Int> = []; e.reversed() == []`\norder: reverse source order, keyed from 0\ntraps: never\nexample: `[1, 2, 3].reversed() == [3, 2, 1]`", "xs.reversed()", "List<Int>"),
    row!(List, "sort", Method, "; by: T -> K = x => x, cmp: ?((T, T) -> Bool) = None", "", true, "Sort `self` in place by `by`; descending is a key, `by: x => desc(x)`. Stable: equal keys keep their order.\nWhen no single key orders the elements (an exchange argument, an angle), `cmp: (a, b) => Bool` says\nwhether `a` goes before `b`; give `by:` or `cmp:`, not both (decision 87).\nWorks on any `IndexedMut`, including slices: `a[l..=r].sort()`.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) extra space\nempty: `var a: List<Int> = []; a.sort(); a == []`\nties: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\ntraps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order (`cmp(x, x)`, or both `cmp(a, b)` and `cmp(b, a)`); otherwise never (beyond what `by` or `cmp` traps on)\nexample: `var a = [3, 1, 2]; a.sort(); a == [1, 2, 3]`\nexample: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\nexample: `var a = [(3, 1), (1, 2), (2, 2)]; a.sort(cmp: (p, q) => p.0 * q.1 < q.0 * p.1); a == [(1, 2), (2, 2), (3, 1)]`\nexample: `var a = [2, 1]; a.sort(cmp: (x, y) => x <= y); a == [1, 2]` (traps R0001)\nexample: `var a = [2, 1]; a.sort(by: x => x, cmp: (x, y) => x < y); a == [1, 2]` (error E0309)", "xs.sort()", ""),
    row!(List, "sorted", Method, "; by: T -> K = x => x, cmp: ?((T, T) -> Bool) = None", "List<T>", false, "The elements of `self` sorted by `by` or `cmp`: the value form of `sort`. A new `List`, keyed\nfrom 0; to keep keys, sort the keys instead (`a.keys.sorted(by: i => a[i])`).\nStable: equal keys keep their relative order.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) space\nempty: `let a: List<Int> = []; a.sorted() == []`\nties: `[(1, 'b'), (0, 'z'), (1, 'a')].sorted(by: p => p.0) == [(0, 'z'), (1, 'b'), (1, 'a')]`\norder: increasing `by` (decreasing with `by: x => desc(x)`), keyed from 0\ntraps: as `sort`\nexample: `[3, 1, 2].sorted() == [1, 2, 3]`\nexample: `[3, 1, 2].sorted(by: x => desc(x)) == [3, 2, 1]`\nexample: `[(3, 1), (1, 2), (2, 2)].sorted(cmp: (p, q) => p.0 * q.1 < q.0 * p.1) == [(1, 2), (2, 2), (3, 1)]`", "xs.sorted()", "List<Int>"),
    row!(List, "map", Method, "f: T -> U", "List<U>", false, "A view giving `f(x)` for each element; compiled, it is built as a list in \\(O(n)\\). Indexed when `self` is, with the same\nkeys: `a.map(f)[i]` is `f(a[i])`, calling `f` on every read (`collect()` once to cache).\n\ncost: \\(O(n)\\) to make when compiled; one call of `f` per element read\nempty: `let a: List<Int> = []; a.map(x => x * x).collect() == []`\norder: source order, source keys\ntraps: never (beyond what `f` traps on)\nexample: `[1, 2, 3].map(x => x * x).collect() == [1, 4, 9]`\nexample: `domain V = 1..=3; let a: [V]Int = [4, 5, 6]; a.map(x => -x)[V(2)] == -5`", "xs.map(x => x * 2)", "List<Int>"),
    row!(List, "filter", Method, "p: T -> Bool", "List<T>", false, "A view of the elements satisfying `p`, in order; compiled, it is built as a list in \\(O(n)\\).\n\nKeys survive: each element keeps its source key, so `a.filter(p).items` gives `(i, a[i])`\nwith `a`'s own `i`. `p` runs as the view is walked, once per element per walk.\nBidirectional when `self` is.\n\ncost: \\(O(n)\\) to make when compiled; \\(O(n)\\) calls of `p` per walk\nempty: `let a: List<Int> = []; a.filter(x => x > 0).collect() == []`\norder: source order\ntraps: never (beyond what `p` traps on)\nexample: `[1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4]`\nexample: `[5, 6, 7].filter(x => x > 5).items.collect() == [(1, 6), (2, 7)]`", "xs.filter(x => x > 1)", "_"),
    row!(List, "contains", Method, "x: T", "Bool", false, "\\(O(n)\\). Whether some element equals `x`. A type with a faster `contains` of its own (a set,\na map, a bag, a hashed or keyed type) declares it, and its declared member wins; only those\nsub-linear `Contains` types take `x in s` (R12.4.7).\n\ncost: \\(O(n)\\) comparisons\nempty: `let a: List<Int> = []; a.contains(3) == false`\ntraps: never\nexample: `(1..=3).map(x => x * x).contains(4)`", "xs.contains(2)", "Bool"),
    row!(List, "count", Method, "p: T -> Bool = _ => true", "Int", false, "How many elements satisfy `p` (every element without `p`).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.count() == 0`\ntraps: never\nexample: `[1, 2, 3, 4].count(x => x % 2 == 0) == 2`\nexample: `[1, 2, 3].pairs().count() == 3`", "xs.count(x => x == 2)", "Int"),
    row!(List, "sum", Method, "of: T -> U = x => x", "T", false, "Sum of all elements (`T.zero` when empty), on any `Seq` of a type with `+`; `of:` maps each\nelement first (`xs.sum(of: x => x * x)`). \\(O(n)\\).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.sum() == 0`\ntraps: never (overflow wraps in release, traps in debug)\nexample: `[1, 2, 3].sum(of: x => x * x) == 14`", "xs.sum()", "Int"),
    row!(List, "min", Method, "by: T -> K = x => x, ties: Ties = Ties.First", "?T", false, "The least element by `by`, the first on ties (`ties: Ties.Last` for the last); `None` when empty.\n\nA method call (`s.min()`) picks this fold; a plain call with two values (`min(a, b)`) is the core binary `min`. `a.keys.min(by: i => a[i])`\nis argmin. \\(O(n)\\).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.min() == None`\nties: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\ntraps: never\nexample: `[3, 1, 1].min() == Some(1)`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0, ties: Ties.Last) == Some((1, 'b'))`", "xs.min()", "?Int"),
    row!(List, "max", Method, "by: T -> K = x => x, ties: Ties = Ties.First", "?T", false, "The greatest element by `by`, the first on ties (`ties: Ties.Last` for the last); `None`\nwhen empty.\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.max() == None`\nties: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\ntraps: never\nexample: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\nexample: `[(2, 'a'), (2, 'b')].max(by: .0, ties: Ties.Last) == Some((2, 'b'))`", "xs.max()", "?Int"),
    row!(List, "all", Method, "p: T -> Bool", "Bool", false, "Whether every element satisfies `p`; stops at the first that doesn't.\n\ncost: \\(O(n)\\), stopping at the first failure\nempty: `let a: List<Int> = []; a.all(x => x > 0) == true`\ntraps: never\nexample: `[1, 2].all(x => x > 1) == false`", "xs.all(x => x > 0)", "Bool"),
    row!(List, "any", Method, "p: T -> Bool", "Bool", false, "Whether some element satisfies `p`; stops at the first.\n\ncost: \\(O(n)\\), stopping at the first match\nempty: `let a: List<Int> = []; not a.any(x => x > 0)`\ntraps: never\nexample: `[1, 2].any(x => x > 1) == true`", "xs.any(x => x > 0)", "Bool"),
    row!(List, "find", Method, "x: T", "?Int", false, "The index of the first element equal to `x`. The first element a test holds for is `xs.first(x => x > 1)`. O(n).\n\ncost: \\(O(n)\\)", "xs.find(2)", "?Int"),
    row!(List, "items", Get, "", "List<(Int, T)>", false, "A view of `(key, element)` pairs with the source's own keys. A `for` or a comprehension reads\neach element in place; as a value, compiled, it is built as a list in \\(O(n)\\).\n\ncost: \\(O(n)\\) as a value; \\(O(1)\\) as a `for` source\nempty: `let a: List<Int> = []; a.items.collect() == []`\norder: source order\ntraps: never\nexample: `[5, 6][1..].items.collect() == [(1, 6)]`", "xs.items", "List<(Int, Int)>"),
    row!(List, "first_ge", Method, "x: T", "?Int", false, "In a sorted list, the index of the first element `>= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "xs.first_ge(2)", "?Int"),
    row!(List, "first_gt", Method, "target: K, by: T -> K = x => x", "?Int", false, "First index in sorted `self` with `by(self[i]) > target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.first_gt(3) == None`\nties: `[3, 3, 4].first_gt(3) == Some(2)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].first_gt(3) == Some(3)`", "xs.first_gt(2)", "?Int"),
    row!(List, "last_le", Method, "x: T", "?Int", false, "In a sorted list, the index of the last element `<= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "xs.last_le(2)", "?Int"),
    row!(List, "last_lt", Method, "target: K, by: T -> K = x => x", "?Int", false, "Last index in sorted `self` with `by(self[i]) < target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.last_lt(3) == None`\nties: `[2, 3, 3].last_lt(3) == Some(0)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].last_lt(3) == Some(0)`", "xs.last_lt(2)", "?Int"),
    row!(List, "reach", Method, "while: (T, T) -> Bool; cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(xs[i], xs[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; checked builds trap when `while` is not monotone.\n\ncost: \\(O(n)\\)", "xs.reach(while: (a, b) => a + b <= 10)", "List<Int>"),

    row!(Str, "len", Get, "", "Int", false, "How many characters.", "s.len", "Int"),
    row!(Str, "items", Get, "", "List<(Int, Char)>", false, "A view of `(key, element)` pairs with the source's own keys. A `for` or a comprehension reads\neach element in place; as a value, compiled, it is built as a list in \\(O(n)\\).\n\ncost: \\(O(n)\\) as a value; \\(O(1)\\) as a `for` source\nempty: `let a: List<Int> = []; a.items.collect() == []`\norder: source order\ntraps: never\nexample: `[5, 6][1..].items.collect() == [(1, 6)]`", "s.items", "List<(Int, Char)>"),
    row!(Str, "is_empty", Get, "", "Bool", false, "True for the empty text.", "s.is_empty", "Bool"),
    row!(Str, "first", Get, "", "?Char", false, "\\(O(1)\\). The first element, `None` when empty. For the first element satisfying a predicate,\nuse `find(p)`; for its key, `keys.find(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.first == None`\nties: `[(1, 'a'), (1, 'b')].first == Some((1, 'a'))`\ntraps: never\nexample: `[4, 5].first == Some(4)`", "s.first", "?Char"),
    row!(Str, "last", Get, "", "?Char", false, "\\(O(1)\\). The last element, `None` when empty. For the last element satisfying a predicate,\nuse `rfind(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.last == None`\nties: `[(1, 'a'), (1, 'b')].last == Some((1, 'b'))`\ntraps: never\nexample: `[4, 5].last == Some(5)`", "s.last", "?Char"),
    row!(Str, "get", Method, "i: Int", "?Char", false, "The character at `i`, or `None` when `i` is outside the text. O(1).\n\ncost: \\(O(1)\\)", "s.get(1)", "?Char"),
    row!(Str, "push", Method, "c: Char", "", true, "Adds `c` at the end in O(1) amortized.\n\ncost: \\(O(1)\\) amortized", "s.push('!')", ""),
    row!(Str, "pop", Method, "", "?Char", true, "Takes the last character off in O(1), or gives `None` when empty.\n\ncost: \\(O(1)\\)", "s.pop()", "?Char"),
    row!(Str, "swap", Method, "i: Int, j: Int", "", true, "Exchanges the characters at `i` and `j`. O(1).\n\ncost: \\(O(1)\\)", "s.swap(0, 1)", ""),
    row!(Str, "resize", Method, "n: Int, c: Char", "", true, "Makes the length `n`, filling new places with `c`.", "s.resize(7, '!')", ""),
    row!(Str, "truncate", Method, "n: Int", "", true, "Keeps the first `n` characters.", "s.truncate(2)", ""),
    row!(Str, "fill", Method, "c: Char", "", true, "Sets every character to `c`. O(n).\n\ncost: \\(O(n)\\)", "s.fill('a')", ""),
    row!(Str, "clear", Method, "", "", true, "Removes every character.", "s.clear()", ""),
    row!(Str, "clone", Method, "", "Str", false, "\\(O(n)\\), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.\n\ncost: \\(O(n)\\) for a value of size \\(n\\), allocates for heap values\ntraps: never\nexample: `let a = [1]; var b = a.clone(); b.push(2); a == [1]`", "s.clone()", "Str"),
    row!(Str, "to_str", Method, "", "Str", false, "An owned `Str` copy of this string or view. O(n).\n\ncost: \\(O(n)\\)", "s[0..2].to_str()", "Str"),
    row!(Str, "reverse", Method, "range: ?Range<Int> = None", "", true, "\\(O(n)\\). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).\nWorks on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).\n\ncost: \\(O(n)\\), \\(O(1)\\) extra space\nempty: `var a: List<Int> = []; a.reverse(); a == []`\ntraps: when `range` falls outside the keys\nexample: `var a = [1, 2, 3]; a.reverse(); a == [3, 2, 1]`", "s.reverse()", ""),
    row!(Str, "reversed", Method, "", "Str", false, "\\(O(n)\\). A new list of the elements in reverse order; `self.rev` walks them without copying.\n\ncost: \\(O(n)\\)\nempty: `let e: List<Int> = []; e.reversed() == []`\norder: reverse source order, keyed from 0\ntraps: never\nexample: `[1, 2, 3].reversed() == [3, 2, 1]`", "s.reversed()", "Str"),
    row!(Str, "sort", Method, "; by: Char -> K = x => x, cmp: ?((Char, Char) -> Bool) = None", "", true, "Sort `self` in place by `by`; descending is a key, `by: x => desc(x)`. Stable: equal keys keep their order.\nWhen no single key orders the elements (an exchange argument, an angle), `cmp: (a, b) => Bool` says\nwhether `a` goes before `b`; give `by:` or `cmp:`, not both (decision 87).\nWorks on any `IndexedMut`, including slices: `a[l..=r].sort()`.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) extra space\nempty: `var a: List<Int> = []; a.sort(); a == []`\nties: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\ntraps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order (`cmp(x, x)`, or both `cmp(a, b)` and `cmp(b, a)`); otherwise never (beyond what `by` or `cmp` traps on)\nexample: `var a = [3, 1, 2]; a.sort(); a == [1, 2, 3]`\nexample: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\nexample: `var a = [(3, 1), (1, 2), (2, 2)]; a.sort(cmp: (p, q) => p.0 * q.1 < q.0 * p.1); a == [(1, 2), (2, 2), (3, 1)]`\nexample: `var a = [2, 1]; a.sort(cmp: (x, y) => x <= y); a == [1, 2]` (traps R0001)\nexample: `var a = [2, 1]; a.sort(by: x => x, cmp: (x, y) => x < y); a == [1, 2]` (error E0309)", "s.sort()", ""),
    row!(Str, "sorted", Method, "; by: Char -> K = x => x, cmp: ?((Char, Char) -> Bool) = None", "List<Char>", false, "The elements of `self` sorted by `by` or `cmp`: the value form of `sort`. A new `List`, keyed\nfrom 0; to keep keys, sort the keys instead (`a.keys.sorted(by: i => a[i])`).\nStable: equal keys keep their relative order.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) space\nempty: `let a: List<Int> = []; a.sorted() == []`\nties: `[(1, 'b'), (0, 'z'), (1, 'a')].sorted(by: p => p.0) == [(0, 'z'), (1, 'b'), (1, 'a')]`\norder: increasing `by` (decreasing with `by: x => desc(x)`), keyed from 0\ntraps: as `sort`\nexample: `[3, 1, 2].sorted() == [1, 2, 3]`\nexample: `[3, 1, 2].sorted(by: x => desc(x)) == [3, 2, 1]`\nexample: `[(3, 1), (1, 2), (2, 2)].sorted(cmp: (p, q) => p.0 * q.1 < q.0 * p.1) == [(1, 2), (2, 2), (3, 1)]`", "s.sorted()", "List<Char>"),
    row!(Str, "map", Method, "f: Char -> U", "List<U>", false, "A view giving `f(x)` for each element; compiled, it is built as a list in \\(O(n)\\). Indexed when `self` is, with the same\nkeys: `a.map(f)[i]` is `f(a[i])`, calling `f` on every read (`collect()` once to cache).\n\ncost: \\(O(n)\\) to make when compiled; one call of `f` per element read\nempty: `let a: List<Int> = []; a.map(x => x * x).collect() == []`\norder: source order, source keys\ntraps: never (beyond what `f` traps on)\nexample: `[1, 2, 3].map(x => x * x).collect() == [1, 4, 9]`\nexample: `domain V = 1..=3; let a: [V]Int = [4, 5, 6]; a.map(x => -x)[V(2)] == -5`", "s.map(c => Int(c == 'l'))", "List<Int>"),
    row!(Str, "filter", Method, "p: Char -> Bool", "List<Char>", false, "A view of the elements satisfying `p`, in order; compiled, it is built as a list in \\(O(n)\\).\n\nKeys survive: each element keeps its source key, so `a.filter(p).items` gives `(i, a[i])`\nwith `a`'s own `i`. `p` runs as the view is walked, once per element per walk.\nBidirectional when `self` is.\n\ncost: \\(O(n)\\) to make when compiled; \\(O(n)\\) calls of `p` per walk\nempty: `let a: List<Int> = []; a.filter(x => x > 0).collect() == []`\norder: source order\ntraps: never (beyond what `p` traps on)\nexample: `[1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4]`\nexample: `[5, 6, 7].filter(x => x > 5).items.collect() == [(1, 6), (2, 7)]`", "s.filter(c => c > 'e')", "_"),
    row!(Str, "join", Method, "parts: S", "Str", false, "The elements of `parts` in order with `self` between each adjacent pair: `\", \".join(xs)`.\n\nEach element is written as string interpolation `\"{x}\"` writes it, so `Str`, `Char` and\nnumbers all join without a `map`.\n\ncost: \\(O(\\lvert \\mathit{output} \\rvert)\\)\nempty: `\", \".join(List<Str>()) == \"\"`\ntraps: never\nexample: `\", \".join([\"a\", \"b\", \"c\"]) == \"a, b, c\"`\nexample: `\" \".join([1, 2, 3]) == \"1 2 3\"`\nexample: `\"-\".join([\"x\"]) == \"x\"`", "\", \".join([\"a\", \"b\"])", "Str"),
    row!(Str, "find_all", Method, "pat: P", "List<Int>", false, "The start key of every occurrence of `pat` in `self`, overlapping ones included\n(Knuth-Morris-Pratt).\n\nWorks on any `Indexed` pair (`Str`, `List<T>`, `[D]T`, views) without conversion; keys\nare those of `self`. An empty pattern occurs at every key, never at the end: a match starts at an element.\n\ncost: \\(O(\\lvert \\mathit{self} \\rvert + \\lvert \\mathit{pat} \\rvert)\\)\nempty: `(\"ab\".find_all(\"\").collect(), \"\".find_all(\"a\").collect()) == ([0, 1], [])`\nties: `\"aaa\".find_all(\"aa\").collect() == [0, 1]`\norder: increasing start key\ntraps: never\nexample: `\"aaaa\".find_all(\"aa\").collect() == [0, 1, 2]`\nexample: `\"ab\".find_all(\"\").collect() == [0, 1]`", "s.find_all(\"l\")", "List<Int>"),
    row!(Str, "contains", Method, "x: Char", "Bool", false, "\\(O(n)\\). Whether some element equals `x`. A type with a faster `contains` of its own (a set,\na map, a bag, a hashed or keyed type) declares it, and its declared member wins; only those\nsub-linear `Contains` types take `x in s` (R12.4.7).\n\ncost: \\(O(n)\\) comparisons\nempty: `let a: List<Int> = []; a.contains(3) == false`\ntraps: never\nexample: `(1..=3).map(x => x * x).contains(4)`", "s.contains('e')", "Bool"),
    row!(Str, "count", Method, "p: Char -> Bool = _ => true", "Int", false, "How many elements satisfy `p` (every element without `p`).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.count() == 0`\ntraps: never\nexample: `[1, 2, 3, 4].count(x => x % 2 == 0) == 2`\nexample: `[1, 2, 3].pairs().count() == 3`", "s.count(c => c == 'l')", "Int"),
    row!(Str, "min", Method, "by: Char -> K = x => x, ties: Ties = Ties.First", "?Char", false, "The least element by `by`, the first on ties (`ties: Ties.Last` for the last); `None` when empty.\n\nA method call (`s.min()`) picks this fold; a plain call with two values (`min(a, b)`) is the core binary `min`. `a.keys.min(by: i => a[i])`\nis argmin. \\(O(n)\\).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.min() == None`\nties: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\ntraps: never\nexample: `[3, 1, 1].min() == Some(1)`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0, ties: Ties.Last) == Some((1, 'b'))`", "s.min()", "?Char"),
    row!(Str, "max", Method, "by: Char -> K = x => x, ties: Ties = Ties.First", "?Char", false, "The greatest element by `by`, the first on ties (`ties: Ties.Last` for the last); `None`\nwhen empty.\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.max() == None`\nties: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\ntraps: never\nexample: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\nexample: `[(2, 'a'), (2, 'b')].max(by: .0, ties: Ties.Last) == Some((2, 'b'))`", "s.max()", "?Char"),
    row!(Str, "all", Method, "p: Char -> Bool", "Bool", false, "Whether every element satisfies `p`; stops at the first that doesn't.\n\ncost: \\(O(n)\\), stopping at the first failure\nempty: `let a: List<Int> = []; a.all(x => x > 0) == true`\ntraps: never\nexample: `[1, 2].all(x => x > 1) == false`", "s.all(c => c >= 'a')", "Bool"),
    row!(Str, "any", Method, "p: Char -> Bool", "Bool", false, "Whether some element satisfies `p`; stops at the first.\n\ncost: \\(O(n)\\), stopping at the first match\nempty: `let a: List<Int> = []; not a.any(x => x > 0)`\ntraps: never\nexample: `[1, 2].any(x => x > 1) == true`", "s.any(c => c == 'e')", "Bool"),
    row!(Str, "cut_after", Method, "i: Int", "Int", false, "The cut after index `i` in walk order: `i + 1`, or `i` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "s.cut_after(1)", "Int"),
    row!(Str, "cut_before", Method, "i: Int", "Int", false, "The cut before index `i` in walk order: `i`, or `i + 1` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "s.cut_before(1)", "Int"),
    row!(Str, "find", Method, "c: Char", "?Int", false, "The index of the first character equal to `c`, or where the first copy of `c` starts when `c` is a `Str`.", "s.find('e')", "?Int"),
    row!(Str, "first_ge", Method, "c: Char", "?Int", false, "In a sorted string, the index of the first character `>= c`. O(log n).\n\ncost: \\(O(\\log n)\\)", "s.first_ge('e')", "?Int"),
    row!(Str, "first_gt", Method, "target: K, by: Char -> K = x => x", "?Int", false, "First index in sorted `self` with `by(self[i]) > target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.first_gt(3) == None`\nties: `[3, 3, 4].first_gt(3) == Some(2)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].first_gt(3) == Some(3)`", "s.first_gt('e')", "?Int"),
    row!(Str, "last_le", Method, "c: Char", "?Int", false, "In a sorted string, the index of the last character `<= c`. O(log n).\n\ncost: \\(O(\\log n)\\)", "s.last_le('l')", "?Int"),
    row!(Str, "last_lt", Method, "target: K, by: Char -> K = x => x", "?Int", false, "Last index in sorted `self` with `by(self[i]) < target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.last_lt(3) == None`\nties: `[2, 3, 3].last_lt(3) == Some(0)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].last_lt(3) == Some(0)`", "s.last_lt('l')", "?Int"),
    row!(Str, "reach", Method, "while: (Char, Char) -> Bool; cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(s[i], s[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; checked builds trap when `while` is not monotone.\n\ncost: \\(O(n)\\)", "s.reach(while: (a, b) => a <= b)", "List<Int>"),

    row!(Map, "len", Get, "", "Int", false, "How many keys.", "m.len", "Int"),
    row!(Map, "items", Get, "", "List<(K, V)>", false, "Each key with its value, smallest key first: `for (k, v) in m.items`.\n\ncost: \\(O(n)\\)", "m.items", "List<(Int, Int)>"),
    row!(Map, "keys", Get, "", "List<K>", false, "Each key, smallest first (O(1) view in loops and folds): `for k in m.keys`.\n\ncost: \\(O(1)\\)", "m.keys", "List<Int>"),
    row!(Map, "values", Get, "", "List<V>", false, "Each value, smallest key first (O(1) view in loops and folds): `for v in m.values`.\n\ncost: \\(O(1)\\)", "m.values", "List<Int>"),
    row!(Map, "is_empty", Get, "", "Bool", false, "True when there is no key.", "m.is_empty", "Bool"),
    row!(Map, "first", Get, "", "?(K, V)", false, "The smallest key with its value.", "m.first", "?(Int, Int)"),
    row!(Map, "last", Get, "", "?(K, V)", false, "The largest key with its value.", "m.last", "?(Int, Int)"),
    row!(Map, "get", Method, "k: K", "?V", false, "The value at `k`, or `None` when `k` is not a key. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.get(1)", "?Int"),
    row!(Map, "contains", Method, "k: K", "Bool", false, "True when `k` is a key; `k in m` reads the same. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.contains(1)", "Bool"),
    row!(Map, "remove", Method, "k: K", "?V", true, "Takes `k` out and gives the value it had. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.remove(1)", "?Int"),
    row!(Map, "clear", Method, "", "", true, "Removes every key.\n\ncost: \\(O(n)\\)", "m.clear()", ""),
    row!(Map, "reserve", Method, "n: Int", "", true, "Nothing: a sorted map has no room to make, so `reserve` keeps code that switches between `Map` and `HashMap` unchanged.\n\ncost: \\(O(1)\\)", "m.reserve(5)", ""),
    row!(Map, "clone", Method, "", "Map<K, V>", false, "A separate map with the same entries.\n\ncost: \\(O(n)\\)", "m.clone()", "Map<Int, Int>"),
    row!(Map, "pop_first", Method, "", "?(K, V)", true, "Takes out the smallest key with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.pop_first()", "?(Int, Int)"),
    row!(Map, "pop_last", Method, "", "?(K, V)", true, "Takes out the largest key with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.pop_last()", "?(Int, Int)"),
    row!(Map, "first_ge", Method, "k: K", "?(K, V)", false, "The smallest key `>= k` with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.first_ge(2)", "?(Int, Int)"),
    row!(Map, "first_gt", Method, "k: K", "?(K, V)", false, "The smallest key `> k` with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.first_gt(1)", "?(Int, Int)"),
    row!(Map, "last_le", Method, "k: K", "?(K, V)", false, "The largest key `<= k` with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.last_le(4)", "?(Int, Int)"),
    row!(Map, "last_lt", Method, "k: K", "?(K, V)", false, "The largest key `< k` with its value. O(log n).\n\ncost: \\(O(\\log n)\\)", "m.last_lt(5)", "?(Int, Int)"),
    row!(Map, "range", Method, "r: Range<K>", "SortedRange<(K, V)>", false, "O(1) key-range view; `.first`, `.last`, `.is_empty` are O(log n) and iteration is O(log n + k). Use `SortedSet`/`SortedBag` for O(log n) `.len`.\n\ncost: \\(O(1)\\)", "m.range(1..5).first", "?(Int, Int)"),

    row!(Set, "len", Get, "", "Int", false, "How many elements.", "st.len", "Int"),
    row!(Set, "is_empty", Get, "", "Bool", false, "True when there is no element.", "st.is_empty", "Bool"),
    row!(Set, "first", Get, "", "?T", false, "The smallest element.", "st.first", "?Int"),
    row!(Set, "last", Get, "", "?T", false, "The largest element.", "st.last", "?Int"),
    row!(Set, "insert", Method, "x: T", "Bool", true, "Adds `x`; true when it was not there before. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.insert(5)", "Bool"),
    row!(Set, "add", Method, "x: T", "Bool", true, "The same as `insert`. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.add(5)", "Bool"),
    row!(Set, "remove", Method, "x: T", "Bool", true, "Takes `x` out; true when it was there. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.remove(4)", "Bool"),
    row!(Set, "set", Method, "x: T, on: Bool", "", true, "Adds `x` when `on` is true, or removes it otherwise. O(log n) on `Set`, O(1) amortized on `HashSet`.\n\ncost: \\(O(\\log n)\\)", "st.set(5, on: true)", ""),
    row!(Set, "contains", Method, "x: T", "Bool", false, "True when `x` is in the set; `x in st` reads the same. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.contains(4)", "Bool"),
    row!(Set, "clear", Method, "", "", true, "Removes every element.\n\ncost: \\(O(n)\\)", "st.clear()", ""),
    row!(Set, "clone", Method, "", "Set<T>", false, "A separate set with the same elements.\n\ncost: \\(O(n)\\)", "st.clone()", "Set<Int>"),
    row!(Set, "sorted", Method, "", "List<T>", false, "The elements as a sorted list.\n\ncost: \\(O(n)\\)", "st.sorted()", "List<Int>"),
    row!(Set, "min", Method, "", "?T", false, "The smallest element. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.min()", "?Int"),
    row!(Set, "max", Method, "", "?T", false, "The largest element. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.max()", "?Int"),
    row!(Set, "pop_first", Method, "", "?T", true, "Takes out the smallest element. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.pop_first()", "?Int"),
    row!(Set, "pop_last", Method, "", "?T", true, "Takes out the largest element. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.pop_last()", "?Int"),
    row!(Set, "first_ge", Method, "x: T", "?T", false, "The smallest element `>= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.first_ge(5)", "?Int"),
    row!(Set, "first_gt", Method, "x: T", "?T", false, "The smallest element `> x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.first_gt(4)", "?Int"),
    row!(Set, "last_le", Method, "x: T", "?T", false, "The largest element `<= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.last_le(5)", "?Int"),
    row!(Set, "last_lt", Method, "x: T", "?T", false, "The largest element `< x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "st.last_lt(9)", "?Int"),
    row!(Set, "range", Method, "r: Range<T>", "SortedRange<T>", false, "O(1) key-range view; `.first`, `.last`, `.is_empty` are O(log n) and iteration is O(log n + k). Use `SortedSet`/`SortedBag` for O(log n) `.len`.\n\ncost: \\(O(1)\\)", "st.range(4..9).first", "?Int"),
    row!(Set, "drain", Method, "; after: ?T = none, before: ?T = none, while ok: (T) -> Bool", "List<T>", true, "Removes and returns the maximal run strictly after `after` (ascending), strictly before `before` (descending), or from the smallest element (ascending) while `ok(k)` holds. O(k log n).\n\ncost: \\(O(k \\log n)\\)", "st.drain(after: 4, while: x => x < 9)", "List<Int>"),

    row!(HashMap, "len", Get, "", "Int", false, "How many keys.\n\ncost: \\(O(1)\\)", "hm.len", "Int"),
    row!(HashMap, "items", Get, "", "List<(K, V)>", false, "Each key with its value, in no defined order: `for (k, v) in hm.items`.\n\ncost: \\(O(n)\\)", "hm.items", "List<(Int, Int)>"),
    row!(HashMap, "keys", Get, "", "List<K>", false, "Each key, in no defined order (O(1) view in loops and folds): `for k in hm.keys`.\n\ncost: \\(O(1)\\)", "hm.keys", "List<Int>"),
    row!(HashMap, "values", Get, "", "List<V>", false, "Each value, in no defined order (O(1) view in loops and folds): `for v in hm.values`.\n\ncost: \\(O(1)\\)", "hm.values", "List<Int>"),
    row!(HashMap, "is_empty", Get, "", "Bool", false, "True when there is no key.\n\ncost: \\(O(1)\\)", "hm.is_empty", "Bool"),
    row!(HashMap, "get", Method, "k: K", "?V", false, "The value at `k`, or `None` when `k` is not a key, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hm.get(1)", "?Int"),
    row!(HashMap, "contains", Method, "k: K", "Bool", false, "True when `k` is a key; `k in hm` reads the same, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hm.contains(1)", "Bool"),
    row!(HashMap, "remove", Method, "k: K", "?V", true, "Takes `k` out and gives the value it had, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hm.remove(1)", "?Int"),
    row!(HashMap, "clear", Method, "", "", true, "Removes every key.\n\ncost: \\(O(n)\\)", "hm.clear()", ""),
    row!(HashMap, "reserve", Method, "n: Int", "", true, "Makes room for at least `n` keys, so the map fills without rehashing.\n\ncost: \\(O(n)\\)", "hm.reserve(5)", ""),
    row!(HashMap, "clone", Method, "", "HashMap<K, V>", false, "A separate map with the same entries.\n\ncost: \\(O(n)\\)", "hm.clone().len", "Int"),

    row!(HashSet, "len", Get, "", "Int", false, "How many elements.\n\ncost: \\(O(1)\\)", "hs.len", "Int"),
    row!(HashSet, "is_empty", Get, "", "Bool", false, "True when there is no element.\n\ncost: \\(O(1)\\)", "hs.is_empty", "Bool"),
    row!(HashSet, "insert", Method, "x: T", "Bool", true, "Adds `x`; true when it was not there before, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hs.insert(5)", "Bool"),
    row!(HashSet, "add", Method, "x: T", "Bool", true, "The same as `insert`, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hs.add(5)", "Bool"),
    row!(HashSet, "remove", Method, "x: T", "Bool", true, "Takes `x` out; true when it was there, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hs.remove(4)", "Bool"),
    row!(HashSet, "set", Method, "x: T, on: Bool", "", true, "Adds `x` when `on` is true, or removes it otherwise, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hs.set(5, on: true)", ""),
    row!(HashSet, "contains", Method, "x: T", "Bool", false, "True when `x` is in the set; `x in hs` reads the same, in expected O(1) time by hashing.\n\ncost: \\(O(1)\\)", "hs.contains(4)", "Bool"),
    row!(HashSet, "clear", Method, "", "", true, "Removes every element.\n\ncost: \\(O(n)\\)", "hs.clear()", ""),
    row!(HashSet, "clone", Method, "", "HashSet<T>", false, "A separate set with the same elements.\n\ncost: \\(O(n)\\)", "hs.clone().len", "Int"),
    row!(HashSet, "sorted", Method, "", "List<T>", false, "The elements as a sorted list.\n\ncost: \\(O(n \\log n)\\)", "hs.sorted()", "List<Int>"),
    row!(HashSet, "min", Method, "", "?T", false, "The smallest element, by a scan of the table.\n\ncost: \\(O(n)\\)", "hs.min()", "?Int"),
    row!(HashSet, "max", Method, "", "?T", false, "The largest element, by a scan of the table.\n\ncost: \\(O(n)\\)", "hs.max()", "?Int"),

    row!(Array, "len", Get, "", "Int", false, "How many cells.", "a.len", "Int"),
    row!(Array, "is_empty", Get, "", "Bool", false, "True when the array has no cells.", "a.is_empty", "Bool"),
    row!(Array, "first", Get, "", "?T", false, "\\(O(1)\\). The first element, `None` when empty. For the first element satisfying a predicate,\nuse `find(p)`; for its key, `keys.find(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.first == None`\nties: `[(1, 'a'), (1, 'b')].first == Some((1, 'a'))`\ntraps: never\nexample: `[4, 5].first == Some(4)`", "a.first", "?Int"),
    row!(Array, "last", Get, "", "?T", false, "\\(O(1)\\). The last element, `None` when empty. For the last element satisfying a predicate,\nuse `rfind(p)`.\nTies don't apply: position decides.\n\ncost: \\(O(1)\\)\nempty: `let a: List<Int> = []; a.last == None`\nties: `[(1, 'a'), (1, 'b')].last == Some((1, 'b'))`\ntraps: never\nexample: `[4, 5].last == Some(5)`", "a.last", "?Int"),
    row!(Array, "get", Method, "i: Int", "?T", false, "The cell at `i`, or `None` when `i` is outside the array. O(1).\n\ncost: \\(O(1)\\)", "a.get(1)", "?Int"),
    row!(Array, "swap", Method, "i: D, j: D", "", true, "Exchanges the cells at `i` and `j`. O(1).\n\ncost: \\(O(1)\\)", "a.swap(0, 1)", ""),
    row!(Array, "fill", Method, "x: T", "", true, "Sets every cell to `x`. O(n).\n\ncost: \\(O(n)\\)", "a.fill(2)", ""),
    row!(Array, "clone", Method, "", "[D]T", false, "\\(O(n)\\), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.\n\ncost: \\(O(n)\\) for a value of size \\(n\\), allocates for heap values\ntraps: never\nexample: `let a = [1]; var b = a.clone(); b.push(2); a == [1]`", "a.clone()", "[0..3]Int"),
    row!(Array, "reverse", Method, "range: ?Range<Int> = None", "", true, "\\(O(n)\\). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).\nWorks on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).\n\ncost: \\(O(n)\\), \\(O(1)\\) extra space\nempty: `var a: List<Int> = []; a.reverse(); a == []`\ntraps: when `range` falls outside the keys\nexample: `var a = [1, 2, 3]; a.reverse(); a == [3, 2, 1]`", "a.reverse()", ""),
    row!(Array, "reversed", Method, "", "List<T>", false, "\\(O(n)\\). A new list of the elements in reverse order; `self.rev` walks them without copying.\n\ncost: \\(O(n)\\)\nempty: `let e: List<Int> = []; e.reversed() == []`\norder: reverse source order, keyed from 0\ntraps: never\nexample: `[1, 2, 3].reversed() == [3, 2, 1]`", "a.reversed()", "List<Int>"),
    row!(Array, "sort", Method, "; by: T -> K = x => x, cmp: ?((T, T) -> Bool) = None", "", true, "Sort `self` in place by `by`; descending is a key, `by: x => desc(x)`. Stable: equal keys keep their order.\nWhen no single key orders the elements (an exchange argument, an angle), `cmp: (a, b) => Bool` says\nwhether `a` goes before `b`; give `by:` or `cmp:`, not both (decision 87).\nWorks on any `IndexedMut`, including slices: `a[l..=r].sort()`.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) extra space\nempty: `var a: List<Int> = []; a.sort(); a == []`\nties: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\ntraps: in `cpl run` and `--checked`, when `cmp` is not a strict weak order (`cmp(x, x)`, or both `cmp(a, b)` and `cmp(b, a)`); otherwise never (beyond what `by` or `cmp` traps on)\nexample: `var a = [3, 1, 2]; a.sort(); a == [1, 2, 3]`\nexample: `var a = [(1, 'b'), (0, 'z'), (1, 'a')]; a.sort(by: p => p.0); a == [(0, 'z'), (1, 'b'), (1, 'a')]`\nexample: `var a = [(3, 1), (1, 2), (2, 2)]; a.sort(cmp: (p, q) => p.0 * q.1 < q.0 * p.1); a == [(1, 2), (2, 2), (3, 1)]`\nexample: `var a = [2, 1]; a.sort(cmp: (x, y) => x <= y); a == [1, 2]` (traps R0001)\nexample: `var a = [2, 1]; a.sort(by: x => x, cmp: (x, y) => x < y); a == [1, 2]` (error E0309)", "a.sort()", ""),
    row!(Array, "sorted", Method, "; by: T -> K = x => x, cmp: ?((T, T) -> Bool) = None", "List<T>", false, "The elements of `self` sorted by `by` or `cmp`: the value form of `sort`. A new `List`, keyed\nfrom 0; to keep keys, sort the keys instead (`a.keys.sorted(by: i => a[i])`).\nStable: equal keys keep their relative order.\n\ncost: \\(O(n \\log n)\\) comparisons, \\(O(n)\\) space\nempty: `let a: List<Int> = []; a.sorted() == []`\nties: `[(1, 'b'), (0, 'z'), (1, 'a')].sorted(by: p => p.0) == [(0, 'z'), (1, 'b'), (1, 'a')]`\norder: increasing `by` (decreasing with `by: x => desc(x)`), keyed from 0\ntraps: as `sort`\nexample: `[3, 1, 2].sorted() == [1, 2, 3]`\nexample: `[3, 1, 2].sorted(by: x => desc(x)) == [3, 2, 1]`\nexample: `[(3, 1), (1, 2), (2, 2)].sorted(cmp: (p, q) => p.0 * q.1 < q.0 * p.1) == [(1, 2), (2, 2), (3, 1)]`", "a.sorted()", "List<Int>"),
    row!(Array, "map", Method, "f: T -> U", "[D]U", false, "A view giving `f(x)` for each element; compiled, it is built as a list in \\(O(n)\\). Indexed when `self` is, with the same\nkeys: `a.map(f)[i]` is `f(a[i])`, calling `f` on every read (`collect()` once to cache).\n\ncost: \\(O(n)\\) to make when compiled; one call of `f` per element read\nempty: `let a: List<Int> = []; a.map(x => x * x).collect() == []`\norder: source order, source keys\ntraps: never (beyond what `f` traps on)\nexample: `[1, 2, 3].map(x => x * x).collect() == [1, 4, 9]`\nexample: `domain V = 1..=3; let a: [V]Int = [4, 5, 6]; a.map(x => -x)[V(2)] == -5`", "a.map(x => x + 1)", "[0..3]Int"),
    row!(Array, "filter", Method, "p: T -> Bool", "List<T>", false, "A view of the elements satisfying `p`, in order; compiled, it is built as a list in \\(O(n)\\).\n\nKeys survive: each element keeps its source key, so `a.filter(p).items` gives `(i, a[i])`\nwith `a`'s own `i`. `p` runs as the view is walked, once per element per walk.\nBidirectional when `self` is.\n\ncost: \\(O(n)\\) to make when compiled; \\(O(n)\\) calls of `p` per walk\nempty: `let a: List<Int> = []; a.filter(x => x > 0).collect() == []`\norder: source order\ntraps: never (beyond what `p` traps on)\nexample: `[1, 2, 3, 4].filter(x => x % 2 == 0).collect() == [2, 4]`\nexample: `[5, 6, 7].filter(x => x > 5).items.collect() == [(1, 6), (2, 7)]`", "a.filter(x => x > 1)", "_"),
    row!(Array, "contains", Method, "x: T", "Bool", false, "\\(O(n)\\). Whether some element equals `x`. A type with a faster `contains` of its own (a set,\na map, a bag, a hashed or keyed type) declares it, and its declared member wins; only those\nsub-linear `Contains` types take `x in s` (R12.4.7).\n\ncost: \\(O(n)\\) comparisons\nempty: `let a: List<Int> = []; a.contains(3) == false`\ntraps: never\nexample: `(1..=3).map(x => x * x).contains(4)`", "a.contains(7)", "Bool"),
    row!(Array, "count", Method, "p: T -> Bool = _ => true", "Int", false, "How many elements satisfy `p` (every element without `p`).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.count() == 0`\ntraps: never\nexample: `[1, 2, 3, 4].count(x => x % 2 == 0) == 2`\nexample: `[1, 2, 3].pairs().count() == 3`", "a.count(x => x == 7)", "Int"),
    row!(Array, "sum", Method, "of: T -> U = x => x", "T", false, "Sum of all elements (`T.zero` when empty), on any `Seq` of a type with `+`; `of:` maps each\nelement first (`xs.sum(of: x => x * x)`). \\(O(n)\\).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.sum() == 0`\ntraps: never (overflow wraps in release, traps in debug)\nexample: `[1, 2, 3].sum(of: x => x * x) == 14`", "a.sum()", "Int"),
    row!(Array, "min", Method, "by: T -> K = x => x, ties: Ties = Ties.First", "?T", false, "The least element by `by`, the first on ties (`ties: Ties.Last` for the last); `None` when empty.\n\nA method call (`s.min()`) picks this fold; a plain call with two values (`min(a, b)`) is the core binary `min`. `a.keys.min(by: i => a[i])`\nis argmin. \\(O(n)\\).\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.min() == None`\nties: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\ntraps: never\nexample: `[3, 1, 1].min() == Some(1)`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0) == Some((1, 'a'))`\nexample: `[(1, 'a'), (1, 'b')].min(by: .0, ties: Ties.Last) == Some((1, 'b'))`", "a.min()", "?Int"),
    row!(Array, "max", Method, "by: T -> K = x => x, ties: Ties = Ties.First", "?T", false, "The greatest element by `by`, the first on ties (`ties: Ties.Last` for the last); `None`\nwhen empty.\n\ncost: \\(O(n)\\)\nempty: `let a: List<Int> = []; a.max() == None`\nties: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\ntraps: never\nexample: `[(2, 'a'), (2, 'b')].max(by: .0) == Some((2, 'a'))`\nexample: `[(2, 'a'), (2, 'b')].max(by: .0, ties: Ties.Last) == Some((2, 'b'))`", "a.max()", "?Int"),
    row!(Array, "all", Method, "p: T -> Bool", "Bool", false, "Whether every element satisfies `p`; stops at the first that doesn't.\n\ncost: \\(O(n)\\), stopping at the first failure\nempty: `let a: List<Int> = []; a.all(x => x > 0) == true`\ntraps: never\nexample: `[1, 2].all(x => x > 1) == false`", "a.all(x => x > 0)", "Bool"),
    row!(Array, "any", Method, "p: T -> Bool", "Bool", false, "Whether some element satisfies `p`; stops at the first.\n\ncost: \\(O(n)\\), stopping at the first match\nempty: `let a: List<Int> = []; not a.any(x => x > 0)`\ntraps: never\nexample: `[1, 2].any(x => x > 1) == true`", "a.any(x => x > 0)", "Bool"),
    row!(Array, "cut_after", Method, "i: I", "I.cuts", false, "The cut after key `i` in walk order: `i.r`, or `i.l` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "a.cut_after(1)", "Int"),
    row!(Array, "cut_before", Method, "i: I", "I.cuts", false, "The cut before key `i` in walk order: `i.l`, or `i.r` on a `.rev`. O(1).\n\ncost: \\(O(1)\\)", "a.cut_before(1)", "Int"),
    row!(Array, "find", Method, "x: T", "?I", false, "The index of the first cell equal to `x`. The first cell a test holds for is `a.first(x => x > 1)`.", "a.find(7)", "?Int"),
    row!(Array, "items", Get, "", "List<(I, T)>", false, "A view of `(key, element)` pairs with the source's own keys. A `for` or a comprehension reads\neach element in place; as a value, compiled, it is built as a list in \\(O(n)\\).\n\ncost: \\(O(n)\\) as a value; \\(O(1)\\) as a `for` source\nempty: `let a: List<Int> = []; a.items.collect() == []`\norder: source order\ntraps: never\nexample: `[5, 6][1..].items.collect() == [(1, 6)]`", "a.items", "List<(Int, Int)>"),
    row!(Array, "first_ge", Method, "x: T", "?I", false, "In a sorted array, the index of the first cell `>= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "a.first_ge(2)", "?Int"),
    row!(Array, "first_gt", Method, "target: K, by: T -> K = x => x", "?I", false, "First index in sorted `self` with `by(self[i]) > target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.first_gt(3) == None`\nties: `[3, 3, 4].first_gt(3) == Some(2)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].first_gt(3) == Some(3)`", "a.first_gt(2)", "?Int"),
    row!(Array, "last_le", Method, "x: T", "?I", false, "In a sorted array, the index of the last cell `<= x`. O(log n).\n\ncost: \\(O(\\log n)\\)", "a.last_le(2)", "?Int"),
    row!(Array, "last_lt", Method, "target: K, by: T -> K = x => x", "?I", false, "Last index in sorted `self` with `by(self[i]) < target`, or `None`.\n\ncost: \\(O(\\log n)\\)\nempty: `let a: List<Int> = []; a.last_lt(3) == None`\nties: `[2, 3, 3].last_lt(3) == Some(0)`\ntraps: a checked build may trap R0401 on unsorted input, whose search is then wrong-way;\notherwise unsorted input gives an unspecified key\nexample: `[1, 3, 3, 5].last_lt(3) == Some(0)`", "a.last_lt(2)", "?Int"),
    row!(Array, "reach", Method, "while: (T, T) -> Bool; cyclic: Bool = false", "List<Int>", false, "For each `i`, the largest `k` with `while(a[i], a[i + j])` for all `1 <= j <= k` (mod `len` and `k < len` when `cyclic`). O(n) calls when reach drops by at most 1 per step; checked builds trap when `while` is not monotone.\n\ncost: \\(O(n)\\)", "a.reach(while: (x, y) => x + y <= 10)", "List<Int>"),
];

/// The members an optional has of its own, whatever std declares: each
/// name, its signature over the payload `T`, and what it gives. The
/// checker types these calls; completion offers them.
pub const OPTIONAL_MEMBERS: &[(&str, &str, &str)] = &[
    (
        "map",
        "map<U>(f: T -> U) -> ?U",
        "`f` of the value, or `None` when there is none",
    ),
    (
        "filter",
        "filter(f: T -> Bool) -> ?T",
        "the value when `f` holds on it, else `None`",
    ),
];

/// Whether `name` is one of an optional's own members.
pub fn optional_member(name: &str) -> bool {
    OPTIONAL_MEMBERS.iter().any(|(member, ..)| *member == name)
}

/// The members `member_ty` gives every domain (`V.first`), position
/// (`v.next`) and range (`r.start`) without a declaration to read: the
/// one list "did you mean" suggests from and completion offers.
pub const DOMAIN_MEMBERS: &[(&str, &str)] = &[
    ("count", "how many positions the domain has"),
    ("first", "the first position, or `None` when empty"),
    ("last", "the last position, or `None` when empty"),
    ("rev", "the positions in reverse order"),
    ("cuts", "the gaps between and around positions"),
    ("at", "the position at a 0-based index, or `None` outside"),
    ("nth", "the position at a 0-based index"),
    ("wrap", "the position at an index taken modulo the size"),
    ("mask", "a bit mask over the positions"),
    ("pairs", "every pair of positions `(u, v)` with `u < v`"),
    ("after", "the positions after a cut"),
    ("before", "the positions before a cut"),
    ("partition", "the positions split by a predicate"),
    ("filter", "the positions a predicate holds on"),
];
pub const POS_MEMBERS: &[(&str, &str)] = &[
    ("ord", "the 0-based rank of the position"),
    ("idx", "the position as a raw 0-based index"),
    ("label", "the value the position stands for"),
    ("prev", "the position before, or `None` at the first"),
    ("next", "the position after, or `None` at the last"),
    ("offset", "the position `k` steps away, or `None` outside"),
    ("ahead", "the position `k` steps after, or `None` outside"),
    ("back", "the position `k` steps before, or `None` outside"),
    ("l", "the cut before the position"),
    ("r", "the cut after the position"),
    ("before", "the cut before the position"),
    ("after", "the cut after the position"),
];
pub const RANGE_MEMBERS: &[(&str, &str)] = &[
    ("start", "the lower endpoint of the range"),
    ("end", "the exclusive upper endpoint of the range"),
    ("first", "the first element, or `None` when empty"),
    ("last", "the last element, or `None` when empty"),
    ("len", "the number of elements in the range"),
    ("is_empty", "true when the range holds no elements"),
    ("rev", "the range in reverse order"),
];

/// Whether `name` is in `members`.
pub fn is_member(members: &[(&str, &str)], name: &str) -> bool {
    members.iter().any(|(member, _)| *member == name)
}

/// `i: Int, f: (Int, Int) -> Int` split at its top-level commas, trimmed.
pub fn split_params(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, 0);
    // `Map<K, V>` nests; `a < b` in a default does not.
    let mut angles = 0i32;
    for (i, c) in text.char_indices() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '<' if text[..i].ends_with(|p: char| p.is_alphanumeric()) => angles += 1,
            '>' if angles > 0 && !text[..i].ends_with(['-', '=']) => angles -= 1,
            ',' if depth == 0 && angles > 0 => {}
            ',' if depth == 0 => {
                out.push(text[start..i].trim());
                start = i + 1;
            }
            // `while: (T, T) -> Bool; cyclic: ..`: the options start a new
            // piece that keeps its `;`.
            ';' if depth == 0 && angles == 0 && !text[start..i].trim().is_empty() => {
                out.push(text[start..i].trim());
                start = i;
            }
            _ => {}
        }
    }
    if !text[start..].trim().is_empty() {
        out.push(text[start..].trim());
    }
    out
}

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
        let d = &check("domain V = 1..=9\nprint(V.bound)\n")[0];
        assert_eq!(d.help.as_deref(), Some("did you mean `count`?"));
        let d = &check("domain V = 1..=9\nfor v in V:\n    print(v.nxt)\n")[0];
        assert_eq!(d.help.as_deref(), Some("did you mean `next`?"));
        // Every listed member is one `member_ty` knows.
        for (owner, members) in [
            ("V", crate::builtins::DOMAIN_MEMBERS),
            ("v", crate::builtins::POS_MEMBERS),
        ] {
            for (m, _) in members.iter() {
                let src = format!("domain V = 1..=9\nfor v in V:\n    let x = {owner}.{m}\n");
                let found: Vec<_> = check(&src).iter().map(|d| d.code.0).collect();
                assert!(!found.contains(&"E0203"), "{owner}.{m}: {found:?}");
            }
        }
    }
}

#[cfg(test)]
mod builtin_type_tests {
    use super::{prelude, Receiver, BUILTIN_TYPES};

    #[test]
    fn every_receiver_and_container_name_is_a_documented_type() {
        for receiver in Receiver::ALL {
            assert!(
                BUILTIN_TYPES.iter().any(|t| t.members == Some(receiver)),
                "{receiver:?}"
            );
        }
        for t in BUILTIN_TYPES {
            assert!(
                t.members == Some(Receiver::Array) || prelude::TYPE_NAMES.contains(&t.name),
                "{}",
                t.name
            );
        }
    }
}
