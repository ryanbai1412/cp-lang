//! Highlighting agrees with the parser, for the server: every semantic token
//! `cpl lsp` sends for a conformance case covers exactly one name the lexer
//! reads (`TokenKind::Ident`, or `self`, the receiver's name), or a name in a
//! string's `{...}` interpolation, never a keyword, number, comment or operator. The generated grammars are held to the same lexer by
//! `crates/cpl-cli/tests/highlight_agrees.rs`.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use cpl_syntax::TokenKind;
use std::collections::BTreeSet;
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

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

/// Each name the lexer reads, as the LSP places it: (line, UTF-16 column, UTF-16 length).
fn names(src: &str) -> BTreeSet<(u32, u32, u32)> {
    let lexed = cpl_syntax::lex(src);
    let mut spans = Vec::new();
    for t in &lexed.tokens {
        let (start, end) = (t.span.start as usize, t.span.end as usize);
        match t.kind {
            TokenKind::Ident | TokenKind::SelfKw => spans.push((start, end)),
            TokenKind::Str => {
                let mut depth = 0;
                let mut word = None;
                for (i, c) in src[start..end].char_indices().map(|(i, c)| (start + i, c)) {
                    let in_word = depth > 0 && (c.is_alphanumeric() || c == '_');
                    match (word, in_word) {
                        (None, true) if !c.is_ascii_digit() => word = Some(i),
                        (Some(w), false) => {
                            spans.push((w, i));
                            word = None;
                        }
                        _ => {}
                    }
                    match c {
                        '{' => depth += 1,
                        '}' if depth > 0 => depth -= 1,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    (spans.into_iter())
        .map(|(start, end)| {
            let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
            let line = src[..start].matches('\n').count() as u32;
            let col = src[line_start..start].encode_utf16().count() as u32;
            (line, col, src[start..end].encode_utf16().count() as u32)
        })
        .collect()
}

#[test]
fn every_semantic_token_is_a_name_the_lexer_reads() {
    // The checker recurses as deep as a case nests, as on `cpl lsp`'s own thread.
    let run = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(check);
    run.unwrap().join().unwrap();
}

fn check() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = Vec::new();
    walk(&root.join("spec/conformance"), &mut paths);
    paths.sort();
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let (mut id, mut checked, mut tokens) = (1, 0, 0);
    let mut wrong = Vec::new();
    let step = if cfg!(debug_assertions) { 32 } else { 1 };
    for path in paths.iter().step_by(step) {
        let src = std::fs::read_to_string(path).unwrap();
        if !cpl_syntax::lex(&src).diagnostics.is_empty() {
            continue;
        }
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let uri = format!("file:///semantic/{rel}");
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
        id += 1;
        let reply = server.handle(&message(
            Some(id),
            "textDocument/semanticTokens/full",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        let data: Vec<u32> = (reply.iter())
            .find(|m| !m.at(&["id"]).is_null())
            .and_then(|m| m.at(&["result", "data"]).as_array().map(|a| a.to_vec()))
            .unwrap_or_default()
            .iter()
            .map(|v| v.as_f64().unwrap_or(-1.0) as u32)
            .collect();
        let lexed = names(&src);
        let (mut line, mut col) = (0, 0);
        for t in data.chunks(5) {
            (line, col) = (line + t[0], if t[0] == 0 { col + t[1] } else { t[1] });
            tokens += 1;
            if !lexed.contains(&(line, col, t[2])) && wrong.len() < 30 {
                let text = src.lines().nth(line as usize).unwrap_or("");
                wrong.push(format!("{rel}:{}:{}: {}", line + 1, col + 1, text.trim()));
            }
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        checked += 1;
    }
    assert!(
        checked > 100 && tokens > 2_000,
        "{checked} cases, {tokens} tokens"
    );
    assert!(
        wrong.is_empty(),
        "semantic tokens that are not one name the lexer reads:\n{}",
        wrong.join("\n")
    );
}
