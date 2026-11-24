//! The runtime pays only for what the program reaches of each piece: a
//! definition in `namespace cpl` that nothing reached names is left out,
//! whole fns and structs as well as one-liners, with the comment above it.
//! Anything without a name to look for stays: an operator (called without
//! its name), a global, a `using`. Each side of a `#if` is read alike.

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
/// nothing outside them reaches.
pub(crate) fn unused(file: &str) -> Vec<Range<usize>> {
    const OPEN: &str = "namespace cpl {\n";
    let Some(start) = file.find(OPEN).map(|at| at + OPEN.len()) else {
        return Vec::new();
    };
    let end = file[start..]
        .find("}  // namespace cpl\n")
        .map_or(file.len(), |at| start + at);
    let items = items(file, start..end);
    let defined: BTreeSet<&str> = items.iter().filter(|i| i.primary).map(|i| i.name).collect();
    let mut needed = BTreeSet::new();
    let mut last = 0;
    for item in &items {
        words(&file[last..item.at.start], &mut needed);
        last = item.at.end;
    }
    words(&file[last..], &mut needed);
    let mut kept = vec![false; items.len()];
    loop {
        let mut grew = false;
        for (keep, item) in kept.iter_mut().zip(&items) {
            if !*keep && (needed.contains(item.name) || !defined.contains(item.name)) {
                *keep = true;
                grew = true;
                words(&file[item.at.clone()], &mut needed);
            }
        }
        if !grew {
            break;
        }
    }
    items
        .into_iter()
        .zip(kept)
        .filter(|(_, k)| !k)
        .map(|(i, _)| i.at)
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
        if t.starts_with('#') || t.is_empty() {
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
    (!name.is_empty() && !before[..at].ends_with("::")).then_some((name, true))
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

/// Adds the names `text` mentions to `out`: not in a comment, and not a
/// member (`x.first`, `p->first`), which no runtime definition is.
fn words<'a>(text: &'a str, out: &mut BTreeSet<&'a str>) {
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
            let member = code[..start].ends_with('.') || code[..start].ends_with("->");
            if !member {
                out.insert(&code[start..i]);
            }
        }
    }
}
