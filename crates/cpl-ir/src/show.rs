//! The parts of printing that are decided by type alone, shared by both
//! back ends so `print` reads the same in the interpreter and in C++.
//!
//! The rules (`plan/16-io.md`): a sequence of numbers is space-separated,
//! a sequence of characters is written solid like a string, and a sequence
//! of anything with parts of its own (records, lists, rows) is one per
//! line. Records and tuples are their fields, space-separated.

use crate::{Dim, EnumDef, StructDef, Ty};

/// What goes between the elements of a printed sequence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sep {
    None,
    Space,
    Line,
}

impl Sep {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Sep::None => b"",
            Sep::Space => b" ",
            Sep::Line => b"\n",
        }
    }
}

/// The separator for a sequence whose elements have type `elem`. A struct
/// that says how it is shown (`impl Show`) is one word, as a number is:
/// a list of `Mod`s is one line.
pub fn sep_of(elem: &Ty, structs: &[StructDef]) -> Sep {
    match elem {
        Ty::Char => Sep::None,
        Ty::Struct(id) if structs[*id].show_fn.is_some() => Sep::Space,
        Ty::Opt(inner) => match sep_of(inner, structs) {
            Sep::None => Sep::Space,
            sep => sep,
        },
        Ty::Tuple(_)
        | Ty::Record(_)
        | Ty::Struct(_)
        | Ty::Enum(_)
        | Ty::Array { .. }
        | Ty::List(_)
        | Ty::Range(_)
        | Ty::Map(..)
        | Ty::HashMap(..)
        | Ty::Set(_)
        | Ty::HashSet(_) => Sep::Line,
        _ => Sep::Space,
    }
}

pub(crate) fn ty_name(
    ty: &Ty,
    domains: &[String],
    structs: &[StructDef],
    enums: &[EnumDef],
) -> String {
    let name = |t: &Ty| ty_name(t, domains, structs, enums);
    let list = |items: &[Ty]| items.iter().map(name).collect::<Vec<_>>().join(", ");
    match ty {
        Ty::Unit => "()".into(),
        Ty::Int => "Int".into(),
        Ty::I128 => "I128".into(),
        Ty::U64 => "U64".into(),
        Ty::Float => "F64".into(),
        Ty::Bool => "Bool".into(),
        Ty::Char => "Char".into(),
        Ty::Str | Ty::StrView => "Str".into(),
        Ty::Pos(d) => source_name(&domains[*d]).into(),
        Ty::Bound(d) => format!("{}.cuts", source_name(&domains[*d])),
        Ty::Range(inner) => format!("Range<{}>", name(inner)),
        Ty::DomainVal => "domain".into(),
        Ty::Array { dims, elem } => {
            let dims: Vec<String> = dims
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => source_name(&domains[*d]).into(),
                    Dim::Bound(d) => format!("{}.cuts", source_name(&domains[*d])),
                    Dim::Sym(d) => format!("Sym<{}>", source_name(&domains[*d])),
                    Dim::Enum(e) => enums[*e].name.clone(),
                    Dim::Int | Dim::IntFrom(_) => "Int".into(),
                })
                .collect();
            format!("[{}]{}", dims.join(", "), name(elem))
        }
        Ty::Opt(inner) => format!("?{}", name(inner)),
        Ty::Tuple(items) => format!("({})", list(items)),
        Ty::Record(fields) => {
            let fields: Vec<String> = fields
                .iter()
                .map(|(f, t)| format!("{f}: {}", name(t)))
                .collect();
            format!("({})", fields.join(", "))
        }
        Ty::List(elem) | Ty::ListView(elem, _) | Ty::Keyed(elem, _) | Ty::MutView(elem, _) => {
            format!("List<{}>", name(elem))
        }
        Ty::Map(k, v) => format!("Map<{}, {}>", name(k), name(v)),
        Ty::HashMap(k, v) => format!("HashMap<{}, {}>", name(k), name(v)),
        Ty::Set(elem) => format!("Set<{}>", name(elem)),
        Ty::HashSet(elem) => format!("HashSet<{}>", name(elem)),
        Ty::Fn(args, ret) => format!("({}) -> {}", list(args), name(ret)),
        Ty::Mut(of) => format!("mut {}", name(of)),
        Ty::Struct(id) => {
            let s = &structs[*id];
            shown(&s.name, &s.source, &list(&s.targs))
        }
        Ty::Enum(id) => {
            let e = &enums[*id];
            shown(&e.name, &e.source, &list(&e.targs))
        }
        Ty::Error => "_".into(),
    }
}

/// A type as the program writes it: `Pt`, not the module's `m1_Pt`.
fn shown(name: &str, source: &str, args: &str) -> String {
    match (source, args) {
        ("", _) => source_name(name).into(),
        (source, "") => source.into(),
        (source, args) => format!("{source}<{args}>"),
    }
}

/// `V` for a module's `m2_V`.
pub(crate) fn source_name(name: &str) -> &str {
    name.strip_prefix('m')
        .and_then(|rest| {
            let digits = rest.find(|c: char| !c.is_ascii_digit())?;
            (digits > 0).then_some(())?;
            rest[digits..].strip_prefix('_')
        })
        .unwrap_or(name)
}

/// "expected `want`, found `found`", naming views when both read the same.
pub(crate) fn mismatch(want: &str, found: &str, want_ty: &Ty, found_ty: &Ty) -> String {
    if want != found {
        format!("expected `{want}`, found `{found}`")
    } else if Ty::holds_view(found_ty) {
        format!(
            "expected `{want}`, found `{found}` made of views into another sequence, not copies"
        )
    } else if Ty::holds_view(want_ty) {
        format!(
            "expected `{want}` made of views into another sequence (it was built from a slice such as `xs[a..b]`), found an owned `{found}`; build it from copies, `xs[a..b].to_list()`"
        )
    } else {
        format!("expected `{want}`, found `{found}`")
    }
}
