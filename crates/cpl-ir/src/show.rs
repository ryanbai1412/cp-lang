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

/// The separator for a sequence whose elements have type `elem`.
pub fn sep_of(elem: &Ty) -> Sep {
    match elem {
        Ty::Char => Sep::None,
        Ty::Opt(inner) => match sep_of(inner) {
            Sep::None => Sep::Space,
            sep => sep,
        },
        Ty::Tuple(_)
        | Ty::Record(_)
        | Ty::Struct(_)
        | Ty::Enum(_)
        | Ty::Array { .. }
        | Ty::List(_)
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
        Ty::Float => "Float".into(),
        Ty::Bool => "Bool".into(),
        Ty::Char => "Char".into(),
        Ty::Str => "Str".into(),
        Ty::Pos(d) => domains[*d].clone(),
        Ty::Bound(d) => format!("{}.bounds", domains[*d]),
        Ty::Range(inner) => format!("Range<{}>", name(inner)),
        Ty::DomainVal => "domain".into(),
        Ty::Array { dims, elem } => {
            let dims: Vec<String> = dims
                .iter()
                .map(|d| match d {
                    Dim::Pos(d) => domains[*d].clone(),
                    Dim::Bound(d) => format!("{}.bounds", domains[*d]),
                    Dim::Int => "Int".into(),
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
        Ty::List(elem) => format!("List<{}>", name(elem)),
        Ty::Map(k, v) => format!("Map<{}, {}>", name(k), name(v)),
        Ty::HashMap(k, v) => format!("HashMap<{}, {}>", name(k), name(v)),
        Ty::Set(elem) => format!("Set<{}>", name(elem)),
        Ty::HashSet(elem) => format!("HashSet<{}>", name(elem)),
        Ty::Fn(args, ret) => format!("({}) -> {}", list(args), name(ret)),
        Ty::Struct(id) => structs[*id].name.clone(),
        Ty::Enum(id) => enums[*id].name.clone(),
        Ty::Error => "_".into(),
    }
}
