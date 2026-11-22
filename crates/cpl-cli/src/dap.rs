//! Time-travel Debug Adapter Protocol (DAP) server (`cpl dap`) over stdio.
//!
//! Runs the `.cp` program once in `cpl-interp` with full frame and variable
//! snapshots, then serves standard DAP requests over `Content-Length`-framed
//! JSON on stdin/stdout — supporting breakpoints, forward stepping (`next`,
//! `stepIn`, `stepOut`, `continue`), backward time-travel (`stepBack`,
//! `reverseContinue`), call stacks, domain-labeled variables, hover/watch
//! evaluation, and exception stops on runtime traps.

use crate::{compile_side, render_trap, Side};
use cpl_interp::trace::{first_change, string as json_str, Change, Recording, TraceConfig};
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
    brute: bool,
    /// What every recording of this session reads, restarts included.
    input: Vec<u8>,
    /// Where the latest restart first changed a value, against the run before it.
    change: Option<Change>,
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
            brute: false,
            input: Vec::new(),
            change: None,
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
                self.brute = args
                    .and_then(|a| a.get("brute"))
                    .and_then(Json::as_bool)
                    .unwrap_or(false);
                self.program_path = program.to_string();
                self.source_name = Path::new(program)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(program)
                    .to_string();
                match read_input(program, args) {
                    Ok((bytes, note)) => {
                        self.input = bytes;
                        if let Some(note) = note {
                            self.send_output(writer, "console", &note)?;
                        }
                    }
                    Err(msg) => {
                        self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                        return Ok(false);
                    }
                }
                if let Err(msg) = self.record(writer)? {
                    self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                    return Ok(false);
                }
                self.cursor = 0;
                self.launched = true;
                self.send_response(writer, req_seq, command, true, None, None)?;
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
                                emit_variable(&mut body, &name, &shown(&val), "");
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
                            let value = Json::parse(val_json).map_or_else(|_| val_json.to_string(), |v| shown(&v));
                            emit_variable(&mut body, &var.name, &value, &var.ty);
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
                        let msg = format!("can't evaluate `{expr}` at step {}: {}", self.cursor + 1, self.eval_error(expr));
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
                    if let Some(text) = self.change_at(i) {
                        self.cursor = i;
                        self.send_stopped(writer, "divergence", Some(&text))?;
                        return Ok(false);
                    }
                    if self.breakpoints.contains(&self.recording.steps[i].line) {
                        self.cursor = i;
                        self.send_stopped(writer, "breakpoint", None)?;
                        return Ok(false);
                    }
                }
                self.cursor = 0;
                self.send_stopped(writer, "step", None)?;
            }
            // Records the file on disk again, on the same input, and stops
            // where the old run stood: the same visit to the same line.
            "restart" => {
                let config = args.and_then(|a| a.get("arguments"));
                if config.is_some_and(|c| c.get("input").is_some() || c.get("inputText").is_some()) {
                    match read_input(&self.program_path, config) {
                        Ok((bytes, _)) => self.input = bytes,
                        Err(msg) => {
                            self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                            return Ok(false);
                        }
                    }
                }
                let (at, place) = (self.cursor, self.recording.place(self.cursor));
                let before = std::mem::take(&mut self.recording);
                let (trap_message, trap_step) = (self.trap_message.clone(), self.trap_step);
                if let Err(msg) = self.record(writer)? {
                    (self.recording, self.trap_message, self.trap_step) = (before, trap_message, trap_step);
                    let msg = format!("{msg}\nkept the last good recording");
                    self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                    return Ok(false);
                }
                self.cursor = place.map_or(0, |p| self.recording.find(p, at));
                self.change = first_change(&before, &self.recording);
                self.send_response(writer, req_seq, command, true, None, None)?;
                if self.recording.steps.is_empty() {
                    self.send_terminated(writer)?;
                    return Ok(false);
                }
                let note = match &self.change {
                    None => "re-ran: every value and every print is the same as before".to_string(),
                    Some(change) => format!("re-ran: {}", describe_change(change)),
                };
                self.send_output(writer, "console", &format!("{note}\n"))?;
                let line = self.recording.steps[self.cursor].line;
                let kept = match place {
                    Some(p) if p.line == line => format!("kept your place: line {line}, visit {}", p.visit),
                    _ => format!("nearest step to where you were: line {line}"),
                };
                self.send_stopped(writer, "goto", Some(&kept))?;
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

    fn change_at(&self, step: usize) -> Option<String> {
        self.change.as_ref().filter(|c| c.step == step).map(describe_change)
    }

    /// Compiles the file as it is on disk now and records one run of it on
    /// the session's input. On a compile error the old recording stays.
    fn record(&mut self, writer: &mut dyn Write) -> io::Result<Result<(), String>> {
        let side = if self.brute { Side::Brute } else { Side::Solution };
        let mut compile_err = String::new();
        let program = self.program_path.clone();
        let compiled = match compile_side(&program, side, &mut compile_err) {
            Ok(c) => c,
            Err(_) => {
                self.send_output(writer, "stderr", &compile_err)?;
                return Ok(Err(compile_err.trim().to_string()));
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
            cpl_interp::Io::fixed(&self.input),
            &mut printed,
            Some(config),
        );
        self.recording = observed.recording.unwrap_or_default();
        self.trap_message = observed.trap.as_ref().map(|t| render_trap(&program, &compiled.text, t));
        self.trap_step = observed
            .trap
            .as_ref()
            .and_then(|_| self.recording.steps.len().checked_sub(1));
        self.change = None;
        if !printed.is_empty() {
            self.send_output(writer, "stdout", &String::from_utf8_lossy(&printed))?;
        }
        if !observed.debug.is_empty() {
            self.send_output(writer, "console", &String::from_utf8_lossy(&observed.debug))?;
        }
        if let Some(trap) = self.trap_message.clone() {
            self.send_output(writer, "stderr", &trap)?;
        }
        Ok(Ok(()))
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
            if let Some(text) = self.change_at(i) {
                self.cursor = i;
                return self.send_stopped(writer, "divergence", Some(&text));
            }
            if self.breakpoints.contains(&self.recording.steps[i].line) {
                self.cursor = i;
                return self.send_stopped(writer, "breakpoint", None);
            }
        }
        self.cursor = self.recording.steps.len().saturating_sub(1);
        self.send_terminated(writer)
    }

    /// A watch or console expression over the recorded state at the cursor:
    /// names, integers, `a[i]` by the index's own labels, `a.field`, `len(a)`,
    /// arithmetic, comparisons, `and`, `or` and `not`.
    fn eval_expr(&self, expr: &str) -> Option<(String, String)> {
        let step = self.recording.steps.get(self.cursor)?;
        if let Some(var) = self.recording.vars.iter().find(|v| v.name == expr || v.id == expr) {
            let (_, value) = step.values.iter().find(|(i, _)| self.recording.vars[*i].id == var.id)?;
            let value = Json::parse(value).ok()?;
            return Some((shown(&value), var.ty.clone()));
        }
        self.evaluate(expr).ok().map(|v| (shown(&v.json), String::new()))
    }

    fn eval_error(&self, expr: &str) -> String {
        match self.evaluate(expr) {
            Err(why) => why,
            Ok(_) => "nothing is recorded here".into(),
        }
    }

    fn evaluate(&self, expr: &str) -> Result<Valued, String> {
        let step = self.recording.steps.get(self.cursor).ok_or("nothing is recorded here")?;
        let mut names = BTreeMap::new();
        for (index, value) in &step.values {
            let var = &self.recording.vars[*index];
            let labels = Json::parse(&format!("{{\"_\":0{}}}", var.extra))
                .ok()
                .and_then(|extra| extra.get("labels").cloned());
            if let Ok(json) = Json::parse(value) {
                names.insert(var.name.clone(), Valued { json, labels });
            }
        }
        if let Ok(Json::Object(map)) = Json::parse(&step.locals) {
            for (name, json) in map {
                names.insert(name, Valued { json, labels: None });
            }
        }
        let tokens = tokens(expr)?;
        let mut eval = Eval { tokens: &tokens, pos: 0, names: &names };
        let value = eval.or()?;
        match eval.tokens.get(eval.pos) {
            None => Ok(value),
            Some(token) => Err(format!("unexpected `{token}`")),
        }
    }
}

/// A recorded value as the source would write it: `(w: 1, v: 1)`, `None`.
pub(crate) fn shown(json: &Json) -> String {
    match json {
        Json::Null => "None".into(),
        Json::Str(s) => s.clone(),
        Json::Array(items) => format!("[{}]", items.iter().map(shown).collect::<Vec<_>>().join(", ")),
        Json::Object(map) => {
            format!("({})", map.iter().map(|(k, v)| format!("{k}: {}", shown(v))).collect::<Vec<_>>().join(", "))
        }
        other => other.to_compact_string(),
    }
}

#[derive(Clone)]
struct Valued {
    json: Json,
    /// The index's labels, position by position, when they don't start at 0.
    labels: Option<Json>,
}

fn tokens(expr: &str) -> Result<Vec<String>, String> {
    let chars: Vec<char> = expr.chars().collect();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if c.is_ascii_alphanumeric() || c == '_' {
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
        } else if i + 1 < chars.len() && ["==", "!=", "<=", ">=", "&&", "||"].contains(&format!("{c}{}", chars[i + 1]).as_str()) {
            i += 2;
        } else if "+-*/%()[].,<>!".contains(c) {
            i += 1;
        } else {
            return Err(format!("`{c}` is not part of a watch expression"));
        }
        out.push(chars[start..i].iter().collect());
    }
    Ok(out)
}

struct Eval<'a> {
    tokens: &'a [String],
    pos: usize,
    names: &'a BTreeMap<String, Valued>,
}

impl Eval<'_> {
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(String::as_str)
    }

    fn eat(&mut self, token: &str) -> bool {
        let found = self.peek() == Some(token);
        self.pos += found as usize;
        found
    }

    fn or(&mut self) -> Result<Valued, String> {
        let mut left = self.and()?;
        while self.eat("or") || self.eat("||") {
            let right = self.and()?;
            left = boolean(truth(&left)? || truth(&right)?);
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Valued, String> {
        let mut left = self.compare()?;
        while self.eat("and") || self.eat("&&") {
            let right = self.compare()?;
            left = boolean(truth(&left)? && truth(&right)?);
        }
        Ok(left)
    }

    fn compare(&mut self) -> Result<Valued, String> {
        let left = self.sum()?;
        for op in ["==", "!=", "<=", ">=", "<", ">"] {
            if self.eat(op) {
                let right = self.sum()?;
                if matches!(op, "==" | "!=") {
                    return Ok(boolean((left.json == right.json) == (op == "==")));
                }
                let (a, b) = (int(&left)?, int(&right)?);
                return Ok(boolean(match op {
                    "<=" => a <= b,
                    ">=" => a >= b,
                    "<" => a < b,
                    _ => a > b,
                }));
            }
        }
        Ok(left)
    }

    fn sum(&mut self) -> Result<Valued, String> {
        let mut left = self.product()?;
        loop {
            let op = match self.peek() {
                Some(op @ ("+" | "-")) => op.to_string(),
                _ => return Ok(left),
            };
            self.pos += 1;
            let (a, b) = (int(&left)?, int(&self.product()?)?);
            left = number(if op == "+" { a.checked_add(b) } else { a.checked_sub(b) })?;
        }
    }

    fn product(&mut self) -> Result<Valued, String> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Some(op @ ("*" | "/" | "%")) => op.to_string(),
                _ => return Ok(left),
            };
            self.pos += 1;
            let (a, b) = (int(&left)?, int(&self.unary()?)?);
            left = number(match op.as_str() {
                "*" => a.checked_mul(b),
                "/" => a.checked_div_euclid(b),
                _ => a.checked_rem_euclid(b),
            })?;
        }
    }

    fn unary(&mut self) -> Result<Valued, String> {
        if self.eat("-") {
            return number(int(&self.unary()?)?.checked_neg());
        }
        if self.eat("not") || self.eat("!") {
            return Ok(boolean(!truth(&self.unary()?)?));
        }
        let mut value = self.atom()?;
        loop {
            if self.eat("[") {
                let index = self.or()?;
                if !self.eat("]") {
                    return Err("expected `]`".into());
                }
                value = at(&value, &index)?;
            } else if self.eat(".") {
                let field = self.tokens.get(self.pos).ok_or("expected a field after `.`")?.clone();
                self.pos += 1;
                value = match value.json.get(&field) {
                    Some(json) => Valued { json: json.clone(), labels: None },
                    None => return Err(format!("no field `{field}` here")),
                };
            } else {
                return Ok(value);
            }
        }
    }

    fn atom(&mut self) -> Result<Valued, String> {
        let token = self.peek().ok_or("the expression ends early")?.to_string();
        self.pos += 1;
        if token == "(" {
            let value = self.or()?;
            return if self.eat(")") { Ok(value) } else { Err("expected `)`".into()) };
        }
        if token == "len" && self.eat("(") {
            let value = self.or()?;
            if !self.eat(")") {
                return Err("expected `)`".into());
            }
            let n = value.json.as_array().ok_or("`len` needs a list")?.len();
            return number(Some(n as i128));
        }
        if token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return number(Some(token.parse().map_err(|_| format!("`{token}` is not a number"))?));
        }
        match token.as_str() {
            "true" | "false" => Ok(boolean(token == "true")),
            "None" => Ok(Valued { json: Json::Null, labels: None }),
            _ => self.names.get(&token).cloned().ok_or(format!("`{token}` is not in scope here")),
        }
    }
}

fn at(list: &Valued, index: &Valued) -> Result<Valued, String> {
    let items = list.json.as_array().ok_or("only a list can be indexed")?;
    let key = index.json.to_compact_string();
    let position = match list.labels.as_ref().and_then(Json::as_array) {
        Some(labels) => labels.iter().position(|l| l.to_compact_string() == key || shown(l) == key),
        None => usize::try_from(int(index)?).ok(),
    };
    let json = position.and_then(|p| items.get(p)).ok_or(format!("{key} is out of range"))?;
    Ok(Valued { json: json.clone(), labels: None })
}

fn int(value: &Valued) -> Result<i128, String> {
    match &value.json {
        Json::Num(n) | Json::Str(n) => n.parse().map_err(|_| format!("`{}` is not an integer", shown(&value.json))),
        other => Err(format!("`{}` is not an integer", shown(other))),
    }
}

fn truth(value: &Valued) -> Result<bool, String> {
    value.json.as_bool().ok_or(format!("`{}` is not a Bool", shown(&value.json)))
}

fn boolean(b: bool) -> Valued {
    Valued { json: Json::Bool(b), labels: None }
}

fn number(n: Option<i128>) -> Result<Valued, String> {
    let n = n.ok_or("the arithmetic overflows")?;
    Ok(Valued { json: Json::Num(n.to_string()), labels: None })
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

/// The input a launch names, `inputText` or an `input` file; with neither,
/// the case `cpl stress` last saved for the file, said in the note.
fn read_input(program: &str, args: Option<&Json>) -> Result<(Vec<u8>, Option<String>), String> {
    if let Some(text) = args.and_then(|a| a.get("inputText")).and_then(Json::as_str) {
        return Ok((text.as_bytes().to_vec(), None));
    }
    let named = args.and_then(|a| a.get("input")).and_then(Json::as_str).filter(|s| !s.is_empty());
    let (path, note) = match named {
        Some(path) => (std::path::PathBuf::from(path), None),
        None => match crate::trace::pinned_input(Path::new(program)) {
            Some(path) => {
                let note = format!("input: {} (the last case saved for this file)\n", path.display());
                (path, Some(note))
            }
            None => return Ok((Vec::new(), None)),
        },
    };
    std::fs::read(&path)
        .map(|bytes| (bytes, note))
        .map_err(|err| format!("cannot read input `{}`: {err}", path.display()))
}

fn describe_change(change: &Change) -> String {
    let was = change.before.as_deref().unwrap_or("the run had ended");
    let now = change.after.as_deref().unwrap_or("the run has ended");
    match &change.variable {
        Some(var) => format!("first change at line {}: `{var}`: {now} (before the edit: {was})", change.line),
        None => format!("first change at line {}: {now} (before the edit: {was})", change.line),
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub(crate) fn parse(input: &str) -> Result<Json, ()> {
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

    pub(crate) fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
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

    pub(crate) fn as_i64(&self) -> Option<i64> {
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
