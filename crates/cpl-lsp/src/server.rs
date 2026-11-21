//! The language server: protocol messages in, protocol messages out.
//!
//! `Server` owns the same [`Database`] the command line uses and holds no
//! analysis of its own, so the editor and `cpl check` cannot disagree.
//! Each message is handled in isolation: a panic becomes an error response
//! (or an internal-error diagnostic) and the server keeps running.

use crate::complete::{self, Cursor, StdLib};
use crate::json::{obj, Value};
use crate::rename;
use cpl_check::{Database, DeclKind};
use cpl_diag::{codes, Diagnostic, LineCol, LineIndex, Severity, Span};
use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

const METHOD_NOT_FOUND: i32 = -32601;
const INVALID_PARAMS: i32 = -32602;
const INTERNAL_ERROR: i32 = -32603;
const INVALID_REQUEST: i32 = -32600;
const SERVER_NOT_INITIALIZED: i32 = -32002;

pub struct Server {
    db: Database,
    /// Open documents by URI, with their current text.
    docs: BTreeMap<String, String>,
    /// The standard library's declarations, for completion.
    std: StdLib,
    initialized: bool,
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
            db: Database::new(),
            docs: BTreeMap::new(),
            std: StdLib::default(),
            initialized: false,
            shutdown: false,
            exit: None,
        }
    }

    /// The process exit code, once the client has sent `exit`.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit
    }

    /// Handles one message from the client and returns the messages to
    /// send back, in order.
    pub fn handle(&mut self, message: &Value) -> Vec<Value> {
        let id = message.get("id").clone();
        let Some(method) = message.get("method").as_str() else {
            // A response to a request of ours; the server sends none.
            return if id.is_null() {
                vec![error(Value::Null, INVALID_REQUEST, "message has no method")]
            } else {
                Vec::new()
            };
        };
        let is_request = !id.is_null();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            self.dispatch(method, message.get("params"))
        }));
        match outcome {
            Ok(Ok(result)) => {
                let mut out = Vec::new();
                if is_request {
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
            Err(panic) => self.recover(method, message.get("params"), id, panic_text(&*panic)),
        }
    }

    /// After a panic: rebuild the database from the open documents, answer
    /// the request with an error, and tell the user through a diagnostic.
    fn recover(&mut self, method: &str, params: &Value, id: Value, what: String) -> Vec<Value> {
        self.db = Database::new();
        for (uri, text) in &self.docs {
            self.db.set_file(uri, text.as_str());
        }
        let text = format!("internal error in cpl while handling `{method}`: {what}");
        let mut out = Vec::new();
        if !id.is_null() {
            out.push(error(id, INTERNAL_ERROR, &text));
        }
        if let Some(uri) = params.at(&["textDocument", "uri"]).as_str() {
            let diag = Diagnostic::error(codes::INTERNAL, Span::point(0), text).with_help(
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
                Ok(Reply::result(capabilities()))
            }
            "initialized"
            | "$/cancelRequest"
            | "$/setTrace"
            | "textDocument/didSave"
            | "workspace/didChangeConfiguration" => Ok(Reply::default()),
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
                self.db.set_file(uri, text);
                Ok(Reply::notify(self.diagnostics(uri)))
            }
            "textDocument/didChange" => {
                let uri = uri_of(params)?;
                let mut text = self
                    .docs
                    .get(uri)
                    .cloned()
                    .ok_or((INVALID_PARAMS, format!("`{uri}` is not open")))?;
                for change in params.get("contentChanges").as_array() {
                    apply_change(&mut text, change)?;
                }
                self.db.set_file(uri, text.as_str());
                self.docs.insert(uri.to_string(), text);
                Ok(Reply::notify(self.diagnostics(uri)))
            }
            "textDocument/didClose" => {
                let uri = uri_of(params)?;
                self.docs.remove(uri);
                self.db.remove_file(uri);
                Ok(Reply::notify(publish(uri, Vec::new())))
            }
            "textDocument/hover" => {
                let (uri, offset) = self.position(params)?;
                let Some(hover) = self.db.hover(&uri, offset) else {
                    return Ok(Reply::default());
                };
                let analysis = self.db.analyze(&uri).expect("hover implies an analysis");
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
                let (uri, offset) = self.position(params)?;
                let analysis = self
                    .db
                    .analyze(&uri)
                    .expect("a position implies an analysis");
                let cursor = Cursor {
                    uri: &uri,
                    analysis: &analysis,
                    offset,
                };
                let items = complete::completion(&cursor, &mut self.std);
                Ok(Reply::result(obj([
                    ("isIncomplete", false.into()),
                    ("items", Value::Array(items)),
                ])))
            }
            "textDocument/signatureHelp" => {
                let (uri, offset) = self.position(params)?;
                let analysis = self
                    .db
                    .analyze(&uri)
                    .expect("a position implies an analysis");
                let cursor = Cursor {
                    uri: &uri,
                    analysis: &analysis,
                    offset,
                };
                Ok(match complete::signature_help(&cursor, &mut self.std) {
                    Some(help) => Reply::result(help),
                    None => Reply::default(),
                })
            }
            "textDocument/references" => {
                let (uri, offset) = self.position(params)?;
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let with_decl =
                    params.at(&["context", "includeDeclaration"]) != &Value::Bool(false);
                let analysis = self.db.analyze(&uri).expect("references imply an analysis");
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
                let (uri, offset) = self.position(params)?;
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let analysis = self.db.analyze(&uri).expect("references imply an analysis");
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
                        let mut range = vec![
                            ("startLine".to_string(), start.into()),
                            ("endLine".to_string(), end.into()),
                        ];
                        if let Some(kind) = kind {
                            range.push(("kind".to_string(), kind.into()));
                        }
                        Value::Object(range)
                    })
                    .collect();
                Ok(Reply::result(Value::Array(ranges)))
            }
            // Only a name declared in this file can be renamed: a library's
            // names belong to the library.
            "textDocument/prepareRename" => {
                let (uri, offset) = self.position(params)?;
                let analysis = self
                    .db
                    .analyze(&uri)
                    .expect("a position implies an analysis");
                let Some((span, name)) = rename::word_at(&analysis.text, offset) else {
                    return Ok(Reply::default());
                };
                let name = name.to_string();
                match self.db.definition_location(&uri, offset) {
                    Some(def) if def.path == uri => Ok(Reply::result(obj([
                        ("range", range_json(span, &analysis.text, &analysis.lines)),
                        ("placeholder", name.into()),
                    ]))),
                    Some(_) => Err((
                        INVALID_PARAMS,
                        format!("`{name}` is declared in another file"),
                    )),
                    None => Ok(Reply::default()),
                }
            }
            "textDocument/rename" => {
                let (uri, offset) = self.position(params)?;
                let new_name = params.get("newName").as_str().unwrap_or("").to_string();
                if let Some(why) = rename::invalid_name(&new_name) {
                    return Err((INVALID_PARAMS, why));
                }
                let Some((target, spans)) = rename::references(&mut self.db, &uri, offset) else {
                    return Ok(Reply::default());
                };
                if target.path != uri {
                    return Err((
                        INVALID_PARAMS,
                        "the name is declared in another file".to_string(),
                    ));
                }
                let analysis = self.db.analyze(&uri).expect("references imply an analysis");
                let edits: Vec<Value> = spans
                    .into_iter()
                    .map(|span| {
                        obj([
                            ("range", range_json(span, &analysis.text, &analysis.lines)),
                            ("newText", new_name.as_str().into()),
                        ])
                    })
                    .collect();
                Ok(Reply::result(obj([(
                    "changes",
                    Value::Object(vec![(uri.clone(), Value::Array(edits))]),
                )])))
            }
            "textDocument/definition" => {
                let (uri, offset) = self.position(params)?;
                let Some(def) = self.db.definition_location(&uri, offset) else {
                    return Ok(Reply::default());
                };
                let analysis = self
                    .db
                    .analyze(&def.path)
                    .expect("definition implies an analysis");
                let target_uri = if def.path.starts_with("file://") || def.path == uri {
                    def.path.clone()
                } else {
                    format!("file://{}", def.path)
                };
                Ok(Reply::result(obj([
                    ("uri", target_uri.into()),
                    (
                        "range",
                        range_json(def.span, &analysis.text, &analysis.lines),
                    ),
                ])))
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
            "textDocument/codeAction" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let start = position_of(params.at(&["range", "start"]))?;
                let end = position_of(params.at(&["range", "end"]))?;
                let wanted = Span::new(
                    analysis.lines.offset_utf16(start, &analysis.text),
                    analysis.lines.offset_utf16(end, &analysis.text),
                );
                let mut actions = Vec::new();
                for diag in &analysis.diagnostics {
                    if diag.span.end < wanted.start || diag.span.start > wanted.end {
                        continue;
                    }
                    for fix in &diag.fixes {
                        let edits: Vec<Value> = fix
                            .edits
                            .iter()
                            .map(|(span, text)| {
                                obj([
                                    ("range", range_json(*span, &analysis.text, &analysis.lines)),
                                    ("newText", text.as_str().into()),
                                ])
                            })
                            .collect();
                        actions.push(obj([
                            ("title", fix.title.as_str().into()),
                            ("kind", "quickfix".into()),
                            (
                                "diagnostics",
                                vec![diagnostic_json(diag, uri, &analysis.text, &analysis.lines)]
                                    .into(),
                            ),
                            (
                                "edit",
                                obj([(
                                    "changes",
                                    Value::Object(vec![(uri.to_string(), edits.into())]),
                                )]),
                            ),
                        ]));
                    }
                }
                Ok(Reply::result(Value::Array(actions)))
            }
            // The type of each `let`, `var` and loop variable that is written
            // without one, shown after its name, and the parameter each
            // literal argument fills.
            "textDocument/inlayHint" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let mut hints = Vec::new();
                for item in &analysis.items {
                    for entry in &item.check.hovers {
                        let declares = entry.decl == Some(entry.span)
                            && matches!(entry.keyword, "let" | "var" | "for");
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
                                ("label", format!(": {}", entry.ty).into()),
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
                hints.sort_by_key(|(at, _)| *at);
                Ok(Reply::result(Value::Array(
                    hints.into_iter().map(|(_, h)| h).collect(),
                )))
            }
            // `cpl fmt` on the open text: one edit that replaces the file,
            // or none when it is already formatted or can't be.
            "textDocument/formatting" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let edits = match cpl_fmt::format(&analysis.text) {
                    Ok(formatted) if formatted != *analysis.text => {
                        let whole = Span::new(0, analysis.text.len() as u32);
                        vec![obj([
                            ("range", range_json(whole, &analysis.text, &analysis.lines)),
                            ("newText", formatted.into()),
                        ])]
                    }
                    _ => Vec::new(),
                };
                Ok(Reply::result(Value::Array(edits)))
            }
            #[cfg(test)]
            "cpl/testPanic" => panic!("deliberate test panic"),
            _ => Err((METHOD_NOT_FOUND, format!("`{method}` is not supported"))),
        }
    }

    /// The `publishDiagnostics` notification for a document.
    fn diagnostics(&mut self, uri: &str) -> Value {
        let Some(analysis) = self.db.analyze(uri) else {
            return publish(uri, Vec::new());
        };
        // The same answer as `cpl check`: the checker's diagnostics, then
        // what lowering rejects.
        let items = cpl_ir::check(&analysis)
            .iter()
            .map(|d| diagnostic_json(d, uri, &analysis.text, &analysis.lines))
            .collect();
        publish(uri, items)
    }

    /// The document and byte offset a position request refers to.
    fn position(&mut self, params: &Value) -> Result<(String, u32), (i32, String)> {
        let uri = uri_of(params)?;
        let position = position_of(params.get("position"))?;
        let analysis = self
            .db
            .analyze(uri)
            .ok_or((INVALID_PARAMS, format!("`{uri}` is not open")))?;
        Ok((
            uri.to_string(),
            analysis.lines.offset_utf16(position, &analysis.text),
        ))
    }
}

#[derive(Default)]
struct Reply {
    response: Value,
    notifications: Vec<Value>,
}

impl Reply {
    fn result(response: Value) -> Reply {
        Reply {
            response,
            notifications: Vec::new(),
        }
    }

    fn notify(notification: Value) -> Reply {
        Reply {
            response: Value::Null,
            notifications: vec![notification],
        }
    }
}

fn capabilities() -> Value {
    obj([
        (
            "capabilities",
            obj([
                ("positionEncoding", "utf-16".into()),
                // Open/close notifications, incremental changes.
                (
                    "textDocumentSync",
                    obj([("openClose", true.into()), ("change", 2.into())]),
                ),
                ("hoverProvider", true.into()),
                (
                    "completionProvider",
                    obj([("triggerCharacters", vec![".".into()].into())]),
                ),
                (
                    "signatureHelpProvider",
                    obj([
                        ("triggerCharacters", vec!["(".into(), ",".into()].into()),
                        ("retriggerCharacters", vec![",".into()].into()),
                    ]),
                ),
                ("definitionProvider", true.into()),
                ("referencesProvider", true.into()),
                ("renameProvider", obj([("prepareProvider", true.into())])),
                ("documentSymbolProvider", true.into()),
                ("documentHighlightProvider", true.into()),
                ("foldingRangeProvider", true.into()),
                ("documentFormattingProvider", true.into()),
                ("inlayHintProvider", true.into()),
                (
                    "codeActionProvider",
                    obj([("codeActionKinds", vec!["quickfix".into()].into())]),
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
    ])
}

fn error(id: Value, code: i32, message: &str) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("id", id),
        (
            "error",
            obj([
                ("code", Value::Number(f64::from(code))),
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
    out.sort();
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

fn hover_markdown(hover: &cpl_check::Hover) -> String {
    let code = format!("```cp\n{}\n```", hover.text);
    match &hover.doc {
        Some(doc) => format!("{code}\n\n{doc}"),
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
    match (value.get("line").as_u32(), value.get("character").as_u32()) {
        (Some(line), Some(col)) => Ok(LineCol { line, col }),
        _ => Err((INVALID_PARAMS, "missing or malformed position".to_string())),
    }
}

/// Applies one `contentChanges` entry: a ranged edit, or a full
/// replacement when it has no range.
fn apply_change(text: &mut String, change: &Value) -> Result<(), (i32, String)> {
    let new_text = change
        .get("text")
        .as_str()
        .ok_or((INVALID_PARAMS, "content change without text".to_string()))?;
    let range = change.get("range");
    if range.is_null() {
        *text = new_text.to_string();
        return Ok(());
    }
    let lines = LineIndex::new(text);
    let start = lines.offset_utf16(position_of(range.get("start"))?, text) as usize;
    let end = lines.offset_utf16(position_of(range.get("end"))?, text) as usize;
    if start > end {
        return Err((
            INVALID_PARAMS,
            "content change range ends before it starts".to_string(),
        ));
    }
    text.replace_range(start..end, new_text);
    Ok(())
}

fn position_json(pos: LineCol) -> Value {
    obj([("line", pos.line.into()), ("character", pos.col.into())])
}

fn range_json(span: Span, text: &str, lines: &LineIndex) -> Value {
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
    let mut members = vec![
        ("range".to_string(), range_json(diag.span, text, lines)),
        ("severity".to_string(), Value::from(severity as u32)),
        ("code".to_string(), diag.code.0.into()),
        ("source".to_string(), "cpl".into()),
        ("message".to_string(), message.into()),
    ];
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
        members.push(("relatedInformation".to_string(), related.into()));
    }
    Value::Object(members)
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
