//! Names every file can use without a `use`: the prelude of
//! `plan/40-tooling.md` (`print`, `read`, `min`, `max`, `abs`, `List`,
//! `Map`, `Set`, `Int`, `Bool`, `Char`, `Str`, `INF`) and the reductions
//! over sequences that `plan/12-domains-and-sequences.md` writes bare
//! (`sum`, `count`, `all`, `any`, `find`, `first`, `last`, `argmax`,
//! `argmin`).
//!
//! The standard library is not ported to cp yet, so the prelude is a table
//! here. Once `std` exists as cp source this file goes away and these names
//! resolve like any other import.

use crate::ty::Ty;

/// The type of a prelude value, or `None` if `name` is not one. Fns whose
/// types need generics the checker does not model yet are `Unknown`, which
/// accepts any call.
pub fn value(name: &str) -> Option<Ty> {
    Some(match name {
        "INF" => Ty::Int,
        "print" | "read" | "min" | "max" | "abs" => Ty::Unknown,
        "sum" | "count" | "all" | "any" | "find" | "first" | "last" | "argmax" | "argmin" => {
            Ty::Unknown
        }
        "List" | "Map" | "Set" | "Int" | "Bool" | "Char" | "Str" | "Float" => {
            Ty::TypeName(name.into())
        }
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
        "List" | "Map" | "Set" => Ty::Named {
            name: name.into(),
            args: Vec::new(),
        },
        _ => return None,
    })
}

pub const VALUE_NAMES: &[&str] = &[
    "INF", "print", "read", "min", "max", "abs", "sum", "count", "all", "any", "find", "first",
    "last", "argmax", "argmin", "List", "Map", "Set", "Int", "Bool", "Char", "Str", "Float",
];
