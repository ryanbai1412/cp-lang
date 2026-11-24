//! `cpl.import`: the server takes the problem Competitive Companion posts,
//! creates it in the folder the editor named, and opens it with
//! `window/showDocument`, written to the editor between replies.

use cpl_json::{self as json, Value};
use std::io::{BufReader, Read, Write};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

const COMPANION: &str = r#"{"name": "D. Range Max", "group": "Codeforces",
  "url": "https://codeforces.com/problemset/problem/1/D", "interactive": false,
  "memoryLimit": 512, "timeLimit": 1500, "tests": [{"input": "3\n", "output": "6\n"}],
  "batch": {"id": "b", "size": 1}}"#;

/// The editor's side of stdin: the messages, then open until `closed` fires.
struct Editor {
    bytes: Vec<u8>,
    at: usize,
    closed: Receiver<()>,
}

impl Read for Editor {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.at == self.bytes.len() {
            let _ = self.closed.recv();
            return Ok(0);
        }
        let n = buf.len().min(self.bytes.len() - self.at);
        buf[..n].copy_from_slice(&self.bytes[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

/// What the server wrote, with a signal on every write.
#[derive(Clone, Default)]
struct Screen(Arc<(Mutex<Vec<u8>>, Condvar)>);

impl Write for Screen {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0 .0.lock().unwrap().extend_from_slice(buf);
        self.0 .1.notify_all();
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn messages(bytes: &[u8]) -> Vec<Value> {
    let mut input = bytes;
    std::iter::from_fn(|| cpl_lsp::read_message(&mut input).ok().flatten())
        .map(|body| json::parse(&body).unwrap())
        .collect()
}

impl Screen {
    /// The first message `found` accepts, waiting on writes for up to 20 s.
    fn wait_for(&self, found: impl Fn(&Value) -> bool) -> Value {
        let deadline = Instant::now() + Duration::from_secs(20);
        let (bytes, written) = &*self.0;
        let mut bytes = bytes.lock().unwrap();
        loop {
            if let Some(message) = messages(&bytes).into_iter().find(&found) {
                return message;
            }
            let left = deadline.saturating_duration_since(Instant::now());
            assert!(!left.is_zero(), "{:?}", messages(&bytes));
            bytes = written.wait_timeout(bytes, left).unwrap().0;
        }
    }
}

fn framed(message: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{message}", message.len())
}

#[test]
fn import_creates_the_problem_and_opens_it_in_the_editor() {
    let dir = std::env::temp_dir().join(format!("cpl-lsp-import-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let folder = json::Value::from(dir.to_string_lossy().as_ref()).to_string();
    let bytes = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#.to_string(),
        format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"workspace/executeCommand","params":{{"command":"cpl.import","arguments":[{folder},0]}}}}"#
        ),
    ]
    .map(|m| framed(&m))
    .concat();
    let (close, closed) = channel();
    let screen = Screen::default();
    let serving = {
        let screen = screen.clone();
        let editor = Editor {
            bytes: bytes.into_bytes(),
            at: 0,
            closed,
        };
        std::thread::spawn(move || {
            cpl_lsp::serve_with(cpl_lsp::Server::new(), BufReader::new(editor), screen)
        })
    };

    let caps = screen.wait_for(|m| m.get("id") == Some(&Value::from(1i64)));
    let commands = caps.at(&[
        "result",
        "capabilities",
        "executeCommandProvider",
        "commands",
    ]);
    assert_eq!(
        commands,
        &Value::from(
            [
                "cpl.exec",
                "cpl.command",
                "cpl.show",
                "cpl.clear",
                "cpl.import"
            ]
            .map(Value::from)
            .to_vec()
        )
    );
    let listening = screen.wait_for(|m| m.get("id") == Some(&Value::from(2i64)));
    let port = listening.at(&["result", "port"]).as_u64().unwrap() as u16;
    screen.wait_for(|m| m.at(&["method"]).as_str() == Some("window/showMessage"));

    let mut post = std::net::TcpStream::connect(("127.0.0.1", port)).unwrap();
    let head = format!(
        "POST / HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
        COMPANION.len()
    );
    post.write_all(head.as_bytes()).unwrap();
    post.write_all(COMPANION.as_bytes()).unwrap();

    let shown = screen.wait_for(|m| m.at(&["method"]).as_str() == Some("window/showDocument"));
    let uri = shown.at(&["params", "uri"]).as_str().unwrap().to_string();
    assert!(
        shown.get("id").is_some(),
        "showDocument is a request: {shown}"
    );
    assert_eq!(shown.at(&["params", "takeFocus"]), &Value::Bool(true));
    let source = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.extension().is_some_and(|e| e == "cp"))
        .expect("the problem's .cp file");
    assert!(
        uri.starts_with("file://")
            && uri.ends_with(&*source.file_name().unwrap().to_string_lossy()),
        "{uri}"
    );
    assert!(std::fs::read_to_string(&source)
        .unwrap()
        .contains("Range Max"));

    close.send(()).unwrap();
    serving.join().unwrap().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}
