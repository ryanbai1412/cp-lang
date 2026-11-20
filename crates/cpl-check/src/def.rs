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
    },
    Fn {
        ty: Ty,
        generics: Vec<String>,
    },
    Struct {
        generics: Vec<String>,
        /// Fields in declaration order: also the constructor's parameters.
        fields: Vec<crate::ty::Param>,
        methods: Vec<(String, Ty)>,
    },
    Enum {
        variants: Vec<(String, Vec<Ty>)>,
        methods: Vec<(String, Ty)>,
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
    Trait,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DomainInfo {
    /// The definition as written: `1..=n`.
    pub text: String,
    /// The most positions the domain can have, when the bounds are known.
    pub max_size: Option<i128>,
    /// Where the size comes from: `n <= 2e5`.
    pub bound: Option<String>,
}

impl Def {
    /// The type of the name used as a value.
    pub fn value_ty(&self, name: &str) -> Ty {
        match self {
            Def::Value { ty, .. } | Def::Fn { ty, .. } => ty.clone(),
            Def::Struct { .. } | Def::Enum { .. } | Def::Alias { .. } | Def::Trait => {
                Ty::TypeName(name.to_string())
            }
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
}
