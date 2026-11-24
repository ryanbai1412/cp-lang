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
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
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

/// A domain's C++ name, the stem of its `_lo`/`_len` statics. A library
/// domain shares its struct's name, and the struct's `len` method is
/// already `S_len`, so it takes `_dom` suffixes until no struct or fn has
/// the name or a static's.
/// A top-level name rebound by `let x = x!` (R02.4.8) is a global of its
/// own: each one after the first gets a suffix, so one scope holds them all.
pub(crate) fn global_ident(program: &Program, id: usize) -> String {
    let name = &program.globals[id].name;
    let earlier = program.globals[..id]
        .iter()
        .filter(|g| &g.name == name)
        .count();
    match earlier {
        0 => ident(name),
        n => format!("{}__{n}", ident(name)),
    }
}

pub(crate) fn dom_ident(program: &Program, id: usize) -> String {
    let mut d = ident(&program.domains[id]);
    let taken = |d: &str| {
        let (lo, len) = (format!("{d}_lo"), format!("{d}_len"));
        program.structs.iter().any(|s| ident(&s.name) == d)
            || program.fns.iter().any(|f| {
                let n = fn_name(&f.name);
                n == lo || n == len
            })
    };
    while taken(&d) {
        d.push_str("_dom");
    }
    d
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

/// `?S` for a struct with a field whose type has a niche no value of it
/// holds (an `Int` at `INT64_MIN`, a position at -1) stores `none` there,
/// so it is as wide as `S` (decision 28.10). A field narrowed below 64 bits
/// has no free value, so it is passed over.
pub(crate) fn struct_niche(program: &Program, id: usize) -> Option<&str> {
    let def = &program.structs[id];
    def.fields
        .iter()
        .enumerate()
        .find_map(|(at, (name, ty))| match ty {
            Ty::Int if !def.narrow_ints.contains_key(&at) => Some(name.as_str()),
            Ty::Pos(_) | Ty::Bound(_) => Some(name.as_str()),
            _ => None,
        })
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
        Ty::StrView => "cpl::StrView".into(),
        Ty::Pos(d) | Ty::Bound(d) => crate::width::pos_cpp(program, *d).into(),
        Ty::Range(inner) => match **inner {
            Ty::Pos(d) | Ty::Bound(d) => {
                format!("cpl::Range<{}>", crate::width::pos_cpp(program, d))
            }
            Ty::I128 => "cpl::Range<__int128>".into(),
            Ty::Float => "cpl::Range<double>".into(),
            _ => "cpl::Range<ll>".into(),
        },
        Ty::DomainVal => "cpl::DomainVal".into(),
        Ty::Array { dims, elem } if dims.len() == 1 => {
            format!("std::vector<{}>", elem_ty(program, niches, elem))
        }
        Ty::Array { dims, elem } => {
            format!(
                "cpl::Grid<{}, {}>",
                elem_ty(program, niches, elem),
                dims.len()
            )
        }
        Ty::Opt(inner) => match **inner {
            // A bare operator the fn was specialized for holds no value; only presence is left.
            Ty::Unit => "bool".into(),
            Ty::Pos(d) | Ty::Bound(d) if crate::width::is_wide(program, d) => "cpl::OptInt".into(),
            Ty::Pos(_) | Ty::Bound(_) => "cpl::OptPos".into(),
            // `Int` stops above INT64_MIN, which is left for `none`.
            Ty::Int => "cpl::OptInt".into(),
            Ty::Bool if niches.bools => "cpl::OptBool".into(),
            Ty::Char if niches.chars => "cpl::OptChar".into(),
            // A tuple with a position in it: `none` is that position at -1.
            Ty::Tuple(_) | Ty::Record(_) => {
                let held = cty(program, niches, inner);
                match pos_path(inner) {
                    Some(path) => {
                        let path: Vec<String> = path.iter().map(usize::to_string).collect();
                        format!("cpl::OptTuple<{held}, {}>", path.join(", "))
                    }
                    None => format!("std::optional<{held}>"),
                }
            }
            Ty::Struct(id) => match struct_niche(program, id) {
                Some(field) => format!(
                    "cpl::OptField<&{}::{}>",
                    ident(&program.structs[id].name),
                    ident(field)
                ),
                None => format!("std::optional<{}>", cty(program, niches, inner)),
            },
            Ty::Mut(ref of) => format!("std::optional<{}>", cty(program, niches, of)),
            _ => format!("std::optional<{}>", cty(program, niches, inner)),
        },
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(|t| value_ty(program, niches, t)).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::Record(fields) => {
            let items: Vec<String> = fields
                .iter()
                .map(|(_, t)| value_ty(program, niches, t))
                .collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        Ty::List(elem) => format!("std::vector<{}>", elem_ty(program, niches, elem)),
        Ty::ListView(elem, _) => format!("cpl::ListView<{}>", elem_ty(program, niches, elem)),
        Ty::Keyed(elem, _) => format!("cpl::Keyed<{}>", elem_ty(program, niches, elem)),
        Ty::MutView(elem, _) => format!("cpl::MutView<{}>", elem_ty(program, niches, elem)),
        Ty::Map(key, value) if program.unordered_maps.contains(key) => {
            cty(program, niches, &Ty::HashMap(key.clone(), value.clone()))
        }
        Ty::Set(elem) if program.unordered_sets.contains(elem) => {
            cty(program, niches, &Ty::HashSet(elem.clone()))
        }
        Ty::Map(key, value) => format!(
            "std::map<{}, {}>",
            value_ty(program, niches, key),
            value_ty(program, niches, value)
        ),
        Ty::HashMap(key, value) => format!(
            "cpl::HashMap<{}, {}, cpl::CpHash>",
            value_ty(program, niches, key),
            value_ty(program, niches, value)
        ),
        Ty::Set(elem) => format!("std::set<{}>", value_ty(program, niches, elem)),
        Ty::HashSet(elem) => format!(
            "cpl::HashSet<{}, cpl::CpHash>",
            value_ty(program, niches, elem)
        ),
        Ty::Mut(of) => format!("{}&", cty(program, niches, of)),
        Ty::Fn(args, ret) => {
            let pointer = fn_pointer(program, args.len());
            let ps: Vec<String> = args
                .iter()
                .map(|t| match t {
                    // A lambda takes these by `const&` (see `Emitter::closure`), and
                    // a function pointer must name the lambda's exact parameters.
                    _ if pointer
                        && !matches!(t, Ty::Mut(_))
                        && !t.is_scalar()
                        && !cached(program, t) =>
                    {
                        format!("const {}&", value_ty(program, niches, t))
                    }
                    _ => value_ty(program, niches, t),
                })
                .collect();
            let erased = if pointer {
                "std::add_pointer_t"
            } else {
                "std::function"
            };
            format!("{erased}<{}({})>", cty(program, niches, ret), ps.join(", "))
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
        other => value_ty(program, niches, other),
    }
}

/// The C++ type of a value that is stored or passed: C++ has no `void`
/// value, so a `()` held in a tuple, a list or a parameter is `cpl::Unit`.
pub(crate) fn value_ty(program: &Program, niches: Niches, ty: &Ty) -> String {
    match ty {
        Ty::Unit => "cpl::Unit".into(),
        other => cty(program, niches, other),
    }
}

/// A type as the judge's grader writes it: what `cpl::Judge<T>::type` is.
pub(crate) fn judge_ty(ty: &Ty) -> String {
    match ty {
        Ty::Unit => "void".into(),
        Ty::Int => "int".into(),
        Ty::Float => "double".into(),
        Ty::Bool => "bool".into(),
        Ty::Char => "char".into(),
        Ty::Str => "std::string".into(),
        Ty::StrView => "cpl::StrView".into(),
        Ty::List(elem) => format!("std::vector<{}>", judge_ty(elem)),
        Ty::ListView(elem, _) => format!("cpl::ListView<{}>", judge_ty(elem)),
        Ty::Keyed(elem, _) => format!("cpl::Keyed<{}>", judge_ty(elem)),
        Ty::MutView(elem, _) => format!("cpl::MutView<{}>", judge_ty(elem)),
        Ty::Tuple(items) => {
            let items: Vec<String> = items.iter().map(judge_ty).collect();
            format!("std::tuple<{}>", items.join(", "))
        }
        _ => unreachable!("lowering lets only these types cross to the judge"),
    }
}

/// `(ty)` applied to all of `text`, C++ of unknown precedence.
pub(crate) fn cast(ty: &str, text: &str) -> String {
    if text
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | ':' | '.'))
    {
        format!("({ty}){text}")
    } else {
        format!("({ty})({text})")
    }
}

/// True for the stdlib `Bitset`: its words are its `w: List<U64>`.
pub(crate) fn is_bitset_ty(ty: &Ty, program: &Program) -> bool {
    let Ty::Struct(sid) = ty else {
        return false;
    };
    let s = &program.structs[*sid];
    s.name.starts_with("Bitset")
        && s.fields.iter().any(|(name, ty)| {
            name == "w" && matches!(ty, Ty::List(elem) if elem.as_ref() == &Ty::U64)
        })
}

/// The path through nested tuples to a position in `ty`, which `?ty` keeps
/// at -1 for `none`: one directly in `ty` first, else the first one nested.
fn pos_path(ty: &Ty) -> Option<Vec<usize>> {
    let fields = ty.fields()?;
    if let Some(at) = fields
        .iter()
        .position(|f| matches!(f, Ty::Pos(_) | Ty::Bound(_)))
    {
        return Some(vec![at]);
    }
    fields.iter().enumerate().find_map(|(at, f)| {
        let mut path = pos_path(f)?;
        path.insert(0, at);
        Some(path)
    })
}

/// Whether a fn value of this arity lowers to a plain function pointer: every
/// closure of that arity in the whole program captures nothing, so each value
/// that can reach the slot is a captureless lambda.
pub(crate) fn fn_pointer(program: &Program, arity: usize) -> bool {
    program
        .closures
        .iter()
        .all(|c| c.params.len() != arity || c.captures.is_empty())
}

/// Whether reading a value may fill a cache: a struct with a `@cache`
/// field, or one holding such a value (`Holder(d: Dsu<V>)`).
pub(crate) fn cached(program: &Program, ty: &Ty) -> bool {
    cached_in(program, ty, &mut Vec::new())
}

fn cached_in(program: &Program, ty: &Ty, seen: &mut Vec<usize>) -> bool {
    match ty {
        Ty::Struct(sid) if !seen.contains(sid) => {
            seen.push(*sid);
            let def = &program.structs[*sid];
            !def.cached_fields.is_empty()
                || def.fields.iter().any(|(_, f)| cached_in(program, f, seen))
        }
        Ty::Tuple(items) => items.iter().any(|t| cached_in(program, t, seen)),
        Ty::Opt(inner) => cached_in(program, inner, seen),
        _ => false,
    }
}
