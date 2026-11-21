//! The server answers every request on every broken half-typed file without
//! an internal error: real ports, cut, spliced and garbled the way typing
//! leaves them, each followed by every request an editor sends.
//!
//! The server turns a panic into an `internal error in cpl` reply and a
//! diagnostic (see `Server::handle`), so the test fails on that text, which
//! also catches a panic the server survived.

use cpl_lsp::json::{obj, Value};
use cpl_lsp::Server;

const URI: &str = "file:///fuzz/solution.cp";
const PORTS: usize = 12;
const EDITS_PER_PORT: usize = 30;

fn edits_per_port() -> usize {
    std::env::var("CPL_LSP_FUZZ_EDITS").ok().and_then(|n| n.parse().ok()).unwrap_or(EDITS_PER_PORT)
}

fn port_count() -> usize {
    std::env::var("CPL_LSP_FUZZ_PORTS").ok().and_then(|n| n.parse().ok()).unwrap_or(PORTS)
}
const SNIPPETS: &[&str] = &[
    "(", ")", "[", "]", "{", "}", "=>", "++", ".", ":", ",", "\n    ", "\n", "let ", "var ",
    "fn ", "if ", "for ", "in ", "\"", "'", "?", "!", "..", "..=", "**", "@", "#", "\t", "é",
    "x.", "_", "0", "-",
];

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n.max(1) as u64) as usize
    }

    fn offset(&mut self, src: &str) -> usize {
        let mut at = self.below(src.len() + 1);
        while !src.is_char_boundary(at) {
            at -= 1;
        }
        at
    }
}

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

fn mutate(src: &str, rng: &mut Rng) -> String {
    let at = rng.offset(src);
    match rng.below(5) {
        0 => src[..at].to_string(),
        1 => {
            let end = rng.offset(&src[at..]) % 40;
            let mut end = (at + end).min(src.len());
            while !src.is_char_boundary(end) {
                end -= 1;
            }
            format!("{}{}", &src[..at], &src[end..])
        }
        2 => {
            let snippet = SNIPPETS[rng.below(SNIPPETS.len())];
            format!("{}{snippet}{}", &src[..at], &src[at..])
        }
        3 => {
            let lines: Vec<&str> = src.lines().collect();
            let i = rng.below(lines.len());
            let mut out: Vec<&str> = lines.clone();
            out.insert(rng.below(lines.len() + 1), lines.get(i).copied().unwrap_or(""));
            out.join("\n")
        }
        _ => {
            let lines: Vec<&str> = src.lines().collect();
            let i = rng.below(lines.len());
            let out: Vec<String> = lines
                .iter()
                .enumerate()
                .map(|(j, l)| match j == i {
                    true => l.trim_start().to_string(),
                    false => l.to_string(),
                })
                .collect();
            out.join("\n")
        }
    }
}

fn requests(src: &str, rng: &mut Rng) -> Vec<(&'static str, Value)> {
    let doc = || obj([("uri", URI.into())]);
    let mut out = Vec::new();
    for _ in 0..2 {
        let p = position(src, rng.offset(src));
        let with_pos = |extra: Vec<(&'static str, Value)>| {
            let mut members = vec![
                ("textDocument".to_string(), doc()),
                ("position".to_string(), p.clone()),
            ];
            members.extend(extra.into_iter().map(|(k, v)| (k.to_string(), v)));
            Value::Object(members)
        };
        out.push(("textDocument/hover", with_pos(vec![])));
        out.push(("textDocument/completion", with_pos(vec![])));
        out.push(("textDocument/signatureHelp", with_pos(vec![])));
        out.push(("textDocument/definition", with_pos(vec![])));
        out.push(("textDocument/documentHighlight", with_pos(vec![])));
        out.push((
            "textDocument/references",
            with_pos(vec![("context", obj([("includeDeclaration", true.into())]))]),
        ));
        out.push(("textDocument/prepareRename", with_pos(vec![])));
        out.push(("textDocument/rename", with_pos(vec![("newName", "renamed".into())])));
        out.push((
            "textDocument/codeAction",
            obj([
                ("textDocument", doc()),
                ("range", obj([("start", p.clone()), ("end", p.clone())])),
                ("context", obj([("diagnostics", Value::Array(Vec::new()))])),
            ]),
        ));
    }
    let whole = obj([
        ("start", obj([("line", 0.into()), ("character", 0.into())])),
        ("end", position(src, src.len())),
    ]);
    out.push(("textDocument/inlayHint", obj([("textDocument", doc()), ("range", whole)])));
    out.push(("textDocument/semanticTokens/full", obj([("textDocument", doc())])));
    out.push(("textDocument/foldingRange", obj([("textDocument", doc())])));
    out.push(("textDocument/documentSymbol", obj([("textDocument", doc())])));
    out.push(("textDocument/codeLens", obj([("textDocument", doc())])));
    out.push((
        "textDocument/formatting",
        obj([
            ("textDocument", doc()),
            ("options", obj([("tabSize", 4.into()), ("insertSpaces", true.into())])),
        ]),
    ));
    out.push(("workspace/symbol", obj([("query", "a".into())])));
    out
}

#[test]
fn every_request_on_broken_ports_is_answered_without_an_internal_error() {
    let ports = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proto/ports");
    let mut files = Vec::new();
    for author in std::fs::read_dir(&ports).unwrap().flatten() {
        let Ok(entries) = std::fs::read_dir(author.path()) else { continue };
        for file in entries.flatten() {
            if file.path().extension().is_some_and(|e| e == "cp") {
                files.push(file.path());
            }
        }
    }
    files.sort();
    let ports = port_count().min(files.len());
    let step = (files.len() / ports).max(1);
    let picked: Vec<_> = files.iter().step_by(step).take(ports).collect();
    assert_eq!(picked.len(), ports);

    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let mut id = 2;
    let mut answered = 0;
    let mut failures = Vec::new();
    for path in picked {
        let original = std::fs::read_to_string(path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", URI.into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", original.as_str().into()),
                ]),
            )]),
        ));
        let mut src = original.clone();
        for edit in 0..edits_per_port() {
            src = if edit % 3 == 0 { mutate(&original, &mut rng) } else { mutate(&src, &mut rng) };
            let change = message(
                None,
                "textDocument/didChange",
                obj([
                    ("textDocument", obj([("uri", URI.into()), ("version", (edit as u32 + 2).into())])),
                    ("contentChanges", Value::Array(vec![obj([("text", src.as_str().into())])])),
                ]),
            );
            let mut replies: Vec<(&str, Vec<Value>)> = vec![("didChange", server.handle(&change))];
            for (method, params) in requests(&src, &mut rng) {
                id += 1;
                replies.push((method, server.handle(&message(Some(id), method, params))));
                answered += 1;
            }
            for (method, reply) in replies {
                if let Some(text) = reply
                    .iter()
                    .map(Value::to_string)
                    .find(|r| r.contains("internal error in cpl"))
                {
                    failures.push(format!("{name} edit {edit} {method}: {text}\n--- source ---\n{src}"));
                }
            }
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", URI.into())]))]),
        ));
    }
    println!("lsp fuzz: {answered} requests on {} broken files", ports * edits_per_port());
    assert!(failures.is_empty(), "{} internal errors; first:\n{}", failures.len(), failures[0]);
}
