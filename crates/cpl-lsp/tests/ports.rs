//! Every hill-climb port (`corpus/hillclimb.json`), opened whole: the server
//! publishes diagnostics, and answers hover and go-to-definition on the first
//! use of every name and completion after every `name.`, without an internal
//! error. Hover on a declared array, optional or struct says what it stores.

use cpl_lsp::json::{obj, Value};
use cpl_lsp::Server;
use std::collections::HashSet;
use std::path::Path;

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members)
}

fn position(src: &str, offset: usize) -> Value {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = src[..offset].matches('\n').count() as u32;
    let col = src[line_start..offset].encode_utf16().count() as u32;
    obj([("line", line.into()), ("character", col.into())])
}

/// Byte offsets of the first occurrence of each identifier outside comments
/// and strings, and of each `.` that follows one.
fn sites(src: &str) -> (Vec<usize>, Vec<usize>) {
    let (mut names, mut dots, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
    let mut at = 0;
    for line in src.split_inclusive('\n') {
        let code = line.split("//").next().unwrap_or("");
        let bytes = code.as_bytes();
        let mut i = 0;
        let mut quoted = false;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'"' {
                quoted = !quoted;
            }
            if !quoted && (c.is_ascii_alphabetic() || c == b'_') {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if seen.insert(&code[start..i]) {
                    names.push(at + start);
                }
                if bytes.get(i) == Some(&b'.') && bytes.get(i + 1) != Some(&b'.') {
                    dots.push(at + i + 1);
                }
                continue;
            }
            i += 1;
        }
        at += line.len();
    }
    (names, dots)
}

#[test]
fn every_hill_climb_port_gets_hover_definition_and_completion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("corpus/hillclimb.json")).unwrap();
    let list = cpl_lsp::json::parse(&list).unwrap();
    let ids: Vec<&str> = list.get("problems").as_array().iter().filter_map(|p| p.as_str()).collect();
    assert_eq!(ids.len(), 100);

    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut id = 1;
    let mut failures = Vec::new();
    let mut storage = 0;
    for pid in &ids {
        let path = root.join(format!("corpus/problems/{pid}/spec.cp"));
        let src = std::fs::read_to_string(&path).unwrap();
        let uri = format!("file:///ports/{pid}.cp");
        let doc = || obj([("uri", uri.as_str().into())]);
        let open = server.handle(&message(
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
        if !open.iter().any(|m| m.get("method").as_str() == Some("textDocument/publishDiagnostics")) {
            failures.push(format!("{pid}: no diagnostics published on open"));
        }
        let (names, dots) = sites(&src);
        let mut stores = false;
        let asks = names
            .iter()
            .flat_map(|&at| [("textDocument/hover", at), ("textDocument/definition", at)])
            .chain(dots.iter().map(|&at| ("textDocument/completion", at)));
        for (method, at) in asks {
            id += 1;
            let params = obj([("textDocument", doc()), ("position", position(&src, at))]);
            let reply: String = server.handle(&message(Some(id), method, params)).iter().map(Value::to_string).collect();
            if reply.is_empty() || reply.contains("internal error in cpl") || reply.contains("\"error\"") {
                failures.push(format!("{pid}: {method} at byte {at}: {reply:.200}"));
            }
            stores |= method == "textDocument/hover" && reply.contains("storage:");
        }
        storage += usize::from(stores);
        server.handle(&message(None, "textDocument/didClose", obj([("textDocument", doc())])));
    }
    assert!(failures.is_empty(), "{} failures:\n{}", failures.len(), failures.join("\n"));
    assert!(storage >= 88, "only {storage} of 100 ports show storage on some hover");
}
