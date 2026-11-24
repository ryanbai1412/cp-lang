//! Diagnostics after a sequence of edits equal a fresh open: each hill-climb
//! port (`corpus/hillclimb.json`) is typed in line by line, with a line cut
//! and put back on the way (compared there too, with its errors), into one server that also holds the other ports;
//! the diagnostics it then reports equal those of a new server opening the
//! final text.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
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

fn started() -> Server {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    server
}

fn open(server: &mut Server, uri: &str, text: &str) {
    server.handle(&message(
        None,
        "textDocument/didOpen",
        obj([(
            "textDocument",
            obj([
                ("uri", uri.into()),
                ("languageId", "cp".into()),
                ("version", 1.into()),
                ("text", text.into()),
            ]),
        )]),
    ));
}

fn change(server: &mut Server, uri: &str, version: u32, text: &str) {
    server.handle(&message(
        None,
        "textDocument/didChange",
        obj([
            (
                "textDocument",
                obj([("uri", uri.into()), ("version", version.into())]),
            ),
            (
                "contentChanges",
                Value::Array(vec![obj([("text", text.into())])]),
            ),
        ]),
    ));
}

/// The pulled diagnostics, as text so two servers' answers compare.
fn diagnostics(server: &mut Server, uri: &str) -> Vec<String> {
    let reply = server.handle(&message(
        Some(2),
        "textDocument/diagnostic",
        obj([("textDocument", obj([("uri", uri.into())]))]),
    ));
    let items = reply
        .iter()
        .find_map(|r| r.at(&["result", "items"]).as_array())
        .unwrap_or(&[]);
    let mut out: Vec<String> = items
        .iter()
        .map(|d| {
            format!(
                "{} {} {}",
                d.at(&["code"]),
                d.at(&["range"]),
                d.at(&["message"]).as_str().unwrap_or("")
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn diagnostics_after_edits_equal_a_fresh_open() {
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
    assert!(!ids.is_empty());
    // A debug build replays every fourth port; release replays them all.
    let step = if cfg!(debug_assertions) { 4 } else { 1 };
    let mut edited = started();
    let mut differ = Vec::new();
    let mut broken = 0;
    for pid in ids.iter().step_by(step) {
        let src =
            std::fs::read_to_string(root.join(format!("corpus/problems/{pid}/spec.cp"))).unwrap();
        let uri = format!("file:///edits/{pid}.cp");
        open(&mut edited, &uri, "");
        let lines: Vec<&str> = src.split_inclusive('\n').collect();
        let mut version = 2;
        for n in 1..=lines.len() {
            change(&mut edited, &uri, version, &lines[..n].concat());
            version += 1;
            if n == lines.len() / 2 && n > 1 {
                // Cut the line before, check, and put it back.
                let cut = [&lines[..n - 2], &lines[n - 1..n]].concat().concat();
                change(&mut edited, &uri, version, &cut);
                let mut fresh = started();
                open(&mut fresh, &uri, &cut);
                let (cut_edited, cut_fresh) = (
                    diagnostics(&mut edited, &uri),
                    diagnostics(&mut fresh, &uri),
                );
                broken += usize::from(!cut_fresh.is_empty());
                if cut_edited != cut_fresh {
                    differ.push(format!(
                        "{pid} cut:\n  edited {cut_edited:?}\n  fresh  {cut_fresh:?}"
                    ));
                }
                version += 1;
                change(&mut edited, &uri, version, &lines[..n].concat());
                version += 1;
            }
        }
        let after_edits = diagnostics(&mut edited, &uri);
        let mut fresh = started();
        open(&mut fresh, &uri, &src);
        let fresh_open = diagnostics(&mut fresh, &uri);
        if after_edits != fresh_open {
            differ.push(format!(
                "{pid}:\n  edited {after_edits:?}\n  fresh  {fresh_open:?}"
            ));
        }
    }
    assert!(
        broken > 0,
        "no cut left a port with a diagnostic to compare"
    );
    assert!(
        differ.is_empty(),
        "diagnostics after edits differ from a fresh open:\n{}",
        differ.join("\n")
    );
}
