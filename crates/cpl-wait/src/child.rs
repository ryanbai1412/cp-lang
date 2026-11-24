//! A child's exit as a blocking wait: a thread owns `Child::wait` and sends
//! the status, so a caller blocks until the exit or its own deadline, and an
//! exit wakes it at once.

use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// A running child whose exit can be waited on with a deadline.
pub struct Exit {
    pid: u32,
    rx: Receiver<io::Result<ExitStatus>>,
    ended: Option<Result<ExitStatus, String>>,
}

/// Hands `child` to a thread that waits for it. Take its pipes first.
pub fn on_exit(mut child: Child) -> Exit {
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });
    Exit {
        pid,
        rx,
        ended: None,
    }
}

impl Exit {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// The exit status once the child has ended, waiting for it until
    /// `deadline`; `None` if it still runs then.
    pub fn by(&mut self, deadline: Instant) -> io::Result<Option<ExitStatus>> {
        let ended = match self.ended.take() {
            Some(ended) => ended,
            None => match self
                .rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(status) => status.map_err(|e| e.to_string()),
                Err(RecvTimeoutError::Timeout) => return Ok(None),
                Err(RecvTimeoutError::Disconnected) => Err("the child's waiter ended".into()),
            },
        };
        self.ended = Some(ended.clone());
        ended.map(Some).map_err(io::Error::other)
    }

    /// `by` a deadline `wait` from now.
    pub fn within(&mut self, wait: Duration) -> io::Result<Option<ExitStatus>> {
        self.by(Instant::now() + wait)
    }

    /// Kills the child and its process group, then waits for it to end.
    pub fn kill(&mut self) -> io::Result<ExitStatus> {
        if let Some(status) = self.within(Duration::ZERO)? {
            return Ok(status);
        }
        kill(self.pid);
        let ended = match self.rx.recv() {
            Ok(status) => status.map_err(|e| e.to_string()),
            Err(_) => Err("the child's waiter ended".into()),
        };
        self.ended = Some(ended.clone());
        ended.map_err(io::Error::other)
    }
}

/// Kills process `pid` and the process group it leads, if it leads one:
/// a script's interpreter may have started children that hold its pipes open.
pub fn kill(pid: u32) {
    #[cfg(unix)]
    for target in [format!("-{pid}"), pid.to_string()] {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &target])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    // `py -3` runs the script in a child python.exe, which `Child::kill`
    // leaves running with the pipes open.
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(not(any(unix, windows)))]
    let _ = pid;
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;

    fn spawn(script: &str) -> Exit {
        on_exit(Command::new("sh").args(["-c", script]).spawn().unwrap())
    }

    #[test]
    fn an_exit_wakes_the_wait_before_its_deadline() {
        let start = Instant::now();
        let status = spawn("exit 3").within(Duration::from_secs(30)).unwrap();
        assert_eq!(status.and_then(|s| s.code()), Some(3));
        assert!(start.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_running_child_is_none_at_its_deadline_and_kill_ends_it() {
        let mut child = spawn("exec sleep 30");
        assert!(child.within(Duration::from_millis(50)).unwrap().is_none());
        let start = Instant::now();
        assert!(!child.kill().unwrap().success());
        assert!(start.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn the_status_stays_after_the_first_wait() {
        let mut child = spawn("exit 0");
        let first = child.within(Duration::from_secs(30)).unwrap();
        assert_eq!(first, child.within(Duration::ZERO).unwrap());
    }
}
