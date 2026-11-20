//! C++ names for the program's names, and C++ types for its types.

use cpl_ir::{Program, Ty};

/// Names the emitted file can't give to the user's things: C++ keywords,
/// macros of the headers it includes, and the names the file itself uses.
const RESERVED: &[&str] = &[
    "alignas",
    "alignof",
    "and",
    "and_eq",
    "asm",
    "auto",
    "bitand",
    "bitor",
    "bool",
    "break",
    "case",
    "catch",
    "char",
    "char16_t",
    "char32_t",
    "class",
    "compl",
    "const",
    "constexpr",
    "const_cast",
    "continue",
    "decltype",
    "default",
    "delete",
    "do",
    "double",
    "dynamic_cast",
    "else",
    "enum",
    "explicit",
    "export",
    "extern",
    "false",
    "float",
    "for",
    "friend",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "not_eq",
    "nullptr",
    "operator",
    "or",
    "or_eq",
    "private",
    "protected",
    "public",
    "register",
    "reinterpret_cast",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "static_cast",
    "struct",
    "switch",
    "template",
    "this",
    "thread_local",
    "throw",
    "true",
    "try",
    "typedef",
    "typeid",
    "typename",
    "union",
    "unsigned",
    "using",
    "virtual",
    "void",
    "volatile",
    "wchar_t",
    "while",
    "xor",
    "xor_eq",
    // This file's own names and common macros.
    "ll",
    "cpl",
    "std",
    "prog",
    "io",
    "main",
    "solve",
    "tag",
    "cpl_tie",
    "assert",
    "errno",
    "stdin",
    "stdout",
    "stderr",
    "EOF",
    "NULL",
    "RAND_MAX",
    "INT_MAX",
    "INT_MIN",
    "LLONG_MAX",
    "LLONG_MIN",
    "LONG_MAX",
    "LONG_MIN",
    "CHAR_BIT",
    "SIZE_MAX",
    "HUGE_VAL",
    "INFINITY",
    "NAN",
    "BUFSIZ",
    "SEEK_SET",
    "SEEK_CUR",
    "SEEK_END",
    "EXIT_SUCCESS",
    "EXIT_FAILURE",
    "M_PI",
    "M_E",
    "linux",
    "unix",
];

/// `name` as a C++ identifier: itself, or with `_` added when C++ has
/// another use for the word.
pub(crate) fn ident(name: &str) -> String {
    if RESERVED.contains(&name) || name.starts_with("cpl_") {
        format!("{name}_")
    } else {
        name.to_string()
    }
}

/// A fn's C++ name. A method `Edge.other` is the free fn `Edge_other`.
pub(crate) fn fn_name(name: &str) -> String {
    match name.split_once('.') {
        Some((owner, method)) => format!("{owner}_{method}"),
        None => ident(name),
    }
}

/// True when every variant of the enum is a bare name, so it can be a
/// C++ `enum class`.
pub(crate) fn is_plain_enum(program: &Program, id: usize) -> bool {
    program.enums[id]
        .variants
        .iter()
        .all(|(_, fields)| fields.is_empty())
}

/// The C++ type of a value of type `ty`.
pub(crate) fn cty(program: &Program, ty: &Ty) -> String {
    match ty {
        Ty::Unit | Ty::Error => "void".into(),
        Ty::Int => "ll".into(),
        Ty::Float => "double".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "cpl::u8".into(),
        Ty::Str => "std::string".into(),
        Ty::Pos(_) | Ty::Bound(_) => "int".into(),
        Ty::Array { dims, elem } if dims.len() == 1 => {
            format!("std::vector<{}>", elem_ty(program, elem))
        }
        Ty::Array { dims, elem } => {
            format!("cpl::Grid<{}, {}>", elem_ty(program, elem), dims.len())
        }
        Ty::Opt(inner) => format!("std::optional<{}>", cty(program, inner)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(|t| cty(program, t)).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::Record(fields) => {
            let items: Vec<String> = fields.iter().map(|(_, t)| cty(program, t)).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::List(elem) => format!("std::vector<{}>", elem_ty(program, elem)),
        Ty::Map(key, value) => format!("std::map<{}, {}>", cty(program, key), cty(program, value)),
        Ty::Set(elem) => format!("std::set<{}>", cty(program, elem)),
        Ty::Struct(id) => ident(&program.structs[*id].name),
        Ty::Enum(id) => ident(&program.enums[*id].name),
    }
}

/// The C++ type of an element of a vector. `Bool` is stored as a byte:
/// `std::vector<bool>` packs bits and hands out proxies, not references.
pub(crate) fn elem_ty(program: &Program, ty: &Ty) -> String {
    match ty {
        Ty::Bool => "char".into(),
        other => cty(program, other),
    }
}
