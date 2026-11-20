//! Runs a child process with a time limit and captures what it prints.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    /// Killed by a signal (a crash), or by us after the time limit.
    Signal,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub exit: Exit,
    pub elapsed: Duration,
}

impl Output {
    pub fn ok(&self) -> bool {
        self.exit == Exit::Code(0)
    }
}

/// Runs `cmd`, feeding it `stdin`, for at most `limit`.
pub fn run(cmd: &mut Command, stdin: &[u8], limit: Duration) -> std::io::Result<Output> {
    let start = Instant::now();
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();
    let (stdout, stderr, exit) = std::thread::scope(|s| {
        // Pipes are drained on their own threads so a child that prints a
        // lot, or does not read its input, cannot block us or itself.
        s.spawn(move || {
            let _ = child_stdin.write_all(stdin);
        });
        let out = s.spawn(move || {
            let mut buf = Vec::new();
            let _ = child_stdout.read_to_end(&mut buf);
            buf
        });
        let err = s.spawn(move || {
            let mut buf = Vec::new();
            let _ = child_stderr.read_to_end(&mut buf);
            buf
        });
        let exit = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status.code().map_or(Exit::Signal, Exit::Code),
                Ok(None) if start.elapsed() >= limit => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Exit::TimedOut;
                }
                Ok(None) => std::thread::sleep(Duration::from_micros(500)),
                Err(_) => break Exit::Signal,
            }
        };
        (out.join().unwrap(), err.join().unwrap(), exit)
    });
    Ok(Output {
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        exit,
        elapsed: start.elapsed(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_output_and_exit_code() {
        let out = run(
            Command::new("sh").args(["-c", "cat; echo err >&2; exit 3"]),
            b"hello\n",
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(out.stdout, "hello\n");
        assert_eq!(out.stderr, "err\n");
        assert_eq!(out.exit, Exit::Code(3));
    }

    #[test]
    fn kills_a_process_that_runs_too_long() {
        let out = run(
            Command::new("sleep").arg("30"),
            b"",
            Duration::from_millis(100),
        )
        .unwrap();
        assert_eq!(out.exit, Exit::TimedOut);
        assert!(out.elapsed < Duration::from_secs(5));
    }
}
