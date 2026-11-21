//! End-to-end stdio Debug Adapter Protocol (`cpl dap`) tests:
//! breakpoints, call stacks, domain-labeled variables, stepping, backward
//! time-travel (`stepBack`, `reverseContinue`), hover evaluation, and traps.

use std::io::{BufRead, BufReader, Read, Write};
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
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body).unwrap();
        self.stdin.flush().unwrap();
        seq
    }

    fn recv(&mut self) -> String {
        let mut len: Option<usize> = None;
        loop {
            let mut line = String::new();
            let n = self.stdout.read_line(&mut line).unwrap();
            assert!(n > 0, "unexpected EOF from cpl dap");
            let t = line.trim();
            if t.is_empty() {
                break;
            }
            if let Some((k, v)) = t.split_once(':') {
                if k.eq_ignore_ascii_case("Content-Length") {
                    len = v.trim().parse().ok();
                }
            }
        }
        let len = len.expect("missing Content-Length header");
        let mut buf = vec![0u8; len];
        self.stdout.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
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
var last_v: V = V.first
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
    assert!(init.contains("\"supportsStepBack\":true"), "missing stepBack cap: {init}");
    let _ = dap.recv_until("\"event\":\"initialized\"");

    dap.send(
        "launch",
        Some(&format!("{{\"program\":\"{p}\",\"stopOnEntry\":false}}")),
    );
    let launch = dap.recv_until("\"command\":\"launch\"");
    assert!(launch.contains("\"success\":true"), "launch failed: {launch}");

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
    assert!(stopped.contains("\"reason\":\"breakpoint\""), "expected breakpoint stop: {stopped}");

    // Stack trace should show `add_vertex` at line 6 called from `<main>` at line 8.
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let stack = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(stack.contains("\"name\":\"add_vertex\""), "missing add_vertex frame: {stack}");
    assert!(stack.contains("\"line\":6"), "expected line 6: {stack}");
    assert!(stack.contains("\"name\":\"main\""), "missing main caller: {stack}");

    // Globals scope: `acc` should be 1 and `last_v` should be 1 (1-based domain label).
    dap.send("variables", Some("{\"variablesReference\":2}"));
    let vars = dap.recv_until("\"command\":\"variables\"");
    assert!(vars.contains("\"name\":\"acc\",\"value\":\"1\""), "expected acc=1: {vars}");
    assert!(vars.contains("\"name\":\"last_v\",\"value\":\"1\""), "expected last_v=1: {vars}");

    // Continue to next hit of line 6 (second vertex, v = 2, acc = 3).
    dap.send("continue", Some("{\"threadId\":1}"));
    let stopped2 = dap.recv_until("\"event\":\"stopped\"");
    assert!(stopped2.contains("\"reason\":\"breakpoint\""), "expected second bp: {stopped2}");

    dap.send("evaluate", Some("{\"expression\":\"acc\"}"));
    let eval_acc = dap.recv_until("\"command\":\"evaluate\"");
    assert!(eval_acc.contains("\"result\":\"3\""), "expected acc=3 at v=2: {eval_acc}");

    dap.send("evaluate", Some("{\"expression\":\"last_v\"}"));
    let eval_v = dap.recv_until("\"command\":\"evaluate\"");
    assert!(eval_v.contains("\"result\":\"2\""), "expected 1-based last_v=2: {eval_v}");

    // Time-travel backward with `reverseContinue`: should return to first hit of line 6 (acc = 1)!
    dap.send("reverseContinue", Some("{\"threadId\":1}"));
    let rev = dap.recv_until("\"event\":\"stopped\"");
    assert!(rev.contains("\"reason\":\"breakpoint\""), "expected reverse bp: {rev}");

    dap.send("evaluate", Some("{\"expression\":\"acc\"}"));
    let eval_back = dap.recv_until("\"command\":\"evaluate\"");
    assert!(eval_back.contains("\"result\":\"1\""), "time travel should restore acc=1: {eval_back}");

    // Single `stepBack` goes to line 5 (`acc += v.ord + 1`).
    dap.send("stepBack", Some("{\"threadId\":1}"));
    let _ = dap.recv_until("\"event\":\"stopped\"");
    dap.send("stackTrace", Some("{\"threadId\":1}"));
    let stack_back = dap.recv_until("\"command\":\"stackTrace\"");
    assert!(stack_back.contains("\"line\":5"), "stepBack should land on line 5: {stack_back}");

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
    assert!(eval_d.contains("\"result\":\"0\""), "expected d=0 before crash: {eval_d}");

    dap.finish();
}
