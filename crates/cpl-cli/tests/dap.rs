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
    assert!(
        vars.contains("\"name\":\"acc\",\"value\":\"1\""),
        "expected acc=1: {vars}"
    );
    assert!(
        vars.contains("\"name\":\"last_v\",\"value\":\"1\""),
        "expected last_v=1: {vars}"
    );

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
