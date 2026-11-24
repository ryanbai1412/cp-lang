//! The text around the fns: the file's header, inline hints, which fns
//! are declared ahead, and list forms nothing calls.

use std::ops::Range;

/// GCC at -O2 inlines a fn not declared `inline` only when it is tiny, so
/// the many small helpers a library method lowers to (a node's size, one
/// tag push) stay calls. A short, non-recursive, non-template definition
/// gets the hint; recursion and long bodies are left to the compiler.
pub(super) fn inline_hint(def: &str, signature: &str, name: &str) -> bool {
    const MAX_LINES: usize = 16;
    let body = def
        .trim_start_matches('\n')
        .split_once('\n')
        .map_or("", |(_, b)| b);
    let stmts = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && *l != "}" && *l != "} else {" && !l.starts_with("cpl::fail("))
        .count();
    !signature.starts_with("template")
        && !signature.contains(" main(")
        && stmts <= MAX_LINES
        && !body.contains(&format!("{name}("))
}

/// The comment that opens the file: what it is, and the source it was
/// built from, so that whoever reads the C++ can read the program.
pub(super) fn header(name: &str, source: Option<&str>, stamp: Option<&str>) -> String {
    let version = match (option_env!("CPL_CHANNEL"), option_env!("CPL_COMMIT")) {
        (Some(channel), Some(commit)) => {
            format!(
                "{} ({channel} {})",
                env!("CARGO_PKG_VERSION"),
                &commit[..commit.len().min(8)]
            )
        }
        _ => env!("CARGO_PKG_VERSION").to_string(),
    };
    let stamp = stamp
        .map(|stamp| format!("// For {stamp}.\n"))
        .unwrap_or_default();
    let Some(source) = source else {
        return format!(
            "// {name}: C++17 written by cpl {version} (https://cp-lang.org) from CP Lang source, which is left out\n\
             // here to keep this file within the size a judge accepts.\n{stamp}\n"
        );
    };
    let mut out =
        format!("// {name}: C++17 written by cpl {version} (https://cp-lang.org) from the CP Lang source below.\n{stamp}//\n");
    for line in source.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            out.push_str("//\n");
        } else if line.ends_with('\\') {
            // A `\` at the end would join the next line to this comment.
            out.push_str(&format!("//     {line} .\n"));
        } else {
            out.push_str(&format!("//     {line}\n"));
        }
    }
    out.push('\n');
    out
}

/// A fn's declaration, and where its definition starts in the text of
/// the fns.
pub(super) struct Proto {
    pub(super) name: String,
    pub(super) line: String,
    pub(super) at: usize,
    /// Named by text that is written elsewhere: a struct's operators, the
    /// judge's side.
    pub(super) always: bool,
}

/// True when `text` has `name` as a whole word.
pub(super) fn names(text: &str, name: &str) -> bool {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    text.match_indices(name).any(|(at, _)| {
        let before = text.as_bytes()[..at].last().copied();
        let after = text.as_bytes().get(at + name.len()).copied();
        !before.is_some_and(word) && !after.is_some_and(word)
    })
}

/// A `yields` fn every loop walks through its `cpl_each_f` form is called
/// by nothing in its list form, so that form is taken back out of `fns`,
/// its declaration and its chunk. `forms` holds each list form's name, its
/// range in `fns` and its chunk; `elsewhere` is the rest of the file.
pub(super) fn drop_unread_list_forms(
    fns: &mut String,
    protos: &mut Vec<Proto>,
    chunks: &mut [(crate::Part, String, String)],
    forms: Vec<(String, Range<usize>, usize)>,
    elsewhere: &[&str],
) {
    for (name, range, chunk) in forms.into_iter().rev() {
        let called = [&fns[..range.start], &fns[range.end..]]
            .into_iter()
            .chain(elsewhere.iter().copied())
            .any(|text| names(text, &name));
        if called {
            continue;
        }
        let text: String = fns.drain(range.clone()).collect();
        protos.retain(|p| !(p.name == name && p.at == range.start));
        for p in protos.iter_mut() {
            if p.at >= range.end {
                p.at -= range.len();
            }
        }
        chunks[chunk].2 = chunks[chunk].2.replacen(&text, "", 1);
    }
}

/// An `@cpp` fn: its body as written, after it names each param as the
/// source does, even one renamed for a global it shadows.
pub(super) fn cpp_fn_def<'n>(
    signature: &str,
    params: impl Iterator<Item = (&'n str, &'n str)>,
    code: &str,
) -> String {
    let aliases: String = params
        .filter(|(source, cpp)| source != cpp)
        .map(|(source, cpp)| format!("    auto& {source} = {cpp};\n"))
        .collect();
    format!("{signature} {{\n{aliases}    {code}\n}}\n")
}
