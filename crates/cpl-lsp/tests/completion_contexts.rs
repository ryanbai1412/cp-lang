//! Completion in every context, one case per context the completion use
//! cases name (`handbook/product/editor/completion-usecases.md`): after
//! `.`, argument names, names in scope, `use` paths, types and patterns.
//! Each case in `completion-contexts.json` is a document with the cursor
//! at `$0` and the labels completion must offer there.

use cpl_json::{obj, Value};

/// The cases, shared with the browser editor's parity test.
const CASES: &str = include_str!("completion-contexts.json");
use cpl_lsp::Server;

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

#[test]
fn completion_offers_what_each_context_needs() {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut misses = Vec::new();
    let cases = cpl_json::parse(CASES).expect("completion-contexts.json parses");
    for (id, case) in (2..).zip(cases.as_array().expect("a list of cases")) {
        let context = case.at(&["context"]).as_str().expect("a context");
        let marked = case.at(&["source"]).as_str().expect("a source");
        let want: Vec<&str> = case
            .at(&["want"])
            .as_array()
            .expect("the labels it wants")
            .iter()
            .filter_map(|w| w.as_str())
            .collect();
        assert!(!want.is_empty(), "{context} names no label it must offer");
        let at = marked.find("$0").expect("a cursor");
        let text = marked.replacen("$0", "", 1);
        let uri = format!("file:///contexts/{}.cp", context.replace('/', "-"));
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", text.as_str().into()),
                ]),
            )]),
        ));
        let reply = server.handle(&message(
            Some(id),
            "textDocument/completion",
            obj([
                ("textDocument", obj([("uri", uri.as_str().into())])),
                ("position", position(&text, at)),
            ]),
        ));
        // As the playground shows them: those the word before the cursor
        // starts, by sortText, the first 100.
        let word = text[..at]
            .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
            .next()
            .unwrap_or("")
            .to_lowercase();
        let mut ranked: Vec<(&str, &str)> = reply
            .iter()
            .filter_map(|r| r.at(&["result", "items"]).as_array())
            .flatten()
            .filter_map(|item| {
                let label = item.at(&["label"]).as_str()?;
                Some((item.at(&["sortText"]).as_str().unwrap_or(label), label))
            })
            .filter(|(_, label)| label.to_lowercase().starts_with(&word))
            .collect();
        ranked.sort();
        let labels: Vec<&str> = ranked.iter().take(100).map(|&(_, l)| l).collect();
        let offered = |w: &&str| {
            labels
                .iter()
                .any(|l| l == w || l.strip_suffix(':') == Some(w))
        };
        if !want.iter().all(offered) {
            misses.push(format!(
                "{context}: wanted {want:?}, got {:?}",
                &labels[..labels.len().min(40)]
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "completion misses:\n{}",
        misses.join("\n")
    );
}
