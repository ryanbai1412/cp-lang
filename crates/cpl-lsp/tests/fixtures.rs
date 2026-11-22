//! LSP fixture tests (`plan/60-editor.md` 6.7): each `fixtures/<name>.cp`
//! names the requests to send on its first line (`// lsp: hover, completion`),
//! marks cursor positions with `$0`, and is checked against
//! `fixtures/<name>.expected`, the responses rendered for reading.
//! Positions there are 1-based `line:col`. Rerun with `BLESS=1` to rewrite
//! the `.expected` files after an intended change.

use std::fs;
use std::path::{Path, PathBuf};

use cpl_json::{obj, Value};
use cpl_lsp::Server;

const MARKER: &str = "$0";

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

fn at(p: &Value) -> String {
    format!(
        "{}:{}",
        p.at(&["line"]).as_u32().unwrap() + 1,
        p.at(&["character"]).as_u32().unwrap() + 1
    )
}

fn span(range: &Value) -> String {
    format!("{}-{}", at(range.at(&["start"])), at(range.at(&["end"])))
}

/// A document URI as the reader should see it: the fixture itself, or a
/// path from the repository root, so the snapshots name no machine.
fn file(uri: &str, own: &str) -> String {
    if uri == own {
        return "here".to_string();
    }
    match uri.find("/spec/std/") {
        Some(i) => uri[i + 1..].to_string(),
        None => uri.to_string(),
    }
}

fn location(loc: &Value, own: &str) -> String {
    let uri = loc
        .at(&["uri"])
        .as_str()
        .or(loc.at(&["targetUri"]).as_str())
        .unwrap_or("?");
    let range = match loc.at(&["range"]).is_null() {
        true => loc.at(&["targetSelectionRange"]),
        false => loc.at(&["range"]),
    };
    format!("{} {}", file(uri, own), span(range))
}

fn items(result: &Value) -> &[Value] {
    match result {
        Value::Array(items) => items,
        _ => result.at(&["items"]).as_array().unwrap_or(&[]),
    }
}

fn render(method: &str, result: &Value, own: &str) -> String {
    if result.is_null() {
        return "(none)".to_string();
    }
    let lines: Vec<String> = match method {
        "hover" => vec![result
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("?")
            .to_string()],
        "completion" => items(result)
            .iter()
            .take(12)
            .map(|item| {
                let detail = item.at(&["detail"]).as_str().unwrap_or("");
                format!(
                    "{} [{}] {detail}",
                    item.at(&["label"]).as_str().unwrap_or("?"),
                    item.at(&["kind"]).as_u32().unwrap_or(0)
                )
                .trim_end()
                .to_string()
            })
            .collect(),
        "signatureHelp" => {
            let active = result.at(&["activeSignature"]).as_u32().unwrap_or(0) as usize;
            let sig = &result.at(&["signatures"]).as_array().unwrap_or(&[])[active];
            let param = result
                .at(&["activeParameter"])
                .as_u32()
                .or(sig.at(&["activeParameter"]).as_u32());
            let labels: Vec<String> = sig
                .at(&["parameters"])
                .as_array()
                .unwrap_or(&[])
                .iter()
                .map(|p| match p.at(&["label"]) {
                    Value::String(s) => s.clone(),
                    offsets => {
                        let ends: Vec<usize> = offsets
                            .as_array()
                            .unwrap_or(&[])
                            .iter()
                            .filter_map(|n| n.as_u32())
                            .map(|n| n as usize)
                            .collect();
                        let units: Vec<u16> = sig
                            .at(&["label"])
                            .as_str()
                            .unwrap_or("")
                            .encode_utf16()
                            .collect();
                        String::from_utf16_lossy(&units[ends[0]..ends[1]])
                    }
                })
                .collect();
            let current = param
                .and_then(|i| labels.get(i as usize))
                .map_or("-".to_string(), Clone::clone);
            vec![
                sig.at(&["label"]).as_str().unwrap_or("?").to_string(),
                format!("active: {current}"),
            ]
        }
        "definition" | "references" | "documentHighlight" => match result {
            Value::Array(locs) => locs
                .iter()
                .map(
                    |l| match l.at(&["uri"]).is_null() && l.at(&["targetUri"]).is_null() {
                        true => span(l.at(&["range"])),
                        false => location(l, own),
                    },
                )
                .collect(),
            _ => vec![location(result, own)],
        },
        "rename" => {
            let mut out = Vec::new();
            if let Value::Object(files) = result.at(&["changes"]) {
                for (uri, edits) in files {
                    for edit in edits.as_array().unwrap_or(&[]) {
                        out.push(format!(
                            "{} {} -> {}",
                            file(uri, own),
                            span(edit.at(&["range"])),
                            edit.at(&["newText"]).as_str().unwrap_or("?")
                        ));
                    }
                }
            }
            out.sort();
            out
        }
        "codeAction" => result
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|a| a.at(&["title"]).as_str().unwrap_or("?").to_string())
            .collect(),
        "inlayHint" => result
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|h| {
                let label = match h.at(&["label"]) {
                    Value::String(s) => s.clone(),
                    parts => parts
                        .as_array()
                        .unwrap_or(&[])
                        .iter()
                        .filter_map(|p| p.at(&["value"]).as_str())
                        .collect(),
                };
                format!("{} {label}", at(h.at(&["position"])))
            })
            .collect(),
        "foldingRange" => result
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|r| {
                let kind = r.at(&["kind"]).as_str().unwrap_or("region");
                format!(
                    "{}-{} {kind}",
                    r.at(&["startLine"]).as_u32().unwrap() + 1,
                    r.at(&["endLine"]).as_u32().unwrap() + 1
                )
            })
            .collect(),
        "documentSymbol" => {
            fn walk(symbols: &[Value], depth: usize, out: &mut Vec<String>) {
                for s in symbols {
                    out.push(format!(
                        "{}{} [{}]",
                        "  ".repeat(depth),
                        s.at(&["name"]).as_str().unwrap_or("?"),
                        s.at(&["kind"]).as_u32().unwrap_or(0)
                    ));
                    walk(
                        s.at(&["children"]).as_array().unwrap_or(&[]),
                        depth + 1,
                        out,
                    );
                }
            }
            let mut out = Vec::new();
            walk(result.as_array().unwrap_or(&[]), 0, &mut out);
            out
        }
        _ => vec![result.to_string()],
    };
    match lines.is_empty() {
        true => "(empty)".to_string(),
        false => lines.join("\n"),
    }
}

fn params(method: &str, uri: &str, src: &str, offset: usize) -> Value {
    let doc = || obj([("uri", uri.into())]);
    let p = position(src, offset);
    let whole = || {
        obj([
            ("start", position(src, 0)),
            ("end", position(src, src.len())),
        ])
    };
    match method {
        "inlayHint" => obj([("textDocument", doc()), ("range", whole())]),
        "foldingRange" | "documentSymbol" => obj([("textDocument", doc())]),
        "codeAction" => obj([
            ("textDocument", doc()),
            ("range", obj([("start", p.clone()), ("end", p)])),
            ("context", obj([("diagnostics", Value::Array(Vec::new()))])),
        ]),
        "references" => obj([
            ("textDocument", doc()),
            ("position", p),
            ("context", obj([("includeDeclaration", true.into())])),
        ]),
        "rename" => obj([
            ("textDocument", doc()),
            ("position", p),
            ("newName", "renamed".into()),
        ]),
        _ => obj([("textDocument", doc()), ("position", p)]),
    }
}

fn whole_file(method: &str) -> bool {
    matches!(method, "inlayHint" | "foldingRange" | "documentSymbol")
}

fn run(path: &Path) -> String {
    let marked = fs::read_to_string(path).unwrap();
    let header = marked.lines().next().unwrap_or("");
    let methods: Vec<&str> = header
        .strip_prefix("// lsp:")
        .unwrap_or_else(|| {
            panic!(
                "{}: the first line must be `// lsp: <requests>`",
                path.display()
            )
        })
        .split(',')
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .collect();
    let mut src = String::new();
    let mut markers = Vec::new();
    let mut rest = marked.as_str();
    while let Some(i) = rest.find(MARKER) {
        src.push_str(&rest[..i]);
        markers.push(src.len());
        rest = &rest[i + MARKER.len()..];
    }
    src.push_str(rest);
    let uri = format!(
        "file:///fixtures/{}",
        path.file_name().unwrap().to_string_lossy()
    );

    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let published = server.handle(&message(
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
    let mut out = String::from("== diagnostics\n");
    let diagnostics: Vec<String> = published
        .iter()
        .flat_map(|m| {
            m.at(&["params", "diagnostics"])
                .as_array()
                .unwrap_or(&[])
                .to_vec()
        })
        .map(|d| {
            format!(
                "{} {} {}",
                span(d.at(&["range"])),
                d.at(&["code"]).as_str().unwrap_or("-"),
                d.at(&["message"]).as_str().unwrap_or("?")
            )
        })
        .collect();
    out.push_str(&if diagnostics.is_empty() {
        "(none)".to_string()
    } else {
        diagnostics.join("\n")
    });
    out.push('\n');
    let mut id = 2;
    for method in methods {
        let offsets: Vec<usize> = if whole_file(method) {
            vec![0]
        } else {
            markers.clone()
        };
        assert!(
            !offsets.is_empty(),
            "{}: `{method}` needs a {MARKER} marker",
            path.display()
        );
        for offset in offsets {
            let reply = server.handle(&message(
                Some(id),
                &format!("textDocument/{method}"),
                params(method, &uri, &src, offset),
            ));
            id += 1;
            let reply = reply
                .iter()
                .find(|m| !m.at(&["id"]).is_null())
                .unwrap_or_else(|| panic!("{}: no reply to {method}", path.display()));
            assert!(
                reply.at(&["error"]).is_null(),
                "{}: {method} failed: {reply}",
                path.display()
            );
            match whole_file(method) {
                true => out.push_str(&format!("\n== {method}\n")),
                false => out.push_str(&format!(
                    "\n== {method} @ {}\n",
                    at(&position(&src, offset))
                )),
            }
            out.push_str(&render(method, reply.at(&["result"]), &uri));
            out.push('\n');
        }
    }
    out
}

#[test]
fn every_fixture_answers_as_its_snapshot_says() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let bless = std::env::var_os("BLESS").is_some();
    let mut fixtures: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    fixtures.sort();
    assert!(!fixtures.is_empty());
    let mut wrong = Vec::new();
    for fixture in &fixtures {
        let got = run(fixture);
        let expected = fixture.with_extension("expected");
        if bless {
            fs::write(&expected, &got).unwrap();
            continue;
        }
        let want = fs::read_to_string(&expected).unwrap_or_default();
        if got != want {
            wrong.push(format!("--- {}\n{got}", expected.display()));
        }
    }
    assert!(
        wrong.is_empty(),
        "rerun with BLESS=1 if intended:\n{}",
        wrong.join("\n")
    );
}
