//! Session lifecycle: closing, shutdown, framing and requests that must not crash.

use super::*;

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
