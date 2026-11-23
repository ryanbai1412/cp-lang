//! End-to-end verification of every LSP feature on edition 0.3 (`cpl lsp --stdio`
//! with no `CPL_SPEC`/`CPL_STD` env set):
//! 1. hover types + costs (spec/std methods + user fns)
//! 2. go-to-definition into `spec/std/`
//! 3. completion including `use std.` modules, members after `.`, and auto-import
//! 4. rename (user names succeed; `spec/std` names are refused)
//! 5. inlay type hints
//! 6. code actions from `cpl-lint` fixes (quickfix + `source.fixAll.cpl`)
//! 7. formatting (`textDocument/formatting`)

use cpl_json::{obj, Value};
use std::collections::BTreeMap;
use std::io::{BufReader, Write};
use std::process::{Command, Stdio};

fn frame(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{body}", body.len())
}

fn msg(id: Option<u32>, method: &str, params: Value) -> String {
    let mut m = BTreeMap::new();
    m.insert("jsonrpc".to_string(), Value::from("2.0"));
    if let Some(id) = id {
        m.insert("id".to_string(), id.into());
    }
    m.insert("method".to_string(), method.into());
    m.insert("params".to_string(), params);
    Value::Object(m).to_string()
}

fn run_session(messages: &[String]) -> (BTreeMap<u64, Value>, Vec<Value>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["lsp", "--stdio"])
        .env_remove("CPL_SPEC")
        .env_remove("CPL_STD")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cpl lsp starts");
    {
        let mut stdin = child.stdin.take().unwrap();
        for message in messages {
            stdin.write_all(frame(message).as_bytes()).unwrap();
        }
    }
    let out = child.wait_with_output().expect("cpl lsp exits");
    assert!(
        out.status.success(),
        "cpl lsp exited with {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let mut reader = BufReader::new(stdout.as_bytes());
    let mut replies = BTreeMap::new();
    let mut notifications = Vec::new();
    while let Some(body) = cpl_lsp::read_message(&mut reader).unwrap() {
        let value = cpl_json::parse(&body).unwrap();
        if let Some(id) = value.at(&["id"]).as_u64() {
            replies.insert(id, value);
        } else {
            notifications.push(value);
        }
    }
    (replies, notifications)
}

fn pos(line: u32, character: u32) -> Value {
    obj([("line", line.into()), ("character", character.into())])
}

fn doc_pos(uri: &str, line: u32, character: u32) -> Value {
    obj([
        ("textDocument", obj([("uri", uri.into())])),
        ("position", pos(line, character)),
    ])
}

fn completion_labels(reply: &Value) -> Vec<String> {
    let result = reply.at(&["result"]);
    let items = result
        .get("items")
        .unwrap_or(result)
        .as_array()
        .unwrap_or(&[]);
    items
        .iter()
        .filter_map(|i| i.at(&["label"]).as_str().map(str::to_string))
        .collect()
}

#[test]
fn every_lsp_feature_works_on_v03() {
    // Document 1: hover types+costs, go-to-def into spec/std, member & auto-import completion,
    // rename, inlay types.
    let main_uri = "file:///v03/main.cp";
    let main_src = concat!(
        "use std.heap.{MinMaxHeap}\n",                    // 0
        "let n: 1..=100 = read()\n",                      // 1
        "domain V = 0..n\n",                              // 2
        "fn tally_sum(xs: [V]Int) -> Int:\n",             // 3
        "    var acc = 0\n",                              // 4
        "    for v in V:\n",                              // 5
        "        acc += xs[v]\n",                         // 6
        "    acc\n",                                      // 7
        "var h = MinMaxHeap<Int>()\n",                    // 8
        "h.push(3)\n",                                    // 9
        "let a: [V]Int = read()\n",                       // 10
        "let total = tally_sum(a)\n",                     // 11
        "let p = bsearch_real(0.0, 1.0, x => x > 0.5)\n", // 12
        "print(total, p)\n",                              // 13
    );

    // Document 2: completion probes (`use std.`, `use std.heap.`, `h.`, `Lazy`).
    let comp_uri = "file:///v03/complete.cp";
    let comp_src = concat!(
        "use std.\n",                  // 0: col 8
        "use std.heap.{MinMaxHeap}\n", // 1
        "var h = MinMaxHeap<Int>()\n", // 2
        "h.\n",                        // 3: col 2
        "let x = Lazy\n",              // 4: col 12
    );

    // Document 3: lint fixes (code actions) + formatting.
    let fix_uri = "file:///v03/fix.cp";
    let fix_src = concat!(
        "let  n: Int=read::<Int>()\n", // 0: legacy turbofish lint + unformatted spacing
        "print(n)\n",                  // 1
    );

    let open = |uri: &str, text: &str| {
        msg(
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
        )
    };

    let session = vec![
        msg(Some(1), "initialize", obj([])),
        msg(None, "initialized", obj([])),
        open(main_uri, main_src),
        open(comp_uri, comp_src),
        open(fix_uri, fix_src),
        // 1. Hover: spec/std struct, spec/std method (with cost), user fn (with cost/loops), array
        msg(Some(10), "textDocument/hover", doc_pos(main_uri, 8, 10)), // MinMaxHeap
        msg(Some(11), "textDocument/hover", doc_pos(main_uri, 9, 3)),  // push
        msg(Some(12), "textDocument/hover", doc_pos(main_uri, 3, 5)),  // tally_sum
        msg(Some(13), "textDocument/hover", doc_pos(main_uri, 10, 4)), // a: [V]Int
        // 2. Go-to-definition into spec/std
        msg(
            Some(20),
            "textDocument/definition",
            doc_pos(main_uri, 8, 10),
        ), // MinMaxHeap -> spec/std/heap.cp
        msg(Some(21), "textDocument/definition", doc_pos(main_uri, 9, 3)), // push -> spec/std/heap.cp
        msg(
            Some(22),
            "textDocument/definition",
            doc_pos(main_uri, 12, 10),
        ), // bsearch_real -> spec/std/search.cp
        // 3. Completion: `use std.`, member `h.`, auto-import `Lazy`
        msg(Some(30), "textDocument/completion", doc_pos(comp_uri, 0, 8)), // use std.
        msg(Some(31), "textDocument/completion", doc_pos(comp_uri, 3, 2)), // h.
        msg(
            Some(32),
            "textDocument/completion",
            doc_pos(comp_uri, 4, 12),
        ), // Lazy
        // 4. Rename: user fn succeeds; spec/std symbol refused
        msg(
            Some(40),
            "textDocument/rename",
            obj([
                ("textDocument", obj([("uri", main_uri.into())])),
                ("position", pos(3, 5)),
                ("newName", "sum_all".into()),
            ]),
        ),
        msg(
            Some(41),
            "textDocument/rename",
            obj([
                ("textDocument", obj([("uri", main_uri.into())])),
                ("position", pos(8, 10)),
                ("newName", "MyHeap".into()),
            ]),
        ),
        // 5. Inlay hints
        msg(
            Some(50),
            "textDocument/inlayHint",
            obj([("textDocument", obj([("uri", main_uri.into())]))]),
        ),
        // 6. Code actions from lint fixes
        msg(
            Some(60),
            "textDocument/codeAction",
            obj([
                ("textDocument", obj([("uri", fix_uri.into())])),
                ("range", obj([("start", pos(0, 0)), ("end", pos(0, 26))])),
                ("context", obj([("diagnostics", Value::Array(vec![]))])),
            ]),
        ),
        // 7. Formatting
        msg(
            Some(70),
            "textDocument/formatting",
            obj([
                ("textDocument", obj([("uri", fix_uri.into())])),
                (
                    "options",
                    obj([("tabSize", 4.into()), ("insertSpaces", true.into())]),
                ),
            ]),
        ),
        msg(Some(99), "shutdown", Value::Null),
        msg(None, "exit", Value::Null),
    ];

    let (replies, _notifs) = run_session(&session);

    // 1. Hover types + costs
    let hover = |id: u64| {
        replies[&id]
            .at(&["result", "contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    let h_struct = hover(10);
    assert!(
        h_struct.contains("struct MinMaxHeap<T>"),
        "hover struct: {h_struct}"
    );
    let h_push = hover(11);
    assert!(
        h_push.contains("cost: O(log n)"),
        "hover method cost: {h_push}"
    );
    let h_fn = hover(12);
    assert!(h_fn.contains("fn tally_sum"), "hover user fn: {h_fn}");
    assert!(
        h_fn.contains("loops:") || h_fn.contains("estimated cost:"),
        "hover user fn cost: {h_fn}"
    );
    let h_arr = hover(13);
    assert!(h_arr.contains("a: [V]Int"), "hover array type: {h_arr}");
    assert!(h_arr.contains("storage:"), "hover array storage: {h_arr}");

    // 2. Go-to-definition into spec/std
    let def_uri = |id: u64| {
        replies[&id]
            .at(&["result", "uri"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    assert!(
        def_uri(20).ends_with("/spec/std/heap.cp"),
        "def MinMaxHeap: {}",
        replies[&20]
    );
    assert!(
        def_uri(21).ends_with("/spec/std/heap.cp"),
        "def h.push: {}",
        replies[&21]
    );
    assert!(
        def_uri(22).ends_with("/spec/std/search.cp"),
        "def bsearch_real: {}",
        replies[&22]
    );

    // 3. Completion incl. members & v0.3 modules
    let mods = completion_labels(&replies[&30]);
    for wanted in ["io", "opt", "pair", "search", "subset", "heap"] {
        assert!(
            mods.iter().any(|m| m == wanted),
            "use std. missing {wanted}: {mods:?}"
        );
    }
    for removed in ["tour"] {
        assert!(
            !mods.iter().any(|m| m == removed),
            "use std. has v0.2-only {removed}: {mods:?}"
        );
    }
    let members = completion_labels(&replies[&31]);
    for wanted in ["push", "pop_first", "pop_last", "len"] {
        assert!(
            members.iter().any(|m| m == wanted),
            "h. missing {wanted}: {members:?}"
        );
    }
    let auto_imports = completion_labels(&replies[&32]);
    assert!(
        auto_imports.iter().any(|m| m == "LazyTreap"),
        "Lazy missing LazyTreap: {auto_imports:?}"
    );
    assert!(
        !auto_imports.iter().any(|m| m == "LazySegTree"),
        "Lazy leaked v0.2 LazySegTree: {auto_imports:?}"
    );

    // 4. Rename
    let rename_edits = replies[&40]
        .at(&["result", "changes", main_uri])
        .as_array()
        .expect("rename returns edits for user fn");
    assert_eq!(
        rename_edits.len(),
        2,
        "expected decl + call site edits: {rename_edits:?}"
    );
    assert!(
        rename_edits
            .iter()
            .all(|e| e.at(&["newText"]).as_str() == Some("sum_all")),
        "{rename_edits:?}"
    );
    let std_rename_err = replies[&41]
        .at(&["error", "message"])
        .as_str()
        .unwrap_or("");
    assert!(
        std_rename_err.contains("standard library"),
        "expected stdlib rename refusal, got: {}",
        replies[&41]
    );

    // 5. Inlay hints
    let hints: Vec<String> = replies[&50]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|h| h.at(&["label"]).as_str().map(str::to_string))
        .collect();
    assert!(
        hints.iter().any(|h| h.contains("MinMaxHeap<Int>")),
        "inlay hints missing MinMaxHeap<Int>: {hints:?}"
    );
    assert!(
        hints.iter().any(|h| h == ": V"),
        "inlay hints missing loop domain `: V`: {hints:?}"
    );
    assert!(
        hints.iter().any(|h| h.starts_with(": Int")),
        "inlay hints missing `: Int`: {hints:?}"
    );

    // 6. Code actions from lint fixes
    let actions = replies[&60]
        .at(&["result"])
        .as_array()
        .expect("codeAction array");
    assert!(
        actions
            .iter()
            .any(|a| a.at(&["kind"]).as_str() == Some("quickfix")
                && a.at(&["edit", "changes", fix_uri])
                    .as_array()
                    .is_some_and(|edits| !edits.is_empty())),
        "expected lint quickfix action: {actions:?}"
    );
    assert!(
        actions
            .iter()
            .any(|a| a.at(&["kind"]).as_str() == Some("source.fixAll.cpl")),
        "expected source.fixAll.cpl action: {actions:?}"
    );

    // 7. Formatting
    let fmt_edits = replies[&70]
        .at(&["result"])
        .as_array()
        .expect("formatting edits array");
    assert_eq!(
        fmt_edits.len(),
        1,
        "expected one full-doc formatting edit: {fmt_edits:?}"
    );
    let formatted = fmt_edits[0].at(&["newText"]).as_str().unwrap_or("");
    assert!(
        formatted.contains("let n: Int = read::<Int>()"),
        "unexpected formatted text: {formatted:?}"
    );
}
