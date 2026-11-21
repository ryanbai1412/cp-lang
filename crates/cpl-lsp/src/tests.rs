use crate::json::{self, obj, Value};
use crate::{serve, write_message, Server};

const URI: &str = "file:///work/roads.cp";

const SOURCE: &str = "\
let n: 1..=2e5 = read()
domain V = 1..=n
domain E = 0..n
var dist: [V]?Int = none
var total = 0
for v in V:
    total += dist[v] ?? 0
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
        messages[0].get("method").as_str(),
        Some("textDocument/publishDiagnostics")
    );
    assert_eq!(messages[0].at(&["params", "uri"]).as_str(), Some(URI));
    messages[0]
        .at(&["params", "diagnostics"])
        .as_array()
        .to_vec()
}

fn hover_at(server: &mut Server, line: u32, character: u32) -> Value {
    let reply = server.handle(&request(
        7,
        "textDocument/hover",
        obj([("textDocument", doc()), ("position", pos(line, character))]),
    ));
    assert_eq!(reply.len(), 1);
    assert_eq!(reply[0].get("id").as_u32(), Some(7));
    reply[0].get("result").clone()
}

#[test]
fn initialize_announces_what_the_server_can_do() {
    let mut server = Server::new();
    let reply = server.handle(&request(1, "initialize", obj([])));
    let caps = reply[0].at(&["result", "capabilities"]);
    assert_eq!(caps.at(&["textDocumentSync", "change"]).as_u32(), Some(2));
    assert_eq!(caps.get("hoverProvider"), &Value::Bool(true));
    assert_eq!(caps.get("definitionProvider"), &Value::Bool(true));
    assert_eq!(caps.get("documentSymbolProvider"), &Value::Bool(true));
    assert_eq!(
        reply[0].at(&["result", "serverInfo", "name"]).as_str(),
        Some("cpl")
    );
}

#[test]
fn requests_before_initialize_are_refused() {
    let mut server = Server::new();
    let reply = server.handle(&request(1, "textDocument/hover", obj([])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32002.0));
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
    assert_eq!(d.get("code").as_str(), Some("E0201"));
    assert_eq!(d.get("source").as_str(), Some("cpl"));
    assert_eq!(d.get("severity").as_u32(), Some(1));
    assert_eq!(
        d.get("message").as_str(),
        Some("no `w` in scope\nhelp: declare it first: `let w = ...`")
    );
    assert_eq!(d.at(&["range", "start"]), &pos(6, 18));
    assert_eq!(d.at(&["range", "end"]), &pos(6, 19));
    let d = &diagnostics[1];
    assert_eq!(d.get("code").as_str(), Some("E0401"));
    assert_eq!(d.at(&["range", "start"]), &pos(7, 0));

    let (_, diagnostics) = open("type R = Int\ntype R = Bool\n");
    let related = diagnostics[0].get("relatedInformation").as_array();
    assert_eq!(
        related[0].get("message").as_str(),
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
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E0101"));
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(8, 5));
    // Finish the line: clean again.
    let published = server.handle(&did_change((8, 11), (8, 11), "V.first])\n"));
    assert!(diagnostics_of(&published).is_empty());
    // Break a line in the middle of the file by deleting `in V`.
    let published = server.handle(&did_change((5, 5), (5, 10), ""));
    let diagnostics = diagnostics_of(&published);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E0103"));
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
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E0302"));
}

#[test]
fn edits_use_utf16_positions() {
    // Where `+` can't combine its operands. The literal is also reported
    // for not being ASCII; that is not what this test is about.
    fn bad_operands(diagnostics: &[Value]) -> &Value {
        diagnostics
            .iter()
            .find(|d| d.get("code").as_str() == Some("E0309"))
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
        Some("```cp\ndist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max\n```")
    );
    assert_eq!(hover.at(&["range", "start"]), &pos(6, 13));
    assert_eq!(hover.at(&["range", "end"]), &pos(6, 17));
    assert!(hover_at(&mut server, 6, 10).is_null());
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
        .get("result")
        .as_array()
        .iter()
        .filter_map(|s| s.get("name").as_str())
        .collect();
    assert_eq!(names, vec!["n", "V", "E", "dist", "total"]);
    assert_eq!(
        reply[0].get("result").as_array()[1].get("kind").as_u32(),
        Some(26)
    );
}

#[test]
fn quick_fixes_come_from_the_diagnostics() {
    let (mut server, diagnostics) = open(&SOURCE.replace("print(total)", "print(totl)"));
    assert_eq!(diagnostics.len(), 1);
    let reply = server.handle(&request(
        5,
        "textDocument/codeAction",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(7, 6)), ("end", pos(7, 10))])),
            ("context", obj([("diagnostics", diagnostics.into())])),
        ]),
    ));
    let actions = reply[0].get("result").as_array();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].get("title").as_str(), Some("change to `total`"));
    let edits = actions[0].at(&["edit", "changes"]).get(URI).as_array();
    assert_eq!(edits[0].get("newText").as_str(), Some("total"));
    assert_eq!(edits[0].at(&["range", "start"]), &pos(7, 6));
    assert_eq!(edits[0].at(&["range", "end"]), &pos(7, 10));
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
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
}

#[test]
fn unknown_and_malformed_messages_get_errors_not_crashes() {
    let (mut server, _) = open(SOURCE);
    let reply = server.handle(&request(2, "textDocument/semanticTokens/full", obj([])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32601.0));
    assert!(server
        .handle(&notification("workspace/didChangeWatchedFiles", obj([])))
        .is_empty());
    let reply = server.handle(&request(
        3,
        "textDocument/hover",
        obj([("textDocument", doc())]),
    ));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
    let reply = server.handle(&request(4, "textDocument/hover", Value::Null));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
    // Positions far outside the document clamp.
    assert!(hover_at(&mut server, 9999, 9999).is_null());
    // A change for a document that is not open.
    let mut other = did_change((0, 0), (0, 0), "x");
    if let Value::Object(members) = &mut other {
        members[2].1 = obj([
            ("textDocument", obj([("uri", "file:///nope.cp".into())])),
            ("contentChanges", Value::Array(Vec::new())),
        ]);
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
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32603.0));
    assert!(reply[0]
        .at(&["error", "message"])
        .as_str()
        .unwrap()
        .contains("deliberate test panic"));
    let diagnostics = diagnostics_of(&reply[1..]);
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E9999"));
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
    assert_eq!(reply[0].get("result"), &Value::Null);
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
    assert_eq!(replies.len(), 4);
    assert_eq!(replies[0].get("id").as_u32(), Some(1));
    let diagnostics = replies[1].at(&["params", "diagnostics"]).as_array();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].get("message").as_str(),
        Some("`dist` is indexed by V, but `n` is a 1..=2e5\nhelp: convert it with `V(n)`, or write `dist.raw[n]` to index by number")
    );
    assert_eq!(
        replies[2].at(&["result", "contents", "value"]).as_str(),
        Some("```cp\ndomain V = 1..=n\n  at most 2e5 positions (n <= 2e5)\n  storage: 4 bytes per position\n```")
    );
    assert_eq!(replies[3].get("id").as_u32(), Some(3));
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
    assert_eq!(replies[0].at(&["error", "code"]), &Value::Number(-32700.0));
    assert_eq!(replies[1].get("id").as_u32(), Some(1));

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
    let src =
        "use std.mod.{Mod, Comb}\nlet c = Comb<1000000007>.new(10)\nprint(c.choose(5, 2).v)\n";

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
                ("version", Value::Number(1.0)),
                ("text", src.into()),
            ]),
        )]),
    ));
    assert_eq!(published.len(), 1);
    let diags = published[0].at(&["params", "diagnostics"]).as_array();
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
        hover_text.contains("Comb"),
        "expected Comb hover, got: {hover_text}"
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
        def_uri.ends_with("/stdlib/mod.cp"),
        "expected definition in stdlib/mod.cp, got: {def_uri}"
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
        "use std.mod.{Mod, Comb}\nlet c = Comb<1000000007>.new(10)\nprint(c.choose(5, 2).v)\n";
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    server.handle(&notification("initialized", obj([])));
    let text_document = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", Value::Number(1.0)),
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
    assert!(text.starts_with("```cp\nchoose: fn(n: Int, k: Int) -> Mod<"));
    assert_eq!(
        text.split_once("```\n\n").unwrap().1,
        "The number of `k`-subsets of `n` things; 0 when `k` is outside `0..=n`.\n\
         Uses Lucas's theorem automatically when `n >= P` (for prime `P`)."
    );
    let def = server.handle(&request(3, "textDocument/definition", at(2, 9)));
    assert!(def[0]
        .at(&["result", "uri"])
        .as_str()
        .unwrap()
        .ends_with("/stdlib/mod.cp"));
    assert_eq!(def[0].at(&["result", "range", "start"]), &pos(120, 11));
    assert_eq!(def[0].at(&["result", "range", "end"]), &pos(120, 17));
}

const V22: &str = "\
let (n, x): (1..=100, 1..=1e6) = read()
let coins: [0..n]Int = read()
var dp: [0..=x]?Int = none
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
            (line, d.get("message").as_str().unwrap().to_string())
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
        Some("```cp\nbest: ?Int\n  storage: 8 bytes\n```")
    );
    assert_eq!(hover.at(&["range", "start"]), &pos(7, 4));
    assert_eq!(hover.at(&["range", "end"]), &pos(7, 8));
    let hover = hover_at(&mut server, 2, 4);
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\ndp: [0..=x]?Int\n  storage: 8 bytes per element\n```")
    );
}

#[test]
fn v22_removed_forms_are_reported_with_what_to_write() {
    let (_, diagnostics) = open(
        &V22.replace("dp[s] min= dp[s - c] + 1", "dp[s] += 1")
            .replace(
                "print(best ?? -1)",
                "print(best ?? 0 + 1)\ndefer print(1)\nprint(sum(dp))",
            ),
    );
    for d in &diagnostics {
        assert_eq!(d.get("severity").as_u32(), Some(1));
        assert_eq!(d.get("source").as_str(), Some("cpl"));
    }
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|d| d.get("code").as_str().unwrap())
        .collect();
    assert_eq!(codes, ["E0309", "E0110", "E0503", "E0309"]);
    let found = lines_and_messages(&diagnostics);
    let expected = [
        (6, "`+=` can't be applied to an ?Int\nhelp: write `x = x + y` when `none` should stay `none`"),
        (8, "`??` mixed with `+ - * / %` needs parentheses\nhelp: write `(a ?? 0) + b`"),
        (9, "`defer` was removed in CP Lang 0.2.2\nhelp: undo by hand after the call"),
        (10, "`sum` adds numbers, not an ?Int\nhelp: `sum` does not skip or absorb `none`"),
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
fn formatting_replaces_the_file_with_what_cpl_fmt_writes() {
    let (mut server, _) = open("let  x=1\nprint( x )\n");
    let reply = server.handle(&request(
        9,
        "textDocument/formatting",
        obj([("textDocument", doc())]),
    ));
    let edits = reply[0].get("result").as_array().to_vec();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].get("newText").as_str(), Some("let x = 1\nprint(x)\n"));
    assert_eq!(edits[0].at(&["range", "start", "line"]).as_u32(), Some(0));
    assert_eq!(edits[0].at(&["range", "end", "line"]).as_u32(), Some(2));

    // A file that is already formatted has nothing to change.
    let (mut server, _) = open("let x = 1\nprint(x)\n");
    let reply = server.handle(&request(
        9,
        "textDocument/formatting",
        obj([("textDocument", doc())]),
    ));
    assert!(reply[0].get("result").as_array().is_empty());
}

#[test]
fn inlay_hints_show_the_types_that_are_not_written() {
    let (mut server, _) = open("let n = 3\nlet m: Int = 4\nvar xs = [n, m]\nfor x in xs:\n    print(x)\n");
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let hints = reply[0].get("result").as_array().to_vec();
    let shown: Vec<(u32, u32, &str)> = hints
        .iter()
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.get("label").as_str().unwrap(),
            )
        })
        .collect();
    // `m` has its type written, so it gets none.
    assert_eq!(shown, [(0, 5, ": Int"), (2, 6, ": List<Int>"), (3, 5, ": Int")]);
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
    reply[0].get("result").clone()
}

fn completions(src: &str) -> Vec<(String, u32, String)> {
    ask_at("textDocument/completion", src)
        .get("items")
        .as_array()
        .iter()
        .map(|item| {
            (
                item.get("label").as_str().unwrap().to_string(),
                item.get("kind").as_u32().unwrap(),
                item.get("detail").as_str().unwrap_or("").to_string(),
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

#[test]
fn completion_after_a_trailing_dot_lists_fields_getters_and_methods() {
    let items = completions(&format!("{ROAD}let r = Road(1, 2)\nprint(r.|)\n"));
    assert_eq!(labels(&items), ["to", "w", "twice", "heavy"]);
    assert_eq!(detail_of(&items, "w"), "w: Int");
    assert_eq!(detail_of(&items, "twice"), "fn twice(self) -> Int");
    assert_eq!(items.iter().find(|i| i.0 == "heavy").unwrap().1, 10);
    // A fn without `self` belongs to the type, not to a value.
    assert!(!labels(&items).contains(&"make"), "{items:?}");
}

#[test]
fn dot_completion_works_on_a_line_that_does_not_parse_yet() {
    // The item ends mid-expression: the checker still types `r`.
    let items = completions(&format!("{ROAD}let r = Road(1, 2)\nlet x = r.|\n"));
    assert!(labels(&items).contains(&"twice"), "{items:?}");
    // And partway through a member's name, after a call.
    let items = completions(&format!(
        "{ROAD}fn f() -> Road = Road(1, 2)\nprint(f().tw|)\n"
    ));
    assert!(labels(&items).contains(&"twice"), "{items:?}");
}

#[test]
fn dot_completion_on_a_type_lists_what_it_has_without_a_value() {
    let items = completions(&format!(
        "{ROAD}enum Dir:\n    L\n    R\nprint(Dir.|)\nprint(Road.make(1))\n"
    ));
    assert_eq!(labels(&items), ["L", "R"]);
    let items = completions(&format!("{ROAD}let r = Road.|\n"));
    assert_eq!(labels(&items), ["make"]);
}

#[test]
fn dot_completion_on_a_library_type_substitutes_its_type_arguments() {
    let src = "domain V = 0..5\nvar t = SegTree<V, Int>.of(max, fn(v) -> 0)\nt.|\n";
    let items = completions(src);
    for method in ["get", "set", "fold", "update"] {
        assert!(
            labels(&items).contains(&method),
            "{method} missing: {items:?}"
        );
    }
    assert_eq!(detail_of(&items, "set"), "fn set(mut self, i: V, x: Int)");
}

#[test]
fn dot_completion_on_records_domains_and_positions() {
    let items = completions("let p = (x: 1, y: 2)\nprint(p.|)\n");
    assert_eq!(labels(&items), ["x", "y"]);
    let items = completions("domain V = 0..5\nprint(V.|)\n");
    assert!(labels(&items).contains(&"first") && labels(&items).contains(&"rev"));
    let items = completions("domain V = 0..5\nfor v in V:\n    print(v.|)\n");
    assert!(labels(&items).contains(&"ord") && labels(&items).contains(&"r"));
}

/// The members offered on every domain and position are ones the checker types.
#[test]
fn builtin_domain_and_position_members_are_the_checkers() {
    use crate::complete::{DOMAIN_MEMBERS, POS_MEMBERS};
    for (members, receiver) in [(DOMAIN_MEMBERS, "V"), (POS_MEMBERS, "v")] {
        for (name, _) in members {
            let src = format!("domain V = 0..5\nfor v in V:\n    let x = {receiver}.{name}\n");
            let (mut server, replies) = open(&src);
            // The back ends may not lower every one yet (E0501); the checker types them.
            let errors: Vec<&Value> = replies
                .iter()
                .filter(|d| d.get("code").as_str() != Some("E0501"))
                .collect();
            assert!(errors.is_empty(), "{receiver}.{name}: {errors:?}");
            let hover = hover_at(&mut server, 2, 8);
            let text = hover.at(&["contents", "value"]).as_str().unwrap_or("");
            assert!(
                text.contains("x: ") && !text.contains('?') && !text.contains('_'),
                "{receiver}.{name} is untyped: {text}"
            );
        }
    }
}

#[test]
fn completion_offers_locals_declarations_and_the_prelude_with_signatures() {
    let src = format!("{ROAD}/// Adds one.\nfn inc(x: Int) -> Int = x + 1\nfn main(n: Int):\n    let total = n * 2\n    pr|\n");
    let items = completions(&src);
    let names = labels(&items);
    // Locals first, then this file, then the prelude, then keywords.
    for name in [
        "total", "n", "inc", "Road", "SegTree", "Dsu", "zip", "while",
    ] {
        assert!(names.contains(&name), "{name} missing: {names:?}");
    }
    assert!(names.iter().position(|n| *n == "total") < names.iter().position(|n| *n == "inc"));
    assert_eq!(detail_of(&items, "total"), "total: Int");
    assert_eq!(detail_of(&items, "inc"), "fn inc(x: Int) -> Int");
    let reply = ask_at("textDocument/completion", &src);
    let inc = reply
        .get("items")
        .as_array()
        .iter()
        .find(|i| i.get("label").as_str() == Some("inc"))
        .unwrap();
    assert_eq!(
        inc.at(&["documentation", "value"]).as_str(),
        Some("Adds one.")
    );
    let zip = detail_of(&items, "zip");
    assert!(zip.starts_with("fn zip"), "{zip}");
    // A local of another fn is not in scope.
    let items = completions("fn a():\n    let hidden = 1\nfn b():\n    h|\n");
    assert!(!labels(&items).contains(&"hidden"), "{items:?}");
}

#[test]
fn use_std_completes_module_names_and_then_their_exports() {
    let items = completions("use std.|\n");
    for module in ["segtree", "seq", "graph", "math"] {
        assert!(labels(&items).contains(&module), "{module}: {items:?}");
    }
    assert!(!labels(&items).contains(&"prelude"));
    assert!(items.iter().all(|(_, kind, _)| *kind == 9));
    let items = completions("use std.segtree.{Seg|}\n");
    assert!(labels(&items).contains(&"SegTree"), "{items:?}");
    assert!(completions("// see xs.|\n").is_empty());
}

fn signature(src: &str) -> (String, u32, Vec<String>) {
    let help = ask_at("textDocument/signatureHelp", src);
    let sig = &help.get("signatures").as_array()[0];
    let label = sig.get("label").as_str().unwrap().to_string();
    let params = sig
        .get("parameters")
        .as_array()
        .iter()
        .map(|p| {
            let range = p.get("label").as_array();
            let (s, e) = (
                range[0].as_u32().unwrap() as usize,
                range[1].as_u32().unwrap() as usize,
            );
            label[s..e].to_string()
        })
        .collect();
    (label, help.get("activeParameter").as_u32().unwrap(), params)
}

#[test]
fn signature_help_tracks_the_argument_being_typed() {
    let decl = "/// Moves along.\nfn walk(from: Int, steps: Int, back: Bool) -> Int = from\n";
    let (label, active, params) = signature(&format!("{decl}print(walk(|))\n"));
    assert_eq!(label, "walk(from: Int, steps: Int, back: Bool) -> Int");
    assert_eq!(params, ["from: Int", "steps: Int", "back: Bool"]);
    assert_eq!(active, 0);
    let (_, active, _) = signature(&format!("{decl}print(walk(1, [2, 3].len, |))\n"));
    assert_eq!(active, 2);
    // Inside a nested call it is that call's signature.
    let (label, ..) = signature(&format!("{decl}print(walk(1, walk(|"));
    assert!(label.starts_with("walk("));
    let help = ask_at(
        "textDocument/signatureHelp",
        &format!("{decl}print(walk(1, |"),
    );
    assert_eq!(
        help.at(&["signatures"]).as_array()[0]
            .at(&["documentation", "value"])
            .as_str(),
        Some("Moves along.")
    );
    assert!(ask_at("textDocument/signatureHelp", "let x = 1|\n").is_null());
}

#[test]
fn signature_help_for_methods_and_library_fns() {
    let (label, active, _) = signature(&format!("{ROAD}let r = Road(1, 2)\nprint(Road.make(|))\n"));
    assert_eq!((label.as_str(), active), ("make(to: Int) -> Road", 0));
    let src = "domain V = 0..5\nvar t = SegTree<V, Int>.of(max, fn(v) -> 0)\nt.set(V.first, |)\n";
    let (label, active, params) = signature(src);
    assert!(label.starts_with("set("), "{label}");
    assert_eq!(params.len(), 2, "{label}");
    assert_eq!(active, 1);
    let (label, ..) = signature("print(gcd(4, |))\n");
    assert!(label.starts_with("gcd("), "{label}");
}

#[test]
fn inlay_hints_name_the_parameters_of_literal_arguments() {
    let src =
        "fn walk(from: Int, steps: Int) -> Int = from + steps\nlet a = 1\nprint(walk(a, 3))\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let hints: Vec<(u32, u32, String, u32)> = reply[0]
        .get("result")
        .as_array()
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["position", "character"]).as_u32().unwrap(),
                h.get("label").as_str().unwrap().to_string(),
                h.get("kind").as_u32().unwrap(),
            )
        })
        .collect();
    // `a` names itself; the literal `3` gets its parameter's name.
    assert_eq!(
        hints,
        [
            (1, 5, ": Int".to_string(), 1),
            (2, 14, "steps:".to_string(), 2)
        ]
    );
}

#[test]
fn the_server_advertises_completion_and_signature_help() {
    let mut server = Server::default();
    let reply = server.handle(&request(1, "initialize", obj([])));
    let caps = reply[0].at(&["result", "capabilities"]);
    assert_eq!(
        caps.at(&["completionProvider", "triggerCharacters"])
            .as_array(),
        [Value::from(".")]
    );
    assert_eq!(
        caps.at(&["signatureHelpProvider", "triggerCharacters"])
            .as_array(),
        [Value::from("("), Value::from(",")]
    );
}

#[test]
fn dot_completion_on_library_structs_tells_properties_from_methods() {
    let items = completions("var q = Deque<Int>()\nq.|\n");
    let kind = |label: &str| {
        items
            .iter()
            .find(|i| i.0 == label)
            .unwrap_or_else(|| panic!("no `{label}` in {items:?}"))
            .1
    };
    // `len` and `first` are `@get` properties; `push` is called.
    assert_eq!((kind("len"), kind("first"), kind("push")), (10, 10, 2));
    let items = completions("var h = MinHeap<Int>()\nh.|\n");
    assert!(
        labels(&items).contains(&"push") && labels(&items).contains(&"len"),
        "{items:?}"
    );
}

/// The `(line, character)` of each location or edit in a reply.
fn starts(values: &[Value]) -> Vec<(u32, u32)> {
    values
        .iter()
        .map(|v| {
            let start = v.at(&["range", "start"]);
            (
                start.get("line").as_u32().unwrap(),
                start.get("character").as_u32().unwrap(),
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
    starts(reply.as_array())
}

/// Like `ask_at`, with more request params.
fn ask_at_with(method: &str, src: &str, extra: Value) -> Value {
    let at = src.find('|').expect("a cursor");
    let text = src.replacen('|', "", 1);
    let line = src[..at].matches('\n').count() as u32;
    let character = (at - src[..at].rfind('\n').map_or(0, |i| i + 1)) as u32;
    let (mut server, _) = open(&text);
    let mut params = vec![
        ("textDocument".to_string(), doc()),
        ("position".to_string(), pos(line, character)),
    ];
    if let Value::Object(more) = extra {
        params.extend(more);
    }
    let reply = server.handle(&request(21, method, Value::Object(params)));
    let result = reply[0].get("result").clone();
    if result.is_null() && !reply[0].get("error").is_null() {
        return reply[0].get("error").clone();
    }
    result
}

#[test]
fn references_find_locals_params_fns_structs_domains_and_fields() {
    // A local, and a shadowing one of the same name that is not it.
    let src = "fn f(n: Int) -> Int:\n    let x = n\n    let y = x| + x\n    let x = 5\n    x\n";
    assert_eq!(references_at(src), [(1, 8), (2, 12), (2, 16)]);
    // A parameter.
    assert_eq!(
        references_at("fn f(n|: Int) -> Int = n * n\n"),
        [(0, 5), (0, 22), (0, 26)]
    );
    // A fn, from a call.
    let src = "fn sq(n: Int) -> Int = n * n\nprint(sq(2) + s|q(3))\n";
    assert_eq!(references_at(src), [(0, 3), (1, 6), (1, 14)]);
    // A struct and a field, used through a value and inside a method.
    let src = format!("{ROAD}let r = Road(1, 2)\nprint(r.w|)\n");
    // The declaration and the access through a value.
    assert_eq!(references_at(&src), [(2, 4), (12, 8)]);
    let src = format!("{ROAD}let r = Ro|ad(1, 2)\n");
    assert_eq!(references_at(&src), [(0, 7), (10, 31), (11, 8)]);
    // A domain.
    let src = "domain V = 0..5\nvar a: [V]Int = 0\nfor v in V|:\n    a[v] = 1\n";
    assert_eq!(references_at(src), [(0, 7), (1, 8), (2, 9)]);
}

#[test]
fn rename_edits_every_reference_and_refuses_library_names() {
    let src = "fn sq(n: Int) -> Int = n * n\nprint(sq(2) + s|q(3))\n";
    let prepared = ask_at("textDocument/prepareRename", src);
    assert_eq!(prepared.get("placeholder").as_str(), Some("sq"));
    let edit = ask_at_with(
        "textDocument/rename",
        src,
        obj([("newName", "square".into())]),
    );
    let edits = edit.at(&["changes", URI]).as_array();
    assert_eq!(starts(edits), [(0, 3), (1, 6), (1, 14)]);
    assert!(edits
        .iter()
        .all(|e| e.get("newText").as_str() == Some("square")));
    // Not a name, a keyword, or a name the prelude owns.
    let bad = ask_at_with("textDocument/rename", src, obj([("newName", "fn".into())]));
    assert!(
        bad.get("message").as_str().unwrap().contains("keyword"),
        "{bad:?}"
    );
    let bad = ask_at_with("textDocument/rename", src, obj([("newName", "a b".into())]));
    assert!(
        bad.get("message").as_str().unwrap().contains("not a name"),
        "{bad:?}"
    );
    let library = ask_at("textDocument/prepareRename", "print(gc|d(4, 6))\n");
    assert!(library.is_null(), "{library:?}");
}

#[test]
fn dot_completion_on_built_in_types_reads_the_compiler_catalog() {
    let kind_of = |items: &[(String, u32, String)], label: &str| {
        let item = items.iter().find(|i| i.0 == label);
        item.unwrap_or_else(|| panic!("no `{label}` in {items:?}"))
            .clone()
    };
    // A list: `len` reads as a property, `push` is called, `T` is `Int`.
    let items = completions("var xs = [3, 1, 2]\nxs.|\n");
    assert_eq!(kind_of(&items, "len").1, 10);
    assert_eq!(kind_of(&items, "push").1, 2);
    assert_eq!(kind_of(&items, "get").2, "get(i: Int) -> ?Int");
    // A string, a map with its key and value types, a set, an array.
    let items = completions("let s = \"hello\"\ns.|\n");
    assert!(labels(&items).contains(&"find_all"), "{items:?}");
    let items = completions("var m: Map<Int, Str> = Map()\nm.|\n");
    assert_eq!(kind_of(&items, "first").2, "first: ?(Int, Str)");
    assert_eq!(kind_of(&items, "pop_first").1, 2);
    let items = completions("var st: Set<Int> = Set()\nst.|\n");
    assert!(labels(&items).contains(&"first_ge"), "{items:?}");
    let items = completions("domain D = 0..3\nvar a: [D]Int = 0\na.|\n");
    assert_eq!(kind_of(&items, "fill").2, "fill(x: Int)");
    // On a half-typed line too.
    let items = completions("var xs = [3, 1, 2]\nlet k = xs.|\nprint(k)\n");
    assert!(labels(&items).contains(&"pop"), "{items:?}");
}

#[test]
fn signature_help_covers_built_in_methods() {
    let help = signature("var xs = [3, 1, 2]\nxs.insert(0, |)\n");
    assert_eq!(help.0, "insert(i: Int, x: Int)");
    assert_eq!(help.1, 1);
}

#[test]
fn bare_names_include_the_compiler_prelude() {
    let items = completions("let n = 3\npr|\n");
    let print = items
        .iter()
        .find(|i| i.0 == "print")
        .expect("print is offered");
    assert_eq!(print.1, 3);
    let items = completions("let n: I| = 3\n");
    assert!(
        labels(&items).contains(&"Int") && labels(&items).contains(&"I128"),
        "{items:?}"
    );
    // A stdlib fn that needs a `use` is not in scope.
    assert!(!labels(&completions("z_|\n")).contains(&"z_function"));
}
