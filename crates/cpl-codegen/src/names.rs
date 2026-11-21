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
/// another use for the word. Any non-identifier chars are replaced by `_`.
pub(crate) fn ident(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect();
    let clean = if clean.is_empty() || clean.as_bytes()[0].is_ascii_digit() {
        format!("_{clean}")
    } else {
        clean
    };
    if RESERVED.contains(&clean.as_str()) || clean.starts_with("cpl_") {
        format!("{clean}_")
    } else {
        clean
    }
}

/// A fn's C++ name. A method `Edge.other` is the free fn `Edge_other`.
pub(crate) fn fn_name(name: &str) -> String {
    match name.split_once('.') {
        Some((owner, method)) => ident(&format!("{owner}_{method}")),
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

/// Which optionals of a one-byte value are held in one scalar. A position
/// always is; these two are not when the program takes the inside of one
/// as a place (`(flag ??= false) = true`), which needs a `bool&` or a
/// `u8&` that the packed form does not have.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Niches {
    pub bools: bool,
    pub chars: bool,
}

/// The C++ type of a value of type `ty`.
pub(crate) fn cty(program: &Program, niches: Niches, ty: &Ty) -> String {
    match ty {
        Ty::Unit | Ty::Error => "void".into(),
        Ty::Int => "ll".into(),
        Ty::I128 => "__int128".into(),
        Ty::U64 => "unsigned long long".into(),
        Ty::Float => "double".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "cpl::u8".into(),
        Ty::Str => "std::string".into(),
        Ty::Pos(_) | Ty::Bound(_) => "int".into(),
        Ty::Range(inner) => match **inner {
            Ty::Pos(_) | Ty::Bound(_) => "cpl::Range<int>".into(),
            Ty::I128 => "cpl::Range<__int128>".into(),
            _ => "cpl::Range<ll>".into(),
        },
        Ty::DomainVal => "cpl::DomainVal".into(),
        Ty::Array { dims, elem } if dims.len() == 1 => {
            format!("std::vector<{}>", elem_ty(program, niches, elem))
        }
        Ty::Array { dims, elem } => {
            format!("cpl::Grid<{}, {}>", elem_ty(program, niches, elem), dims.len())
        }
        Ty::Opt(inner) => match **inner {
            Ty::Pos(_) | Ty::Bound(_) => "cpl::OptPos".into(),
            // `Int` stops above INT64_MIN, which is left for `none`.
            Ty::Int => "cpl::OptInt".into(),
            Ty::Bool if niches.bools => "cpl::OptBool".into(),
            Ty::Char if niches.chars => "cpl::OptChar".into(),
            // A tuple with a position in it: `none` is that position at -1.
            Ty::Tuple(_) | Ty::Record(_) => {
                let held = cty(program, niches, inner);
                let at = inner
                    .fields()
                    .unwrap_or_default()
                    .iter()
                    .position(|item| matches!(item, Ty::Pos(_) | Ty::Bound(_)));
                match at {
                    Some(at) => format!("cpl::OptTuple<{held}, {at}>"),
                    None => format!("std::optional<{held}>"),
                }
            }
            _ => format!("std::optional<{}>", cty(program, niches, inner)),
        },
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(|t| cty(program, niches, t)).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::Record(fields) => {
            let items: Vec<String> = fields.iter().map(|(_, t)| cty(program, niches, t)).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::List(elem) => format!("std::vector<{}>", elem_ty(program, niches, elem)),
        Ty::Map(key, value) if program.unordered_maps.contains(key) => {
            cty(program, niches, &Ty::HashMap(key.clone(), value.clone()))
        }
        Ty::Set(elem) if program.unordered_sets.contains(elem) => cty(program, niches, &Ty::HashSet(elem.clone())),
        Ty::Map(key, value) => format!("std::map<{}, {}>", cty(program, niches, key), cty(program, niches, value)),
        Ty::HashMap(key, value) => format!(
            "std::unordered_map<{}, {}, cpl::CpHash>",
            cty(program, niches, key),
            cty(program, niches, value)
        ),
        Ty::Set(elem) => format!("std::set<{}>", cty(program, niches, elem)),
        Ty::HashSet(elem) => format!("std::unordered_set<{}, cpl::CpHash>", cty(program, niches, elem)),
        Ty::Mut(of) => format!("{}&", cty(program, niches, of)),
        Ty::Fn(args, ret) => {
            let ps: Vec<String> = args.iter().map(|t| cty(program, niches, t)).collect();
            format!("std::function<{}({})>", cty(program, niches, ret), ps.join(", "))
        }
        Ty::Struct(id) => ident(&program.structs[*id].name),
        Ty::Enum(id) => ident(&program.enums[*id].name),
    }
}

/// The C++ type of an element of a vector. `Bool` is stored as a byte:
/// `std::vector<bool>` packs bits and hands out proxies, not references.
pub(crate) fn elem_ty(program: &Program, niches: Niches, ty: &Ty) -> String {
    match ty {
        Ty::Bool => "char".into(),
        other => cty(program, niches, other),
    }
}
