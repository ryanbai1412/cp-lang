//! Rename keeps the program's meaning: in each hill-climb port
//! (`corpus/hillclimb.json`), every name the editor offers to rename is
//! renamed through `cpl lsp`, one after another. The result checks clean,
//! and on the port's first test it prints what the original prints.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::path::Path;
use std::process::Command;

#[path = "support/corpus_cases.rs"]
mod corpus_cases;

/// Renames known to change meaning or be refused wrongly, `port word`; the
/// checker's causes are with compiler. Only shrinks: a fixed one fails the
/// test until it is removed here.
const KNOWN: &[&str] = &[
    // `f(..).dist`, `r.len`: a member resolves to the top-level name.
    "apio23p1 dist",
    "btoi18p4 s",
    "btoi18p4 lo",
    "btoi18p4 hi",
    "cco04p4 mv",
    "cco20p4 k",
    "ioi16p6 line",
    "ioi16p6 count",
    "nccc8s3 last",
    "noi17p2 W",
    "noi17p2 len",
    "sdoi17r2p4 dist",
    // A variant's `match` arms are not renamed with it.
    "cco04p4 X",
    "cco04p4 Y",
    "noi17p2 Join",
    "noi17p2 Split",
    "noi17p2 Ask",
    // A parameter's `name:` labels at calls are not renamed with it.
    "egoi23p4 over",
    "noi10p2 best",
    // A name used as a compile-time constant (E0005 after the rename).
    "etsr23r4p2 p",
    // A method a std type calls on the user's type (`LiChao` needs `at`).
    "ioi16p6 at",
];

const URI: &str = "file:///rename/port.cp";

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn ask(server: &mut Server, method: &str, params: Value) -> Value {
    let replies = server.handle(&message(Some(7), method, params));
    let reply = replies
        .into_iter()
        .find(|r| r.at(&["id"]).as_u64() == Some(7));
    reply.map_or(Value::Null, |r| {
        let error = r.at(&["error"]);
        if error.is_null() {
            r.at(&["result"]).clone()
        } else {
            error.clone()
        }
    })
}

fn change(server: &mut Server, version: u32, text: &str) {
    server.handle(&message(
        None,
        "textDocument/didChange",
        obj([
            (
                "textDocument",
                obj([("uri", URI.into()), ("version", version.into())]),
            ),
            (
                "contentChanges",
                Value::Array(vec![obj([("text", text.into())])]),
            ),
        ]),
    ));
}

fn errors(server: &mut Server) -> Vec<String> {
    let reply = ask(
        server,
        "textDocument/diagnostic",
        obj([("textDocument", obj([("uri", URI.into())]))]),
    );
    reply
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|d| d.at(&["severity"]).as_u64() == Some(1))
        .map(|d| format!("{} {}", d.at(&["code"]), d.at(&["message"])))
        .collect()
}

/// Byte offset of the first use of `word` outside comments and strings.
fn first_use(src: &str, word: &str) -> Option<usize> {
    names(src)
        .into_iter()
        .find(|&(_, w)| w == word)
        .map(|(at, _)| at)
}

/// Each identifier's first use, in order, outside comments and strings.
fn names(src: &str) -> Vec<(usize, &str)> {
    let (mut out, mut seen, mut at) = (Vec::new(), std::collections::HashSet::new(), 0);
    for line in src.split_inclusive('\n') {
        let code = line.split("//").next().unwrap_or("");
        let bytes = code.as_bytes();
        let (mut i, mut quoted) = (0, false);
        while i < bytes.len() {
            let c = bytes[i];
            quoted ^= c == b'"';
            if !quoted && (c.is_ascii_alphabetic() || c == b'_') {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let prev = code[..start].chars().next_back();
                let word = &code[start..i];
                if prev != Some('.')
                    && !prev.is_some_and(|p| p.is_ascii_digit())
                    && seen.insert(word)
                {
                    out.push((at + start, word));
                }
                continue;
            }
            i += 1;
        }
        at += line.len();
    }
    out
}

fn position(src: &str, at: usize) -> Value {
    let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
    let line = src[..at].matches('\n').count() as u32;
    let col = src[line_start..at].encode_utf16().count() as u32;
    obj([("line", line.into()), ("character", col.into())])
}

fn offset(src: &str, pos: &Value) -> usize {
    let line = pos.at(&["line"]).as_u64().unwrap() as usize;
    let col = pos.at(&["character"]).as_u64().unwrap() as usize;
    let start: usize = src.split_inclusive('\n').take(line).map(str::len).sum();
    let text = &src[start..];
    start
        + text
            .char_indices()
            .scan(0, |u, (i, c)| {
                let here = (*u, i);
                *u += c.len_utf16();
                Some(here)
            })
            .find(|&(u, _)| u >= col)
            .map_or(text.len(), |(_, i)| i)
}

/// `src` with every renamable name renamed, and how many were.
fn rename_all(server: &mut Server, src: &str) -> (String, usize, Vec<(String, String)>) {
    let (mut text, mut version, mut renamed, mut odd) = (src.to_string(), 2, 0, Vec::new());
    let words: Vec<String> = names(src).into_iter().map(|(_, w)| w.to_string()).collect();
    for word in words {
        let Some(at) = first_use(&text, &word) else {
            continue;
        };
        let doc = obj([("uri", URI.into())]);
        let here = obj([
            ("textDocument", doc.clone()),
            ("position", position(&text, at)),
        ]);
        let prepared = ask(server, "textDocument/prepareRename", here);
        // Null, or refused (a std name): the editor offers no rename.
        if prepared.is_null() || !prepared.at(&["code"]).is_null() {
            continue;
        }
        let upper = word.starts_with(|c: char| c.is_ascii_uppercase());
        let new = if upper {
            format!("{word}Rn")
        } else {
            format!("{word}_rn")
        };
        let edit = ask(
            server,
            "textDocument/rename",
            obj([
                ("textDocument", doc),
                ("position", position(&text, at)),
                ("newName", new.as_str().into()),
            ]),
        );
        let Some(changes) = edit.at(&["changes"]).as_object() else {
            odd.push((word.clone(), format!("refused: {}", edit.at(&["message"]))));
            continue;
        };
        let mut edits: Vec<(usize, usize, String)> = changes
            .get(URI)
            .and_then(Value::as_array)
            .unwrap_or(&[])
            .iter()
            .map(|e| {
                let (s, t) = (e.at(&["range", "start"]), e.at(&["range", "end"]));
                let new = e.at(&["newText"]).as_str().unwrap_or("").to_string();
                (offset(&text, s), offset(&text, t), new)
            })
            .collect();
        edits.sort_by_key(|a| std::cmp::Reverse(a.0));
        let before = text.clone();
        for (s, t, new) in edits {
            text.replace_range(s..t, &new);
        }
        change(server, version, &text);
        version += 1;
        // Each rename is checked on its own, and undone if it broke the file.
        let after = errors(server);
        if after.is_empty() {
            renamed += 1;
        } else {
            odd.push((word.clone(), format!("-> `{new}` left {after:?}")));
            text = before;
            change(server, version, &text);
            version += 1;
        }
    }
    (text, renamed, odd)
}

fn run(dir: &Path, name: &str, src: &str, input: &Path) -> String {
    let file = dir.join(name);
    std::fs::write(&file, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&file)
        .arg(input)
        .output()
        .unwrap();
    format!("{}{}", String::from_utf8_lossy(&out.stdout), out.status)
}

#[test]
fn renaming_every_name_keeps_each_port_checking_clean_and_printing_the_same() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list =
        cpl_json::parse(&std::fs::read_to_string(root.join("corpus/hillclimb.json")).unwrap())
            .unwrap();
    let ids: Vec<String> = list
        .at(&["problems"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| p.as_str().map(str::to_string))
        .collect();
    assert!(!ids.is_empty());
    let dir = std::env::temp_dir().join(format!("cpl-rename-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // A debug build replays every fourth port; release replays them all.
    let step = if cfg!(debug_assertions) { 4 } else { 1 };
    let (mut renamed, mut ran, mut broken, mut seen) = (0, 0, Vec::new(), Vec::new());
    let replayed: Vec<&str> = ids.iter().step_by(step).map(String::as_str).collect();
    corpus_cases::make(&root, &replayed);
    for pid in replayed {
        let problem = root.join("corpus/problems").join(pid);
        let src = std::fs::read_to_string(problem.join("spec.cp")).unwrap();
        let mut server = Server::new();
        server.handle(&message(Some(1), "initialize", obj([])));
        server.handle(&message(None, "initialized", obj([])));
        let doc = obj([
            ("uri", URI.into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", src.as_str().into()),
        ]);
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        if !errors(&mut server).is_empty() {
            continue;
        }
        let (text, n, odd) = rename_all(&mut server, &src);
        renamed += n;
        for (word, why) in odd {
            let key = format!("{pid} {word}");
            if !KNOWN.contains(&key.as_str()) {
                broken.push(format!("{key}: {why}"));
            }
            seen.push(key);
        }
        let input = cpl_contest::problem::find_tests_dir(&problem.join("spec.cp")).join("01.in");
        if input.is_file() {
            let (before, now) = (
                run(&dir, "a.cp", &src, &input),
                run(&dir, "b.cp", &text, &input),
            );
            ran += 1;
            if before != now {
                broken.push(format!(
                    "{pid}: output changed\n  was {before:?}\n  now {now:?}"
                ));
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    eprintln!("rename: {renamed} names renamed, {ran} ports run before and after");
    assert!(
        renamed > 100 && ran > 3,
        "too little was renamed or run: {renamed}, {ran}"
    );
    assert!(
        broken.is_empty(),
        "rename changed a port's meaning:\n{}",
        broken.join("\n")
    );
}
