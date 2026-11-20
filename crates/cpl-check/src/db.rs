//! The incremental database.
//!
//! One `Database` serves every tool: `cpl check`, `cpl run`, `cpl build`
//! and `cpl lsp` all call [`Database::analyze`], so they cannot disagree.
//! It is kept warm by the language server and thrown away after one use by
//! the command line; the code path is the same.
//!
//! # How work is reused
//!
//! A file is lexed whole (that is a few dozen microseconds) and cut into
//! top-level items. From there everything is per item and memoized:
//!
//! - **Syntax** is keyed by the item's text. Spans in a tree are relative
//!   to the item, so an item that only moved is not parsed again.
//! - **Definitions** (`Def`: what other items can see of a name) and
//!   **checks** (diagnostics and hover data for an item) record each
//!   top-level name they looked up, with a fingerprint of the `Def` they
//!   saw. A memo is reused when every fingerprint still matches.
//!
//! The second rule gives early cutoff. Editing a fn body changes that
//! item's text, so it is re-parsed and re-checked; but its `Def`, the
//! signature, has the same fingerprint, so no caller is touched. Editing
//! the signature re-checks exactly the items that looked the fn up.

use crate::check::{Checker, Env, GlobalRef, HoverEntry};
use crate::def::{hash_str, DeclKind, Def};
use cpl_diag::{codes, Diagnostic, LineIndex, Sink, Span};
use cpl_syntax::ast::{self, StmtKind};
use cpl_syntax::ParsedItem;
use std::collections::HashMap;
use std::sync::Arc;

/// Counters for how much work the database did, so tests can prove what
/// was reused.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Stats {
    pub files_lexed: u64,
    pub items_parsed: u64,
    pub items_checked: u64,
    pub items_reused: u64,
    pub defs_computed: u64,
}

/// One top-level item's syntax, shared by every revision in which its text
/// is unchanged.
#[derive(Debug)]
pub struct ItemSyntax {
    pub text: String,
    /// Column of the item's first token.
    pub col: u32,
    pub hash: u64,
    pub parsed: ParsedItem,
    pub decls: Vec<Decl>,
    pub has_use: bool,
}

/// A top-level name an item declares. The span is relative to the item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Decl {
    pub name: String,
    pub span: Span,
    pub kind: DeclKind,
}

/// The result of checking one item. Spans are relative to the item.
#[derive(Debug, Default)]
pub struct ItemCheck {
    pub diagnostics: Vec<Diagnostic>,
    pub hovers: Vec<HoverEntry>,
    pub refs: Vec<GlobalRef>,
}

#[derive(Debug)]
pub struct AnalyzedItem {
    /// Where the item starts in the file.
    pub offset: u32,
    pub span: Span,
    pub syntax: Arc<ItemSyntax>,
    pub check: Arc<ItemCheck>,
}

#[derive(Debug, Default)]
pub(crate) struct Scope {
    /// Name to the index of the item that defines it.
    pub names: HashMap<String, usize>,
    pub names_fp: u64,
    pub has_imports: bool,
}

/// Everything known about one revision of a file.
#[derive(Debug)]
pub struct Analysis {
    pub text: Arc<str>,
    pub lines: LineIndex,
    /// Every diagnostic, in file positions and source order.
    pub diagnostics: Vec<Diagnostic>,
    pub items: Vec<AnalyzedItem>,
    pub(crate) scope: Arc<Scope>,
}

impl Analysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == cpl_diag::Severity::Error)
    }

    /// The item at `offset`, with the offset relative to it.
    pub(crate) fn item_at(&self, offset: u32) -> Option<(&AnalyzedItem, u32)> {
        let at = self.items.partition_point(|item| item.span.end < offset);
        let item = self.items.get(at)?;
        item.span.touches(offset).then(|| (item, offset - item.offset))
    }

    /// Where a top-level name is declared, in file positions.
    pub(crate) fn decl_span(&self, name: &str) -> Option<Span> {
        let item = &self.items[*self.scope.names.get(name)?];
        let decl = item.syntax.decls.iter().find(|d| d.name == name)?;
        Some(decl.span.shift(item.offset))
    }
}

struct FileState {
    id: u32,
    text: Arc<str>,
    analysis: Option<Arc<Analysis>>,
}

struct SyntaxMemo {
    item: Arc<ItemSyntax>,
    last_used: u64,
}

struct DefMemo {
    def: Arc<Def>,
    fingerprint: u64,
    deps: Vec<(String, u64)>,
    verified_at: u64,
    last_used: u64,
}

struct CheckMemo {
    result: Arc<ItemCheck>,
    deps: Vec<(String, u64)>,
    verified_at: u64,
    last_used: u64,
}

/// Dependency keys that are not top-level names.
const DEP_NAMES: &str = "\0names";
const DEP_IMPORTS: &str = "\0imports";

/// Memos unused for this many revisions are dropped.
const KEEP_REVISIONS: u64 = 32;

#[derive(Default)]
pub struct Database {
    files: HashMap<String, FileState>,
    next_file: u32,
    revision: u64,
    syntax: HashMap<u64, SyntaxMemo>,
    defs: HashMap<(u32, u64, String), DefMemo>,
    checks: HashMap<(u32, u64), CheckMemo>,
    stats: Stats,
}

impl Database {
    pub fn new() -> Database {
        Database::default()
    }

    /// Sets the text of a file. Nothing is computed until it is asked for.
    pub fn set_file(&mut self, path: &str, text: impl Into<Arc<str>>) {
        let text = text.into();
        match self.files.get_mut(path) {
            Some(file) if file.text == text => {}
            Some(file) => {
                file.text = text;
                file.analysis = None;
            }
            None => {
                let id = self.next_file;
                self.next_file += 1;
                self.files.insert(
                    path.to_string(),
                    FileState {
                        id,
                        text,
                        analysis: None,
                    },
                );
            }
        }
    }

    pub fn remove_file(&mut self, path: &str) {
        if let Some(file) = self.files.remove(path) {
            self.defs.retain(|key, _| key.0 != file.id);
            self.checks.retain(|key, _| key.0 != file.id);
        }
    }

    pub fn file_text(&self, path: &str) -> Option<Arc<str>> {
        self.files.get(path).map(|f| f.text.clone())
    }

    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// Lexes, parses and checks a file, reusing everything an edit did not
    /// affect. Asking again without an edit returns the same `Analysis`.
    pub fn analyze(&mut self, path: &str) -> Option<Arc<Analysis>> {
        let file = self.files.get(path)?;
        if let Some(analysis) = &file.analysis {
            return Some(analysis.clone());
        }
        let (id, text) = (file.id, file.text.clone());
        self.revision += 1;
        let revision = self.revision;

        let lexed = cpl_syntax::lex(&text);
        self.stats.files_lexed += 1;
        let ranges = cpl_syntax::split_items(&lexed.tokens);
        let lines = LineIndex::new(&text);

        let mut items: Vec<Arc<ItemSyntax>> = Vec::with_capacity(ranges.len());
        for range in &ranges {
            let item_text = range.span.text(&text);
            let col = lines.line_col(range.span.start).col;
            let hash = hash_str(item_text) ^ u64::from(col).wrapping_mul(0x9e37_79b9_7f4a_7c15);
            let cached = self.syntax.get_mut(&hash).filter(|m| m.item.text == item_text && m.item.col == col);
            let item = match cached {
                Some(memo) => {
                    memo.last_used = revision;
                    memo.item.clone()
                }
                None => {
                    let tokens = &lexed.tokens[range.tokens.0..range.tokens.1];
                    let poison = cpl_syntax::poison_for_item(&lexed, range);
                    let parsed = cpl_syntax::parse_item(&text, tokens, range.span.start, poison);
                    self.stats.items_parsed += 1;
                    let item = Arc::new(ItemSyntax {
                        text: item_text.to_string(),
                        col,
                        hash,
                        decls: decls_of(&parsed.stmts),
                        has_use: parsed.stmts.iter().any(|s| matches!(s.kind, StmtKind::Use { .. })),
                        parsed,
                    });
                    self.syntax.insert(
                        hash,
                        SyntaxMemo {
                            item: item.clone(),
                            last_used: revision,
                        },
                    );
                    item
                }
            };
            items.push(item);
        }

        // The file's scope: the first declaration of a name wins, and later
        // ones are reported below.
        let mut scope = Scope::default();
        let mut duplicates: Vec<(usize, Decl)> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            scope.has_imports |= item.has_use;
            for decl in &item.decls {
                match scope.names.get(&decl.name) {
                    None => {
                        scope.names.insert(decl.name.clone(), index);
                    }
                    Some(_) if decl.kind == DeclKind::Variant => {}
                    Some(_) => duplicates.push((index, decl.clone())),
                }
            }
        }
        let mut sorted: Vec<&String> = scope.names.keys().collect();
        sorted.sort();
        scope.names_fp = hash_str(&sorted.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")) | 2;
        let scope = Arc::new(scope);
        let items = Arc::new(items);

        let mut ctx = FileCtx {
            db: self,
            file: id,
            items: items.clone(),
            scope: scope.clone(),
            frames: Vec::new(),
            in_progress: Vec::new(),
        };
        let checks: Vec<Arc<ItemCheck>> = (0..items.len()).map(|i| ctx.check_item(i)).collect();

        // Diagnostics, one per root cause: the lexer's first, then each
        // item's syntax errors, then what the checker found outside them.
        let mut sink = Sink::with_poisoned(lexed.poisoned.clone());
        for diag in &lexed.diagnostics {
            sink.push(diag.clone());
        }
        for ((range, item), check) in ranges.iter().zip(items.iter()).zip(&checks) {
            let offset = range.span.start;
            for diag in item.parsed.diagnostics.iter().chain(&check.diagnostics) {
                sink.push(diag.clone().shift(offset));
            }
        }
        for (index, decl) in duplicates {
            let offset = ranges[index].span.start;
            let first_index = scope.names[&decl.name];
            let first = items[first_index].decls.iter().find(|d| d.name == decl.name);
            let mut diag = Diagnostic::error(
                codes::DUPLICATE_DEFINITION,
                decl.span.shift(offset),
                format!("`{}` is already defined in this file", decl.name),
            )
            .with_help("rename one of them; a top-level name has one definition");
            if let Some(first) = first {
                diag = diag.with_label(first.span.shift(ranges[first_index].span.start), "first defined here");
            }
            sink.push(diag);
        }
        for (index, check) in checks.iter().enumerate() {
            let offset = ranges[index].span.start;
            for r in check.refs.iter().filter(|r| r.top_level) {
                let Some(&def_index) = scope.names.get(&r.name) else { continue };
                if def_index < index {
                    continue;
                }
                let Some(decl) = items[def_index].decls.iter().find(|d| d.name == r.name) else { continue };
                if decl.kind != DeclKind::Value {
                    continue;
                }
                let decl_span = decl.span.shift(ranges[def_index].span.start);
                // Within the declaring item only `let x = x + 1` is a use
                // before the name is set.
                if def_index == index && !inside_own_init(&items[index], &r.name, r.span) {
                    continue;
                }
                let line = lines.line_col(decl_span.start).line + 1;
                let message = if def_index == index {
                    format!("`{}` is used in its own definition", r.name)
                } else {
                    format!("`{}` is used here before it is set on line {line}", r.name)
                };
                let mut diag = Diagnostic::error(codes::USED_BEFORE_SET, r.span.shift(offset), message);
                if def_index != index {
                    diag = diag
                        .with_label(decl_span, "set here")
                        .with_help("top-level code runs in order; move this below the definition");
                }
                sink.push(diag);
            }
        }

        let analysis = Arc::new(Analysis {
            text: text.clone(),
            lines,
            diagnostics: sink.finish(),
            items: ranges
                .iter()
                .zip(items.iter())
                .zip(checks)
                .map(|((range, syntax), check)| AnalyzedItem {
                    offset: range.span.start,
                    span: range.span,
                    syntax: syntax.clone(),
                    check,
                })
                .collect(),
            scope,
        });
        self.collect_garbage(items.len());
        if let Some(file) = self.files.get_mut(path) {
            file.analysis = Some(analysis.clone());
        }
        Some(analysis)
    }

    /// Every diagnostic for a file, in source order.
    pub fn diagnostics(&mut self, path: &str) -> Vec<Diagnostic> {
        self.analyze(path).map(|a| a.diagnostics.clone()).unwrap_or_default()
    }

    /// The definition of a top-level name in an analyzed file.
    pub(crate) fn lookup(&mut self, path: &str, analysis: &Analysis, name: &str) -> Option<Arc<Def>> {
        let id = self.files.get(path)?.id;
        let items: Vec<Arc<ItemSyntax>> = analysis.items.iter().map(|i| i.syntax.clone()).collect();
        let mut ctx = FileCtx {
            db: self,
            file: id,
            items: Arc::new(items),
            scope: analysis.scope.clone(),
            frames: Vec::new(),
            in_progress: Vec::new(),
        };
        ctx.def(name).0
    }

    /// Drops memos that no recent revision used, once they outnumber the
    /// live ones several times over.
    fn collect_garbage(&mut self, live_items: usize) {
        let limit = 8 * live_items + 512;
        let oldest = self.revision.saturating_sub(KEEP_REVISIONS);
        if self.syntax.len() > limit {
            self.syntax.retain(|_, m| m.last_used >= oldest);
        }
        if self.checks.len() > limit {
            self.checks.retain(|_, m| m.last_used >= oldest);
        }
        if self.defs.len() > 2 * limit {
            self.defs.retain(|_, m| m.last_used >= oldest);
        }
    }

    #[cfg(test)]
    pub(crate) fn memo_counts(&self) -> (usize, usize, usize) {
        (self.syntax.len(), self.defs.len(), self.checks.len())
    }
}

/// True if `span` is inside the initializer of the top-level `let` that
/// declares `name` in `item`.
fn inside_own_init(item: &ItemSyntax, name: &str, span: Span) -> bool {
    item.parsed.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Let { pat, init: Some(init), .. } => init.span.contains(span) && pattern_binds(pat, name),
        StmtKind::Const { name: n, init: Some(init), .. } => init.span.contains(span) && n.text == name,
        _ => false,
    })
}

fn pattern_binds(pat: &ast::Pattern, name: &str) -> bool {
    match &pat.kind {
        ast::PatternKind::Name(n) => n == name,
        ast::PatternKind::Tuple(items) => items.iter().any(|i| pattern_binds(i, name)),
        _ => false,
    }
}

/// The top-level names a list of statements declares.
fn decls_of(stmts: &[ast::Stmt]) -> Vec<Decl> {
    fn pattern(pat: &ast::Pattern, out: &mut Vec<Decl>) {
        match &pat.kind {
            ast::PatternKind::Name(name) if name != "_" => out.push(Decl {
                name: name.clone(),
                span: pat.span,
                kind: DeclKind::Value,
            }),
            ast::PatternKind::Tuple(items) => items.iter().for_each(|i| pattern(i, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    let push = |out: &mut Vec<Decl>, name: &ast::Name, kind: DeclKind| {
        if !name.text.is_empty() {
            out.push(Decl {
                name: name.text.clone(),
                span: name.span,
                kind,
            });
        }
    };
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Let { pat, ty, .. } => {
                pattern(pat, &mut out);
                if let Some(ty) = ty {
                    crate::check::own_domains(ty, &mut |name| push(&mut out, name, DeclKind::Domain));
                }
            }
            StmtKind::Const { name, .. } => push(&mut out, name, DeclKind::Value),
            StmtKind::Fn(f) => push(&mut out, &f.name, DeclKind::Fn),
            StmtKind::Struct(s) => push(&mut out, &s.name, DeclKind::Struct),
            StmtKind::Enum(e) => {
                push(&mut out, &e.name, DeclKind::Enum);
                for variant in &e.variants {
                    push(&mut out, &variant.name, DeclKind::Variant);
                }
            }
            StmtKind::Domain { name, .. } => push(&mut out, name, DeclKind::Domain),
            StmtKind::TypeAlias { name, .. } => push(&mut out, name, DeclKind::Alias),
            StmtKind::Trait { name, .. } => push(&mut out, name, DeclKind::Trait),
            _ => {}
        }
    }
    out
}

/// The database seen from inside one file: resolves top-level names and
/// records what the running query depended on.
struct FileCtx<'d> {
    db: &'d mut Database,
    file: u32,
    items: Arc<Vec<Arc<ItemSyntax>>>,
    scope: Arc<Scope>,
    /// Dependencies of each query being computed, innermost last.
    frames: Vec<Vec<(String, u64)>>,
    /// Names whose definitions are being computed, to stop cycles.
    in_progress: Vec<String>,
}

impl FileCtx<'_> {
    fn record(&mut self, key: &str, fingerprint: u64) {
        if let Some(frame) = self.frames.last_mut() {
            if !frame.iter().any(|(k, _)| k == key) {
                frame.push((key.to_string(), fingerprint));
            }
        }
    }

    /// The current fingerprint of a dependency key.
    fn fingerprint(&mut self, key: &str) -> u64 {
        match key {
            DEP_NAMES => self.scope.names_fp,
            DEP_IMPORTS => u64::from(self.scope.has_imports),
            name => self.def(name).1,
        }
    }

    fn deps_hold(&mut self, deps: &[(String, u64)]) -> bool {
        deps.iter().all(|(key, fingerprint)| self.fingerprint(key) == *fingerprint)
    }

    /// The definition of a top-level name and its fingerprint (0 if the
    /// file does not define it).
    fn def(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let Some(&index) = self.scope.names.get(name) else { return (None, 0) };
        if self.in_progress.iter().any(|n| n == name) {
            return (None, 0);
        }
        let item = self.items[index].clone();
        let revision = self.db.revision;
        let key = (self.file, item.hash, name.to_string());
        if let Some(memo) = self.db.defs.get_mut(&key) {
            memo.last_used = revision;
            if memo.verified_at == revision {
                return (Some(memo.def.clone()), memo.fingerprint);
            }
            let deps = memo.deps.clone();
            self.in_progress.push(name.to_string());
            let holds = self.deps_hold(&deps);
            self.in_progress.pop();
            if holds {
                let memo = self.db.defs.get_mut(&key).unwrap();
                memo.verified_at = revision;
                return (Some(memo.def.clone()), memo.fingerprint);
            }
        }

        // One statement can define several names (`let (a, b) = ...`, an
        // enum and its variants); they are computed and stored together.
        self.in_progress.push(name.to_string());
        self.frames.push(Vec::new());
        let mut defs = Vec::new();
        {
            let mut checker = Checker::new(self, &item.text, Vec::new());
            for stmt in &item.parsed.stmts {
                defs.extend(checker.defs_of(stmt));
            }
        }
        let deps = self.frames.pop().unwrap();
        self.in_progress.pop();
        self.db.stats.defs_computed += 1;

        let mut found = (None, 0);
        let mut stored: Vec<String> = Vec::new();
        for (def_name, def) in defs {
            if stored.contains(&def_name) {
                continue;
            }
            let fingerprint = def.fingerprint();
            let def = Arc::new(def);
            if def_name == name {
                found = (Some(def.clone()), fingerprint);
            }
            stored.push(def_name.clone());
            self.db.defs.insert(
                (self.file, item.hash, def_name),
                DefMemo {
                    def,
                    fingerprint,
                    deps: deps.clone(),
                    verified_at: revision,
                    last_used: revision,
                },
            );
        }
        found
    }

    fn check_item(&mut self, index: usize) -> Arc<ItemCheck> {
        let item = self.items[index].clone();
        let revision = self.db.revision;
        let key = (self.file, item.hash);
        if let Some(memo) = self.db.checks.get_mut(&key) {
            memo.last_used = revision;
            let result = memo.result.clone();
            if memo.verified_at == revision {
                return result;
            }
            let deps = memo.deps.clone();
            if self.deps_hold(&deps) {
                self.db.checks.get_mut(&key).unwrap().verified_at = revision;
                self.db.stats.items_reused += 1;
                return result;
            }
        }
        self.frames.push(Vec::new());
        let checked = {
            let mut checker = Checker::new(self, &item.text, item.parsed.poisoned.clone());
            checker.item(&item.parsed.stmts);
            checker.finish()
        };
        let deps = self.frames.pop().unwrap();
        self.db.stats.items_checked += 1;
        let result = Arc::new(ItemCheck {
            diagnostics: checked.diagnostics,
            hovers: checked.hovers,
            refs: checked.refs,
        });
        self.db.checks.insert(
            key,
            CheckMemo {
                result: result.clone(),
                deps,
                verified_at: revision,
                last_used: revision,
            },
        );
        result
    }
}

impl Env for FileCtx<'_> {
    fn global(&mut self, name: &str) -> Option<Arc<Def>> {
        let (def, fingerprint) = self.def(name);
        self.record(name, fingerprint);
        def
    }

    fn has_imports(&mut self) -> bool {
        self.record(DEP_IMPORTS, u64::from(self.scope.has_imports));
        self.scope.has_imports
    }

    fn names(&mut self) -> Vec<String> {
        self.record(DEP_NAMES, self.scope.names_fp);
        let mut names: Vec<String> = self.scope.names.keys().cloned().collect();
        names.sort();
        names
    }
}
