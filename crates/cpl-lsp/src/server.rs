//! The language server: protocol messages in, protocol messages out.
//!
//! `Server` owns the same [`Database`] the command line uses and holds no
//! analysis of its own, so the editor and `cpl check` cannot disagree.
//! Each message is handled in isolation: a panic becomes an error response
//! (or an internal-error diagnostic) and the server keeps running.

use crate::complete::{self, Cursor, StdLib};
use crate::json::{obj, Value};
use crate::rename;
use cpl_check::{Database, DeclKind, Definition};
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
    /// The client asked for run, stress and trace lenses
    /// (`initializationOptions.codeLens`); VS Code draws its own.
    lenses: bool,
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
            lenses: false,
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
                self.lenses =
                    params.at(&["initializationOptions", "codeLens"]) == &Value::Bool(true);
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
                let proven = self.loop_hover(&uri, offset);
                if let Some(hover) = proven.or_else(|| self.arith_hover(&uri, offset)) {
                    let analysis = self.db.analyze(&uri).expect("hover implies an analysis");
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
                let Some(mut hover) = self.db.hover(&uri, offset) else {
                    return Ok(Reply::default());
                };
                let declared = self
                    .db
                    .definition(&uri, offset)
                    .map_or(hover.span.start, |d| d.start);
                let analysis = self.db.analyze(&uri).expect("hover implies an analysis");
                let proven = cpl_lint::proven_ranges(&analysis.text)
                    .into_iter()
                    .find(|&(at, lo, hi)| at == declared && lo != hi)
                    .filter(|_| {
                        hover
                            .text
                            .lines()
                            .next()
                            .is_some_and(|l| l.ends_with(": Int"))
                    });
                if let Some((_, lo, hi)) = proven {
                    let (first, rest) = hover.text.split_once('\n').unwrap_or((&hover.text, ""));
                    let rest = if rest.is_empty() {
                        String::new()
                    } else {
                        format!("\n{rest}")
                    };
                    hover.text = format!("{first} in {lo}..={hi}{rest}");
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
            "textDocument/semanticTokens/full" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(obj([("data", Value::Array(Vec::new()))])));
                };
                Ok(Reply::result(obj([(
                    "data",
                    semantic_tokens(&analysis)
                        .into_iter()
                        .map(Value::from)
                        .collect::<Vec<_>>()
                        .into(),
                )])))
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
            // A name declared in an open file or a library of the user's can
            // be renamed; the standard library's names belong to it.
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
                let (uri, offset) = self.position(params)?;
                let new_name = params.get("newName").as_str().unwrap_or("").to_string();
                if let Some(why) = rename::invalid_name(&new_name) {
                    return Err((INVALID_PARAMS, why));
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
                let analysis = self.db.analyze(&uri).expect("references imply an analysis");
                let name = rename::word_at(&analysis.text, offset)
                    .map_or(String::new(), |(_, w)| w.to_string());
                // Every open file, the declaring file, and each file of the
                // project on disk that imports the name.
                let mut files: Vec<String> = self.docs.keys().cloned().collect();
                let on_disk = std::iter::once(target.path.clone())
                    .chain(rename::project_files(&uri, &name));
                for path in on_disk {
                    if files.iter().any(|f| rename::same_file(f, &path)) {
                        continue;
                    }
                    if path != target.path && self.db.file_text(&path).is_none() {
                        let Ok(text) = std::fs::read_to_string(&path) else { continue };
                        self.db.set_file(&path, text);
                    }
                    files.push(path);
                }
                let mut changes = Vec::new();
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
                    let taken = analysis.imports().contains_key(&new_name)
                        || self.db.symbols(&file).iter().any(|s| s.name == new_name);
                    if taken {
                        let shown = path_of_uri(&file).unwrap_or(file);
                        return Err((
                            INVALID_PARAMS,
                            format!("`{new_name}` is already a name in {shown}"),
                        ));
                    }
                    let edits: Vec<Value> = spans
                        .into_iter()
                        .map(|span| {
                            obj([
                                ("range", range_json(span, &analysis.text, &analysis.lines)),
                                ("newText", new_name.as_str().into()),
                            ])
                        })
                        .collect();
                    let file_uri = if self.docs.contains_key(&file) {
                        file
                    } else {
                        uri_of_path(&file)
                    };
                    changes.push((file_uri, Value::Array(edits)));
                }
                Ok(Reply::result(obj([("changes", Value::Object(changes))])))
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
                let target_uri = if def.path == uri {
                    def.path.clone()
                } else {
                    uri_of_path(&def.path)
                };
                Ok(Reply::result(obj([
                    ("uri", target_uri.into()),
                    (
                        "range",
                        range_json(def.span, &analysis.text, &analysis.lines),
                    ),
                ])))
            }
            "textDocument/codeLens" => {
                let uri = uri_of(params)?;
                let text = self.docs.get(uri).map(String::as_str).unwrap_or("");
                let lenses = match (self.lenses, path_of_uri(uri)) {
                    (true, Some(path)) => code_lenses(&path, text),
                    _ => Vec::new(),
                };
                let items = lenses
                    .into_iter()
                    .map(|(title, args)| {
                        obj([
                            (
                                "range",
                                obj([
                                    ("start", position_json(LineCol { line: 0, col: 0 })),
                                    ("end", position_json(LineCol { line: 0, col: 0 })),
                                ]),
                            ),
                            (
                                "command",
                                obj([
                                    ("title", title.into()),
                                    ("command", "cpl.exec".into()),
                                    (
                                        "arguments",
                                        Value::Array(args.into_iter().map(Value::from).collect()),
                                    ),
                                ]),
                            ),
                        ])
                    })
                    .collect();
                Ok(Reply::result(Value::Array(items)))
            }
            "workspace/symbol" => {
                let query = params.get("query").as_str().unwrap_or("").to_lowercase();
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
                let lints = lints(uri, &analysis.text);
                for diag in analysis.diagnostics.iter().chain(&lints) {
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
                // `cpl fix` on the document, for an editor's fix-all (on save).
                let only = params.at(&["context", "only"]).as_array();
                let fix_all = only.is_empty()
                    || only.iter().filter_map(Value::as_str).any(|kind| {
                        "source.fixAll".starts_with(kind) || kind.starts_with("source.fixAll")
                    });
                let fixed = fix_all.then(|| fix_file(uri, &analysis.text)).flatten();
                if let Some(text) = fixed {
                    let whole = Span::new(0, analysis.text.len() as u32);
                    actions.push(obj([
                        ("title", "apply every `cpl fix` fix".into()),
                        ("kind", "source.fixAll.cpl".into()),
                        (
                            "edit",
                            obj([(
                                "changes",
                                Value::Object(vec![(
                                    uri.to_string(),
                                    vec![obj([
                                        (
                                            "range",
                                            range_json(whole, &analysis.text, &analysis.lines),
                                        ),
                                        ("newText", text.into()),
                                    ])]
                                    .into(),
                                )]),
                            )]),
                        ),
                    ]));
                }
                Ok(Reply::result(Value::Array(actions)))
            }
            // The type of each `let`, `var`, loop variable and lambda
            // parameter that is written without one, shown after its name
            // (an index space for a domain's), and the parameter each
            // literal argument fills.
            "textDocument/inlayHint" => {
                let uri = uri_of(params)?;
                let Some(analysis) = self.db.analyze(uri) else {
                    return Ok(Reply::result(Value::Array(Vec::new())));
                };
                let mut hints = Vec::new();
                let proven: std::collections::HashMap<u32, (i128, i128)> =
                    cpl_lint::proven_ranges(&analysis.text)
                        .into_iter()
                        .map(|(at, lo, hi)| (at, (lo, hi)))
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
                                        match proven.get(&start) {
                                            Some((lo, hi))
                                                if entry.keyword == "let" && lo != hi =>
                                            {
                                                format!(": {} in {lo}..={hi}", entry.ty)
                                            }
                                            _ => format!(": {}", entry.ty),
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
        // what lowering rejects; then, as `cpl lint` reports them, the lints.
        let items = cpl_ir::check(&analysis)
            .iter()
            .chain(&lints(uri, &analysis.text))
            .chain(&overflows(&analysis.text))
            .map(|d| diagnostic_json(d, uri, &analysis.text, &analysis.lines))
            .collect();
        publish(uri, items)
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

    /// On the head of a `for` whose count the declared bounds prove: how
    /// many times its body runs, alone and with the loops around it.
    fn loop_hover(&mut self, uri: &str, offset: u32) -> Option<cpl_check::Hover> {
        let analysis = self.db.analyze(uri)?;
        let proven = cpl_lint::proven_loops(&analysis.text)
            .into_iter()
            .find(|l| l.head.0 <= offset && offset < l.head.1)?;
        let mut text = format!("for: runs at most {} times", count_text(proven.times));
        match proven.total {
            Some(total) if total != proven.times => {
                text += &format!("\n  {} times in all, with the loops around it", count_text(total));
            }
            None => text += "\n  inside a loop with no proven count",
            _ => {}
        }
        Some(cpl_check::Hover {
            span: Span::new(proven.head.0, proven.head.1),
            text,
            doc: None,
        })
    }

    /// On a `+`, `-`, `*`, `/`, `%` or `**` whose operands the declared bounds prove: the
    /// range of its value, and whether it can leave `Int`.
    fn arith_hover(&mut self, uri: &str, offset: u32) -> Option<cpl_check::Hover> {
        let analysis = self.db.analyze(uri)?;
        let proven = cpl_lint::proven_expr_ranges(&analysis.text)
            .into_iter()
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
        Some(cpl_check::Hover {
            span: Span { start, end },
            text: format!("{source}: Int in {}..={}{fits}", proven.lo, proven.hi),
            doc: None,
        })
    }

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
                ("workspaceSymbolProvider", true.into()),
                ("codeLensProvider", obj([("resolveProvider", false.into())])),
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
                        ("full", true.into()),
                    ]),
                ),
                ("documentFormattingProvider", true.into()),
                ("inlayHintProvider", true.into()),
                (
                    "codeActionProvider",
                    obj([(
                        "codeActionKinds",
                        vec!["quickfix".into(), "source.fixAll.cpl".into()].into(),
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
fn semantic_tokens(analysis: &cpl_check::Analysis) -> Vec<u32> {
    let mut tokens: Vec<(Span, u32, u32)> = Vec::new();
    for item in &analysis.items {
        for entry in &item.check.hovers {
            let span = entry.span.shift(item.offset);
            if analysis.text.get(span.start as usize..span.end as usize) != Some(&entry.name) {
                continue;
            }
            let callable = matches!(entry.ty, cpl_check::Ty::Fn { .. });
            let kind = match entry.keyword {
                "type" | "domain" => 0,
                "fn" => 1,
                "field" if callable => 2,
                "field" => 3,
                "param" => 5,
                "let" | "var" | "for" | "const" => 4,
                "global" => match entry.ty {
                    cpl_check::Ty::TypeName(_) | cpl_check::Ty::Domain(_) => 0,
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
    tokens.sort_by_key(|(span, ..)| (span.start, span.end));
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

/// What `cpl lint` reports for the document, under the `cp.toml` that
/// governs its file; each lint's fix is the one `cpl fix` applies.
///
/// Not in the playground: the lint rules would put its page over the
/// site bundle's size cap.
/// The buffer after `cpl fix`, if that changes it.
#[cfg(target_arch = "wasm32")]
fn fix_file(_uri: &str, _text: &str) -> Option<String> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn fix_file(uri: &str, text: &str) -> Option<String> {
    let path = path_of_uri(uri);
    let (fixed, applied) = cpl_lint::fix_at(
        path.as_deref().map(std::path::Path::new),
        text,
        &lint_config(uri),
    );
    (applied > 0).then_some(fixed)
}

/// `n`, and from 1e4 on its size too: `200000 (2e5)`, `249999500000 (2.5e11)`.
pub(crate) fn count_text(n: u128) -> String {
    if n < 10_000 {
        return n.to_string();
    }
    let digits = n.to_string();
    let exp = digits.len() - 1;
    let tenths = (n as f64 / 10f64.powi(exp as i32) * 10.0).round() as u64;
    let (tenths, exp) = if tenths >= 100 { (10, exp + 1) } else { (tenths, exp) };
    let mantissa = if tenths % 10 == 0 {
        (tenths / 10).to_string()
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    };
    format!("{n} ({mantissa}e{exp})")
}

/// A soft note on each operation the prover shows leaves `Int` for inputs
/// in their declared ranges, naming those inputs.
fn overflows(text: &str) -> Vec<Diagnostic> {
    cpl_lint::proven_expr_ranges(text)
        .into_iter()
        .filter_map(|e| {
            let inputs = e.overflow?;
            let span = Span::new(e.expr.0, e.expr.1);
            let source = text.get(e.expr.0 as usize..e.expr.1 as usize)?;
            let source = source.split_whitespace().collect::<Vec<_>>().join(" ");
            let mut diag = Diagnostic::warning(
                cpl_diag::Code("overflow"),
                span,
                format!("`{source}` overflows Int when {inputs}"),
            );
            diag.severity = cpl_diag::Severity::Note;
            Some(diag)
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
fn lints(_uri: &str, _text: &str) -> Vec<Diagnostic> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn lints(uri: &str, text: &str) -> Vec<Diagnostic> {
    cpl_lint::lint(text, &lint_config(uri))
}

#[cfg(not(target_arch = "wasm32"))]
fn lint_config(uri: &str) -> cpl_lint::Config {
    path_of_uri(uri)
        .map(|path| cpl_lint::Config::discover(std::path::Path::new(&path)).0)
        .unwrap_or_default()
}

/// The file a `file://` URI names, with `%XX` escapes decoded.
pub(crate) fn path_of_uri(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file://")?.as_bytes();
    let mut out = Vec::with_capacity(rest.len());
    let mut i = 0;
    while i < rest.len() {
        let hex = rest
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (rest[i], hex) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// The run, stress and trace lenses of the file at `path`, each a title and
/// the `cpl` arguments it runs: the same samples VS Code finds (`i1` beside
/// the file, `<stem>.tests/*.in`, `<stem>.samples/*.in`), the first of them
/// run and traced, and stress when the file has a brute, checker or interactor.
fn code_lenses(path: &str, text: &str) -> Vec<(String, Vec<String>)> {
    let file = std::path::Path::new(path);
    let (Some(folder), Some(stem)) = (file.parent(), file.file_stem().and_then(|s| s.to_str()))
    else {
        return Vec::new();
    };
    let names = |dir: &std::path::Path| -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        names.sort_by_key(|n| {
            let digits: String = n.chars().filter(char::is_ascii_digit).collect();
            (digits.parse::<u64>().unwrap_or(u64::MAX), n.clone())
        });
        names
    };
    let mut samples: Vec<(String, String)> = names(folder)
        .into_iter()
        .filter(|n| n.len() > 1 && n.starts_with('i') && n[1..].bytes().all(|b| b.is_ascii_digit()))
        .map(|n| (n.clone(), folder.join(&n).display().to_string()))
        .collect();
    for dir in [format!("{stem}.tests"), format!("{stem}.samples")] {
        let dir = folder.join(dir);
        for n in names(&dir) {
            if let Some(base) = n.strip_suffix(".in") {
                samples.push((base.to_string(), dir.join(&n).display().to_string()));
            }
        }
    }
    let has_tests = names(&folder.join(format!("{stem}.tests")))
        .iter()
        .any(|n| n.ends_with(".in"));
    let mut lenses = Vec::new();
    let mut with_input = |title: &str, mut args: Vec<String>| match samples.first() {
        Some((label, input)) => {
            args.insert(2, input.clone());
            lenses.push((format!("{title} on {label}"), args));
        }
        None => lenses.push((title.to_string(), args)),
    };
    with_input("Run", vec!["run".into(), path.into()]);
    with_input("Trace", vec!["run".into(), path.into(), "--trace".into()]);
    if has_tests {
        lenses.push(("Test".into(), vec!["test".into(), path.into()]));
    }
    if ["@brute", "@checker", "@interactor"]
        .iter()
        .any(|d| text.contains(d))
    {
        lenses.push(("Stress".into(), vec!["stress".into(), path.into()]));
    }
    lenses
}

fn uri_of_path(path: &str) -> String {
    if path.contains("://") {
        path.to_string()
    } else {
        format!("file://{path}")
    }
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
