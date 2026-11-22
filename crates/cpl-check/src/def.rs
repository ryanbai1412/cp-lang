//! What a top-level name means to the rest of the file.
//!
//! A `Def` holds exactly what other items can observe about a name: a
//! value's type, a fn's signature, a struct's fields. Bodies are not part
//! of it. That is what makes incremental checking cut off early: editing a
//! fn body leaves its `Def` equal, so nothing that calls it is re-checked.

use crate::ty::Ty;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Def {
    /// `let`, `var` or `const`.
    Value {
        ty: Ty,
        mutable: bool,
        keyword: &'static str,
        /// `const Sum<T> = ...`: one value per choice of `T`.
        generics: Vec<String>,
        /// `const FAST = true`: the `Bool` it is known to be when
        /// compiling, for `if const FAST:`.
        settled: Option<bool>,
    },
    Fn {
        ty: Ty,
        generics: Vec<String>,
    },
    Struct {
        generics: Vec<String>,
        defaults: Vec<Option<Ty>>,
        /// Fields in declaration order: also the constructor's parameters.
        fields: Vec<crate::ty::Param>,
        methods: Vec<(String, Ty)>,
        /// The parameters of `fn new(..)` when the struct defines it:
        /// outside the struct's body, `S(..)` calls `new`.
        ctor: Option<Vec<crate::ty::Param>>,
        /// Methods marked `@get`: read like fields, without `()`.
        getters: Vec<String>,
        /// Methods that take `mut self`.
        mutators: Vec<String>,
        /// Domains and constants declared in the body: `Rooted<V>.Pre`.
        owned: Vec<(String, Ty)>,
        projections: Vec<Projection>,
    },
    Enum {
        variants: Vec<(String, Vec<Ty>)>,
        methods: Vec<(String, Ty)>,
        /// `@tags("X", "O")` says how each variant is written in the
        /// input.
        tagged: bool,
    },
    /// A variant of the named enum, usable bare.
    Variant {
        of: String,
        fields: Vec<Ty>,
    },
    Domain(DomainInfo),
    Alias {
        ty: Ty,
    },
    Trait {
        /// `trait Seq: Index + Ord` lists `Index` and `Ord`.
        supers: Vec<String>,
        methods: Vec<TraitMethod>,
    },
    /// `impl Add for Pt` or `impl Pt`, under the name [`impl_name`] gives it.
    Impl {
        trait_args: Vec<Ty>,
        out_ty: Option<Ty>,
        methods: Vec<(String, Ty)>,
        getters: Vec<String>,
        mutators: Vec<String>,
        owned: Vec<(String, Ty)>,
        projections: Vec<Projection>,
    },
}

/// `fn at(self, i: V) lends mut Int`: a method that lends part of its
/// receiver.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Projection {
    pub name: String,
    /// `lends mut`: the caller may write through it.
    pub mutable: bool,
    /// The lent type as written, for messages.
    pub lent: String,
    /// Its `law disjoint(..) = ..` as source: when two of its lends are
    /// apart. Without one they always conflict.
    pub law: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TraitMethod {
    pub name: String,
    pub ty: Ty,
    /// Declared without a body, so every impl must write it.
    pub required: bool,
}

/// The name an impl is filed under, next to the fns and types of its file:
/// `impl Add for Pt`. An impl without a trait is `impl Pt`.
pub fn impl_name(trait_name: Option<&str>, target: &str) -> String {
    match trait_name {
        Some(trait_name) => format!("impl {trait_name} for {target}"),
        None => format!("impl {target}"),
    }
}

/// The type an impl name is for, and its trait if it has one.
pub fn impl_parts(name: &str) -> Option<(Option<&str>, &str)> {
    let rest = name.strip_prefix("impl ")?;
    Some(match rest.split_once(" for ") {
        Some((trait_name, target)) => (Some(trait_name), target),
        None => (None, rest),
    })
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DomainInfo {
    /// The definition as written: `1..=n`.
    pub text: String,
    /// The most positions the domain can have, when the bounds are known.
    pub max_size: Option<i128>,
    /// Where the size comes from: `n <= 2e5`.
    pub bound: Option<String>,
    /// Factor types for product domains (`domain State = (V, K)` or `(r: R, s: K)`).
    pub factors: Vec<(Option<String>, Ty)>,
}

impl Def {
    /// The type of the name used as a value.
    pub fn value_ty(&self, name: &str) -> Ty {
        match self {
            Def::Value { ty, .. } | Def::Fn { ty, .. } => ty.clone(),
            Def::Struct { .. } | Def::Enum { .. } | Def::Alias { .. } | Def::Trait { .. } => {
                Ty::TypeName(name.to_string())
            }
            Def::Impl { .. } => Ty::Error,
            Def::Variant { of, fields } if fields.is_empty() => Ty::Named {
                name: of.clone(),
                args: Vec::new(),
            },
            Def::Variant { of, fields } => Ty::Fn {
                params: fields
                    .iter()
                    .map(|ty| crate::ty::Param {
                        name: String::new(),
                        ty: ty.clone(),
                        mutable: false,
                        has_default: false,
                        named_only: false,
                        variadic: false,
                        lazy: false,
                    })
                    .collect(),
                ret: Box::new(Ty::Named {
                    name: of.clone(),
                    args: Vec::new(),
                }),
            },
            Def::Domain(_) => Ty::Domain(name.to_string()),
        }
    }

    pub fn fingerprint(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        // 0 and 1 are reserved for "no such name" and "prelude name".
        hasher.finish() | 2
    }
}

pub fn hash_str(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// The kind of a top-level declaration, for outlines.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DeclKind {
    Value,
    Fn,
    Struct,
    Enum,
    Variant,
    Domain,
    Alias,
    Trait,
    Impl,
}

impl Def {
    /// The def of a declaration whose first line did not parse. A
    /// function's signature is that line, so its type is unknown and calls
    /// to it report nothing. A value's type comes from its initializer,
    /// where a missing or broken piece is already `Ty::Error`, so it is kept
    /// for hover and completion, unless its broken part is the annotation
    /// (`let a: [V1..=n = read()`), which nothing downstream can trust.
    pub fn recovered(self, annotation_broken: bool) -> Def {
        match self {
            Def::Fn { generics, .. } => Def::Fn {
                ty: Ty::Error,
                generics,
            },
            Def::Value { mutable, keyword, generics, settled, .. } if annotation_broken => Def::Value {
                ty: Ty::Error,
                mutable,
                keyword,
                generics,
                settled,
            },
            other => other,
        }
    }
}
