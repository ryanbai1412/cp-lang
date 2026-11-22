//! Names every file can use without a `use`: the prelude of
//! `plan/40-tooling.md` (`print`, `read`, `min`, `max`, `abs`, `List`,
//! `Map`, `Set`, `Int`, `Bool`, `Char`, `Str`, `INF`), the reductions
//! over sequences that `plan/12-domains-and-sequences.md` writes bare
//! (`sum`, `count`, `all`, `any`, `find`, `first`, `last`, `argmax`,
//! `argmin`), and the intrinsics the libraries are written on top of
//! (`rand`, `sort`, `swap`, `popcount`, ...; `stdlib/README.md` lists them).
//!
//! Everything else, `std` included, is cp source and resolves through
//! `use`. The `std` names a program has without writing one are listed in
//! `stdlib/prelude.cp`, which [`names`] reads.

use std::sync::OnceLock;

use cpl_diag::Span;
use cpl_syntax::ast;

use crate::ty::{Param, Ty};

/// What `stdlib/prelude.cp` lists: each name, and the module its `use`
/// takes it from (`std.mod`, `Mod`). The compiler has no list of its own.
pub fn names() -> &'static [(Vec<ast::Name>, ast::Name)] {
    static NAMES: OnceLock<Vec<(Vec<ast::Name>, ast::Name)>> = OnceLock::new();
    NAMES.get_or_init(|| {
        // `CPL_STD` may hold a spec prelude (spec/README.md); it wins like any module.
        let spec = std::env::var_os("CPL_STD")
            .and_then(|dir| {
                std::fs::read_to_string(std::path::Path::new(&dir).join("prelude.cp")).ok()
            })
            .map(|text| &*Box::leak(text.into_boxed_str()));
        let text = spec
            .or_else(|| crate::modules::bundled_std(&["prelude".to_string()]))
            .expect("stdlib/prelude.cp is built into the compiler");
        let mut found = Vec::new();
        for (_, item) in cpl_syntax::parse_file(text).items {
            for stmt in item.stmts {
                if let ast::StmtKind::Use { path, names, .. } = stmt.kind {
                    // The spans are of the prelude's text, not the file's.
                    let bare = |name: &ast::Name| ast::Name {
                        text: name.text.clone(),
                        span: cpl_diag::Span::new(0, 0),
                    };
                    let module: Vec<ast::Name> = path.iter().map(bare).collect();
                    found.extend(names.iter().map(|name| (module.clone(), bare(name))));
                }
            }
        }
        found
    })
}

/// Words of the language over a walk (`argmax(f(v) for v in V)`) that,
/// given a whole list or array, are the library's method of that name:
/// `argmax(a)` is `a.argmax`. Writing the word brings the method in.
pub const SEQUENCE_FORMS: [&str; 2] = ["argmax", "argmin"];

/// The name a method of the standard library is imported under when the
/// file declares that name itself: `find of std`. The space keeps it out
/// of reach of what a program can write.
pub fn std_method(name: &str) -> String {
    format!("{name} of std")
}

/// Every fn, method and getter name the active standard library declares.
/// A struct's fields are not members of the built-in types (R02.6.4). Under `CPL_SPEC`, a member of a built-in
/// type that the checker can't type and no module declares is a spec gap.
pub fn spec_member_names() -> &'static std::collections::HashSet<String> {
    static NAMES: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let ident = |s: &str| -> String {
            s.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect()
        };
        let mut names = std::collections::HashSet::new();
        for (_, text) in crate::modules::active_std_modules() {
            for line in text.lines() {
                let code = line.split('#').next().unwrap_or("").trim_start();
                if code.starts_with("///") {
                    continue;
                }
                if let Some(at) = code.find("fn ") {
                    names.insert(ident(&code[at + 3..]));
                }
            }
        }
        names
    })
}

/// The names of the top-level `fn name(self: S, ..)` methods: all a value
/// of no declared struct, such as a generator, can be given.
pub fn spec_receiver_method_names() -> &'static std::collections::HashSet<String> {
    static NAMES: OnceLock<std::collections::HashSet<String>> = OnceLock::new();
    NAMES.get_or_init(|| {
        crate::modules::active_std_modules()
            .iter()
            .flat_map(|(_, text)| scan_top_level_methods(text))
            .map(str::to_string)
            .collect()
    })
}

/// The methods the standard library adds to the types of the language:
/// each `pub fn name(self: T, ..)` at the top of a module, with that
/// module. A program that writes `.name` gets the fn without a `use`,
/// as it gets the methods a type is declared with. A method name does
/// not stand in the way of a program's own names, so these are not on
/// the prelude's list. A name two modules give is left to a `use`.
pub fn methods() -> &'static [(Vec<ast::Name>, ast::Name)] {
    static METHODS: OnceLock<Vec<(Vec<ast::Name>, ast::Name)>> = OnceLock::new();
    METHODS.get_or_init(|| {
        let bare = |text: &str| ast::Name {
            text: text.to_string(),
            span: cpl_diag::Span::new(0, 0),
        };
        let mut found: Vec<(Vec<ast::Name>, ast::Name)> = Vec::new();
        for (module, text) in crate::modules::active_std_modules() {
            if *module == "prelude" || !text.contains("self:") {
                continue;
            }
            let path: Vec<ast::Name> = std::iter::once("std")
                .chain(module.split('/'))
                .map(bare)
                .collect();
            for name in scan_top_level_methods(text) {
                found.push((path.clone(), bare(name)));
            }
        }
        let twice = |name: &str| found.iter().filter(|(_, other)| other.text == name).count() > 1;
        let unique: Vec<_> = found
            .iter()
            .filter(|(_, name)| !twice(&name.text))
            .cloned()
            .collect();
        unique
    })
}

/// Scans top-level `[@dec ...] pub [@dec ...] fn <name>[<...>]([mut] self: ...)`
/// headers without parsing function bodies.
fn scan_top_level_methods(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        let rest = &text[offset..];
        // Top-level declarations start at column 0 with `pub` or `@decorator`.
        if rest.starts_with("pub ") || rest.starts_with('@') {
            let mut cur = rest;
            let mut is_pub = false;
            loop {
                cur = cur.trim_start_matches([' ', '\t']);
                if let Some(after_pub) = cur.strip_prefix("pub ") {
                    is_pub = true;
                    cur = after_pub;
                    continue;
                }
                if let Some(dec) = cur.strip_prefix('@') {
                    let dec_len = dec
                        .bytes()
                        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                        .count();
                    cur = &dec[dec_len..];
                    if cur.starts_with('(') {
                        if let Some(close) = cur.find(')') {
                            cur = &cur[close + 1..];
                        }
                    }
                    continue;
                }
                break;
            }
            if is_pub {
                if let Some(after_pub_fn) = cur.strip_prefix("fn ") {
                    let name_len = after_pub_fn
                        .bytes()
                        .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                        .count();
                    if name_len > 0 {
                        let name = &after_pub_fn[..name_len];
                        let mut tail = after_pub_fn[name_len..].trim_start();
                        if tail.starts_with('<') {
                            let mut depth = 0usize;
                            let mut end = 0;
                            for (idx, b) in tail.bytes().enumerate() {
                                match b {
                                    b'<' => depth += 1,
                                    b'>' => {
                                        depth -= 1;
                                        if depth == 0 {
                                            end = idx + 1;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if end > 0 {
                                tail = tail[end..].trim_start();
                            }
                        }
                        if let Some(mut params) = tail.strip_prefix('(') {
                            params = params.trim_start();
                            if let Some(after_mut) = params.strip_prefix("mut") {
                                if after_mut.starts_with(|c: char| c.is_whitespace()) {
                                    params = after_mut.trim_start();
                                }
                            }
                            if let Some(after_self) = params.strip_prefix("self") {
                                if after_self.trim_start().starts_with(':') {
                                    out.push(name);
                                }
                            }
                        }
                    }
                }
            }
        }
        match bytes[offset..].iter().position(|&b| b == b'\n') {
            Some(nl) => offset += nl + 1,
            None => break,
        }
    }
    out
}

/// The words of the language that are written with a library fn: a
/// program that uses the word gets the fn, as if it had named it.
/// `prefix(a)` dispatches by what `a` is (`[D]T`, `[R, C]T`, or `List<T>`).
pub const BUILT_ON: &[(&str, &str)] = &[
    ("prefix", "Prefix"),
    ("adjacency", "Bitset"),
    ("all", "Bitset"),
    ("without", "Bitset"),
];

/// Internal shape specializations brought in from a stdlib module when a
/// file uses the unified family name (`prefix`, `Prefix`, `Diff`), without
/// spending prelude slots or exposing internal names to users.
pub const INTERNAL_BUILT_ON: &[(&str, &str, &str)] = &[
    ("prefix", "prefix", "_Prefix2"),
    ("prefix", "prefix", "_PrefixList"),
    ("prefix", "prefix", "FillPrefix"),
    ("prefix", "prefix", "_FillPrefix2"),
    ("prefix", "prefix", "_FillPrefixList"),
    ("prefix", "prefix", "FillOrder"),
    ("Prefix", "prefix", "_Prefix2"),
    ("Prefix", "prefix", "_PrefixList"),
    ("Prefix", "prefix", "FillPrefix"),
    ("Prefix", "prefix", "_FillPrefix2"),
    ("Prefix", "prefix", "_FillPrefixList"),
    ("Prefix", "prefix", "FillOrder"),
    ("FillPrefix", "prefix", "_FillPrefix2"),
    ("FillPrefix", "prefix", "_FillPrefixList"),
    ("FillPrefix", "prefix", "FillOrder"),
    ("Diff", "prefix", "_Diff2"),
];

fn int_fn(arity: usize) -> Ty {
    Ty::Fn {
        params: (0..arity)
            .map(|_| Param {
                name: String::new(),
                ty: Ty::Int,
                mutable: false,
                has_default: false,
                named_only: false,
                variadic: false,
                lazy: false,
            })
            .collect(),
        ret: Box::new(Ty::Int),
    }
}

/// The type of a prelude value, or `None` if `name` is not one. Fns that
/// are generic over what they are given are `Unknown`, which accepts any
/// call. `VALUE_NAMES` is the one list of them.
pub fn value(name: &str) -> Option<Ty> {
    Some(match name {
        "INF" => Ty::Int,
        "bit_ceil" | "isqrt" => int_fn(1),
        // `ask("?", x)` prints a query, flushes, and reads the judge's
        // answer; `Subset<D>` is the subsets of a domain. Both are lowered
        // by `cpl-ir`.
        "ask" | "Subset" => Ty::Unknown,
        _ if TYPE_NAMES.contains(&name) => Ty::TypeName(name.into()),
        _ if VALUE_NAMES.contains(&name) => Ty::Unknown,
        _ => return None,
    })
}

/// The doc of a prelude value the compiler defines, for a call with `args`
/// arguments or, with `None`, the value itself: `min(xs)` walks `xs` where
/// `min(a, b)` compares two values. `fixed` says the checker found the
/// values a `print` or `read` handles are of a fixed size.
pub fn value_doc(name: &str, args: Option<usize>, fixed: bool) -> Option<&'static str> {
    Some(match (name, args) {
        ("print", _) if fixed => "Writes its arguments and a newline, in O(1) for numbers, characters and tuples of them.",
        ("print", _) => "Writes its arguments and a newline, lists space-separated and grids by row, in time linear in what it writes.",
        ("read", _) if fixed => "Reads the next value of the type it is bound to, in O(1) for a number, a character or a tuple of them.",
        ("read", _) => "Reads the next value of the type it is bound to, in time linear in what it reads.",
        ("ask", _) if fixed => "Writes a query of its arguments, flushes and reads the judge's reply, in O(1) for numbers, characters and tuples of them, not counting the judge's own time.",
        ("ask", _) => "Writes a query of its arguments, flushes and reads the judge's reply, in time linear in what it writes and reads, not counting the judge's own time.",
        ("Int" | "I128" | "U64" | "F64" | "Char" | "Bool", Some(1)) if fixed => "Converts a number, character or `Bool` to this type, in O(1).",
        ("rand", Some(0 | 2)) => "A random `Int`, from `lo` to `hi` when given, in O(1).",
        ("abs", _) => "The absolute value, in O(1).",
        ("min", Some(1)) => "The smallest element of a list, an array, a slice `xs[a..b]` or a generator, in O(n) for n elements.",
        ("max", Some(1)) => "The largest element of a list, an array, a slice `xs[a..b]` or a generator, in O(n) for n elements.",
        ("min", _) => "The smallest of its arguments, in O(1) per argument; `min(xs)` gives the smallest element of `xs`.",
        ("max", _) => "The largest of its arguments, in O(1) per argument; `max(xs)` gives the largest element of `xs`.",
        ("minmax", Some(2) | None) => "`(min(a, b), max(a, b))`, in O(1).",
        ("choose2", _) => "`n * (n - 1) / 2` for `n >= 2`, else `0`. O(1).",
        ("popcount", _) => "The number of 1 bits, in O(1).",
        ("ctz", _) => "The number of 0 bits below the lowest 1 bit, in O(1).",
        ("bit_width", _) => "The number of bits needed to write the value, in O(1).",
        ("bit_ceil", _) => "The smallest power of two that is at least the value, in O(1).",
        ("isqrt", _) => "The largest `r` with `r * r <= n`, in O(1).",
        ("swap", _) => "Exchanges the values of two `mut` places, in O(1) whatever their size.",
        ("some", _) => "The value as a present `?T`, in O(1).",
        _ => return None,
    })
}

/// A prelude type by name.
pub fn ty(name: &str) -> Option<Ty> {
    Some(match name {
        "Int" => Ty::Int,
        "F64" => Ty::Float,
        "Bool" => Ty::Bool,
        "Char" => Ty::Char,
        "Str" => Ty::Str,
        "I128" => Ty::I128,
        "U64" => Ty::U64,
        "List" | "Map" | "HashMap" | "Set" | "HashSet" | "Iter" | "Seq" | "Indexed" | "Opt"
        | "Range" | "Cut" | "SortedRange" | "Sym" | "Counter" => Ty::Named {
            name: name.into(),
            args: Vec::new(),
        },
        // The solution as an `@interactor` sees it: `sol.recv()` reads what
        // it printed, `sol.send(x)` answers it.
        "Interactor" => Ty::Unknown,
        // The subsets of a domain, lowered by `cpl-ir` as a bitmask.
        "Subset" => Ty::Unknown,
        _ => return None,
    })
}

/// A prelude trait: what `T: Ord` and `impl Add for V` may name.
pub fn is_trait(name: &str) -> bool {
    TRAIT_NAMES.contains(&name)
}

pub const TYPE_NAMES: &[&str] = &[
    "List",
    "Map",
    "Set",
    "Int",
    "I128",
    "U64",
    "Bool",
    "Char",
    "Str",
    "F64",
    "Iter",
    "Opt",
    "Range",
    "Cut",
    "Counter",
    "HashMap",
    "HashSet",
    "Interactor",
    "Subset",
    "Sym",
    "Seq",
    "Indexed",
];

/// The traits of `plan/19-index-and-lends.md` and `stdlib/README.md`.
pub const TRAIT_NAMES: &[&str] = &[
    "Index",
    "Seq",
    "Indexed",
    "IndexedMut",
    "Keyed",
    "Sized",
    "Bidir",
    "Contains",
    "Ord",
    "Eq",
    "Hash",
    "Show",
    "Read",
    "Gen",
    "Shrink",
    "Add",
    "Sub",
    "Mul",
    "Div",
    "Neg",
    "Rem",
    "Pow",
    "Zero",
    "One",
    "Bounded",
    "Num",
    "Copy",
    "Bits",
];

pub const VALUE_NAMES: &[&str] = &[
    "print",
    "read",
    "debug",
    "read_output",
    "min",
    "max",
    "minmax",
    "choose2",
    "adjacency",
    "abs",
    "sum",
    "prod",
    "count",
    "all",
    "any",
    "find",
    "first",
    "last",
    "argmax",
    "argmin",
    "sort",
    "sorted",
    "shuffle",
    "swap",
    "copy",
    "rand",
    "some",
    "laws_hold",
    "popcount",
    "ctz",
    "bit_width",
    "bit_ceil",
    "isqrt",
    "sqrt",
    "atan2",
    "flush",
    "floor",
    "trunc",
    "ceil",
    "round",
    "len",
    "counts",
    "prefix",
    "span",
    "take",
    "columns",
    "List",
    "Map",
    "Set",
    "HashSet",
    "Int",
    "I128",
    "U64",
    "Bool",
    "Char",
    "Str",
    "F64",
    "Iter",
    "Opt",
    "Range",
    "Cut",
];

/// The standard library module that declares the public name `name`, for
/// the hint on a name that was used without its `use`.
pub fn std_module(name: &str) -> Option<&'static str> {
    STD_NAMES
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .ok()
        .map(|at| STD_NAMES[at].1)
}

/// The edit that brings `name` in from `std.<module>`: into the braces of
/// the file's `use std.<module>.{..}` when it has one, else as a line after
/// its last top-level `use`, else above its first line of code.
pub fn import_edit(text: &str, module: &str, name: &str) -> (Span, String) {
    let head = format!("use std.{module}.{{");
    let mut after_uses = None;
    let mut first_code = None;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let end = at + line.len();
        let body = line.trim_end();
        if let Some(close) = body.strip_prefix(&head).and_then(|_| body.rfind('}')) {
            let inside = body[head.len()..close].trim();
            let edit = if inside.is_empty() {
                name.to_string()
            } else {
                format!(", {name}")
            };
            let close = (at + close) as u32;
            return (Span::point(close), edit);
        }
        if body.starts_with("use ") {
            after_uses = Some(end);
        } else if first_code.is_none()
            && !body.trim_start().is_empty()
            && !body.trim_start().starts_with("//")
        {
            first_code = Some(at);
        }
        at = end;
    }
    let line = format!("use std.{module}.{{{name}}}\n");
    match (after_uses, first_code) {
        (Some(end), _) if !text[..end].ends_with('\n') => {
            (Span::point(end as u32), format!("\n{line}"))
        }
        (Some(end), _) => (Span::point(end as u32), line),
        (None, Some(start)) => (Span::point(start as u32), line),
        (None, None) => (Span::point(text.len() as u32), line),
    }
}

/// Every public name of `stdlib/` and its module, sorted by name.
/// `build.rs` reads it off the files.
pub const STD_NAMES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_names.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_method_scan_matches_full_parse_on_bundled_stdlib() {
        for (module, text) in crate::modules::bundled_std_modules() {
            if *module == "prelude" || !text.contains("self:") {
                continue;
            }
            let mut parsed = Vec::new();
            for (_, item) in cpl_syntax::parse_file(text).items {
                for stmt in item.stmts {
                    if let ast::StmtKind::Fn(decl) = &stmt.kind {
                        let on_a_type = decl
                            .params
                            .first()
                            .is_some_and(|p| p.name.text == "self" && p.ty.is_some());
                        if stmt.public && on_a_type {
                            parsed.push(decl.name.text.clone());
                        }
                    }
                }
            }
            let scanned: Vec<String> = scan_top_level_methods(text)
                .into_iter()
                .map(str::to_string)
                .collect();
            assert_eq!(scanned, parsed, "method scan mismatch in std.{module}");
        }
    }
}
