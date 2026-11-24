//! The runtime pays only for what the program reaches of each piece: a
//! definition in `namespace cpl` that nothing reached names is left out,
//! whole fns and structs as well as one-liners, with the comment above it.
//! In a struct that stays, so does each member fn something calls. Anything
//! without a name to look for stays: an operator (called without its name),
//! a constructor, a global, a `using`. Each side of a `#if` is read alike.

use std::collections::BTreeSet;
use std::ops::Range;

/// One definition the file can do without when nothing names it.
struct Item<'a> {
    at: Range<usize>,
    name: &'a str,
    /// `false` for a specialization, which goes with its primary template.
    primary: bool,
}

/// The byte ranges of `file`, in order, holding runtime definitions that
/// nothing outside them reaches: top-level ones, and the member fns of a
/// struct that stays that nothing calls.
pub(crate) fn unused(file: &str) -> Vec<Range<usize>> {
    const OPEN: &str = "namespace cpl {\n";
    let Some(start) = file.find(OPEN).map(|at| at + OPEN.len()) else {
        return Vec::new();
    };
    let end = file[start..]
        .find("}  // namespace cpl\n")
        .map_or(file.len(), |at| start + at);
    let tops = items(file, start..end);
    let defined: BTreeSet<&str> = tops.iter().filter(|i| i.primary).map(|i| i.name).collect();
    // Each top-level item, then the members of each struct, with its parent.
    let mut all: Vec<(Item, Option<usize>)> = Vec::new();
    for top in tops {
        let body = body(file, &top.at);
        let k = all.len();
        all.push((top, None));
        if let Some(body) = body {
            let members = items(file, body);
            all.extend(members.into_iter().map(|m| (m, Some(k))));
        }
    }
    // `names` are what a top-level item answers to; `calls` also count members.
    let (mut names, mut calls) = (BTreeSet::new(), BTreeSet::new());
    let mut last = 0;
    for (item, _) in all.iter().filter(|(_, parent)| parent.is_none()) {
        words(&file[last..item.at.start], &mut names, &mut calls);
        last = item.at.end;
    }
    words(&file[last..], &mut names, &mut calls);
    let mut kept = vec![false; all.len()];
    loop {
        let mut grew = false;
        for k in 0..all.len() {
            let (item, parent) = &all[k];
            let keep = match parent {
                None => names.contains(item.name) || !defined.contains(item.name),
                Some(p) => {
                    kept[*p]
                        && (calls.contains(item.name)
                            || item.name == all[*p].0.name
                            || FOUND_BY_NAME.contains(&item.name))
                }
            };
            if kept[k] || !keep {
                continue;
            }
            kept[k] = true;
            grew = true;
            // A struct's own words leave out its members', which count once kept.
            let mut from = item.at.start;
            for (member, _) in all.iter().filter(|(_, p)| *p == Some(k)) {
                words(&file[from..member.at.start], &mut names, &mut calls);
                from = member.at.end;
            }
            words(&file[from..item.at.end], &mut names, &mut calls);
        }
        if !grew {
            break;
        }
    }
    let mut out: Vec<Range<usize>> = (0..all.len())
        .filter(|&k| !kept[k] && all[k].1.is_none_or(|p| kept[p]))
        .map(|k| all[k].0.at.clone())
        .collect();
    out.sort_by_key(|at| at.start);
    out
}

/// Member names the language or the library calls on its own: a range
/// `for`, `std::size`, a structured binding, `std::back_inserter`.
const FOUND_BY_NAME: &[&str] = &[
    "begin",
    "end",
    "rbegin",
    "rend",
    "cbegin",
    "cend",
    "size",
    "empty",
    "data",
    "get",
    "swap",
    "push_back",
    "insert",
    "emplace_back",
    "front",
    "back",
];

/// The lines inside the braces of a struct written over several lines.
fn body(file: &str, at: &Range<usize>) -> Option<Range<usize>> {
    let text = &file[at.clone()];
    let mut head = 0;
    let mut depth = 0;
    for line in text.split_inclusive('\n') {
        let code = line.trim_start();
        if depth == 0 && !code.starts_with("//") {
            let opens = braces(line);
            if opens == 1 && line.trim_end().ends_with('{') {
                let kind = after_template(code).unwrap_or(code);
                let starts = kind.starts_with("struct ") || kind.starts_with("class ");
                if !starts {
                    return None;
                }
                let last = text.trim_end_matches('\n').rfind('\n')? + 1;
                return (head + line.len() <= last)
                    .then(|| at.start + head + line.len()..at.start + last);
            }
            depth += opens;
        }
        head += line.len();
    }
    None
}

/// What follows a `template <...>` head in `code`, or None when `code` has
/// no template head: a `class` in the parameter list is no struct.
fn after_template(code: &str) -> Option<&str> {
    let rest = code.strip_prefix("template")?.trim_start();
    let mut depth = 0;
    let close = rest.bytes().position(|c| {
        depth += (c == b'<') as i32 - (c == b'>') as i32;
        c == b'>' && depth == 0
    })?;
    Some(rest[close + 1..].trim_start())
}

/// The names the top-level definitions in namespace `ns` of `file` define,
/// compacted or not.
pub(crate) fn defined_in<'a>(file: &'a str, ns: &str) -> Vec<&'a str> {
    let mut at = 0;
    let mut start = None;
    let mut depth = 0;
    for line in file.split_inclusive('\n') {
        let begin = at;
        at += line.len();
        match start {
            None => {
                let t = line.trim();
                if t.strip_prefix("namespace ")
                    .and_then(|r| r.strip_suffix('{'))
                    .map(str::trim)
                    == Some(ns)
                {
                    start = Some(at);
                    depth = 1;
                }
            }
            Some(from) => {
                depth += braces(line);
                if depth <= 0 {
                    return items(file, from..begin)
                        .into_iter()
                        .map(|i| i.name)
                        .collect();
                }
            }
        }
    }
    Vec::new()
}

/// The names the top-level definitions in `code` define.
pub(crate) fn defined(code: &str) -> Vec<&str> {
    items(code, 0..code.len())
        .into_iter()
        .map(|i| i.name)
        .collect()
}

/// `text` without the byte ranges `cut`, which are in order and apart.
pub(crate) fn without(text: &str, cut: &[Range<usize>]) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut last = 0;
    for at in cut {
        kept.push_str(&text[last..at.start]);
        last = at.end;
    }
    kept + &text[last..]
}

/// The named top-level definitions in `file[region]`.
fn items(file: &str, region: Range<usize>) -> Vec<Item<'_>> {
    let mut items = Vec::new();
    let mut depth = 0i32;
    let mut comment: Option<usize> = None;
    let mut open: Option<(usize, usize)> = None;
    let mut at = region.start;
    for line in file[region].split_inclusive('\n') {
        let begin = at;
        at += line.len();
        let t = line.trim();
        if let Some((from, head)) = open {
            depth += braces(line);
            if depth == 0 && (t.ends_with('}') || t.ends_with(';')) {
                open = None;
                items.extend(item(file, from..at, head));
            }
            continue;
        }
        if t.starts_with('#') || t.is_empty() || t.ends_with(':') && !t.ends_with("::") {
            comment = None;
            continue;
        }
        if t.starts_with("//") {
            comment.get_or_insert(begin);
            continue;
        }
        let from = comment.take().unwrap_or(begin);
        depth = braces(line);
        if depth != 0 || !(t.ends_with('}') || t.ends_with(';')) {
            open = Some((from, begin));
        } else {
            items.extend(item(file, from..at, begin));
        }
    }
    items
}

fn item(file: &str, at: Range<usize>, head: usize) -> Option<Item<'_>> {
    let text = &file[head..at.end];
    let (name, primary) = name(&text[..text.find('{').unwrap_or(text.len())])?;
    Some(Item { at, name, primary })
}

/// The name a definition's head (the text before its body) defines, and
/// whether it is a primary template rather than a specialization.
fn name(head: &str) -> Option<(&str, bool)> {
    let mut h = head.trim_start();
    let mut special = false;
    if let Some(rest) = h.strip_prefix("template") {
        let rest = rest.trim_start();
        let mut depth = 0;
        let close = rest.bytes().position(|c| {
            depth += (c == b'<') as i32 - (c == b'>') as i32;
            c == b'>' && depth == 0
        })?;
        special = rest[..=close].replace(' ', "") == "<>";
        h = rest[close + 1..].trim_start();
    }
    for word in ["inline ", "static ", "constexpr "] {
        h = h.strip_prefix(word).unwrap_or(h);
    }
    if let Some(rest) = h
        .strip_prefix("struct ")
        .or_else(|| h.strip_prefix("class "))
    {
        let end = rest.bytes().position(|c| !word(c)).unwrap_or(rest.len());
        let name = &rest[..end];
        let partial = rest[end..].trim_start().starts_with('<');
        return (!name.is_empty()).then_some((name, !special && !partial));
    }
    let paren = h.find('(')?;
    let before = h[..paren].trim_end();
    if before.contains('=') || before.contains("operator") || h.starts_with("using ") {
        return None;
    }
    let at = before.bytes().rposition(|c| !word(c))? + 1;
    let name = &before[at..];
    let named = !name.is_empty()
        && !before[..at].ends_with("::")
        && !matches!(
            name,
            "decltype" | "sizeof" | "alignof" | "alignas" | "static_assert" | "noexcept"
        );
    named.then_some((name, true))
}

fn word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The change in brace depth across `line`, outside literals and comments.
fn braces(line: &str) -> i32 {
    let b = line.as_bytes();
    let (mut depth, mut i, mut quote) = (0, 0, None);
    while i < b.len() {
        match (quote, b[i]) {
            (Some(_), b'\\') => i += 1,
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => quote = Some(b[i]),
            (None, b'/') if b.get(i + 1) == Some(&b'/') => break,
            (None, b'{') => depth += 1,
            (None, b'}') => depth -= 1,
            _ => {}
        }
        i += 1;
    }
    depth
}

/// Adds the names `text` mentions, not in a comment, to `calls`, and to
/// `names` unless written as a member (`x.first`, `p->first`), which no
/// top-level runtime definition is.
fn words<'a>(text: &'a str, names: &mut BTreeSet<&'a str>, calls: &mut BTreeSet<&'a str>) {
    for line in text.lines() {
        let code = line.find("//").map_or(line, |at| &line[..at]);
        let b = code.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if !word(b[i]) || b[i].is_ascii_digit() {
                i += 1;
                continue;
            }
            let start = i;
            while i < b.len() && word(b[i]) {
                i += 1;
            }
            calls.insert(&code[start..i]);
            let member = code[..start].ends_with('.') || code[..start].ends_with("->");
            if !member {
                names.insert(&code[start..i]);
            }
        }
    }
}
