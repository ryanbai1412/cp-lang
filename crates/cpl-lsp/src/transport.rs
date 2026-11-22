//! Reading and writing protocol messages over a byte stream, framed with
//! `Content-Length` headers.

use cpl_json::{self as json, Value};
use crate::server::Server;
use std::io::{self, BufRead, Write};

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
pub fn serve(mut input: impl BufRead, mut output: impl Write) -> io::Result<i32> {
    let mut server = Server::new();
    while let Some(body) = read_message(&mut input)? {
        let replies = match json::parse(&body) {
            Ok(message) => server.handle(&message),
            Err(what) => vec![json::obj([
                ("jsonrpc", "2.0".into()),
                ("id", Value::Null),
                (
                    "error",
                    json::obj([("code", (-32700i64).into()), ("message", what.to_string().into())]),
                ),
            ])],
        };
        for reply in &replies {
            write_message(&mut output, reply)?;
        }
        if let Some(code) = server.exit_code() {
            return Ok(code);
        }
    }
    // The client went away without `exit`.
    Ok(1)
}
