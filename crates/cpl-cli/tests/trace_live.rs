//! `cpl run --trace --live` over HTTP: the pinned input, a save that records
//! again, and a broken save that keeps the last good trace. Also `cpl run
//! --trace` typed at a terminal, which records the pinned case.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SOURCE: &str =
    "let n: 1..=10 = read()\nvar total = 0\nfor i in 1..=n:\n    total += i\nprint(total)\n";

fn get(base: &str, path: &str) -> String {
    let address = base.trim_start_matches("http://").trim_end_matches('/');
    let mut stream = TcpStream::connect(address).unwrap();
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response.split_once("\r\n\r\n").unwrap().1.to_string()
}

/// Starts `cpl run --trace --live` with `args` and waits for its serving
/// line, the last it prints before taking requests. Returns the server, its
/// address and what it printed before. Standard error is closed after: the
/// server must keep serving and logging without anyone reading it.
#[allow(clippy::zombie_processes, reason = "the caller kills and waits for it")]
fn serve(args: &[&std::ffi::OsStr]) -> (std::process::Child, String, String) {
    let mut server = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", "--trace", "--live"])
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stderr = BufReader::new(server.stderr.take().unwrap());
    let mut before = String::new();
    loop {
        let mut line = String::new();
        if stderr.read_line(&mut line).unwrap_or(0) == 0 {
            let ended = server.wait().unwrap();
            panic!("no serving line ({ended}): {before}");
        }
        if line.contains("serving the trace") {
            let base = line.split_whitespace().last().unwrap().to_string();
            return (server, base, before);
        }
        before.push_str(&line);
    }
}

/// `/live` once it shows `wanted`: each request waits for the next save.
fn until(base: &str, wanted: &str) -> String {
    let start = Instant::now();
    let mut changes = 0;
    loop {
        let live = get(base, &format!("/live?after={changes}"));
        if live.contains(wanted) || start.elapsed() > Duration::from_secs(10) {
            return live;
        }
        changes = live
            .split("\"changes\":")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
            .and_then(|n| n.parse().ok())
            .unwrap_or(changes);
    }
}

#[test]
fn live_trace_records_each_save_on_the_pinned_input() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace_live_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("live.tests")).unwrap();
    let file = dir.join("live.cp");
    std::fs::write(&file, SOURCE).unwrap();
    std::fs::write(dir.join("live.tests/stress-1.in"), "4\n").unwrap();
    let (mut server, base, input) = serve(&[file.as_os_str()]);
    assert!(input.contains("stress-1.in"), "{input}");

    assert_eq!(
        get(&base, "/live"),
        r#"{"version":1,"changes":0,"error":null}"#
    );
    let page = get(&base, "/");
    assert!(page.contains(r#"<meta name="cpl-live" content="1">"#));
    assert!(get(&base, "/trace.json").contains("total += i"));
    let window = get(&base, "/trace.json?from=3");
    assert!(window.contains(r#""firstStep":3,"steps""#), "{window}");

    std::fs::write(&file, SOURCE.replace("total += i", "total += 2 * i")).unwrap();
    assert_eq!(
        until(&base, r#""version":2"#),
        r#"{"version":2,"changes":1,"error":null}"#
    );
    assert!(get(&base, "/trace.json").contains("total += 2 * i"));

    std::fs::write(&file, format!("{SOURCE}fn (\n")).unwrap();
    let broken = until(&base, "E0101");
    assert!(
        broken.starts_with(r#"{"version":2,"changes":2,"error":"error[E0101]"#),
        "{broken}"
    );
    assert!(get(&base, "/trace.json").contains("total += 2 * i"));
    server.kill().unwrap();
    server.wait().unwrap();
}

/// A save made as soon as the serving line is out is a save after the
/// recording the server shows, so it records again.
#[test]
fn a_save_right_after_the_serving_line_records_again() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace_live_early_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("live.tests")).unwrap();
    let file = dir.join("live.cp");
    std::fs::write(&file, SOURCE).unwrap();
    std::fs::write(dir.join("live.tests/stress-1.in"), "4\n").unwrap();
    let (mut server, base, _) = serve(&[file.as_os_str()]);
    std::fs::write(&file, SOURCE.replace("total += i", "total += 2 * i")).unwrap();
    let live = until(&base, r#""version":2"#);
    let trace = get(&base, "/trace.json");
    server.kill().unwrap();
    server.wait().unwrap();
    assert_eq!(live, r#"{"version":2,"changes":1,"error":null}"#);
    assert!(trace.contains("total += 2 * i"));
}

#[cfg(target_os = "linux")]
#[test]
fn trace_at_a_terminal_records_the_pinned_case() {
    if Command::new("script").arg("--version").output().is_err() {
        return;
    }
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace_tty_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("live.tests")).unwrap();
    std::fs::write(dir.join("live.cp"), SOURCE).unwrap();
    std::fs::write(dir.join("live.tests/stress-1.in"), "4\n").unwrap();
    let command = format!("{} run live.cp --trace", env!("CARGO_BIN_EXE_cpl"));
    let out = Command::new("script")
        .args(["-qec", &command, "/dev/null"])
        .current_dir(&dir)
        .output()
        .unwrap();
    let shown = String::from_utf8_lossy(&out.stdout);
    assert!(shown.contains("10"), "ran on the pinned 4: {shown}");
    assert!(
        shown.contains("input live.tests/stress-1.in"),
        "says which case: {shown}"
    );
    let piped = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", "live.cp", "--trace"])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    piped.stdin.as_ref().unwrap().write_all(b"3\n").unwrap();
    let piped = piped.wait_with_output().unwrap();
    assert_eq!(
        String::from_utf8_lossy(&piped.stdout).trim(),
        "6",
        "piped input wins over the pinned case"
    );
}

#[test]
fn a_live_trace_of_an_interactive_file_runs_it_against_its_judge() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("trace_live_interactive_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("guess.cp");
    std::fs::write(
        &file,
        "let n: 1..=100 = read()\nvar lo = 1\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    let r: Int = ask(\"?\", mid, limit: 7)\n    if r == 1: hi = mid\n    else: lo = mid + 1\nprint(\"!\", lo)\n\n\
         @interactor fn judge(sol: Interactor):\n    let n: 1..=100 = read()\n    let x: 1..=n = read()\n    sol.send(n)\n    while true:\n        let (c, v): (Char, Int) = sol.recv()\n        if c == '!':\n            assert v == x, \"wrong answer\"\n            return\n        sol.send(if x <= v: 1 else: 0)\n",
    )
    .unwrap();
    let input = dir.join("guess.in");
    std::fs::write(&input, "100 37\n").unwrap();
    let steps = |doc: &str| {
        doc.split_once("\"steps\":")
            .unwrap()
            .1
            .trim_end()
            .to_string()
    };
    let traced = |flags: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(["run", "--trace", "--format", "json"])
            .args(flags)
            .arg(&file)
            .arg(&input)
            .output()
            .unwrap();
        steps(&String::from_utf8_lossy(&out.stdout))
    };
    let (mut server, base, _) = serve(&[file.as_os_str(), input.as_os_str()]);
    let whole = get(&base, "/trace.json");
    let window = get(&base, "/trace.json?from=6");
    server.kill().unwrap();
    server.wait().unwrap();
    assert!(whole.contains("the program ended"), "{whole}");
    assert_eq!(steps(&whole), traced(&[]));
    assert_eq!(steps(&window), traced(&["--from", "6"]));
}
