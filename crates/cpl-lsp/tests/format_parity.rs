//! The editor formats as `cpl fmt` does: on every conformance case,
//! `textDocument/formatting` applied to the file gives `cpl_fmt::format`'s
//! text, and a file `cpl fmt` can't format gets no edits.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::path::{Path, PathBuf};

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn cases(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            cases(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

/// `text` with the LSP `edits` applied; each edit's range is whole lines.
fn apply(text: &str, edits: &[Value]) -> String {
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let mut edits: Vec<&Value> = edits.iter().collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.at(&["range", "start", "line"]).as_u32()));
    for edit in edits {
        let line = |end: &str| edit.at(&["range", end, "line"]).as_u32().unwrap() as usize;
        let character = |end: &str| edit.at(&["range", end, "character"]).as_u32().unwrap_or(0);
        assert_eq!(
            (character("start"), character("end")),
            (0, 0),
            "a whole-line edit"
        );
        let (start, end) = (line("start"), line("end").min(lines.len()));
        let new = edit.at(&["newText"]).as_str().unwrap_or("");
        lines.splice(start..end, new.split_inclusive('\n').map(str::to_string));
    }
    lines.concat()
}

#[test]
fn editor_formatting_equals_cpl_fmt_on_every_conformance_case() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/conformance");
    let mut files = Vec::new();
    cases(&root, &mut files);
    assert!(
        files.len() > 1000,
        "found {} conformance cases",
        files.len()
    );
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    let mut differ = Vec::new();
    for (id, path) in (2..).zip(&files) {
        let text = std::fs::read_to_string(path).unwrap();
        let uri = format!("file:///format/{id}.cp");
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
            "textDocument/formatting",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        let edits = reply
            .iter()
            .find_map(|r| r.at(&["result"]).as_array())
            .map(<[Value]>::to_vec)
            .unwrap_or_default();
        let want = cpl_fmt::format(&text).unwrap_or_else(|_| text.clone());
        if apply(&text, &edits) != want {
            differ.push(path.strip_prefix(&root).unwrap().display().to_string());
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
    }
    assert!(
        differ.is_empty(),
        "the editor formats {} cases unlike cpl fmt:\n{}",
        differ.len(),
        differ.join("\n")
    );
}
