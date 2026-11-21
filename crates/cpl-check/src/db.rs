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

use crate::check::lends::{flow_of, item_names, own_effects, Effects, Flow, Replay};
use crate::check::{Checker, Env, GlobalRef, HoverEntry};
use crate::def::{hash_str, impl_name, impl_parts, DeclKind, Def};
use crate::modules::{FsLoader, Loader};
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
    /// What the item leaves behind for the top-level statements after it.
    pub flow: Flow,
}

/// A top-level name an item declares. The span is relative to the item.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Decl {
    pub name: String,
    pub span: Span,
    pub kind: DeclKind,
    /// Declared with `pub`: other modules may import it.
    pub public: bool,
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
    /// True if a `use` names a module that could not be loaded: a name
    /// that is not found may have come from it, so it is not reported.
    pub has_imports: bool,
    /// Changes when anything this file's imports can reach changes shape.
    pub imports_fp: u64,
    /// Names brought in by `use`, by the name this file calls them.
    pub imports: HashMap<String, Import>,
    /// Modules imported whole (`use lib.treap`, `use lib.flow as fl`), by
    /// the name this file calls them: their public names are `treap.X`.
    pub aliases: HashMap<String, usize>,
    /// Every module this file reaches through `use`, its own imports
    /// first and theirs after. A type that comes back from an imported fn
    /// is looked up here even when this file never names it.
    pub modules: Vec<Module>,
}

/// One name of `use std.algebra.{Monoid}`.
#[derive(Debug)]
pub struct Import {
    /// Index into [`Analysis::modules`].
    pub module: usize,
    /// What the module calls it.
    pub name: String,
}

#[derive(Debug)]
pub struct Module {
    /// `std.algebra`.
    pub name: String,
    pub path: String,
    pub analysis: Arc<Analysis>,
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
        self.diagnostics
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error)
    }

    /// The item at `offset`, with the offset relative to it.
    pub(crate) fn item_at(&self, offset: u32) -> Option<(&AnalyzedItem, u32)> {
        let at = self.items.partition_point(|item| item.span.end < offset);
        let item = self.items.get(at)?;
        item.span
            .touches(offset)
            .then(|| (item, offset - item.offset))
    }

    /// Where a top-level name is declared, in file positions.
    /// The names other modules may import, sorted.
    pub(crate) fn public_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .items
            .iter()
            .flat_map(|item| &item.syntax.decls)
            .filter(|d| d.public && d.kind != DeclKind::Impl)
            .map(|d| d.name.clone())
            .collect();
        names.sort();
        names.dedup();
        names
    }

    pub(crate) fn decl_span(&self, name: &str) -> Option<Span> {
        let item = &self.items[*self.scope.names.get(name)?];
        let decl = item.syntax.decls.iter().find(|d| d.name == name)?;
        Some(decl.span.shift(item.offset))
    }

    /// Every module this file reaches through `use`, its direct imports first.
    pub fn modules(&self) -> &[Module] {
        &self.scope.modules
    }

    /// Names brought in directly by `use`, keyed by their local name.
    pub fn imports(&self) -> &HashMap<String, Import> {
        &self.scope.imports
    }

    /// Modules imported as namespaces (`use lib.treap`, `use lib.flow as fl`).
    pub fn aliases(&self) -> &HashMap<String, usize> {
        &self.scope.aliases
    }
}

struct FileState {
    id: u32,
    text: Arc<str>,
    analysis: Option<Arc<Analysis>>,
    /// Read by the module loader, not given by `set_file`: the loader may
    /// replace its text.
    loaded: bool,
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
/// The dependency on the earlier top-level statements that reach an item.
const DEP_LENT: &str = "\0lent";
/// Prefix of the dependency on what calling a fn touches outside itself.
const DEP_EFFECTS: &str = "\0fx:";
/// Prefix of the dependency on what a type's name resolves to.
const DEP_TYPE: &str = "\0type:";

/// Memos unused for this many revisions are dropped.
const KEEP_REVISIONS: u64 = 32;

pub struct Database {
    files: HashMap<String, FileState>,
    next_file: u32,
    revision: u64,
    syntax: HashMap<u64, SyntaxMemo>,
    defs: HashMap<(u32, u64, String), DefMemo>,
    checks: HashMap<(u32, u64), CheckMemo>,
    stats: Stats,
    loader: Box<dyn Loader>,
    /// The files being analyzed, outermost first: a module that imports
    /// one of them is a cycle.
    analyzing: Vec<String>,
}

impl Default for Database {
    fn default() -> Database {
        Database::with_loader(Box::new(FsLoader))
    }
}

impl Database {
    /// A database that reads the modules its files import from disk.
    pub fn new() -> Database {
        Database::default()
    }

    /// A database that asks `loader` for the modules its files import.
    pub fn with_loader(loader: Box<dyn Loader>) -> Database {
        Database {
            files: HashMap::new(),
            next_file: 0,
            revision: 0,
            syntax: HashMap::new(),
            defs: HashMap::new(),
            checks: HashMap::new(),
            stats: Stats::default(),
            loader,
            analyzing: Vec::new(),
        }
    }

    /// Sets the text of a file. Nothing is computed until it is asked for.
    pub fn set_file(&mut self, path: &str, text: impl Into<Arc<str>>) {
        self.store(path, text.into(), false);
    }

    fn store(&mut self, path: &str, text: Arc<str>, loaded: bool) {
        match self.files.get_mut(path) {
            // What the editor holds wins over what is on disk.
            Some(file) if loaded && !file.loaded => {}
            Some(file) if file.text == text => file.loaded &= loaded,
            Some(file) => {
                file.text = text;
                file.loaded = loaded;
                // Any file may import this one, and re-analyzing an
                // unchanged file is a walk over memoized items.
                for file in self.files.values_mut() {
                    file.analysis = None;
                }
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
                        loaded,
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
            let cached = self
                .syntax
                .get_mut(&hash)
                .filter(|m| m.item.text == item_text && m.item.col == col);
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
                        flow: flow_of(&parsed.stmts),
                        has_use: parsed
                            .stmts
                            .iter()
                            .any(|s| matches!(s.kind, StmtKind::Use { .. })),
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
        let mut import_diags: Vec<(usize, Diagnostic)> = Vec::new();
        self.analyzing.push(path.to_string());
        for (index, item) in items.iter().enumerate() {
            for stmt in item.parsed.stmts.iter().filter(|_| item.has_use) {
                if let StmtKind::Use {
                    path: module,
                    names,
                    glob,
                    alias,
                } = &stmt.kind
                {
                    let mut diags = Vec::new();
                    self.import(
                        path,
                        module,
                        names,
                        *glob,
                        alias.as_ref(),
                        &mut scope,
                        &mut diags,
                    );
                    import_diags.extend(diags.into_iter().map(|d| (index, d)));
                }
            }
            // Also preload modules referenced by block-scoped `use` statements inside functions/tests.
            for stmt in &item.parsed.stmts {
                if let StmtKind::Fn(f) = &stmt.kind {
                    if let cpl_syntax::ast::FnBody::Block(block) = &f.body {
                        for inner in &block.stmts {
                            if let StmtKind::Use { path: module, .. } = &inner.kind {
                                let mut dummy_scope = Scope::default();
                                let mut dummy_diags = Vec::new();
                                self.import(
                                    path,
                                    module,
                                    &[],
                                    false,
                                    None,
                                    &mut dummy_scope,
                                    &mut dummy_diags,
                                );
                                for m in dummy_scope.modules {
                                    if !scope.modules.iter().any(|e| e.path == m.path) {
                                        scope.modules.push(m);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // The prelude (`stdlib/prelude.cp`): every module — root, std and
        // user libraries alike — sees the same prelude. A name the prelude
        // lists that the file uses and neither declares nor imports is
        // imported for it, so `Deque<Int>` and `Mod<P>` work everywhere.
        // A `use` that failed may have been meant to bring the name.
        if import_diags.is_empty() && !is_prelude(path) {
            let declared: std::collections::HashSet<&str> = items
                .iter()
                .flat_map(|item| item.decls.iter().map(|decl| decl.name.as_str()))
                .collect();
            let imported_module = self.analyzing.len() > 1;
            let mut ident_tokens: std::collections::HashSet<&str> = std::collections::HashSet::new();
            let mut dot_ident_tokens: std::collections::HashSet<&str> =
                std::collections::HashSet::new();
            for item in &items {
                if imported_module
                    && item.parsed.stmts.iter().any(|s| {
                        s.decorators.iter().any(|d| {
                            !d.directive
                                && matches!(
                                    d.name.text.as_str(),
                                    "test" | "checker" | "interactor" | "brute"
                                )
                        })
                    })
                {
                    continue;
                }
                let lexed = cpl_syntax::lex(&item.text);
                let mut prev_dot = false;
                for tok in &lexed.tokens {
                    if tok.kind == cpl_syntax::TokenKind::Ident {
                        let s = &item.text[tok.span.start as usize..tok.span.end as usize];
                        ident_tokens.insert(s);
                        if prev_dot {
                            dot_ident_tokens.insert(s);
                        }
                    }
                    prev_dot = tok.kind == cpl_syntax::TokenKind::Dot;
                }
            }
            let uses = |name: &str| ident_tokens.contains(name);
            // A method the library adds to a type is used as `.name`.
            let calls = |name: &str| dot_ident_tokens.contains(name);
            let listed = crate::prelude::names().iter().map(|entry| (entry, false));
            let methods = crate::prelude::methods().iter().map(|entry| (entry, true));
            for ((module, name), method) in listed.chain(methods) {
                let used = if method {
                    calls(&name.text)
                        || (crate::prelude::SEQUENCE_FORMS.contains(&name.text.as_str()) && uses(&name.text))
                } else {
                    uses(&name.text)
                        || (!is_std_path(path)
                            && crate::prelude::BUILT_ON
                                .iter()
                                .any(|(word, on)| *on == name.text && uses(word)))
                };
                if scope.imports.contains_key(&name.text) || !used {
                    continue;
                }
                // `count(..)` the language has itself is not the library's
                // `s.count(c)` either.
                let own = declared.contains(name.text.as_str())
                    || (method && crate::prelude::VALUE_NAMES.contains(&name.text.as_str()));
                if own && !method {
                    continue;
                }
                let had_imports = scope.has_imports;
                let mods_before = scope.modules.len();
                let mut diags = Vec::new();
                self.import(
                    path,
                    module,
                    std::slice::from_ref(name),
                    false,
                    None,
                    &mut scope,
                    &mut diags,
                );
                scope.has_imports = had_imports;
                let cycled = scope.modules[mods_before..].iter().any(|m| {
                    m.analysis
                        .diagnostics
                        .iter()
                        .any(|d| d.code == codes::UNRESOLVED_MODULE)
                });
                if cycled || !diags.is_empty() {
                    scope.modules.truncate(mods_before);
                    scope.imports.remove(&name.text);
                    continue;
                }
                // Beside a name the file declares itself, the library's
                // method stays reachable for the types the file's own fn
                // does not take, under a name no program can write.
                if own {
                    if let Some(import) = scope.imports.remove(&name.text) {
                        scope.imports.insert(crate::prelude::std_method(&name.text), import);
                    }
                }
            }
        }
        self.analyzing.pop();
        // Loading the modules analyzed them, which moved the revision on.
        scope.imports_fp = scope.fingerprint_imports();
        let mut duplicates: Vec<(usize, Decl)> = Vec::new();
        for (index, item) in items.iter().enumerate() {
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
        scope.names_fp = hash_str(
            &sorted
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
        ) | 2;
        let scope = Arc::new(scope);
        let items = Arc::new(items);

        let mut ctx = FileCtx {
            db: self,
            file: id,
            items: items.clone(),
            scope: scope.clone(),
            frames: Vec::new(),
            in_progress: Vec::new(),
            current: None,
            names: None,
            effects: HashMap::new(),
            own_effects: HashMap::new(),
            effects_of: Vec::new(),
            vars: None,
        };
        let checks: Vec<Arc<ItemCheck>> = (0..items.len()).map(|i| ctx.check_item(i)).collect();

        // Diagnostics, one per root cause: the lexer's first, then each
        // item's syntax errors, then what the checker found outside them.
        let mut sink = Sink::new();
        for diag in &lexed.diagnostics {
            sink.push(diag.clone());
        }
        for &span in &lexed.poisoned {
            sink.poison(span);
        }
        for ((range, item), check) in ranges.iter().zip(items.iter()).zip(&checks) {
            let offset = range.span.start;
            for diag in item.parsed.diagnostics.iter().chain(&check.diagnostics) {
                sink.push(diag.clone().shift(offset));
            }
        }
        let imports_ok = import_diags.is_empty();
        for (index, diag) in import_diags {
            sink.push(diag.shift(ranges[index].span.start));
        }
        // `@!multitest` divides the file once (decision 20.5), and
        // `@!interactive` is said once.
        for (name, help) in [
            ("multitest", "the file has one divider: what is below it runs once per test case"),
            ("interactive", "one is enough: it holds for the whole file"),
        ] {
            let found = items.iter().enumerate().flat_map(|(index, item)| {
                let decorators = item.parsed.stmts.iter().flat_map(|stmt| &stmt.decorators);
                decorators
                    .filter(move |d| d.name.text == name && (d.directive || name == "multitest"))
                    .map(move |d| (index, d))
            });
            for (index, decorator) in found.skip(1) {
                sink.push(
                    Diagnostic::error(
                        codes::UNEXPECTED_TOKEN,
                        decorator.span.shift(ranges[index].span.start),
                        format!("a second `@!{name}`"),
                    )
                    .with_help(help),
                );
            }
        }
        for (index, decl) in duplicates {
            let offset = ranges[index].span.start;
            let first_index = scope.names[&decl.name];
            let first = items[first_index]
                .decls
                .iter()
                .find(|d| d.name == decl.name);
            let mut diag = Diagnostic::error(
                codes::DUPLICATE_DEFINITION,
                decl.span.shift(offset),
                format!("`{}` is already defined in this file", decl.name),
            )
            .with_help("rename one of them; a top-level name has one definition");
            if let Some(first) = first {
                diag = diag.with_label(
                    first.span.shift(ranges[first_index].span.start),
                    "first defined here",
                );
            }
            sink.push(diag);
        }
        for (index, check) in checks.iter().enumerate() {
            let offset = ranges[index].span.start;
            for r in check.refs.iter().filter(|r| r.top_level) {
                let Some(&def_index) = scope.names.get(&r.name) else {
                    continue;
                };
                if def_index < index {
                    continue;
                }
                let Some(decl) = items[def_index].decls.iter().find(|d| d.name == r.name) else {
                    continue;
                };
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
                let mut diag =
                    Diagnostic::error(codes::USED_BEFORE_SET, r.span.shift(offset), message);
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
        if imports_ok {
            if let Some(file) = self.files.get_mut(path) {
                file.analysis = Some(analysis.clone());
            }
        }
        Some(analysis)
    }

    /// Every diagnostic for a file, in source order.
    pub fn diagnostics(&mut self, path: &str) -> Vec<Diagnostic> {
        self.analyze(path)
            .map(|a| a.diagnostics.clone())
            .unwrap_or_default()
    }

    /// The definition of a top-level name in an analyzed file.
    /// Loads what one `use` names into `scope`.
    #[allow(clippy::too_many_arguments)]
    fn import(
        &mut self,
        from: &str,
        module: &[ast::Name],
        names: &[ast::Name],
        glob: bool,
        alias: Option<&ast::Name>,
        scope: &mut Scope,
        diags: &mut Vec<Diagnostic>,
    ) {
        let (Some(first), Some(last)) = (module.first(), module.last()) else {
            return;
        };
        let whole = first.span.to(last.span);
        let parts: Vec<String> = module.iter().map(|n| n.text.clone()).collect();
        let mut single = None;
        let mut found = self.load_module(from, &parts, whole, diags);
        // `use std.math.gcd`: the last name is a name in the module before it.
        if found.is_none() && diags.is_empty() && names.is_empty() && !glob && parts.len() > 2 {
            found = self.load_module(from, &parts[..parts.len() - 1], whole, diags);
            single = found.as_ref().map(|_| last);
        }
        let Some(loaded) = found else {
            if diags.is_empty() {
                let dotted = parts.join(".");
                let help = match first.text.as_str() {
                    "std" => "the standard library's modules are the files of `stdlib/`".to_string(),
                    lib => format!(
                        "`{lib}` is a library: name its folder under `[libs]` in a `cp.toml` at or above this file (`{lib} = \"path\"`), or put this file below a folder whose `lib.toml` says `name = \"{lib}\"`"
                    ),
                };
                diags.push(
                    Diagnostic::error(
                        codes::UNRESOLVED_MODULE,
                        whole,
                        if first.text == "std" {
                            format!("no module `{dotted}`, and not in the stdlib (yet)")
                        } else {
                            format!("no module `{dotted}`")
                        },
                    )
                    .with_help(help),
                );
            }
            scope.has_imports = true;
            return;
        };
        let index = scope.add_module(loaded);
        let module_name = scope.modules[index].name.clone();
        let analysis = scope.modules[index].analysis.clone();
        let decl_of = |name: &str| {
            let item = &analysis.items[*analysis.scope.names.get(name)?];
            item.syntax.decls.iter().find(|d| d.name == name)
        };
        let listed: Vec<&ast::Name> = single.into_iter().chain(names).collect();
        for name in listed {
            match decl_of(&name.text) {
                Some(decl) if decl.public => {
                    scope.imports.insert(
                        name.text.clone(),
                        Import {
                            module: index,
                            name: name.text.clone(),
                        },
                    );
                }
                Some(_) => diags.push(
                    Diagnostic::error(
                        codes::PRIVATE_NAME,
                        name.span,
                        format!("`{}` is private to `{module_name}`", name.text),
                    )
                    .with_help(format!("declare it `pub` in `{module_name}` to import it")),
                ),
                None => {
                    let public = analysis.public_names();
                    let mut diag = Diagnostic::error(
                        codes::UNRESOLVED_NAME,
                        name.span,
                        format!("`{module_name}` has no `{}`", name.text),
                    );
                    let std = module_name.starts_with("std.");
                    if let Some(s) =
                        crate::check::closest(&name.text, public.iter().map(String::as_str))
                    {
                        diag = diag.with_help(format!("did you mean `{s}`?")).with_fix(
                            format!("change to `{s}`"),
                            name.span,
                            s,
                        );
                    } else if let Some(m) = std
                        .then(|| crate::prelude::std_module(&name.text))
                        .flatten()
                    {
                        diag = diag.with_help(format!("`{}` is in `std.{m}`", name.text));
                    } else if std {
                        diag.message.push_str(", and not in the stdlib (yet)");
                    }
                    diags.push(diag);
                }
            }
        }
        if glob {
            for name in analysis.public_names() {
                scope.imports.entry(name.clone()).or_insert(Import {
                    module: index,
                    name,
                });
            }
        } else if names.is_empty() && single.is_none() {
            let local = alias.map_or(&last.text, |a| &a.text);
            scope.aliases.insert(local.clone(), index);
        }
    }

    /// Finds, reads and analyzes the module `parts`, imported by `from`.
    fn load_module(
        &mut self,
        from: &str,
        parts: &[String],
        span: Span,
        diags: &mut Vec<Diagnostic>,
    ) -> Option<Module> {
        let (path, text) = self.loader.load(from, parts)?;
        let name = parts.join(".");
        if self.analyzing.contains(&path) {
            let message = format!("`{name}` imports this file, so they import each other");
            diags.push(
                Diagnostic::error(codes::UNRESOLVED_MODULE, span, message)
                    .with_help("move what both need into a third module"),
            );
            return None;
        }
        self.store(&path, text.into(), true);
        let analysis = self.analyze(&path)?;
        Some(Module {
            name,
            path,
            analysis,
        })
    }

    pub(crate) fn lookup(
        &mut self,
        path: &str,
        analysis: &Analysis,
        name: &str,
    ) -> Option<Arc<Def>> {
        let id = self.files.get(path)?.id;
        let items: Vec<Arc<ItemSyntax>> = analysis.items.iter().map(|i| i.syntax.clone()).collect();
        let mut ctx = FileCtx {
            db: self,
            file: id,
            items: Arc::new(items),
            scope: analysis.scope.clone(),
            frames: Vec::new(),
            in_progress: Vec::new(),
            current: None,
            names: None,
            effects: HashMap::new(),
            own_effects: HashMap::new(),
            effects_of: Vec::new(),
            vars: None,
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
impl Scope {
    fn fingerprint_imports(&self) -> u64 {
        let mut imports: Vec<String> = self
            .imports
            .iter()
            .map(|(local, import)| format!("{local}={}:{}", import.module, import.name))
            .chain(self.aliases.iter().map(|(alias, m)| format!("{alias}={m}")))
            .collect();
        imports.sort();
        let modules: Vec<String> = self
            .modules
            .iter()
            .map(|m| format!("{}@{}", m.path, m.analysis.scope.names_fp))
            .collect();
        hash_str(&format!(
            "{}\n{}\n{}",
            self.has_imports,
            imports.join("\n"),
            modules.join("\n")
        )) | 2
    }

    /// Adds a module and the modules it reaches, once each; returns its
    /// index.
    fn add_module(&mut self, module: Module) -> usize {
        if let Some(index) = self.modules.iter().position(|m| m.path == module.path) {
            return index;
        }
        let reached: Vec<Module> = module
            .analysis
            .scope
            .modules
            .iter()
            .map(|m| Module {
                name: m.name.clone(),
                path: m.path.clone(),
                analysis: m.analysis.clone(),
            })
            .collect();
        self.has_imports |= module.analysis.scope.has_imports;
        self.modules.push(module);
        let index = self.modules.len() - 1;
        for module in reached {
            self.add_module(module);
        }
        index
    }
}

fn inside_own_init(item: &ItemSyntax, name: &str, span: Span) -> bool {
    item.parsed.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Let {
            pat,
            init: Some(init),
            ..
        } => init.span.contains(span) && pattern_binds(pat, name),
        StmtKind::Const {
            name: n,
            init: Some(init),
            ..
        } => init.span.contains(span) && n.text == name,
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
    let mut out = Vec::new();
    for stmt in stmts {
        let from = out.len();
        stmt_decls(stmt, &mut out);
        for decl in &mut out[from..] {
            decl.public = stmt.public;
        }
    }
    out
}

/// True if a syntax error was reported on the first line of `stmt`, where a
/// declaration says what it declares: the names stay bound, but what the
/// rest of the file can know about them is unknown.
fn head_is_poisoned(text: &str, stmt: &ast::Stmt, poisoned: &[Span]) -> bool {
    let (start, end) = (stmt.span.start as usize, stmt.span.end as usize);
    let mut depth = 0u32;
    let mut head_end = end;
    for (i, b) in text.as_bytes()[start..end.min(text.len())].iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b'\n' if depth == 0 => {
                head_end = start + i;
                break;
            }
            _ => {}
        }
    }
    poisoned
        .iter()
        .any(|p| (p.start as usize) < head_end && (p.end as usize) > start)
}

fn stmt_decls(stmt: &ast::Stmt, out: &mut Vec<Decl>) {
    fn pattern(pat: &ast::Pattern, out: &mut Vec<Decl>) {
        match &pat.kind {
            ast::PatternKind::Name(name) if name != "_" => out.push(Decl {
                name: name.clone(),
                span: pat.span,
                kind: DeclKind::Value,
                public: false,
            }),
            ast::PatternKind::Domain(name) => out.push(Decl {
                name: name.text.clone(),
                span: name.span,
                kind: DeclKind::Domain,
                public: false,
            }),
            // `let LIM = 100`: a capitalized name on its own is a value
            // (or a domain, when the value is one).
            ast::PatternKind::Variant { path, args } if path.len() == 1 && args.is_empty() => {
                out.push(Decl {
                    name: path[0].text.clone(),
                    span: pat.span,
                    kind: DeclKind::Value,
                    public: false,
                })
            }
            ast::PatternKind::Tuple(items) => {
                for item in items {
                    match crate::check::domain_binding(item) {
                        Some(name) => out.push(Decl {
                            name: name.to_string(),
                            span: item.span,
                            kind: DeclKind::Domain,
                            public: false,
                        }),
                        None => pattern(item, out),
                    }
                }
            }
            _ => {}
        }
    }
    let push = |out: &mut Vec<Decl>, name: &ast::Name, kind: DeclKind| {
        if !name.text.is_empty() {
            out.push(Decl {
                name: name.text.clone(),
                span: name.span,
                kind,
                public: false,
            });
        }
    };
    {
        match &stmt.kind {
            StmtKind::Let { pat, ty, .. } => {
                pattern(pat, out);
                if let Some(ty) = ty {
                    crate::check::own_domains(ty, &mut |name| push(out, name, DeclKind::Domain));
                }
            }
            StmtKind::Const { name, .. } => push(out, name, DeclKind::Value),
            StmtKind::Fn(f) => {
                if let Some((table, span)) = crate::check::memo_table(stmt) {
                    out.push(Decl {
                        name: table.to_string(),
                        span,
                        kind: DeclKind::Value,
                        public: false,
                    });
                }
                push(out, &f.name, DeclKind::Fn)
            }
            StmtKind::Struct(s) => push(out, &s.name, DeclKind::Struct),
            StmtKind::Enum(e) => {
                push(out, &e.name, DeclKind::Enum);
                for variant in &e.variants {
                    push(out, &variant.name, DeclKind::Variant);
                }
            }
            StmtKind::Domain { name, .. } => push(out, name, DeclKind::Domain),
            StmtKind::TypeAlias { name, .. } => push(out, name, DeclKind::Alias),
            StmtKind::Trait { name, .. } => push(out, name, DeclKind::Trait),
            // An impl is filed under a name of its own, so it is found,
            // memoized and reported twice like any other definition.
            StmtKind::Impl {
                trait_name, target, ..
            } => {
                if let Some(target_name) = crate::check::type_head(target) {
                    let trait_head = trait_name.as_ref().and_then(crate::check::trait_impl_key);
                    out.push(Decl {
                        name: impl_name(trait_head.as_deref(), &target_name),
                        span: trait_name
                            .as_ref()
                            .map_or(target.span, |t| t.span.to(target.span)),
                        kind: DeclKind::Impl,
                        public: true,
                    });
                }
            }
            _ => {}
        }
    }
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
    /// The item being checked.
    current: Option<usize>,
    /// What each item mentions and assigns, once a statement needs it.
    names: Option<Vec<(Vec<String>, Vec<String>)>>,
    /// What calling each fn touches, once a call needs it.
    effects: HashMap<String, (Arc<Effects>, u64)>,
    /// What each fn's own body touches.
    own_effects: HashMap<String, Option<Arc<Effects>>>,
    /// The fns whose effects are being worked out, innermost last.
    effects_of: Vec<String>,
    /// The top-level `var`s.
    vars: Option<std::collections::HashSet<String>>,
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
            DEP_IMPORTS => self.scope.imports_fp,
            DEP_LENT => self.replays_fingerprint(),
            key => match (key.strip_prefix(DEP_TYPE), key.strip_prefix(DEP_EFFECTS)) {
                (Some(name), _) => self.type_def(name).1,
                (_, Some(name)) => self.effects(name).1,
                _ => self.def(key).1,
            },
        }
    }

    fn deps_hold(&mut self, deps: &[(String, u64)]) -> bool {
        deps.iter()
            .all(|(key, fingerprint)| self.fingerprint(key) == *fingerprint)
    }

    /// The definition of a top-level name and its fingerprint (0 if the
    /// file does not define it).
    fn def(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let Some(&index) = self.scope.names.get(name) else {
            return self.imported(name);
        };
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
                let found = checker.defs_of(stmt);
                if head_is_poisoned(&item.text, stmt, &item.parsed.poisoned) {
                    defs.extend(found.into_iter().map(|(name, def)| (name, def.recovered())));
                } else {
                    defs.extend(found);
                }
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

    /// The definition of a name this file does not define: one it imports
    /// (`Monoid` after `use std.algebra.{Monoid}`, `fl.Graph` after `use
    /// lib.flow as fl`), or an impl in a module it reaches.
    fn imported(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let scope = self.scope.clone();
        if let Some(import) = scope.imports.get(name) {
            return self.module_def(&scope.modules[import.module], &import.name);
        }
        if let Some((alias, member)) = name.split_once('.') {
            return match scope.aliases.get(alias) {
                Some(&index) => {
                    let module = &scope.modules[index];
                    let public = module.analysis.public_names();
                    if public.iter().any(|n| n == member) {
                        self.module_def(module, member)
                    } else {
                        (None, 0)
                    }
                }
                None => (None, 0),
            };
        }
        if impl_parts(name).is_some() {
            return self.reached(name);
        }
        (None, 0)
    }

    /// The definition of `name` in the first module this file reaches
    /// that has one.
    fn reached(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        let scope = self.scope.clone();
        for module in &scope.modules {
            if module.analysis.scope.names.contains_key(name) {
                return self.module_def(module, name);
            }
        }
        (None, 0)
    }

    /// The definition behind a type's name: this file's own or imported
    /// one, else the one in a module it reaches. A value of a library's
    /// type can come back from a call without the file ever importing the
    /// type by name.
    fn type_def(&mut self, name: &str) -> (Option<Arc<Def>>, u64) {
        match self.def(name) {
            (None, _) => self.reached(name),
            found => found,
        }
    }

    fn module_def(&mut self, module: &Module, name: &str) -> (Option<Arc<Def>>, u64) {
        match self.db.lookup(&module.path, &module.analysis, name) {
            Some(def) => {
                let fingerprint = def.fingerprint();
                (Some(def), fingerprint)
            }
            None => (None, 0),
        }
    }

    /// The earlier top-level statements whose effect reaches item
    /// `current`: views still named at or after it, and asserts none of
    /// whose names was assigned since. Empty for almost every file.
    fn replays(&mut self, current: usize) -> Vec<Replay> {
        let items = self.items.clone();
        if !items[..current].iter().any(|item| item.flow != Flow::None) {
            return Vec::new();
        }
        let names = self.names.get_or_insert_with(|| {
            items
                .iter()
                .map(|item| item_names(&item.parsed.stmts))
                .collect()
        });
        let mut out = Vec::new();
        for (index, item) in items[..current].iter().enumerate() {
            match &item.flow {
                Flow::None => {}
                Flow::View(name) => {
                    let mentioned = |k: usize| names[k].0.contains(name);
                    let live_after = (current + 1..items.len()).any(mentioned);
                    if !(live_after || mentioned(current)) {
                        continue;
                    }
                    // A later `let` of the same name is another value.
                    if (index + 1..current).any(|k| items[k].decls.iter().any(|d| d.name == *name))
                    {
                        continue;
                    }
                    out.push(Replay {
                        item: item.clone(),
                        live_after,
                        written: (current..items.len()).any(|k| names[k].1.contains(name)),
                    });
                }
                Flow::Assert(about) => {
                    let assigned = (index + 1..current)
                        .any(|k| names[k].1.iter().any(|written| about.contains(written)));
                    if !assigned {
                        out.push(Replay {
                            item: item.clone(),
                            live_after: false,
                            written: false,
                        });
                    }
                }
            }
        }
        out
    }

    /// What the body of the fn `name` of this file touches by itself: the
    /// top-level `var`s it changes and reads, and the fns of this file it
    /// calls. `None` if this file has no such fn.
    fn own_effects(&mut self, name: &str) -> Option<Arc<Effects>> {
        if let Some(known) = self.own_effects.get(name) {
            return known.clone();
        }
        let items = self.items.clone();
        let vars = self.vars.get_or_insert_with(|| {
            let mutable = |item: &&Arc<ItemSyntax>| {
                item.parsed.stmts.iter().any(|stmt| {
                    matches!(
                        stmt.kind,
                        StmtKind::Let { mutable: true, .. }
                            | StmtKind::Let {
                                init: Some(cpl_syntax::ast::Expr {
                                    kind: cpl_syntax::ast::ExprKind::Mut(_),
                                    ..
                                }),
                                ..
                            }
                    )
                })
            };
            items
                .iter()
                .filter(mutable)
                .flat_map(|item| item.decls.iter().map(|decl| decl.name.clone()))
                .collect()
        });
        let own = self.scope.names.get(name).and_then(|&index| {
            items[index]
                .parsed
                .stmts
                .iter()
                .find_map(|stmt| match &stmt.kind {
                    StmtKind::Fn(f) if f.name.text == name => Some(own_effects(f)),
                    _ => None,
                })
        });
        let own = own.map(|mut own| {
            own.writes.retain(|n| vars.contains(n));
            own.reads.retain(|n| vars.contains(n));
            let scope = &self.scope;
            own.calls.retain(|n| {
                scope.names.get(n).is_some_and(|&index| {
                    items[index]
                        .decls
                        .iter()
                        .any(|d| d.name == *n && d.kind == DeclKind::Fn)
                })
            });
            Arc::new(own)
        });
        self.own_effects.insert(name.to_string(), own.clone());
        own
    }

    /// What calling the fn `name` of this file touches outside itself,
    /// counting the fns it calls, and the fingerprint of that. A fn from
    /// a module touches nothing here.
    fn effects(&mut self, name: &str) -> (Arc<Effects>, u64) {
        if let Some(known) = self.effects.get(name) {
            return known.clone();
        }
        let mut all = Effects::default();
        // Each fn is asked once: a fn it calls answers from what it
        // already worked out. A fn that calls itself through others, or a
        // very long chain of calls, is followed with a list instead.
        let nested = self.effects_of.iter().any(|n| n == name) || self.effects_of.len() > 200;
        if nested {
            let mut todo = vec![name.to_string()];
            let mut seen: Vec<String> = Vec::new();
            while let Some(next) = todo.pop() {
                if seen.contains(&next) {
                    continue;
                }
                if let Some((known, _)) = self.effects.get(&next) {
                    all.absorb(known);
                } else if let Some(own) = self.own_effects(&next) {
                    all.absorb(&own);
                    todo.extend(own.calls.iter().cloned());
                }
                seen.push(next);
            }
        } else if let Some(own) = self.own_effects(name) {
            all.absorb(&own);
            self.effects_of.push(name.to_string());
            for callee in &own.calls {
                let (theirs, _) = self.effects(callee);
                all.absorb(&theirs);
            }
            self.effects_of.pop();
        }
        all.writes.sort();
        all.reads.sort();
        let fingerprint = hash_str(&format!("{:?}{:?}", all.writes, all.reads)) | 2;
        let out = (Arc::new(all), fingerprint);
        self.effects.insert(name.to_string(), out.clone());
        out
    }

    fn replays_fingerprint(&mut self) -> u64 {
        let Some(current) = self.current else {
            return 2;
        };
        let replays = self.replays(current);
        if replays.is_empty() {
            return 2;
        }
        let text: Vec<String> = replays
            .iter()
            .map(|r| format!("{}:{}:{}", r.item.hash, r.live_after, r.written))
            .collect();
        hash_str(&text.join("\n")) | 2
    }

    fn check_item(&mut self, index: usize) -> Arc<ItemCheck> {
        self.current = Some(index);
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

    fn effects(&mut self, name: &str) -> Arc<Effects> {
        let (effects, fingerprint) = self.effects(name);
        self.record(&format!("{DEP_EFFECTS}{name}"), fingerprint);
        effects
    }

    fn lent_before(&mut self) -> Vec<Replay> {
        let fingerprint = self.replays_fingerprint();
        self.record(DEP_LENT, fingerprint);
        match self.current {
            Some(current) if fingerprint != 2 => self.replays(current),
            _ => Vec::new(),
        }
    }

    fn type_def(&mut self, name: &str) -> Option<Arc<Def>> {
        let (def, fingerprint) = self.type_def(name);
        self.record(&format!("{DEP_TYPE}{name}"), fingerprint);
        def
    }

    fn is_module(&mut self, alias: &str) -> bool {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        self.scope.aliases.contains_key(alias)
    }

    fn impls(&mut self, target: &str) -> Vec<(Option<String>, Arc<Def>)> {
        self.record(DEP_NAMES, self.scope.names_fp);
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let scope = self.scope.clone();
        let reached = scope
            .modules
            .iter()
            .flat_map(|module| module.analysis.scope.names.keys());
        let mut names: Vec<(String, Option<String>)> = scope
            .names
            .keys()
            .chain(reached)
            .filter_map(|name| match impl_parts(name) {
                Some((trait_name, of)) if of == target => {
                    Some((name.clone(), trait_name.map(str::to_string)))
                }
                _ => None,
            })
            .collect();
        names.sort();
        names.dedup();
        names
            .into_iter()
            .filter_map(|(name, trait_name)| Some((trait_name, self.global(&name)?)))
            .collect()
    }

    fn has_imports(&mut self) -> bool {
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        self.scope.has_imports
    }

    fn names(&mut self) -> Vec<String> {
        self.record(DEP_NAMES, self.scope.names_fp);
        self.record(DEP_IMPORTS, self.scope.imports_fp);
        let mut names: Vec<String> = self
            .scope
            .names
            .keys()
            .chain(self.scope.imports.keys())
            .chain(self.scope.aliases.keys())
            .filter(|name| impl_parts(name).is_none())
            .cloned()
            .collect();
        names.sort();
        names
    }
}

/// Whether `path` is the prelude itself, which is not given its own names.
fn is_prelude(path: &str) -> bool {
    path == "prelude.cp" || path.ends_with("stdlib/prelude.cp")
}

/// Whether `path` is a standard-library module (which declares its own imports
/// explicitly, so `BUILT_ON` must not introduce stdlib-to-stdlib cycles).
fn is_std_path(path: &str) -> bool {
    path.contains("/stdlib/") || path.starts_with("stdlib/") || path.starts_with("std/")
}
