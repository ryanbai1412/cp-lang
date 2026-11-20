//! A minimal LSP client for timing a language server, and a stub server
//! to test the client against until `cpl lsp` exists.

use crate::json::Json;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

fn write_message(out: &mut impl Write, message: &Json) -> std::io::Result<()> {
    let body = message.to_string();
    write!(out, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    out.flush()
}

/// Reads one `Content-Length` framed message; `None` at end of input.
fn read_message(input: &mut impl BufRead) -> Option<Json> {
    let mut length = None;
    loop {
        let mut header = String::new();
        if input.read_line(&mut header).ok()? == 0 {
            return None;
        }
        let header = header.trim();
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let mut body = vec![0; length?];
    input.read_exact(&mut body).ok()?;
    Json::parse(std::str::from_utf8(&body).ok()?).ok()
}

pub struct Client {
    child: Child,
    stdin: ChildStdin,
    incoming: Receiver<Json>,
    /// Messages read while waiting for a different one.
    pending: Vec<Json>,
    next_id: u64,
}

impl Client {
    /// Starts `command` and speaks LSP over its stdin and stdout.
    pub fn start(command: &[String]) -> std::io::Result<Client> {
        let mut child = Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        let (tx, incoming) = mpsc::channel();
        std::thread::spawn(move || {
            while let Some(message) = read_message(&mut stdout) {
                if tx.send(message).is_err() {
                    break;
                }
            }
        });
        Ok(Client {
            child,
            stdin,
            incoming,
            pending: vec![],
            next_id: 0,
        })
    }

    pub fn notify(&mut self, method: &str, params: Json) -> Result<(), String> {
        let message = Json::obj([
            ("jsonrpc", Json::str("2.0")),
            ("method", Json::str(method)),
            ("params", params),
        ]);
        write_message(&mut self.stdin, &message).map_err(|e| format!("writing to the server: {e}"))
    }

    /// Sends a request and waits for its response's `result`.
    pub fn request(&mut self, method: &str, params: Json, limit: Duration) -> Result<Json, String> {
        self.next_id += 1;
        let id = self.next_id as f64;
        let message = Json::obj([
            ("jsonrpc", Json::str("2.0")),
            ("id", Json::Num(id)),
            ("method", Json::str(method)),
            ("params", params),
        ]);
        write_message(&mut self.stdin, &message)
            .map_err(|e| format!("writing to the server: {e}"))?;
        let response = self
            .wait_for(limit, |m| {
                m.get("method").is_none() && m.get("id").and_then(Json::as_f64) == Some(id)
            })
            .map_err(|e| format!("`{method}`: {e}"))?;
        match response.get("error") {
            Some(error) => Err(format!("`{method}` failed: {error}")),
            None => Ok(response.get("result").cloned().unwrap_or(Json::Null)),
        }
    }

    /// Waits for the next notification named `method` and returns its
    /// `params`. Earlier ones that nobody waited for are dropped first.
    pub fn notification(&mut self, method: &str, limit: Duration) -> Result<Json, String> {
        let message = self
            .wait_for(limit, |m| {
                m.get("method").and_then(Json::as_str) == Some(method)
            })
            .map_err(|e| format!("`{method}`: {e}"))?;
        Ok(message.get("params").cloned().unwrap_or(Json::Null))
    }

    /// Forgets every message that has already arrived.
    pub fn drain(&mut self) {
        self.pending.clear();
        while self.incoming.try_recv().is_ok() {}
    }

    fn wait_for(
        &mut self,
        limit: Duration,
        wanted: impl Fn(&Json) -> bool,
    ) -> Result<Json, String> {
        if let Some(i) = self.pending.iter().position(&wanted) {
            return Ok(self.pending.remove(i));
        }
        let deadline = Instant::now() + limit;
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            match self.incoming.recv_timeout(left) {
                Ok(message) if wanted(&message) => return Ok(message),
                Ok(message) => self.pending.push(message),
                Err(RecvTimeoutError::Timeout) => {
                    return Err(format!("no answer within {limit:?}"))
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("the server closed its output".into())
                }
            }
        }
    }

    pub fn shutdown(mut self) {
        let _ = self.request("shutdown", Json::Null, Duration::from_secs(2));
        let _ = self.notify("exit", Json::Null);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && matches!(self.child.try_wait(), Ok(None)) {
            std::thread::sleep(Duration::from_millis(5));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A server that answers just enough to exercise [`Client`]: it publishes
/// empty diagnostics for every opened or changed document and returns a
/// fixed hover. `cpl-verify lsp-stub` runs it.
pub fn run_stub(input: impl Read, mut output: impl Write) -> std::io::Result<()> {
    let mut input = BufReader::new(input);
    while let Some(message) = read_message(&mut input) {
        let method = message.get("method").and_then(Json::as_str).unwrap_or("");
        let reply = |result: Json| {
            Json::obj([
                ("jsonrpc", Json::str("2.0")),
                ("id", message.get("id").cloned().unwrap_or(Json::Null)),
                ("result", result),
            ])
        };
        match method {
            "initialize" => {
                let capabilities = Json::obj([
                    ("hoverProvider", Json::Bool(true)),
                    ("textDocumentSync", Json::Num(1.0)),
                ]);
                write_message(
                    &mut output,
                    &reply(Json::obj([("capabilities", capabilities)])),
                )?;
            }
            "textDocument/didOpen" | "textDocument/didChange" => {
                let uri = message
                    .get("params")
                    .and_then(|p| p.get("textDocument"))
                    .and_then(|d| d.get("uri"))
                    .cloned()
                    .unwrap_or(Json::Null);
                let params = Json::obj([("uri", uri), ("diagnostics", Json::Arr(vec![]))]);
                let publish = Json::obj([
                    ("jsonrpc", Json::str("2.0")),
                    ("method", Json::str("textDocument/publishDiagnostics")),
                    ("params", params),
                ]);
                write_message(&mut output, &publish)?;
            }
            "textDocument/hover" => {
                write_message(
                    &mut output,
                    &reply(Json::obj([("contents", Json::str("stub hover"))])),
                )?;
            }
            "shutdown" => write_message(&mut output, &reply(Json::Null))?,
            "exit" => break,
            _ if message.get("id").is_some() => write_message(&mut output, &reply(Json::Null))?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_framed_with_content_length() {
        let mut wire = Vec::new();
        let message = Json::obj([("method", Json::str("é"))]);
        write_message(&mut wire, &message).unwrap();
        assert!(wire.starts_with(b"Content-Length: 15\r\n\r\n"));
        assert_eq!(read_message(&mut &wire[..]), Some(message));
        assert_eq!(read_message(&mut &b""[..]), None);
    }

    #[test]
    fn the_stub_answers_hover_and_publishes_diagnostics() {
        let mut wire = Vec::new();
        let doc = Json::obj([(
            "textDocument",
            Json::obj([("uri", Json::str("file:///a.cp"))]),
        )]);
        for message in [
            Json::obj([("id", Json::Num(1.0)), ("method", Json::str("initialize"))]),
            Json::obj([
                ("method", Json::str("textDocument/didOpen")),
                ("params", doc.clone()),
            ]),
            Json::obj([
                ("id", Json::Num(2.0)),
                ("method", Json::str("textDocument/hover")),
                ("params", doc),
            ]),
            Json::obj([("method", Json::str("exit"))]),
        ] {
            write_message(&mut wire, &message).unwrap();
        }
        let mut out = Vec::new();
        run_stub(&wire[..], &mut out).unwrap();
        let mut out = &out[..];
        let replies: Vec<Json> = std::iter::from_fn(|| read_message(&mut out)).collect();
        assert_eq!(replies.len(), 3);
        assert!(replies[0]
            .get("result")
            .unwrap()
            .get("capabilities")
            .is_some());
        assert_eq!(
            replies[1]
                .get("params")
                .unwrap()
                .get("uri")
                .and_then(Json::as_str),
            Some("file:///a.cp")
        );
        assert_eq!(replies[2].get("id").and_then(Json::as_f64), Some(2.0));
    }
}
