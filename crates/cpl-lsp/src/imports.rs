//! Organize imports: the file's leading `use` lines merged by module,
//! sorted, and without names the file never uses.

use cpl_diag::Span;

/// The leading `use` block and its organized text, if that changes it.
pub fn organize(text: &str) -> Option<(Span, String)> {
    let mut start = None;
    let mut end = 0;
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") {
            start.get_or_insert(at);
            end = at + line.len();
        } else if start.is_some() && !trimmed.is_empty() {
            break;
        } else if start.is_none() && !(trimmed.is_empty() || trimmed.starts_with('#')) {
            return None;
        }
        at += line.len();
    }
    let start = start?;
    let block = &text[start..end];
    let rest = format!("{}{}", &text[..start], &text[end..]);
    let mut merged: Vec<(String, Vec<String>)> = Vec::new();
    let mut verbatim: Vec<String> = Vec::new();
    for line in block.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let body = line["use ".len()..].trim();
        match body.split_once(".{") {
            Some((path, names)) if names.ends_with('}') && !path.contains(' ') => {
                let names = names[..names.len() - 1].split(',').map(str::trim);
                let slot = match merged.iter().position(|(p, _)| p == path) {
                    Some(i) => i,
                    None => {
                        merged.push((path.to_string(), Vec::new()));
                        merged.len() - 1
                    }
                };
                merged[slot]
                    .1
                    .extend(names.filter(|n| !n.is_empty()).map(String::from));
            }
            _ => verbatim.push(line.to_string()),
        }
    }
    let mut lines = verbatim;
    for (path, mut names) in merged {
        names.sort();
        names.dedup();
        names.retain(|name| used(&rest, name));
        if !names.is_empty() {
            lines.push(format!("use {path}.{{{}}}", names.join(", ")));
        }
    }
    lines.sort_by(|a, b| module(a).cmp(module(b)).then(a.cmp(b)));
    lines.dedup();
    let organized: String = lines.iter().map(|l| format!("{l}\n")).collect();
    (organized != block).then(|| (Span::new(start as u32, end as u32), organized))
}

fn module(line: &str) -> &str {
    let body = &line["use ".len()..];
    body.split([' ', '{'])
        .next()
        .unwrap_or(body)
        .trim_end_matches('.')
}

/// Whether `name` appears as a whole word in `text`.
fn used(text: &str, name: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(name).any(|(i, _)| {
        let before = text[..i].chars().next_back().is_none_or(|c| !ident(c));
        let after = text[i + name.len()..]
            .chars()
            .next()
            .is_none_or(|c| !ident(c));
        before && after
    })
}

#[cfg(test)]
mod tests {
    use super::organize;

    #[test]
    fn imports_merge_sort_and_lose_unused_names() {
        let text = "\
use std.segtree.{SegTree}
use std.deque.{Deque, Unused}
use std.segtree.{SegTree, Max}
let d = Deque<Int>()
let t = SegTree(Max, [1])
";
        let (span, organized) = organize(text).expect("a change");
        assert_eq!(span.start, 0);
        assert_eq!(
            organized,
            "use std.deque.{Deque}\nuse std.segtree.{Max, SegTree}\n"
        );
        let applied = format!("{organized}{}", &text[span.end as usize..]);
        assert_eq!(organize(&applied), None, "organizing is idempotent");
    }

    #[test]
    fn a_wholly_unused_import_goes_and_other_forms_stay() {
        let text = "use std.rand.{Rng}\nuse std.io.*\nprint(1)\n";
        let (_, organized) = organize(text).expect("a change");
        assert_eq!(organized, "use std.io.*\n");
        assert_eq!(organize("print(1)\n"), None);
        // Not a leading block: left alone.
        assert_eq!(organize("let a = 1\nuse std.rand.{Rng}\n"), None);
    }
}
