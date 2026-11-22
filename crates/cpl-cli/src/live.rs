//! `cpl run --trace --live`: the trace viewer served on localhost, recorded
//! again from the file on disk each time it is saved, on the same input. The
//! page polls for the new recording, keeps its place and says where the two
//! runs first differ. `POST /replay` replays one call of the shown run with
//! the fn as the page or the file has it now, as `cpl replay` does.

use std::io::{BufRead, BufReader, Read, Write};
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

/// What a replay needs: the file and the input every run is on.
struct Run {
    path: String,
    input: Vec<u8>,
}

/// Serves until interrupted. `record` turns the file as it is now into a
/// trace document, or says why it can't.
pub fn serve(
    path: &str,
    input: &[u8],
    input_note: &str,
    record: &dyn Fn() -> Result<String, String>,
) -> u8 {
    let first = match record() {
        Ok(document) => document,
        Err(error) => {
            eprint!("{error}");
            return crate::EXIT_FAILED;
        }
    };
    let shown = Arc::new(Mutex::new(Shown {
        version: 1,
        document: first,
        error: None,
    }));
    let listener = match TcpListener::bind(("127.0.0.1", 7357))
        .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
    {
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
    let run = Arc::new(Run {
        path: path.to_string(),
        input: input.to_vec(),
    });
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (shown, run) = (Arc::clone(&served), Arc::clone(&run));
            std::thread::spawn(move || answer(stream, &shown, &run));
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

fn answer(stream: TcpStream, shown: &Mutex<Shown>, run: &Run) {
    let mut reader = BufReader::new(&stream);
    let mut request = String::new();
    if reader.read_line(&mut request).is_err() {
        return;
    }
    let (mut header, mut length) = (String::new(), 0);
    while reader.read_line(&mut header).is_ok_and(|n| n > 2) {
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
        header.clear();
    }
    let target = request.split_whitespace().nth(1).unwrap_or("/");
    let target = target.split('?').next().unwrap_or("/");
    let posted = request.starts_with("POST ");
    let (kind, body) = if posted && target == "/replay" {
        let mut body = vec![0; length.min(1 << 24)];
        let body = reader
            .read_exact(&mut body)
            .map(|_| String::from_utf8_lossy(&body).into_owned())
            .unwrap_or_default();
        ("application/json", replay(run, &body))
    } else {
        let shown = shown.lock().unwrap();
        match target {
            "/" | "/index.html" => {
                let page = crate::trace::html(&shown.document);
                let meta = format!(
                    "<head>\n<meta name=\"cpl-live\" content=\"{}\">",
                    shown.version
                );
                (
                    "text/html; charset=utf-8",
                    page.replacen("<head>", &meta, 1),
                )
            }
            "/trace.json" => ("application/json", shown.document.clone()),
            "/live" => {
                let error = shown
                    .error
                    .as_deref()
                    .map_or("null".to_string(), cpl_json::escape_script_safe_string);
                (
                    "application/json",
                    format!("{{\"version\":{},\"error\":{error}}}", shown.version),
                )
            }
            _ => ("text/plain", String::new()),
        }
    };
    let status = if kind == "text/plain" {
        "404 Not Found"
    } else {
        "200 OK"
    };
    let mut stream = &stream;
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

/// `{original, edited, step, frame}`: the call open at `step` (0-based) of
/// the run of `original`, replayed with the fns of `edited`, or of the file
/// on disk when `edited` is null. The answer is [`cpl_interp::replay::Replay::json`],
/// or `{"error": ...}`.
fn replay(run: &Run, body: &str) -> String {
    use crate::dap::Json;
    let error = |message: &str| {
        format!(
            "{{\"error\":{}}}",
            cpl_json::escape_script_safe_string(message.trim_end())
        )
    };
    let Ok(request) = cpl_json::parse(body) else {
        return error("the replay request is not JSON");
    };
    let (Some(original), Some(step)) = (
        request.get("original").and_then(Json::as_str),
        request
            .get("step")
            .and_then(Json::as_i64)
            .and_then(|n| usize::try_from(n).ok()),
    ) else {
        return error("a replay needs the recorded source and a step");
    };
    let edited = request.get("edited").and_then(Json::as_str);
    let frame = request
        .get("frame")
        .and_then(Json::as_i64)
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(0);
    let pick =
        |recorded: &cpl_interp::trace::Recording| crate::replay::call_at(recorded, step, frame);
    match crate::replay::replay_call(&run.path, original, edited, &run.input, &pick) {
        Ok(replay) => replay.json(),
        Err(message) => error(&message),
    }
}
