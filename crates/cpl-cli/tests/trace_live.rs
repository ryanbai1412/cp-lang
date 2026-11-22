//! `cpl run --trace --live` over HTTP: the pinned input, a save that records
//! again, and a broken save that keeps the last good trace.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SOURCE: &str = "let n: 1..=10 = read()\nvar total = 0\nfor i in 1..=n:\n    total += i\nprint(total)\n";

fn get(base: &str, path: &str) -> String {
    let address = base.trim_start_matches("http://").trim_end_matches('/');
    let mut stream = TcpStream::connect(address).unwrap();
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response.split_once("\r\n\r\n").unwrap().1.to_string()
}

fn until(base: &str, wanted: &str) -> String {
    let start = Instant::now();
    loop {
        let live = get(base, "/live");
        if live.contains(wanted) || start.elapsed() > Duration::from_secs(10) {
            return live;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn live_trace_records_each_save_on_the_pinned_input() {
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("trace_live_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("live.tests")).unwrap();
    let file = dir.join("live.cp");
    std::fs::write(&file, SOURCE).unwrap();
    std::fs::write(dir.join("live.tests/stress-1.in"), "4\n").unwrap();
    let mut server = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", "--trace", "--live"])
        .arg(&file)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stderr = BufReader::new(server.stderr.take().unwrap());
    let (mut serving, mut input) = (String::new(), String::new());
    stderr.read_line(&mut serving).unwrap();
    stderr.read_line(&mut input).unwrap();
    let base = serving.split_whitespace().last().unwrap().to_string();
    assert!(input.contains("stress-1.in"), "{input}");

    assert_eq!(get(&base, "/live"), r#"{"version":1,"error":null}"#);
    let page = get(&base, "/");
    assert!(page.contains(r#"<meta name="cpl-live" content="1">"#));
    assert!(get(&base, "/trace.json").contains("total += i"));

    std::fs::write(&file, SOURCE.replace("total += i", "total += 2 * i")).unwrap();
    assert_eq!(until(&base, r#""version":2"#), r#"{"version":2,"error":null}"#);
    assert!(get(&base, "/trace.json").contains("total += 2 * i"));

    std::fs::write(&file, format!("{SOURCE}fn (\n")).unwrap();
    let broken = until(&base, "E0101");
    assert!(broken.starts_with(r#"{"version":2,"error":"error[E0101]"#), "{broken}");
    assert!(get(&base, "/trace.json").contains("total += 2 * i"));
    server.kill().unwrap();
    server.wait().unwrap();
}
