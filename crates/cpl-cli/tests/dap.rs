//! End-to-end stdio Debug Adapter Protocol (`cpl dap`) tests:
//! breakpoints, call stacks, domain-labeled variables, stepping, backward
//! time-travel (`stepBack`, `reverseContinue`), hover evaluation, and traps.

use std::io::BufReader;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn cpl_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cpl")
}

fn write_temp(name: &str, src: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("dap_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("{name}_{}_{n}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    path
}

struct DapClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    seq: i64,
}

impl DapClient {
    fn spawn() -> Self {
        let mut child = Command::new(cpl_bin())
            .arg("dap")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn cpl dap");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            stdin,
            stdout,
            seq: 1,
        }
    }

    fn send(&mut self, command: &str, args_json: Option<&str>) -> i64 {
        let seq = self.seq;
        self.seq += 1;
        let body = match args_json {
            Some(args) => format!(
                "{{\"seq\":{seq},\"type\":\"request\",\"command\":\"{command}\",\"arguments\":{args}}}"
            ),
            None => format!("{{\"seq\":{seq},\"type\":\"request\",\"command\":\"{command}\"}}"),
        };
        cpl_lsp::transport::write_raw_message(&mut self.stdin, &body).unwrap();
        seq
    }

    fn recv(&mut self) -> String {
        cpl_lsp::transport::read_message(&mut self.stdout)
            .unwrap()
            .expect("unexpected EOF from cpl dap")
    }

    fn recv_until(&mut self, needle: &str) -> String {
        for _ in 0..32 {
            let msg = self.recv();
            if msg.contains(needle) {
                return msg;
            }
        }
        panic!("did not receive DAP message containing `{needle}`");
    }

    fn finish(mut self) {
        self.send("disconnect", None);
        let _ = self.recv_until("\"command\":\"disconnect\"");
        let status = self.child.wait().unwrap();
        assert!(status.success(), "cpl dap exited with {status}");
    }
}

#[test]
fn every_adapter_message_numbers_one_more_and_answers_its_request() {
    let path = write_temp("numbered", "let x = 1\nprint(x)\n");
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    let init = dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let launch = dap.send("launch", Some(&format!("{{\"program\":\"{p}\"}}")));
    let mut seqs = Vec::new();
    let mut answered = Vec::new();
    while answered.len() < 2 {
        let msg = cpl_json::parse(&dap.recv()).unwrap();
        seqs.push(msg.get("seq").and_then(cpl_json::Value::as_i64).unwrap());
        if let Some(request) = msg.get("request_seq").and_then(cpl_json::Value::as_i64) {
            answered.push(request);
        }
    }
    dap.finish();
    assert_eq!(seqs, (1..=seqs.len() as i64).collect::<Vec<_>>());
    assert_eq!(answered, [init, launch]);
}

#[test]
fn dap_breakpoints_stack_variables_and_time_travel() {
    let path = write_temp(
        "time_travel",
        "\
domain V = 1..=3
var acc: Int = 0
var last_v: V = V.first!
fn add_vertex(v: V):
    acc += v.ord + 1
    last_v = v
for v in V:
    add_vertex(v)
print(acc)
",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();

    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let init = dap.recv_until("\"command\":\"initialize\"");
    assert!(
        init.contains("\"supportsStepBack\":true"),
        "missing stepBack cap: {init}"
    );
    let _ = dap.recv_until("\"event\":\"initialized\"");

    dap.send(
        "launch",
        Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":false}}")),
    );
    let launch = dap.recv_until("\"command\":\"launch\"");
    assert!(
        launch.contains("\"success\":true"),
        "launch failed: {launch}"
    );

    // Breakpoint on line 6 (`last_v = v` inside `add_vertex`).
    dap.send(
        "setBreakpoints",
        Some(&format!(
            "{{\"source\":{{\"path\":\"{p}\"}},\"breakpoints\":[{{\"line\":6}}]}}"
        )),
    );
    let bp = dap.recv_until("\"command\":\"setBreakpoints\"");
    assert!(bp.contains("\"verified\":true"), "bp not verified: {bp}");

    dap.send("configurationDone", None);
    let stopped = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        stopped.contains("\"reason\":\"breakpoint\""),
        "expected breakpoint stop: {stopped}"
    );

    // Stack trace should show `add_vertex` at line 6 called from `<main>` at line 8.
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let stack = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(
        stack.contains("\"name\":\"add_vertex\""),
        "missing add_vertex frame: {stack}"
    );
    assert!(stack.contains("\"line\":6"), "expected line 6: {stack}");
    assert!(
        stack.contains("\"name\":\"main\""),
        "missing main caller: {stack}"
    );

    // Globals scope: `acc` should be 1 and `last_v` should be 1 (1-based domain label).
    dap.send("variables", Some("{\"variablesReference\":2}"));
    let vars = dap.recv_until("\"command\":\"variables\"");
    assert_eq!(variable(&vars, "acc").as_deref(), Some("1"), "{vars}");
    assert_eq!(variable(&vars, "last_v").as_deref(), Some("1"), "{vars}");

    // Continue to next hit of line 6 (second vertex, v = 2, acc = 3).
    dap.send("continue", Some("{\"threadId\":1}"));
    let stopped2 = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        stopped2.contains("\"reason\":\"breakpoint\""),
        "expected second bp: {stopped2}"
    );

    dap.send("evaluate", Some("{\"expression\":\"acc\"}"));
    let eval_acc = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        eval_acc.contains("\"result\":\"3\""),
        "expected acc=3 at v=2: {eval_acc}"
    );

    dap.send("evaluate", Some("{\"expression\":\"last_v\"}"));
    let eval_v = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        eval_v.contains("\"result\":\"2\""),
        "expected 1-based last_v=2: {eval_v}"
    );

    // Time-travel backward with `reverseContinue`: should return to first hit of line 6 (acc = 1)!
    dap.send("reverseContinue", Some("{\"threadId\":1}"));
    let rev = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        rev.contains("\"reason\":\"breakpoint\""),
        "expected reverse bp: {rev}"
    );

    dap.send("evaluate", Some("{\"expression\":\"acc\"}"));
    let eval_back = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        eval_back.contains("\"result\":\"1\""),
        "time travel should restore acc=1: {eval_back}"
    );

    // Single `stepBack` goes to line 5 (`acc += v.ord + 1`).
    dap.send("stepBack", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let stack_back = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(
        stack_back.contains("\"line\":5"),
        "stepBack should land on line 5: {stack_back}"
    );

    dap.finish();
}

#[test]
fn dap_stops_on_runtime_trap_and_steps_back() {
    let path = write_temp(
        "trap_dap",
        "\
var x: Int = 10
var d: Int = 2
x = x / d
d = 0
x = x / d
print(x)
",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();

    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");

    dap.send(
        "launch",
        Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":false}}")),
    );
    let _ = dap.recv_until("\"command\":\"launch\"");

    dap.send("configurationDone", None);
    let stopped = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        stopped.contains("\"reason\":\"exception\""),
        "expected exception stop on division by zero: {stopped}"
    );
    assert!(
        stopped.contains("division by zero"),
        "expected concrete trap message: {stopped}"
    );

    // Step back from the crash to inspect `d == 0` right before the division.
    dap.send("stepBack", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("evaluate", Some("{\"expression\":\"d\"}"));
    let eval_d = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        eval_d.contains("\"result\":\"0\""),
        "expected d=0 before crash: {eval_d}"
    );

    dap.finish();
}

#[test]
fn dap_restart_rerecords_the_edit_on_the_pinned_input_and_keeps_place() {
    let path = write_temp(
        "restart",
        "\
let n: 1..=9 = read()
var total = 0
for i in 0..n:
    total += i + 1
print(total)
",
    );
    let tests = path.with_extension("tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("stress-1.in"), "4\n").unwrap();
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");

    // No input named: the case `cpl stress` saved is the input, said once.
    dap.send(
        "launch",
        Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":false}}")),
    );
    let note = dap.recv_until("\"category\":\"console\"");
    assert!(
        note.contains("stress-1.in"),
        "the pinned input is not named: {note}"
    );
    let out = dap.recv_until("\"category\":\"stdout\"");
    assert!(
        out.contains("\"output\":\"10\\n\""),
        "not run on the pinned input: {out}"
    );
    let launch = dap.recv_until("\"command\":\"launch\"");
    assert!(
        launch.contains("\"success\":true"),
        "launch failed: {launch}"
    );
    dap.send(
        "setBreakpoints",
        Some(&format!(
            "{{\"source\":{{\"path\":\"{p}\"}},\"breakpoints\":[{{\"line\":4}}]}}"
        )),
    );
    let _ = dap.recv_until("\"command\":\"setBreakpoints\"");
    dap.send("configurationDone", None);
    let _ = dap.recv_until("\"reason\":\"breakpoint\"");
    for _ in 0..2 {
        dap.send("continue", Some("{\"threadId\":1}"));
        let _ = dap.recv_until("\"reason\":\"breakpoint\"");
    }
    dap.send("evaluate", Some("{\"expression\":\"total\"}"));
    let total = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        total.contains("\"result\":\"6\""),
        "third visit should hold 6: {total}"
    );

    // The edit changes the second visit on: the restart says where, keeps
    // the third visit, and reverse-continue stops at the first change.
    std::fs::write(
        &path,
        "\
let n: 1..=9 = read()
var total = 0
for i in 0..n:
    total += 2 * i + 1
print(total)
",
    )
    .unwrap();
    dap.send("restart", Some("{\"arguments\":{}}"));
    let out = dap.recv_until("\"category\":\"stdout\"");
    assert!(
        out.contains("\"output\":\"16\\n\""),
        "restart did not reread the file: {out}"
    );
    let note = dap.recv_until("re-ran");
    assert!(
        note.contains("first change at line 4"),
        "no first change: {note}"
    );
    let stop = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        stop.contains("\"reason\":\"goto\"") && stop.contains("visit 3"),
        "place not kept: {stop}"
    );
    dap.send("evaluate", Some("{\"expression\":\"total\"}"));
    let total = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        total.contains("\"result\":\"9\""),
        "third visit after the edit should hold 9: {total}"
    );
    dap.send("reverseContinue", Some("{\"threadId\":1}"));
    let stop = dap.recv_until("\"event\":\"stopped\"");
    assert!(
        stop.contains("\"reason\":\"divergence\""),
        "no divergence stop: {stop}"
    );
    dap.send("evaluate", Some("{\"expression\":\"total\"}"));
    let total = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        total.contains("\"result\":\"4\""),
        "the first change is the second visit: {total}"
    );

    // A broken edit keeps the last good recording.
    std::fs::write(&path, "let n: 1..=9 = read(\n").unwrap();
    dap.send("restart", Some("{\"arguments\":{}}"));
    let failed = dap.recv_until("\"command\":\"restart\"");
    assert!(
        failed.contains("\"success\":false") && failed.contains("kept the last good recording"),
        "{failed}"
    );
    dap.send("evaluate", Some("{\"expression\":\"total\"}"));
    let total = dap.recv_until("\"command\":\"evaluate\"");
    assert!(total.contains("\"result\":\"4\""), "{total}");
    dap.finish();
}

#[test]
fn dap_evaluates_watch_expressions_in_source_terms() {
    let path = write_temp(
        "watch_dap",
        "\
let (n, cap): (1..=6, 0..=12) = read()
domain Item = 1..=n
let items: [Item](w: 1..=6, v: 1..=9) = read()
var best: [0..=cap]Int = 0
for i in Item:
    let (w, v) = (items[i].w, items[i].v)
    for c in 0..=cap:
        if c >= w: best[c] max= best[c - w] + v
print(best[cap])
",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");
    dap.send(
        "launch",
        Some(&format!(
            "{{\"program\":\"{p}\",\"stopOnEntry\":false,\"inputText\":\"1 2\\n1 1\\n\"}}"
        )),
    );
    let launch = dap.recv_until("\"command\":\"launch\"");
    assert!(
        launch.contains("\"success\":true"),
        "launch failed: {launch}"
    );
    dap.send(
        "setBreakpoints",
        Some(&format!(
            "{{\"source\":{{\"path\":\"{p}\"}},\"breakpoints\":[{{\"line\":8}}]}}"
        )),
    );
    let _ = dap.recv_until("\"command\":\"setBreakpoints\"");
    dap.send("configurationDone", None);
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("continue", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");

    let mut eval = |expr: &str| {
        dap.send("evaluate", Some(&format!("{{\"expression\":\"{expr}\"}}")));
        dap.recv_until("\"command\":\"evaluate\"")
    };
    let item = eval("items[1]");
    assert!(
        item.contains("\"result\":\"(v: 1, w: 1)\""),
        "a record in source terms, by its domain label: {item}"
    );
    let c = eval("c");
    assert!(
        c.contains("\"result\":\"2\""),
        "second hit of line 8 is c = 2: {c}"
    );
    let best = eval("best");
    assert!(
        best.contains("\"result\":\"[0, 1, 2]\""),
        "the planted reuse of the item: {best}"
    );
    let candidate = eval("best[c - w] + v");
    assert!(
        candidate.contains("\"result\":\"2\""),
        "index arithmetic over the recorded state: {candidate}"
    );
    let test = eval("c >= w and len(best) == 3");
    assert!(
        test.contains("\"result\":\"true\""),
        "comparisons and len: {test}"
    );
    let missing = eval("items[2]");
    assert!(
        missing.contains("2 is out of range"),
        "an index outside the domain says so: {missing}"
    );
    let unknown = eval("nope + 1");
    assert!(
        unknown.contains("`nope` is not in scope here"),
        "names the unknown variable: {unknown}"
    );
    dap.finish();
}

#[test]
fn dap_replays_one_call_with_the_edited_fn() {
    let demo =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/debugging/shows.cp");
    let source = std::fs::read_to_string(demo).unwrap();
    let path = write_temp("replay_dap", &source);
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");
    let input = "2\\n1\\n0 1\\n3\\n0 2\\n0 1\\n1 1\\n";
    dap.send(
        "launch",
        Some(&format!(
            "{{\"program\":\"{p}\",\"inputText\":\"{input}\",\"stopOnEntry\":false}}"
        )),
    );
    let _ = dap.recv_until("\"command\":\"launch\"");
    let line = source
        .lines()
        .position(|l| l.starts_with("print(most_watched())"))
        .unwrap()
        + 1;
    dap.send(
        "setBreakpoints",
        Some(&format!(
            "{{\"source\":{{\"path\":\"{p}\"}},\"breakpoints\":[{{\"line\":{line}}}]}}"
        )),
    );
    let _ = dap.recv_until("\"command\":\"setBreakpoints\"");
    dap.send("configurationDone", None);
    let _ = dap.recv_until("\"reason\":\"breakpoint\"");
    dap.send("continue", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"reason\":\"breakpoint\"");

    std::fs::write(
        &path,
        source.replace(
            "by: i => shows[i].start)",
            "by: i => shows[i].start + shows[i].len)",
        ),
    )
    .unwrap();
    for args in ["{\"fn\":\"most_watched\"}", "{\"frameId\":1}"] {
        dap.send("replayCall", Some(args));
        let replay = dap.recv_until("\"command\":\"replayCall\"");
        assert!(replay.contains("\"success\":true"), "{replay}");
        assert!(
            replay.contains("\"ordinal\":2") && replay.contains("\"call\":\"most_watched()\""),
            "the call at the cursor: {replay}"
        );
        assert!(
            replay.contains("\"returned\":\"2\"") && replay.contains("\"before\":\"1\""),
            "{replay}"
        );
        let sorted = source
            .lines()
            .position(|l| l.trim() == "var free_at = 0")
            .unwrap()
            + 1;
        assert!(
            replay.contains(&format!("\"changeLine\":{sorted}")) && !replay.contains("fallback"),
            "{replay}"
        );
        assert!(
            replay.contains("`order` = [2, 1, 3] (before the edit: [1, 2, 3])"),
            "the edit's new order: {replay}"
        );
    }

    let wider = source.replace(
        "fn most_watched() -> Int:",
        "fn most_watched(from: Int) -> Int:",
    );
    std::fs::write(
        &path,
        wider.replace("print(most_watched())", "print(most_watched(0))"),
    )
    .unwrap();
    dap.send("replayCall", Some("{\"fn\":\"most_watched\"}"));
    let replay = dap.recv_until("\"command\":\"replayCall\"");
    assert!(
        replay.contains("\"success\":true")
            && replay.contains("\"fallback\":\"the edit changes what"),
        "{replay}"
    );
    dap.send("replayCall", Some("{\"fn\":\"nope\"}"));
    let replay = dap.recv_until("\"command\":\"replayCall\"");
    assert!(
        replay.contains("\"success\":false") && replay.contains("never calls `nope`"),
        "{replay}"
    );
    dap.finish();
}

#[test]
fn dap_reaches_every_step_of_a_run_longer_than_a_recording() {
    // A recording that keeps 10^5 steps, so a run of 2*10^5 iterations is
    // longer than one recording, and still far below the run's 10 s limit
    // when the machine is loaded.
    let path = write_temp(
        "long_dap",
        "\
let n: Int = read()
var i = 0
while i < n:
    i += 1
let d = i - i
print(i / d)
",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");
    dap.send(
        "launch",
        Some(&format!(
            "{{\"program\":\"{p}\",\"stopOnEntry\":false,\"recordSteps\":100000,\"inputText\":\"200000\\n\"}}"
        )),
    );
    let _ = dap.recv_until("\"command\":\"launch\"");
    dap.send("configurationDone", None);
    let stopped = dap.recv_until("\"event\":\"stopped\"");
    assert!(stopped.contains("division by zero"), "{stopped}");
    dap.send("evaluate", Some("{\"expression\":\"i\"}"));
    let i = dap.recv_until("\"command\":\"evaluate\"");
    assert!(
        i.contains("\"result\":\"200000\""),
        "stopped at the trap: {i}"
    );
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let frame = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(frame.contains("\"line\":6"), "{frame}");
    // Recorded again unchanged, it keeps the place past the first recording.
    dap.send("restart", Some("{\"arguments\":{}}"));
    let note = dap.recv_until("re-ran");
    assert!(
        note.contains("of the first 10000 steps is the same"),
        "{note}"
    );
    let kept = dap.recv_until("\"event\":\"stopped\"");
    assert!(kept.contains("kept your place: line 6"), "{kept}");
    dap.send("evaluate", Some("{\"expression\":\"i\"}"));
    let i = dap.recv_until("\"command\":\"evaluate\"");
    assert!(i.contains("\"result\":\"200000\""), "{i}");
    // Back through every window to the first step.
    dap.send("reverseContinue", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let frame = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(frame.contains("\"line\":1,"), "{frame}");
    dap.send("next", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("evaluate", Some("{\"expression\":\"n\"}"));
    let n = dap.recv_until("\"command\":\"evaluate\"");
    assert!(n.contains("\"result\":\"200000\""), "{n}");
    dap.finish();
}

#[test]
fn a_message_without_a_content_length_ends_the_session_with_exit_1() {
    let mut client = DapClient::spawn();
    use std::io::Write;
    client
        .stdin
        .write_all(b"Content-Type: x\r\n\r\n{}")
        .unwrap();
    drop(client.stdin);
    let out = client.child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("cpl dap: missing or oversized Content-Length"),
        "{stderr}"
    );
}

/// Random DAP requests against a straight run: the run stepped in from
/// its first step to its last is the oracle, and every `next`, `stepOut`,
/// `stepBack`, `continue` and `reverseContinue` must stop at the step the
/// protocol names, with that step's stack and values.
#[test]
fn threads_scopes_and_next_over_a_call_answer_as_the_protocol_says() {
    let path = write_temp(
        "next_over",
        "fn sq(x: Int) -> Int:\n    let y = x * x\n    y\nvar t = 3\nvar a = sq(t)\nvar b = a + 1\nprint(b)\n",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let mut dap = DapClient::spawn();
    dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
    let _ = dap.recv_until("\"event\":\"initialized\"");
    dap.send(
        "launch",
        Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":true}}")),
    );
    let _ = dap.recv_until("\"command\":\"launch\"");
    dap.send("configurationDone", None);
    let _ = dap.recv_until("\"event\":\"stopped\"");
    let body = |dap: &mut DapClient, command: &str, args: Option<&str>| {
        dap.send(command, args);
        let reply = dap.recv_until(&format!("\"command\":\"{command}\""));
        cpl_json::parse(&reply).unwrap().at(&["body"]).clone()
    };
    let threads = body(&mut dap, "threads", None);
    assert_eq!(
        threads.to_string(),
        r#"{"threads":[{"id":1,"name":"main"}]}"#
    );
    let scopes = body(&mut dap, "scopes", Some("{\"frameId\":1}"));
    let scopes: Vec<(String, i64)> = scopes
        .at(&["scopes"])
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s.at(&["name"]).as_str().unwrap().to_string(),
                s.at(&["variablesReference"]).as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        scopes,
        [("Locals".to_string(), 1), ("Globals".to_string(), 2)]
    );
    let line = |dap: &mut DapClient| {
        let stack = body(dap, "stackTrace", Some("{\"threadId\":1}"));
        let frames = stack.at(&["stackFrames"]).as_array().unwrap();
        (frames.len(), frames[0].at(&["line"]).as_i64().unwrap())
    };
    let mut stops = vec![line(&mut dap)];
    // `next` from the top level runs each call whole: no stop lands inside `sq`.
    while stops.len() < 4 {
        dap.send("next", Some("{\"threadId\":1}"));
        let _ = dap.recv_until("\"event\":\"stopped\"");
        stops.push(line(&mut dap));
    }
    assert_eq!(stops, [(1, 4), (1, 5), (1, 6), (1, 7)]);
    dap.finish();
}

#[test]
fn random_dap_requests_stop_where_a_straight_run_says() {
    let path = write_temp(
        "random_dap",
        "\
fn sq(x: Int) -> Int:
    let y = x * x
    y
var t = 0
var acc = 0
while t < 6:
    t += 1
    acc += sq(t)
print(acc)
",
    );
    let p = path.to_str().unwrap().replace('\\', "\\\\");
    let breakpoints = [2u32, 8];
    let launch = |dap: &mut DapClient, with_breakpoints: bool| {
        dap.send("initialize", Some("{\"adapterID\":\"cp\"}"));
        let _ = dap.recv_until("\"event\":\"initialized\"");
        dap.send(
            "launch",
            Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":true}}")),
        );
        let _ = dap.recv_until("\"command\":\"launch\"");
        if with_breakpoints {
            let lines = breakpoints.map(|l| format!("{{\"line\":{l}}}")).join(",");
            dap.send(
                "setBreakpoints",
                Some(&format!(
                    "{{\"source\":{{\"path\":\"{p}\"}},\"breakpoints\":[{lines}]}}"
                )),
            );
            let _ = dap.recv_until("\"command\":\"setBreakpoints\"");
        }
        dap.send("configurationDone", None);
        let _ = dap.recv_until("\"event\":\"stopped\"");
    };
    // (frames, line, what `t` and `acc` evaluate to) where the session stands.
    let state = |dap: &mut DapClient| {
        dap.send("stackTrace", Some("{\"threadId\":1}"));
        let stack = dap.recv_until("\"command\":\"stackTrace\"");
        let after = |key: &str| {
            let at = stack
                .find(key)
                .unwrap_or_else(|| panic!("{key} in {stack}"))
                + key.len();
            let digits: String = stack[at..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse::<u32>().unwrap()
        };
        let (frames, line) = (after("\"totalFrames\":"), after("\"line\":"));
        let mut values = Vec::new();
        for name in ["t", "acc"] {
            dap.send("evaluate", Some(&format!("{{\"expression\":\"{name}\"}}")));
            let reply = dap.recv_until("\"command\":\"evaluate\"");
            let body = cpl_json::parse(&reply).map(|reply| reply.at(&["body"]).to_string());
            values.push(body.unwrap_or(reply));
        }
        (frames, line, values)
    };
    // The straight run.
    let mut dap = DapClient::spawn();
    launch(&mut dap, false);
    let mut straight = vec![state(&mut dap)];
    loop {
        dap.send("stepIn", Some("{\"threadId\":1}"));
        let stop = loop {
            let msg = dap.recv();
            if msg.contains("\"event\":\"stopped\"") || msg.contains("\"event\":\"terminated\"") {
                break msg;
            }
        };
        if stop.contains("\"event\":\"terminated\"") {
            break;
        }
        straight.push(state(&mut dap));
    }
    dap.finish();
    assert!(straight.len() > 20, "{} steps", straight.len());
    let last = straight.len() - 1;
    let at_break = |i: usize| breakpoints.contains(&straight[i].1);
    // Where each request stops by the protocol, or None past the run's end.
    let target = |op: &str, k: usize| -> Option<usize> {
        let (depth, line) = (straight[k].0, straight[k].1);
        let mut later = k + 1..=last;
        match op {
            "stepIn" => (k < last).then_some(k + 1),
            "stepBack" => Some(k.saturating_sub(1)),
            "next" => later.find(|&i| {
                at_break(i) || straight[i].0 <= depth && (straight[i].1 != line || i == last)
            }),
            "stepOut" => later.find(|&i| at_break(i) || straight[i].0 < depth),
            "continue" => later.find(|&i| at_break(i)),
            "reverseContinue" => Some((0..k).rev().find(|&i| at_break(i)).unwrap_or(0)),
            _ => unreachable!(),
        }
    };
    let ops = [
        "stepIn",
        "stepBack",
        "next",
        "stepOut",
        "continue",
        "reverseContinue",
    ];
    for seed in 1..=4u64 {
        let mut dap = DapClient::spawn();
        launch(&mut dap, true);
        let (mut k, mut rng, mut done) = (0, seed, Vec::new());
        for _ in 0..40 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let op = ops[(rng % ops.len() as u64) as usize];
            let Some(next) = target(op, k) else { continue };
            dap.send(op, Some("{\"threadId\":1}"));
            let _ = dap.recv_until("\"event\":\"stopped\"");
            done.push(op);
            assert_eq!(
                state(&mut dap),
                straight[next],
                "seed {seed}: after {done:?} the session should stand at step {next}"
            );
            k = next;
        }
        dap.finish();
    }
}

/// The value a `variables` response shows for `name`.
fn variable(response: &str, name: &str) -> Option<String> {
    let response = cpl_json::parse(response).ok()?;
    let found = response
        .at(&["body", "variables"])
        .as_array()?
        .iter()
        .find(|v| v.get("name").and_then(cpl_json::Value::as_str) == Some(name))?;
    found.get("value")?.as_str().map(str::to_string)
}
