//! The language server: protocol messages in, protocol messages out.
//!
//! `Server` owns the same [`Database`] the command line uses and holds no
//! analysis of its own, so the editor and `cpl check` cannot disagree.
//! Each message is handled in isolation: a panic becomes an error response
//! (or an internal-error diagnostic) and the server keeps running.

use crate::complete::{self, Cursor, StdLib};
use crate::outline::Outline;
use crate::rename;
use crate::uri::uri_of_path;
use cpl_diag::{Diagnostic, LineCol, LineIndex, Severity, Span};
use cpl_facade::today::{DeclKind, Ty};
use cpl_facade::Definition;
use cpl_facade::Session;
use cpl_json::{obj, Value};
#[cfg(not(target_arch = "wasm32"))]
use diagnostics::lint_config;
use diagnostics::reported;
use std::collections::{BTreeMap, HashMap};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

mod actions;
mod calls;
// How an editor runs `cpl`: the browser has no `cpl` to run.
#[cfg(not(target_arch = "wasm32"))]
mod command;
mod cost;
mod diagnostics;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod exec;
pub(crate) mod format;
#[cfg(not(target_arch = "wasm32"))]
mod import;
#[cfg(not(target_arch = "wasm32"))]
mod lens;
mod nav;
mod proven;
pub(crate) mod tokens;
pub(crate) use cost::count_text;
use cost::{fn_call_costs, steps_of};
use proven::Proven;

const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;
const INVALID_REQUEST: i32 = -32600;
const SERVER_NOT_INITIALIZED: i32 = -32002;

pub struct Server {
    pub(crate) db: Session,
    /// Open documents by URI, with their current text.
    docs: BTreeMap<String, String>,
    /// The standard library's declarations, for completion.
    std: StdLib,
    /// What the declared bounds prove about each open document's text.
    proven: HashMap<String, Arc<Proven>>,
    initialized: bool,
    /// The client asked for its documents formatted as they are saved
    /// (`initializationOptions.formatOnSave`), through `willSaveWaitUntil`.
    format_on_save: bool,
    /// The document whose lints and proofs are still to be published.
    owed: Option<String>,
    /// The client asked for run, stress and trace lenses
    /// (`initializationOptions.codeLens`).
    #[cfg(not(target_arch = "wasm32"))]
    lenses: bool,
    /// The client asked for a Replay lens above each solution fn
    /// (`initializationOptions.replayLenses`), shown while debugging.
    #[cfg(not(target_arch = "wasm32"))]
    replay: bool,
    /// The Replay lenses wait for `cpl/debugging` to say a `cp` debug
    /// session runs (`replayLenses: "debugging"`), and whether one does.
    #[cfg(not(target_arch = "wasm32"))]
    replay_debugging: bool,
    #[cfg(not(target_arch = "wasm32"))]
    debugging: bool,
    /// The workspace's folder, where Import a Problem lands by default.
    #[cfg(not(target_arch = "wasm32"))]
    root: Option<String>,
    /// The `cpl` commands `cpl/commands` completes, from the binary serving.
    #[cfg(not(target_arch = "wasm32"))]
    commands: Vec<&'static str>,
    /// Where messages the server sends unasked go (`window/showDocument`
    /// for the problems `cpl.import` creates): the transport writes them.
    #[cfg(not(target_arch = "wasm32"))]
    outbox: Option<std::sync::mpsc::Sender<Value>>,
    /// The `cpl` runs editors ask for and the views they show.
    #[cfg(not(target_arch = "wasm32"))]
    runs: exec::Runs,
    /// The semantic tokens last sent for each document, newest last, with
    /// their result ids: a client may hold any of the last few (VS Code asks
    /// again for a visible editor while another request is in flight).
    tokens: HashMap<String, Vec<(u64, Vec<u32>)>>,
    token_ids: u64,
    shutdown: bool,
    exit: Option<i32>,
}

impl Default for Server {
    fn default() -> Server {
        Server::new()
    }
}

impl Server {
    pub fn new() -> Server {
        Server {
            db: Session::default(),
            docs: BTreeMap::new(),
            std: StdLib::default(),
            proven: HashMap::new(),
            initialized: false,
            format_on_save: false,
            owed: None,
            #[cfg(not(target_arch = "wasm32"))]
            lenses: false,
            #[cfg(not(target_arch = "wasm32"))]
            replay: false,
            #[cfg(not(target_arch = "wasm32"))]
            replay_debugging: false,
            #[cfg(not(target_arch = "wasm32"))]
            debugging: false,
            #[cfg(not(target_arch = "wasm32"))]
            root: None,
            #[cfg(not(target_arch = "wasm32"))]
            commands: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            outbox: None,
            #[cfg(not(target_arch = "wasm32"))]
            runs: exec::Runs::new(),
            tokens: HashMap::new(),
            token_ids: 0,
            shutdown: false,
            exit: None,
        }
    }

    /// The server, completing these `cpl` command names (`cpl/commands`).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_commands(mut self, names: Vec<&'static str>) -> Server {
        self.commands = names;
        self
    }

    /// The server, sending the messages it sends unasked to `outbox`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_outbox(mut self, outbox: std::sync::mpsc::Sender<Value>) -> Server {
        self.outbox = Some(outbox);
        self
    }

    /// The server, running this `cpl` for `cpl.exec`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_cpl(mut self, cpl: std::path::PathBuf) -> Server {
        self.runs.cpl = cpl;
        self
    }

    /// Runs each `cpl.exec` on its own thread and hands it to `on_finish`
    /// when it ends, for the transport to pass to [`Server::finish`].
    #[cfg(not(target_arch = "wasm32"))]
    pub fn on_finish(&mut self, on_finish: exec::OnFinish) {
        self.runs.on_finish = Some(on_finish);
    }

    /// Answers from now on with the std `snapshot` is checked against
    /// (`Snapshot::with_std`); every answer is worked out again.
    pub fn use_std_of(&mut self, snapshot: &cpl_facade::Snapshot) {
        self.db = self.db.at(self.db.snapshot().with_std_of(snapshot));
        self.std = StdLib::of(snapshot);
        self.proven.clear();
    }

    /// The process exit code, once the client has sent `exit`.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit
    }

    /// Handles one message from the client and returns the messages to
    /// send back at once, in order; [`Server::idle`] has the rest.
    pub fn respond(&mut self, message: &Value) -> Vec<Value> {
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            // A response to a request of ours; the server sends none.
            return if id.is_null() {
                vec![error(Value::Null, INVALID_REQUEST, "message has no method")]
            } else {
                Vec::new()
            };
        };
        let is_request = !id.is_null();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.runs.request = id.clone();
        }
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.dispatch(method, message.get("params").unwrap_or(&Value::Null))
        }));
        match outcome {
            Ok(Ok(result)) => {
                let mut out = Vec::new();
                if is_request && !result.deferred {
                    out.push(obj([
                        ("jsonrpc", "2.0".into()),
                        ("id", id),
                        ("result", result.response),
                    ]));
                }
                out.extend(result.notifications);
                out
            }
            Ok(Err((code, text))) => {
                if is_request {
                    vec![error(id, code, &text)]
                } else {
                    Vec::new()
                }
            }
            Err(panic) => self.recover(
                method,
                message.get("params").unwrap_or(&Value::Null),
                id,
                panic_text(&*panic),
            ),
        }
    }

    /// After a panic: rebuild the database from the open documents, answer
    /// the request with an error, and tell the user through a diagnostic.
    fn recover(&mut self, method: &str, params: &Value, id: Value, what: String) -> Vec<Value> {
        let empty = cpl_facade::Snapshot::default().with_std_of(self.db.snapshot());
        self.db = Session::open(empty);
        for (uri, text) in &self.docs {
            self.db = self.db.with_file(uri, text.as_str());
        }
        let args: &[&dyn std::fmt::Display] = &[&method, &what];
        let text = cpl_diag::msgs::check::INTERNAL_ERROR.say(args);
        let mut out = Vec::new();
        if !id.is_null() {
            out.push(error(id, INTERNAL_ERROR, &text));
        }
        if let Some(uri) = params.at(&["textDocument", "uri"]).as_str() {
            let diag = cpl_diag::msgs::check::INTERNAL_ERROR
                .error(Span::point(0), args)
                .with_help(
                    "this is a bug in cpl, not in your code; please report it with this file",
                );
            let source = self.docs.get(uri).map_or("", String::as_str);
            let lines = LineIndex::new(source);
            out.push(publish(
                uri,
                vec![diagnostic_json(&diag, uri, source, &lines)],
            ));
        }
        out
    }

    fn dispatch(&mut self, method: &str, params: &Value) -> Result<Reply, (i32, String)> {
        if !self.initialized && !matches!(method, "initialize" | "exit" | "initialized") {
            return Err((
                SERVER_NOT_INITIALIZED,
                "the server has not been initialized".into(),
            ));
        }
        match method {
            "initialize" => {
                self.initialized = true;
                self.format_on_save =
                    params.at(&["initializationOptions", "formatOnSave"]) == &Value::Bool(true);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.lenses =
                        params.at(&["initializationOptions", "codeLens"]) == &Value::Bool(true);
                    let replay = params.at(&["initializationOptions", "replayLenses"]);
                    self.replay_debugging = replay.as_str() == Some("debugging");
                    self.replay = replay == &Value::Bool(true) || self.replay_debugging;
                    self.runs.trusted =
                        params.at(&["initializationOptions", "trusted"]) != &Value::Bool(false);
                    self.runs.panel_messages =
                        params.at(&["initializationOptions", "panel"]).as_str() == Some("message");
                    self.runs.lens_refresh =
                        params.at(&["capabilities", "workspace", "codeLens", "refreshSupport"])
                            == &Value::Bool(true);
                    let folder = params.at(&["workspaceFolders"]).as_array();
                    let folder = folder.and_then(|f| f.first()).map(|f| f.at(&["uri"]));
                    let root = folder.unwrap_or(params.at(&["rootUri"])).as_str();
                    self.root = root.and_then(path_of_uri);
                    self.runs.refresh =
                        params.at(&["capabilities", "workspace", "inlayHint", "refreshSupport"])
                            == &Value::Bool(true);
                }
                Ok(Reply::result(capabilities()))
            }
            "initialized" | "$/cancelRequest" | "$/setTrace" => Ok(Reply::default()),
            #[cfg(target_arch = "wasm32")]
            "textDocument/didSave" | "workspace/didChangeConfiguration" => Ok(Reply::default()),
            #[cfg(not(target_arch = "wasm32"))]
            "textDocument/didSave" => self.saved(params),
            #[cfg(not(target_arch = "wasm32"))]
            "workspace/didChangeConfiguration" => Ok(self.configured(params)),
            #[cfg(not(target_arch = "wasm32"))]
            "workspace/executeCommand" => self.execute(params),
            #[cfg(not(target_arch = "wasm32"))]
            "cpl/debugging" => Ok(self.debugging(params)),
            "shutdown" => {
                self.shutdown = true;
                Ok(Reply::default())
            }
            "exit" => {
                self.exit = Some(if self.shutdown { 0 } else { 1 });
                Ok(Reply::default())
            }
            "textDocument/didOpen" => {
                let uri = uri_of(params)?;
                let text = params
                    .at(&["textDocument", "text"])
                    .as_str()
                    .ok_or((INVALID_PARAMS, "didOpen without text".to_string()))?;
                self.docs.insert(uri.to_string(), text.to_string());
                self.db = self.db.with_file(uri, text);
                Ok(Reply::notify(self.first_diagnostics(uri)))
            }
            "textDocument/didChange" => {
                let uri = uri_of(params)?;
                let mut text = self
                    .docs
                    .get(uri)
                    .cloned()
                    .ok_or((INVALID_PARAMS, format!("`{uri}` is not open")))?;
                for change in params
                    .get("contentChanges")
                    .and_then(Value::as_array)
                    .unwrap_or(&[])
                {
                    apply_change(&mut text, change)?;
                }
                self.db = self.db.with_file(uri, text.as_str());
                self.docs.insert(uri.to_string(), text);
                #[cfg(not(target_arch = "wasm32"))]
                self.runs.edited(uri);
                Ok(Reply::notify(self.first_diagnostics(uri)))
            }
            "textDocument/didClose" => {
                let uri = uri_of(params)?;
                self.docs.remove(uri);
                self.proven.remove(uri);
                #[cfg(not(target_arch = "wasm32"))]
                self.runs.edited(uri);
                self.db = self.db.without_file(uri);
                Ok(Reply::notify(publish(uri, Vec::new())))
            }
            "textDocument/hover" => {
                let (uri, offset, analysis) = self.position(params)?;
                let proven = self.loop_hover(&uri, offset);
                let proven = proven.or_else(|| self.index_hover(&uri, offset, false));
                if let Some(hover) = proven.or_else(|| self.arith_hover(&uri, offset)) {
                    return Ok(Reply::result(obj([
                        (
                            "contents",
                            obj([
                                ("kind", "markdown".into()),
                                ("value", hover_markdown(&hover).into()),
                            ]),
                        ),
                        (
                            "range",
                            range_json(hover.span, &analysis.text, &analysis.lines),
                        ),
                    ])));
                }
                let hover = self.named_hover(&uri, &analysis, offset);
                let Some(mut hover) = hover else {
                    return Ok(Reply::default());
                };
                // R06.1.6: a name with several fns shows each.
                let word = analysis
                    .text
                    .get(hover.span.start as usize..hover.span.end as usize);
                let outline = crate::complete::outline_of(&analysis);
                let fns: Vec<&str> = (outline.entries.iter())
                    .filter(|e| Some(e.name.as_str()) == word && e.kind == crate::outline::Kind::Fn)
                    .map(|e| e.detail.as_str())
                    .collect();
                if fns.len() > 1 {
                    hover.text = fns.join("\n");
                }
                if let Some(def) = self.db.definition(&uri, offset) {
                    if def.path != uri {
                        let text = self.db.file_text(&def.path).unwrap_or_default();
                        let outline = self.std.outline_of(&def.path, text);
                        hover.text = offered_lines(&hover.text, &outline);
                    }
                }
                let declared = self
                    .db
                    .definition(&uri, offset)
                    .filter(|d| d.path == uri)
                    .map_or(hover.span.start, |d| d.span.start);
                let facts = self.proven_of(&uri, analysis.clone());
                let proven = facts
                    .ranges()
                    .iter()
                    .copied()
                    .find(|&(at, lo, hi)| at == declared && lo != hi)
                    .filter(|_| hover.typed == Some(cpl_facade::Typed::Int));
                if let Some((_, lo, hi)) = proven {
                    let (first, rest) = hover.text.split_once('\n').unwrap_or((&hover.text, ""));
                    let rest = if rest.is_empty() {
                        String::new()
                    } else {
                        format!("\n{rest}")
                    };
                    hover.text = format!("{first} in {lo}..={hi}{rest}");
                }
                if let Some(index) = self.index_hover(&uri, offset, true) {
                    hover.text = format!("{}\n  {}", hover.text, index.text);
                }
                let sized = hover.cost.as_deref().and_then(|cost| {
                    let size = facts
                        .sizes()
                        .iter()
                        .find(|s| s.receiver.1 + 1 == hover.span.start)?;
                    let steps = steps_of(cost, size.max as f64)?;
                    let receiver =
                        &facts.text()[size.receiver.0 as usize..size.receiver.1 as usize];
                    let named = if size.count.chars().all(|c| c.is_ascii_digit()) {
                        String::new()
                    } else {
                        format!("{} = ", size.count)
                    };
                    Some((
                        format!("cost: {cost}"),
                        format!(
                            "cost: {cost}, about {} steps: {receiver} has at most {named}{} items",
                            count_text(steps.ceil() as u128),
                            count_text(size.max)
                        ),
                    ))
                });
                if let Some((plain, sized)) = sized {
                    hover.text = hover.text.replacen(&plain, &sized, 1);
                }
                let proven = facts
                    .fns()
                    .iter()
                    .find(|f| f.name.0 == declared && hover.signature);
                if let Some(f) = proven {
                    let (calls, total) = fn_call_costs(&mut self.db, &uri, f, &facts);
                    let named = |count: Option<&str>| {
                        count
                            .filter(|c| !c.chars().all(|c| c.is_ascii_digit()))
                            .map_or(String::new(), |c| format!("{c} = "))
                    };
                    if let Some(steps) = f.steps.filter(|&s| s > 0 && total.is_none()) {
                        let rest = if calls.is_empty() {
                            "; methods and the stdlib not counted"
                        } else {
                            ""
                        };
                        hover.text += &format!(
                            "\n  loops: at most {}{} iterations per call, with the fns it calls{rest}",
                            named(f.count.as_deref()),
                            count_text(steps)
                        );
                    }
                    if !calls.is_empty() {
                        hover.text += &format!("\n  documented costs: {}", calls.join("; "));
                    }
                    if let Some(cost::Steps { steps, cost }) = total {
                        let cost = cost.map_or(String::new(), |c| format!("{c}, "));
                        hover.text += &format!(
                            "\n  estimated cost: {cost}about {} steps at max input",
                            count_text(steps.ceil() as u128)
                        );
                    }
                }
                Ok(Reply::result(obj([
                    (
                        "contents",
                        obj([
                            ("kind", "markdown".into()),
                            ("value", hover_markdown(&hover).into()),
                        ]),
                    ),
                    (
                        "range",
                        range_json(hover.span, &analysis.text, &analysis.lines),
                    ),
                ])))
            }
            "textDocument/completion" => {
                let (uri, offset, analysis) = self.position(params)?;
                let db = std::cell::RefCell::new(&mut self.db);
                let fit = |ty: &Ty, names: &[&str]| db.borrow_mut().method_fits(&uri, ty, names);
                let cursor = Cursor {
                    uri: &uri,
                    analysis: &analysis,
                    offset,
                    fit: Some(&fit),
                };
                let trigger = params
                    .at(&["context", "triggerCharacter"])
                    .as_str()
                    .and_then(|c| c.chars().next());
                let items = complete::completion(&cursor, &mut self.std, trigger);
                Ok(Reply::result(obj([
                    ("isIncomplete", false.into()),
                    ("items", Value::Array(items)),
                ])))
            }
            "completionItem/resolve" => {
                let mut item = params.clone();
                if let (Some(doc), Value::Object(members)) =
                    (complete::resolve(params, &mut self.std), &mut item)
                {
                    let doc = obj([("kind", "markdown".into()), ("value", doc.into())]);
                    members.insert("documentation".to_string(), doc);
                }
                Ok(Reply::result(item))
            }
            "textDocument/signatureHelp" => {
                let (uri, offset, analysis) = self.position(params)?;
                let mut cursor = Cursor {
                    uri: &uri,
                    analysis: &analysis,
                    offset,
                    fit: None,
                };
                let doc = complete::callee_start(&cursor)
                    .and_then(|at| self.named_hover(&uri, &analysis, at))
                    .and_then(|h| h.doc);
                let db = std::cell::RefCell::new(&mut self.db);
                let fit = |ty: &Ty, names: &[&str]| db.borrow_mut().method_fits(&uri, ty, names);
                cursor.fit = Some(&fit);
                Ok(
                    match complete::signature_help(&cursor, &mut self.std, doc) {
                        Some(help) => Reply::result(help),
                        None => Reply::default(),
                    },
                )
            }
            "textDocument/references" => {
                let (uri, offset, analysis) = self.position(params)?;
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let with_decl =
                    params.at(&["context", "includeDeclaration"]) != &Value::Bool(false);
                let locations = spans
                    .into_iter()
                    .filter(|span| with_decl || target.path != uri || *span != target.span)
                    .map(|span| {
                        obj([
                            ("uri", uri.as_str().into()),
                            ("range", range_json(span, &analysis.text, &analysis.lines)),
                        ])
                    })
                    .collect();
                Ok(Reply::result(Value::Array(locations)))
            }
            // The references in this file, the declaration marked as the
            // write: the same answer as find-references.
            "textDocument/documentHighlight" => {
                let (uri, offset, analysis) = self.position(params)?;
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let highlights = spans
                    .into_iter()
                    .map(|span| {
                        let declared = target.path == uri && span == target.span;
                        obj([
                            ("range", range_json(span, &analysis.text, &analysis.lines)),
                            ("kind", if declared { 3 } else { 1 }.into()),
                        ])
                    })
                    .collect();
                Ok(Reply::result(Value::Array(highlights)))
            }
            "textDocument/foldingRange" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let ranges = folding_ranges(&analysis.text)
                    .into_iter()
                    .map(|(start, end, kind)| {
                        let mut range = BTreeMap::from([
                            ("startLine".to_string(), start.into()),
                            ("endLine".to_string(), end.into()),
                        ]);
                        if let Some(kind) = kind {
                            range.insert("kind".to_string(), kind.into());
                        }
                        Value::Object(range)
                    })
                    .collect();
                Ok(Reply::result(Value::Array(ranges)))
            }
            // A name declared in an open file or a library of the user's can
            // be renamed; the standard library's names belong to it.
            "textDocument/prepareRename" => {
                let (uri, offset, analysis) = self.position(params)?;
                let Some((span, name)) = rename::word_at(&analysis.text, offset) else {
                    return Ok(Reply::default());
                };
                // `self` is the method's receiver, not a name to choose.
                if name == "self" {
                    return Ok(Reply::default());
                }
                let name = name.to_string();
                match self.db.definition(&uri, offset) {
                    Some(def) if rename::spelled_by_input(&mut self.db, &def, &[&uri]) => {
                        Err((INVALID_PARAMS, Self::spelled_by_input(&name)))
                    }
                    Some(def) if rename::std_calls(&mut self.db, &uri, &def, &name).is_some() => {
                        let by = rename::std_calls(&mut self.db, &uri, &def, &name);
                        Err((
                            INVALID_PARAMS,
                            Self::std_calls(&name, &by.unwrap_or_default()),
                        ))
                    }
                    Some(def) if self.renamable(&uri, &def) => Ok(Reply::result(obj([
                        ("range", range_json(span, &analysis.text, &analysis.lines)),
                        ("placeholder", name.into()),
                    ]))),
                    Some(_) => Err((
                        INVALID_PARAMS,
                        format!("`{name}` belongs to the standard library"),
                    )),
                    None => Ok(Reply::default()),
                }
            }
            "textDocument/rename" => {
                let (uri, offset, analysis) = self.position(params)?;
                let new_name = params
                    .get("newName")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if let Some(why) = rename::invalid_name(&new_name) {
                    return Err((INVALID_PARAMS, why));
                }
                let receiver = self.db.analyze(&uri).is_some_and(|a| {
                    rename::word_at(&a.text, offset).is_some_and(|(_, w)| w == "self")
                });
                if receiver {
                    return Err((
                        INVALID_PARAMS,
                        "`self` is the method's receiver and keeps its name".to_string(),
                    ));
                }
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::default());
                };
                if !self.renamable(&uri, &target) {
                    return Err((
                        INVALID_PARAMS,
                        "the name belongs to the standard library".to_string(),
                    ));
                }
                if rename::spelled_by_input(&mut self.db, &target, &[&uri]) {
                    let name = rename::word_at(&analysis.text, offset).map(|(_, w)| w.to_string());
                    return Err((
                        INVALID_PARAMS,
                        Self::spelled_by_input(&name.unwrap_or_default()),
                    ));
                }
                let name = rename::word_at(&analysis.text, offset)
                    .map_or(String::new(), |(_, w)| w.to_string());
                if let Some(by) = rename::std_calls(&mut self.db, &uri, &target, &name) {
                    return Err((INVALID_PARAMS, Self::std_calls(&name, &by)));
                }
                // Every open file, the declaring file, and each file of the
                // project on disk that imports the name.
                let mut files: Vec<String> = self.docs.keys().cloned().collect();
                let on_disk =
                    std::iter::once(target.path.clone()).chain(rename::project_files(&uri, &name));
                for path in on_disk {
                    if files.iter().any(|f| rename::same_file(f, &path)) {
                        continue;
                    }
                    if path != target.path && self.db.file_text(&path).is_none() {
                        let Ok(text) = std::fs::read_to_string(&path) else {
                            continue;
                        };
                        self.db = self.db.with_file(&path, text);
                    }
                    files.push(path);
                }
                let top_level = self
                    .db
                    .symbols(&target.path)
                    .iter()
                    .any(|s| s.name_span == target.span);
                let mut changes = BTreeMap::new();
                let mut planned = Vec::new();
                for file in files {
                    let spans = if file == uri {
                        spans.clone()
                    } else {
                        rename::uses_in(&mut self.db, &file, &target, &name)
                    };
                    let Some(analysis) = self.db.analyze(&file).filter(|_| !spans.is_empty())
                    else {
                        continue;
                    };
                    // A local may shadow a top-level name it doesn't use;
                    // `changed_meaning` refuses one that captures a use.
                    let taken = top_level
                        && (analysis.imports().contains_key(&new_name)
                            || self.db.symbols(&file).iter().any(|s| s.name == new_name));
                    if taken {
                        let shown = path_of_uri(&file).unwrap_or(file);
                        return Err((
                            INVALID_PARAMS,
                            format!("`{new_name}` is already a name in {shown}"),
                        ));
                    }
                    let labels = rename::shorthand_args(&mut self.db, &file);
                    let texts: Vec<(Span, String)> = spans
                        .into_iter()
                        .map(|span| rename::edit_at(span, &labels, &name, &new_name))
                        .collect();
                    planned.push((file.clone(), texts.clone()));
                    let edits: Vec<Value> = texts
                        .into_iter()
                        .map(|(span, text)| {
                            obj([
                                ("range", range_json(span, &analysis.text, &analysis.lines)),
                                ("newText", text.into()),
                            ])
                        })
                        .collect();
                    let file_uri = if self.docs.contains_key(&file) {
                        file
                    } else {
                        uri_of_path(&file)
                    };
                    changes.insert(file_uri, Value::Array(edits));
                }
                if let Some(why) = rename::changed_meaning(&mut self.db, &planned, &name, &new_name)
                {
                    return Err((INVALID_PARAMS, why));
                }
                Ok(Reply::result(obj([("changes", Value::Object(changes))])))
            }
            "textDocument/definition" => {
                let (uri, offset, _) = self.position(params)?;
                let defs = self.db.definitions(&uri, offset);
                Ok(match defs.as_slice() {
                    [] => Reply::default(),
                    [def] => self
                        .location(&uri, def)
                        .map_or_else(Reply::default, Reply::result),
                    _ => Reply::result(Value::Array(
                        defs.iter().filter_map(|d| self.location(&uri, d)).collect(),
                    )),
                })
            }
            #[cfg(not(target_arch = "wasm32"))]
            "textDocument/codeLens" => self.code_lens(params),
            #[cfg(not(target_arch = "wasm32"))]
            "cpl/command" => Ok(Reply::result(command::command(params))),
            #[cfg(not(target_arch = "wasm32"))]
            "cpl/commands" => Ok(Reply::result(command::commands(&self.commands, params))),
            "workspace/symbol" => {
                let query = params
                    .get("query")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_lowercase();
                let mut paths: Vec<String> = self.docs.keys().cloned().collect();
                for uri in self.docs.keys() {
                    if let Some(analysis) = self.db.analyze(uri) {
                        paths.extend(analysis.modules().iter().map(|m| m.path.clone()));
                    }
                }
                let mut seen = std::collections::HashSet::new();
                paths.retain(|p| seen.insert(p.clone()));
                let mut items = Vec::new();
                for path in &paths {
                    let Some(analysis) = self.db.analyze(path) else {
                        continue;
                    };
                    let uri = if self.docs.contains_key(path) {
                        path.clone()
                    } else {
                        uri_of_path(path)
                    };
                    for s in self.db.symbols(path) {
                        let library = !self.docs.contains_key(path);
                        if s.kind == DeclKind::Impl
                            || (library && !s.public)
                            || !fuzzy(&query, &s.name.to_lowercase())
                        {
                            continue;
                        }
                        items.push(obj([
                            ("name", s.name.as_str().into()),
                            ("kind", symbol_kind(s.kind).into()),
                            (
                                "location",
                                obj([
                                    ("uri", uri.as_str().into()),
                                    (
                                        "range",
                                        range_json(s.name_span, &analysis.text, &analysis.lines),
                                    ),
                                ]),
                            ),
                        ]));
                    }
                }
                Ok(Reply::result(Value::Array(items)))
            }
            "textDocument/documentSymbol" => {
                let uri = uri_of(params)?;
                let symbols = self.db.symbols(uri);
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let items = symbols
                    .iter()
                    .map(|s| {
                        obj([
                            ("name", s.name.as_str().into()),
                            ("kind", symbol_kind(s.kind).into()),
                            (
                                "range",
                                range_json(s.span.to(s.name_span), &analysis.text, &analysis.lines),
                            ),
                            (
                                "selectionRange",
                                range_json(s.name_span, &analysis.text, &analysis.lines),
                            ),
                        ])
                    })
                    .collect();
                Ok(Reply::result(Value::Array(items)))
            }
            "textDocument/codeAction" => self.code_actions(params),
            // The type of each `let`, `var`, loop variable and lambda
            // parameter that is written without one, shown after its name
            // (an index space for a domain's), and the parameter each
            // literal argument fills.
            "textDocument/inlayHint" => {
                let uri = uri_of(params)?;
                #[cfg(not(target_arch = "wasm32"))]
                if self.runs.hints_off() {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                }
                let Some(facts) = self.proven(uri) else {
                    return Ok(Reply::result(Value::Array(self.run_hints(uri))));
                };
                let analysis = facts.analysis().clone();
                let mut hints = Vec::new();
                let proven: HashMap<u32, (i128, i128)> = facts
                    .ranges()
                    .iter()
                    .map(|&(at, lo, hi)| (at, (lo, hi)))
                    .collect();
                for item in &analysis.items {
                    for entry in &item.check.hovers {
                        let declares = entry.decl == Some(entry.span)
                            && matches!(entry.keyword, "let" | "var" | "for" | "param");
                        let at = item.offset + entry.span.end;
                        let typed =
                            entry.keyword != "for" && annotated(&analysis.text[at as usize..]);
                        if !declares || typed || entry.ty.has_vague() {
                            continue;
                        }
                        hints.push((
                            at,
                            obj([
                                (
                                    "position",
                                    position_json(analysis.lines.utf16(at, &analysis.text)),
                                ),
                                (
                                    "label",
                                    {
                                        let start = item.offset + entry.span.start;
                                        let ty_text = entry.ty.to_string();
                                        match proven.get(&start) {
                                            Some((lo, hi))
                                                if entry.keyword == "let"
                                                    && lo != hi
                                                    && entry.ty == cpl_facade::today::Ty::Int =>
                                            {
                                                format!(": Int in {lo}..={hi}")
                                            }
                                            _ => format!(": {ty_text}"),
                                        }
                                    }
                                    .into(),
                                ),
                                ("kind", 1.into()),
                            ]),
                        ));
                    }
                }
                for (at, label) in complete::parameter_hints(&analysis) {
                    hints.push((
                        at,
                        obj([
                            (
                                "position",
                                position_json(analysis.lines.utf16(at, &analysis.text)),
                            ),
                            ("label", label.into()),
                            ("kind", 2.into()),
                            ("paddingRight", true.into()),
                        ]),
                    ));
                }
                cpl_diag::sort_by_key(&mut hints, |(at, _)| u64::from(*at));
                let mut hints: Vec<Value> = hints.into_iter().map(|(_, h)| h).collect();
                hints.extend(self.run_hints(uri));
                Ok(Reply::result(Value::Array(hints)))
            }
            // `cpl fmt` on the open text, in the style of the nearest
            // `cp.toml`: one edit that replaces the file, or none when it is
            // already formatted or can't be.
            "textDocument/formatting" => {
                let uri = uri_of(params)?;
                Ok(Reply::result(Value::Array(self.format_edits(uri))))
            }
            // Format on save, when the client asked for it.
            "textDocument/willSaveWaitUntil" => {
                let uri = uri_of(params)?;
                let edits = match self.format_on_save {
                    true => self.format_edits(uri),
                    false => Vec::new(),
                };
                Ok(Reply::result(Value::Array(edits)))
            }
            #[cfg(test)]
            "cpl/testPanic" => panic!("deliberate test panic"),
            _ => self
                .navigate(method, params)
                .or_else(|| self.calls(method, params))
                .or_else(|| self.format_part(method, params))
                .or_else(|| self.tokens(method, params))
                .unwrap_or_else(|| Err((METHOD_NOT_FOUND, format!("`{method}` is not supported")))),
        }
    }

    /// The edit that formats `uri` as `cpl fmt` would, if it changes anything.
    fn format_edits(&mut self, uri: &str) -> Vec<Value> {
        let Some(analysis) = self.db.analyze(uri) else {
            return Vec::new();
        };
        let style = path_of_uri(uri)
            .map(|path| cpl_fmt::Style::discover(std::path::Path::new(&path)).0)
            .unwrap_or_default();
        match cpl_fmt::format_with(&analysis.text, &style) {
            Ok(formatted) if formatted != *analysis.text => {
                let whole = Span::new(0, analysis.text.len() as u32);
                vec![obj([
                    ("range", range_json(whole, &analysis.text, &analysis.lines)),
                    ("newText", formatted.into()),
                ])]
            }
            _ => Vec::new(),
        }
    }

    /// Why a variant the input spells keeps its name.
    fn std_calls(name: &str, by: &str) -> String {
        format!("`{name}` keeps its name: {by} calls it on your type")
    }

    fn spelled_by_input(name: &str) -> String {
        format!(
            "`{name}` is how the input spells this variant: renaming it changes what `read` accepts"
        )
    }

    /// The document and byte offset a position request refers to.
    /// Whether the name declared at `target` may be renamed from `uri`: it
    /// is declared in an open file or in a module that is not the standard
    /// library's, reached through `uri`'s imports.
    fn renamable(&mut self, uri: &str, target: &Definition) -> bool {
        if self.docs.keys().any(|d| rename::same_file(d, &target.path)) {
            return true;
        }
        let mut queue = vec![uri.to_string()];
        let mut seen = std::collections::HashSet::new();
        while let Some(path) = queue.pop() {
            let Some(analysis) = self.db.analyze(&path).filter(|_| seen.insert(path)) else {
                continue;
            };
            for module in analysis.modules() {
                if rename::same_file(&module.path, &target.path) {
                    return !module.name.starts_with("std.");
                }
                queue.push(module.path.clone());
            }
        }
        false
    }

    /// What the declared bounds prove about `uri`'s current text, computed
    /// once per edit and shared by every request until the next.
    fn proven(&mut self, uri: &str) -> Option<Arc<Proven>> {
        let analysis = self.db.analyze(uri)?;
        Some(self.proven_of(uri, analysis))
    }

    fn proven_of(&mut self, uri: &str, analysis: Arc<cpl_facade::today::Analysis>) -> Arc<Proven> {
        if let Some(facts) = self.proven.get(uri).filter(|f| {
            Arc::ptr_eq(f.analysis(), &analysis) || *f.analysis().text == *analysis.text
        }) {
            return facts.clone();
        }
        let facts = Arc::new(Proven::new(analysis));
        self.proven.insert(uri.to_string(), facts.clone());
        facts
    }

    /// What hover shows for the name at `offset`, before the facts proven about
    /// it; signature help takes its doc from here too.
    fn named_hover(
        &mut self,
        uri: &str,
        analysis: &cpl_facade::today::Analysis,
        offset: u32,
    ) -> Option<cpl_facade::Hover> {
        let checked = self.db.hover(uri, offset);
        // `j.jump(a, b, k)` beside a field `jump`: the call names the fn.
        let called_field = checked.as_ref().is_some_and(|h| {
            !matches!(h.typed, None | Some(cpl_facade::Typed::Fn)) && {
                let text = &analysis.text;
                let name = &text[h.span.start as usize..h.span.end as usize];
                text[..h.span.start as usize].ends_with('.')
                    && text[h.span.end as usize..].starts_with('(')
                    && h.text.starts_with(&format!("{name}: "))
            }
        });
        // `s.runs` names a library `fn runs(self: S, ..)`: the checker's
        // fact is the free form, so show the declaration as written,
        // as completion and signature help do.
        if called_field || checked.as_ref().is_none_or(|h| h.signature) {
            let db = std::cell::RefCell::new(&mut self.db);
            let fit = |ty: &Ty, names: &[&str]| db.borrow_mut().method_fits(uri, ty, names);
            let cursor = Cursor {
                uri,
                analysis,
                offset,
                fit: Some(&fit),
            };
            let member = complete::member_hover(&cursor, &mut self.std);
            match checked {
                Some(_) if called_field && member.is_some() => member,
                // The checker's cost lines stay when the doc writes none.
                Some(checked) => Some(match member {
                    Some(mut member) if member.cost.is_none() => {
                        for line in checked.text.lines().skip(1) {
                            member.text.push('\n');
                            member.text.push_str(line);
                        }
                        member.cost = checked.cost;
                        member
                    }
                    Some(member) => member,
                    None => checked,
                }),
                None => crate::syntax_hover::hover(analysis, offset).or(member),
            }
        } else {
            checked
        }
    }

    /// On the head of a `for` whose count the declared bounds prove: how
    /// many times its body runs, alone and with the loops around it.
    fn loop_hover(&mut self, uri: &str, offset: u32) -> Option<cpl_facade::Hover> {
        let facts = self.proven(uri)?;
        let proven = facts
            .loops()
            .iter()
            .find(|l| l.head.0 <= offset && offset < l.head.1)?;
        let named = |count: &str| {
            if count.chars().all(|c| c.is_ascii_digit()) {
                String::new()
            } else {
                format!("{count} = ")
            }
        };
        let mut text = format!(
            "for: runs at most {}{} times",
            named(&proven.count),
            count_text(proven.times)
        );
        match proven.total {
            Some(total) if total != proven.times => {
                let count = named(proven.total_count.as_deref().unwrap_or_default());
                text += &format!(
                    "\n  {count}{} times in all, with the loops around it",
                    count_text(total)
                );
            }
            None => text += "\n  inside a loop with no proven count",
            _ => {}
        }
        Some(cpl_facade::Hover {
            typed: None,
            signature: false,
            span: Span::new(proven.head.0, proven.head.1),
            text,
            doc: None,
            cost: None,
        })
    }

    /// On the brackets of an index the bounds prove in range, or (`on_index`)
    /// on the index itself: why it is in bounds.
    fn index_hover(&mut self, uri: &str, offset: u32, on_index: bool) -> Option<cpl_facade::Hover> {
        let analysis = self.db.analyze(uri)?;
        let inside = |(start, end): (u32, u32)| start <= offset && offset < end;
        let facts = self.proven(uri)?;
        let proven = facts
            .indexes()
            .iter()
            .find(|p| inside(p.access) && !inside(p.seq) && inside(p.index) == on_index);
        let proven = match proven {
            Some(p) => p.clone(),
            None => {
                let typed = self.db.typed_index(uri, offset, on_index)?;
                let span = |s: Span| (s.start, s.end);
                cpl_lint::ProvenIndex {
                    access: span(typed.access),
                    seq: span(typed.seq),
                    index: span(typed.index),
                    proof: typed.proof,
                }
            }
        };
        let (start, end) = if on_index {
            proven.index
        } else {
            proven.access
        };
        let access = analysis
            .text
            .get(proven.access.0 as usize..proven.access.1 as usize)?;
        let access = access.split_whitespace().collect::<Vec<_>>().join(" ");
        let text = match on_index {
            true => format!("in bounds of {access}: {}", proven.proof),
            false => format!("{access}: in bounds, {}", proven.proof),
        };
        Some(cpl_facade::Hover {
            typed: None,
            signature: false,
            span: Span { start, end },
            text,
            doc: None,
            cost: None,
        })
    }

    /// On a `+`, `-`, `*`, `/`, `%` or `**` whose operands the declared bounds prove: the
    /// range of its value, and whether it can leave `Int`.
    fn arith_hover(&mut self, uri: &str, offset: u32) -> Option<cpl_facade::Hover> {
        let analysis = self.db.analyze(uri)?;
        let facts = self.proven(uri)?;
        let proven = facts
            .exprs()
            .iter()
            .find(|e| e.op.0 <= offset && offset < e.op.1)?;
        let (start, end) = proven.expr;
        let source = analysis.text.get(start as usize..end as usize)?;
        let source = source.split_whitespace().collect::<Vec<_>>().join(" ");
        let int = i64::MAX as i128;
        let fits = match &proven.overflow {
            Some(inputs) => format!("\n  overflows Int when {inputs}"),
            None if proven.lo < -int - 1 || proven.hi > int => "\n  may overflow Int".to_string(),
            None => String::new(),
        };
        let value = match (proven.lo, proven.hi) {
            (lo, hi) if lo == hi => format!("= {lo}"),
            (lo, hi) => format!("in {lo}..={hi}"),
        };
        Some(cpl_facade::Hover {
            typed: None,
            signature: false,
            span: Span { start, end },
            text: format!("{source}: Int {value}{fits}"),
            doc: None,
            cost: None,
        })
    }

    /// The document, offset and analysis a positioned request is about.
    fn position(
        &mut self,
        params: &Value,
    ) -> Result<(String, u32, Arc<cpl_facade::today::Analysis>), (i32, String)> {
        let uri = uri_of(params)?;
        let position = position_of(params.get("position").unwrap_or(&Value::Null))?;
        let analysis = self
            .db
            .analyze(uri)
            .ok_or((INVALID_PARAMS, format!("`{uri}` is not open")))?;
        let offset = analysis.lines.offset_utf16(position, &analysis.text);
        Ok((uri.to_string(), offset, analysis))
    }
}

#[derive(Default)]
struct Reply {
    response: Value,
    notifications: Vec<Value>,
    /// Answered later, among the notifications of a run that ends.
    deferred: bool,
}

impl Reply {
    fn result(response: Value) -> Reply {
        Reply {
            response,
            ..Reply::default()
        }
    }

    fn notify(notification: Value) -> Reply {
        Reply {
            notifications: vec![notification],
            ..Reply::default()
        }
    }
}

fn capabilities() -> Value {
    let caps = obj([
        (
            "capabilities",
            obj([
                ("positionEncoding", "utf-16".into()),
                // Open/close notifications, incremental changes.
                (
                    "textDocumentSync",
                    obj([
                        ("openClose", true.into()),
                        ("change", 2.into()),
                        ("save", true.into()),
                        ("willSaveWaitUntil", true.into()),
                    ]),
                ),
                ("hoverProvider", true.into()),
                (
                    "completionProvider",
                    obj([
                        (
                            "triggerCharacters",
                            vec![
                                ".".into(),
                                "(".into(),
                                ",".into(),
                                "{".into(),
                                "@".into(),
                                "!".into(),
                            ]
                            .into(),
                        ),
                        ("resolveProvider", true.into()),
                    ]),
                ),
                (
                    "signatureHelpProvider",
                    obj([
                        ("triggerCharacters", vec!["(".into(), ",".into()].into()),
                        ("retriggerCharacters", vec![",".into()].into()),
                    ]),
                ),
                ("definitionProvider", true.into()),
                ("typeDefinitionProvider", true.into()),
                ("implementationProvider", true.into()),
                ("callHierarchyProvider", true.into()),
                (
                    "documentLinkProvider",
                    obj([("resolveProvider", false.into())]),
                ),
                ("referencesProvider", true.into()),
                ("renameProvider", obj([("prepareProvider", true.into())])),
                ("documentSymbolProvider", true.into()),
                ("workspaceSymbolProvider", true.into()),
                ("documentHighlightProvider", true.into()),
                ("foldingRangeProvider", true.into()),
                (
                    "semanticTokensProvider",
                    obj([
                        (
                            "legend",
                            obj([
                                (
                                    "tokenTypes",
                                    TOKEN_TYPES
                                        .iter()
                                        .map(|&t| t.into())
                                        .collect::<Vec<_>>()
                                        .into(),
                                ),
                                (
                                    "tokenModifiers",
                                    TOKEN_MODIFIERS
                                        .iter()
                                        .map(|&t| t.into())
                                        .collect::<Vec<_>>()
                                        .into(),
                                ),
                            ]),
                        ),
                        ("full", obj([("delta", true.into())])),
                    ]),
                ),
                ("documentFormattingProvider", true.into()),
                ("documentRangeFormattingProvider", true.into()),
                (
                    "documentOnTypeFormattingProvider",
                    obj([("firstTriggerCharacter", "\n".into())]),
                ),
                ("inlayHintProvider", true.into()),
                ("linkedEditingRangeProvider", true.into()),
                (
                    "codeActionProvider",
                    obj([(
                        "codeActionKinds",
                        vec![
                            "quickfix".into(),
                            "source.fixAll.cpl".into(),
                            "source.organizeImports".into(),
                        ]
                        .into(),
                    )]),
                ),
            ]),
        ),
        (
            "serverInfo",
            obj([
                ("name", "cpl".into()),
                ("version", env!("CARGO_PKG_VERSION").into()),
            ]),
        ),
    ]);
    #[cfg(not(target_arch = "wasm32"))]
    let caps = lens::advertise(caps);
    caps
}

fn error(id: Value, code: i32, message: &str) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("id", id),
        (
            "error",
            obj([
                ("code", Value::Int(i64::from(code))),
                ("message", message.into()),
            ]),
        ),
    ])
}

fn publish(uri: &str, diagnostics: Vec<Value>) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", "textDocument/publishDiagnostics".into()),
        (
            "params",
            obj([("uri", uri.into()), ("diagnostics", diagnostics.into())]),
        ),
    ])
}

const TOKEN_TYPES: [&str; 6] = [
    "type",
    "function",
    "method",
    "property",
    "variable",
    "parameter",
];
const TOKEN_MODIFIERS: [&str; 2] = ["declaration", "readonly"];

/// Each name the checker resolved, classified by what it names, in the
/// LSP's relative encoding: what highlighting by syntax can't know, such
/// as which names are types or domains and which bindings are `var`.
fn semantic_tokens(analysis: &cpl_facade::today::Analysis) -> Vec<u32> {
    let mut tokens: Vec<(Span, u32, u32)> = Vec::new();
    for item in &analysis.items {
        for entry in &item.check.hovers {
            let span = entry.span.shift(item.offset);
            let named = entry
                .name
                .starts_with(|c: char| c.is_alphabetic() || c == '_');
            if !named
                || analysis.text.get(span.start as usize..span.end as usize) != Some(&entry.name)
            {
                continue;
            }
            let callable = matches!(entry.ty, cpl_facade::today::Ty::Fn { .. });
            let kind = match entry.keyword {
                "type" | "domain" => 0,
                "fn" => 1,
                "field" if callable => 2,
                "field" => 3,
                "param" => 5,
                "let" | "var" | "for" | "const" => 4,
                "global" => match entry.ty {
                    cpl_facade::today::Ty::TypeName(_) | cpl_facade::today::Ty::Domain(_) => 0,
                    _ if callable => 1,
                    _ => 4,
                },
                _ => continue,
            };
            let mut modifiers = 0;
            if entry.decl == Some(entry.span) {
                modifiers |= 1;
            }
            if matches!(entry.keyword, "let" | "for" | "const" | "param") {
                modifiers |= 2;
            }
            tokens.push((span, kind, modifiers));
        }
    }
    cpl_diag::sort_by_key(&mut tokens, |(span, ..)| span.order());
    tokens.dedup_by_key(|(span, ..)| span.start);
    let (mut line, mut col) = (0, 0);
    let mut data = Vec::with_capacity(tokens.len() * 5);
    for (span, kind, modifiers) in tokens {
        let at = analysis.lines.utf16(span.start, &analysis.text);
        let end = analysis.lines.utf16(span.end, &analysis.text);
        if end.line != at.line {
            continue;
        }
        let delta_col = if at.line == line {
            at.col - col
        } else {
            at.col
        };
        data.extend([at.line - line, delta_col, end.col - at.col, kind, modifiers]);
        (line, col) = (at.line, at.col);
    }
    data
}

/// The lines that fold, as `(first, last, kind)`: each block under the line
/// that opens it (an indented body runs to its last non-blank line), and
/// each run of two or more comment or `use` lines.
fn folding_ranges(text: &str) -> Vec<(u32, u32, Option<&'static str>)> {
    let lines: Vec<&str> = text.lines().collect();
    let indent = |line: &str| line.len() - line.trim_start().len();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let mut last = i;
        for (j, below) in lines.iter().enumerate().skip(i + 1) {
            if below.trim().is_empty() {
                continue;
            }
            if indent(below) <= indent(line) {
                break;
            }
            last = j;
        }
        if last > i {
            out.push((i as u32, last as u32, None));
        }
    }
    let runs = [("comment", "//"), ("imports", "use ")];
    for (kind, prefix) in runs {
        let mut i = 0;
        while i < lines.len() {
            let starts = |line: &str| line.trim_start().starts_with(prefix);
            let first = i;
            while i < lines.len() && starts(lines[i]) {
                i += 1;
            }
            if i > first + 1 {
                out.push((first as u32, i as u32 - 1, Some(kind)));
            }
            i = i.max(first + 1);
        }
    }
    cpl_diag::sort_by_str_key(&mut out, |(.., kind)| kind.unwrap_or(""));
    cpl_diag::sort_by_key(&mut out, |&(first, last, _)| {
        u64::from(first) << 32 | u64::from(last)
    });
    out
}

/// Whether the `let` or `var` binding whose name ends where `rest` starts
/// has its type written: `n: Int`, or `(m, k): (Int, Int)` after any name in the tuple.
fn annotated(rest: &str) -> bool {
    rest.trim_start_matches(|c: char| {
        c.is_alphanumeric() || c == '_' || c == ',' || c == ')' || c.is_whitespace()
    })
    .starts_with(':')
}

/// A struct from another module, as hover shows it: what it offers its
/// users, without its internal fields and methods.
fn offered_lines(text: &str, outline: &Outline) -> String {
    let Some(name) = text
        .strip_prefix("struct ")
        .map(|rest| rest.split(['<', '\n', ' ']).next().unwrap_or(rest))
    else {
        return text.to_string();
    };
    let members: Vec<_> = outline.members_of(name).collect();
    text.lines()
        .filter(|line| {
            let Some(member) = line.strip_prefix("  ") else {
                return true;
            };
            // The member's name: its first identifier, after any `fn`.
            let word = cpl_syntax::lex(member)
                .tokens
                .into_iter()
                .find(|t| t.kind == cpl_syntax::TokenKind::Ident)
                .map(|t| &member[t.span.start as usize..t.span.end as usize]);
            members
                .iter()
                .find(|m| Some(m.name.as_str()) == word)
                .is_none_or(|m| outline.offers(name, m))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn hover_markdown(hover: &cpl_facade::Hover) -> String {
    let code = format!("```cp\n{}\n```", hover.text);
    match &hover.doc {
        Some(doc) => format!("{code}\n\n{}", cpl_doc::plain::text(doc)),
        None => code,
    }
}

fn uri_of(params: &Value) -> Result<&str, (i32, String)> {
    params
        .at(&["textDocument", "uri"])
        .as_str()
        .ok_or((INVALID_PARAMS, "missing textDocument.uri".to_string()))
}

fn position_of(value: &Value) -> Result<LineCol, (i32, String)> {
    match (
        value.get("line").and_then(Value::as_u32),
        value.get("character").and_then(Value::as_u32),
    ) {
        (Some(line), Some(col)) => Ok(LineCol { line, col }),
        _ => Err((INVALID_PARAMS, "missing or malformed position".to_string())),
    }
}

/// Applies one `contentChanges` entry: a ranged edit, or a full
/// replacement when it has no range.
fn apply_change(text: &mut String, change: &Value) -> Result<(), (i32, String)> {
    let new_text = change
        .get("text")
        .and_then(Value::as_str)
        .ok_or((INVALID_PARAMS, "content change without text".to_string()))?;
    let Some(range) = change.get("range").filter(|r| !r.is_null()) else {
        *text = new_text.to_string();
        return Ok(());
    };
    let lines = LineIndex::new(text);
    let start = lines.offset_utf16(
        position_of(range.get("start").unwrap_or(&Value::Null))?,
        text,
    ) as usize;
    let end =
        lines.offset_utf16(position_of(range.get("end").unwrap_or(&Value::Null))?, text) as usize;
    if start > end {
        return Err((
            INVALID_PARAMS,
            "content change range ends before it starts".to_string(),
        ));
    }
    let edit = (
        cpl_diag::Span::new(start as u32, end as u32),
        new_text.to_string(),
    );
    *text = cpl_diag::splice(text, vec![edit]).ok_or((
        INVALID_PARAMS,
        "content change range splits a character".to_string(),
    ))?;
    Ok(())
}

fn position_json(pos: LineCol) -> Value {
    obj([("line", pos.line.into()), ("character", pos.col.into())])
}

pub(crate) fn range_json(span: Span, text: &str, lines: &LineIndex) -> Value {
    obj([
        ("start", position_json(lines.utf16(span.start, text))),
        ("end", position_json(lines.utf16(span.end, text))),
    ])
}

fn diagnostic_json(diag: &Diagnostic, uri: &str, text: &str, lines: &LineIndex) -> Value {
    let mut message = diag.message.clone();
    for note in &diag.notes {
        message += &format!("\nnote: {note}");
    }
    if let Some(help) = &diag.help {
        message += &format!("\nhelp: {help}");
    }
    let severity = match diag.severity {
        Severity::Error => 1,
        Severity::Warning => 2,
        Severity::Note => 3,
    };
    let mut members = BTreeMap::from([
        ("range".to_string(), range_json(diag.span, text, lines)),
        ("severity".to_string(), Value::from(severity as u32)),
        ("code".to_string(), diag.code.0.into()),
        ("source".to_string(), "cpl".into()),
        ("message".to_string(), message.into()),
    ]);
    if let Some(href) = cpl_diag::registry::page(diag.code.0) {
        members.insert("codeDescription".to_string(), obj([("href", href.into())]));
    }
    if !diag.labels.is_empty() {
        let related: Vec<Value> = diag
            .labels
            .iter()
            .map(|label| {
                obj([
                    (
                        "location",
                        obj([
                            ("uri", uri.into()),
                            ("range", range_json(label.span, text, lines)),
                        ]),
                    ),
                    ("message", label.message.as_str().into()),
                ])
            })
            .collect();
        members.insert("relatedInformation".to_string(), related.into());
    }
    Value::Object(members)
}

/// What `cpl lint` reports for the document, under the `cp.toml` that
/// governs its file; each lint's fix is the one `cpl fix` applies.
///
/// Not in the playground: the lint rules would put its page over the
/// site bundle's size cap.
/// The buffer after `cpl fix`, if that changes it.
#[cfg(target_arch = "wasm32")]
fn fix_file(_db: &mut Session, _uri: &str, _text: &str) -> Option<String> {
    None
}

/// Fixed in the server's own database, which then gets the buffer back:
/// the client hasn't applied the edit yet.
#[cfg(not(target_arch = "wasm32"))]
fn fix_file(db: &mut Session, uri: &str, text: &str) -> Option<String> {
    let (fixed, applied, _) = crate::fix::document(db, uri, text, &lint_config(uri));
    *db = db.with_file(uri, text);
    (applied > 0).then_some(fixed)
}

/// The name or field path before the `.` that ends at `at`: `g[u]` is
/// not one, `self.xs` is.
fn receiver_before(text: &str, at: usize) -> Option<&str> {
    let head = text.get(..at)?.strip_suffix('.')?;
    let start = head
        .char_indices()
        .rev()
        .take_while(|&(_, c)| c.is_alphanumeric() || c == '_' || c == '.')
        .last()
        .map(|(i, _)| i)?;
    let path = &head[start..];
    (!path.starts_with('.') && !path.starts_with(|c: char| c.is_ascii_digit())).then_some(path)
}

/// The file a `file://` URI names, with `%XX` escapes decoded.
pub(crate) fn path_of_uri(uri: &str) -> Option<String> {
    uri.starts_with("file://")
        .then(|| cpl_facade::file_path(uri))
}

/// Whether `query`'s characters appear in `name` in order, as editors'
/// symbol pickers match: `fw` finds `Fenwick`.
fn fuzzy(query: &str, name: &str) -> bool {
    let mut rest = name.chars();
    query.chars().all(|q| rest.any(|c| c == q))
}

fn symbol_kind(kind: DeclKind) -> u32 {
    // LSP SymbolKind: Class 5, Enum 10, Interface 11, Function 12,
    // Variable 13, Object 19, EnumMember 22, Struct 23, TypeParameter 26.
    match kind {
        DeclKind::Value => 13,
        DeclKind::Fn => 12,
        DeclKind::Struct => 23,
        DeclKind::Enum => 10,
        DeclKind::Variant => 22,
        DeclKind::Domain | DeclKind::Alias => 26,
        DeclKind::Trait => 11,
        // Object: an impl groups the fns it gives a type.
        DeclKind::Impl => 19,
    }
}

fn panic_text(panic: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        text.to_string()
    } else if let Some(text) = panic.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic".to_string()
    }
}
