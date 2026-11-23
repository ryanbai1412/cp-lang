//! Types as the checker sees them.

use std::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Ty {
    Int,
    /// 128 bits, signed.
    I128,
    /// 64 bits, unsigned, wrapping.
    U64,
    Float,
    Bool,
    Char,
    Str,
    Unit,
    /// An integer known to lie in `lo..=hi`. A bound is `None` when it
    /// depends on input (`1..=n`); `text` is how the user wrote the type.
    Range {
        lo: Option<i128>,
        hi: Option<i128>,
        text: String,
    },
    /// A position in the named domain.
    Pos(String),
    /// The domain itself, as in `for v in V`.
    Domain(String),
    /// `[V]T`, `[Row, Col]T`.
    Array {
        index: Vec<Ty>,
        elem: Box<Ty>,
    },
    Optional(Box<Ty>),
    Tuple(Vec<Ty>),
    Record(Vec<(String, Ty)>),
    RowWithout(Vec<(String, Ty)>, Box<Ty>, Vec<String>),
    /// A struct, enum, type parameter or library type: `Road`, `List<Int>`.
    Named {
        name: String,
        args: Vec<Ty>,
    },
    /// The name of a struct or enum used as a value: its constructor.
    TypeName(String),
    Fn {
        params: Vec<Param>,
        ret: Box<Ty>,
    },
    /// A type parameter, inside the fn or struct that declares it: the `T`
    /// of `struct Fenwick<D, T>`. Each use of the generic thing replaces it
    /// with the type given there.
    Param(String),
    /// A literal value argument: the `7` of `Mod<7>` (R03.5.1).
    Const(i128),
    /// Not known to this checker. Compatible with everything, so missing
    /// knowledge never becomes an error.
    Unknown,
    /// The type of code that already has a diagnostic. Compatible with
    /// everything, so one root cause is reported once.
    Error,
}

/// What each type parameter stands for at one use.
pub type Bindings = crate::fx::HashMap<String, Ty>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
    pub has_default: bool,
    pub named_only: bool,
    pub variadic: bool,
    pub lazy: bool,
}

impl Ty {
    /// A number, character or tuple of them: what `print` and `read` handle
    /// in O(1). A position is not, since its label may be any value.
    pub fn is_fixed_size(&self) -> bool {
        match self {
            Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Bool | Ty::Char | Ty::Range { .. } => {
                true
            }
            Ty::Tuple(items) => items.iter().all(Ty::is_fixed_size),
            _ => false,
        }
    }

    pub fn list(elem: Ty) -> Ty {
        Ty::Named {
            name: "List".into(),
            args: vec![elem],
        }
    }

    /// Element type of a built-in `Seq` / `Indexed` sequence (`Str`, `List<T>`,
    /// 1D `[D]T`, `Range<I>`, domains `D`, and lent slice/reverse views which keep their base type).
    pub fn seq_elem(&self) -> Option<Ty> {
        match self {
            Ty::Str => Some(Ty::Char),
            Ty::Domain(d) => Some(Ty::Pos(d.clone())),
            Ty::Array { index, elem } if index.len() == 1 => Some((**elem).clone()),
            Ty::Named { name, args } if name == "List" => {
                Some(args.first().cloned().unwrap_or(Ty::Unknown))
            }
            Ty::Named { name, args }
                if matches!(
                    name.as_str(),
                    "Range" | "Seq" | "Iter" | "Gen" | "Filter" | "Zip"
                ) && args.len() == 1 =>
            {
                Some(args[0].clone())
            }
            Ty::Named { name, args } if name == "Indexed" && args.len() == 2 => {
                Some(args[1].clone())
            }
            _ => None,
        }
    }

    /// True when this type implements the built-in `Seq` trait.
    pub fn is_seq(&self) -> bool {
        self.seq_elem().is_some()
    }

    /// True when this type implements the built-in `Sized` trait (known O(1) `.len`).
    pub fn is_sized(&self) -> bool {
        match self {
            Ty::Named { name, .. }
                if matches!(name.as_str(), "Gen" | "Iter" | "Seq" | "Filter") =>
            {
                false
            }
            _ => self.seq_elem().is_some(),
        }
    }

    /// True when this type implements the built-in `Indexed` trait (O(1) random read).
    pub fn is_indexed(&self) -> bool {
        self.is_sized()
    }

    /// True when this type implements the built-in `Bidir` trait (`.last`, `.rev`).
    pub fn is_bidir(&self) -> bool {
        match self {
            Ty::Named { name, .. } if matches!(name.as_str(), "Gen" | "Iter" | "Seq") => false,
            _ => self.seq_elem().is_some(),
        }
    }

    /// True when this type implements the built-in `Keyed` trait (`.items`, `.argmax`, `.argmin`).
    pub fn is_keyed(&self) -> bool {
        match self {
            Ty::Named { name, .. } if matches!(name.as_str(), "Gen" | "Iter" | "Seq") => false,
            _ => self.seq_elem().is_some(),
        }
    }

    /// True when this type implements the built-in `IndexedMut` trait (O(1) random read/write).
    pub fn is_indexed_mut(&self) -> bool {
        match self {
            Ty::Str => true,
            Ty::Array { index, .. } if index.len() == 1 => true,
            Ty::Named { name, args } if name == "List" && args.len() == 1 => true,
            _ => false,
        }
    }

    /// True when nothing can be said about this type.
    pub fn is_vague(&self) -> bool {
        matches!(self, Ty::Unknown | Ty::Error)
    }

    /// True when the type, or any part of it, is not known.
    pub fn has_vague(&self) -> bool {
        match self {
            Ty::Unknown | Ty::Error => true,
            Ty::Array { index, elem } => index.iter().any(Ty::has_vague) || elem.has_vague(),
            Ty::Optional(t) => t.has_vague(),
            Ty::Tuple(ts) => ts.iter().any(Ty::has_vague),
            Ty::Record(fs) => fs.iter().any(|(_, t)| t.has_vague()),
            Ty::RowWithout(prefix, base, _) => {
                prefix.iter().any(|(_, t)| t.has_vague()) || base.has_vague()
            }
            Ty::Named { args, .. } => args.iter().any(Ty::has_vague),
            Ty::Fn { params, ret } => params.iter().any(|p| p.ty.has_vague()) || ret.has_vague(),
            _ => false,
        }
    }

    /// True when the type, or any part of it, comes from code that was
    /// already reported.
    pub fn has_error(&self) -> bool {
        match self {
            Ty::Error => true,
            Ty::Array { index, elem } => index.iter().any(Ty::has_error) || elem.has_error(),
            Ty::Optional(t) => t.has_error(),
            Ty::Tuple(ts) => ts.iter().any(Ty::has_error),
            Ty::Record(fs) => fs.iter().any(|(_, t)| t.has_error()),
            Ty::RowWithout(prefix, base, _) => {
                prefix.iter().any(|(_, t)| t.has_error()) || base.has_error()
            }
            Ty::Named { args, .. } => args.iter().any(Ty::has_error),
            Ty::Fn { params, ret } => params.iter().any(|p| p.ty.has_error()) || ret.has_error(),
            _ => false,
        }
    }

    /// True for `Int`, integer ranges, `I128` and `U64`.
    pub fn is_int(&self) -> bool {
        matches!(self, Ty::Int | Ty::I128 | Ty::U64 | Ty::Range { .. })
    }

    pub fn is_numeric(&self) -> bool {
        self.is_int() || *self == Ty::Float
    }

    /// The type with range information dropped: what arithmetic produces.
    pub fn widen(&self) -> Ty {
        match self {
            Ty::Range { .. } => Ty::Int,
            other => other.clone(),
        }
    }

    /// Can a value of type `self` be used where `want` is expected?
    ///
    /// Integer ranges are checked by the range analysis, not here, so any
    /// integer fits any integer type. Domains are strict: a `V` is not an
    /// `E` and not an `Int` (decision 2.11).
    pub fn fits(&self, want: &Ty) -> bool {
        self.fits_in(want, &mut |_| false)
    }

    /// `fits`, where `number(name)` says whether the struct `name` is a
    /// number an `Int` is made into: one with a `zero`, a `one` and a
    /// `new(x: Int)`, as `Mod<P>` is (decision 3.10).
    pub fn fits_in(&self, want: &Ty, number: &mut dyn FnMut(&str) -> bool) -> bool {
        match (self, want) {
            (a, b) if a == b => true,
            (a, b) if a.is_vague() || b.is_vague() => true,
            // What a type parameter can do is its bounds' business; the
            // code is checked again with real types at each use.
            (Ty::Param(_), _) | (_, Ty::Param(_)) => true,
            (a, b) if a.is_int() && b.is_int() => true,
            (a, Ty::Float) if a.is_int() => false,
            (a, Ty::Named { name, .. }) if a.is_int() && number(name) => true,
            (Ty::Optional(a), Ty::Optional(b)) => a.fits_in(b, number),
            (a, Ty::Optional(inner)) => a.fits_in(inner, number),
            (Ty::Named { name, args }, b) if name == "Slice" && args.len() == 1 => {
                args[0].fits_in(b, number)
            }
            (a, Ty::Named { name, args }) if name == "Slice" && args.len() == 1 => {
                a.fits_in(&args[0], number)
            }
            (
                Ty::Array {
                    index: ai,
                    elem: ae,
                },
                Ty::Array {
                    index: bi,
                    elem: be,
                },
            ) => {
                ai.len() == bi.len()
                    && ai.iter().zip(bi).all(|(a, b)| a.fits_in(b, number))
                    && ae.fits_in(be, number)
            }
            (Ty::Tuple(a), Ty::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.fits_in(b, number))
            }
            // A record with more fields fits where fewer are asked for:
            // edges `(u, v, w)` are edges `(u, v)`.
            (Ty::Record(a), Ty::Record(b)) => {
                let open_b = b.last().is_some_and(|(n, _)| n == "..");
                let b_fields = if open_b { &b[..b.len() - 1] } else { &b[..] };
                if !open_b {
                    a.len() == b_fields.len()
                        && a.iter()
                            .zip(b_fields)
                            .all(|((an, at), (bn, bt))| an == bn && at.fits_in(bt, number))
                } else {
                    b_fields.iter().all(|(bn, bt)| {
                        a.iter()
                            .find(|(an, _)| an == bn)
                            .is_some_and(|(_, at)| at.fits_in(bt, number))
                    })
                }
            }
            (Ty::RowWithout(pa, a, ra), Ty::RowWithout(pb, b, rb)) => {
                pa.len() == pb.len()
                    && pa
                        .iter()
                        .zip(pb)
                        .all(|((na, ta), (nb, tb))| na == nb && ta.fits_in(tb, number))
                    && ra == rb
                    && a.fits_in(b, number)
            }
            (Ty::Tuple(a), Ty::Record(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, (_, b))| a.fits_in(b, number))
            }
            // `xs: Seq<T>` takes anything a `for` walks to `T`s; `Indexed<D, T>`
            // anything indexed by `D` too.
            (a, Ty::Named { name, args }) if name == "Seq" && args.len() == 1 => {
                a.seq_elem().is_some_and(|e| e.fits_in(&args[0], number))
            }
            (a, Ty::Named { name, args }) if name == "Indexed" && args.len() == 2 => {
                a.is_indexed() && a.seq_elem().is_some_and(|e| e.fits_in(&args[1], number))
            }
            // A domain is the range, list or bitset of its positions:
            // `tally(Pos, Pos, 1)` or `zip(I, I.rev)`.
            (Ty::Domain(d), Ty::Named { name: bn, args: ba })
                if matches!(bn.as_str(), "Range" | "List" | "Bitset") =>
            {
                ba.first()
                    .is_none_or(|elem| Ty::Pos(d.clone()).fits_in(elem, number))
            }
            // A range written out is the list or bitset of what it goes over:
            // `place(D.all)` where a `List<D>` or `Bitset<D>` is wanted.
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba })
                if an == "Range"
                    && matches!(bn.as_str(), "List" | "Bitset")
                    && aa.len() == 1
                    && ba.len() == 1 =>
            {
                aa[0].fits_in(&ba[0], number)
            }
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba }) => {
                // `[k: v]` is written the same for either kind of map.
                (an == bn || (an == "Map" && bn == "HashMap"))
                    && (aa.is_empty()
                        || ba.is_empty()
                        || (aa.len() == ba.len()
                            && aa.iter().zip(ba).all(|(a, b)| a.fits_in(b, number))))
            }
            (
                Ty::Fn {
                    params: ap,
                    ret: ar,
                },
                Ty::Fn {
                    params: bp,
                    ret: br,
                },
            ) => {
                if ap.iter().any(|p| p.has_default || p.variadic)
                    || bp.iter().any(|p| p.has_default || p.variadic)
                {
                    true
                } else {
                    ap.len() == bp.len()
                        && ap.iter().zip(bp).all(|(a, b)| b.ty.fits_in(&a.ty, number))
                        && ar.fits_in(br, number)
                }
            }
            // `Cut<V>` is how library code spells `V.cuts`.
            (Ty::Pos(bounds), Ty::Named { name, args })
            | (Ty::Named { name, args }, Ty::Pos(bounds))
                if name == "Cut" =>
            {
                match args.as_slice() {
                    [Ty::Pos(domain)] => bounds.strip_suffix(".cuts") == Some(domain),
                    _ => bounds.ends_with(".cuts"),
                }
            }
            // The boundaries of a plain `0..n` index are plain `Int`s:
            // `prefix(a).prefix(k)` on a `[0..n]T`.
            (a, Ty::Named { name, args }) | (Ty::Named { name, args }, a)
                if name == "Cut"
                    && a.is_int()
                    && matches!(args.as_slice(), [arg] if arg.is_int()) =>
            {
                true
            }
            // A string is a list of its characters (decision 9: `Str` is
            // `[domain S]Char`, one flat byte array).
            (Ty::Str, Ty::Named { name, args }) if name == "List" => args
                .first()
                .is_none_or(|elem| Ty::Char.fits_in(elem, number)),
            (Ty::Named { name, args }, Ty::Str) if name == "List" => args
                .first()
                .is_none_or(|elem| elem.fits_in(&Ty::Char, number)),
            // An array is read where a list is asked for: `running_sum(a)`.
            (Ty::Array { elem, .. }, Ty::Named { name, args }) if name == "List" => {
                args.first().is_none_or(|want| elem.fits_in(want, number))
            }
            // A domain `D` is also the full range `D.first..=D.last` (`Range<D>`).
            (Ty::Domain(d), Ty::Named { name, args }) if name == "Range" => {
                matches!(args.as_slice(), [Ty::Pos(p)] if p == d)
            }
            // A record or tuple literal builds a struct with those fields;
            // the fields are checked where the struct is known.
            (Ty::Record(_) | Ty::Tuple(_), Ty::Named { .. }) => true,
            _ => false,
        }
    }

    /// Rebuilds the type with `leaf` applied to every type inside it.
    fn map(&self, leaf: &mut dyn FnMut(&Ty) -> Option<Ty>) -> Ty {
        if let Some(replaced) = leaf(self) {
            return replaced;
        }
        match self {
            Ty::Array { index, elem } => Ty::Array {
                index: index.iter().map(|t| t.map(leaf)).collect(),
                elem: Box::new(elem.map(leaf)),
            },
            Ty::Optional(inner) => Ty::Optional(Box::new(inner.map(leaf))),
            Ty::Tuple(items) => Ty::Tuple(items.iter().map(|t| t.map(leaf)).collect()),
            Ty::Record(fields) => Ty::Record(
                fields
                    .iter()
                    .map(|(name, t)| (name.clone(), t.map(leaf)))
                    .collect(),
            ),
            Ty::RowWithout(prefix, base, removed) => {
                let mut out: Vec<(String, Ty)> = prefix
                    .iter()
                    .map(|(n, t)| (n.clone(), t.map(leaf)))
                    .collect();
                match base.map(leaf) {
                    Ty::Record(fs) => {
                        out.extend(
                            fs.into_iter()
                                .filter(|(n, _)| !removed.iter().any(|r| r == n)),
                        );
                        Ty::Record(out)
                    }
                    other => Ty::RowWithout(out, Box::new(other), removed.clone()),
                }
            }
            Ty::Named { name, args } => Ty::Named {
                name: name.clone(),
                args: args.iter().map(|t| t.map(leaf)).collect(),
            },
            Ty::Fn { params, ret } => Ty::Fn {
                params: params
                    .iter()
                    .map(|p| Param {
                        ty: p.ty.map(leaf),
                        ..p.clone()
                    })
                    .collect(),
                ret: Box::new(ret.map(leaf)),
            },
            other => other.clone(),
        }
    }

    /// Replaces every mention of domain `from` (and derived domains like
    /// `from.cuts`) with `to`.
    pub fn rename_domain(&self, from: &str, to: &str) -> Ty {
        if from == to {
            return self.clone();
        }
        let rename_name = |name: &str| -> Option<String> {
            if name == from {
                Some(to.to_string())
            } else {
                name.strip_prefix(from)
                    .and_then(|r| r.strip_prefix('.'))
                    .map(|rest| format!("{to}.{rest}"))
            }
        };
        self.map(&mut |ty| match ty {
            Ty::Pos(name) => rename_name(name).map(Ty::Pos),
            Ty::Domain(name) => rename_name(name).map(Ty::Domain),
            _ => None,
        })
    }

    /// Base domain names mentioned anywhere in this type (stripping `.cuts`/`.subsets`).
    pub fn mentioned_domains(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.collect_domains(&mut out);
        out
    }

    fn collect_domains(&self, out: &mut Vec<String>) {
        let mut push = |name: &str| {
            let base = name.split('.').next().unwrap_or(name);
            if !base.is_empty() && !out.iter().any(|existing| existing == base) {
                out.push(base.to_string());
            }
        };
        match self {
            Ty::Pos(name) | Ty::Domain(name) => push(name),
            Ty::Array { index, elem } => {
                for idx in index {
                    idx.collect_domains(out);
                }
                elem.collect_domains(out);
            }
            Ty::Optional(inner) => inner.collect_domains(out),
            Ty::Tuple(items) => {
                for item in items {
                    item.collect_domains(out);
                }
            }
            Ty::Record(fields) => {
                for (_, ty) in fields {
                    ty.collect_domains(out);
                }
            }
            Ty::RowWithout(prefix, base, _) => {
                for (_, t) in prefix {
                    t.collect_domains(out);
                }
                base.collect_domains(out);
            }
            Ty::Named { args, .. } => {
                for arg in args {
                    arg.collect_domains(out);
                }
            }
            Ty::Fn { params, ret } => {
                for p in params {
                    p.ty.collect_domains(out);
                }
                ret.collect_domains(out);
            }
            _ => {}
        }
    }

    /// Existential domain names introduced by a return type such as
    /// `(domain P, [P]Int)` or `(domain C, comp: [V]C)`.
    pub fn existential_domains(&self) -> Vec<String> {
        match self {
            Ty::Domain(name) => vec![name.clone()],
            Ty::Tuple(items) => items
                .iter()
                .filter_map(|t| match t {
                    Ty::Domain(name) => Some(name.clone()),
                    _ => None,
                })
                .collect(),
            Ty::Record(fields) => fields
                .iter()
                .filter_map(|(_, t)| match t {
                    Ty::Domain(name) => Some(name.clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// The type with each type parameter in `bindings` replaced by its
    /// type: `[D]T` with `D = V, T = Int` is `[V]Int`.
    pub fn subst(&self, bindings: &Bindings) -> Ty {
        if bindings.is_empty() {
            return self.clone();
        }
        let list_index = Ty::list_index();
        let listed = bindings.values().any(|ty| *ty == list_index);
        let bound = self.map(&mut |ty| match ty {
            Ty::Param(name) => bindings.get(name).cloned(),
            _ => None,
        });
        if !listed {
            return bound;
        }
        // `[D]T` where a list was given is that list's type, and one of
        // its positions is an `Int`.
        bound.map(&mut |ty| match ty {
            Ty::Array { index, elem } if *index == [Ty::list_index()] => Some(Ty::Named {
                name: "List".to_string(),
                args: vec![elem.subst(bindings)],
            }),
            ty if *ty == list_index => Some(Ty::Int),
            _ => None,
        })
    }

    /// What a `D` is when a `List<T>` is given where a `[D]T` is asked
    /// for: the list's own positions, `0..len`.
    fn list_index() -> Ty {
        Ty::Named {
            name: "List".to_string(),
            args: Vec::new(),
        }
    }

    /// The type as code outside the generic sees it when nothing says what
    /// a parameter is: every parameter left is unknown.
    pub fn erase(&self) -> Ty {
        self.map(&mut |ty| matches!(ty, Ty::Param(_)).then_some(Ty::Unknown))
    }

    /// Like [`Ty::erase`], but the parameters in `keep` stay: they belong
    /// to the code the type is used in.
    pub fn settle(&self, keep: &[String]) -> Ty {
        self.map(&mut |ty| match ty {
            Ty::Param(name) if !keep.contains(name) => Some(Ty::Unknown),
            _ => None,
        })
    }

    /// Reads type parameters off a use: matching the declared `[D]T`
    /// against a `[V]Int` binds `D = V` and `T = Int`. The first type seen
    /// for a parameter stays.
    pub fn unify(&self, got: &Ty, bindings: &mut Bindings) {
        match (self, got) {
            (_, Ty::Unknown | Ty::Error) => {}
            (Ty::Param(name), got) => {
                bindings.entry(name.clone()).or_insert_with(|| got.widen());
            }
            (
                Ty::Array { index, elem },
                Ty::Array {
                    index: got_index,
                    elem: got_elem,
                },
            ) => {
                for (want, got) in index.iter().zip(got_index) {
                    want.unify(got, bindings);
                }
                elem.unify(got_elem, bindings);
            }
            (Ty::Optional(want), Ty::Optional(got)) => want.unify(got, bindings),
            (Ty::Optional(want), got) => want.unify(got, bindings),
            (Ty::Tuple(want), Ty::Tuple(got)) => {
                for (want, got) in want.iter().zip(got) {
                    want.unify(got, bindings);
                }
            }
            (Ty::Record(want), Ty::Record(got)) => {
                for (wname, wty) in want {
                    if let Some((_, gty)) = got.iter().find(|(gname, _)| gname == wname) {
                        wty.unify(gty, bindings);
                    }
                }
                for ((_, wty), (_, gty)) in want.iter().zip(got) {
                    wty.unify(gty, bindings);
                }
            }
            (Ty::Record(want), Ty::Tuple(got)) => {
                for ((_, wty), gty) in want.iter().zip(got) {
                    wty.unify(gty, bindings);
                }
            }
            (Ty::Tuple(want), Ty::Record(got)) => {
                for (wty, (_, gty)) in want.iter().zip(got) {
                    wty.unify(gty, bindings);
                }
            }
            (
                Ty::Named { name, args },
                Ty::Named {
                    name: got,
                    args: got_args,
                },
            ) if name == got => {
                for (want, got) in args.iter().zip(got_args) {
                    want.unify(got, bindings);
                }
            }
            // A list is an array over its own positions.
            (
                Ty::Array { index, elem },
                Ty::Named {
                    name,
                    args: got_args,
                },
            ) if name == "List"
                && got_args.len() == 1
                && matches!(index.as_slice(), [Ty::Param(_)]) =>
            {
                index[0].unify(&Ty::list_index(), bindings);
                elem.unify(&got_args[0], bindings);
            }
            (Ty::Array { index, elem }, Ty::Str) if matches!(index.as_slice(), [Ty::Param(_)]) => {
                index[0].unify(&Ty::list_index(), bindings);
                elem.unify(&Ty::Char, bindings);
            }
            (Ty::Named { name, args }, Ty::Str) if name == "List" && args.len() == 1 => {
                args[0].unify(&Ty::Char, bindings);
            }
            (
                Ty::Fn { params, ret },
                Ty::Fn {
                    params: got_params,
                    ret: got_ret,
                },
            ) => {
                for (want, got) in params.iter().zip(got_params) {
                    want.ty.unify(&got.ty, bindings);
                }
                ret.unify(got_ret, bindings);
            }
            _ => {}
        }
    }

    /// "a V", "an E", "an Int": the type with its article, for messages.
    pub fn with_article(&self) -> String {
        let text = self.to_string();
        // "an F64": the name is said "eff".
        let vowel = text == "F64"
            || text
                .trim_start_matches(['[', '?', '('])
                .starts_with(['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u']);
        format!("{} {text}", if vowel { "an" } else { "a" })
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let list = |f: &mut fmt::Formatter<'_>, items: &[Ty]| -> fmt::Result {
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                write!(f, "{item}")?;
            }
            Ok(())
        };
        match self {
            Ty::Int => f.write_str("Int"),
            Ty::I128 => f.write_str("I128"),
            Ty::U64 => f.write_str("U64"),
            Ty::Float => f.write_str("F64"),
            Ty::Bool => f.write_str("Bool"),
            Ty::Char => f.write_str("Char"),
            Ty::Str => f.write_str("Str"),
            Ty::Unit => f.write_str("()"),
            Ty::Range { text, .. } => f.write_str(text),
            Ty::Pos(name) | Ty::TypeName(name) | Ty::Param(name) => f.write_str(name),
            Ty::Domain(name) => write!(f, "domain {name}"),
            Ty::Array { index, elem } => {
                f.write_str("[")?;
                list(f, index)?;
                write!(f, "]{elem}")
            }
            Ty::Optional(inner) => write!(f, "?{inner}"),
            Ty::Tuple(items) => {
                f.write_str("(")?;
                list(f, items)?;
                f.write_str(")")
            }
            Ty::Record(fields) => {
                f.write_str("(")?;
                for (i, (name, ty)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{name}: {ty}")?;
                }
                f.write_str(")")
            }
            Ty::RowWithout(prefix, base, removed) => {
                f.write_str("(")?;
                for (n, t) in prefix {
                    write!(f, "{n}: {t}, ")?;
                }
                write!(f, "..{base} without ({}))", removed.join(", "))
            }
            Ty::Named { name, args } => {
                let shown = match name.as_str() {
                    "_Prefix2" | "_PrefixList" => "Prefix",
                    "_FillPrefix2" | "_FillPrefixList" => "FillPrefix",
                    "_Diff2" => "Diff",
                    other => other,
                };
                f.write_str(shown)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    list(f, args)?;
                    f.write_str(">")?;
                }
                Ok(())
            }
            Ty::Fn { params, ret } => {
                f.write_str("fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    if p.variadic {
                        f.write_str("..")?;
                    }
                    if p.mutable {
                        f.write_str("mut ")?;
                    }
                    if p.name.is_empty() {
                        write!(f, "{}", p.ty)?;
                    } else {
                        write!(f, "{}: {}", p.name, p.ty)?;
                    }
                }
                f.write_str(")")?;
                if **ret != Ty::Unit {
                    write!(f, " -> {ret}")?;
                }
                Ok(())
            }
            Ty::Const(v) => write!(f, "{v}"),
            Ty::Unknown => f.write_str("_"),
            Ty::Error => f.write_str("{error}"),
        }
    }
}

/// Storage for one value of a type, in bytes, following
/// `plan/11-values-and-types.md`: a range takes the narrowest width that
/// holds it, and `?T` uses a spare value of `T` when there is one.
///
/// `domain_size` gives the most positions a domain can have, and
/// `named_size` the size of a struct.
pub fn size_of(
    ty: &Ty,
    domain_size: &dyn Fn(&str) -> Option<i128>,
    named_size: &dyn Fn(&str) -> Option<u64>,
) -> Option<u64> {
    Some(match ty {
        Ty::Int | Ty::U64 | Ty::Float => 8,
        Ty::I128 => 16,
        Ty::Bool | Ty::Char => 1,
        Ty::Unit => 0,
        Ty::Range { lo, hi, .. } => match (lo, hi) {
            (Some(lo), Some(hi)) => int_width(*lo, *hi),
            _ => 8,
        },
        // Positions are 32-bit unless the domain is known to be small.
        Ty::Pos(name) => match domain_size(name) {
            Some(n) if n < 1 << 8 => 1,
            Some(n) if n < 1 << 16 => 2,
            _ => 4,
        },
        Ty::Optional(inner) => {
            let inner_size = size_of(inner, domain_size, named_size)?;
            if has_niche(inner, inner_size) {
                inner_size
            } else {
                inner_size + inner_size.clamp(1, 8)
            }
        }
        Ty::Tuple(items) => struct_size(items.iter().map(|t| size_of(t, domain_size, named_size)))?,
        Ty::Record(fields) => struct_size(
            fields
                .iter()
                .map(|(_, t)| size_of(t, domain_size, named_size)),
        )?,
        Ty::Named { name, args } if args.is_empty() => named_size(name)?,
        _ => return None,
    })
}

/// Size of fields laid out in order with natural alignment.
pub fn struct_size(fields: impl Iterator<Item = Option<u64>>) -> Option<u64> {
    let sizes: Option<Vec<u64>> = fields.collect();
    Some(struct_layout(&sizes?).size)
}

/// Where each field of a struct, tuple or record sits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub offsets: Vec<u64>,
    pub size: u64,
    pub align: u64,
}

impl Layout {
    /// Unused bytes before field `i`, or after the last field when `i` is
    /// the field count.
    pub fn padding_before(&self, i: usize, sizes: &[u64]) -> u64 {
        let end = |j: usize| {
            if j == 0 {
                0
            } else {
                self.offsets[j - 1] + sizes[j - 1]
            }
        };
        match self.offsets.get(i) {
            Some(&at) => at - end(i),
            None => self.size - end(i),
        }
    }
}

pub fn struct_layout(sizes: &[u64]) -> Layout {
    let mut size = 0u64;
    let mut align = 1u64;
    let mut offsets = Vec::with_capacity(sizes.len());
    for &field in sizes {
        let a = field.clamp(1, 8).next_power_of_two().min(8);
        align = align.max(a);
        size = size.div_ceil(a) * a;
        offsets.push(size);
        size += field;
    }
    Layout {
        offsets,
        size: size.div_ceil(align) * align,
        align,
    }
}

/// Whether `?T` stores `None` in a value `T` never takes, so it needs no
/// flag and costs what `T` costs.
pub fn has_niche(inner: &Ty, inner_size: u64) -> bool {
    match inner {
        Ty::Int | Ty::Pos(_) => true,
        Ty::Range {
            lo: Some(lo),
            hi: Some(hi),
            ..
        } => inner_size < 16 && hi - lo + 1 < 1i128 << (8 * inner_size),
        _ => false,
    }
}

fn int_width(lo: i128, hi: i128) -> u64 {
    let fits = |bits: u32| {
        if lo >= 0 {
            hi < 1i128 << bits
        } else {
            lo >= -(1i128 << (bits - 1)) && hi < 1i128 << (bits - 1)
        }
    };
    if fits(8) {
        1
    } else if fits(16) {
        2
    } else if fits(32) {
        4
    } else {
        8
    }
}

/// `1.6 MB`, `400 KB`, `12 bytes`.
pub fn human_bytes(bytes: u128) -> String {
    let unit = |n: u128, div: u128, name: &str| {
        let tenths = n * 10 / div;
        if tenths.is_multiple_of(10) || tenths >= 1000 {
            format!("{} {name}", tenths / 10)
        } else {
            format!("{}.{} {name}", tenths / 10, tenths % 10)
        }
    };
    match bytes {
        b if b >= 1_000_000_000 => unit(b, 1_000_000_000, "GB"),
        b if b >= 1_000_000 => unit(b, 1_000_000, "MB"),
        b if b >= 1_000 => unit(b, 1_000, "KB"),
        1 => "1 byte".into(),
        b => format!("{b} bytes"),
    }
}

/// `2e5`, `1e9`, `2000`: a bound the way the user would write it.
pub fn human_int(n: i128) -> String {
    if n.abs() >= 10_000 {
        let mut mantissa = n;
        let mut exp = 0;
        while mantissa % 10 == 0 {
            mantissa /= 10;
            exp += 1;
        }
        if mantissa.abs() < 10 && exp >= 4 {
            return format!("{mantissa}e{exp}");
        }
    }
    n.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(lo: i128, hi: i128) -> Ty {
        Ty::Range {
            lo: Some(lo),
            hi: Some(hi),
            text: format!("{lo}..={hi}"),
        }
    }

    fn no_domains(_: &str) -> Option<i128> {
        None
    }

    fn no_structs(_: &str) -> Option<u64> {
        None
    }

    #[test]
    fn sizes_follow_the_values_and_types_plan() {
        assert_eq!(size_of(&Ty::Int, &no_domains, &no_structs), Some(8));
        assert_eq!(
            size_of(&range(0, 1_000_000_000), &no_domains, &no_structs),
            Some(4)
        );
        assert_eq!(size_of(&range(1, 2000), &no_domains, &no_structs), Some(2));
        assert_eq!(size_of(&range(0, 255), &no_domains, &no_structs), Some(1));
        assert_eq!(size_of(&range(-1, 255), &no_domains, &no_structs), Some(2));
        // `?0..=1e9` is still 4 bytes: the spare values hold none.
        assert_eq!(
            size_of(
                &Ty::Optional(Box::new(range(0, 1_000_000_000))),
                &no_domains,
                &no_structs
            ),
            Some(4)
        );
        assert_eq!(
            size_of(&Ty::Optional(Box::new(Ty::Int)), &no_domains, &no_structs),
            Some(8)
        );
        assert_eq!(
            size_of(
                &Ty::Optional(Box::new(Ty::Pos("V".into()))),
                &no_domains,
                &no_structs
            ),
            Some(4)
        );
        assert_eq!(
            size_of(
                &Ty::Optional(Box::new(range(0, 255))),
                &no_domains,
                &no_structs
            ),
            Some(2)
        );
        assert_eq!(
            size_of(
                &Ty::Tuple(vec![Ty::Pos("V".into()), Ty::Int]),
                &no_domains,
                &no_structs
            ),
            Some(16)
        );
        assert_eq!(
            size_of(&Ty::Pos("V".into()), &|_| Some(200), &no_structs),
            Some(1)
        );
        assert_eq!(size_of(&Ty::list(Ty::Int), &no_domains, &no_structs), None);
    }

    #[test]
    fn domains_are_strict_and_integers_are_not() {
        let v = Ty::Pos("V".into());
        let e = Ty::Pos("E".into());
        assert!(v.fits(&v));
        assert!(!v.fits(&e));
        assert!(!v.fits(&Ty::Int));
        assert!(!Ty::Int.fits(&v));
        assert!(range(0, 5).fits(&Ty::Int));
        assert!(Ty::Int.fits(&range(0, 5)));
        assert!(!Ty::Int.fits(&Ty::Float));
        assert!(!Ty::Float.fits(&Ty::Int));
        assert!(v.fits(&Ty::Optional(Box::new(v.clone()))));
        assert!(Ty::Unknown.fits(&v) && v.fits(&Ty::Error));
    }

    #[test]
    fn types_print_the_way_they_are_written() {
        let dist = Ty::Array {
            index: vec![Ty::Pos("V".into())],
            elem: Box::new(Ty::Optional(Box::new(Ty::Int))),
        };
        assert_eq!(dist.to_string(), "[V]?Int");
        assert_eq!(
            Ty::list(Ty::Tuple(vec![Ty::Pos("V".into()), Ty::Int])).to_string(),
            "List<(V, Int)>"
        );
        assert_eq!(Ty::Pos("E".into()).with_article(), "an E");
        assert_eq!(Ty::Pos("V".into()).with_article(), "a V");
        assert_eq!(dist.with_article(), "a [V]?Int");
    }

    #[test]
    fn numbers_print_the_way_contest_limits_are_written() {
        assert_eq!(human_int(200_000), "2e5");
        assert_eq!(human_int(1_000_000_000), "1e9");
        assert_eq!(human_int(2000), "2000");
        assert_eq!(human_int(123_456), "123456");
        assert_eq!(human_bytes(1_600_000), "1.6 MB");
        assert_eq!(human_bytes(800_000), "800 KB");
        assert_eq!(human_bytes(12), "12 bytes");
    }

    #[test]
    fn unify_binds_caller_params_across_nested_type_constructors() {
        let p = |s: &str| Ty::Param(s.into());
        let arr = |i: Ty, e: Ty| Ty::Array {
            index: vec![i],
            elem: Box::new(e),
        };
        let opt = |t: Ty| Ty::Optional(Box::new(t));
        let tup = |xs: Vec<Ty>| Ty::Tuple(xs);
        let rec =
            |xs: Vec<(&str, Ty)>| Ty::Record(xs.into_iter().map(|(k, v)| (k.into(), v)).collect());
        let named = |n: &str, args: Vec<Ty>| Ty::Named {
            name: n.into(),
            args,
        };

        let templates = vec![
            arr(p("V"), opt(rec(vec![("from", p("V")), ("via", p("A"))]))),
            arr(p("V"), opt(tup(vec![p("V"), p("A")]))),
            opt(arr(p("V"), tup(vec![opt(p("A")), p("V")]))),
            named("Pair", vec![opt(p("V")), arr(p("V"), p("A"))]),
            rec(vec![("a", opt(p("A"))), ("b", arr(p("V"), opt(p("A"))))]),
            tup(vec![
                named("List", vec![opt(p("V"))]),
                rec(vec![("x", p("A"))]),
            ]),
        ];
        let renamings = [
            vec![("V", p("S")), ("A", p("M"))],
            vec![("V", p("V")), ("A", p("M"))],
            vec![("V", p("A")), ("A", p("V"))],
            vec![("V", Ty::Pos("Node".into())), ("A", p("W"))],
        ];
        for template in &templates {
            for pairs in &renamings {
                let want_map: Bindings = pairs
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), v.clone()))
                    .collect();
                let got = template.subst(&want_map);
                let mut inferred = Bindings::default();
                template.unify(&got, &mut inferred);
                assert_eq!(
                    template.subst(&inferred),
                    got,
                    "template={template}, got={got}, inferred={inferred:?}"
                );
            }
        }
    }
}
