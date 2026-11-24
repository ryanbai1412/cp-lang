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
        bounds: Vec<(String, Vec<String>)>,
        lends: bool,
        row_bounds: Vec<(String, Vec<(String, Ty)>)>,
        trait_bounds: Vec<(String, String)>,
        /// `@droppable`: its result may be dropped without W0101.
        droppable: bool,
    },
    Struct {
        generics: Vec<String>,
        defaults: Vec<Option<Ty>>,
        /// Fields in declaration order: also the constructor's parameters.
        fields: Vec<crate::ty::Param>,
        /// Field names declared with `var` (assignable after construction).
        var_fields: Vec<String>,
        methods: Vec<(String, Ty)>,
        /// The parameters of `fn new(..)` when the struct defines it:
        /// outside the struct's body, `S(..)` calls `new`.
        ctor: Option<Vec<crate::ty::Param>>,
        /// Methods marked `@get`: read like fields, without `()`.
        getters: Vec<String>,
        /// Methods that take `mut self`.
        mutators: Vec<String>,
        /// Methods that take `self`: `T.name` on the type itself doesn't reach them.
        receivers: Vec<String>,
        /// Domains and constants declared in the body: `Rooted<V>.Pre`.
        owned: Vec<(String, Ty)>,
        projections: Vec<Projection>,
        cached_fields: Vec<String>,
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
        /// The enum's type parameters, which `fields` may name.
        generics: Vec<String>,
    },
    Domain(DomainInfo),
    Alias {
        ty: Ty,
        /// `type Perm<V> = [V]V` names `V`; a use binds it.
        generics: Vec<String>,
    },
    Trait {
        /// `trait Seq: Index + Ord` lists `Index` and `Ord`.
        supers: Vec<String>,
        methods: Vec<TraitMethod>,
        fields: Vec<(String, Ty)>,
        /// `trait Both = Sh + Nm` (R08.2.6).
        alias: bool,
        /// The associated types it declares, `type Found` (R08.1.9), each
        /// with the traits its bound names (`type Arc: Target<V>`) and its
        /// default, if any (`type Item = Int`, decision 469).
        assoc: Vec<(String, Vec<String>, Option<Ty>)>,
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
        /// The type it is for, with the impl's generics as params.
        target: Ty,
        /// The associated types it chooses, `type Found = ?Int` (R08.1.9).
        assoc: Vec<(String, Ty)>,
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
    /// Lowest possible integer position of an integer domain, when known.
    pub min_pos: Option<i128>,
    /// Highest possible integer position of an integer domain, when known.
    pub max_pos: Option<i128>,
    /// Where the size comes from: `n <= 2e5`.
    pub bound: Option<String>,
    /// Factor types for product domains (`domain State = (V, K)` or `(r: R, s: K)`).
    pub factors: Vec<(Option<String>, Ty)>,
}

impl Def {
    /// The type of a variant of enum `of` used as a value.
    pub fn variant_value(of: &str, fields: &[Ty]) -> Ty {
        let (of, fields, generics) = (of.to_string(), fields.to_vec(), Vec::new());
        Def::Variant {
            of,
            fields,
            generics,
        }
        .value_ty("")
    }

    /// The payload a variant pattern binds against a scrutinee of type
    /// `ty`: `One(k)` against a `Boxed<Int>` binds `k: Int`; against a
    /// `Boxed` of unknown arguments, `k` is left open rather than the bare
    /// parameter (R02.7.1). Not a variant: no payload.
    pub fn payload_at(&self, ty: &Ty) -> Vec<Ty> {
        let Def::Variant {
            of,
            fields,
            generics,
        } = self
        else {
            return Vec::new();
        };
        let args = match ty {
            Ty::Named { name, args } if name == of => args.as_slice(),
            _ => &[],
        };
        let bindings = generics
            .iter()
            .enumerate()
            .map(|(i, g)| (g.clone(), args.get(i).cloned().unwrap_or(Ty::Unknown)))
            .collect();
        fields.iter().map(|f| f.subst(&bindings)).collect()
    }

    /// The type of the name used as a value.
    pub fn value_ty(&self, name: &str) -> Ty {
        match self {
            Def::Value { ty, .. } | Def::Fn { ty, .. } => ty.clone(),
            Def::Struct { .. } | Def::Enum { .. } | Def::Alias { .. } | Def::Trait { .. } => {
                Ty::TypeName(name.to_string())
            }
            Def::Impl { .. } => Ty::Error,
            Def::Variant { of, fields, .. } if fields.is_empty() => Ty::Named {
                name: of.clone(),
                args: Vec::new(),
            },
            Def::Variant { of, fields, .. } => Ty::Fn {
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
            Def::Fn {
                generics,
                bounds,
                lends,
                row_bounds,
                trait_bounds,
                droppable,
                ..
            } => Def::Fn {
                ty: Ty::Error,
                generics,
                bounds,
                lends,
                row_bounds,
                trait_bounds,
                droppable,
            },
            Def::Value {
                mutable,
                keyword,
                generics,
                settled,
                ..
            } if annotation_broken => Def::Value {
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

/// A `let` with no annotation whose value lost a piece mid-expression
/// (`read<[n - * 1]Int>()` parses as comparisons): its type is a guess, so
/// its name gives no further errors. A half-typed end (`let total = n +`)
/// or a broken call argument keeps the type of what is there.
pub(crate) fn guessed(stmt: &cpl_syntax::ast::Stmt) -> bool {
    use cpl_syntax::ast::{Expr, ExprKind, StmtKind};
    #[derive(Default)]
    struct Holes(Vec<cpl_diag::Span>, Vec<cpl_diag::Span>);
    impl cpl_syntax::visit::Visitor for Holes {
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Missing | ExprKind::Error => self.0.push(e.span),
                ExprKind::Call { args, .. } => self.1.extend(args.iter().map(|a| a.value.span)),
                _ => {}
            }
        }
    }
    let StmtKind::Let {
        ty: None,
        init: Some(e),
        ..
    } = &stmt.kind
    else {
        return false;
    };
    let mut holes = Holes::default();
    cpl_syntax::visit::walk_expr(e, &mut holes);
    let in_arg = |h: &cpl_diag::Span| holes.1.iter().any(|a| a.start <= h.start && h.end <= a.end);
    holes.0.iter().any(|h| h.end < e.span.end && !in_arg(h))
}
