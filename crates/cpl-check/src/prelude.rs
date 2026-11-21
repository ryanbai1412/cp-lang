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

use cpl_syntax::ast;

use crate::ty::{Param, Ty};

/// What `stdlib/prelude.cp` lists: each name, and the module its `use`
/// takes it from (`std.mod`, `Mod`). The compiler has no list of its own.
pub fn names() -> &'static [(Vec<ast::Name>, ast::Name)] {
    static NAMES: OnceLock<Vec<(Vec<ast::Name>, ast::Name)>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let text = crate::modules::bundled_std(&["prelude".to_string()])
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

/// The name a method of the standard library is imported under when the
/// file declares that name itself: `find of std`. The space keeps it out
/// of reach of what a program can write.
pub fn std_method(name: &str) -> String {
    format!("{name} of std")
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
        for (module, text) in crate::modules::bundled_std_modules() {
            // Only where the text may hold one: parsing every module costs.
            if *module == "prelude" || !text.contains("self:") {
                continue;
            }
            let path: Vec<ast::Name> = std::iter::once("std").chain(module.split('/')).map(bare).collect();
            for (_, item) in cpl_syntax::parse_file(text).items {
                for stmt in item.stmts {
                    let ast::StmtKind::Fn(decl) = &stmt.kind else {
                        continue;
                    };
                    let on_a_type = decl.params.first().is_some_and(|p| p.name.text == "self" && p.ty.is_some());
                    if stmt.public && on_a_type {
                        found.push((path.clone(), bare(&decl.name.text)));
                    }
                }
            }
        }
        let twice = |name: &str| found.iter().filter(|(_, other)| other.text == name).count() > 1;
        let unique: Vec<_> = found.iter().filter(|(_, name)| !twice(&name.text)).cloned().collect();
        unique
    })
}

/// The words of the language that are written with a library fn: a
/// program that uses the word gets the fn, as if it had named it.
/// `argmax(f(v) for v in V)` is `max_by` over the walk, and `prefix(a)` is
/// one of three fns by what `a` is.
pub const BUILT_ON: &[(&str, &str)] = &[
    ("argmax", "max_by"),
    ("argmin", "min_by"),
    ("prefix", "prefix_n"),
    ("prefix", "prefix1"),
    ("prefix", "prefix2"),
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
            })
            .collect(),
        ret: Box::new(Ty::Int),
    }
}

/// The type of a prelude value, or `None` if `name` is not one. Fns that
/// are generic over what they are given are `Unknown`, which accepts any
/// call.
pub fn value(name: &str) -> Option<Ty> {
    Some(match name {
        "INF" => Ty::Int,
        "print" | "read" | "min" | "max" | "abs" => Ty::Unknown,
        "sum" | "count" | "all" | "any" | "find" | "first" | "last" | "argmax" | "argmin" => {
            Ty::Unknown
        }
        "sort" | "sorted" | "shuffle" | "swap" | "copy" | "rand" | "some" | "laws_hold" => {
            Ty::Unknown
        }
        // These three count the bits of an `Int` or of a `U64`.
        "popcount" | "ctz" | "bit_width" => Ty::Unknown,
        "bit_ceil" | "isqrt" => int_fn(1),
        "flush" | "sqrt" | "atan2" | "floor" | "ceil" | "round" | "trunc" | "len" => Ty::Unknown,
        // `counts(xs)` is a `Counter`, `prefix(a)` the prefix sums of an
        // array, and `span(a, b)` the range between two positions in
        // either order (decisions §11).
        "counts" | "prefix" | "span" => Ty::Unknown,
        // `take(x)` moves a value out and leaves its default behind, and
        // `read(columns)` reads an array of records a field at a time
        // (decisions §13).
        "take" | "columns" => Ty::Unknown,
        // `ask("?", x)` prints a query, flushes, and reads the judge's
        // answer; `Subset<D>` is the subsets of a domain. Both are lowered
        // by `cpl-ir`.
        "ask" | "Subset" => Ty::Unknown,
        // `debug(i, best)` shows values on a local run and is no part of
        // the C++ a judge gets; `read_output()` is what the solution
        // printed, to a `@checker`.
        "debug" | "read_output" => Ty::Unknown,
        _ if TYPE_NAMES.contains(&name) => Ty::TypeName(name.into()),
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
        "List" | "Map" | "HashMap" | "Set" | "Iter" | "Opt" | "Range" | "Bounds" | "Counter" => {
            Ty::Named {
                name: name.into(),
                args: Vec::new(),
            }
        }
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
    "List", "Map", "Set", "Int", "I128", "U64", "Bool", "Char", "Str", "F64", "Iter", "Opt", "Range", "Bounds",
    "Counter", "HashMap", "Interactor", "Subset",
];

/// The traits of `plan/19-index-and-lends.md` and `stdlib/README.md`.
pub const TRAIT_NAMES: &[&str] = &[
    "Index",
    "Seq",
    "Indexed",
    "IndexedMut",
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
    "abs",
    "sum",
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
    "Bounds",
];

/// The standard library module that declares the public name `name`, for
/// the hint on a name that was used without its `use`.
pub fn std_module(name: &str) -> Option<&'static str> {
    STD_NAMES
        .binary_search_by(|(known, _)| (*known).cmp(name))
        .ok()
        .map(|at| STD_NAMES[at].1)
}

/// Every public name of `stdlib/` and its module, sorted by name.
/// `build.rs` reads it off the files.
pub const STD_NAMES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_names.rs"));
