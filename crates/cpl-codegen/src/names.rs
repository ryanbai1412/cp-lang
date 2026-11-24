//! C++ names for the program's names, and C++ types for its types.

use cpl_ir::{spare, JudgeTy, Program, Ty};
use std::collections::BTreeSet;

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
    "LOCAL",
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

/// Where `?S` keeps its `None` when `S` has a spare value (R03.2.2): in the
/// first field, in declaration order, that has one, reached through the
/// member pointers `path`, at the spare value after the `taken` ones the
/// packed optionals along the path keep for their own `None`s.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct StructNiche {
    pub field: String,
    pub path: Vec<String>,
    pub taken: u32,
    pub spares: u128,
}

/// `?S` for a struct with a spare value is as wide as `S` (decision 28.10),
/// counted by `cpl_check::ty::spare`'s rules, as layout and `cpl explain` count.
pub(crate) fn struct_niche(program: &Program, id: usize) -> Option<StructNiche> {
    struct_niche_at(program, id, 0)
}

fn struct_niche_at(program: &Program, id: usize, depth: u32) -> Option<StructNiche> {
    if depth > 8 {
        return None;
    }
    let def = &program.structs[id];
    let owner = ident(&def.name);
    let counts = def.fields.iter().enumerate().map(|(at, (name, ty))| {
        let member = format!("&{owner}::{}", ident(name));
        let range = match (def.int_ranges.get(&at), def.narrow_ints.contains_key(&at)) {
            (Some(&(lo, hi)), _) => Some(Some((lo as i128, hi as i128))),
            // A narrowed field's width comes from bounds not kept here.
            (None, true) => None,
            (None, false) => Some(None),
        };
        field_niche(program, member, ty, range, depth)
    });
    let found: Vec<Option<(u128, Vec<String>, u32)>> = counts.collect();
    let (at, spares) = spare::first(found.iter().map(|f| f.as_ref().map_or(0, |f| f.0)))?;
    let (_, path, taken) = found[at].clone()?;
    Some(StructNiche {
        field: def.fields[at].0.clone(),
        path,
        taken,
        spares,
    })
}

/// A field's spare values, the path to the scalar holding them and how many
/// of them its packed optionals take; `range` is an `Int`'s bounds when
/// known, `None` when they are unknown.
fn field_niche(
    program: &Program,
    member: String,
    ty: &Ty,
    range: Option<Option<(i128, i128)>>,
    depth: u32,
) -> Option<(u128, Vec<String>, u32)> {
    let int = |range: Option<Option<(i128, i128)>>| match range? {
        Some((lo, hi)) => Some(spare::of_range(lo, hi)),
        None => Some(1),
    };
    let inside = || format!("&decltype({})::raw", member.trim_start_matches('&'));
    match ty {
        Ty::Int => Some((int(range)?, vec![member], 0)),
        Ty::Pos(_) | Ty::Bound(_) => Some((spare::MANY, vec![member], 0)),
        Ty::Struct(id) => {
            let inner = struct_niche_at(program, *id, depth + 1)?;
            let path = std::iter::once(member).chain(inner.path).collect();
            Some((inner.spares, path, inner.taken))
        }
        Ty::Opt(inner) => match inner.as_ref() {
            Ty::Int => Some((
                spare::of_opt(int(range)?),
                vec![member.clone(), inside()],
                1,
            )),
            Ty::Pos(_) | Ty::Bound(_) => Some((
                spare::of_opt(spare::MANY),
                vec![member.clone(), inside()],
                1,
            )),
            Ty::Struct(id) => {
                let held = struct_niche_at(program, *id, depth + 1)?;
                let path = [member.clone(), inside()]
                    .into_iter()
                    .chain(held.path)
                    .collect();
                Some((spare::of_opt(held.spares), path, held.taken + 1))
            }
            _ => None,
        },
        _ => None,
    }
}

/// The C++ type of a value of type `ty`.
/// `?()`: only presence is left, a `bool` (see `cty`).
pub(crate) fn is_flag_opt(ty: &Ty) -> bool {
    matches!(ty, Ty::Opt(inner) if **inner == Ty::Unit)
}

pub(crate) fn cty(program: &Program, niches: Niches, ty: &Ty) -> String {
    match ty {
        Ty::Unit | Ty::Error => "void".into(),
        Ty::Int => "ll".into(),
        Ty::I128 => "__int128".into(),
        Ty::U64 => "cpl::u64".into(),
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
        Ty::Array { dims, elem } if dims.len() == 1 => match fixed_len(program, &dims[0]) {
            Some(n) => format!("std::array<{}, {n}>", elem_ty(program, niches, elem)),
            None => format!("std::vector<{}>", elem_ty(program, niches, elem)),
        },
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
                Some(niche) => format!("cpl::OptField<{}, {}>", niche.taken, niche.path.join(", ")),
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
            let pointer = fn_pointer(program, args, ret);
            let bound = if pointer {
                None
            } else {
                fn_bound(program, args, ret)
            };
            let exact = pointer || bound.is_some();
            let ps: Vec<String> = args
                .iter()
                .map(|t| match t {
                    // A lambda takes these by `const&` (see `Emitter::closure`), and
                    // a function pointer must name the lambda's exact parameters.
                    _ if exact
                        && !matches!(t, Ty::Mut(_))
                        && !t.is_scalar()
                        && !cached(program, t) =>
                    {
                        format!("const {}&", value_ty(program, niches, t))
                    }
                    _ => value_ty(program, niches, t),
                })
                .collect();
            if let Some(env) = bound {
                let env: Vec<String> = env.iter().map(|t| value_ty(program, niches, t)).collect();
                return format!(
                    "cpl::Bound<cpl::Env<{}>, {}({})>",
                    env.join(", "),
                    cty(program, niches, ret),
                    ps.join(", ")
                );
            }
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
        // `std::vector<bool>` hands out proxies, not references.
        Ty::Bool => "char".into(),
        flag if is_flag_opt(flag) => "char".into(),
        // A position of a domain of at most 255 cells, its end included, is a byte.
        Ty::Pos(d) | Ty::Bound(d) if small_domain(program, *d) => "cpl::u8".into(),
        other => value_ty(program, niches, other),
    }
}

fn small_domain(program: &Program, d: usize) -> bool {
    program.byte_domains.get(d).copied().unwrap_or(false)
}

/// The C++ type of a value that is stored or passed: C++ has no `void`
/// value, so a `()` held in a tuple, a list or a parameter is `cpl::Unit`.
pub(crate) fn value_ty(program: &Program, niches: Niches, ty: &Ty) -> String {
    match ty {
        Ty::Unit => "cpl::Unit".into(),
        other => cty(program, niches, other),
    }
}

/// A type as the judge's grader writes it (R10.1.4).
pub(crate) fn judge_ty(ty: &JudgeTy) -> String {
    match ty {
        JudgeTy::Int => "int".into(),
        JudgeTy::Long => "long long".into(),
        JudgeTy::List(elem) => format!("std::vector<{}>", judge_ty(elem)),
        JudgeTy::Tuple(items) => {
            let items: Vec<String> = items.iter().map(judge_ty).collect();
            let of = if items.len() == 2 { "pair" } else { "tuple" };
            format!("std::{of}<{}>", items.join(", "))
        }
        JudgeTy::Void => "void".into(),
        JudgeTy::Double => "double".into(),
        JudgeTy::Bool => "bool".into(),
        JudgeTy::Char => "char".into(),
        JudgeTy::Str => "std::string".into(),
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

/// The `@bits` field of a struct, a `List<U64>` holding its words: its
/// name and position.
pub(crate) fn bits_field<'p>(ty: &Ty, program: &'p Program) -> Option<(&'p str, usize)> {
    let Ty::Struct(sid) = ty else {
        return None;
    };
    bits_field_of(&program.structs[*sid])
}

pub(crate) fn bits_field_of(def: &cpl_ir::StructDef) -> Option<(&str, usize)> {
    let at = *def.bits_fields.first()?;
    let (name, ty) = &def.fields[at];
    matches!(ty, Ty::List(elem) if elem.as_ref() == &Ty::U64).then_some((name.as_str(), at))
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

/// Whether a fn value of this signature lowers to a plain function pointer:
/// every closure of that signature the program can make captures nothing, so
/// each value that can reach the slot is a captureless lambda.
pub(crate) fn fn_pointer(program: &Program, args: &[Ty], ret: &Ty) -> bool {
    let same = |param: &Ty, arg: &Ty| param == arg || matches!(arg, Ty::Mut(of) if **of == *param);
    program.closures.iter().enumerate().all(|(cid, c)| {
        c.captures.is_empty()
            || program.unreached_closures.contains(&cid)
            || c.ret != *ret
            || c.params.len() != args.len()
            || !c
                .params
                .iter()
                .zip(args)
                .all(|(&p, a)| same(&c.locals[p].ty, a))
    })
}

/// The capture types shared by every capturing closure a fn of this
/// signature can hold, when each copies its captures in the same order:
/// the fn value is then a function beside its captures (`cpl::Bound`).
pub(crate) fn fn_bound(program: &Program, args: &[Ty], ret: &Ty) -> Option<Vec<Ty>> {
    let same = |param: &Ty, arg: &Ty| param == arg || matches!(arg, Ty::Mut(of) if **of == *param);
    let mut env: Option<Vec<Ty>> = None;
    for (cid, c) in program.closures.iter().enumerate() {
        if c.captures.is_empty()
            || program.unreached_closures.contains(&cid)
            || c.ret != *ret
            || c.params.len() != args.len()
            || !c
                .params
                .iter()
                .zip(args)
                .all(|(&p, a)| same(&c.locals[p].ty, a))
        {
            continue;
        }
        if c.captures.iter().any(|cap| cap.is_global || cap.by_ref) {
            return None;
        }
        let tys: Vec<Ty> = c
            .captures
            .iter()
            .map(|cap| c.locals[cap.inner].ty.clone())
            .collect();
        // A capture holding an erased fn could name this type inside
        // itself, and one holding a heap value is lent where the closure is
        // local.
        if tys.iter().any(|t| {
            holds_erased_fn(program, t, &mut BTreeSet::new()) || crate::lend::holds_heap(program, t)
        }) {
            return None;
        }
        match &env {
            Some(e) if *e != tys => return None,
            Some(_) => {}
            None => env = Some(tys),
        }
    }
    env
}

/// Whether `ty` holds by value a fn that is not a plain function pointer.
fn holds_erased_fn(program: &Program, ty: &Ty, seen: &mut BTreeSet<(bool, usize)>) -> bool {
    match ty {
        Ty::Fn(args, ret) => !fn_pointer(program, args, ret),
        Ty::Tuple(items) => items.iter().any(|t| holds_erased_fn(program, t, seen)),
        Ty::Record(fields) => fields
            .iter()
            .any(|(_, t)| holds_erased_fn(program, t, seen)),
        Ty::Opt(inner) | Ty::Mut(inner) | Ty::List(inner) => holds_erased_fn(program, inner, seen),
        Ty::Array { elem, .. } => holds_erased_fn(program, elem, seen),
        Ty::Struct(id) => {
            seen.insert((false, *id))
                && program.structs[*id]
                    .fields
                    .iter()
                    .any(|(_, t)| holds_erased_fn(program, t, seen))
        }
        Ty::Enum(id) => {
            seen.insert((true, *id))
                && program.enums[*id]
                    .variants
                    .iter()
                    .any(|(_, fields)| fields.iter().any(|t| holds_erased_fn(program, t, seen)))
        }
        _ => false,
    }
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

/// How many cells an array axis has when every run gives it the same small
/// count: the array is then a `std::array` in every position it appears.
pub(crate) fn fixed_len(program: &Program, dim: &cpl_ir::Dim) -> Option<u32> {
    match dim {
        cpl_ir::Dim::Pos(d) => program.fixed_lens.get(*d).copied().flatten(),
        _ => None,
    }
}
