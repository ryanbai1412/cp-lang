//! What a browser build of cpl-check embeds of `std`: shared by build.rs,
//! which writes it, and by the test that holds it to its gzipped cap.

use crate::std_files::{call_names, outline, without_tests, EMBEDDED};

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
/// `EMBEDDED` modules whole, without comments, and the others' outlines.
pub fn browser_index(name: &str, text: &str) -> String {
    let text = without_tests(text);
    if EMBEDDED.contains(&name) {
        without_comments(&text)
    } else {
        outline(&text)
    }
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
