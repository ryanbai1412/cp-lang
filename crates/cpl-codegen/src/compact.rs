//! The last ways a file is made to fit its size bar, after the comments,
//! the source and half the indentation: no space the compiler does not
//! need, then shorter names for the instances codegen numbers. Each works
//! a line at a time, so a part of the file shrinks as it does in the file.

use std::collections::{BTreeMap, BTreeSet};

/// Two characters that read as one token when nothing is between them.
const JOINED: &[&str] = &[
    "++", "--", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "->",
    "::", "//", "/*", "*/", ".*", "<:", ":>", "<%", "%>", "%:", "##", "..",
];

fn word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The tokens of a line of C++ and whether space came before each, up to a
/// `//` comment. `None` for what this does not read: a block comment, a raw
/// string, or a literal the line does not close.
fn tokens(line: &str) -> Option<Vec<(bool, &str)>> {
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
            if c == b'"' && out.last().is_some_and(|&(_, t): &(bool, &str)| t.ends_with('R')) && !space {
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
                let exponent = if hex { matches!(d, b'p' | b'P') } else { matches!(d, b'e' | b'E') };
                if exponent && matches!(b.get(at + 1), Some(b'+' | b'-')) {
                    at += 2;
                } else if word(d) || d == b'.' || d == b'\'' && b.get(at + 1).is_some_and(|&n| word(n)) {
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
fn needs_space(a: &str, b: &str) -> bool {
    let (x, y) = (*a.as_bytes().last().unwrap(), b.as_bytes()[0]);
    let quote = |c: u8| c == b'"' || c == b'\'';
    (word(x) || quote(x)) && (word(y) || quote(y))
        || a.as_bytes()[0].is_ascii_digit() && y == b'.'
        || y == b'.' && b.len() > 1 && b.as_bytes()[1].is_ascii_digit()
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
    for (space, token) in tokens {
        let token = rename.get(token).map_or(token, String::as_str);
        if let Some(last) = last {
            if space && needs_space(last, token) {
                out.push(' ');
            }
        }
        out.push_str(token);
        last = Some(token);
    }
    match out.is_empty() {
        true => out,
        false => out + "\n",
    }
}

/// `cpp` with every line compacted and `rename` applied; the comments at
/// the top stay when `header`.
pub(crate) fn compact(cpp: &str, header: bool, rename: &BTreeMap<String, String>) -> String {
    let mut leading = header;
    let mut out = String::with_capacity(cpp.len());
    for line in cpp.split_inclusive('\n') {
        leading &= line.starts_with("//");
        match leading {
            true => out.push_str(line),
            false => out.push_str(&compact_line(line, rename)),
        }
    }
    out
}

/// A name codegen numbered: an instance (`Mod_998244353_9_mul`), a local
/// made unique (`i_2`); a part of it after the first `_` is all digits.
/// Never a name from a library header, but for the few the file reaches as
/// `std::`, which keep theirs.
fn numbered(name: &str) -> bool {
    name.as_bytes()[0].is_ascii_alphabetic()
        && name.bytes().any(|c| c.is_ascii_lowercase())
        && name.split('_').skip(1).any(|part| !part.is_empty() && part.bytes().all(|c| c.is_ascii_digit()))
}

const KEYWORDS: &[&str] = &["do", "if", "or", "and", "asm", "for", "int", "new", "not", "try", "xor"];

/// Short names for the numbered names in `cpp`, the most bytes saved
/// first: `Q0`, `Qa`, ..., then three characters. A name written after
/// `std::` or on a preprocessor line keeps its name.
pub(crate) fn short_names(cpp: &str) -> BTreeMap<String, String> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    let mut taken: BTreeSet<&str> = BTreeSet::new();
    let mut keep: BTreeSet<&str> = BTreeSet::new();
    for line in cpp.lines() {
        let code = line.trim_start();
        let Some(tokens) = tokens(code).filter(|_| !code.starts_with('#')) else {
            for t in code.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                keep.insert(t);
            }
            continue;
        };
        for (i, &(_, t)) in tokens.iter().enumerate() {
            if !word(t.as_bytes()[0]) {
                continue;
            }
            taken.insert(t);
            let after_std = i >= 3 && tokens[i - 1].1 == ":" && tokens[i - 2].1 == ":" && tokens[i - 3].1 == "std";
            if after_std {
                keep.insert(t);
            } else if numbered(t) {
                *count.entry(t).or_default() += 1;
            }
        }
    }
    let mut names: Vec<(&str, usize)> = count.into_iter().filter(|(name, _)| !keep.contains(name)).collect();
    names.sort_by_key(|&(name, n)| (std::cmp::Reverse(n * name.len()), name));
    const DIGITS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut fresh = (0..).map(|i: usize| {
        let mut name = vec![b'Q'];
        let mut i = i;
        let mut rest = Vec::new();
        loop {
            rest.push(DIGITS[i % 36]);
            i /= 36;
            if i == 0 {
                break;
            }
            i -= 1;
        }
        name.extend(rest.iter().rev());
        String::from_utf8(name).unwrap()
    });
    let mut rename = BTreeMap::new();
    for (name, _) in names {
        let short = fresh.by_ref().find(|s| !taken.contains(s.as_str()) && !KEYWORDS.contains(&s.as_str())).unwrap();
        if short.len() >= name.len() {
            break;
        }
        rename.insert(name.to_string(), short);
    }
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
        assert_eq!(bare(cpp), "inline ll f(ll a,ll b){\nreturn a- -b+(a< ::std::max(a,b))?a++ + +b:x.y;\n}\n");
        assert_eq!(
            bare("auto s = u8\"a b\" \"c\";\nchar c = ' ';\nauto d = 1.5e-3 .f;\n"),
            "auto s=u8\"a b\" \"c\";\nchar c=' ';\nauto d=1.5e-3 .f;\n"
        );
    }

    #[test]
    fn comments_blank_lines_and_preprocessor_indentation_go() {
        let cpp = "// header\n\n  #include <vector>\n  // a note\n  int x = 1;  // why\n";
        assert_eq!(compact(cpp, true, &BTreeMap::new()), "// header\n#include <vector>\nint x=1;\n");
        assert_eq!(bare(cpp), "#include <vector>\nint x=1;\n");
    }

    #[test]
    fn a_line_it_cannot_read_is_only_trimmed() {
        assert_eq!(bare("  auto s = R\"(a  b)\";\n"), "auto s = R\"(a  b)\";\n");
        assert_eq!(bare("  int x = 1; /* a  b */\n"), "int x = 1; /* a  b */\n");
    }

    #[test]
    fn numbered_names_shorten_but_library_names_keep_theirs() {
        let cpp = "struct Mod_7_1{};\nMod_7_1 Mod_7_1_mul(Mod_7_1 a){return a;}\nint Q0;\nstd::int_fast32_t v_2=Mod_7_1_mul(Mod_7_1{}).x_3;\nuint64_t y;\n";
        let names = short_names(cpp);
        assert_eq!(names.get("Mod_7_1").map(String::as_str), Some("Q1"));
        assert!(names.contains_key("Mod_7_1_mul") && names.contains_key("v_2"));
        assert!(!names.contains_key("int_fast32_t") && !names.contains_key("uint64_t") && !names.contains_key("Q0"));
        assert!(!names.values().any(|short| short == "Q0"), "{names:?}");
        let short = compact(cpp, false, &names);
        assert!(short.contains("std::int_fast32_t") && !short.contains("Mod_7_1"), "{short}");
    }

    #[test]
    fn a_name_on_a_preprocessor_line_keeps_it() {
        assert!(!short_names("#define f_1 2\nint g_2=f_1+f_1+f_1;\n").contains_key("f_1"));
    }
}
