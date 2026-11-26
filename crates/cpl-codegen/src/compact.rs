//! The last ways a file is made to fit its size bar, after the comments,
//! the source and half the indentation: no space the compiler does not
//! need, then shorter names for the instances codegen numbers and the
//! runtime's own definitions. Each works
//! a line at a time, so a part of the file shrinks as it does in the file.

use cpl_rt::cpp::TYPE_TEMPLATES;
use std::collections::{BTreeMap, BTreeSet};

/// Two characters that read as one token when nothing is between them.
const JOINED: &[&str] = &[
    "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "<=", ">=", "==", "!=",
    "&&", "||", "->", "::", "//", "/*", "*/", ".*", "<:", ":>", "<%", "%>", "%:", "##", "..",
];

fn word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The tokens of a line of C++ and whether space came before each, up to a
/// `//` comment. `None` for what this does not read: a block comment, a raw
/// string, or a literal the line does not close.
pub(crate) fn tokens(line: &str) -> Option<Vec<(bool, &str)>> {
    let b = line.as_bytes();
    let mut out = Vec::new();
    let (mut at, mut space) = (0, false);
    while at < b.len() {
        let c = b[at];
        let start = at;
        if c.is_ascii_whitespace() {
            space = true;
            at += 1;
            continue;
        }
        if c == b'/' && b.get(at + 1) == Some(&b'/') {
            break;
        }
        if c == b'/' && b.get(at + 1) == Some(&b'*') {
            return None;
        }
        if c == b'"' || c == b'\'' {
            if c == b'"'
                && out
                    .last()
                    .is_some_and(|&(_, t): &(bool, &str)| t.ends_with('R'))
                && !space
            {
                return None;
            }
            at += 1;
            loop {
                match b.get(at) {
                    None => return None,
                    Some(b'\\') => at += 2,
                    Some(&q) if q == c => break,
                    Some(_) => at += 1,
                }
            }
            at += 1;
        } else if c.is_ascii_digit() || c == b'.' && b.get(at + 1).is_some_and(u8::is_ascii_digit) {
            let hex = c == b'0' && matches!(b.get(at + 1), Some(b'x' | b'X'));
            at += 1;
            while at < b.len() {
                let d = b[at];
                let exponent = if hex {
                    matches!(d, b'p' | b'P')
                } else {
                    matches!(d, b'e' | b'E')
                };
                if exponent && matches!(b.get(at + 1), Some(b'+' | b'-')) {
                    at += 2;
                } else if word(d)
                    || d == b'.'
                    || d == b'\'' && b.get(at + 1).is_some_and(|&n| word(n))
                {
                    at += 1;
                } else {
                    break;
                }
            }
        } else if word(c) {
            while at < b.len() && word(b[at]) {
                at += 1;
            }
        } else {
            at += line[at..].chars().next().map_or(1, char::len_utf8);
        }
        out.push((space, &line[start..at]));
        space = false;
    }
    Some(out)
}

/// Whether `b` written right after `a` would read as other tokens.
pub(crate) fn needs_space(a: &str, b: &str) -> bool {
    let (Some(&first), Some(&x), [y, after @ ..]) =
        (a.as_bytes().first(), a.as_bytes().last(), b.as_bytes())
    else {
        return false;
    };
    let y = *y;
    let quote = |c: u8| c == b'"' || c == b'\'';
    (word(x) || quote(x)) && (word(y) || quote(y))
        || first.is_ascii_digit() && y == b'.'
        || y == b'.' && after.first().is_some_and(u8::is_ascii_digit)
        || JOINED.iter().any(|j| j.as_bytes() == [x, y])
}

/// One line with only the spaces the compiler needs; a comment line, or
/// one with nothing on it, is gone. A preprocessor line loses only its
/// indentation.
fn compact_line(line: &str, rename: &BTreeMap<String, String>) -> String {
    let code = line.trim();
    if code.is_empty() || code.starts_with("//") {
        return String::new();
    }
    if code.starts_with('#') {
        return format!("{code}\n");
    }
    let Some(tokens) = tokens(code) else {
        return format!("{code}\n");
    };
    let mut out = String::with_capacity(code.len());
    let mut last: Option<&str> = None;
    let mut i = 0;
    while i < tokens.len() {
        let (space, mut token) = tokens[i];
        let qualified = |ns: &str| {
            token == ns
                && tokens.get(i + 1).map(|t| t.1) == Some(":")
                && tokens.get(i + 2).map(|t| t.1) == Some(":")
        };
        if qualified("std") && i + 3 < tokens.len() {
            let name = tokens[i + 3].1;
            let template = tokens.get(i + 4).map(|t| t.1) == Some("<");
            if let Some(alias) = rename.get(&format!("std::{name}")) {
                if template || alias == name {
                    token = alias;
                    i += 3;
                }
            }
        } else if qualified("cpl") && i + 3 < tokens.len() {
            if let Some(short) = rename.get(&format!("cpl::{}", tokens[i + 3].1)) {
                token = short;
                i += 3;
            }
        }
        // A call codegen wrote `::prog::f(..)` to keep it from a library fn
        // of the same name needs no namespace once `f` is a short name with
        // a capital, which no library name has.
        let home = [":", ":", "prog", ":", ":"];
        if tokens.len() > i + 5 && (0..5).all(|k| tokens[i + k].1 == home[k]) {
            if let Some(short) = rename.get(tokens[i + 5].1) {
                if short.bytes().any(|b| b.is_ascii_uppercase()) {
                    token = short;
                    i += 5;
                }
            }
        }
        let token = rename.get(token).map_or(token, String::as_str);
        if let Some(last) = last {
            if space && needs_space(last, token) {
                out.push(' ');
            }
        }
        out.push_str(token);
        last = Some(token);
        i += 1;
    }
    match out.is_empty() {
        true => out,
        false => out + "\n",
    }
}

/// What `rename` needs declared ahead of the code: an alias for each
/// library template it shortens, and the runtime's namespace opened for
/// the runtime names it writes without `cpl::`.
fn aliases(rename: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (name, short) in rename {
        if name.strip_prefix("std::") == Some(short) {
            out += &format!("using {name};\n");
        } else if name.starts_with("std::") {
            out += &format!("template<class...T>using {short}={name}<T...>;\n");
        } else if TYPEDEFS.contains(&name.as_str()) {
            out += &format!("using {short}={name};\n");
        }
    }
    if rename.keys().any(|name| name.starts_with("cpl::")) {
        out += "namespace cpl{}using namespace cpl;\n";
    }
    out
}

/// `cpp` with every line compacted and `rename` applied; the comments at
/// the top stay when `header`. What the renames need declared goes after
/// the line that names `ll`, ahead of everything else the file defines.
pub(crate) fn compact(cpp: &str, header: bool, rename: &BTreeMap<String, String>) -> String {
    let mut leading = header;
    let mut out = String::with_capacity(cpp.len());
    for line in cpp.split_inclusive('\n') {
        leading &= line.starts_with("//");
        match leading {
            true => out.push_str(line),
            false => {
                let line = compact_line(line, rename);
                out.push_str(&line);
                if line == "using ll=long long;\n" {
                    out.push_str(&aliases(rename));
                }
            }
        }
    }
    out
}

/// A name codegen numbered: an instance (`Mod_998244353_9_mul`), a local
/// made unique (`i_2`); a part of it after the first `_` is all digits.
/// Never a name from a library header, but for the few the file reaches as
/// `std::`, which keep theirs.
fn numbered(name: &str) -> bool {
    module_item(name)
        || name.as_bytes()[0].is_ascii_alphabetic()
            && name.bytes().any(|c| c.is_ascii_lowercase())
            && name
                .split('_')
                .skip(1)
                .any(|part| !part.is_empty() && part.bytes().all(|c| c.is_ascii_digit()))
}

/// A name the emitter makes up, `cpl_tie`: no library's member starts `cpl_`.
/// The program's entry, `cpl_main`, keeps its name, so a reader finds it.
fn ours(name: &str) -> bool {
    name != "cpl_main"
        && name
            .strip_prefix("cpl_")
            .is_some_and(|rest| !rest.is_empty())
}

/// An item of an imported module: `m1_MaxSub`, the module's number before the name.
fn module_item(name: &str) -> bool {
    name.strip_prefix('m')
        .and_then(|rest| rest.split_once('_'))
        .is_some_and(|(m, item)| {
            !m.is_empty() && m.bytes().all(|c| c.is_ascii_digit()) && !item.is_empty()
        })
}

/// The names the runtime's pieces define. One is shortened like a numbered
/// name unless the file also writes it as a member (`x.at`), which may be a
/// library's, or it is one the language or library finds by its name.
fn runtime_names() -> &'static BTreeSet<&'static str> {
    static NAMES: std::sync::OnceLock<BTreeSet<&'static str>> = std::sync::OnceLock::new();
    NAMES.get_or_init(|| {
        crate::runtime::pieces()
            .flat_map(|p| crate::runtime::shake::defined(p.code))
            .filter(|name| !FOUND_BY_NAME.contains(name))
            .collect()
    })
}

const FOUND_BY_NAME: &[&str] = &[
    "begin", "end", "swap", "get", "size", "data", "hash", "main",
];

/// Library names a `using std::x;` can bring in, so the file writes them
/// bare where it writes no other `x`.
const STD_NAMES: &[&str] = &[
    "move", "get", "max", "min", "string", "nullopt", "tie", "forward",
];

/// Library type names written bare, which a `using` can stand for.
const TYPEDEFS: &[&str] = &[
    "size_t", "uint64_t", "uint32_t", "uint8_t", "int64_t", "int32_t",
];

const KEYWORDS: &[&str] = &[
    "do", "if", "or", "and", "asm", "for", "int", "new", "not", "try", "xor",
];

/// The `i`th short name: `Q0` .. `Qz`, then `Q00` .. `Qzz`, and so on.
fn short_name(mut i: usize) -> String {
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut rest = Vec::new();
    loop {
        rest.push(char::from(DIGITS[i % 36]));
        i /= 36;
        if i == 0 {
            break;
        }
        i -= 1;
    }
    std::iter::once('Q').chain(rest.into_iter().rev()).collect()
}

/// Short names for the numbered names in `cpp` and the runtime's, the most bytes saved
/// first: `Q0`, `Qa`, ..., then three characters. A name written after
/// `std::` or on a preprocessor line keeps its name.
pub(crate) fn short_names(cpp: &str) -> BTreeMap<String, String> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    let mut taken: BTreeSet<&str> = BTreeSet::new();
    let mut keep: BTreeSet<&str> = BTreeSet::new();
    let mut members: BTreeSet<&str> = BTreeSet::new();
    let mut templates: BTreeMap<&str, usize> = BTreeMap::new();
    // Names written outside namespace cpl without `cpl::`: a local there may
    // share the name, so a runtime name among them keeps its `cpl::`.
    let mut bare_outside: BTreeSet<&str> = BTreeSet::new();
    // Each name of `STD_NAMES` written after `std::`, and every name written
    // without it.
    let mut std_names: BTreeMap<&str, usize> = BTreeMap::new();
    let mut unqualified: BTreeSet<&str> = BTreeSet::new();
    let mut cpl_depth = 0i32;
    // The names the file's own namespaces define, as well as the runtime's.
    let mut in_cpl: BTreeSet<&str> = runtime_names().clone();
    let own = |ns| {
        crate::runtime::shake::defined_in(cpp, ns)
            .into_iter()
            .filter(|n| !FOUND_BY_NAME.contains(n))
    };
    in_cpl.extend(own("cpl"));
    let mut owned = in_cpl.clone();
    owned.extend(own("prog"));
    let prog: BTreeSet<&str> = own("prog").collect();
    // Outside every namespace, a fn of the program's is the one the judge's
    // grader links by its name.
    let mut ns_depth = 0i32;
    for line in cpp.lines() {
        let code = line.trim_start();
        let inside = cpl_depth > 0;
        let linked = ns_depth == 0;
        if ns_depth > 0 {
            ns_depth += code.matches('{').count() as i32 - code.matches('}').count() as i32;
        } else if code.starts_with("namespace ") && code.trim_end().ends_with('{') {
            ns_depth = 1;
        }
        if inside {
            cpl_depth += code.matches('{').count() as i32 - code.matches('}').count() as i32;
        } else if matches!(code.trim_end(), "namespace cpl{" | "namespace cpl {") {
            cpl_depth = 1;
        }
        let Some(tokens) = tokens(code).filter(|_| !code.starts_with('#')) else {
            for t in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                keep.insert(t);
                if !code.starts_with("#include") {
                    unqualified.insert(t);
                }
            }
            continue;
        };
        for (i, &(_, t)) in tokens.iter().enumerate() {
            if !word(t.as_bytes()[0]) {
                continue;
            }
            taken.insert(t);
            let after_cpl = i >= 3
                && tokens[i - 1].1 == ":"
                && tokens[i - 2].1 == ":"
                && tokens[i - 3].1 == "cpl";
            if !inside && !after_cpl {
                bare_outside.insert(t);
            }
            let after_prog = i >= 3
                && tokens[i - 1].1 == ":"
                && tokens[i - 2].1 == ":"
                && tokens[i - 3].1 == "prog";
            if linked && !after_prog && prog.contains(t) {
                keep.insert(t);
            }
            let after_std = i >= 3
                && tokens[i - 1].1 == ":"
                && tokens[i - 2].1 == ":"
                && tokens[i - 3].1 == "std";
            let arrow = i >= 2 && tokens[i - 2].1 == "-" && tokens[i - 1].1 == ">";
            // `X<T>::value` names a member of whatever `X` is, as `x.value` does.
            let scoped = i >= 2
                && tokens[i - 1].1 == ":"
                && tokens[i - 2].1 == ":"
                && (i < 3 || !matches!(tokens[i - 3].1, "cpl" | "prog"));
            if i >= 1 && tokens[i - 1].1 == "." || arrow || scoped && !after_std {
                members.insert(t);
            }
            if !after_std || tokens.get(i + 1).map(|t| t.1) == Some(":") {
                unqualified.insert(t);
            } else if STD_NAMES.contains(&t) {
                *std_names.entry(t).or_default() += 1;
            }
            if after_std {
                keep.insert(t);
                if TYPE_TEMPLATES.contains(&t) && tokens.get(i + 1).map(|t| t.1) == Some("<") {
                    *templates.entry(t).or_default() += 1;
                }
            } else if numbered(t) || ours(t) || owned.contains(t) || TYPEDEFS.contains(&t) {
                *count.entry(t).or_default() += 1;
            }
        }
    }
    let mut names: Vec<(String, usize)> = count
        .into_iter()
        .filter(|(name, _)| !keep.contains(name))
        .filter(|(name, _)| numbered(name) || ours(name) || !members.contains(name))
        // A `using` for a library type costs about 20 bytes.
        .filter(|(name, n)| !TYPEDEFS.contains(name) || n * (name.len() - 2) > 20)
        .map(|(name, n)| (name.to_string(), n))
        .collect();
    // An alias costs its declaration, about 40 bytes.
    names.extend(
        templates
            .into_iter()
            .map(|(name, n)| (format!("std::{name}"), n))
            .filter(|(name, n)| n * (name.len() - 3) > 48),
    );
    names.sort_by(|(a, n), (b, m)| (m * b.len()).cmp(&(n * a.len())).then(a.cmp(b)));
    let mut fresh = 0;
    let mut rename = BTreeMap::new();
    let mut next = None;
    for (name, _) in names {
        let short = next.take().unwrap_or_else(|| loop {
            let s = short_name(fresh);
            fresh += 1;
            if !taken.contains(s.as_str()) && !KEYWORDS.contains(&s.as_str()) {
                break s;
            }
        });
        // A name no longer than the next short one keeps its own.
        if short.len() >= name.len() {
            next = Some(short);
            continue;
        }
        rename.insert(name, short);
    }
    // A runtime name, short now, is written without `cpl::`.
    let bare: Vec<(String, String)> = rename
        .iter()
        .filter(|(name, _)| in_cpl.contains(name.as_str()) && !bare_outside.contains(name.as_str()))
        .map(|(name, short)| (format!("cpl::{name}"), short.clone()))
        .collect();
    rename.extend(bare);
    // `using std::move;` costs its line, and has no other `move` to meet.
    rename.extend(
        std_names
            .into_iter()
            .filter(|(name, n)| !unqualified.contains(name) && n * 5 > 13 + name.len())
            .map(|(name, _)| (format!("std::{name}"), name.to_string())),
    );
    rename
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bare(cpp: &str) -> String {
        compact(cpp, false, &BTreeMap::new())
    }

    #[test]
    fn only_the_spaces_between_tokens_that_would_join_stay() {
        let cpp =
            "    inline ll f(ll a, ll b) {\n        return a - -b + (a < ::std::max(a, b)) ? a++ + +b : x.y;\n    }\n";
        assert_eq!(
            bare(cpp),
            "inline ll f(ll a,ll b){\nreturn a- -b+(a< ::std::max(a,b))?a++ + +b:x.y;\n}\n"
        );
        assert_eq!(
            bare("auto s = u8\"a b\" \"c\";\nchar c = ' ';\nauto d = 1.5e-3 .f;\n"),
            "auto s=u8\"a b\" \"c\";\nchar c=' ';\nauto d=1.5e-3 .f;\n"
        );
    }

    #[test]
    fn an_empty_side_never_needs_a_space() {
        assert!(needs_space("a", "b"));
        assert!(!needs_space("", "b"));
        assert!(!needs_space("a", ""));
    }

    #[test]
    fn the_emitters_names_shorten_but_the_entry_keeps_its_own() {
        assert!(ours("cpl_tie") && ours("cpl_go"));
        assert!(!ours("cpl_main") && !ours("cpl_") && !ours("main"));
    }

    #[test]
    fn short_names_count_up_through_every_length() {
        let names: Vec<String> = [0, 35, 36, 36 + 36 * 36 - 1, 36 + 36 * 36]
            .into_iter()
            .map(short_name)
            .collect();
        assert_eq!(names, ["Q0", "Qz", "Q00", "Qzz", "Q000"]);
    }

    #[test]
    fn comments_blank_lines_and_preprocessor_indentation_go() {
        let cpp = "// header\n\n  #include <vector>\n  // a note\n  int x = 1;  // why\n";
        assert_eq!(
            compact(cpp, true, &BTreeMap::new()),
            "// header\n#include <vector>\nint x=1;\n"
        );
        assert_eq!(bare(cpp), "#include <vector>\nint x=1;\n");
    }

    #[test]
    fn a_line_it_cannot_read_is_only_trimmed() {
        assert_eq!(bare("  auto s = R\"(a  b)\";\n"), "auto s = R\"(a  b)\";\n");
        assert_eq!(bare("  int x = 1; /* a  b */\n"), "int x = 1; /* a  b */\n");
    }

    #[test]
    fn a_fn_the_grader_links_keeps_its_name() {
        let cpp = "namespace prog {\nint f_2(int a_1) { return a_1 + a_1 + a_1; }\nint create_map(int x) { return f_2(x) + f_2(x); }\n}\nint create_map(int x) { return prog::create_map(x) + prog::create_map(x); }\n";
        let names = short_names(cpp);
        assert!(names.contains_key("f_2"), "{names:?}");
        assert!(!names.contains_key("create_map"), "{names:?}");
    }

    #[test]
    fn numbered_names_shorten_but_library_names_keep_theirs() {
        let cpp = "struct Mod_7_1{};\nMod_7_1 Mod_7_1_mul(Mod_7_1 a){return a;}\nint Q0;\nstd::int_fast32_t v_2=Mod_7_1_mul(Mod_7_1{}).x_3;\nuint64_t y;\n";
        let names = short_names(cpp);
        assert_eq!(names.get("Mod_7_1").map(String::as_str), Some("Q1"));
        assert!(names.contains_key("Mod_7_1_mul") && names.contains_key("v_2"));
        let imported = short_names("struct m1_MaxSub{};\nm1_MaxSub a,b,c;\nint m_x,mx_1y;\n");
        assert!(imported.contains_key("m1_MaxSub"), "{imported:?}");
        assert!(!imported.contains_key("m_x"), "{imported:?}");
        let tied = short_names("struct P{auto cpl_tie()const{return 1;}};\nbool f(P a,P b){return a.cpl_tie()<b.cpl_tie();}\n");
        assert!(tied.contains_key("cpl_tie"), "{tied:?}");
        assert!(
            !names.contains_key("int_fast32_t")
                && !names.contains_key("uint64_t")
                && !names.contains_key("Q0")
        );
        assert!(!names.values().any(|short| short == "Q0"), "{names:?}");
        let short = compact(cpp, false, &names);
        assert!(
            short.contains("std::int_fast32_t") && !short.contains("Mod_7_1"),
            "{short}"
        );
    }

    #[test]
    fn a_library_fn_written_often_is_brought_in_unless_the_file_writes_its_name() {
        let cpp = concat!(
            "#include <utility>\n",
            "using ll=long long;\n",
            "ll f(ll a){ll b=std::move(a),c=std::move(b),d=std::move(c);return std::move(d);}\n",
            "ll g(ll a){return std::get<0>(std::tie(a))+std::get<0>(std::tie(a))+x.get();}\n",
        );
        let names = short_names(cpp);
        assert_eq!(names.get("std::move").map(String::as_str), Some("move"));
        assert!(!names.contains_key("std::get"), "{names:?}");
        let out = compact(cpp, false, &names);
        assert!(
            out.contains("using ll=long long;\nusing std::move;\n"),
            "{out}"
        );
        assert!(
            out.contains("{ll b=move(a),c=move(b),d=move(c);return move(d);}"),
            "{out}"
        );
        assert!(out.contains("std::get<0>"), "{out}");
    }

    #[test]
    fn a_shortened_fn_is_called_without_its_namespace() {
        let cpp = concat!(
            "using ll=long long;\n",
            "namespace prog{\n",
            "ll walk_Int_3(std::vector<ll>&v){return v.size();}\n",
            "ll solve(std::vector<ll>&v){return ::prog::walk_Int_3(v)+::prog::walk_Int_3(v)+::prog::run(v);}\n",
            "}\n",
        );
        let names = short_names(cpp);
        let short = &names["walk_Int_3"];
        let out = compact(cpp, false, &names);
        assert!(
            out.contains(&format!("{{return {short}(v)+{short}(v)+::prog::run(v);}}")),
            "{out}"
        );
    }

    #[test]
    fn a_name_on_a_preprocessor_line_keeps_it() {
        assert!(!short_names("#define f_1 2\nint g_2=f_1+f_1+f_1;\n").contains_key("f_1"));
    }

    #[test]
    fn runtime_names_shorten_unless_written_as_members_or_beside_locals() {
        let cpp = concat!(
            "using ll=long long;\n",
            "namespace cpl{\n",
            "inline void need_digit(){}\n",
            "inline void skip_space(){}\n",
            "inline int steps(){return 0;}\n",
            "inline int value(){return std::is_same<int,int>::value;}\n",
            "}\n",
            "namespace prog{\n",
            "void run(){cpl::need_digit();cpl::need_digit();cpl::need_digit();cpl::skip_space();p.skip_space();}\n",
            "int f(){int steps=cpl::steps()+cpl::steps()+cpl::steps();return steps+cpl::value()+cpl::value();}\n",
            "}\n",
        );
        let names = short_names(cpp);
        let short = &names["need_digit"];
        assert_eq!(names.get("cpl::need_digit"), Some(short));
        assert!(!names.contains_key("skip_space"), "{names:?}");
        assert!(!names.contains_key("value"), "{names:?}");
        assert!(
            names.contains_key("steps") && !names.contains_key("cpl::steps"),
            "{names:?}"
        );
        let out = compact(cpp, false, &names);
        assert!(out.contains(&format!("(){{{short}();{short}();")), "{out}");
        assert!(
            out.contains("using ll=long long;\nnamespace cpl{}using namespace cpl;\n"),
            "{out}"
        );
    }

    #[test]
    fn a_library_template_written_often_gets_a_short_alias() {
        let uses = "std::vector<ll> a;".repeat(8);
        let cpp = format!("using ll=long long;\nstd::vector<ll> f();\n{uses}\nstd::vector x;\n");
        let names = short_names(&cpp);
        let alias = &names["std::vector"];
        let out = compact(&cpp, false, &names);
        assert!(
            out.contains(&format!(
                "template<class...T>using {alias}=std::vector<T...>;\n"
            )),
            "{out}"
        );
        assert!(out.contains(&format!("{alias}<ll>f();")), "{out}");
        assert!(out.contains("std::vector x;"), "{out}");
    }

    #[test]
    fn a_name_no_longer_than_the_next_short_one_does_not_stop_the_rest() {
        let cpp = format!("{}{}", "int ab_1=0;".repeat(40), "int long_name_2=0;");
        let names = short_names(&cpp);
        assert!(names.contains_key("long_name_2"), "{names:?}");
    }
}
