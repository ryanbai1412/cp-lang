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
    /// Not known to this checker. Compatible with everything, so missing
    /// knowledge never becomes an error.
    Unknown,
    /// The type of code that already has a diagnostic. Compatible with
    /// everything, so one root cause is reported once.
    Error,
}

/// What each type parameter stands for at one use.
pub type Bindings = std::collections::HashMap<String, Ty>;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
    pub mutable: bool,
    pub has_default: bool,
    pub named_only: bool,
    pub variadic: bool,
}

impl Ty {
    pub fn list(elem: Ty) -> Ty {
        Ty::Named {
            name: "List".into(),
            args: vec![elem],
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
        match (self, want) {
            (a, b) if a == b => true,
            (a, b) if a.is_vague() || b.is_vague() => true,
            // What a type parameter can do is its bounds' business; the
            // code is checked again with real types at each use.
            (Ty::Param(_), _) | (_, Ty::Param(_)) => true,
            (a, b) if a.is_int() && b.is_int() => true,
            (a, Ty::Float) if a.is_int() => true,
            // An `Int` is reduced into a `Mod<P>` (decision 3.10).
            (a, Ty::Named { name, .. }) if a.is_int() && name == "Mod" => true,
            (a, Ty::Optional(inner)) => a.fits(inner),
            (
                Ty::Array {
                    index: ai,
                    elem: ae,
                },
                Ty::Array {
                    index: bi,
                    elem: be,
                },
            ) => ai.len() == bi.len() && ai.iter().zip(bi).all(|(a, b)| a.fits(b)) && ae.fits(be),
            (Ty::Tuple(a), Ty::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.fits(b))
            }
            // A record with more fields fits where fewer are asked for:
            // edges `(u, v, w)` are edges `(u, v)`.
            (Ty::Record(a), Ty::Record(b)) => b.iter().all(|(bn, bt)| {
                a.iter()
                    .find(|(an, _)| an == bn)
                    .is_some_and(|(_, at)| at.fits(bt))
            }),
            (Ty::Tuple(a), Ty::Record(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, (_, b))| a.fits(b))
            }
            (Ty::Named { name: an, args: aa }, Ty::Named { name: bn, args: ba }) => {
                // `[k: v]` is written the same for either kind of map.
                (an == bn || (an == "Map" && bn == "HashMap"))
                    && (aa.is_empty()
                        || ba.is_empty()
                        || (aa.len() == ba.len() && aa.iter().zip(ba).all(|(a, b)| a.fits(b))))
            }
            (Ty::Fn { .. }, Ty::Fn { .. }) => true,
            // `Bounds<V>` is how library code spells `V.bounds`.
            (Ty::Pos(bounds), Ty::Named { name, args })
            | (Ty::Named { name, args }, Ty::Pos(bounds))
                if name == "Bounds" =>
            {
                match args.as_slice() {
                    [Ty::Pos(domain)] => bounds.strip_suffix(".bounds") == Some(domain),
                    _ => bounds.ends_with(".bounds"),
                }
            }
            // A string is a list of its characters (decision 9: `Str` is
            // `[domain S]Char`, one flat byte array).
            (Ty::Str, Ty::Named { name, args }) if name == "List" => {
                args.first().is_none_or(|elem| Ty::Char.fits(elem))
            }
            (Ty::Named { name, args }, Ty::Str) if name == "List" => {
                args.first().is_none_or(|elem| elem.fits(&Ty::Char))
            }
            // An array is read where a list is asked for: `running_sum(a)`.
            (Ty::Array { elem, .. }, Ty::Named { name, args }) if name == "List" => {
                args.first().is_none_or(|want| elem.fits(want))
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

    /// The type with each type parameter in `bindings` replaced by its
    /// type: `[D]T` with `D = V, T = Int` is `[V]Int`.
    pub fn subst(&self, bindings: &Bindings) -> Ty {
        if bindings.is_empty() {
            return self.clone();
        }
        self.map(&mut |ty| match ty {
            Ty::Param(name) => bindings.get(name).cloned(),
            _ => None,
        })
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
            (_, Ty::Unknown | Ty::Error | Ty::Param(_)) => {}
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
                for ((_, want), (_, got)) in want.iter().zip(got) {
                    want.unify(got, bindings);
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
            Ty::Named { name, args } => {
                f.write_str(name)?;
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
            let spare = match &**inner {
                Ty::Int | Ty::Pos(_) => true,
                Ty::Range {
                    lo: Some(lo),
                    hi: Some(hi),
                    ..
                } => hi - lo + 1 < 1i128 << (8 * inner_size),
                _ => false,
            };
            if spare {
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
    let mut size = 0u64;
    let mut align = 1u64;
    for field in fields {
        let field = field?;
        let a = field.clamp(1, 8).next_power_of_two().min(8);
        align = align.max(a);
        size = size.div_ceil(a) * a + field;
    }
    Some(size.div_ceil(align) * align)
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
        assert!(Ty::Int.fits(&Ty::Float));
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
}
