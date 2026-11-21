//! `cpl run` reads standard input as the program asks for it: a program
//! at one end of a pipe answers a line before the pipe is closed.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn cpl_run_answers_before_standard_input_ends() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("live_stdin");
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("echo.cp");
    std::fs::write(
        &source,
        "let n: Int = read()\nprint(n * 2)\nflush()\nlet m: Int = read()\nprint(m + 1)\n",
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(&source)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    // Had `cpl run` waited for the end of its input, this read would never
    // return, so a helper thread ends the wait after ten seconds.
    let (answered, waiting) = std::sync::mpsc::channel();
    let pid = child.id();
    std::thread::spawn(move || {
        if waiting.recv_timeout(std::time::Duration::from_secs(10)).is_err() {
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
    });
    stdin.write_all(b"21\n").unwrap();
    stdin.flush().unwrap();
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    answered.send(()).unwrap();
    assert_eq!(line, "42\n", "the first answer arrives while the pipe is open");
    stdin.write_all(b"9\n").unwrap();
    drop(stdin);
    line.clear();
    stdout.read_line(&mut line).unwrap();
    assert_eq!(line, "10\n");
    assert!(child.wait().unwrap().success());
}
