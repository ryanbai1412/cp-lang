use crate::json::{self, obj, Value};
use crate::{serve, write_message, Server};

const URI: &str = "file:///work/roads.cp";

const SOURCE: &str = "\
let n: 1..=2e5 = read()
domain V = 1..=n
domain E = 0..n
let dist: [V]?Int = none
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

/// `text` with the edits of the first quick fix for its first diagnostic.
fn quick_fixed(server: &mut Server, uri: &str, text: &str, diagnostic: &Value) -> (String, String) {
    let params = obj([
        ("textDocument", obj([("uri", uri.into())])),
        ("range", diagnostic.get("range").clone()),
        (
            "context",
            obj([("diagnostics", vec![diagnostic.clone()].into())]),
        ),
    ]);
    let reply = server.handle(&request(9, "textDocument/codeAction", params));
    let actions = reply[0].get("result").as_array();
    let action = actions
        .first()
        .unwrap_or_else(|| panic!("no fix for {diagnostic:?}"));
    let offset = |p: &Value| {
        let line = p.get("line").as_u32().unwrap() as usize;
        let start: usize = text.split_inclusive('\n').take(line).map(str::len).sum();
        start + p.get("character").as_u32().unwrap() as usize
    };
    let mut edits: Vec<(usize, usize, String)> = action
        .at(&["edit", "changes", uri])
        .as_array()
        .iter()
        .map(|e| {
            let (start, end) = (
                offset(e.at(&["range", "start"])),
                offset(e.at(&["range", "end"])),
            );
            (start, end, e.get("newText").as_str().unwrap().to_string())
        })
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut fixed = text.to_string();
    for (start, end, new) in edits {
        fixed.replace_range(start..end, &new);
    }
    (action.get("title").as_str().unwrap().to_string(), fixed)
}

/// The most frequent first errors over the broken ports
/// (`crates/cpl-cli/tests/recovery.rs` prints the table), each broken into
/// a real port and undone by its one-click fix.
#[test]
fn quick_fixes_undo_the_most_frequent_mistakes_in_a_real_port() {
    assert_fixes_restore(
        "cf2062g",
        &[
            // E0101: a call left open.
            (
                "let p: [I]I = read()",
                "let p: [I]I = read(",
                "E0101",
                "insert `)`",
            ),
            // E0103: a block without its colon.
            (
                "fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:",
                "fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>",
                "E0103",
                "insert `:`",
            ),
            // E0102: an operator typed twice.
            (
                "abs(i.ord - j.ord) + abs(",
                "abs(i.ord - j.ord) + * abs(",
                "E0102",
                "remove the extra `*`",
            ),
            // E0004: a stray character.
            ("fn sort_perm<D>", "fn $sort_perm<D>", "E0004", "remove `$`"),
        ],
    );
}

/// A stdlib name the port forgot to import is one quick fix away, and the
/// `use` lands where the port had it.
#[test]
fn quick_fixes_import_a_stdlib_name_where_the_file_imports_its_module() {
    assert_fixes_restore(
        "cf2062e2",
        &[(
            "use std.seq.{buckets}\n",
            "",
            "E0201",
            "add `use std.seq.{buckets}`",
        )],
    );
    assert_fixes_restore(
        "ucup5s05f",
        &[(
            "{suffix_array, lcp_array}",
            "{suffix_array}",
            "E0201",
            "add `use std.strings.{lcp_array}`",
        )],
    );
}

#[test]
fn completion_offers_unimported_stdlib_names_with_their_use() {
    let src = "use std.math.{gcd}\nprint(floor_s|)\n";
    let reply = ask_at("textDocument/completion", src);
    let items = reply.get("items").as_array();
    let item = items
        .iter()
        .find(|i| i.get("label").as_str() == Some("floor_sum"))
        .unwrap_or_else(|| panic!("no floor_sum in {items:?}"));
    assert_eq!(item.at(&["labelDetails", "description"]).as_str(), Some("std.math"));
    assert!(item.get("detail").as_str().unwrap().starts_with("fn floor_sum"));
    let edits = item.get("additionalTextEdits").as_array();
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].get("newText").as_str(), Some(", floor_sum"));
    assert_eq!(edits[0].at(&["range", "start", "line"]).as_u32(), Some(0));
    assert_eq!(edits[0].at(&["range", "start", "character"]).as_u32(), Some(17));
    // An imported name, or one the prelude brings, needs no `use`.
    let src = "use std.math.{gcd}\nprint(gc|)\n";
    let reply = ask_at("textDocument/completion", src);
    let gcd: Vec<&Value> = reply
        .get("items")
        .as_array()
        .iter()
        .filter(|i| i.get("label").as_str() == Some("gcd"))
        .collect();
    assert_eq!(gcd.len(), 1);
    assert!(gcd[0].get("additionalTextEdits").as_array().is_empty());
}

/// A closer dropped mid-line is put back where it was, before the `:` or
/// `=` it can't enclose, not at the end of the file.
#[test]
fn quick_fixes_close_a_bracket_where_it_was_dropped() {
    assert_fixes_restore(
        "cf2084c",
        &[(
            "target[mid] = I(n / 2 + 1)",
            "target[mid = I(n / 2 + 1)",
            "E0101",
            "insert `]`",
        )],
    );
    assert_fixes_restore(
        "cf2150c",
        &[(
            "if let (nxt_k, nxt_v) = deltas",
            "if let (nxt_k, nxt_v = deltas",
            "E0101",
            "insert `)`",
        )],
    );
}

/// Each `(good, bad, code, title)` breaks the ecnerwala `port` by replacing
/// `good` with `bad`; its first diagnostic is `code` and the first quick
/// fix, titled `title`, restores the port.
fn assert_fixes_restore(port: &str, breaks: &[(&str, &str, &str, &str)]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../proto/ports/ecnerwala/{port}.cp"));
    let original = std::fs::read_to_string(&path).unwrap();
    let uri = format!("file://{}", path.display());
    for (good, bad, code, title) in breaks {
        assert!(original.contains(good), "{good}");
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
        let diagnostics = published[0].at(&["params", "diagnostics"]).as_array();
        let first = diagnostics
            .first()
            .unwrap_or_else(|| panic!("`{bad}` checked clean"));
        assert_eq!(first.get("code").as_str(), Some(*code), "{first:?}");
        let (got, fixed) = quick_fixed(&mut server, &uri, &broken, first);
        assert_eq!(got, *title);
        assert_eq!(
            fixed, original,
            "the fix for `{bad}` did not restore the port"
        );
    }
}

#[test]
fn quick_fixes_cover_c_spellings_and_mutation() {
    let src = SOURCE.replace("var total", "let total").replace("for v in V:", "for v in V:\n    if v > 1 && v < n:\n        total += 0");
    let (mut server, diagnostics) = open(&src);
    let reply = server.handle(&request(
        6,
        "textDocument/codeAction",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(12, 0))])),
            ("context", obj([("diagnostics", diagnostics.into())])),
        ]),
    ));
    let titles: Vec<&str> = reply[0]
        .get("result")
        .as_array()
        .iter()
        .filter_map(|a| a.get("title").as_str())
        .collect();
    assert_eq!(titles, vec!["replace with `and`", "declare `total` with `var`"]);
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
    let reply = server.handle(&request(2, "textDocument/selectionRange", obj([])));
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
        .contains("internal error in cpl while handling `cpl/testPanic`: deliberate test panic"));
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
        "use std.rand.{Rng}\nvar r = Rng.new(7)\nprint(r.int(1, 6))\n";

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
        def_uri.ends_with("/stdlib/rand.cp"),
        "expected definition in stdlib/rand.cp, got: {def_uri}"
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
         Uses Lucas's theorem automatically when `n >= M.MOD` (for prime `P`)."
    );
    let def = server.handle(&request(3, "textDocument/definition", at(2, 9)));
    assert!(def[0]
        .at(&["result", "uri"])
        .as_str()
        .unwrap()
        .ends_with("/stdlib/mod.cp"));
    let choose_line = include_str!("../../../stdlib/mod.cp")
        .lines()
        .position(|l| l.trim_start().starts_with("pub fn choose("))
        .unwrap() as u32;
    assert_eq!(
        def[0].at(&["result", "range", "start"]),
        &pos(choose_line, 11)
    );
    assert_eq!(
        def[0].at(&["result", "range", "end"]),
        &pos(choose_line, 17)
    );
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
    assert_eq!(
        edits[0].get("newText").as_str(),
        Some("let x = 1\nprint(x)\n")
    );
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
    let (mut server, _) =
        open("let n = 3\nlet m: Int = 4\nvar xs = [n, m]\nfor x in xs:\n    print(x)\n");
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
    assert_eq!(
        shown,
        [(0, 5, ": Int"), (2, 6, ": List<Int>"), (3, 5, ": Int")]
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
        .get("result")
        .as_array()
        .iter()
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.get("label").as_str().unwrap().to_string(),
            )
        })
        .collect();
    let at = |line, character| (line, character, ": Int".to_string());
    assert_eq!(shown, [at(1, 6), at(1, 9), at(2, 6), at(2, 9)]);
}

#[test]
fn inlay_hints_show_the_index_space_of_lambda_parameters() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../proto/ports/ecnerwala/cf2062g.cp");
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
        .get("result")
        .as_array()
        .iter()
        .filter(|hint| hint.at(&["position", "line"]).as_u32() == Some(line))
        .map(|hint| {
            (
                hint.at(&["position", "character"]).as_u32().unwrap(),
                hint.get("label").as_str().unwrap().to_string(),
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
        .get("result")
        .as_array()
        .iter()
        .filter(|hint| hint.get("kind").as_u32() == Some(1))
        .map(|hint| {
            (
                hint.at(&["position", "line"]).as_u32().unwrap(),
                hint.get("label").as_str().unwrap(),
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
fn completion_and_hover_see_names_from_a_declaration_that_did_not_parse() {
    let src = "let total = 2 * $x\nfn main(n: Int):\n    let half = n / $2\n    print(to|)\n";
    let names = labels(&completions(src))
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    assert!(
        names.contains(&"total".into()) && names.contains(&"half".into()),
        "{names:?}"
    );
    let hover = ask_at(
        "textDocument/hover",
        "let ($left, right) = (4, 6)\nprint(ri|ght)\n",
    );
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\nright: Int\n```")
    );
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
    let items = completions("use std.graph.{|}\n");
    assert!(labels(&items).contains(&"bfs01"), "{items:?}");
    assert!(
        !labels(&items)
            .iter()
            .any(|name| name.ends_with("_of_pairs") || name.ends_with("_of_triples")),
        "{items:?}"
    );
    assert!(completions("// see xs.|\n").is_empty());
}

fn signature(src: &str) -> (String, u32, Vec<String>) {
    let help = ask_at("textDocument/signatureHelp", src);
    assert!(!help.is_null(), "no signature help in {src:?}");
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

/// A local of the same name in a fn, a `for` or a lambda is another
/// binding, so references and rename leave it alone.
#[test]
fn references_skip_the_bindings_that_shadow_the_name() {
    let src = "let x| = 1\nfn f() -> Int:\n    let x = 2\n    x\nprint(x)\n";
    assert_eq!(references_at(src), [(0, 4), (4, 6)]);
    let src = "let x = 1\nfn f() -> Int:\n    let x| = 2\n    x\nprint(x)\n";
    assert_eq!(references_at(src), [(2, 8), (3, 4)]);
    let src = "let x = 1\nfor x| in 0..3:\n    print(x)\nprint(x)\n";
    assert_eq!(references_at(src), [(1, 4), (2, 10)]);
    let src = "let x| = 1\nlet f = (x) => x + 1\nprint(f(x))\n";
    assert_eq!(references_at(src), [(0, 4), (2, 8)]);
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
fn rename_reaches_every_open_file_and_the_library_that_declares_the_name() {
    let root = std::env::temp_dir().join(format!("cpl-rename-{}", std::process::id()));
    std::fs::create_dir_all(root.join("kit")).unwrap();
    std::fs::write(root.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib =
        "pub fn area(w: Int, h: Int) -> Int = w * h\npub fn square(s: Int) -> Int = area(s, s)\n";
    std::fs::write(root.join("kit/geo.cp"), lib).unwrap();
    let uri = |name: &str| format!("file://{}", root.join(name).display());
    let (a, b, geo) = (uri("a.cp"), uri("b.cp"), uri("kit/geo.cp"));
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    let open_at = |server: &mut Server, uri: &str, text: &str| {
        let doc = obj([
            ("uri", uri.into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", text.into()),
        ]);
        let published = server.handle(&notification(
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        let diagnostics = published[0].at(&["params", "diagnostics"]).as_array();
        assert!(diagnostics.is_empty(), "{published:?}");
    };
    open_at(&mut server, &a, "use kit.geo.{area}\nprint(area(2, 3))\n");
    open_at(
        &mut server,
        &b,
        "use kit.geo.{area}\nlet area2 = area(4, 4)\nprint(area(1, 1) + area2)\n",
    );
    let rename = |server: &mut Server, uri: &str, line: u32, col: u32| {
        let params = obj([
            ("textDocument", obj([("uri", uri.into())])),
            ("position", pos(line, col)),
            ("newName", "rect".into()),
        ]);
        server.handle(&request(2, "textDocument/rename", params))[0].clone()
    };
    let lines = |reply: &Value, uri: &str| -> Vec<(u32, u32)> {
        let edits = reply.at(&["result", "changes", uri]).as_array();
        starts(edits)
    };
    // From a use: the import and both calls here, the other open file, and
    // the library file, which is not open.
    let reply = rename(&mut server, &a, 1, 7);
    assert_eq!(lines(&reply, &a), [(0, 13), (1, 6)]);
    assert_eq!(lines(&reply, &b), [(0, 13), (1, 12), (2, 6)]);
    assert_eq!(lines(&reply, &geo), [(0, 7), (1, 31)]);
    // A file of the project that is not open, from the declaration too.
    std::fs::write(root.join("c.cp"), "use kit.geo.{area}\nprint(area(5, 5))\n").unwrap();
    std::fs::write(root.join("d.cp"), "fn area() -> Int = 1\nprint(area())\n").unwrap();
    let reply = rename(&mut server, &a, 1, 7);
    let c = root.join("c.cp").display().to_string();
    let c_uri = reply.at(&["result", "changes"]);
    let Value::Object(changed) = c_uri else {
        panic!("{reply:?}");
    };
    let c_key = changed
        .iter()
        .map(|(k, _)| k.clone())
        .find(|k| k.ends_with("/c.cp"))
        .unwrap_or_else(|| panic!("{c} not renamed: {reply:?}"));
    assert_eq!(lines(&reply, &c_key), [(0, 13), (1, 6)]);
    assert!(!changed.iter().any(|(k, _)| k.ends_with("/d.cp")), "{reply:?}");
    std::fs::remove_file(root.join("c.cp")).unwrap();
    std::fs::remove_file(root.join("d.cp")).unwrap();
    // From the declaration, once the library is open too: each file once.
    open_at(&mut server, &geo, lib);
    let reply = rename(&mut server, &geo, 0, 8);
    assert_eq!(lines(&reply, &geo), [(0, 7), (1, 31)]);
    assert_eq!(lines(&reply, &a), [(0, 13), (1, 6)]);
    let Value::Object(changes) = reply.at(&["result", "changes"]) else {
        panic!("{reply:?}");
    };
    assert_eq!(changes.len(), 3, "{changes:?}");
    // A name the library already declares would collide.
    let params = obj([
        ("textDocument", obj([("uri", a.as_str().into())])),
        ("position", pos(1, 7)),
        ("newName", "square".into()),
    ]);
    let reply = server.handle(&request(3, "textDocument/rename", params));
    let message = reply[0].at(&["error", "message"]).as_str().unwrap_or("");
    assert!(
        message.contains("`square` is already a name in"),
        "{reply:?}"
    );
    std::fs::remove_dir_all(&root).unwrap();
    // The standard library's names stay its own.
    let (mut server, _) = open("use std.mod.{Comb}\nlet c = Comb<7>.new()\n");
    let params = obj([("textDocument", doc()), ("position", pos(1, 9))]);
    let reply = server.handle(&request(3, "textDocument/prepareRename", params));
    let message = reply[0].at(&["error", "message"]).as_str().unwrap_or("");
    assert!(message.contains("standard library"), "{reply:?}");
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
    // A stdlib fn that needs a `use` comes with it, after what is in scope.
    let reply = ask_at("textDocument/completion", "z_|\n");
    let z = reply
        .get("items")
        .as_array()
        .iter()
        .find(|i| i.get("label").as_str() == Some("z_function"))
        .cloned()
        .expect("z_function is offered");
    assert!(z.get("sortText").as_str().unwrap().starts_with('5'), "{z:?}");
    assert_eq!(
        z.at(&["additionalTextEdits"]).as_array()[0].get("newText").as_str(),
        Some("use std.strings.{z_function}\n")
    );
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
        .get("result")
        .as_array()
        .iter()
        .map(|h| {
            (
                h.at(&["range", "start", "line"]).as_u32().unwrap(),
                h.at(&["range", "start", "character"]).as_u32().unwrap(),
                h.get("kind").as_u32().unwrap(),
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
        .get("result")
        .as_array()
        .iter()
        .map(|r| {
            (
                r.get("startLine").as_u32().unwrap(),
                r.get("endLine").as_u32().unwrap(),
                r.get("kind").as_str(),
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
        .get("result")
        .as_array()
        .iter()
        .map(|r| {
            (
                r.get("startLine").as_u32().unwrap(),
                r.get("endLine").as_u32().unwrap(),
                r.get("kind").as_str(),
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
            .get("result")
            .as_array()
            .iter()
            .map(|s| {
                (
                    s.get("name").as_str().unwrap().to_string(),
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
    let src = "let (a, b): (Int, Int) = read()\nprint(a + b)\n@brute(main)\nfn slow() = 0\n";
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
            .get("result")
            .as_array()
            .iter()
            .map(|l| {
                assert_eq!(l.at(&["command", "command"]).as_str(), Some("cpl.exec"));
                (
                    l.at(&["command", "title"]).as_str().unwrap().to_string(),
                    l.at(&["command", "arguments"])
                        .as_array()
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
        .get("result")
        .as_array()
        .iter()
        .filter(|h| h.get("kind").as_u32() == Some(1))
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.get("label").as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(labels, [(1, ": Int in 2..=10001"), (2, ": Int")]);
}

/// The labels completion ranks first, the ones that fit the expected type.
fn best_fits(src: &str) -> Vec<String> {
    let mut fits: Vec<String> = ask_at("textDocument/completion", src)
        .get("items")
        .as_array()
        .iter()
        .filter(|item| item.get("sortText").as_str().unwrap().starts_with('0'))
        .map(|item| item.get("label").as_str().unwrap().to_string())
        .collect();
    fits.sort();
    fits
}

#[test]
fn completion_ranks_first_what_fits_the_expected_type() {
    let decls = "fn area(w: Int, h: Int) -> Int = w * h\nfn name() -> Str = \"a\"\nlet k: Int = 3\nlet s = \"x\"\nvar c = 0\n";
    let fits = |src: String, want: &[&str], not: &[&str]| {
        let got = best_fits(&src);
        for w in want {
            assert!(
                got.iter().any(|g| g == w),
                "{w} should fit in {src:?}: {got:?}"
            );
        }
        for n in not {
            assert!(
                !got.iter().any(|g| g == n),
                "{n} should not fit in {src:?}: {got:?}"
            );
        }
    };
    let ints = ["area", "c", "k", "gcd"];
    let strs = ["name", "s"];
    // An argument: the parameter's type; a named one: that parameter's.
    fits(format!("{decls}print(area(k, |))\n"), &ints, &strs);
    fits(format!("{decls}print(area(h: |))\n"), &ints, &strs);
    // A `let` with its type written, and an assignment.
    fits(format!("{decls}let t: Str = |\n"), &strs, &ints);
    fits(format!("{decls}c = |\n"), &["area", "k"], &strs);
    fits(format!("{decls}c += |\n"), &["area", "k"], &strs);
    // After a dot, the members of the right type.
    assert_eq!(
        best_fits(&format!("{ROAD}let r = Road(1, 2)\nlet b: Bool = r.|\n")),
        ["heavy"]
    );
    // Nothing expected: no ranking by type.
    assert_eq!(
        best_fits(&format!("{decls}print(|)\n")),
        Vec::<String>::new()
    );
}

#[test]
fn signature_help_shows_defaults_and_named_only_parameters() {
    let decl = "fn walk(from: Int, steps: Int = 1; back: Bool = false) -> Int = from\n";
    let (label, active, params) = signature(&format!("{decl}print(walk(1, |))\n"));
    assert_eq!(
        label,
        "walk(from: Int, steps: Int = 1; back: Bool = false) -> Int"
    );
    assert_eq!(
        params,
        ["from: Int", "steps: Int = 1", "back: Bool = false"]
    );
    assert_eq!(active, 1);
    // A named argument is its parameter, even one only passed by name.
    let (_, active, _) = signature(&format!("{decl}print(walk(1, back: |))\n"));
    assert_eq!(active, 2);
    // Past the positional ones, the first option not given yet.
    let (_, active, _) = signature(&format!("{decl}print(walk(1, 2, |))\n"));
    assert_eq!(active, 2);
    let (_, active, _) = signature(&format!("{decl}print(walk(1, 2, back: true, |))\n"));
    assert!(active > 2, "{active}");
    // A stdlib method's default, as its declaration writes it.
    let (label, ..) = signature("use std.mod.{Comb}\nlet c = Comb<7>.new(|)\n");
    assert!(label.starts_with("new(limit: Int = 0) -> Comb<"), "{label}");
}

/// The lints `cpl lint` reports are published with the checker's errors,
/// each with the fix `cpl fix` would apply as a quick fix, and the whole of
/// `cpl fix` is one fix-all action.
#[test]
fn lints_are_published_with_their_cpl_fix_codemods_as_quick_fixes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../proto/ports/ecnerwala/cf2062g.cp");
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
    let diagnostics = published[0].at(&["params", "diagnostics"]).as_array();
    let never_changed = diagnostics
        .iter()
        .find(|d| d.get("code").as_str() == Some("L0201"))
        .unwrap_or_else(|| panic!("no L0201 in {diagnostics:?}"));
    assert_eq!(never_changed.get("severity").as_u32(), Some(2));
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
    let actions = reply[0].get("result").as_array();
    let fix_all = actions
        .iter()
        .find(|a| a.get("kind").as_str() == Some("source.fixAll.cpl"))
        .expect("a fix-all action");
    let edit = &fix_all.at(&["edit", "changes", uri.as_str()]).as_array()[0];
    let all = edit.get("newText").as_str().unwrap();
    assert!(all.contains(good) && !all.contains(bad));
    let left = cpl_lint::lint(all, &cpl_lint::Config::default());
    assert!(left.iter().all(|d| d.fixes.is_empty()), "{left:?}");
    assert!(left.len() < diagnostics.len());
}

fn open_port(server: &mut Server, port: &str, edit: (&str, &str)) -> (String, String, Vec<Value>) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../proto/ports/ecnerwala/{port}.cp"));
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
        .to_vec();
    (uri, text, diagnostics)
}

/// Each spelling the language replaced, put back into a real port, is
/// reported with the `cpl fix` codemod as its quick fix, and the fix
/// restores the port.
#[test]
fn replaced_spellings_get_their_cpl_fix_codemod_as_a_quick_fix() {
    for (port, good, bad, code) in [
        (
            "cf2066b",
            "in a.items if x != 0",
            "in a.items where x != 0",
            "E0503",
        ),
        ("agc077a", "@!multitest\n", "--- multitest ---\n", "E0503"),
        (
            "cf2062h",
            "Z(2) ** inside - ways",
            "Z(2).pow(inside) - ways",
            "L0205",
        ),
        ("cf2066b", "(0..b.len).rev:", "(0..b.len()).rev:", "L0206"),
        ("cf2053i1", "reach.start", "reach.lo", "E0503"),
    ] {
        let mut server = Server::new();
        let (uri, text, diagnostics) = open_port(&mut server, port, (good, bad));
        let old = diagnostics
            .iter()
            .find(|d| d.get("code").as_str() == Some(code))
            .unwrap_or_else(|| panic!("{port}: no {code} for `{bad}` in {diagnostics:?}"));
        let (_, fixed) = quick_fixed(&mut server, &uri, &text, old);
        assert_eq!(
            fixed,
            text.replacen(bad, good, 1),
            "{port}: the fix for `{bad}`"
        );
    }
}

/// Fix-all is `cpl fix`: the removed syntax first, which lets the file
/// parse, then the lints it uncovers.
#[test]
fn fix_all_rewrites_removed_syntax_then_the_lints_it_uncovers() {
    let mut server = Server::new();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../proto/ports/ecnerwala/cf2066b.cp");
    let original = std::fs::read_to_string(&path).unwrap();
    let (uri, text, _) = open_port(
        &mut server,
        "cf2066b",
        ("(0..b.len).rev:", "(0..b.len()).rev:"),
    );
    let text = text.replacen("in a.items if x != 0", "in a.items where x != 0", 1);
    let change = obj([
        (
            "textDocument",
            obj([("uri", uri.as_str().into()), ("version", 2.into())]),
        ),
        (
            "contentChanges",
            vec![obj([("text", text.as_str().into())])].into(),
        ),
    ]);
    server.handle(&notification("textDocument/didChange", change));
    let params = obj([
        ("textDocument", obj([("uri", uri.as_str().into())])),
        (
            "range",
            obj([
                ("start", obj([("line", 0.into()), ("character", 0.into())])),
                ("end", obj([("line", 0.into()), ("character", 0.into())])),
            ]),
        ),
        (
            "context",
            obj([
                ("diagnostics", Vec::<Value>::new().into()),
                ("only", vec!["source.fixAll".into()].into()),
            ]),
        ),
    ]);
    let reply = server.handle(&request(2, "textDocument/codeAction", params));
    let actions = reply[0].get("result").as_array();
    let [fix_all] = actions else {
        panic!("{actions:?}")
    };
    assert_eq!(fix_all.get("kind").as_str(), Some("source.fixAll.cpl"));
    let edits = fix_all.at(&["edit", "changes", uri.as_str()]).as_array();
    assert_eq!(edits[0].get("newText").as_str(), Some(original.as_str()));
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
        .get("items")
        .as_array()
        .iter()
        .find(|item| item.get("label").as_str() == Some("area"))
        .unwrap()
        .clone();
    assert!(
        area.get("insertText").as_str().is_none_or(|t| t == "area"),
        "{area:?}"
    );

    let text = src.replacen("print(b.|)\n", "", 1);
    let (mut server, diagnostics) = open(&text);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.get("severity").as_u32() != Some(1)),
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
        sorted.starts_with("```cp\nsorted() -> List<T>\ncost: O(n log n)\n```"),
        "{sorted}"
    );
    let rev = shown(&mut server, 4, 23);
    assert!(
        rev.starts_with("```cp\nrev: List<Int>\ncost: O(1)\n```"),
        "{rev}"
    );
    let fold = shown(&mut server, 4, 28);
    assert!(fold.contains("-> T\ncost: O(n)\n```"), "{fold}");
    assert_eq!(shown(&mut server, 2, 4), "```cp\nm: Int in 2..=10001\n```");
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
        .get("result")
        .as_array()
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.get("label").as_str().unwrap(),
            )
        })
        .filter(|(_, label)| label.contains(" in "))
        .collect();
    assert_eq!(
        ranged,
        [
            (2, ": Int in 2..=200"),
            (6, ": Int in 2..=299"),
            (10, ": Int in 1..=10000")
        ]
    );
    let hover = hover_at(&mut server, 6, 12);
    assert!(
        hover
            .at(&["contents", "value"])
            .as_str()
            .unwrap()
            .contains("j: Int in 2..=299"),
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
    assert_fixes_restore(
        "cf1284e",
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
    .get("items")
    .as_array()
    .iter()
    .find(|i| i.get("label").as_str() == Some("reach"))
    .cloned()
    .unwrap();
    assert_eq!(reach.get("kind").as_u32(), Some(2));
    assert_eq!(
        reach.get("detail").as_str(),
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
    assert_eq!(help.get("activeParameter").as_u32(), Some(1), "{help:?}");
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
        .get("result")
        .as_array()
        .iter()
        .map(|r| {
            (
                r.get("startLine").as_u32().unwrap(),
                r.get("endLine").as_u32().unwrap(),
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
    assert!(hover_text(&mut server, 11, 8).contains("d: Int"));
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
        .get("result")
        .as_array()
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["position", "character"]).as_u32().unwrap(),
                h.get("label").as_str().unwrap(),
            )
        })
        .collect();
    for hint in [(10, 17, ": Int"), (11, 9, ": Int"), (14, 34, ": Int")] {
        assert!(hints.contains(&hint), "{hint:?} in {hints:?}");
    }
    let items = ask_at(
        "textDocument/completion",
        &src.replacen("vals[Node.Src]", "vals[Node.|]", 1),
    );
    let variants: Vec<&str> = items
        .get("items")
        .as_array()
        .iter()
        .filter_map(|i| i.get("label").as_str())
        .collect();
    assert_eq!(variants, ["Src", "Snk", "Cell"]);
    assert_fixes_restore(
        "cf2089d",
        &[(
            "e + \":{chain",
            "e ++ \":{chain",
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
    assert!(text.contains("Int"), "expected Int hover on punned arg, got: {hover:?}");
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
        .get("result")
        .as_array()
        .iter()
        .map(|r| {
            (
                r.get("startLine").as_u32().unwrap(),
                r.get("endLine").as_u32().unwrap(),
            )
        })
        .collect();
    assert!(ranges.contains(&(0, 2)), "expected loop fold (0, 2), got {ranges:?}");
    assert!(ranges.contains(&(3, 4)), "expected else fold (3, 4), got {ranges:?}");
}

#[test]
fn loop_counts_read_as_a_number_and_its_size() {
    let shown: Vec<String> = [7, 9999, 10000, 99999, 200000, 249999500000]
        .into_iter()
        .map(crate::server::count_text)
        .collect();
    assert_eq!(
        shown,
        ["7", "9999", "10000 (1e4)", "99999 (1e5)", "200000 (2e5)", "249999500000 (2.5e11)"]
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
        ("fn f(n: Int) -> Int:\n    let a| = 1\n    let b = 2\n    a + b\nprint(f(1))\n", "b"),
        ("fn f(n: Int) -> Int:\n    let a| = 1\n    a + n\nprint(f(1))\n", "n"),
        ("let x| = 1\nfn f() -> Int:\n    let y = 2\n    x + y\nprint(f())\n", "y"),
        ("let g = 5\nfn f() -> Int:\n    let a| = 1\n    a + g\nprint(f())\n", "g"),
        ("let g = 5\nfor i| in 0..3:\n    print(i + g)\n", "g"),
        ("let x| = 1\nlet f = (y) => x + y\nprint(f(2))\n", "y"),
    ];
    for (src, to) in refused {
        let reply = rename(src, to);
        let message = reply.get("message").as_str().unwrap_or_else(|| panic!("{src}: {reply:?}"));
        assert!(message.contains(&format!("`{to}`")), "{src}: {message}");
    }
    let allowed = [
        ("fn f() -> Int:\n    let a| = 1\n    a\nfn g() -> Int:\n    let b = 2\n    b\nprint(f() + g())\n", "b"),
        ("fn f(n: Int) -> Int:\n    let a| = 1\n    a + n\nprint(f(1))\n", "c"),
        ("let x = 1\nfn f() -> Int:\n    let a| = 2\n    a\nprint(f() + x)\n", "x"),
    ];
    for (src, to) in allowed {
        let reply = rename(src, to);
        assert!(matches!(reply.get("changes"), Value::Object(_)), "{src}: {reply:?}");
    }
}
