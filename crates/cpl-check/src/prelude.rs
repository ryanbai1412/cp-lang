//! Names every file can use without a `use`: the prelude of
//! `plan/40-tooling.md` (`print`, `read`, `min`, `max`, `abs`, `List`,
//! `Map`, `Set`, `Int`, `Bool`, `Char`, `Str`, `INF`), the reductions
//! over sequences that `plan/12-domains-and-sequences.md` writes bare
//! (`sum`, `count`, `all`, `any`, `find`, `first`, `last`, `argmax`,
//! `argmin`), and the intrinsics the libraries are written on top of
//! (`rand`, `sort`, `swap`, `popcount`, ...; `stdlib/README.md` lists them).
//!
//! Everything else, `std` included, is cp source and resolves through `use`.

use crate::ty::{Param, Ty};

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
        "popcount" | "ctz" | "bit_width" | "bit_ceil" | "isqrt" => int_fn(1),
        "sqrt" | "atan2" | "floor" | "ceil" | "round" | "len" => Ty::Unknown,
        // `counts(xs)` is a `Counter`, `prefix(a)` the prefix sums of an
        // array, and `span(a, b)` the range between two positions in
        // either order (decisions §11).
        "counts" | "prefix" | "span" => Ty::Unknown,
        // `take(x)` moves a value out and leaves its default behind, and
        // `read(columns)` reads an array of records a field at a time
        // (decisions §13).
        "take" | "columns" => Ty::Unknown,
        _ if TYPE_NAMES.contains(&name) => Ty::TypeName(name.into()),
        _ if PROTOTYPE_NAMES.contains(&name) => Ty::Unknown,
        _ => return None,
    })
}

/// A prelude type by name.
pub fn ty(name: &str) -> Option<Ty> {
    Some(match name {
        "Int" => Ty::Int,
        "Float" => Ty::Float,
        "Bool" => Ty::Bool,
        "Char" => Ty::Char,
        "Str" => Ty::Str,
        // Fixed-width integers for bit-level work.
        "I32" | "U8" | "U64" | "I128" => Ty::Int,
        "List" | "Map" | "HashMap" | "Set" | "Iter" | "Opt" | "Range" | "Bounds" | "Counter" => {
            Ty::Named {
                name: name.into(),
                args: Vec::new(),
            }
        }
        // The solution as an `@interactor` sees it: `sol.recv()` reads what
        // it printed, `sol.send(x)` answers it.
        "Interactor" => Ty::Unknown,
        _ if PROTOTYPE_NAMES.contains(&name) => Ty::Unknown,
        _ => return None,
    })
}

/// Names the prototype compiler (`proto/cpc`) builds in, which the ports
/// in `proto/ports/` were written against before `std` existed as cp
/// source. They are accepted without a type so those ports still check;
/// each one goes away when the ports that use it import its `std` module.
const PROTOTYPE_NAMES: &[&str] = &[
    "gcd",
    "lcm",
    "runs",
    "distinct",
    "product",
    "compress",
    "inf",
    "ask",
    "dijkstra",
    "Rooted",
    "SegTree",
    "StrHash",
    "MinHeap",
    "HashMap",
    "AddMax",
    "WithMin",
    "Perm",
    "Subset",
    "Mod",
    "Graph",
    "Dsu",
    "AssignSum",
    "condense",
    "scc",
];

/// A prelude trait: what `T: Ord` and `impl Add for V` may name.
pub fn is_trait(name: &str) -> bool {
    TRAIT_NAMES.contains(&name)
}

const TYPE_NAMES: &[&str] = &[
    "List", "Map", "Set", "Int", "Bool", "Char", "Str", "Float", "Iter", "Opt", "Range", "Bounds",
    "Counter",
];

/// The traits of `plan/19-index-and-lends.md` and `stdlib/README.md`.
const TRAIT_NAMES: &[&str] = &[
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
    "INF",
    "print",
    "read",
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
    "floor",
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
    "Bool",
    "Char",
    "Str",
    "Float",
    "Iter",
    "Opt",
    "Range",
    "Bounds",
];
