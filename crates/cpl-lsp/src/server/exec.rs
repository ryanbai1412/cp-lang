//! `workspace/executeCommand`: the server runs the `cpl` arguments a lens or
//! an editor command names (`cpl.exec`) and shows the report's `view`
//! through the protocol itself, so an editor forwards the command and draws
//! nothing of its own: the panel as a `window/logMessage`, the status as a
//! `window/showMessage`, each line's mark and the status at the end of its
//! line as inlay hints, and the problem as a diagnostic. `cpl.show` shows a
//! view another tool made (the debugger's replay) and `cpl.clear` takes a
//! document's away; so does an edit, as the marks belong to the text that ran.
//! With `cp.liveValues.enabled` set, each save runs the samples quietly.

use super::*;
use cpl_diag::view::Kind;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use std::time::Duration;

/// The commands `workspace/executeCommand` runs.
pub(super) const COMMANDS: [&str; 5] = [
    "cpl.exec",
    "cpl.command",
    "cpl.show",
    "cpl.clear",
    "cpl.import",
];

/// Hands a finished run back to the transport's loop.
pub type OnFinish = Arc<dyn Fn(Finished) + Send + Sync>;

/// What the server keeps for running `cpl` and showing its reports.
pub(super) struct Runs {
    /// The `cpl` to run: the binary serving.
    pub(super) cpl: PathBuf,
    /// Where a run on its own thread reports; none runs it at once.
    pub(super) on_finish: Option<OnFinish>,
    /// The client's `cp` settings (`workspace/didChangeConfiguration`).
    pub(super) settings: Value,
    /// Each document's view, and whether a quiet run made it.
    views: HashMap<String, (Value, bool)>,
    /// Each document's edit count: a run that began before an edit is stale.
    edits: HashMap<String, u64>,
    /// The client can be told to ask for inlay hints, and code lenses, again.
    pub(super) refresh: bool,
    pub(super) lens_refresh: bool,
    /// The folder is trusted to run `cpl` (`initializationOptions.trusted`).
    pub(super) trusted: bool,
    /// A view's panel is shown as a message (`window/showMessage`), for a
    /// client with no panel of its own (`initializationOptions.panel`).
    pub(super) panel_messages: bool,
    requests: u64,
    /// The request being answered.
    pub(super) request: Value,
}

impl Runs {
    pub(super) fn new() -> Runs {
        Runs {
            cpl: std::env::current_exe().unwrap_or_else(|_| "cpl".into()),
            on_finish: None,
            settings: Value::Null,
            views: HashMap::new(),
            edits: HashMap::new(),
            refresh: false,
            lens_refresh: false,
            trusted: true,
            panel_messages: false,
            requests: 0,
            request: Value::Null,
        }
    }

    /// After an edit to, or the close of, `uri`.
    pub(super) fn edited(&mut self, uri: &str) {
        *self.edits.entry(uri.to_string()).or_default() += 1;
        self.views.remove(uri);
    }

    fn setting(&self, path: &[&str]) -> &Value {
        self.settings.at(path)
    }

    fn live(&self) -> bool {
        self.trusted && self.setting(&["liveValues", "enabled"]) == &Value::Bool(true)
    }

    /// `cp.inlayHints.enabled` is off.
    pub(super) fn hints_off(&self) -> bool {
        self.setting(&["inlayHints", "enabled"]) == &Value::Bool(false)
    }
}

/// A `cpl` run that has ended, for [`Server::finish`].
pub struct Finished {
    job: Job,
    stdout: String,
    stderr: String,
    code: Option<i32>,
}

struct Job {
    /// The request it answers; a quiet run answers none and says nothing.
    id: Option<Value>,
    uri: Option<String>,
    edits: u64,
    progress: Option<Value>,
    cpl: PathBuf,
    argv: Vec<String>,
    stdin: Option<String>,
    timeout: Duration,
}

impl Server {
    /// The client's settings; turning live values on runs every open
    /// document, and off takes their values away.
    pub(super) fn configured(&mut self, params: &Value) -> Reply {
        let settings = params.at(&["settings", "cp"]);
        if settings.as_object().is_none() {
            return Reply::default();
        }
        let was = self.runs.live();
        let hints_were_off = self.runs.hints_off();
        self.runs.settings = settings.clone();
        let mut out = Vec::new();
        if self.runs.refresh && hints_were_off != self.runs.hints_off() {
            out.push(self.ask("workspace/inlayHint/refresh"));
        }
        if was && !self.runs.live() {
            let quiet: Vec<String> = (self.runs.views.iter())
                .filter(|(_, (_, quiet))| *quiet)
                .map(|(uri, _)| uri.clone())
                .collect();
            for uri in quiet {
                self.runs.views.remove(&uri);
                out.extend(self.redraw(&uri));
            }
        } else if !was && self.runs.live() {
            let open: Vec<String> = self.docs.keys().cloned().collect();
            for uri in open {
                out.extend(self.live_values(&uri));
            }
        }
        Reply {
            notifications: out,
            ..Reply::default()
        }
    }

    /// After a save: the live values, when they are on.
    pub(super) fn saved(&mut self, params: &Value) -> Result<Reply, (i32, String)> {
        let uri = uri_of(params)?.to_string();
        let out = if self.runs.live() {
            self.live_values(&uri)
        } else {
            Vec::new()
        };
        Ok(Reply {
            notifications: out,
            ..Reply::default()
        })
    }

    pub(super) fn execute(&mut self, params: &Value) -> Result<Reply, (i32, String)> {
        let args = params.get("arguments").and_then(Value::as_array);
        let args = args.map(<[Value]>::to_vec).unwrap_or_default();
        let text = |i: usize| {
            args.get(i)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let shown = |notifications| Reply {
            notifications,
            ..Reply::default()
        };
        match params.get("command").and_then(Value::as_str) {
            // `:Cpl <args>` on a file: `cpl/command`'s arguments, then run.
            Some("cpl.command") => {
                let given = obj([
                    ("file", args.first().cloned().unwrap_or_default()),
                    ("arguments", args.get(1..).unwrap_or(&[]).to_vec().into()),
                ]);
                let argv = super::command::argv(&given).into_iter().map(Value::from);
                let mut run = params.clone();
                if let Some(run) = run.as_object_mut() {
                    run.insert("command".into(), "cpl.exec".into());
                    run.insert("arguments".into(), argv.collect::<Vec<_>>().into());
                }
                self.execute(&run)
            }
            Some("cpl.exec" | "cpl.import") if !self.runs.trusted => Err((
                INVALID_PARAMS,
                "This folder is in Restricted Mode, so CP Lang runs nothing: trust the folder to run `cpl`.".into(),
            )),
            Some("cpl.exec") => {
                let mut argv: Vec<String> = (args.iter())
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect();
                if argv.is_empty() {
                    return Err((INVALID_PARAMS, "`cpl.exec` takes cpl's arguments".into()));
                }
                // An object among the arguments gives the run's standard
                // input and expected output, and flags by name (`case`).
                // `stdin` may list choices, the first with text taken (a
                // selection, then the clipboard); `typed` is `[line]`, the
                // expected output typed with `\n` for each line break, or
                // `[null]` when the prompt was cancelled, which runs nothing.
                let given = args.iter().find(|a| a.as_object().is_some());
                let at = |key| given.and_then(|g| g.get(key)).and_then(Value::as_str);
                let typed = given.and_then(|g| g.get("typed")).and_then(Value::as_array);
                let typed = typed.map(|t| t.first().and_then(Value::as_str));
                if let Some(None) = typed {
                    return Ok(Reply::result(Value::Null));
                }
                let typed = typed.flatten().map(|t| t.replace("\\n", "\n"));
                for (flag, value) in given.and_then(Value::as_object).into_iter().flatten() {
                    let value = value.as_str().filter(|v| !v.is_empty());
                    if let (false, Some(value)) =
                        (matches!(flag.as_str(), "stdin" | "expect"), value)
                    {
                        argv.extend([format!("--{flag}"), value.into()]);
                    }
                }
                let expect = typed.as_deref().or(at("expect"));
                if let Some(expect) = expect.filter(|e| !e.is_empty()) {
                    let file = std::env::temp_dir().join(format!(
                        "cpl-expect-{}-{}",
                        std::process::id(),
                        self.runs.requests
                    ));
                    std::fs::write(&file, expect)
                        .map_err(|e| (INTERNAL_ERROR, format!("{}: {e}", file.display())))?;
                    argv.extend(["--expect".into(), file.display().to_string()]);
                }
                self.argue(&mut argv);
                let uri = self.uri_of_file(argv.get(1).map_or("", String::as_str));
                let id = Some(self.runs.request.clone());
                let progress = params.get("workDoneToken").cloned();
                let stdin = given.and_then(|g| g.get("stdin")).and_then(|s| match s.as_array() {
                    Some(choices) => (choices.iter().filter_map(Value::as_str))
                        .find(|c| !c.is_empty())
                        .map(String::from),
                    None => s.as_str().map(String::from),
                });
                let mut out = Vec::new();
                if let Some(token) = &progress {
                    let title = format!("cpl {}", argv.join(" "));
                    out.push(progress_of(
                        token,
                        obj([("kind", "begin".into()), ("title", title.into())]),
                    ));
                }
                out.extend(self.start(self.job(id, uri, argv, stdin, progress)));
                Ok(Reply {
                    notifications: out,
                    deferred: true,
                    ..Reply::default()
                })
            }
            Some("cpl.show") => {
                let view = args.get(1).cloned().unwrap_or_default();
                Ok(shown(self.show(Some(text(0)), view, false)))
            }
            Some("cpl.import") => self.import(&args).map(Reply::result),
            Some("cpl.clear") => {
                self.runs.views.remove(&text(0));
                Ok(shown(self.redraw(&text(0))))
            }
            other => Err((
                INVALID_PARAMS,
                format!("no command `{}`", other.unwrap_or("")),
            )),
        }
    }

    /// The arguments as the client's settings complete them: the report as
    /// JSON (a trace as its page), `cp.stress.args` after `stress`, `--copy`
    /// after `submit` unless `cp.submit.copy` is off, and the samples' live
    /// values of `cp.liveValues.input`.
    fn argue(&self, argv: &mut Vec<String>) {
        let has = |argv: &[String], flag: &str| argv.iter().any(|a| a == flag);
        let strings = |v: &Value| -> Vec<String> {
            (v.as_array().unwrap_or(&[]).iter())
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        };
        match argv[0].as_str() {
            "stress" => argv.extend(strings(self.runs.setting(&["stress", "args"]))),
            "submit" if self.runs.setting(&["submit", "copy"]) != &Value::Bool(false) => {
                argv.push("--copy".into())
            }
            "run" if has(argv, "--samples") && !has(argv, "--case") => {
                let input = self.runs.setting(&["liveValues", "input"]).as_str();
                if let Some(input) = input.filter(|i| !i.is_empty()) {
                    argv.extend(["--values-of".into(), input.into()]);
                }
            }
            _ => {}
        }
        if argv[0] == "submit" {
            argv.push("--json".into());
        } else if !has(argv, "--format") {
            let format = if has(argv, "--trace") { "html" } else { "json" };
            argv.extend(["--format".into(), format.into()]);
        }
    }

    /// The open document at `file`, a path as a lens spells it.
    fn uri_of_file(&self, file: &str) -> Option<String> {
        (self.docs.keys())
            .find(|uri| path_of_uri(uri).is_some_and(|p| crate::uri::native_path(&p) == file))
            .cloned()
    }

    fn job(
        &self,
        id: Option<Value>,
        uri: Option<String>,
        argv: Vec<String>,
        stdin: Option<String>,
        progress: Option<Value>,
    ) -> Job {
        let ms = self
            .runs
            .setting(&["run", "timeoutMs"])
            .as_i64()
            .unwrap_or(10_000);
        Job {
            id,
            edits: uri
                .as_ref()
                .and_then(|u| self.runs.edits.get(u))
                .copied()
                .unwrap_or(0),
            uri,
            progress,
            cpl: self.runs.cpl.clone(),
            argv,
            stdin,
            timeout: Duration::from_millis(ms.max(0) as u64),
        }
    }

    /// A quiet run of `uri`'s samples for its live values.
    fn live_values(&mut self, uri: &str) -> Vec<Value> {
        let Some(path) = path_of_uri(uri) else {
            return Vec::new();
        };
        let mut argv = vec![
            "run".into(),
            crate::uri::native_path(&path),
            "--samples".into(),
        ];
        self.argue(&mut argv);
        self.start(self.job(None, Some(uri.into()), argv, None, None))
    }

    fn start(&mut self, job: Job) -> Vec<Value> {
        self.runs.requests += 1;
        match self.runs.on_finish.clone() {
            Some(finish) => {
                std::thread::spawn(move || finish(run(job)));
                Vec::new()
            }
            None => self.finish(run(job)),
        }
    }

    /// The replies to a run that ended: its answer, and its view shown.
    pub fn finish(&mut self, done: Finished) -> Vec<Value> {
        let Finished {
            job,
            stdout,
            stderr,
            code,
        } = done;
        let page = stdout.trim_start().starts_with('<');
        let report = match page {
            true => Value::Null,
            false => cpl_json::parse(&stdout).unwrap_or_default(),
        };
        let view = match report.at(&["view"]) {
            v if v.as_object().is_some() => v.clone(),
            _ if page => obj([("panel", stderr.as_str().into())]),
            _ => obj([("panel", format!("{stdout}{stderr}").into())]),
        };
        let mut out = Vec::new();
        if let Some(token) = &job.progress {
            out.push(progress_of(token, obj([("kind", "end".into())])));
        }
        let quiet = job.id.is_none();
        if let Some(id) = job.id {
            let html = if page { stdout.into() } else { Value::Null };
            if page {
                out.push(obj([
                    ("jsonrpc", "2.0".into()),
                    ("method", "cpl/page".into()),
                    ("params", obj([("html", html.clone())])),
                ]));
            }
            let tests = view.at(&["tests"]);
            let teamcity = match tests.as_array() {
                Some(_) => cpl_contest::test::teamcity(tests, &tests_dir(&job.argv)).into(),
                None => Value::Null,
            };
            let result = obj([
                ("view", view.clone()),
                ("html", html),
                ("teamcity", teamcity),
                ("ok", (code == Some(0)).into()),
                ("code", code.map_or(Value::Null, |c| c.into())),
                ("stderr", stderr.into()),
            ]);
            out.push(obj([
                ("jsonrpc", "2.0".into()),
                ("id", id),
                ("result", result),
            ]));
        }
        let fresh = job.uri.filter(|uri| {
            self.docs.contains_key(uri)
                && self.runs.edits.get(uri).copied().unwrap_or(0) == job.edits
        });
        if !page {
            out.extend(self.show(fresh, view, quiet));
        }
        out
    }

    /// Shows `view`: its panel and status, unless `quiet`, and its marks on `uri`.
    fn show(&mut self, uri: Option<String>, view: Value, quiet: bool) -> Vec<Value> {
        let mut out = Vec::new();
        let panel = view.at(&["panel"]).as_str().unwrap_or("").trim_end();
        if !quiet && !panel.is_empty() {
            let method = match self.runs.panel_messages {
                true => "window/showMessage",
                false => "window/logMessage",
            };
            out.push(message(method, 3, panel));
        }
        let status = view.at(&["status"]);
        if let (false, Some(label)) = (quiet, status.at(&["label"]).as_str()) {
            let ok = status.at(&["ok"]) == &Value::Bool(true);
            out.push(message("window/showMessage", if ok { 3 } else { 2 }, label));
        }
        if let Some(uri) = uri {
            self.runs.views.insert(uri.clone(), (view, quiet));
            out.extend(self.redraw(&uri));
        }
        out
    }

    /// `uri`'s diagnostics again, and the client asked to fetch its hints.
    fn redraw(&mut self, uri: &str) -> Vec<Value> {
        let mut out = vec![self.diagnostics(uri)];
        if self.runs.refresh {
            out.push(self.ask("workspace/inlayHint/refresh"));
        }
        out
    }

    /// A request of the server's to the client, with no parameters.
    pub(super) fn ask(&mut self, method: &str) -> Value {
        self.runs.requests += 1;
        obj([
            ("jsonrpc", "2.0".into()),
            ("id", format!("cpl-{}", self.runs.requests).into()),
            ("method", method.into()),
        ])
    }

    /// The problem `uri`'s view reports, as a diagnostic to its line's end.
    pub(super) fn run_diagnostics(&self, uri: &str) -> Vec<Value> {
        let Some((view, _)) = self.runs.views.get(uri) else {
            return Vec::new();
        };
        let d = view.at(&["diagnostic"]);
        let (Some(line), Some(message)) = (d.at(&["line"]).as_i64(), d.at(&["message"]).as_str())
        else {
            return Vec::new();
        };
        let line = line as u32;
        let col = d.at(&["col"]).as_i64().unwrap_or(0) as u32;
        let message = match d.at(&["help"]).as_str() {
            Some(help) => format!("{message} ({help})"),
            None => message.to_string(),
        };
        let end = self.line_end(uri, line).max(col);
        let range = obj([
            ("start", position_json(LineCol { line, col })),
            ("end", position_json(LineCol { line, col: end })),
        ]);
        vec![obj([
            ("range", range),
            ("severity", 1.into()),
            ("source", d.at(&["source"]).as_str().unwrap_or("cpl").into()),
            ("code", d.at(&["code"]).clone()),
            ("message", message.into()),
        ])]
    }

    /// The marks of `uri`'s view as inlay hints at their lines' ends: the
    /// strongest kind on each line, and the status on a line with none,
    /// each clipped to `cp.liveValues.maxLength` with the full text on hover.
    pub(super) fn run_hints(&self, uri: &str) -> Vec<Value> {
        let Some((view, _)) = self.runs.views.get(uri) else {
            return Vec::new();
        };
        let mut marks: BTreeMap<u32, (usize, String, String)> = BTreeMap::new();
        for mark in view.at(&["lines"]).as_array().unwrap_or(&[]) {
            let (Some(line), Some(label)) =
                (mark.at(&["line"]).as_i64(), mark.at(&["label"]).as_str())
            else {
                continue;
            };
            let kind = mark.at(&["kind"]).as_str().unwrap_or("");
            let rank = Kind::ALL
                .iter()
                .position(|k| k.as_str() == kind)
                .unwrap_or(Kind::ALL.len());
            let hover = mark.at(&["hover"]).as_str().unwrap_or(label);
            let held = marks
                .get(&(line as u32))
                .is_some_and(|(r, _, _)| *r <= rank);
            if !held {
                marks.insert(line as u32, (rank, label.into(), hover.into()));
            }
        }
        let status = view.at(&["status"]);
        if let (Some(line), Some(label)) = (
            status.at(&["line"]).as_i64(),
            status.at(&["label"]).as_str(),
        ) {
            let mark = if status.at(&["ok"]) == &Value::Bool(true) {
                '\u{2713}'
            } else {
                '\u{2717}'
            };
            let hover = match status.at(&["detail"]).as_str() {
                Some(detail) => {
                    format!("{label}\n\n{}", cpl_diag::view::code_block(detail, "text"))
                }
                None => label.to_string(),
            };
            marks
                .entry(line as u32)
                .or_insert((usize::MAX, format!("{mark} {label}"), hover));
        }
        let max = self
            .runs
            .setting(&["liveValues", "maxLength"])
            .as_i64()
            .unwrap_or(60);
        (marks.into_iter())
            .map(|(line, (_, label, hover))| {
                let at = LineCol {
                    line,
                    col: self.line_end(uri, line),
                };
                obj([
                    ("position", position_json(at)),
                    ("label", clip(&label, max.max(1) as usize).into()),
                    ("paddingLeft", true.into()),
                    (
                        "tooltip",
                        obj([("kind", "markdown".into()), ("value", hover.into())]),
                    ),
                ])
            })
            .collect()
    }

    /// The UTF-16 length of `uri`'s 0-based `line`.
    fn line_end(&self, uri: &str, line: u32) -> u32 {
        let text = self.docs.get(uri).map(String::as_str).unwrap_or("");
        let line = text.lines().nth(line as usize).unwrap_or("");
        line.encode_utf16().count() as u32
    }
}

/// `text` on one line of at most `max` characters, ending in an ellipsis when cut.
pub(crate) fn clip(text: &str, max: usize) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() <= max {
        return line;
    }
    let kept: String = line.chars().take(max.saturating_sub(1).max(1)).collect();
    format!("{}\u{2026}", kept.trim_end())
}

fn message(method: &str, kind: i64, text: &str) -> Value {
    let params = obj([("type", kind.into()), ("message", text.into())]);
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", method.into()),
        ("params", params),
    ])
}

fn progress_of(token: &Value, value: Value) -> Value {
    let params = obj([("token", token.clone()), ("value", value)]);
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "$/progress".into()),
        ("params", params),
    ])
}

/// Runs `job`'s `cpl`, killing it after its timeout.
fn run(job: Job) -> Finished {
    let mut command = Command::new(&job.cpl);
    command.args(&job.argv);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(dir) = job
        .argv
        .get(1)
        .map(std::path::Path::new)
        .and_then(|f| f.parent())
    {
        if dir.is_dir() {
            command.current_dir(dir);
        }
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => {
            let stderr = format!("cpl lsp: can't run `{}`: {e}\n", job.cpl.display());
            return Finished {
                job,
                stdout: String::new(),
                stderr,
                code: None,
            };
        }
    };
    let input = job.stdin.clone().unwrap_or_default();
    let mut pipe = child.stdin.take();
    let fed = std::thread::spawn(move || {
        if let Some(pipe) = pipe.as_mut() {
            let _ = pipe.write_all(input.as_bytes());
        }
    });
    let (stdout, stderr) = (drain(child.stdout.take()), drain(child.stderr.take()));
    let mut exit = cpl_wait::on_exit(child);
    let status = exit.within(job.timeout).ok().flatten();
    if status.is_none() {
        let _ = exit.kill();
    }
    let _ = fed.join();
    let stdout = stdout.join().unwrap_or_default();
    let mut stderr = stderr.join().unwrap_or_default();
    if status.is_none() {
        stderr += &format!(
            "cpl lsp: stopped after {} ms (cp.run.timeoutMs)\n",
            job.timeout.as_millis()
        );
    }
    Finished {
        job,
        stdout,
        stderr,
        code: status.and_then(|s| s.code()),
    }
}

fn drain<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<String> {
    std::thread::spawn(move || {
        let mut text = String::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_string(&mut text);
        }
        text
    })
}

/// Where a `cpl test` run of `argv` reads its cases: `--tests`, else the
/// problem's own folder, which the `teamcity` messages locate each case in.
fn tests_dir(argv: &[String]) -> PathBuf {
    let given = argv.iter().position(|a| a == "--tests");
    match given.and_then(|i| argv.get(i + 1)) {
        Some(dir) => PathBuf::from(dir),
        None => {
            let file = argv.get(1).map_or("", String::as_str);
            cpl_contest::problem::find_tests_dir(&cpl_contest::problem::source_path(file))
        }
    }
}
