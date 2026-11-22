//! `cpl run --trace --live`: the trace viewer served on localhost, recorded
//! again from the file on disk each time it is saved, on the same input. The
//! page polls for the new recording, keeps its place and says where the two
//! runs first differ.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct Shown {
    version: u64,
    document: String,
    /// Why the latest save did not record: the last good document stays.
    error: Option<String>,
}

/// Serves until interrupted. `record` turns the file as it is now into a
/// trace document, or says why it can't.
pub fn serve(path: &str, input_note: &str, record: &dyn Fn() -> Result<String, String>) -> u8 {
    let first = match record() {
        Ok(document) => document,
        Err(error) => {
            eprint!("{error}");
            return crate::EXIT_FAILED;
        }
    };
    let shown = Arc::new(Mutex::new(Shown { version: 1, document: first, error: None }));
    let listener = match TcpListener::bind(("127.0.0.1", 7357)).or_else(|_| TcpListener::bind(("127.0.0.1", 0))) {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("cpl: can't serve the trace: {error}");
            return crate::EXIT_USAGE;
        }
    };
    let port = listener.local_addr().map_or(0, |a| a.port());
    eprintln!("cpl: serving the trace of {path} at http://127.0.0.1:{port}/");
    eprintln!("cpl: input: {input_note}; save the file to record it again (Ctrl-C stops)");
    let served = Arc::clone(&shown);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let shown = Arc::clone(&served);
            std::thread::spawn(move || answer(stream, &shown));
        }
    });
    let mut last = std::fs::read(path).ok();
    loop {
        std::thread::sleep(Duration::from_millis(200));
        let now = std::fs::read(path).ok();
        if now.is_none() || now == last {
            continue;
        }
        last = now;
        let result = record();
        let mut shown = shown.lock().unwrap();
        match result {
            Ok(document) => {
                shown.version += 1;
                shown.document = document;
                shown.error = None;
                eprintln!("cpl: recorded the edit (version {})", shown.version);
            }
            Err(error) => {
                eprint!("{error}");
                shown.error = Some(error.trim().to_string());
            }
        }
    }
}

fn answer(stream: TcpStream, shown: &Mutex<Shown>) {
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    if reader.read_line(&mut request).is_err() {
        return;
    }
    let mut header = String::new();
    while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
        header.clear();
    }
    let target = request.split_whitespace().nth(1).unwrap_or("/");
    let target = target.split('?').next().unwrap_or("/");
    let (kind, body) = {
        let shown = shown.lock().unwrap();
        match target {
            "/" | "/index.html" => {
                let page = crate::trace::html(&shown.document);
                let meta = format!("<head>\n<meta name=\"cpl-live\" content=\"{}\">", shown.version);
                ("text/html; charset=utf-8", page.replacen("<head>", &meta, 1))
            }
            "/trace.json" => ("application/json", shown.document.clone()),
            "/live" => {
                let error = shown.error.as_deref().map_or("null".to_string(), crate::trace::json_string);
                ("application/json", format!("{{\"version\":{},\"error\":{error}}}", shown.version))
            }
            _ => ("text/plain", String::new()),
        }
    };
    let status = if kind == "text/plain" { "404 Not Found" } else { "200 OK" };
    let mut stream = &stream;
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}
