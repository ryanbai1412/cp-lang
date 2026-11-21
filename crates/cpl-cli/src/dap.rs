//! Time-travel Debug Adapter Protocol (DAP) server (`cpl dap`) over stdio.
//!
//! Runs the `.cp` program once in `cpl-interp` with full frame and variable
//! snapshots, then serves standard DAP requests over `Content-Length`-framed
//! JSON on stdin/stdout — supporting breakpoints, forward stepping (`next`,
//! `stepIn`, `stepOut`, `continue`), backward time-travel (`stepBack`,
//! `reverseContinue`), call stacks, domain-labeled variables, hover/watch
//! evaluation, and exception stops on runtime traps.

use crate::{compile_side, render_trap, Side};
use cpl_interp::trace::{string as json_str, Recording, TraceConfig};
use std::collections::{BTreeMap, HashSet};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn serve_stdio() -> u8 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();
    match serve(&mut reader, &mut writer) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("cpl dap: {err}");
            1
        }
    }
}

/// Runs the DAP session over arbitrary streams (used by stdio and tests).
pub fn serve(reader: &mut dyn BufRead, writer: &mut dyn Write) -> io::Result<()> {
    let mut session = Session::new();
    while let Some(body) = read_frame(reader)? {
        let Ok(msg) = Json::parse(&body) else {
            continue;
        };
        if msg.get("type").and_then(Json::as_str) != Some("request") {
            continue;
        }
        let req_seq = msg.get("seq").and_then(Json::as_i64).unwrap_or(0);
        let command = msg.get("command").and_then(Json::as_str).unwrap_or("");
        let args = msg.get("arguments");
        let quit = session.handle(req_seq, command, args, writer)?;
        writer.flush()?;
        if quit {
            break;
        }
    }
    Ok(())
}

struct Session {
    seq: i64,
    program_path: String,
    source_name: String,
    recording: Recording,
    trap_message: Option<String>,
    /// 0-based index into `recording.steps`.
    cursor: usize,
    /// 1-based line breakpoints for the launched file.
    breakpoints: HashSet<u32>,
    stop_on_entry: bool,
    launched: bool,
    /// Step index that holds the final trap step, if the run trapped.
    trap_step: Option<usize>,
}

impl Session {
    fn new() -> Self {
        Self {
            seq: 1,
            program_path: String::new(),
            source_name: String::new(),
            recording: Recording::default(),
            trap_message: None,
            cursor: 0,
            breakpoints: HashSet::new(),
            stop_on_entry: true,
            launched: false,
            trap_step: None,
        }
    }

    fn next_seq(&mut self) -> i64 {
        let s = self.seq;
        self.seq += 1;
        s
    }

    fn send_response(
        &mut self,
        writer: &mut dyn Write,
        req_seq: i64,
        command: &str,
        success: bool,
        message: Option<&str>,
        body_json: Option<&str>,
    ) -> io::Result<()> {
        let seq = self.next_seq();
        let mut out = format!(
            "{{\"seq\":{seq},\"type\":\"response\",\"request_seq\":{req_seq},\"success\":{success},\"command\":"
        );
        json_str(&mut out, command);
        if let Some(msg) = message {
            out.push_str(",\"message\":");
            json_str(&mut out, msg);
        }
        if let Some(body) = body_json {
            out.push_str(",\"body\":");
            out.push_str(body);
        }
        out.push('}');
        write_frame(writer, &out)
    }

    fn send_event(&mut self, writer: &mut dyn Write, event: &str, body_json: Option<&str>) -> io::Result<()> {
        let seq = self.next_seq();
        let mut out = format!("{{\"seq\":{seq},\"type\":\"event\",\"event\":");
        json_str(&mut out, event);
        if let Some(body) = body_json {
            out.push_str(",\"body\":");
            out.push_str(body);
        }
        out.push('}');
        write_frame(writer, &out)
    }

    fn send_output(&mut self, writer: &mut dyn Write, category: &str, text: &str) -> io::Result<()> {
        let mut body = String::from("{\"category\":");
        json_str(&mut body, category);
        body.push_str(",\"output\":");
        json_str(&mut body, text);
        body.push('}');
        self.send_event(writer, "output", Some(&body))
    }

    fn send_stopped(&mut self, writer: &mut dyn Write, reason: &str, text: Option<&str>) -> io::Result<()> {
        let mut body = String::from("{\"reason\":");
        json_str(&mut body, reason);
        body.push_str(",\"threadId\":1,\"allThreadsStopped\":true");
        if let Some(text) = text {
            body.push_str(",\"text\":");
            json_str(&mut body, text);
        }
        body.push('}');
        self.send_event(writer, "stopped", Some(&body))
    }

    fn send_terminated(&mut self, writer: &mut dyn Write) -> io::Result<()> {
        let exit_code = i32::from(self.trap_message.is_some());
        self.send_event(writer, "exited", Some(&format!("{{\"exitCode\":{exit_code}}}")))?;
        self.send_event(writer, "terminated", Some("{}"))
    }

    fn current_depth(&self) -> usize {
        self.recording
            .steps
            .get(self.cursor)
            .map_or(1, |s| s.frames.len().max(1))
    }

    fn handle(
        &mut self,
        req_seq: i64,
        command: &str,
        args: Option<&Json>,
        writer: &mut dyn Write,
    ) -> io::Result<bool> {
        match command {
            "initialize" => {
                let caps = "\
{\
\"supportsConfigurationDoneRequest\":true,\
\"supportsStepBack\":true,\
\"supportsEvaluateForHovers\":true,\
\"supportsTerminateRequest\":true,\
\"supportsRestartRequest\":true\
}";
                self.send_response(writer, req_seq, command, true, None, Some(caps))?;
                self.send_event(writer, "initialized", None)?;
            }
            "launch" => {
                let program = args
                    .and_then(|a| a.get("program"))
                    .and_then(Json::as_str)
                    .unwrap_or("");
                if program.is_empty() {
                    self.send_response(
                        writer,
                        req_seq,
                        command,
                        false,
                        Some("`launch` requires a `program` path"),
                        None,
                    )?;
                    return Ok(false);
                }
                self.stop_on_entry = args
                    .and_then(|a| a.get("stopOnEntry"))
                    .and_then(Json::as_bool)
                    .unwrap_or(true);
                let brute = args
                    .and_then(|a| a.get("brute"))
                    .and_then(Json::as_bool)
                    .unwrap_or(false);
                let input_bytes = if let Some(text) = args.and_then(|a| a.get("inputText")).and_then(Json::as_str) {
                    text.as_bytes().to_vec()
                } else if let Some(input_path) = args
                    .and_then(|a| a.get("input"))
                    .and_then(Json::as_str)
                    .filter(|s| !s.is_empty())
                {
                    match std::fs::read(input_path) {
                        Ok(bytes) => bytes,
                        Err(err) => {
                            let msg = format!("cannot read input `{input_path}`: {err}");
                            self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                            return Ok(false);
                        }
                    }
                } else {
                    Vec::new()
                };

                let side = if brute { Side::Brute } else { Side::Solution };
                let mut compile_err = String::new();
                let compiled = match compile_side(program, side, &mut compile_err) {
                    Ok(c) => c,
                    Err(_) => {
                        self.send_output(writer, "stderr", &compile_err)?;
                        self.send_response(writer, req_seq, command, false, Some(compile_err.trim()), None)?;
                        return Ok(false);
                    }
                };

                let config = TraceConfig {
                    source: compiled.text.clone(),
                    record_frames: true,
                    ..TraceConfig::default()
                };
                let mut printed = Vec::new();
                let observed = cpl_interp::run_observed(
                    &compiled.program,
                    cpl_interp::Io::fixed(&input_bytes),
                    &mut printed,
                    Some(config),
                );
                self.program_path = program.to_string();
                self.source_name = Path::new(program)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(program)
                    .to_string();
                self.recording = observed.recording.unwrap_or_default();
                self.cursor = 0;
                self.launched = true;
                self.trap_message = observed.trap.as_ref().map(|t| render_trap(program, &compiled.text, t));
                self.trap_step = observed
                    .trap
                    .as_ref()
                    .and_then(|_| self.recording.steps.len().checked_sub(1));

                self.send_response(writer, req_seq, command, true, None, None)?;
                if !printed.is_empty() {
                    let out = String::from_utf8_lossy(&printed);
                    self.send_output(writer, "stdout", &out)?;
                }
                if !observed.debug.is_empty() {
                    let dbg = String::from_utf8_lossy(&observed.debug);
                    self.send_output(writer, "console", &dbg)?;
                }
                if let Some(trap) = self.trap_message.clone() {
                    self.send_output(writer, "stderr", &trap)?;
                }
            }
            "setBreakpoints" => {
                self.breakpoints.clear();
                let mut verified = String::from("{\"breakpoints\":[");
                if let Some(list) = args.and_then(|a| a.get("breakpoints")).and_then(Json::as_array) {
                    for (i, bp) in list.iter().enumerate() {
                        if i > 0 {
                            verified.push(',');
                        }
                        let line = bp.get("line").and_then(Json::as_i64).unwrap_or(1).max(1) as u32;
                        self.breakpoints.insert(line);
                        verified += &format!("{{\"verified\":true,\"line\":{line}}}");
                    }
                }
                verified.push_str("]}");
                self.send_response(writer, req_seq, command, true, None, Some(&verified))?;
            }
            "configurationDone" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                if !self.launched || self.recording.steps.is_empty() {
                    self.send_terminated(writer)?;
                } else if self.stop_on_entry {
                    self.cursor = 0;
                    self.send_stopped(writer, "entry", None)?;
                } else {
                    self.advance_continue(writer, true)?;
                }
            }
            "threads" => {
                self.send_response(
                    writer,
                    req_seq,
                    command,
                    true,
                    None,
                    Some("{\"threads\":[{\"id\":1,\"name\":\"main\"}]}"),
                )?;
            }
            "stackTrace" => {
                let mut body = String::from("{\"stackFrames\":[");
                let count = if let Some(step) = self.recording.steps.get(self.cursor) {
                    let frames: Vec<(&str, u32)> = if step.frames.is_empty() {
                        vec![("main", step.line)]
                    } else {
                        step.frames.iter().map(|(n, l)| (n.as_str(), *l)).collect()
                    };
                    for (idx, (name, line)) in frames.iter().enumerate() {
                        if idx > 0 {
                            body.push(',');
                        }
                        body += &format!("{{\"id\":{},\"name\":", idx + 1);
                        json_str(&mut body, name);
                        body.push_str(",\"source\":{\"name\":");
                        json_str(&mut body, &self.source_name);
                        body.push_str(",\"path\":");
                        json_str(&mut body, &self.program_path);
                        body += &format!("}},\"line\":{line},\"column\":1}}");
                    }
                    frames.len()
                } else {
                    0
                };
                body += &format!("],\"totalFrames\":{count}}}");
                self.send_response(writer, req_seq, command, true, None, Some(&body))?;
            }
            "scopes" => {
                let body = "\
{\
\"scopes\":[\
{\"name\":\"Locals\",\"variablesReference\":1,\"expensive\":false},\
{\"name\":\"Globals\",\"variablesReference\":2,\"expensive\":false}\
]\
}";
                self.send_response(writer, req_seq, command, true, None, Some(body))?;
            }
            "variables" => {
                let reference = args
                    .and_then(|a| a.get("variablesReference"))
                    .and_then(Json::as_i64)
                    .unwrap_or(0);
                let mut body = String::from("{\"variables\":[");
                if let Some(step) = self.recording.steps.get(self.cursor) {
                    let mut first = true;
                    if reference == 1 {
                        if let Ok(Json::Object(map)) = Json::parse(&step.locals) {
                            for (name, val) in map {
                                if !first {
                                    body.push(',');
                                }
                                first = false;
                                emit_variable(&mut body, &name, &val.to_compact_string(), "");
                            }
                        }
                    } else if reference == 2 {
                        for (var_idx, val_json) in &step.values {
                            let var = &self.recording.vars[*var_idx];
                            if var.id == "stdout" {
                                continue;
                            }
                            if !first {
                                body.push(',');
                            }
                            first = false;
                            emit_variable(&mut body, &var.name, val_json, &var.ty);
                        }
                    }
                }
                body.push_str("]}");
                self.send_response(writer, req_seq, command, true, None, Some(&body))?;
            }
            "evaluate" => {
                let expr = args
                    .and_then(|a| a.get("expression"))
                    .and_then(Json::as_str)
                    .unwrap_or("")
                    .trim();
                match self.eval_expr(expr) {
                    Some((val, ty)) => {
                        let mut body = String::from("{\"result\":");
                        json_str(&mut body, &val);
                        if !ty.is_empty() {
                            body.push_str(",\"type\":");
                            json_str(&mut body, &ty);
                        }
                        body.push_str(",\"variablesReference\":0}");
                        self.send_response(writer, req_seq, command, true, None, Some(&body))?;
                    }
                    None => {
                        let msg = format!("`{expr}` is not in scope at step {}", self.cursor);
                        self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                    }
                }
            }
            "continue" => {
                self.send_response(
                    writer,
                    req_seq,
                    command,
                    true,
                    None,
                    Some("{\"allThreadsContinued\":true}"),
                )?;
                self.advance_continue(writer, false)?;
            }
            "next" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let depth = self.current_depth();
                let start_line = self.recording.steps.get(self.cursor).map_or(0, |s| s.line);
                if self.cursor + 1 >= self.recording.steps.len() {
                    self.send_terminated(writer)?;
                } else {
                    let mut i = self.cursor + 1;
                    while i < self.recording.steps.len() {
                        let step = &self.recording.steps[i];
                        let d = step.frames.len().max(1);
                        if self.trap_step == Some(i) {
                            self.cursor = i;
                            let msg = self.trap_message.clone();
                            self.send_stopped(writer, "exception", msg.as_deref())?;
                            return Ok(false);
                        }
                        if self.breakpoints.contains(&step.line) {
                            self.cursor = i;
                            self.send_stopped(writer, "breakpoint", None)?;
                            return Ok(false);
                        }
                        if d <= depth && (step.line != start_line || i + 1 == self.recording.steps.len()) {
                            self.cursor = i;
                            self.send_stopped(writer, "step", None)?;
                            return Ok(false);
                        }
                        i += 1;
                    }
                    self.cursor = self.recording.steps.len() - 1;
                    self.send_terminated(writer)?;
                }
            }
            "stepIn" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                if self.cursor + 1 >= self.recording.steps.len() {
                    self.send_terminated(writer)?;
                } else {
                    self.cursor += 1;
                    if self.trap_step == Some(self.cursor) {
                        let msg = self.trap_message.clone();
                        self.send_stopped(writer, "exception", msg.as_deref())?;
                    } else {
                        self.send_stopped(writer, "step", None)?;
                    }
                }
            }
            "stepOut" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let target_depth = self.current_depth().saturating_sub(1);
                let mut found = None;
                for i in (self.cursor + 1)..self.recording.steps.len() {
                    if self.trap_step == Some(i) {
                        self.cursor = i;
                        let msg = self.trap_message.clone();
                        self.send_stopped(writer, "exception", msg.as_deref())?;
                        return Ok(false);
                    }
                    let d = self.recording.steps[i].frames.len().max(1);
                    if d <= target_depth || self.breakpoints.contains(&self.recording.steps[i].line) {
                        found = Some(i);
                        break;
                    }
                }
                match found {
                    Some(i) => {
                        self.cursor = i;
                        let reason = if self.breakpoints.contains(&self.recording.steps[i].line) {
                            "breakpoint"
                        } else {
                            "step"
                        };
                        self.send_stopped(writer, reason, None)?;
                    }
                    None => self.send_terminated(writer)?,
                }
            }
            "stepBack" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                self.send_stopped(writer, "step", None)?;
            }
            "reverseContinue" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let mut i = self.cursor;
                while i > 0 {
                    i -= 1;
                    if self.breakpoints.contains(&self.recording.steps[i].line) {
                        self.cursor = i;
                        self.send_stopped(writer, "breakpoint", None)?;
                        return Ok(false);
                    }
                }
                self.cursor = 0;
                self.send_stopped(writer, "step", None)?;
            }
            "restart" => {
                self.cursor = 0;
                self.send_response(writer, req_seq, command, true, None, None)?;
                if !self.recording.steps.is_empty() {
                    self.send_stopped(writer, "entry", None)?;
                }
            }
            "disconnect" | "terminate" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                return Ok(true);
            }
            _ => {
                self.send_response(writer, req_seq, command, true, None, Some("{}"))?;
            }
        }
        Ok(false)
    }

    fn advance_continue(&mut self, writer: &mut dyn Write, include_current: bool) -> io::Result<()> {
        let start = if include_current {
            self.cursor
        } else {
            self.cursor + 1
        };
        for i in start..self.recording.steps.len() {
            if self.trap_step == Some(i) {
                self.cursor = i;
                let msg = self.trap_message.clone();
                return self.send_stopped(writer, "exception", msg.as_deref());
            }
            if self.breakpoints.contains(&self.recording.steps[i].line) {
                self.cursor = i;
                return self.send_stopped(writer, "breakpoint", None);
            }
        }
        self.cursor = self.recording.steps.len().saturating_sub(1);
        self.send_terminated(writer)
    }

    fn eval_expr(&self, expr: &str) -> Option<(String, String)> {
        let step = self.recording.steps.get(self.cursor)?;
        if expr.is_empty() {
            return None;
        }
        if let Ok(Json::Object(map)) = Json::parse(&step.locals) {
            if let Some(v) = map.get(expr) {
                return Some((v.to_compact_string(), String::new()));
            }
        }
        for (var_idx, val_json) in &step.values {
            let var = &self.recording.vars[*var_idx];
            if var.name == expr || var.id == expr {
                return Some((val_json.to_string(), var.ty.clone()));
            }
        }
        None
    }
}

fn emit_variable(out: &mut String, name: &str, value: &str, ty: &str) {
    out.push_str("{\"name\":");
    json_str(out, name);
    out.push_str(",\"value\":");
    json_str(out, value);
    if !ty.is_empty() {
        out.push_str(",\"type\":");
        json_str(out, ty);
    }
    out.push_str(",\"variablesReference\":0}");
}

// ----- Content-Length stdio framing -----

fn read_frame(reader: &mut dyn BufRead) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            return Ok(None);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            if k.eq_ignore_ascii_case("Content-Length") {
                content_length = v.trim().parse().ok();
            }
        }
    }
    let len = match content_length {
        Some(len) => len,
        None => return Ok(None),
    };
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
}

fn write_frame(writer: &mut dyn Write, body: &str) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
}

// ----- Minimal self-contained JSON parser -----

#[derive(Clone, Debug)]
enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    fn parse(input: &str) -> Result<Json, ()> {
        let mut p = Parser {
            bytes: input.as_bytes(),
            pos: 0,
        };
        let val = p.value()?;
        p.ws();
        if p.pos == p.bytes.len() {
            Ok(val)
        } else {
            Err(())
        }
    }

    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(map) => map.get(key),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Json::Num(n) => n.parse().ok(),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    fn to_compact_string(&self) -> String {
        match self {
            Json::Null => "null".into(),
            Json::Bool(b) => b.to_string(),
            Json::Num(n) => n.clone(),
            Json::Str(s) => {
                let mut out = String::new();
                json_str(&mut out, s);
                out
            }
            Json::Array(items) => {
                let parts: Vec<String> = items.iter().map(Json::to_compact_string).collect();
                format!("[{}]", parts.join(", "))
            }
            Json::Object(map) => {
                let parts: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.to_compact_string()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn ws(&mut self) {
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn value(&mut self) -> Result<Json, ()> {
        self.ws();
        match self.peek().ok_or(())? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => self.string().map(Json::Str),
            b't' => self.literal(b"true", Json::Bool(true)),
            b'f' => self.literal(b"false", Json::Bool(false)),
            b'n' => self.literal(b"null", Json::Null),
            b'-' | b'0'..=b'9' => self.number(),
            _ => Err(()),
        }
    }

    fn literal(&mut self, lit: &[u8], val: Json) -> Result<Json, ()> {
        if self.bytes[self.pos..].starts_with(lit) {
            self.pos += lit.len();
            Ok(val)
        } else {
            Err(())
        }
    }

    fn number(&mut self) -> Result<Json, ()> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        while self.pos < self.bytes.len()
            && matches!(self.bytes[self.pos], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
        {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| ())?;
        Ok(Json::Num(text.to_string()))
    }

    fn string(&mut self) -> Result<String, ()> {
        if self.peek() != Some(b'"') {
            return Err(());
        }
        self.pos += 1;
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.pos += 1;
            match b {
                b'"' => return Ok(out),
                b'\\' => {
                    let esc = *self.bytes.get(self.pos).ok_or(())?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'b' => out.push('\x08'),
                        b'f' => out.push('\x0c'),
                        b'u' => {
                            let hex = self.bytes.get(self.pos..self.pos + 4).ok_or(())?;
                            self.pos += 4;
                            let s = std::str::from_utf8(hex).map_err(|_| ())?;
                            let code = u32::from_str_radix(s, 16).map_err(|_| ())?;
                            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        _ => return Err(()),
                    }
                }
                _ => {
                    let start = self.pos - 1;
                    while self.pos < self.bytes.len()
                        && self.bytes[self.pos] != b'"'
                        && self.bytes[self.pos] != b'\\'
                    {
                        self.pos += 1;
                    }
                    let chunk = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|_| ())?;
                    out.push_str(chunk);
                }
            }
        }
        Err(())
    }

    fn array(&mut self) -> Result<Json, ()> {
        self.pos += 1;
        let mut out = Vec::new();
        loop {
            self.ws();
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Json::Array(out));
            }
            out.push(self.value()?);
            self.ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Array(out));
                }
                _ => return Err(()),
            }
        }
    }

    fn object(&mut self) -> Result<Json, ()> {
        self.pos += 1;
        let mut map = BTreeMap::new();
        loop {
            self.ws();
            if self.peek() == Some(b'}') {
                self.pos += 1;
                return Ok(Json::Object(map));
            }
            let key = self.string()?;
            self.ws();
            if self.peek() != Some(b':') {
                return Err(());
            }
            self.pos += 1;
            let val = self.value()?;
            map.insert(key, val);
            self.ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Object(map));
                }
                _ => return Err(()),
            }
        }
    }
}
