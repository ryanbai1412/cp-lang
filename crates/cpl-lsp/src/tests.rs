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
    obj([("jsonrpc", "2.0".into()), ("id", id.into()), ("method", method.into()), ("params", params)])
}

fn notification(method: &str, params: Value) -> Value {
    obj([("jsonrpc", "2.0".into()), ("method", method.into()), ("params", params)])
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
            obj([("uri", URI.into()), ("languageId", "cp".into()), ("version", 1.into()), ("text", text.into())]),
        )]),
    )
}

/// A ranged edit: replace `start..end` with `text`.
fn did_change(start: (u32, u32), end: (u32, u32), text: &str) -> Value {
    notification(
        "textDocument/didChange",
        obj([
            ("textDocument", obj([("uri", URI.into()), ("version", 2.into())])),
            (
                "contentChanges",
                vec![obj([
                    ("range", obj([("start", pos(start.0, start.1)), ("end", pos(end.0, end.1))])),
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
    assert!(server.handle(&notification("initialized", obj([]))).is_empty());
    let published = server.handle(&did_open(text));
    (server, diagnostics_of(&published))
}

fn diagnostics_of(messages: &[Value]) -> Vec<Value> {
    assert_eq!(messages.len(), 1, "{messages:?}");
    assert_eq!(messages[0].get("method").as_str(), Some("textDocument/publishDiagnostics"));
    assert_eq!(messages[0].at(&["params", "uri"]).as_str(), Some(URI));
    messages[0].at(&["params", "diagnostics"]).as_array().to_vec()
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
    assert_eq!(reply[0].at(&["result", "serverInfo", "name"]).as_str(), Some("cpl"));
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
    let (_, diagnostics) = open(&SOURCE.replace("dist[v]", "dist[w]").replace("print(total)", "n = 1"));
    assert_eq!(diagnostics.len(), 2);
    let d = &diagnostics[0];
    assert_eq!(d.get("code").as_str(), Some("E0201"));
    assert_eq!(d.get("source").as_str(), Some("cpl"));
    assert_eq!(d.get("severity").as_u32(), Some(1));
    assert_eq!(d.get("message").as_str(), Some("no `w` in scope\nhelp: declare it first: `let w = ...`"));
    assert_eq!(d.at(&["range", "start"]), &pos(6, 18));
    assert_eq!(d.at(&["range", "end"]), &pos(6, 19));
    let d = &diagnostics[1];
    assert_eq!(d.get("code").as_str(), Some("E0401"));
    assert_eq!(d.at(&["range", "start"]), &pos(7, 0));

    let (_, diagnostics) = open("type R = Int\ntype R = Bool\n");
    let related = diagnostics[0].get("relatedInformation").as_array();
    assert_eq!(related[0].get("message").as_str(), Some("first defined here"));
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
            ("contentChanges", vec![obj([("text", "let x: Bool = 1\n".into())])].into()),
        ]),
    );
    let diagnostics = diagnostics_of(&server.handle(&change));
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E0302"));
}

#[test]
fn edits_use_utf16_positions() {
    // '𝄞' is two UTF-16 units: it spans characters 9 to 11.
    let (mut server, diagnostics) = open("let a = \"𝄞\" + 1\n");
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(0, 13));
    // Replace exactly the clef with three ASCII letters.
    let published = server.handle(&did_change((0, 9), (0, 11), "abc"));
    let diagnostics = diagnostics_of(&published);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(0, 14));

    let (_, diagnostics) = open("let s = \"é𝄞\" + 1\n");
    assert_eq!(diagnostics[0].at(&["range", "start"]), &pos(0, 14));
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
    assert!(hover.at(&["contents", "value"]).as_str().unwrap().contains("dist: [V]?Int"));
    let hover = hover_at(&mut server, 7, 7);
    assert_eq!(hover.at(&["contents", "value"]).as_str(), Some("```cp\ntotal: Int\n```"));
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

    let reply = server.handle(&request(4, "textDocument/documentSymbol", obj([("textDocument", doc())])));
    let names: Vec<&str> = reply[0].get("result").as_array().iter().filter_map(|s| s.get("name").as_str()).collect();
    assert_eq!(names, vec!["n", "V", "E", "dist", "total"]);
    assert_eq!(reply[0].get("result").as_array()[1].get("kind").as_u32(), Some(26));
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
    let published = server.handle(&notification("textDocument/didClose", obj([("textDocument", doc())])));
    assert!(diagnostics_of(&published).is_empty());
    let reply = server.handle(&request(9, "textDocument/hover", obj([("textDocument", doc()), ("position", pos(0, 4))])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
}

#[test]
fn unknown_and_malformed_messages_get_errors_not_crashes() {
    let (mut server, _) = open(SOURCE);
    let reply = server.handle(&request(2, "textDocument/rename", obj([])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32601.0));
    assert!(server.handle(&notification("workspace/didChangeWatchedFiles", obj([]))).is_empty());
    let reply = server.handle(&request(3, "textDocument/hover", obj([("textDocument", doc())])));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
    let reply = server.handle(&request(4, "textDocument/hover", Value::Null));
    assert_eq!(reply[0].at(&["error", "code"]), &Value::Number(-32602.0));
    // Positions far outside the document clamp.
    assert!(hover_at(&mut server, 9999, 9999).is_null());
    // A change for a document that is not open.
    let mut other = did_change((0, 0), (0, 0), "x");
    if let Value::Object(members) = &mut other {
        members[2].1 = obj([("textDocument", obj([("uri", "file:///nope.cp".into())])), ("contentChanges", Value::Array(Vec::new()))]);
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
    assert!(reply[0].at(&["error", "message"]).as_str().unwrap().contains("deliberate test panic"));
    let diagnostics = diagnostics_of(&reply[1..]);
    assert_eq!(diagnostics[0].get("code").as_str(), Some("E9999"));
    // The open document survived, and the next request is answered.
    assert!(hover_at(&mut server, 6, 13).at(&["contents", "value"]).as_str().unwrap().contains("dist: [V]?Int"));
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
        request(2, "textDocument/hover", obj([("textDocument", doc()), ("position", pos(1, 7))])),
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
    assert_eq!(serve(std::io::BufReader::new(&input[..]), &mut output).unwrap(), 1);
    let replies = unframed(&output);
    assert_eq!(replies[0].at(&["error", "code"]), &Value::Number(-32700.0));
    assert_eq!(replies[1].get("id").as_u32(), Some(1));

    let mut output = Vec::new();
    assert!(serve(std::io::BufReader::new(&b"Content-Type: x\r\n\r\n"[..]), &mut output).is_err());
}
