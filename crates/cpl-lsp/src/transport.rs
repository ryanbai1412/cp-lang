//! Reading and writing protocol messages over a byte stream, framed with
//! `Content-Length` headers.

use crate::server::Server;
use cpl_json::{self as json, Value};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

/// The largest message body accepted. A document sync of a huge file is a
/// few megabytes; anything past this is a broken client.
const MAX_BODY: usize = 256 << 20;

/// Reads one message. `Ok(None)` at a clean end of input.
pub fn read_message(input: &mut (impl BufRead + ?Sized)) -> io::Result<Option<String>> {
    let mut length = None;
    let mut saw_header = false;
    let mut line = String::new();
    loop {
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            if saw_header {
                break;
            }
            // Stray blank lines between messages are skipped.
            continue;
        }
        saw_header = true;
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse::<usize>().ok();
            }
        }
    }
    let length = length.filter(|n| *n <= MAX_BODY).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "missing or oversized Content-Length",
        )
    })?;
    let mut body = vec![0; length];
    input.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "message body is not UTF-8"))
}

/// Writes a pre-serialized body framed with a `Content-Length` header and flushes.
pub fn write_raw_message(output: &mut (impl Write + ?Sized), body: &str) -> io::Result<()> {
    write!(output, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    output.flush()
}

pub fn write_message(output: &mut (impl Write + ?Sized), message: &Value) -> io::Result<()> {
    write_raw_message(output, &message.to_string())
}

/// Serves the protocol until the client sends `exit` or closes the input.
/// Returns the process exit code.
pub fn serve(input: impl BufRead, output: impl Write + Send) -> io::Result<i32> {
    serve_with(Server::new(), input, output)
}

/// [`serve`] with `server`, as the binary serving sets it up. What the
/// server sends unasked (its outbox) is written between replies.
pub fn serve_with(
    server: Server,
    mut input: impl BufRead,
    output: impl Write + Send,
) -> io::Result<i32> {
    let (outbox, unasked) = std::sync::mpsc::channel::<Value>();
    #[cfg(not(target_arch = "wasm32"))]
    let mut server = server.with_outbox(outbox);
    #[cfg(target_arch = "wasm32")]
    let (mut server, _) = (server, outbox);
    let output = Mutex::new(output);
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let (output, done) = (&output, &done);
        scope.spawn(move || {
            while !done.load(Ordering::Relaxed) {
                if let Ok(message) = unasked.recv_timeout(Duration::from_millis(50)) {
                    let _ = write_message(&mut *lock(output), &message);
                }
            }
        });
        let served = serve_messages(&mut server, &mut input, output);
        done.store(true, Ordering::Relaxed);
        served
    })
}

fn lock<W>(output: &Mutex<W>) -> std::sync::MutexGuard<'_, W> {
    output
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn serve_messages(
    server: &mut Server,
    input: &mut impl BufRead,
    output: &Mutex<impl Write>,
) -> io::Result<i32> {
    while let Some(body) = read_message(input)? {
        let replies = match json::parse(&body) {
            Ok(message) => server.respond(&message),
            Err(what) => vec![json::obj([
                ("jsonrpc", "2.0".into()),
                ("id", Value::Null),
                (
                    "error",
                    json::obj([
                        ("code", (-32700i64).into()),
                        ("message", what.to_string().into()),
                    ]),
                ),
            ])],
        };
        for reply in replies.iter().chain(&server.idle()) {
            write_message(&mut *lock(output), reply)?;
        }
        if let Some(code) = server.exit_code() {
            return Ok(code);
        }
    }
    // The client went away without `exit`.
    Ok(1)
}
