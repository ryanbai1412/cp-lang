//! `workspace/executeCommand`: the server runs `cpl` (here a stub that
//! prints a saved report) and shows the report's `view` through the
//! protocol, so an editor only forwards the command.

use super::*;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

struct Stub {
    dir: PathBuf,
    uri: String,
    path: String,
}

impl Stub {
    fn new(name: &str) -> Stub {
        let dir = std::env::temp_dir().join(format!("cpl-lsp-exec-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let cpl = dir.join("cpl");
        let script = "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/argv\"\ncat \"$(dirname \"$0\")/out\"\n";
        std::fs::write(&cpl, script).unwrap();
        std::fs::set_permissions(&cpl, std::fs::Permissions::from_mode(0o755)).unwrap();
        let path = dir.join("a.cp").display().to_string();
        let uri = crate::uri::uri_of_path(&path);
        Stub { dir, uri, path }
    }

    /// The stub also keeps the standard input it is given.
    fn reads_stdin(&self) {
        let script = "#!/bin/sh\ncat > \"$(dirname \"$0\")/stdin\"\nprintf '%s\\n' \"$@\" > \"$(dirname \"$0\")/argv\"\ncat \"$(dirname \"$0\")/out\"\n";
        std::fs::write(self.dir.join("cpl"), script).unwrap();
    }

    fn exec_with(&self, server: &mut Server, args: &[&str], given: Value) -> Vec<Value> {
        let mut arguments: Vec<Value> = args.iter().map(|a| Value::from(*a)).collect();
        arguments.insert(1, self.path.as_str().into());
        arguments.push(given);
        let params = obj([
            ("command", "cpl.exec".into()),
            ("arguments", arguments.into()),
        ]);
        server.handle(&request(7, "workspace/executeCommand", params))
    }

    fn prints(&self, out: &str) {
        std::fs::write(self.dir.join("out"), out).unwrap();
    }

    fn argv(&self) -> String {
        std::fs::read_to_string(self.dir.join("argv"))
            .unwrap()
            .replace('\n', " ")
    }

    fn server(&self) -> Server {
        let mut server = Server::new().with_cpl(self.dir.join("cpl"));
        let refresh = obj([("refreshSupport", true.into())]);
        let caps = obj([("workspace", obj([("inlayHint", refresh)]))]);
        server.handle(&request(1, "initialize", obj([("capabilities", caps)])));
        let doc = obj([
            ("uri", self.uri.as_str().into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", "let x = 3\nprint(x)\n".into()),
        ]);
        server.handle(&notification(
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        server
    }

    fn exec(&self, server: &mut Server, args: &[&str]) -> Vec<Value> {
        let mut arguments: Vec<Value> = args.iter().map(|a| Value::from(*a)).collect();
        arguments.insert(1, self.path.as_str().into());
        let params = obj([
            ("command", "cpl.exec".into()),
            ("arguments", arguments.into()),
        ]);
        server.handle(&request(7, "workspace/executeCommand", params))
    }

    fn hints(&self, server: &mut Server) -> Vec<(i64, i64, String)> {
        let params = obj([("textDocument", obj([("uri", self.uri.as_str().into())]))]);
        let reply = server.handle(&request(8, "textDocument/inlayHint", params));
        (reply[0].at(&["result"]).as_array().unwrap().iter())
            .filter(|h| h.at(&["paddingLeft"]) == &Value::Bool(true))
            .map(|h| {
                let at = |k| h.at(&["position", k]).as_i64().unwrap();
                (
                    at("line"),
                    at("character"),
                    h.at(&["label"]).as_str().unwrap().to_string(),
                )
            })
            .collect()
    }
}

fn of<'a>(replies: &'a [Value], method: &str) -> Vec<&'a Value> {
    replies
        .iter()
        .filter(|r| r.at(&["method"]).as_str() == Some(method))
        .collect()
}

fn report() -> String {
    let status = obj([
        ("line", 1.into()),
        ("ok", false.into()),
        ("label", "0 of 1 samples passed".into()),
        ("detail", "want 4".into()),
    ]);
    let line = |kind: &str, label: &str| {
        obj([
            ("line", 0.into()),
            ("kind", kind.into()),
            ("label", label.into()),
        ])
    };
    let diagnostic = obj([
        ("line", 1.into()),
        ("col", 0.into()),
        ("code", "R0301".into()),
        ("message", "index out of range".into()),
        ("help", "check i".into()),
        ("source", "cpl (i1)".into()),
    ]);
    let view = obj([
        ("panel", "i1: index out of range\n".into()),
        ("status", status),
        (
            "lines",
            vec![line("live", "i1: x = 3"), line("error", "boom")].into(),
        ),
        ("diagnostic", diagnostic),
    ]);
    obj([("view", view)]).to_string()
}

#[test]
fn a_command_runs_cpl_and_shows_the_view_until_an_edit() {
    let stub = Stub::new("view");
    stub.prints(&report());
    let mut server = stub.server();
    let replies = stub.exec(&mut server, &["run", "--samples"]);
    assert_eq!(
        stub.argv(),
        format!("run {} --samples --format json ", stub.path)
    );
    let answer = replies
        .iter()
        .find(|r| r.at(&["id"]) == &Value::Int(7))
        .unwrap();
    assert_eq!(
        answer.at(&["result", "view", "status", "label"]).as_str(),
        Some("0 of 1 samples passed")
    );
    let logged = of(&replies, "window/logMessage");
    assert_eq!(
        logged[0].at(&["params", "message"]).as_str(),
        Some("i1: index out of range")
    );
    let shown = of(&replies, "window/showMessage");
    assert_eq!(shown[0].at(&["params", "type"]), &Value::Int(2));
    let published = of(&replies, "textDocument/publishDiagnostics");
    let run = |p: &Value| {
        (p.at(&["params", "diagnostics"]).as_array().unwrap().iter())
            .find(|d| d.at(&["code"]).as_str() == Some("R0301"))
            .map(|d| d.at(&["message"]).as_str().unwrap().to_string())
    };
    assert_eq!(
        run(published[0]).as_deref(),
        Some("index out of range (check i)")
    );
    assert_eq!(of(&replies, "workspace/inlayHint/refresh").len(), 1);
    // The error outranks the live value on line 0; the status takes its own line.
    assert_eq!(
        stub.hints(&mut server),
        vec![
            (0, 9, "boom".into()),
            (1, 8, "\u{2717} 0 of 1 samples passed".into())
        ]
    );
    let change = obj([
        (
            "textDocument",
            obj([("uri", stub.uri.as_str().into()), ("version", 2.into())]),
        ),
        (
            "contentChanges",
            vec![obj([("text", "let x = 4\nprint(x)\n".into())])].into(),
        ),
    ]);
    let changed = server.handle(&notification("textDocument/didChange", change));
    assert!(stub.hints(&mut server).is_empty());
    assert_eq!(
        run(of(&changed, "textDocument/publishDiagnostics")[0]),
        None
    );
}

#[test]
fn the_settings_complete_the_arguments_and_text_is_the_panel() {
    let stub = Stub::new("settings");
    stub.prints("found a counterexample\n");
    let mut server = stub.server();
    let cp = obj([
        (
            "stress",
            obj([("args", vec![Value::from("--trials"), "9".into()].into())]),
        ),
        ("submit", obj([("copy", false.into())])),
        ("liveValues", obj([("input", "i2".into())])),
    ]);
    server.handle(&notification(
        "workspace/didChangeConfiguration",
        obj([("settings", obj([("cp", cp)]))]),
    ));
    let replies = stub.exec(&mut server, &["stress"]);
    assert_eq!(
        stub.argv(),
        format!("stress {} --trials 9 --format json ", stub.path)
    );
    let logged = of(&replies, "window/logMessage");
    assert_eq!(
        logged[0].at(&["params", "message"]).as_str(),
        Some("found a counterexample")
    );
    stub.exec(&mut server, &["submit"]);
    assert_eq!(stub.argv(), format!("submit {} --json ", stub.path));
    stub.exec(&mut server, &["run", "--samples"]);
    assert_eq!(
        stub.argv(),
        format!("run {} --samples --values-of i2 --format json ", stub.path)
    );
    // An object among the arguments names flags, and the standard input.
    let arguments = vec![
        Value::from("run"),
        stub.path.as_str().into(),
        "--samples".into(),
        obj([
            ("case", "i1".into()),
            ("stdin", "3\n".into()),
            ("expect", "".into()),
        ]),
    ];
    let params = obj([
        ("command", "cpl.exec".into()),
        ("arguments", arguments.into()),
    ]);
    server.handle(&request(9, "workspace/executeCommand", params));
    assert_eq!(
        stub.argv(),
        format!("run {} --samples --case i1 --format json ", stub.path)
    );
}

#[test]
fn a_label_is_clipped_to_one_line_of_the_width() {
    use crate::server::exec::clip;
    assert_eq!(clip("[1,\n  2]", 20), "[1, 2]");
    assert_eq!(clip("abcdefghij", 5), "abcd\u{2026}");
}

#[test]
fn a_trace_answers_with_its_page_for_the_editor_to_open() {
    let stub = Stub::new("trace");
    stub.prints("<!doctype html><p>steps</p>\n");
    let mut server = stub.server();
    let replies = stub.exec(&mut server, &["run", "i1", "--trace", "f"]);
    assert_eq!(
        stub.argv(),
        format!("run {} i1 --trace f --format html ", stub.path)
    );
    let answer = replies
        .iter()
        .find(|r| r.at(&["id"]) == &Value::Int(7))
        .unwrap();
    assert!(answer
        .at(&["result", "html"])
        .as_str()
        .unwrap()
        .starts_with("<!doctype"));
    assert!(of(&replies, "window/logMessage").is_empty());
    assert_eq!(of(&replies, "cpl/page").len(), 1);
}

#[test]
fn live_values_run_quietly_on_each_save() {
    let stub = Stub::new("live");
    stub.prints(&report());
    let mut server = stub.server();
    let cp = obj([("liveValues", obj([("enabled", true.into())]))]);
    let on = server.handle(&notification(
        "workspace/didChangeConfiguration",
        obj([("settings", obj([("cp", cp)]))]),
    ));
    assert_eq!(
        stub.argv(),
        format!("run {} --samples --format json ", stub.path)
    );
    assert!(of(&on, "window/showMessage").is_empty() && of(&on, "window/logMessage").is_empty());
    assert_eq!(stub.hints(&mut server).len(), 2);
    std::fs::remove_file(stub.dir.join("argv")).unwrap();
    let doc = obj([("textDocument", obj([("uri", stub.uri.as_str().into())]))]);
    server.handle(&notification("textDocument/didSave", doc));
    assert!(stub.dir.join("argv").is_file());
    let off = obj([("liveValues", obj([("enabled", false.into())]))]);
    server.handle(&notification(
        "workspace/didChangeConfiguration",
        obj([("settings", obj([("cp", off)]))]),
    ));
    assert!(stub.hints(&mut server).is_empty());
}

#[test]
fn a_run_on_its_own_thread_is_answered_when_it_ends() {
    let stub = Stub::new("thread");
    stub.prints(&report());
    let mut server = stub.server();
    let (tx, rx) = std::sync::mpsc::channel();
    let tx = std::sync::Mutex::new(tx);
    server.on_finish(std::sync::Arc::new(move |run| {
        let _ = tx.lock().unwrap().send(run);
    }));
    let replies = stub.exec(&mut server, &["run", "--samples"]);
    assert!(replies.iter().all(|r| r.at(&["id"]) != &Value::Int(7)));
    let later = server.finish(rx.recv().unwrap());
    assert!(later.iter().any(|r| r.at(&["id"]) == &Value::Int(7)));
}

#[test]
fn a_typed_case_takes_the_first_input_with_text_and_a_cancelled_one_runs_nothing() {
    let stub = Stub::new("typed");
    stub.reads_stdin();
    stub.prints("added 1\n");
    let mut server = stub.server();
    let given = obj([
        ("stdin", vec![Value::from(""), "4".into()].into()),
        ("typed", vec![Value::from("8\\n9")].into()),
    ]);
    let replies = stub.exec_with(&mut server, &["test", "--add", "-"], given);
    let answer = replies
        .iter()
        .find(|r| r.at(&["id"]) == &Value::Int(7))
        .unwrap();
    assert_eq!(answer.at(&["result", "ok"]), &Value::Bool(true));
    let stdin = std::fs::read_to_string(stub.dir.join("stdin")).unwrap();
    assert!(stdin.starts_with('4'), "{stdin}");
    let argv = stub.argv();
    let expect = argv
        .split(' ')
        .skip_while(|a| *a != "--expect")
        .nth(1)
        .unwrap();
    assert_eq!(std::fs::read_to_string(expect).unwrap(), "8\n9");
    std::fs::remove_file(stub.dir.join("argv")).unwrap();
    let cancelled = obj([("typed", vec![Value::Null].into())]);
    let replies = stub.exec_with(&mut server, &["test", "--add", "-"], cancelled);
    assert_eq!(replies[0].at(&["result"]), &Value::Null);
    assert!(!stub.dir.join("argv").exists());
}

#[test]
fn a_folder_not_trusted_runs_nothing() {
    let stub = Stub::new("untrusted");
    let mut server = Server::new().with_cpl(stub.dir.join("cpl"));
    let options = obj([("trusted", false.into())]);
    server.handle(&request(
        1,
        "initialize",
        obj([("initializationOptions", options)]),
    ));
    let replies = stub.exec(&mut server, &["run"]);
    let message = replies[0].at(&["error", "message"]).as_str().unwrap();
    assert!(message.contains("Restricted Mode"), "{message}");
    assert!(!stub.dir.join("argv").exists());
}

#[test]
fn turning_hints_off_hides_a_runs_marks_and_asks_for_hints_again() {
    let stub = Stub::new("hints-off");
    stub.prints(&report());
    let mut server = stub.server();
    stub.exec(&mut server, &["run", "--samples"]);
    assert!(!stub.hints(&mut server).is_empty());
    let off = obj([("inlayHints", obj([("enabled", false.into())]))]);
    let replies = server.handle(&notification(
        "workspace/didChangeConfiguration",
        obj([("settings", obj([("cp", off)]))]),
    ));
    let asked = replies
        .iter()
        .filter(|r| r.at(&["method"]).as_str() == Some("workspace/inlayHint/refresh"));
    assert_eq!(asked.count(), 1);
    assert!(stub.hints(&mut server).is_empty());
}

#[test]
fn import_lands_in_the_workspace_folder_when_given_no_path() {
    let stub = Stub::new("import");
    let import = obj([
        ("command", "cpl.import".into()),
        ("arguments", vec![Value::Null, 0.into()].into()),
    ]);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut server = Server::new().with_outbox(tx);
    let folder = obj([(
        "uri",
        crate::uri::uri_of_path(&stub.dir.display().to_string()).into(),
    )]);
    server.handle(&request(
        1,
        "initialize",
        obj([("workspaceFolders", vec![folder].into())]),
    ));
    let reply = server.handle(&request(2, "workspace/executeCommand", import.clone()));
    assert!(reply[0].at(&["result", "port"]).as_u64().is_some());
    let said = rx.recv().unwrap();
    assert!(said
        .at(&["params", "message"])
        .as_str()
        .unwrap()
        .starts_with("Waiting"));
    let mut server = Server::new().with_outbox(std::sync::mpsc::channel().0);
    server.handle(&request(1, "initialize", obj([])));
    let reply = server.handle(&request(2, "workspace/executeCommand", import));
    let message = reply[0].at(&["error", "message"]).as_str().unwrap();
    assert!(message.contains("open a folder first"), "{message}");
}

#[test]
fn a_typed_command_runs_on_the_file_and_its_panel_can_be_a_message() {
    let stub = Stub::new("command");
    stub.prints(&report());
    let mut server = stub.server();
    let options = obj([("panel", "message".into())]);
    server.handle(&request(
        1,
        "initialize",
        obj([("initializationOptions", options)]),
    ));
    let typed = |args: &[&str]| {
        let mut arguments = vec![Value::from(stub.path.as_str())];
        arguments.extend(args.iter().map(|a| Value::from(*a)));
        let params = obj([
            ("command", "cpl.command".into()),
            ("arguments", arguments.into()),
        ]);
        request(7, "workspace/executeCommand", params)
    };
    let replies = server.handle(&typed(&["stress", "--seed", "4"]));
    assert!(
        stub.argv()
            .starts_with(&format!("stress {} --seed 4 ", stub.path)),
        "{}",
        stub.argv()
    );
    assert!(of(&replies, "window/logMessage").is_empty());
    let shown = of(&replies, "window/showMessage");
    assert_eq!(
        shown[0].at(&["params", "message"]).as_str(),
        Some("i1: index out of range")
    );
    // Nothing typed is `cpl test` on the file.
    server.handle(&typed(&[]));
    assert!(stub.argv().starts_with(&format!("test {} ", stub.path)));
}
