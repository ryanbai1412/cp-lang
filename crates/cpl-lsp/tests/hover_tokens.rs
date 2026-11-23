//! Hover on every token: every hill-climb port (`corpus/hillclimb.json`)
//! opened through `Server`, and hover asked at every identifier and every
//! token of written type syntax (the `[`, `]`, `..` and `?` of
//! `[0..n]?Int`, the brackets of `[n].filled(0)`). Each kind's count of
//! tokens with a non-empty hover is a ratchet that only goes up; the tokens
//! without one are printed, most common first, as the backlog.
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r11-ask-the-analysis`
//! and `handbook/guidelines/commands-checklist.md#r14-measured-over-the-corpus`.

use cpl_json::{obj, Value};
use cpl_lsp::syntax_hover::Written;
use cpl_lsp::Server;
use cpl_syntax::TokenKind;
use std::collections::BTreeMap;
use std::path::Path;

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn position(src: &str, offset: usize) -> Value {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = src[..offset].matches('\n').count() as u32;
    let col = src[line_start..offset].encode_utf16().count() as u32;
    obj([("line", line.into()), ("character", col.into())])
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Kind {
    /// A value, field, fn or parameter name: `xs`, `len`.
    Name,
    /// A capitalized name: `Int`, `V`, `SegTree`.
    TypeName,
    /// Punctuation of a written type: `[`, `]`, `..`, `?`, `<`, `->`.
    TypeSyntax,
}

/// The tokens to hover in `src`, by kind.
fn sites(src: &str) -> Vec<(Kind, usize, usize)> {
    let mut typed: Vec<(u32, u32)> = Vec::new();
    let mut fills: Vec<u32> = Vec::new();
    for (base, item) in cpl_syntax::parse_file(src).items {
        let written = Written::of(&item.stmts);
        typed.extend(
            written
                .types
                .iter()
                .map(|t| (t.span.start + base, t.span.end + base)),
        );
        for (domain, _) in &written.fills {
            let (start, end) = ((domain.start + base) as usize, (domain.end + base) as usize);
            if let (Some(open), Some(close)) = (src[..start].rfind('['), src[end..].find(']')) {
                fills.extend([open as u32, (end + close) as u32]);
            }
        }
    }
    let in_type = |at: u32| typed.iter().any(|&(s, e)| s <= at && at < e);
    cpl_syntax::lex(src)
        .tokens
        .iter()
        .filter_map(|t| {
            let (start, end) = (t.span.start, t.span.end);
            let kind = match t.kind {
                TokenKind::Ident => {
                    if src[start as usize..].starts_with(|c: char| c.is_ascii_uppercase()) {
                        Kind::TypeName
                    } else {
                        Kind::Name
                    }
                }
                _ if end > start && (in_type(start) || fills.contains(&start)) => {
                    let text = &src[start as usize..end as usize];
                    if !text.starts_with(|c: char| c.is_ascii_punctuation()) {
                        return None;
                    }
                    Kind::TypeSyntax
                }
                _ => return None,
            };
            Some((kind, start as usize, end as usize))
        })
        .collect()
}

#[test]
fn every_corpus_token_has_a_hover() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("corpus/hillclimb.json")).unwrap();
    let list = cpl_json::parse(&list).unwrap();
    let ids: Vec<&str> = list
        .at(&["problems"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| p.as_str())
        .collect();
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut id = 1;
    let mut tally: BTreeMap<Kind, (usize, usize)> = BTreeMap::new();
    let mut misses: BTreeMap<(Kind, String), (usize, String)> = BTreeMap::new();
    let step = if cfg!(debug_assertions) { 4 } else { 1 };
    for pid in ids.iter().step_by(step) {
        let path = root.join(format!("corpus/problems/{pid}/spec.cp"));
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let uri = format!("file:///hover_tokens/{pid}/spec.cp");
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", src.as_str().into()),
                ]),
            )]),
        ));
        for (kind, start, end) in sites(&src) {
            id += 1;
            let reply = server.handle(&message(
                Some(id),
                "textDocument/hover",
                obj([
                    ("textDocument", obj([("uri", uri.as_str().into())])),
                    ("position", position(&src, start)),
                ]),
            ));
            let shown = reply
                .iter()
                .find(|m| !m.at(&["id"]).is_null())
                .and_then(|m| {
                    m.at(&["result", "contents", "value"])
                        .as_str()
                        .map(str::to_string)
                })
                .unwrap_or_default();
            let entry = tally.entry(kind).or_default();
            entry.1 += 1;
            if shown.trim().is_empty() {
                let token = src[start..end].to_string();
                let line = src[..start].matches('\n').count() + 1;
                let miss = misses
                    .entry((kind, token))
                    .or_insert((0, format!("{pid}:{line}")));
                miss.0 += 1;
            } else {
                entry.0 += 1;
            }
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
    }
    let mut backlog: Vec<_> = misses.into_iter().collect();
    backlog.sort_by_key(|b| std::cmp::Reverse(b.1 .0));
    for ((kind, token), (count, at)) in backlog.iter().take(60) {
        println!("no hover: {kind:?} `{token}` x{count} (first at {at})");
    }
    // Per mille of each kind's tokens with a hover, over the sampled ports.
    let floors = [
        (Kind::Name, 925),
        (Kind::TypeName, 650),
        (Kind::TypeSyntax, 1000),
    ];
    let mut low = Vec::new();
    for (kind, floor) in floors {
        let (hit, all) = tally.get(&kind).copied().unwrap_or((0, 0));
        let permille = (hit * 1000).checked_div(all).unwrap_or(0);
        println!("hover {kind:?}: {hit}/{all} ({permille}‰)");
        if permille < floor {
            low.push(format!("{kind:?}: {permille}‰ below {floor}‰"));
        }
    }
    assert!(low.is_empty(), "hover coverage fell: {low:?}");
}
