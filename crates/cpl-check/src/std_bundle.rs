//! What a browser build of cpl-check embeds of `std`: shared by build.rs,
//! which writes it, and by the test that holds it to its gzipped cap.

use crate::std_files::{call_names, nesting, outline, without_tests, EMBEDDED};

/// The browser build blanks whole-line comments: nothing it runs reads std's
/// docs, and they are half of std's compressed size. Blank lines keep every
/// line number, so std diagnostics point at the same line on every target.
pub fn without_comments(text: &str) -> String {
    text.split_inclusive('\n')
        .map(|line| {
            if line.trim_start().starts_with("//") {
                if line.ends_with('\n') {
                    "\n"
                } else {
                    ""
                }
            } else {
                line
            }
        })
        .collect()
}

/// What a browser build's index carries for module `name` of `text`: the
/// `EMBEDDED` modules whole, without comments, and the others' outlines
/// without their private fns, each `todo` body on its header's line.
pub fn browser_index(name: &str, text: &str) -> String {
    let text = without_tests(text);
    if EMBEDDED.contains(&name) {
        without_comments(&text)
    } else {
        with_bodies_inline(&without_private_fns(&outline(&text)))
    }
}

/// `text` with each fn whose return type is written and whose body is the
/// outline's `todo` on the next line given it as `= todo` on its header's
/// line, as the outline writes a one-line body: the same declaration, a
/// line lighter. The `todo` line stays, blank.
pub fn with_bodies_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut lines = text.split_inclusive('\n').peekable();
    let indent = |l: &str| l.len() - l.trim_start().len();
    while let Some(line) = lines.next() {
        let code = line.trim_end_matches(['\n', '\r']);
        let typed = code.strip_suffix(':').filter(|head| head.contains("->"));
        let body =
            typed.and_then(|_| lines.next_if(|l| l.trim() == "todo" && indent(l) > indent(code)));
        match (typed, body) {
            (Some(head), Some(body)) => {
                out.push_str(head);
                out.push_str(" = todo");
                out.push_str(&line[code.len()..]);
                out.push_str(&body[body.trim_end_matches(['\n', '\r']).len()..]);
            }
            _ => out.push_str(line),
        }
    }
    out
}

/// `text` with each non-`pub` fn blanked, header and body: only the
/// module's own bodies name one (R11.1.4), and the host gives the module's
/// text before a check reads those. A trait's members and a trait impl's
/// stay, since the trait decides their visibility. Every line stays, blank.
pub fn without_private_fns(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_trait = false;
    let mut dropped: Option<usize> = None;
    let mut open = 0i32;
    for line in text.split_inclusive('\n') {
        let code = line.trim_end_matches(['\n', '\r']);
        let trimmed = code.trim_start();
        let indent = code.len() - trimmed.len();
        if let Some(at) = dropped {
            if open > 0 || trimmed.is_empty() || indent > at {
                open += nesting(code);
                out.push_str(&line[code.len()..]);
                continue;
            }
            dropped = None;
        }
        if indent == 0 && !trimmed.is_empty() {
            in_trait = trait_block(trimmed);
        }
        if !in_trait && private_fn(trimmed) {
            dropped = Some(indent);
            open = nesting(code).max(0);
            out.push_str(&line[code.len()..]);
            continue;
        }
        out.push_str(line);
    }
    out
}

/// Whether a top-level `line` opens a trait or a trait impl.
fn trait_block(line: &str) -> bool {
    let line = line.strip_prefix("pub ").unwrap_or(line);
    line.starts_with("trait ") || (line.starts_with("impl") && line.contains(" for "))
}

/// Whether `line` declares a fn without `pub`, after any `@attr`s.
fn private_fn(mut line: &str) -> bool {
    while let Some(rest) = line.strip_prefix('@') {
        let Some((_, after)) = rest.split_once(' ') else {
            return false;
        };
        line = after;
    }
    line.starts_with("fn ")
}

/// How often the `sources`, doc examples included, write `.name`, for the
/// names written at least twice: a line of counts (blank for none), one for
/// each of `call_names(indexed)` in order, then `name count` for each name
/// no indexed text has. The index already carries the names; written again
/// beside their counts, they were most of what this table weighs.
pub fn call_table<'a>(
    sources: impl IntoIterator<Item = &'a str>,
    indexed: impl IntoIterator<Item = &'a str>,
) -> String {
    let mut counts: std::collections::BTreeMap<&str, u32> = Default::default();
    for text in sources {
        let bytes = text.as_bytes();
        for (at, _) in text.match_indices('.') {
            if at > 0 && bytes[at - 1] == b'.' {
                continue;
            }
            let len = text[at + 1..]
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                .count();
            let name = &text[at + 1..at + 1 + len];
            if name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_') {
                *counts.entry(name).or_default() += 1;
            }
        }
    }
    counts.retain(|_, n| *n >= 2);
    let names = call_names(indexed);
    let mut out = names
        .iter()
        .map(|name| counts.remove(name).map_or(String::new(), |n| n.to_string()))
        .collect::<Vec<_>>()
        .join(" ");
    for (name, count) in counts {
        out.push_str(&format!("\n{name} {count}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{with_bodies_inline, without_private_fns};

    #[test]
    fn a_todo_body_under_a_typed_header_moves_onto_it() {
        let text = "pub fn f(a: Int) -> Int:\n    todo\npub fn g(\n    a: Int,\n) -> Int:\n    todo\npub fn h(self):\n    todo\npub struct S:\n    pub fn m(self) -> ?Int:\n        todo\npub fn k() -> Int = todo\n";
        let want = "pub fn f(a: Int) -> Int = todo\n\npub fn g(\n    a: Int,\n) -> Int = todo\n\npub fn h(self):\n    todo\npub struct S:\n    pub fn m(self) -> ?Int = todo\n\npub fn k() -> Int = todo\n";
        assert_eq!(with_bodies_inline(text), want);
    }

    #[test]
    fn private_fns_are_blanked_and_every_line_stays() {
        let text = "pub struct S:\n    @get fn hid(self) -> Int = todo\n    @get pub fn shown(self) -> Int = todo\n    fn helper(\n        self,\n    ):\n        todo\n\n    pub fn api(self):\n        todo\nfn top(a: Int) -> Int = todo\ntrait T:\n    fn need(self) -> Int\nimpl T for S:\n    fn need(self) -> Int = todo\npub fn last() -> Int = todo\n";
        let want = "pub struct S:\n\n    @get pub fn shown(self) -> Int = todo\n\n\n\n\n\n    pub fn api(self):\n        todo\n\ntrait T:\n    fn need(self) -> Int\nimpl T for S:\n    fn need(self) -> Int = todo\npub fn last() -> Int = todo\n";
        assert_eq!(without_private_fns(text), want);
    }
}
