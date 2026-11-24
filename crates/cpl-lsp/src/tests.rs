use crate::{serve, write_message, Server};
use cpl_json::{self as json, obj, Value};

mod complete;
mod fixes;
mod format;
mod hover;
mod nav;
mod parity;
mod rename;

const URI: &str = "file:///work/roads.cp";

const SOURCE: &str = "\
let n: 1..=2e5 = read()
domain V = 1..=n
domain E = 0..n
let dist: [V]?Int = None
var total = 0
for v in V:
    total ^= dist[v] ?? 0
print(total)
";

fn request(id: u32, method: &str, params: Value) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("id", id.into()),
        ("method", method.into()),
        ("params", params),
    ])
}

fn notification(method: &str, params: Value) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", method.into()),
        ("params", params),
    ])
}

fn doc() -> Value {
    obj([("uri", URI.into())])
}

fn pos(line: u32, character: u32) -> Value {
    obj([("line", line.into()), ("character", character.into())])
}

fn did_open(text: &str) -> Value {
    notification(
        "textDocument/didOpen",
        obj([(
            "textDocument",
            obj([
                ("uri", URI.into()),
                ("languageId", "cp".into()),
                ("version", 1.into()),
                ("text", text.into()),
            ]),
        )]),
    )
}

/// A ranged edit: replace `start..end` with `text`.
fn did_change(start: (u32, u32), end: (u32, u32), text: &str) -> Value {
    notification(
        "textDocument/didChange",
        obj([
            (
                "textDocument",
                obj([("uri", URI.into()), ("version", 2.into())]),
            ),
            (
                "contentChanges",
                vec![obj([
                    (
                        "range",
                        obj([("start", pos(start.0, start.1)), ("end", pos(end.0, end.1))]),
                    ),
                    ("text", text.into()),
                ])]
                .into(),
            ),
        ]),
    )
}

/// A server that has been initialized and has `text` open. Returns the
/// diagnostics published for it.
fn open(text: &str) -> (Server, Vec<Value>) {
    let mut server = Server::new();
    let reply = server.handle(&request(1, "initialize", obj([])));
    assert_eq!(reply.len(), 1);
    assert!(server
        .handle(&notification("initialized", obj([])))
        .is_empty());
    let published = server.handle(&did_open(text));
    (server, diagnostics_of(&published))
}

fn diagnostics_of(messages: &[Value]) -> Vec<Value> {
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert_eq!(
        messages[0].at(&["method"]).as_str(),
        Some("textDocument/publishDiagnostics")
    );
    assert_eq!(messages[0].at(&["params", "uri"]).as_str(), Some(URI));
    messages[0]
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[])
        .to_vec()
}

fn hover_at(server: &mut Server, line: u32, character: u32) -> Value {
    let reply = server.handle(&request(
        7,
        "textDocument/hover",
        obj([("textDocument", doc()), ("position", pos(line, character))]),
    ));
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].at(&["id"]).as_u32(), Some(7));
    reply[0].at(&["result"]).clone()
}

#[test]
fn initialize_announces_what_the_server_can_do() {
    let mut server = Server::new();
    let reply = server.handle(&request(1, "initialize", obj([])));
    let caps = reply[0].at(&["result", "capabilities"]);
    assert_eq!(caps.at(&["textDocumentSync", "change"]).as_u32(), Some(2));
    assert_eq!(caps.at(&["hoverProvider"]), &Value::Bool(true));
    assert_eq!(caps.at(&["definitionProvider"]), &Value::Bool(true));
    assert_eq!(caps.at(&["documentSymbolProvider"]), &Value::Bool(true));
    assert_eq!(
        reply[0].at(&["result", "serverInfo", "name"]).as_str(),
        Some("cpl")
    );
}

#[test]
fn requests_before_initialize_are_refused() {
    let mut server = Server::new();
    let reply = server.handle(&request(1, "textDocument/hover", obj([])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32002));
    assert!(server.handle(&did_open(SOURCE)).is_empty());
}

#[test]
fn opening_a_clean_file_publishes_no_diagnostics() {
    let (_, diagnostics) = open(SOURCE);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn diagnostics_carry_range_code_help_and_related_spans() {
    let (_, diagnostics) = open(
        &SOURCE
            .replace("dist[v]", "dist[w]")
            .replace("print(total)", "n = 1"),
    );
    assert_eq!(diagnostics.len(), 2);
    let d = &diagnostics[0];
    assert_eq!(d.at(&["code"]).as_str(), Some("E0201"));
    assert_eq!(d.at(&["source"]).as_str(), Some("cpl"));
    assert_eq!(d.at(&["severity"]).as_u32(), Some(1));
    assert_eq!(
        d.at(&["message"]).as_str(),
        Some("no `w` in scope\nhelp: declare it first: `let w = todo`")
    );
    assert_eq!(d.at(&["range", "start"]), &pos(6, 18));
    assert_eq!(d.at(&["range", "end"]), &pos(6, 19));
    let d = &diagnostics[1];
    assert_eq!(d.at(&["code"]).as_str(), Some("E0401"));
    assert_eq!(d.at(&["range", "start"]), &pos(7, 0));

    let (_, diagnostics) = open("type R = Int\ntype R = Bool\n");
    let related = diagnostics[0]
        .at(&["relatedInformation"])
        .as_array()
        .unwrap_or(&[]);
    assert_eq!(
        related[0].at(&["message"]).as_str(),
        Some("first defined here")
    );
    assert_eq!(related[0].at(&["location", "uri"]).as_str(), Some(URI));
    assert_eq!(related[0].at(&["location", "range", "start"]), &pos(0, 5));
}

#[test]
fn typing_updates_diagnostics_after_every_edit() {
    let (mut server, _) = open(SOURCE);
    // Type `print(dist[` at the end of the file: an unclosed bracket.
    let published = server.handle(&did_change((8, 0), (8, 0), "print(dist["));
    let diagnostics = diagnostics_of(&published);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].at(&["code"]).as_str(), Some("E0101"));
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(8, 5));
    // Finish the line: clean again.
    let published = server.handle(&did_change((8, 11), (8, 11), "V.first!] ?? 0)\n"));
    assert!(diagnostics_of(&published).is_empty());
    // Break a line in the middle of the file by deleting `in V`.
    let published = server.handle(&did_change((5, 5), (5, 10), ""));
    let diagnostics = diagnostics_of(&published);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].at(&["code"]).as_str(), Some("E0103"));
}

#[test]
fn full_document_changes_are_accepted_too() {
    let (mut server, _) = open(SOURCE);
    let change = notification(
        "textDocument/didChange",
        obj([
            ("textDocument", doc()),
            (
                "contentChanges",
                vec![obj([("text", "let x: Bool = 1\n".into())])].into(),
            ),
        ]),
    );
    let diagnostics = diagnostics_of(&server.handle(&change));
    assert_eq!(diagnostics[0].at(&["code"]).as_str(), Some("E0302"));
}

#[test]
fn edits_use_utf16_positions() {
    // Where `+` can't combine its operands. The literal is also reported
    // for not being ASCII; that is not what this test is about.
    fn bad_operands(diagnostics: &[Value]) -> &Value {
        diagnostics
            .iter()
            .find(|d| d.at(&["code"]).as_str() == Some("E0309"))
            .expect("a diagnostic for `+`")
            .at(&["range", "start"])
    }
    // '𝄞' is two UTF-16 units: it spans characters 9 to 11.
    let (mut server, diagnostics) = open("let a = \"𝄞\" + 1\n");
    assert_eq!(bad_operands(&diagnostics), &pos(0, 13));
    // Replace exactly the clef with three ASCII letters.
    let published = server.handle(&did_change((0, 9), (0, 11), "abc"));
    let diagnostics = diagnostics_of(&published);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(0, 14));

    let (_, diagnostics) = open("let s = \"é𝄞\" + 1\n");
    assert_eq!(bad_operands(&diagnostics), &pos(0, 14));
}

#[test]
fn hover_shows_type_and_cost_with_its_range() {
    let (mut server, _) = open(SOURCE);
    let hover = hover_at(&mut server, 6, 13);
    assert_eq!(hover.at(&["contents", "kind"]).as_str(), Some("markdown"));
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\ndist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 B per element, 1.6 MB max (None is a spare value: no flag)\n```")
    );
    assert_eq!(hover.at(&["range", "start"]), &pos(6, 13));
    assert_eq!(hover.at(&["range", "end"]), &pos(6, 17));
    assert!(hover_at(&mut server, 6, 10).is_null());
}

#[test]
fn representation_query_gives_inlay_hints_the_hover_storage_data() {
    let (mut server, _) = open(SOURCE);
    let offset = SOURCE.find("dist[v]").unwrap() as u32;
    let rep = server.db.representation(URI, offset).unwrap();
    assert_eq!(rep.ty, "[V]?Int");
    assert_eq!(
        (rep.bytes, rep.max_cells, rep.max_bytes),
        (Some(8), Some(200_000), Some(1_600_000))
    );
    assert_eq!(rep.none_is_spare, Some(true));
}

#[test]
fn hover_keeps_working_while_the_buffer_is_broken() {
    let (mut server, _) = open(SOURCE);
    // Leave a call open on the last line and a statement half-typed above it.
    server.handle(&did_change((7, 11), (7, 12), ""));
    let published = server.handle(&did_change((4, 13), (4, 13), " +"));
    assert_eq!(diagnostics_of(&published).len(), 2);
    let hover = hover_at(&mut server, 6, 13);
    assert!(hover
        .at(&["contents", "value"])
        .as_str()
        .unwrap()
        .contains("dist: [V]?Int"));
    let hover = hover_at(&mut server, 7, 7);
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\ntotal: Int\n```")
    );
}

#[test]
fn definition_and_outline() {
    let (mut server, _) = open(SOURCE);
    let reply = server.handle(&request(
        3,
        "textDocument/definition",
        obj([("textDocument", doc()), ("position", pos(6, 14))]),
    ));
    assert_eq!(reply[0].at(&["result", "uri"]).as_str(), Some(URI));
    assert_eq!(reply[0].at(&["result", "range", "start"]), &pos(3, 4));
    assert_eq!(reply[0].at(&["result", "range", "end"]), &pos(3, 8));

    let reply = server.handle(&request(
        4,
        "textDocument/documentSymbol",
        obj([("textDocument", doc())]),
    ));
    let names: Vec<&str> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|s| s.at(&["name"]).as_str())
        .collect();
    assert_eq!(names, vec!["n", "V", "E", "dist", "total"]);
    assert_eq!(
        reply[0].at(&["result"]).as_array().unwrap_or(&[])[1]
            .at(&["kind"])
            .as_u32(),
        Some(26)
    );
}

/// `text` with the edits of the first quick fix for its first diagnostic.
fn quick_fixed(server: &mut Server, uri: &str, text: &str, diagnostic: &Value) -> (String, String) {
    let params = obj([
        ("textDocument", obj([("uri", uri.into())])),
        ("range", diagnostic.at(&["range"]).clone()),
        (
            "context",
            obj([("diagnostics", vec![diagnostic.clone()].into())]),
        ),
    ]);
    let reply = server.handle(&request(9, "textDocument/codeAction", params));
    let actions = reply[0].at(&["result"]).as_array().unwrap_or(&[]);
    let action = actions
        .first()
        .unwrap_or_else(|| panic!("no fix for {diagnostic:?}"));
    let offset = |p: &Value| {
        let line = p.at(&["line"]).as_u32().unwrap() as usize;
        let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
        start + p.at(&["character"]).as_u32().unwrap() as usize
    };
    let mut edits: Vec<(usize, usize, String)> = action
        .at(&["edit", "changes", uri])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|e| {
            let (start, end) = (
                offset(e.at(&["range", "start"])),
                offset(e.at(&["range", "end"])),
            );
            (start, end, e.at(&["newText"]).as_str().unwrap().to_string())
        })
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut fixed = text.to_string();
    for (start, end, new) in edits {
        fixed.replace_range(start..end, &new);
    }
    (action.at(&["title"]).as_str().unwrap().to_string(), fixed)
}

/// Each `(good, bad, code, title)` breaks the ecnerwala `port` by replacing
/// `good` with `bad`; its first diagnostic is `code` and the first quick
/// fix, titled `title`, restores the port.
fn assert_fixes_restore(port: &str, breaks: &[(&str, &str, &str, &str)]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../corpus/problems/{port}/port.cp"));
    let original = std::fs::read_to_string(&path).unwrap();
    let uri = format!("file://{}", path.display());
    assert_fixes_restore_at(&uri, &original, breaks);
}

fn assert_fixes_restore_src(original: &str, breaks: &[(&str, &str, &str, &str)]) {
    assert_fixes_restore_at(URI, original, breaks);
}

fn assert_fixes_restore_at(uri: &str, original: &str, breaks: &[(&str, &str, &str, &str)]) {
    for (good, bad, code, title) in breaks {
        assert!(original.contains(good), "{good}");
        let broken = original.replacen(good, bad, 1);
        let mut server = Server::new();
        server.handle(&request(1, "initialize", obj([])));
        let doc = obj([
            ("uri", uri.into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", broken.as_str().into()),
        ]);
        let published = server.handle(&notification(
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        let diagnostics = published[0]
            .at(&["params", "diagnostics"])
            .as_array()
            .unwrap_or(&[]);
        let first = diagnostics
            .first()
            .unwrap_or_else(|| panic!("`{bad}` checked clean"));
        assert_eq!(first.at(&["code"]).as_str(), Some(*code), "{first:?}");
        let (got, fixed) = quick_fixed(&mut server, uri, &broken, first);
        assert_eq!(got, *title);
        assert_eq!(
            fixed, original,
            "the fix for `{bad}` did not restore the port"
        );
    }
}

#[test]
fn closing_a_document_clears_its_diagnostics() {
    let (mut server, diagnostics) = open("let x = y\n");
    assert_eq!(diagnostics.len(), 1);
    let published = server.handle(&notification(
        "textDocument/didClose",
        obj([("textDocument", doc())]),
    ));
    assert!(diagnostics_of(&published).is_empty());
    let reply = server.handle(&request(
        9,
        "textDocument/hover",
        obj([("textDocument", doc()), ("position", pos(0, 4))]),
    ));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32602));
}

#[test]
fn unknown_and_malformed_messages_get_errors_not_crashes() {
    let (mut server, _) = open(SOURCE);
    let reply = server.handle(&request(2, "textDocument/selectionRange", obj([])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32601));
    assert!(server
        .handle(&notification("workspace/didChangeWatchedFiles", obj([])))
        .is_empty());
    let reply = server.handle(&request(
        3,
        "textDocument/hover",
        obj([("textDocument", doc())]),
    ));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32602));
    let reply = server.handle(&request(4, "textDocument/hover", Value::Null));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32602));
    // Positions far outside the document clamp.
    assert!(hover_at(&mut server, 9999, 9999).is_null());
    // A change for a document that is not open.
    let mut other = did_change((0, 0), (0, 0), "x");
    if let Value::Object(members) = &mut other {
        members.insert(
            "params".to_string(),
            obj([
                ("textDocument", obj([("uri", "file:///nope.cp".into())])),
                ("contentChanges", Value::Array(Vec::new())),
            ]),
        );
    }
    assert!(server.handle(&other).is_empty());
    // The server still works.
    assert!(!hover_at(&mut server, 6, 13).is_null());
}

#[test]
fn a_panic_is_contained_and_reported() {
    let (mut server, _) = open(SOURCE);
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let reply = server.handle(&request(8, "cpl/testPanic", obj([("textDocument", doc())])));
    std::panic::set_hook(quiet);
    assert_eq!(reply.len(), 2);
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Int(-32603));
    assert!(reply[0]
        .at(&["error", "message"])
        .as_str()
        .unwrap()
        .contains("internal error in cpl while handling `cpl/testPanic`: deliberate test panic"));
    let diagnostics = diagnostics_of(&reply[1..]);
    assert_eq!(diagnostics[0].at(&["code"]).as_str(), Some("E9999"));
    // The open document survived, and the next request is answered.
    assert!(hover_at(&mut server, 6, 13)
        .at(&["contents", "value"])
        .as_str()
        .unwrap()
        .contains("dist: [V]?Int"));
}

#[test]
fn shutdown_then_exit_ends_cleanly() {
    let (mut server, _) = open(SOURCE);
    let reply = server.handle(&request(10, "shutdown", Value::Null));
    assert_eq!(reply[0].at(&["result"]), &Value::Null);
    assert_eq!(server.exit_code(), None);
    server.handle(&notification("exit", Value::Null));
    assert_eq!(server.exit_code(), Some(0));

    let mut abrupt = Server::new();
    abrupt.handle(&notification("exit", Value::Null));
    assert_eq!(abrupt.exit_code(), Some(1));
}

/// Frames `messages` the way a client writes them.
fn framed(messages: &[Value]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for message in messages {
        write_message(&mut bytes, message).unwrap();
    }
    bytes
}

/// Splits a server's output back into messages.
fn unframed(bytes: &[u8]) -> Vec<Value> {
    let mut input = std::io::BufReader::new(bytes);
    let mut out = Vec::new();
    while let Some(body) = crate::read_message(&mut input).unwrap() {
        out.push(json::parse(&body).unwrap());
    }
    out
}

#[test]
fn a_whole_session_over_the_wire() {
    let input = framed(&[
        request(1, "initialize", obj([])),
        notification("initialized", obj([])),
        did_open(&SOURCE.replace("dist[v]", "dist[n]")),
        request(
            2,
            "textDocument/hover",
            obj([("textDocument", doc()), ("position", pos(1, 7))]),
        ),
        request(3, "shutdown", Value::Null),
        notification("exit", Value::Null),
    ]);
    let mut output = Vec::new();
    let code = serve(std::io::BufReader::new(&input[..]), &mut output).unwrap();
    assert_eq!(code, 0);
    let replies = unframed(&output);
    assert_eq!((replies.len(), &replies[2]), (5, &replies[1]));
    assert_eq!(replies[0].at(&["id"]).as_u32(), Some(1));
    let diagnostics = replies[1]
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[]);
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].at(&["message"]).as_str(),
        Some("`dist` is indexed by V, but `n` is a 1..=2e5\nhelp: convert it with `V(n)`, or write `dist.raw[n]` to index by number")
    );
    assert_eq!(
        replies[3].at(&["result", "contents", "value"]).as_str(),
        Some("```cp\ndomain V = 1..=n\n  at most 2e5 positions (n <= 2e5)\n  storage: 4 B per position\n```")
    );
    assert_eq!(replies[4].at(&["id"]).as_u32(), Some(3));
}

#[test]
fn bad_frames_and_bad_json_do_not_stop_the_server() {
    let mut input = b"Content-Length: 9\r\n\r\n{not json".to_vec();
    input.extend(framed(&[request(1, "initialize", obj([]))]));
    let mut output = Vec::new();
    // The input ends without `exit`: that is exit code 1.
    assert_eq!(
        serve(std::io::BufReader::new(&input[..]), &mut output).unwrap(),
        1
    );
    let replies = unframed(&output);
    assert_eq!(replies[0].at(&["error", "code"]), &Value::Int(-32700));
    assert_eq!(replies[1].at(&["id"]).as_u32(), Some(1));

    let mut output = Vec::new();
    assert!(serve(
        std::io::BufReader::new(&b"Content-Type: x\r\n\r\n"[..]),
        &mut output
    )
    .is_err());
}

#[test]
fn imported_module_symbols_hover_and_jump_to_definition() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let file_path = repo_root.join("_lsp_imported_test.cp");
    let uri = format!("file://{}", file_path.display());
    let src = "use std.rand.{Rng}\nvar r = Rng.new(7)\nprint(r.int(1, 6))\n";

    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    server.handle(&notification("initialized", obj([])));
    let published = server.handle(&notification(
        "textDocument/didOpen",
        obj([(
            "textDocument",
            obj([
                ("uri", uri.as_str().into()),
                ("languageId", "cp".into()),
                ("version", Value::Int(1)),
                ("text", src.into()),
            ]),
        )]),
    ));
    assert_eq!(published.len(), 1);
    let diags = published[0]
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[]);
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");

    let hover = server.handle(&request(
        2,
        "textDocument/hover",
        obj([
            ("textDocument", obj([("uri", uri.as_str().into())])),
            ("position", pos(1, 9)),
        ]),
    ));
    let hover_text = hover[0]
        .at(&["result", "contents", "value"])
        .as_str()
        .unwrap_or("");
    assert!(
        hover_text.contains("Rng"),
        "expected Rng hover, got: {hover_text}"
    );

    let def = server.handle(&request(
        3,
        "textDocument/definition",
        obj([
            ("textDocument", obj([("uri", uri.as_str().into())])),
            ("position", pos(1, 9)),
        ]),
    ));
    let def_uri = def[0].at(&["result", "uri"]).as_str().unwrap_or("");
    assert!(
        def_uri.ends_with("std/rand.cp"),
        "expected definition in std/rand.cp, got: {def_uri}"
    );
}

#[test]
fn stdlib_methods_hover_with_their_docs_and_jump_into_the_stdlib() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().and_then(|p| p.parent()).unwrap();
    let uri = format!(
        "file://{}",
        repo_root.join("_lsp_method_doc_test.cp").display()
    );
    let src =
        "use std.mod.{Mod, Comb}\nlet c = Comb<Mod<1000000007>>.new(10)\nprint(c.choose(5, 2).v)\n";
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    server.handle(&notification("initialized", obj([])));
    let text_document = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", Value::Int(1)),
        ("text", src.into()),
    ]);
    server.handle(&notification(
        "textDocument/didOpen",
        obj([("textDocument", text_document)]),
    ));
    let at = |line, character| {
        obj([
            ("textDocument", obj([("uri", uri.as_str().into())])),
            ("position", pos(line, character)),
        ])
    };
    let hover = server.handle(&request(2, "textDocument/hover", at(2, 9)));
    let text = hover[0]
        .at(&["result", "contents", "value"])
        .as_str()
        .unwrap();
    assert!(text.starts_with("```cp\nchoose: (n: Int, k: Int) -> Mod<"));
    let doc = text.split_once("```\n\n").unwrap().1;
    assert!(
        doc.starts_with(
            "The number of `k`-subsets of `n` things, `n! / (k! (n - k)!)`; 0 when `k` is outside `0..=n`."
        ),
        "{doc}"
    );
    assert!(doc.contains("Lucas's theorem"), "{doc}");
    let checked = server.db.stats().items_checked;
    let def = server.handle(&request(3, "textDocument/definition", at(2, 9)));
    assert_eq!(server.db.stats().items_checked, checked, "rechecked std");
    assert!(def[0]
        .at(&["result", "uri"])
        .as_str()
        .unwrap()
        .ends_with("std/mod.cp"));
    let choose_line = include_str!("../../../std/mod.cp")
        .lines()
        .position(|l| l.trim_start().starts_with("pub fn choose("))
        .unwrap() as u32;
    let range = obj([
        ("start", pos(choose_line, 11)),
        ("end", pos(choose_line, 17)),
    ]);
    assert_eq!(def[0].at(&["result", "range"]), &range);
}

const V22: &str = "\
let (n, x): (1..=100, 1..=1e6) = read()
let coins: [0..n]Int = read()
var dp: [0..=x]?Int = None
dp[0] = 0
for s in 1..=x:
    for c in coins:
        if c <= s: dp[s] min= dp[s - c] + 1
let best = dp[x] - 1
print(best ?? -1)
";

/// The line and message of every diagnostic, in order.
fn lines_and_messages(diagnostics: &[Value]) -> Vec<(u32, String)> {
    diagnostics
        .iter()
        .map(|d| {
            let line = d.at(&["range", "start", "line"]).as_u32().unwrap();
            (line, d.at(&["message"]).as_str().unwrap().to_string())
        })
        .collect()
}

#[test]
fn v22_optionals_hover_as_optionals_and_check_clean() {
    let (mut server, diagnostics) = open(V22);
    assert_eq!(diagnostics, Vec::<Value>::new());
    // `dp[x] - 1` is `none` when `dp[x]` is.
    let hover = hover_at(&mut server, 7, 4);
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\nbest: ?Int\n  storage: 8 B (None is a spare value: no flag)\n```")
    );
    assert_eq!(hover.at(&["range", "start"]), &pos(7, 4));
    assert_eq!(hover.at(&["range", "end"]), &pos(7, 8));
    let hover = hover_at(&mut server, 2, 4);
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some(
            "```cp\ndp: [0..=x]?Int\n  storage: 8 B per element, 8 MB max (None is a spare value: no flag)\n```"
        )
    );
}

#[test]
fn v22_removed_forms_are_reported_with_what_to_write() {
    let (_, diagnostics) = open(
        &V22.replace("dp[s] min= dp[s - c] + 1", "dp[s] += 1")
            .replace("print(best ?? -1)", "print(best ?? 0 + 1)\ndefer print(1)"),
    );
    for d in &diagnostics {
        assert_eq!(d.at(&["severity"]).as_u32(), Some(1));
        assert_eq!(d.at(&["source"]).as_str(), Some("cpl"));
    }
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|d| d.at(&["code"]).as_str().unwrap())
        .collect();
    assert_eq!(codes, ["E0309", "E0110", "E0503"]);
    let found = lines_and_messages(&diagnostics);
    let expected = [
        (6, "`+=` can't be applied to an ?Int\nhelp: write `x = x + y` when `None` should stay `None`"),
        (8, "`??` mixed with `+ - * / %` needs parentheses\nhelp: write `(best ?? 0) + 1`"),
        (9, "`defer` was removed in CP Lang 0.2.2\nhelp: undo by hand after the call"),
    ];
    assert_eq!(found.len(), expected.len(), "{found:?}");
    for ((line, message), (want_line, want)) in found.iter().zip(expected) {
        assert_eq!(*line, want_line, "{message}");
        assert!(message.starts_with(want), "{message}");
    }

    let (_, diagnostics) = open(&V22.replace("for c in coins", "for mut c in coins"));
    let found = lines_and_messages(&diagnostics);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, 5);
    assert!(found[0]
        .1
        .starts_with("`for mut` was removed in CP Lang 0.2.2\nhelp: loop over the indexes"));
}

#[test]
fn inlay_hints_show_the_types_that_are_not_written() {
    let (mut server, _) =
        open("let n = 3\nlet m: Int = 4\nvar xs = [n, m]\nfor x in xs:\n    print(x)\n");
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let hints = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    let shown: Vec<(u32, u32, &str)> = hints
        .iter()
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.at(&["label"]).as_str().unwrap(),
            )
        })
        .collect();
    // `m` has its type written, so it gets none.
    assert_eq!(
        shown,
        [(0, 5, ": 3..=3"), (2, 6, ": List<Int>"), (3, 5, ": Int")]
    );
}

#[test]
fn inlay_hints_skip_names_in_a_typed_tuple() {
    let (mut server, _) = open("let (m, k): (Int, Int) = (1, 2)\nlet (a, b) = (m, k)\nfor (i, x) in [(0, a)]:\n    print(i + x + b)\n");
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let shown: Vec<(u32, u32, String)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.at(&["label"]).as_str().unwrap().to_string(),
            )
        })
        .collect();
    let at = |line, character| (line, character, ": Int".to_string());
    assert_eq!(shown, [at(1, 6), at(1, 9), at(2, 6), at(2, 9)]);
}

#[test]
fn inlay_hints_show_the_index_space_of_lambda_parameters() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/problems/cf2062g/port.cp");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    let uri = format!("file://{}", path.display());
    let opened = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", 1.into()),
        ("text", text.as_str().into()),
    ]);
    server.handle(&notification(
        "textDocument/didOpen",
        obj([("textDocument", opened)]),
    ));
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
    ));
    let line = text.lines().position(|l| l.contains("(i, j) =>")).unwrap() as u32;
    let shown: Vec<(u32, String)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|hint| hint.at(&["position", "line"]).as_u32() == Some(line))
        .map(|hint| {
            (
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.at(&["label"]).as_str().unwrap().to_string(),
            )
        })
        .collect();
    // `let (_, mate) = min_weight_matching<I, I>((i, j) => ...)`: the
    // matching's `[I]I`, and both lambda parameters range over `I`.
    let hint = |at: u32, label: &str| (at, label.to_string());
    assert_eq!(
        shown,
        [hint(12, ": [I]I"), hint(44, ": I"), hint(47, ": I")]
    );
    // A fn's parameters have their types written, so they get none.
    let (mut server, _) = open(
        "fn twice(x: Int) -> Int = 2 * x\nfn apply(f: Int -> Int, v: Int) -> Int = f(v)\nprint(apply(y => twice(y), 3))\n",
    );
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let labels: Vec<(u32, &str)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|hint| hint.at(&["kind"]).as_u32() == Some(1))
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.at(&["label"]).as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(labels, [(2, ": Int")]);
}

/// Opens `src` with the cursor at `|` (removed) and sends `method` there.
fn ask_at(method: &str, src: &str) -> Value {
    let at = src.find('|').expect("a cursor");
    let text = src.replacen('|', "", 1);
    let line = src[..at].matches('\n').count() as u32;
    let character = (at - src[..at].rfind('\n').map_or(0, |i| i + 1)) as u32;
    let (mut server, _) = open(&text);
    let reply = server.handle(&request(
        20,
        method,
        obj([("textDocument", doc()), ("position", pos(line, character))]),
    ));
    reply[0].at(&["result"]).clone()
}

fn completions(src: &str) -> Vec<(String, u32, String)> {
    ask_at("textDocument/completion", src)
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|item| {
            (
                item.at(&["label"]).as_str().unwrap().to_string(),
                item.at(&["kind"]).as_u32().unwrap(),
                item.at(&["detail"]).as_str().unwrap_or("").to_string(),
            )
        })
        .collect()
}

fn labels(items: &[(String, u32, String)]) -> Vec<&str> {
    items.iter().map(|(label, ..)| label.as_str()).collect()
}

fn detail_of<'a>(items: &'a [(String, u32, String)], label: &str) -> &'a str {
    &items
        .iter()
        .find(|(l, ..)| l == label)
        .unwrap_or_else(|| panic!("no `{label}` in {items:?}"))
        .2
}

const ROAD: &str = "\
struct Road:
    to: Int
    w: Int

    /// Both ends' weight.
    fn twice(self) -> Int = 2 * w

    @get
    fn heavy(self) -> Bool = w > 10

    fn make(to: Int) -> Road = Road(to, 0)
";

fn signature(src: &str) -> (String, u32, Vec<String>) {
    let help = ask_at("textDocument/signatureHelp", src);
    assert!(!help.is_null(), "no signature help in {src:?}");
    let sig = &help.at(&["signatures"]).as_array().unwrap_or(&[])[0];
    let label = sig.at(&["label"]).as_str().unwrap().to_string();
    let params = sig
        .at(&["parameters"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|p| {
            let range = p.at(&["label"]).as_array().unwrap_or(&[]);
            let (s, e) = (
                range[0].as_u32().unwrap() as usize,
                range[1].as_u32().unwrap() as usize,
            );
            label[s..e].to_string()
        })
        .collect();
    (
        label,
        help.at(&["activeParameter"]).as_u32().unwrap(),
        params,
    )
}

/// The `(line, character)` of each location or edit in a reply.
fn starts(values: &[Value]) -> Vec<(u32, u32)> {
    values
        .iter()
        .map(|v| {
            let start = v.at(&["range", "start"]);
            (
                start.at(&["line"]).as_u32().unwrap(),
                start.at(&["character"]).as_u32().unwrap(),
            )
        })
        .collect()
}

fn references_at(src: &str) -> Vec<(u32, u32)> {
    let reply = ask_at_with(
        "textDocument/references",
        src,
        obj([("context", obj([("includeDeclaration", true.into())]))]),
    );
    starts(reply.as_array().unwrap_or(&[]))
}

/// Like `ask_at`, with more request params.
fn ask_at_with(method: &str, src: &str, extra: Value) -> Value {
    let at = src.find('|').expect("a cursor");
    let text = src.replacen('|', "", 1);
    let line = src[..at].matches('\n').count() as u32;
    let character = (at - src[..at].rfind('\n').map_or(0, |i| i + 1)) as u32;
    let (mut server, _) = open(&text);
    let mut params = std::collections::BTreeMap::from([
        ("textDocument".to_string(), doc()),
        ("position".to_string(), pos(line, character)),
    ]);
    if let Value::Object(more) = extra {
        params.extend(more);
    }
    let reply = server.handle(&request(21, method, Value::Object(params)));
    let result = reply[0].at(&["result"]).clone();
    if result.is_null() && !reply[0].at(&["error"]).is_null() {
        return reply[0].at(&["error"]).clone();
    }
    result
}

#[test]
fn document_highlight_marks_the_uses_and_the_declaration() {
    let src = "let n = 3\nfn twice(x: Int) -> Int = 2 * x\nprint(twice(n) + n)\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        30,
        "textDocument/documentHighlight",
        obj([("textDocument", doc()), ("position", pos(2, 18))]),
    ));
    let marks: Vec<(u32, u32, u32)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|h| {
            (
                h.at(&["range", "start", "line"]).as_u32().unwrap(),
                h.at(&["range", "start", "character"]).as_u32().unwrap(),
                h.at(&["kind"]).as_u32().unwrap(),
            )
        })
        .collect();
    assert_eq!(marks, [(0, 4, 3), (2, 12, 1), (2, 17, 1)]);
}

#[test]
fn folding_ranges_cover_blocks_comments_and_imports() {
    let src = "\
use std.fenwick.{Fenwick}
use std.dsu.{Dsu}
// Two lines
// of comment.
fn f(x: Int) -> Int:
    if x > 0:
        return x

    0
print(f(1))
";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        31,
        "textDocument/foldingRange",
        obj([("textDocument", doc())]),
    ));
    let ranges: Vec<(u32, u32, Option<&str>)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|r| {
            (
                r.at(&["startLine"]).as_u32().unwrap(),
                r.at(&["endLine"]).as_u32().unwrap(),
                r.at(&["kind"]).as_str(),
            )
        })
        .collect();
    assert_eq!(
        ranges,
        [
            (0, 1, Some("imports")),
            (2, 3, Some("comment")),
            (4, 8, None),
            (5, 6, None)
        ]
    );
}

#[test]
fn folding_ranges_cover_indented_lambda_blocks() {
    let src = "\
let lv = [0..=2, 0..3] (i, j) =>
    let miss = if j > 0: i + j else: i * 10
    miss + 1
var xs = [3, 1, 2]
xs.sort(by: x =>
    let k = -x
    k
)
";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        32,
        "textDocument/foldingRange",
        obj([("textDocument", doc())]),
    ));
    let ranges: Vec<(u32, u32, Option<&str>)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|r| {
            (
                r.at(&["startLine"]).as_u32().unwrap(),
                r.at(&["endLine"]).as_u32().unwrap(),
                r.at(&["kind"]).as_str(),
            )
        })
        .collect();
    assert_eq!(ranges, [(0, 2, None), (4, 6, None)]);
}

#[test]
fn semantic_tokens_classify_what_each_name_resolves_to() {
    let src = "let n: 1..=10 = read()\ndomain V = 0..n\nvar xs: [V]Int = 0\nfn bump(v: V):\n    xs[v] += 1\nbump(V.first)\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        32,
        "textDocument/semanticTokens/full",
        obj([("textDocument", doc())]),
    ));
    let data: Vec<u32> = reply[0]
        .at(&["result", "data"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|v| v.as_u32().unwrap())
        .collect();
    let (mut line, mut col) = (0, 0);
    let mut named = Vec::new();
    for t in data.chunks(5) {
        line += t[0];
        col = if t[0] == 0 { col + t[1] } else { t[1] };
        let text = src.lines().nth(line as usize).unwrap();
        named.push((
            text[col as usize..(col + t[2]) as usize].to_string(),
            t[3],
            t[4],
        ));
    }
    let has = |name: &str, kind: u32, modifiers: u32| {
        assert!(
            named.contains(&(name.to_string(), kind, modifiers)),
            "no ({name}, {kind}, {modifiers}) in {named:?}"
        );
    };
    has("n", 4, 3); // let n: a readonly declaration
    has("xs", 4, 1); // var xs: a declaration, not readonly
    has("V", 0, 0); // the domain, used as a type
    has("v", 5, 3); // the parameter
    has("bump", 1, 0); // the call
}

#[test]
fn workspace_symbols_find_names_in_open_files_and_the_stdlib_they_use() {
    let src = "use std.fenwick.{Fenwick}\nlet total = 3\nfn twice(x: Int) -> Int = 2 * x\nprint(twice(total))\n";
    let (mut server, _) = open(src);
    let found = |server: &mut Server, query: &str| -> Vec<(String, String)> {
        let reply = server.handle(&request(
            33,
            "workspace/symbol",
            obj([("query", query.into())]),
        ));
        reply[0]
            .at(&["result"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|s| {
                (
                    s.at(&["name"]).as_str().unwrap().to_string(),
                    s.at(&["location", "uri"]).as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    assert_eq!(
        found(&mut server, "twc"),
        [("twice".to_string(), URI.to_string())]
    );
    let fenwick = found(&mut server, "Fenw");
    assert!(
        fenwick.iter().any(|(name, uri)| name == "Fenwick"
            && uri.starts_with("file://")
            && uri.ends_with(".cp")
            && uri != URI),
        "{fenwick:?}"
    );
    assert!(found(&mut server, "").len() > fenwick.len());
}

#[test]
fn code_lenses_run_trace_test_and_stress_with_the_first_sample_when_asked() {
    let dir = std::env::temp_dir().join(format!("cpl-lens-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("sum two.tests")).unwrap();
    std::fs::write(dir.join("sum two.tests/2.in"), "1 2\n").unwrap();
    std::fs::write(dir.join("sum two.tests/10.in"), "3 4\n").unwrap();
    let file = dir.join("sum two.cp");
    let path = file.display().to_string();
    let uri = format!("file://{}", path.replace(' ', "%20"));
    let src = "let (a, b): (Int, Int) = read()\nprint(a + b)\n@alt(program)\nfn slow() = 0\n";
    let lenses = |options: Value| -> Vec<(String, Vec<String>)> {
        let mut server = Server::new();
        server.handle(&request(
            1,
            "initialize",
            obj([("initializationOptions", options)]),
        ));
        server.handle(&notification("initialized", obj([])));
        server.handle(&notification(
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", src.into()),
                ]),
            )]),
        ));
        let reply = server.handle(&request(
            2,
            "textDocument/codeLens",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        reply[0]
            .at(&["result"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|l| {
                assert_eq!(l.at(&["command", "command"]).as_str(), Some("cpl.exec"));
                (
                    l.at(&["command", "title"]).as_str().unwrap().to_string(),
                    l.at(&["command", "arguments"])
                        .as_array()
                        .unwrap_or(&[])
                        .iter()
                        .map(|a| a.as_str().unwrap().to_string())
                        .collect(),
                )
            })
            .collect()
    };
    assert!(
        lenses(Value::Null).is_empty(),
        "VS Code draws its own lenses"
    );
    let input = dir.join("sum two.tests/2.in").display().to_string();
    let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    assert_eq!(
        lenses(obj([("codeLens", true.into())])),
        [
            ("Run on 2".to_string(), s(&["run", &path, &input])),
            (
                "Trace on 2".to_string(),
                s(&["run", &path, &input, "--trace"])
            ),
            ("Test".to_string(), s(&["test", &path])),
            ("Stress".to_string(), s(&["stress", &path])),
        ]
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn inlay_hints_show_the_range_the_bounds_prove() {
    let src =
        "let n: 1..=100 = read()\nlet m = n * n + 1\nlet k = 7\nlet w = read()\nprint(m + k + w)\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        34,
        "textDocument/inlayHint",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(5, 0))])),
        ]),
    ));
    let labels: Vec<(u32, &str)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|h| h.at(&["kind"]).as_u32() == Some(1))
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["label"]).as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(labels, [(1, ": 2..=10001"), (2, ": 7..=7")]);
}

/// The labels completion ranks first, the ones that fit the expected type.
fn best_fits(src: &str) -> Vec<String> {
    let mut fits: Vec<String> = ask_at("textDocument/completion", src)
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|item| item.at(&["sortText"]).as_str().unwrap().starts_with('0'))
        .map(|item| item.at(&["label"]).as_str().unwrap().to_string())
        .collect();
    fits.sort();
    fits
}

/// The lints `cpl lint` reports are published with the checker's errors,
/// each with the fix `cpl fix` would apply as a quick fix, and the whole of
/// `cpl fix` is one fix-all action.
#[test]
fn lints_are_published_with_their_cpl_fix_codemods_as_quick_fixes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/problems/cf2062g/port.cp");
    let original = std::fs::read_to_string(&path).unwrap();
    let uri = format!("file://{}", path.display());
    let (good, bad) = ("let q: [I]I = read()", "var q: [I]I = read()");
    assert!(original.contains(good));
    let broken = original.replacen(good, bad, 1);
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    let doc = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", 1.into()),
        ("text", broken.as_str().into()),
    ]);
    let published = server.handle(&notification(
        "textDocument/didOpen",
        obj([("textDocument", doc)]),
    ));
    let diagnostics = published[0]
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[]);
    let never_changed = diagnostics
        .iter()
        .find(|d| d.at(&["code"]).as_str() == Some("L0201"))
        .unwrap_or_else(|| panic!("no L0201 in {diagnostics:?}"));
    assert_eq!(never_changed.at(&["severity"]).as_u32(), Some(2));
    let (title, fixed) = quick_fixed(&mut server, &uri, &broken, never_changed);
    assert!(title.contains("let"), "{title}");
    assert_eq!(fixed, original);

    let whole = obj([("start", pos(0, 0)), ("end", pos(0, 0))]);
    let reply = server.handle(&request(
        10,
        "textDocument/codeAction",
        obj([
            ("textDocument", obj([("uri", uri.as_str().into())])),
            ("range", whole),
            (
                "context",
                obj([
                    ("diagnostics", Value::Array(Vec::new())),
                    ("only", vec!["source.fixAll".into()].into()),
                ]),
            ),
        ]),
    ));
    let actions = reply[0].at(&["result"]).as_array().unwrap_or(&[]);
    let fix_all = actions
        .iter()
        .find(|a| a.at(&["kind"]).as_str() == Some("source.fixAll.cpl"))
        .expect("a fix-all action");
    let edit = &fix_all
        .at(&["edit", "changes", uri.as_str()])
        .as_array()
        .unwrap_or(&[])[0];
    let all = edit.at(&["newText"]).as_str().unwrap();
    assert!(all.contains(good) && !all.contains(bad));
    let left = cpl_lint::lint(all, &cpl_lint::Config::default());
    assert!(left.iter().all(|d| d.fixes.is_empty()), "{left:?}");
    assert!(left.len() < diagnostics.len());
}

fn open_port(server: &mut Server, port: &str, edit: (&str, &str)) -> (String, String, Vec<Value>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../corpus/problems/{port}/port.cp"));
    let original = std::fs::read_to_string(&path).unwrap();
    assert!(original.contains(edit.0), "{port}: {}", edit.0);
    let text = original.replacen(edit.0, edit.1, 1);
    let uri = format!("file://{}", path.display());
    server.handle(&request(1, "initialize", obj([])));
    let doc = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", 1.into()),
        ("text", text.as_str().into()),
    ]);
    let published = server.handle(&notification(
        "textDocument/didOpen",
        obj([("textDocument", doc)]),
    ));
    let diagnostics = published[0]
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[])
        .to_vec();
    (uri, text, diagnostics)
}

/// A getter is read without parentheses, so the editor offers it as a
/// property and hovers its value's type, for the built-ins and for a
/// user's `@get` alike.
#[test]
fn getters_complete_and_hover_as_properties() {
    let src = "\
struct Box:
    w: Int
    h: Int
    @get fn area(self) -> Int = self.w * self.h
    fn grow(self, by: Int) -> Box = Box(w: self.w + by, h: self.h + by)
let b = Box(w: 2, h: 3)
let xs = [1, 2, 3]
print(b.area, xs.len)
print(b.|)
";
    let items = completions(src);
    let kind = |name: &str| {
        items
            .iter()
            .find(|(label, ..)| label == name)
            .unwrap_or_else(|| panic!("no `{name}` in {items:?}"))
            .1
    };
    assert_eq!(kind("area"), 10, "a getter is a property");
    assert_eq!(kind("grow"), 2, "a method is a method");
    let items = completions("let xs = [1, 2, 3]\nprint(xs.|)\n");
    let len = items.iter().find(|(label, ..)| label == "len").unwrap();
    assert_eq!(len.1, 10, "{len:?}");
    let reply = ask_at("textDocument/completion", src);
    let area = reply
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .find(|item| item.at(&["label"]).as_str() == Some("area"))
        .unwrap()
        .clone();
    assert!(
        area.at(&["insertText"])
            .as_str()
            .is_none_or(|t| t == "area"),
        "{area:?}"
    );

    let text = src.replacen("print(b.|)\n", "", 1);
    let (mut server, diagnostics) = open(&text);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.at(&["severity"]).as_u32() != Some(1)),
        "{diagnostics:?}"
    );
    let shown = |server: &mut Server, character| {
        hover_at(server, 7, character)
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    let area = shown(&mut server, 9);
    assert!(area.contains("area") && area.contains("Int"), "{area}");
    let len = shown(&mut server, 18);
    assert!(len.contains("len") && len.contains("Int"), "{len}");
}

#[test]
fn hover_shows_the_documented_cost_and_the_proven_range() {
    let src = "\
use std.algebra.{fold, Monoid}
let n: 1..=100 = read()
let m = n * n + 1
let xs = [3, 1, 2]
print(xs.sorted(), xs.rev, fold(Monoid.sum, xs), m)
";
    let (mut server, _) = open(src);
    let shown = |server: &mut Server, line, character| {
        hover_at(server, line, character)
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    let sorted = shown(&mut server, 4, 10);
    assert!(
        sorted.starts_with("```cp\nsorted(by: T -> K = x => x, cmp: (T, T) -> Bool = (a, b) => a < b, desc: Bool = false) -> List<T>\ncost: O(n log n)\n```"),
        "{sorted}"
    );
    let rev = shown(&mut server, 4, 23);
    assert!(
        rev.starts_with("```cp\nrev: Rev<List<Int>>\n  storage: none of its own; a view of its source\ncost: O(1)\n```"),
        "{rev}"
    );
    let fold = shown(&mut server, 4, 28);
    assert!(fold.contains("-> T\ncost: O(n)\n```"), "{fold}");
    assert_eq!(
        shown(&mut server, 2, 4),
        "```cp\nm: 2..=10001\n  storage: 2 B\n```"
    );
}

/// A range the bounds prove reaches into fns and loops, and a local that
/// shadows a bounded name hides its bound.
#[test]
fn inlay_hints_and_hover_show_ranges_inside_fns_and_loops() {
    let src = "let n: 1..=100 = read()\nfn f(k: 1..=n) -> Int:\n    let t = k * 2\n    let n = k ^ 3\n    let u = n + 1\n    for i in 0..k:\n        let j = i + t\n        print(j)\n    t + u\nfor i in 1..=n:\n    let sq = i * i\n    print(sq, f(i))\n";
    let (mut server, diagnostics) = open(src);
    assert!(
        !format!("{diagnostics:?}").contains("\"E0"),
        "{diagnostics:?}"
    );
    let reply = server.handle(&request(
        35,
        "textDocument/inlayHint",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(12, 0))])),
        ]),
    ));
    let ranged: Vec<(u32, &str)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["label"]).as_str().unwrap(),
            )
        })
        .filter(|(_, label)| label.contains("..="))
        .collect();
    assert_eq!(
        ranged,
        [
            (2, ": 2..=200"),
            (6, ": 2..=299"),
            (9, ": 1..=n"),
            (10, ": 1..=10000")
        ]
    );
    let hover = hover_at(&mut server, 6, 12);
    assert!(
        hover
            .at(&["contents", "value"])
            .as_str()
            .unwrap()
            .contains("j: 2..=299"),
        "{hover:?}"
    );
    let hover = hover_at(&mut server, 5, 8);
    assert!(
        !hover
            .at(&["contents", "value"])
            .as_str()
            .unwrap()
            .contains(" in "),
        "a loop variable already shows its range as its type: {hover:?}"
    );
}

/// `reach(while:, cyclic:)`: a misspelled option is one click from the port
/// that uses it, and hover, completion and signature help know it.
#[test]
fn reach_completes_hovers_helps_and_fixes_its_options() {
    assert_fixes_restore_src(
        "let xs = [1, 2, 3]\nfor (a, b) in xs.reach(while: (x, y) => x < y, cyclic: true):\n    print(a, b)\n",
        &[(
            "cyclic: true)",
            "cylic: true)",
            "E0310",
            "change to `cyclic`",
        )],
    );
    let reach = ask_at(
        "textDocument/completion",
        "let xs = [1, 2]\nprint(xs.rea|)\n",
    )
    .at(&["items"])
    .as_array()
    .unwrap_or(&[])
    .iter()
    .find(|i| i.at(&["label"]).as_str() == Some("reach"))
    .cloned()
    .unwrap();
    assert_eq!(reach.at(&["kind"]).as_u32(), Some(2));
    assert_eq!(
        reach.at(&["detail"]).as_str(),
        Some("reach(while: (Int, Int) -> Bool, cyclic: Bool = false) -> List<Int>")
    );
    let (mut server, _) =
        open("let xs = [1, 2, 4]\nprint(xs.reach(while: (a, b) => b - a <= 3))\n");
    let hover = hover_at(&mut server, 1, 11);
    let hover = hover.at(&["contents", "value"]).as_str().unwrap();
    assert!(hover.contains("\ncost: O(n)\n"), "{hover}");
    let help = ask_at(
        "textDocument/signatureHelp",
        "let xs = [1, 2]\nprint(xs.reach(while: (a, b) => a < b, |))\n",
    );
    assert_eq!(help.at(&["activeParameter"]).as_u32(), Some(1), "{help:?}");
}

#[test]
fn range_completion_and_hover_show_start_and_end() {
    let items = completions("let r = 2..=4\nprint(r.|)\n");
    let names = labels(&items);
    assert!(names.contains(&"start"), "{names:?}");
    assert!(names.contains(&"end"), "{names:?}");
    assert!(!names.contains(&"lo"), "{names:?}");
    assert!(!names.contains(&"hi"), "{names:?}");

    let (mut server, diags) = open("let r = 2..=4\nprint(r.start, r.end)\n");
    assert!(diags.is_empty(), "{diags:?}");
    let start_hover = hover_at(&mut server, 1, 9);
    assert!(
        start_hover
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .contains("Int"),
        "{start_hover:?}"
    );
    let end_hover = hover_at(&mut server, 1, 18);
    assert!(
        end_hover
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .contains("Int"),
        "{end_hover:?}"
    );
}

#[test]
fn sum_domains_and_block_lambdas_fold_complete_and_hover() {
    let src = "\
domain Row = 0..2
domain Col = 0..3

enum Node:
    Src
    Snk
    Cell(Row, Col)

let vals: [Node]Int = 0
let xs = [3, 1, 2]
let ys = xs.map(x =>
    let d = x % 10
    d + 1
)
print(vals[Node.Src], ys, xs.map(y => y + 1))
";
    let (mut server, diagnostics) = open(src);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let reply = server.handle(&request(
        31,
        "textDocument/foldingRange",
        obj([("textDocument", doc())]),
    ));
    let folds: Vec<(u32, u32)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|r| {
            (
                r.at(&["startLine"]).as_u32().unwrap(),
                r.at(&["endLine"]).as_u32().unwrap(),
            )
        })
        .collect();
    assert!(folds.contains(&(3, 6)), "{folds:?}");
    assert!(folds.contains(&(10, 12)), "{folds:?}");
    let hover_text = |server: &mut Server, line, character| {
        hover_at(server, line, character)
            .at(&["contents", "value"])
            .as_str()
            .unwrap_or("")
            .to_string()
    };
    assert!(hover_text(&mut server, 10, 16).contains("x: Int"));
    assert!(hover_text(&mut server, 11, 8).contains("d: 0..=9"));
    assert!(hover_text(&mut server, 14, 34).contains("y: Int"));
    assert!(hover_text(&mut server, 14, 17).contains("Src: Node"));
    assert!(hover_text(&mut server, 4, 5).contains("Node.Src"));
    assert!(hover_text(&mut server, 6, 5).contains("Node.Cell(Row, Col)"));
    let reply = server.handle(&request(
        32,
        "textDocument/inlayHint",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(20, 0))])),
        ]),
    ));
    let hints: Vec<(u32, u32, &str)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["position", "character"]).as_u32().unwrap(),
                h.at(&["label"]).as_str().unwrap(),
            )
        })
        .collect();
    for hint in [(10, 17, ": Int"), (11, 9, ": 0..=9"), (14, 34, ": Int")] {
        assert!(hints.contains(&hint), "{hint:?} in {hints:?}");
    }
    let items = ask_at(
        "textDocument/completion",
        &src.replacen("vals[Node.Src]", "vals[Node.|]", 1),
    );
    let variants: Vec<&str> = items
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|i| i.at(&["label"]).as_str())
        .collect();
    assert_eq!(variants, ["Src", "Snk", "Cell"]);
    assert_fixes_restore_src(
        "let e = \"a\"\nprint(e + \":b\")\n",
        &[(
            "e + \":b\"",
            "e ++ \":b\"",
            "E0004",
            "replace `++` with `+`",
        )],
    );
}

#[test]
fn hover_on_punned_named_arg_shows_local_type() {
    let src = "\
fn f(moves: Int, cost: Int) -> Int = moves + cost
let moves = 3
let cost = 4
print(f(moves:, cost:))
";
    let (mut server, diagnostics) = open(src);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let hover = hover_at(&mut server, 3, 10);
    let text = hover.at(&["contents", "value"]).as_str().unwrap_or("");
    assert!(
        text.contains("moves: 3..=3"),
        "expected 3..=3 hover on punned arg, got: {hover:?}"
    );
}

#[test]
fn folding_ranges_cover_loop_else() {
    let src = "\
for x in 0..3:
    if x == 2:
        break
else:
    print(-1)
";
    let (mut server, diagnostics) = open(src);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let reply = server.handle(&request(
        33,
        "textDocument/foldingRange",
        obj([("textDocument", doc())]),
    ));
    let ranges: Vec<(u32, u32)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|r| {
            (
                r.at(&["startLine"]).as_u32().unwrap(),
                r.at(&["endLine"]).as_u32().unwrap(),
            )
        })
        .collect();
    assert!(
        ranges.contains(&(0, 2)),
        "expected loop fold (0, 2), got {ranges:?}"
    );
    assert!(
        ranges.contains(&(3, 4)),
        "expected else fold (3, 4), got {ranges:?}"
    );
}

#[test]
fn loop_counts_read_as_a_number_and_its_size() {
    let shown: Vec<String> = [7, 9999, 10000, 99999, 200000, 249999500000]
        .into_iter()
        .map(crate::server::count_text)
        .collect();
    assert_eq!(
        shown,
        [
            "7",
            "9999",
            "10000 (1e4)",
            "99999 (1e5)",
            "200000 (2e5)",
            "249999500000 (2.5e11)"
        ]
    );
}

/// A rename that would make a use resolve to a different binding (two
/// locals merged, a local capturing a global, a global shadowed by a local)
/// is refused; one that changes no resolution goes through.
#[test]
fn rename_refuses_a_name_that_would_capture_or_be_captured() {
    let rename = |src: &str, to: &str| {
        ask_at_with("textDocument/rename", src, obj([("newName", to.into())]))
    };
    let refused = [
        (
            "fn f(n: Int) -> Int:\n    let a| = 1\n    let b = 2\n    a + b\nprint(f(1))\n",
            "b",
        ),
        (
            "fn f(n: Int) -> Int:\n    let a| = 1\n    a + n\nprint(f(1))\n",
            "n",
        ),
        (
            "let x| = 1\nfn f() -> Int:\n    let y = 2\n    x + y\nprint(f())\n",
            "y",
        ),
        (
            "let g = 5\nfn f() -> Int:\n    let a| = 1\n    a + g\nprint(f())\n",
            "g",
        ),
        ("let g = 5\nfor i| in 0..3:\n    print(i + g)\n", "g"),
        ("let x| = 1\nlet f = (y) => x + y\nprint(f(2))\n", "y"),
    ];
    for (src, to) in refused {
        let reply = rename(src, to);
        let message = reply
            .at(&["message"])
            .as_str()
            .unwrap_or_else(|| panic!("{src}: {reply:?}"));
        assert!(message.contains(&format!("`{to}`")), "{src}: {message}");
    }
    let allowed = [
        ("fn f() -> Int:\n    let a| = 1\n    a\nfn g() -> Int:\n    let b = 2\n    b\nprint(f() + g())\n", "b"),
        ("fn f(n: Int) -> Int:\n    let a| = 1\n    a + n\nprint(f(1))\n", "c"),
        ("let x = 1\nfn f() -> Int:\n    let a| = 2\n    a\nprint(f() + x)\n", "x"),
    ];
    for (src, to) in allowed {
        let reply = rename(src, to);
        assert!(
            matches!(reply.at(&["changes"]), Value::Object(_)),
            "{src}: {reply:?}"
        );
    }
}

/// What the bounds prove is kept between requests, and an edit replaces it.
#[test]
fn hover_shows_the_range_the_bounds_prove_after_an_edit() {
    let (mut server, _) = open("let n: 1..=100 = read()\nlet t = n * 2\nprint(t)\n");
    let range = |server: &mut Server| {
        hover_at(server, 1, 4)
            .at(&["contents", "value"])
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(
        range(&mut server).contains("t: 2..=200"),
        "{}",
        range(&mut server)
    );
    assert!(range(&mut server).contains("t: 2..=200"));
    server.handle(&did_change((1, 12), (1, 13), "3"));
    assert!(
        range(&mut server).contains("t: 3..=300"),
        "{}",
        range(&mut server)
    );
}

#[test]
fn a_code_lens_shows_the_memory_the_arrays_take_against_the_judge_limit() {
    let dir = std::env::temp_dir().join(format!("cpl-memory-lens-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("sum two.tests")).unwrap();
    std::fs::write(dir.join("sum two.tests/2.in"), "1 2\n").unwrap();
    std::fs::write(dir.join("sum two.tests/10.in"), "3 4\n").unwrap();
    let file = dir.join("sum two.cp");
    let path = file.display().to_string();
    let uri = format!("file://{}", path.replace(' ', "%20"));
    let src = "// judge: codeforces\nlet n: 1..=200000 = read()\ndomain V = 1..=n\n\
        var dist: [V.cuts]Int = 0\nvar grid: [V, 0..1000]Int = 0\nvar seen: List<V> = []\n\
        var order: List<V> = []\nfor v in V: seen.push(v)\nprint(dist)\n";
    let lenses = |options: Value| -> Vec<(String, Vec<String>)> {
        let mut server = Server::new();
        server.handle(&request(
            1,
            "initialize",
            obj([("initializationOptions", options)]),
        ));
        server.handle(&notification("initialized", obj([])));
        server.handle(&notification(
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", src.into()),
                ]),
            )]),
        ));
        let reply = server.handle(&request(
            2,
            "textDocument/codeLens",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        reply[0]
            .at(&["result"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|l| {
                assert_eq!(l.at(&["command", "command"]).as_str(), Some("cpl.exec"));
                (
                    l.at(&["command", "title"]).as_str().unwrap().to_string(),
                    l.at(&["command", "arguments"])
                        .as_array()
                        .unwrap_or(&[])
                        .iter()
                        .map(|a| a.as_str().unwrap().to_string())
                        .collect(),
                )
            })
            .collect()
    };
    let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    let memory: Vec<_> = lenses(obj([("codeLens", true.into())]))
        .into_iter()
        .filter(|(title, _)| title.starts_with("Memory"))
        .collect();
    let title = "Memory: bounds allow 1.6 GB, over the 256 MB limit on codeforces, \
                 not counting order";
    assert_eq!(memory, [(title.to_string(), s(&["judges", &path]))]);
    assert_eq!(lenses(Value::Null), memory, "VS Code draws only the others");
    std::fs::remove_dir_all(dir).unwrap();
}
