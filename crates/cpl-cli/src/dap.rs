//! Time-travel Debug Adapter Protocol (DAP) server (`cpl dap`) over stdio.
//!
//! Runs the `.cp` program once in `cpl-interp` with full frame and variable
//! snapshots, then serves standard DAP requests over `Content-Length`-framed
//! JSON on stdin/stdout — supporting breakpoints, forward stepping (`next`,
//! `stepIn`, `stepOut`, `continue`), backward time-travel (`stepBack`,
//! `reverseContinue`), call stacks, domain-labeled variables, hover/watch
//! evaluation, and exception stops on runtime traps.

use crate::{compile_side, render_trap, Side};
use cpl_diag::Shown;
use cpl_interp::trace::{
    first_change, first_change_in, string as json_str, Change, Recording, Replay, TraceConfig,
};
pub(crate) use cpl_json::Value as Json;
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
    while let Some(body) = cpl_lsp::transport::read_message(reader)? {
        let Ok(msg) = cpl_json::parse(&body) else {
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
    /// 0-based index in `recording` of the step it stands at.
    cursor: usize,
    /// The steps a recording keeps: a launch's `recordSteps`, or `MAX_STEPS`.
    max_steps: usize,
    /// The run's step that `recording` starts at: past 0 on a long run.
    base: usize,
    /// A run longer than one recording holds traces its other steps again from
    /// its checkpoints, a window at a time.
    pager: Option<Replay>,
    /// The recording was cut by the time limit or a cancel, not by its end.
    stopped: bool,
    /// How `recording` was traced.
    config: TraceConfig,
    /// 1-based line breakpoints for the launched file.
    breakpoints: HashSet<u32>,
    stop_on_entry: bool,
    launched: bool,
    brute: bool,
    /// What every recording of this session reads, restarts included.
    input: Vec<u8>,
    /// Where the latest restart first changed a value, against the run before it.
    change: Option<Change>,
    /// The file as `recording` ran it.
    source: String,
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
            max_steps: MAX_STEPS,
            base: 0,
            pager: None,
            stopped: false,
            config: TraceConfig::default(),
            breakpoints: HashSet::new(),
            stop_on_entry: true,
            launched: false,
            brute: false,
            input: Vec::new(),
            change: None,
            source: String::new(),
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
        cpl_lsp::transport::write_raw_message(writer, &out)
    }

    fn send_event(
        &mut self,
        writer: &mut dyn Write,
        event: &str,
        body_json: Option<&str>,
    ) -> io::Result<()> {
        let seq = self.next_seq();
        let mut out = format!("{{\"seq\":{seq},\"type\":\"event\",\"event\":");
        json_str(&mut out, event);
        if let Some(body) = body_json {
            out.push_str(",\"body\":");
            out.push_str(body);
        }
        out.push('}');
        cpl_lsp::transport::write_raw_message(writer, &out)
    }

    fn send_output(
        &mut self,
        writer: &mut dyn Write,
        category: &str,
        text: &str,
    ) -> io::Result<()> {
        let mut body = String::from("{\"category\":");
        json_str(&mut body, category);
        body.push_str(",\"output\":");
        json_str(&mut body, text);
        body.push('}');
        self.send_event(writer, "output", Some(&body))
    }

    fn send_stopped(
        &mut self,
        writer: &mut dyn Write,
        reason: &str,
        text: Option<&str>,
    ) -> io::Result<()> {
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
        self.send_event(
            writer,
            "exited",
            Some(&format!("{{\"exitCode\":{exit_code}}}")),
        )?;
        self.send_event(writer, "terminated", Some("{}"))
    }

    fn current_depth(&self) -> usize {
        self.recording.depth(self.cursor)
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
                self.max_steps = args
                    .and_then(|a| a.get("recordSteps"))
                    .and_then(Json::as_u64)
                    .map_or(MAX_STEPS, |steps| steps.max(10) as usize);
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
                if let Some(list) = args
                    .and_then(|a| a.get("breakpoints"))
                    .and_then(Json::as_array)
                {
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
                if !self.launched || self.recording.is_empty() {
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
                let count = if let Some(step) = self.recording.step(self.cursor) {
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
                if let Some(step) = self.recording.step(self.cursor) {
                    let mut first = true;
                    if reference == 1 {
                        if let Ok(Json::Object(map)) = cpl_json::parse(&step.locals) {
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
                            let value = cpl_json::parse(val_json)
                                .map_or_else(|_| val_json.to_string(), |v| shown(&v));
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
                        let msg = format!(
                            "can't evaluate `{expr}` at step {}: {}",
                            self.base + self.cursor + 1,
                            self.eval_error(expr)
                        );
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
                let start_line = self.recording.line(self.cursor).unwrap_or(0);
                let mut i = self.cursor + 1;
                loop {
                    if i >= self.recording.len() {
                        if self.page_forward() {
                            i = 0;
                            continue;
                        }
                        self.cursor = self.recording.len().saturating_sub(1);
                        self.send_terminated(writer)?;
                        break;
                    }
                    let (line, d) = (self.recording.line(i).unwrap_or(0), self.recording.depth(i));
                    if self.is_trap(i) {
                        self.cursor = i;
                        let msg = self.trap_message.clone();
                        self.send_stopped(writer, "exception", msg.as_deref())?;
                        return Ok(false);
                    }
                    if self.breakpoints.contains(&line) {
                        self.cursor = i;
                        self.send_stopped(writer, "breakpoint", None)?;
                        return Ok(false);
                    }
                    if d <= depth && (line != start_line || self.is_last(i)) {
                        self.cursor = i;
                        self.send_stopped(writer, "step", None)?;
                        return Ok(false);
                    }
                    i += 1;
                }
            }
            "stepIn" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let moved = if self.cursor + 1 < self.recording.len() {
                    self.cursor += 1;
                    true
                } else {
                    self.page_forward()
                };
                if !moved {
                    self.send_terminated(writer)?;
                } else if self.is_trap(self.cursor) {
                    let msg = self.trap_message.clone();
                    self.send_stopped(writer, "exception", msg.as_deref())?;
                } else {
                    self.send_stopped(writer, "step", None)?;
                }
            }
            "stepOut" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let target_depth = self.current_depth().saturating_sub(1);
                let mut i = self.cursor + 1;
                loop {
                    if i >= self.recording.len() {
                        if self.page_forward() {
                            i = 0;
                            continue;
                        }
                        self.send_terminated(writer)?;
                        break;
                    }
                    if self.is_trap(i) {
                        self.cursor = i;
                        let msg = self.trap_message.clone();
                        self.send_stopped(writer, "exception", msg.as_deref())?;
                        break;
                    }
                    let at_breakpoint =
                        (self.breakpoints).contains(&self.recording.line(i).unwrap_or(0));
                    if self.recording.depth(i) <= target_depth || at_breakpoint {
                        self.cursor = i;
                        let reason = if at_breakpoint { "breakpoint" } else { "step" };
                        self.send_stopped(writer, reason, None)?;
                        break;
                    }
                    i += 1;
                }
            }
            "stepBack" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                if self.cursor > 0 {
                    self.cursor -= 1;
                } else {
                    self.page_back();
                }
                self.send_stopped(writer, "step", None)?;
            }
            "reverseContinue" => {
                self.send_response(writer, req_seq, command, true, None, None)?;
                let mut i = self.cursor;
                loop {
                    if i == 0 {
                        // With nothing to stop at on the way, only the first steps are traced again.
                        let earlier_change = matches!(&self.change, Some(c) if c.step < self.base);
                        if self.breakpoints.is_empty() && !earlier_change && self.goto(0) {
                            self.send_stopped(writer, "step", None)?;
                            break;
                        }
                        if self.page_back() {
                            i = self.recording.len();
                            continue;
                        }
                        self.cursor = 0;
                        self.send_stopped(writer, "step", None)?;
                        break;
                    }
                    i -= 1;
                    if let Some(text) = self.change_at(self.base + i) {
                        self.cursor = i;
                        self.send_stopped(writer, "divergence", Some(&text))?;
                        break;
                    }
                    if self
                        .breakpoints
                        .contains(&self.recording.line(i).unwrap_or(0))
                    {
                        self.cursor = i;
                        self.send_stopped(writer, "breakpoint", None)?;
                        break;
                    }
                }
            }
            // Records the file on disk again, on the same input, and stops
            // where the old run stood: the same visit to the same line.
            "restart" => {
                let config = args.and_then(|a| a.get("arguments"));
                if config.is_some_and(|c| c.get("input").is_some() || c.get("inputText").is_some())
                {
                    match read_input(&self.program_path, config) {
                        Ok((bytes, _)) => self.input = bytes,
                        Err(msg) => {
                            self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                            return Ok(false);
                        }
                    }
                }
                // On a long run the place counts visits from the window's first step.
                let (at, base) = (self.base + self.cursor, self.base);
                let place = self.recording.place(self.cursor);
                let before = std::mem::take(&mut self.recording);
                let (pager, config) = (self.pager.take(), self.config.clone());
                let was_stopped = self.stopped;
                if let Err(msg) = self.record(writer)? {
                    (self.recording, self.base, self.pager) = (before, base, pager);
                    self.stopped = was_stopped;
                    let msg = format!("{msg}\nkept the last good recording");
                    self.send_response(writer, req_seq, command, false, Some(&msg), None)?;
                    return Ok(false);
                }
                let before = match &pager {
                    Some(pager) if base > 0 => {
                        let first = TraceConfig {
                            from_step: Some(0),
                            max_steps: Some(self.page()),
                            ..config
                        };
                        pager.window(first).recording.unwrap_or_default()
                    }
                    _ => before,
                };
                // Two runs longer than what is kept, or a run the time limit
                // cut, compare over what both keep.
                let cut = was_stopped || self.stopped;
                let same_steps = (cut || before.truncated && self.recording.truncated)
                    .then(|| before.len().min(self.recording.len()));
                self.change = match same_steps {
                    Some(n) => first_change_in(&before, 0..n, &self.recording, 0..n),
                    None => first_change(&before, &self.recording),
                };
                if base == 0 {
                    self.cursor = place.map_or(0, |p| self.recording.find(p, at));
                } else if self.goto(base) {
                    self.cursor = place.map_or(0, |p| self.recording.find(p, at - base));
                } else {
                    self.cursor = self.recording.len().saturating_sub(1);
                }
                self.send_response(writer, req_seq, command, true, None, None)?;
                if self.recording.is_empty() {
                    self.send_terminated(writer)?;
                    return Ok(false);
                }
                let note = match &self.change {
                    None => match same_steps {
                        Some(steps) => format!(
                            "re-ran: every value and every print of the first {steps} steps is the same as before"
                        ),
                        None => "re-ran: every value and every print is the same as before".to_string(),
                    },
                    Some(change) => format!("re-ran: {}", describe_change(change)),
                };
                self.send_output(writer, "console", &format!("{note}\n"))?;
                let line = self.recording.line(self.cursor).unwrap_or(0);
                let kept = match place {
                    Some(_) if self.base > 0 => {
                        format!(
                            "kept your place: line {line}, step {}",
                            self.base + self.cursor + 1
                        )
                    }
                    Some(p) if p.line == line => {
                        format!("kept your place: line {line}, visit {}", p.visit)
                    }
                    _ => format!("nearest step to where you were: line {line}"),
                };
                self.send_stopped(writer, "goto", Some(&kept))?;
            }
            // Runs one recorded call again with the fn as the file has it
            // now: the frame's call at the current step, or with `fn`, the
            // call of that fn open there or else its latest before it.
            "replayCall" => {
                let frame = args
                    .and_then(|a| a.get("frameId"))
                    .and_then(Json::as_i64)
                    .map_or(0, |id| id.max(1) as usize - 1);
                let name = args
                    .and_then(|a| a.get("fn"))
                    .and_then(Json::as_str)
                    .map(str::to_string);
                let step = self.base + self.cursor;
                let pick = |recorded: &Recording| match &name {
                    None => crate::replay::call_at(recorded, step, frame),
                    Some(name) => {
                        let named = |c: &&cpl_interp::trace::Call| &c.name == name;
                        let open = recorded
                            .calls
                            .iter()
                            .filter(named)
                            .filter(|c| c.entry <= step && step < c.exit)
                            .max_by_key(|c| c.depth);
                        let done = || {
                            recorded
                                .calls
                                .iter()
                                .filter(named)
                                .rfind(|c| c.exit <= step)
                        };
                        let first = || recorded.calls.iter().find(named);
                        open.or_else(done)
                            .or_else(first)
                            .cloned()
                            .ok_or(format!("the run never calls `{name}`"))
                    }
                };
                let replayed = match self.brute {
                    true => {
                        Err("replay runs the solution: this session debugs the @brute".to_string())
                    }
                    false => crate::replay::replay_call(
                        &self.program_path,
                        &self.source,
                        None,
                        &self.input,
                        &pick,
                    ),
                };
                match replayed {
                    Ok(replay) => {
                        let report = crate::replay::report(&replay);
                        self.send_output(writer, "console", &report)?;
                        let mut body = String::from("{\"report\":");
                        json_str(&mut body, &report);
                        body.push_str(",\"call\":");
                        json_str(
                            &mut body,
                            &crate::replay::call_text(&replay.recorded.name, &replay.recorded.args),
                        );
                        body += &format!(
                            ",\"ordinal\":{},\"entry\":{}",
                            replay.recorded.ordinal,
                            replay.recorded.entry + 1
                        );
                        for (key, value) in [
                            ("returned", &replay.after.returned),
                            ("before", &replay.before.returned),
                            ("fallback", &replay.fallback),
                        ] {
                            if let Some(value) = value {
                                body += &format!(",\"{key}\":");
                                json_str(&mut body, value);
                            }
                        }
                        if let Some(change) = &replay.change {
                            body += &format!(",\"changeLine\":{}", change.line);
                        }
                        body.push('}');
                        self.send_response(writer, req_seq, command, true, None, Some(&body))?;
                    }
                    Err(msg) => {
                        self.send_response(writer, req_seq, command, false, Some(&msg), None)?
                    }
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

    fn change_at(&self, step: usize) -> Option<String> {
        self.change
            .as_ref()
            .filter(|c| c.step == step)
            .map(describe_change)
    }

    /// Compiles the file as it is on disk now and records one run of it on
    /// the session's input. On a compile error the old recording stays.
    fn record(&mut self, writer: &mut dyn Write) -> io::Result<Result<(), String>> {
        let side = if self.brute {
            Side::Brute
        } else {
            Side::Solution
        };
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
            max_steps: Some(self.max_steps),
            ..TraceConfig::default()
        };
        self.config = config.clone();
        let mut printed = Vec::new();
        let observed = cpl_interp::run_observed(
            &compiled.program,
            // A program that never ends still lets the editor step through it.
            cpl_interp::Io::fixed(&self.input)
                .until(cpl_interp::Stop::after(std::time::Duration::from_secs(10))),
            &mut printed,
            Some(config),
        );
        self.recording = observed.recording.unwrap_or_default();
        self.base = 0;
        self.source = compiled.text.clone();
        self.trap_message = observed
            .trap
            .as_ref()
            .map(|t| render_trap(&program, &compiled.text, t));
        // A run stopped from outside would not end at the same step again.
        let ended = !observed.trap.as_ref().is_some_and(|t| {
            [cpl_interp::OUT_OF_TIME, cpl_interp::CANCELLED].contains(&t.message.as_str())
        });
        self.stopped = !ended;
        self.pager = (self.recording.truncated && ended).then(|| {
            Replay::start(
                compiled.program.clone(),
                self.input.clone(),
                self.config.clone(),
            )
        });
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

    fn advance_continue(
        &mut self,
        writer: &mut dyn Write,
        include_current: bool,
    ) -> io::Result<()> {
        let mut start = if include_current {
            self.cursor
        } else {
            self.cursor + 1
        };
        let mut jumped = false;
        loop {
            for i in start..self.recording.len() {
                if self.is_trap(i) {
                    self.cursor = i;
                    let msg = self.trap_message.clone();
                    return self.send_stopped(writer, "exception", msg.as_deref());
                }
                if let Some(text) = self.change_at(self.base + i) {
                    self.cursor = i;
                    return self.send_stopped(writer, "divergence", Some(&text));
                }
                if self
                    .breakpoints
                    .contains(&self.recording.line(i).unwrap_or(0))
                {
                    self.cursor = i;
                    return self.send_stopped(writer, "breakpoint", None);
                }
            }
            // With nothing to stop at on the way, only the run's end is traced again.
            let later_change = matches!(&self.change, Some(c) if c.step >= self.base);
            let jump = !jumped && self.breakpoints.is_empty() && !later_change;
            jumped |= jump;
            let turned = if jump {
                self.page_last()
            } else {
                self.page_forward()
            };
            if !turned {
                break;
            }
            start = self.cursor;
        }
        self.cursor = self.recording.len().saturating_sub(1);
        self.send_terminated(writer)
    }

    /// Whether step `i` of `recording` is the run's last.
    fn is_last(&self, i: usize) -> bool {
        i + 1 == self.recording.len() && !(self.recording.truncated && self.pager.is_some())
    }

    /// Whether step `i` of `recording` is where the run trapped.
    fn is_trap(&self, i: usize) -> bool {
        self.trap_message.is_some() && self.is_last(i)
    }

    /// The steps of a longer run traced again at a time, from its checkpoints.
    fn page(&self) -> usize {
        self.max_steps / 10
    }

    /// Traces the run's steps `from..from + steps` again, as `recording`.
    fn load(&mut self, from: usize, steps: usize) -> bool {
        let Some(pager) = &self.pager else {
            return false;
        };
        let config = TraceConfig {
            from_step: Some(from),
            max_steps: Some(steps),
            ..self.config.clone()
        };
        match pager.window(config).recording {
            Some(rec) if !rec.is_empty() => {
                (self.base, self.recording) = (rec.skipped, rec);
                true
            }
            _ => false,
        }
    }

    /// Moves on to the steps after `recording`, standing at the first.
    fn page_forward(&mut self) -> bool {
        let from = self.base + self.recording.len();
        let turned = self.recording.truncated && self.load(from, self.page());
        if turned {
            self.cursor = 0;
        }
        turned
    }

    /// Moves back to the steps before `recording`, standing at the last.
    fn page_back(&mut self) -> bool {
        let from = self.base.saturating_sub(self.page());
        let turned = self.base > 0 && self.load(from, self.base - from);
        if turned {
            self.cursor = self.recording.len() - 1;
        }
        turned
    }

    /// Moves to the run's last steps, standing at the last.
    fn page_last(&mut self) -> bool {
        let Some(pager) = &self.pager else {
            return false;
        };
        let counted = pager.window(TraceConfig {
            from_step: Some(usize::MAX),
            ..self.config.clone()
        });
        let Some(steps) = counted.recording.map(|rec| rec.skipped) else {
            return false;
        };
        let turned = self.load(steps.saturating_sub(self.page()), self.page());
        if turned {
            self.cursor = self.recording.len() - 1;
        }
        turned
    }

    /// Moves to the run's step `at`, in a window that starts at it.
    fn goto(&mut self, at: usize) -> bool {
        let turned = self.load(at, self.page());
        if turned {
            self.cursor = 0;
        }
        turned
    }

    /// A watch or console expression over the recorded state at the cursor:
    /// names, integers, `a[i]` by the index's own labels, `a.field`, `len(a)`,
    /// arithmetic, comparisons, `and`, `or` and `not`.
    fn eval_expr(&self, expr: &str) -> Option<(String, String)> {
        let step = self.recording.step(self.cursor)?;
        if let Some(var) = self
            .recording
            .vars
            .iter()
            .find(|v| v.name == expr || v.id == expr)
        {
            let (_, value) = step
                .values
                .iter()
                .find(|(i, _)| self.recording.vars[*i].id == var.id)?;
            let value = cpl_json::parse(value).ok()?;
            return Some((shown(&value), var.ty.clone()));
        }
        self.evaluate(expr)
            .ok()
            .map(|v| (shown(&v.json), String::new()))
    }

    fn eval_error(&self, expr: &str) -> String {
        match self.evaluate(expr) {
            Err(why) => why,
            Ok(_) => "nothing is recorded here".into(),
        }
    }

    fn evaluate(&self, expr: &str) -> Result<Valued, String> {
        let step = self
            .recording
            .step(self.cursor)
            .ok_or("nothing is recorded here")?;
        let mut names = BTreeMap::new();
        for (index, value) in &step.values {
            let var = &self.recording.vars[*index];
            let labels = cpl_json::parse(&format!("{{\"_\":0{}}}", var.extra))
                .ok()
                .and_then(|extra| extra.get("labels").cloned());
            if let Ok(json) = cpl_json::parse(value) {
                names.insert(var.name.clone(), Valued { json, labels });
            }
        }
        if let Ok(Json::Object(map)) = cpl_json::parse(&step.locals) {
            for (name, json) in map {
                names.insert(name, Valued { json, labels: None });
            }
        }
        let tokens = tokens(expr)?;
        let mut eval = Eval {
            tokens: &tokens,
            pos: 0,
            names: &names,
        };
        let value = eval.or()?;
        match eval.tokens.get(eval.pos) {
            None => Ok(value),
            Some(token) => Err(format!("unexpected `{token}`")),
        }
    }
}

pub(crate) use cpl_interp::trace::shown;

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
        } else if i + 1 < chars.len()
            && ["==", "!=", "<=", ">=", "&&", "||"]
                .contains(&format!("{c}{}", chars[i + 1]).as_str())
        {
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
            left = number(if op == "+" {
                a.checked_add(b)
            } else {
                a.checked_sub(b)
            })?;
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
                let field = self
                    .tokens
                    .get(self.pos)
                    .ok_or("expected a field after `.`")?
                    .clone();
                self.pos += 1;
                value = match value.json.get(&field) {
                    Some(json) => Valued {
                        json: json.clone(),
                        labels: None,
                    },
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
            return if self.eat(")") {
                Ok(value)
            } else {
                Err("expected `)`".into())
            };
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
            return number(Some(
                token
                    .parse()
                    .map_err(|_| format!("`{token}` is not a number"))?,
            ));
        }
        match token.as_str() {
            "true" | "false" => Ok(boolean(token == "true")),
            "None" => Ok(Valued {
                json: Json::Null,
                labels: None,
            }),
            _ => self
                .names
                .get(&token)
                .cloned()
                .ok_or(format!("`{token}` is not in scope here")),
        }
    }
}

fn at(list: &Valued, index: &Valued) -> Result<Valued, String> {
    let items = list.json.as_array().ok_or("only a list can be indexed")?;
    let key = index.json.to_string();
    let position = match list.labels.as_ref().and_then(Json::as_array) {
        Some(labels) => labels
            .iter()
            .position(|l| l.to_string() == key || shown(l) == key),
        None => usize::try_from(int(index)?).ok(),
    };
    let json = position
        .and_then(|p| items.get(p))
        .ok_or(format!("{key} is out of range"))?;
    Ok(Valued {
        json: json.clone(),
        labels: None,
    })
}

fn int(value: &Valued) -> Result<i128, String> {
    match &value.json {
        Json::Int(n) => Ok(i128::from(*n)),
        Json::String(n) => n
            .parse()
            .map_err(|_| format!("`{}` is not an integer", shown(&value.json))),
        other => Err(format!("`{}` is not an integer", shown(other))),
    }
}

fn truth(value: &Valued) -> Result<bool, String> {
    value
        .json
        .as_bool()
        .ok_or(format!("`{}` is not a Bool", shown(&value.json)))
}

fn boolean(b: bool) -> Valued {
    Valued {
        json: Json::Bool(b),
        labels: None,
    }
}

fn number(n: Option<i128>) -> Result<Valued, String> {
    let n = n.ok_or("the arithmetic overflows")?;
    let json = match i64::try_from(n) {
        Ok(v) => Json::Int(v),
        Err(_) => Json::String(n.to_string()),
    };
    Ok(Valued { json, labels: None })
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

/// The input a launch names, `inputText` or an `input` file; with neither,
/// the case `cpl stress` last saved for the file, said in the note.
fn read_input(program: &str, args: Option<&Json>) -> Result<(Vec<u8>, Option<String>), String> {
    if let Some(text) = args.and_then(|a| a.get("inputText")).and_then(Json::as_str) {
        return Ok((text.as_bytes().to_vec(), None));
    }
    let named = args
        .and_then(|a| a.get("input"))
        .and_then(Json::as_str)
        .filter(|s| !s.is_empty());
    let (path, note) = match named {
        Some(path) => (std::path::PathBuf::from(path), None),
        None => match crate::trace::pinned_input(Path::new(program)) {
            Some(path) => {
                let note = format!(
                    "input: {} (the last case saved for this file)\n",
                    path.shown()
                );
                (path, Some(note))
            }
            None => return Ok((Vec::new(), None)),
        },
    };
    std::fs::read(&path)
        .map(|bytes| (bytes, note))
        .map_err(|err| format!("cannot read input `{}`: {err}", path.shown()))
}

/// What a debug session records: a run of 10^7 steps fits in a few hundred MB.
const MAX_STEPS: usize = 10_000_000;

fn describe_change(change: &Change) -> String {
    let source = |json: &Option<String>, ended: &str| match json.as_deref().map(cpl_json::parse) {
        Some(Ok(json)) => shown(&json),
        Some(Err(_)) => json.clone().unwrap_or_default(),
        None => ended.to_string(),
    };
    let was = source(&change.before, "the run had ended");
    let now = source(&change.after, "the run has ended");
    match &change.variable {
        Some(var) => format!(
            "first change at line {}: `{var}` = {now} (before the edit: {was})",
            change.line
        ),
        None => format!(
            "first change at line {}: {now} (before the edit: {was})",
            change.line
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rerun_change_is_shown_as_the_source_writes_it() {
        let change = Change {
            step: 5,
            line: 11,
            variable: Some("best".into()),
            before: Some("[0,1,0]".into()),
            after: Some("[0,0,1]".into()),
        };
        assert_eq!(
            describe_change(&change),
            "first change at line 11: `best` = [0, 0, 1] (before the edit: [0, 1, 0])"
        );
    }
}
