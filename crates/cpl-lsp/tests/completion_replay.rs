//! Completion recall: every hill-climb port (`corpus/hillclimb.json`) retyped
//! one identifier at a time. At each identifier the contestant typed, the
//! document is the port with that identifier erased (its first letter kept
//! for a name in scope), and completion must offer it: after `.` (a member),
//! after `).` or `].` (a member of a call's or index's result, as in
//! `runs(xs).`), and for a name already used earlier in the file. Each kind's
//! recall is a ratchet that only goes up; misses are printed, most common
//! first, as the backlog.
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r14-measured-over-the-corpus`.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::collections::{BTreeMap, HashSet};
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
    /// `xs.len`: after `name.`
    Member,
    /// `runs(xs).len`, `a[i].len`: after a call's or an index's result
    AfterCall,
    /// `total`: a name used earlier in the file
    Scope,
}

/// Words after which the next name is being declared, not used.
const DECLARES: [&str; 10] = [
    "let", "var", "const", "fn", "for", "struct", "enum", "type", "domain", "use",
];

/// Each identifier site `(start, end, kind)` outside comments and strings.
fn sites(src: &str) -> Vec<(usize, usize, Kind)> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut at = 0;
    for line in src.split_inclusive('\n') {
        let code = line.split("//").next().unwrap_or("");
        let bytes = code.as_bytes();
        let (mut i, mut quoted, mut prev_word) = (0, false, "");
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'"' {
                quoted = !quoted;
            }
            // A letter right after a digit is part of a number (`1_000`, `2e5`).
            let in_number = i > 0 && bytes[i - 1].is_ascii_digit();
            if quoted || in_number || !(c.is_ascii_alphabetic() || c == b'_') {
                if !c.is_ascii_whitespace() {
                    prev_word = "";
                }
                i += 1;
                while in_number
                    && i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_')
                {
                    i += 1;
                }
                continue;
            }
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &code[start..i];
            let before = |k: usize| start.checked_sub(k).map(|j| bytes[j]);
            let rest = code[i..].trim_start();
            let declared = DECLARES.contains(&prev_word)
                || rest.starts_with("=>")
                || (rest.starts_with(':') && !rest.starts_with(":="))
                || lambda_param(&code[i..]);
            let kind = if before(1) == Some(b'.') && before(2) != Some(b'.') {
                Some(if matches!(before(2), Some(b')' | b']')) {
                    Kind::AfterCall
                } else {
                    Kind::Member
                })
            } else if !declared && !cpl_syntax::KEYWORDS.contains(&word) && seen.contains(word) {
                Some(Kind::Scope)
            } else {
                None
            };
            if let Some(kind) = kind {
                out.push((at + start, at + i, kind));
            }
            seen.insert(word);
            prev_word = word;
        }
        at += line.len();
    }
    out
}

/// Whether the text after a name continues a lambda's parameter list:
/// `, b) =>` or `) =>`.
fn lambda_param(after: &str) -> bool {
    let close = after.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == ',' || c == ' '));
    close.is_some_and(|j| {
        after[j..].starts_with(')') && after[j + 1..].trim_start().starts_with("=>")
    })
}

fn labels(reply: &[Value]) -> HashSet<String> {
    reply
        .iter()
        .flat_map(|m| {
            let result = m.at(&["result"]);
            let items = result.at(&["items"]).as_array().or(result.as_array());
            items.unwrap_or(&[]).iter()
        })
        .filter_map(|item| item.at(&["label"]).as_str().map(str::to_string))
        .collect()
}

/// Where `word` sits, 1 first, among the offered labels that start with
/// what is typed of it, in the order an editor shows them (by sortText).
fn rank(reply: &[Value], typed: &str, word: &str) -> Option<usize> {
    let mut items: Vec<(String, String)> = reply
        .iter()
        .flat_map(|m| {
            let result = m.at(&["result"]);
            let items = result.at(&["items"]).as_array().or(result.as_array());
            items.unwrap_or(&[]).iter()
        })
        .filter_map(|item| {
            let label = item.at(&["label"]).as_str()?;
            let sort = item.at(&["sortText"]).as_str().unwrap_or(label);
            label
                .starts_with(typed)
                .then(|| (sort.to_string(), label.to_string()))
        })
        .collect();
    items.sort();
    items.iter().position(|(_, l)| l == word).map(|i| i + 1)
}

#[test]
fn completion_offers_what_each_hill_climb_port_types_next() {
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
    let mut ranks: BTreeMap<Kind, (usize, usize)> = BTreeMap::new();
    let mut misses: BTreeMap<(Kind, String), (usize, String)> = BTreeMap::new();
    for pid in &ids {
        let src =
            std::fs::read_to_string(root.join(format!("corpus/problems/{pid}/spec.cp"))).unwrap();
        let uri = format!("file:///replay/{pid}.cp");
        let doc = |version: u32| obj([("uri", uri.as_str().into()), ("version", version.into())]);
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
        // A debug build replays every eighth site to stay quick; release
        // (the latency job) replays all of them.
        let step = if cfg!(debug_assertions) { 8 } else { 1 };
        let sampled = sites(&src).into_iter().step_by(step);
        for (version, (start, end, kind)) in (2..).zip(sampled) {
            let keep = if kind == Kind::Scope { 1 } else { 0 };
            let cursor = start + keep;
            let text = format!("{}{}", &src[..cursor], &src[end..]);
            server.handle(&message(
                None,
                "textDocument/didChange",
                obj([
                    ("textDocument", doc(version)),
                    (
                        "contentChanges",
                        Value::Array(vec![obj([("text", text.as_str().into())])]),
                    ),
                ]),
            ));
            id += 1;
            let reply = server.handle(&message(
                Some(id),
                "textDocument/completion",
                obj([
                    ("textDocument", obj([("uri", uri.as_str().into())])),
                    ("position", position(&text, cursor)),
                ]),
            ));
            let word = &src[start..end];
            let entry = tally.entry(kind).or_default();
            entry.1 += 1;
            if labels(&reply).contains(word) {
                entry.0 += 1;
                let at = rank(&reply, &src[start..cursor], word).unwrap_or(usize::MAX);
                let ranks = ranks.entry(kind).or_default();
                ranks.0 += at.min(1000);
                ranks.1 += 1;
            } else {
                let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
                let line_end = src[end..].find('\n').map_or(src.len(), |i| end + i);
                let miss = misses.entry((kind, word.to_string())).or_default();
                miss.0 += 1;
                if miss.1.is_empty() {
                    miss.1 = format!("{pid}: {}", src[line_start..line_end].trim());
                }
            }
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
    }
    let mut ranked: Vec<_> = misses.into_iter().collect();
    ranked.sort_by(|a, b| b.1 .0.cmp(&a.1 .0).then_with(|| a.0.cmp(&b.0)));
    for ((kind, word), (n, first)) in ranked.iter().take(40) {
        println!("missed {n:4}  {kind:?}  {word}  (first: {first:.100})");
    }
    // Recall per kind in tenths of a percent; raise these as completion improves.
    let floors = [
        (Kind::Member, 860),
        (Kind::AfterCall, 810),
        (Kind::Scope, 1000),
    ];
    for (kind, floor) in floors {
        let (hit, all) = tally.get(&kind).copied().unwrap_or_default();
        let permille = hit * 1000 / all.max(1);
        println!(
            "recall {kind:?}: {hit}/{all} = {}.{}%",
            permille / 10,
            permille % 10
        );
        assert!(
            permille >= floor,
            "{kind:?} recall fell to {hit}/{all}, below {floor}"
        );
    }
    // Mean rank, in tenths, of the word typed among the labels offered that
    // start with what is typed of it; lower these as ranking improves.
    let ceilings = [
        (Kind::Member, 115),
        (Kind::AfterCall, 310),
        (Kind::Scope, 20),
    ];
    for (kind, ceiling) in ceilings {
        let (sum, n) = ranks.get(&kind).copied().unwrap_or_default();
        let tenths = sum * 10 / n.max(1);
        println!(
            "mean rank {kind:?}: {}.{} over {n}",
            tenths / 10,
            tenths % 10
        );
        assert!(
            tenths <= ceiling,
            "{kind:?} mean rank rose to {tenths} tenths, above {ceiling}"
        );
    }
}
