//! The new pipeline: declarations, typed tree, ownership, generic IR, whole-program link,
//! canonical IR, a reference interpreter and a C++ emitter.
//!
//! It accepts a small part of the language and refuses the rest: a diagnostic with no
//! code from `cpl-diag` is a refusal, not a verdict on the program. Nothing shipped depends on this crate: it is run by its own tests
//! until it accepts every program, and the landing that switches `cpl` to it deletes
//! what it replaces.

mod consts;
pub mod facade;
pub mod linked;
pub mod trace;

use cpl_contract::FileId;
use cpl_next_check as check;
use cpl_next_cpp as cpp;
use cpl_next_db::db::{Db, DbError};
use cpl_next_db::items::Source;
use cpl_next_decls::views::Answers;
use cpl_next_decls::{decls, diag, modules};
use cpl_next_ir::build::{Build, Checks, Judge};
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_ir::run::{Io, Limits, RunInput};
use cpl_next_ir::{ir, ty};
use cpl_next_link as inst;
use cpl_next_lower as lower;
use cpl_next_own as own;
use diag::kind_of;
pub use diag::Kind;
pub use diag::Unbuilt;
pub use modules::{Roots, Std};
use std::collections::HashMap;
use std::path::PathBuf;

pub use cpl_next_ir::run::{Outcome, Trap};

pub use cpl_next_ir::msg::Stage;

/// Where a diagnostic is: a file of the program and a span of its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct At {
    /// `program` for the user's file, else the library module (`std.graph`).
    pub file: String,
    /// Byte offsets into that file's text.
    pub span: cpl_diag::Span,
    /// Of the span's start, from 1; the column counts bytes.
    pub line: u32,
    pub col: u32,
}

#[derive(Clone, Debug)]
pub struct Diag {
    pub stage: Stage,
    pub kind: Kind,
    /// `None`: the layer that reported it does not know where yet.
    pub at: Option<At>,
    /// The library module it is from, when it has no place.
    pub module: Option<String>,
    /// The layer that found it, when `stage` reports it for another.
    pub layer: Option<&'static str>,
    /// The text without a verdict's code, or the refused feature.
    pub message: String,
    /// What to write instead, when the layer knows.
    pub help: Option<String>,
    /// Spans are of `at.file`, as `at.span` is.
    pub labels: Vec<cpl_diag::Label>,
    pub fixes: Vec<cpl_diag::Fix>,
    /// The text of `at.file`.
    src: Option<std::sync::Arc<str>>,
}

impl Diag {
    /// The query database could not answer: the pipeline is at fault, not the program.
    pub(crate) fn internal(e: &cpl_next_db::db::DbError) -> Diag {
        Diag {
            stage: Stage::Link,
            kind: Kind::Verdict(cpl_diag::codes::INTERNAL),
            at: None,
            module: None,
            layer: Some("db"),
            message: e.to_string(),
            help: None,
            labels: Vec::new(),
            fixes: Vec::new(),
            src: None,
        }
    }

    /// The code this diagnostic carries, when it is a verdict.
    pub fn code(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Verdict(c) => Some(c.0),
            Kind::Refusal(_) => None,
        }
    }
    pub fn is_refusal(&self) -> bool {
        matches!(self.kind, Kind::Refusal(_))
    }

    /// The diagnostic every tool reads: an editor, lint, the judge's output. A refusal is
    /// `E0501`, the code of what is not supported.
    pub fn diagnostic(&self) -> cpl_diag::Diagnostic {
        let code = self
            .code()
            .map_or(cpl_diag::codes::UNSUPPORTED, cpl_diag::Code);
        let span = self
            .at
            .as_ref()
            .map_or(cpl_diag::Span::new(0, 0), |a| a.span);
        cpl_diag::Diagnostic {
            labels: self.labels.clone(),
            help: self.help.clone(),
            fixes: self.fixes.clone(),
            ..cpl_diag::msgs::check::NEXT_SAID
                .at(&[&self.message])
                .with_code(code)
                .error(span)
        }
    }

    /// [`cpl_diag::json::line`], with the layer, the file, the module, the refused feature and
    /// `text`, the line `Display` prints.
    pub fn json(&self) -> String {
        let lines = cpl_diag::LineIndex::new(self.src.as_deref().unwrap_or(""));
        let refusal = match &self.kind {
            Kind::Refusal(u) => Some(u.variant()),
            Kind::Verdict(_) => None,
        };
        let (file, shown) = (self.at.as_ref().map(|a| a.file.as_str()), self.to_string());
        let own = [
            ("stage", Some(self.stage.name())),
            ("file", file),
            ("module", self.module.as_deref()),
            ("text", Some(shown.as_str())),
            ("refusal", refusal.as_deref()),
        ];
        cpl_diag::json::line(&self.diagnostic(), &lines, &own)
    }

    /// `m` placed in its file.
    fn new(stage: Stage, m: &diag::Msg, loaded: &modules::Loaded) -> Diag {
        let local = |at: &SourceId| loaded.text_of(at).span(at);
        let src = m.at.as_ref().map(|at| loaded.text_of(at).text.clone());
        let at = m.at.as_ref().map(|at| {
            let (module, span) = (loaded.text_of(at), loaded.text_of(at).span(at));
            let (line, col) = module.line_col(span.start);
            At {
                file: module.name.clone(),
                span,
                line,
                col,
            }
        });
        let kind = kind_of(m);
        let message = match &kind {
            Kind::Verdict(_) => m.text.clone(),
            Kind::Refusal(u) => u.to_string(),
        };
        Diag {
            stage,
            kind,
            at,
            module: m.module.clone(),
            layer: m.layer,
            message,
            help: m.help.clone(),
            labels: (m.labels.iter())
                .map(|(at, message)| cpl_diag::Label {
                    span: local(at),
                    message: message.clone(),
                })
                .collect(),
            fixes: (m.fixes.iter())
                .map(|f| cpl_diag::Fix {
                    title: f.title.clone(),
                    edits: f
                        .edits
                        .iter()
                        .map(|(at, t)| (local(at), t.clone()))
                        .collect(),
                })
                .collect(),
            src,
        }
    }
}

/// Whether two spans share a byte; an empty span is the byte it is at.
fn overlaps(a: cpl_diag::Span, b: cpl_diag::Span) -> bool {
    a.start < b.end.max(b.start + 1) && b.start < a.end.max(a.start + 1)
}

/// A fn's unit: from its signature, whose place `at` is, to the end of its body.
fn fn_extent(db: &Db, at: &SourceId) -> SourceId {
    use cpl_next_syntax::ast::{self, AstNode};
    let item = db
        .get::<cpl_next_db::items::ItemOf>(&at.item)
        .ok()
        .flatten();
    let end = item.and_then(|i| {
        (cpl_next_syntax::SyntaxNode::new_root(i.green).descendants())
            .filter_map(ast::FnDecl::cast)
            .find(|f| f.signature_span().start == at.span.start)
            .map(|f| f.syntax().span().end)
    });
    at.item.at(cpl_diag::Span::new(
        at.span.start,
        end.unwrap_or(at.span.end),
    ))
}

/// Whether two places share a byte of one item.
fn overlaps_in(a: &SourceId, b: &SourceId) -> bool {
    a.item == b.item && overlaps(a.span, b.span)
}

/// The one printer of a diagnostic: `check: program:3:5: E0301: text`, and its help on the
/// next line.
impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: ", self.stage.name())?;
        if let Some(a) = &self.at {
            write!(f, "{}:{}:{}: ", a.file, a.line, a.col)?;
        }
        for part in [self.module.as_deref(), self.layer].into_iter().flatten() {
            write!(f, "{part}: ")?;
        }
        match &self.kind {
            Kind::Verdict(c) => write!(f, "{}: ", c.0)?,
            Kind::Refusal(_) => f.write_str("slice: ")?,
        }
        f.write_str(&self.message)?;
        match &self.help {
            Some(h) => write!(f, "\n  = help: {h}"),
            None => Ok(()),
        }
    }
}

/// A form of the IR the reference interpreter, the tests' oracle, can run (contract 07).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    /// Instances as linked.
    Raw,
    /// After function values are made explicit.
    Canonical,
}

/// A stage of the IR that `cpl-next dump-ir --stage` prints: lowering's generic fns, the linked
/// instances once ownership settled their copies, then canonical and optimized.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IrStage {
    Raw,
    Owned,
    Canonical,
    Optimized,
    /// Not IR: each binding's frozen and lent places, from ownership's reporting walk.
    Borrows,
}

impl IrStage {
    /// Each stage, by the name `--stage` takes.
    const NAMES: [(&str, IrStage); 5] = [
        ("raw", IrStage::Raw),
        ("owned", IrStage::Owned),
        ("canonical", IrStage::Canonical),
        ("optimized", IrStage::Optimized),
        ("borrows", IrStage::Borrows),
    ];

    pub fn named(name: &str) -> Option<IrStage> {
        (Self::NAMES.iter())
            .find(|(n, _)| *n == name)
            .map(|(_, s)| *s)
    }
}

/// Each of `fs` as its `Debug` shows it, one after another.
fn dumped<'a, T: std::fmt::Debug + 'a>(fs: impl IntoIterator<Item = &'a ir::Func<T>>) -> String {
    fs.into_iter().map(|f| format!("{f:#?}\n")).collect()
}

pub(crate) struct Compiled {
    prog: inst::Program,
    /// The line of the user's file each of the program's origins is on (`Program::origins`).
    lines: Vec<u32>,
    canon: Vec<ir::Func<inst::CTy>>,
    /// What the fast engine and the C++ run: canonical IR less the checks the facts prove.
    opt: Vec<ir::Func<inst::CTy>>,
    proved: Vec<cpl_next_ir::facts::Proved>,
    /// What a run may use, from the snapshot's judge.
    limits: Limits,
}

/// Every layer through lowering, each run whatever an earlier one found (contracts 1-3).
pub(crate) struct Front {
    loaded: modules::Loaded,
    /// What the parser could not read; a later layer says nothing there.
    rejected: Vec<SourceId>,
    d: decls::Decls,
    t: check::Typed,
    g: lower::Generic,
    main: Vec<ItemId>,
    diags: Vec<Diag>,
}

/// `es`, said by `stage`, but for what lies where the parser could not read.
fn staged(
    stage: Stage,
    es: &[diag::Msg],
    loaded: &modules::Loaded,
    rejected: &[SourceId],
) -> Vec<Diag> {
    let unread = |m: &&diag::Msg| {
        (m.at.as_ref()).is_some_and(|a| rejected.iter().any(|p| overlaps_in(p, a)))
    };
    (es.iter())
        .filter(|m| stage == Stage::Syntax || !unread(m))
        .map(|e| Diag::new(stage, e, loaded))
        .collect()
}

/// A whole build's link roots: every fn of `fns` and the first `globals` globals.
pub(crate) fn roots(fns: &HashMap<ir::FuncKey, ir::Func<ty::Ty>>, globals: usize) -> inst::Roots {
    inst::Roots {
        fns: fns.keys().cloned().collect(),
        globals: (0..globals).map(cpl_next_ir::whole::GlobalId).collect(),
    }
}

/// Link, then ownership on the linked instances (contract 4), where the program checked clean:
/// the program, its copies settled, and ownership's diagnostics.
fn back(f: &mut Front, events: bool) -> Option<(inst::Program, Vec<Diag>, Vec<own::Bound>)> {
    if !(f.loaded.errors.is_empty() && f.t.errors.is_empty() && f.d.errors.is_empty()) {
        return None;
    }
    let names: Vec<(String, bool)> = (f.t.globals.iter())
        .map(|g| (g.name.clone(), g.mutable))
        .collect();
    let g = &mut f.g;
    let d = &f.d;
    let before = own::before(d, &mut g.funcs, &mut g.globals, &names);
    let (fns, tys) = (&g.funcs, &g.globals[..]);
    let checked = Answers {
        whole: &g.whole,
        deferred: &g.deferred,
        globals: tys,
    };
    let v = inst::Inputs {
        sigs: d,
        fns,
        sums: &g.adds,
        checked: &checked,
    };
    let mut prog = inst::link(v, &roots(fns, tys.len()), events);
    let owned = own::after(d, &mut prog, &names, before);
    let diags = staged(Stage::Own, &owned.errors, &f.loaded, &f.rejected);
    Some((prog, diags, owned.binds))
}

/// `judge` is the judge the snapshot is built for; it is the tool's to resolve (contract 0).
fn front(db: &Db, std: &Std, root: &FileId, judge: Option<&Judge>, checks: Checks) -> Front {
    let loaded = modules::load(db, std, root);
    // What the parser could not read is checked as error nodes; a later layer says nothing
    // on what the parser rejected, and every other unit is checked as if it were alone.
    let rejected: Vec<SourceId> = (loaded.poisoned.iter().cloned())
        .chain(loaded.errors.iter().filter_map(|m| m.at.clone()))
        .collect();
    let diags = |stage, es: &[diag::Msg]| staged(stage, es, &loaded, &rejected);
    let mut out = diags(Stage::Syntax, &loaded.errors);
    let (d, main) = decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
    out.extend(diags(Stage::Decls, &d.errors));
    let values = consts::Evaluator::new(&d, &loaded, &main);
    let t = check::check(
        &d,
        check::Program {
            main: &main,
            ..Default::default()
        },
        &loaded,
        &values,
        Build {
            judge: judge.cloned(),
            checks,
        },
        &check::Ids::default(),
    );
    out.extend(diags(Stage::Check, &t.errors));
    let g = lower::lower(&d, &t, &loaded);
    out.extend(diags(Stage::Lower, &g.errors));
    Front {
        loaded,
        rejected,
        d,
        t,
        g,
        main,
        diags: out,
    }
}

/// Reads into `db` each file `missing` says a load lacks, from disk, until a pass reads
/// none: what is still missing then cannot be read, and the load says so. Each pass but the
/// last reads a file `db` lacked, so there are at most as many passes as files. It sets
/// inputs, so no query calls it. Returns the files it read.
pub(crate) fn read_missing(
    db: &Db,
    missing: impl Fn() -> Vec<PathBuf>,
) -> Result<Vec<FileId>, DbError> {
    let mut read = Vec::new();
    loop {
        let before = read.len();
        for path in missing() {
            if let Ok(text) = std::fs::read_to_string(&path) {
                let file = FileId::new(&path.display().to_string());
                db.set::<Source>(file.clone(), text.into())?;
                read.push(file);
            }
        }
        if read.len() == before {
            return Ok(read);
        }
    }
}

/// Each std module and the diagnostics it has when every one of its fns is checked, not
/// only those a program reaches: std as a program (suite 4 of plan/end-state.toml).
pub(crate) fn std_unchecked(
    std: &Std,
    judge: Option<&Judge>,
    checks: Checks,
) -> Vec<(String, Vec<Diag>)> {
    let db = &Db::default();
    (std.module_names())
        .filter(|m| Some(*m) != std.prelude_module.as_deref())
        .map(|m| {
            // The program that only uses `m`, in a file named for it.
            let root = FileId::new(m);
            let loaded = match db.set::<Source>(root.clone(), format!("use {m}\n").into()) {
                Ok(()) => modules::load(db, std, &root),
                Err(e) => return (m.to_string(), vec![Diag::internal(&e)]),
            };
            let (d, main) =
                decls::collect(std.decls(), &loaded.modules, &loaded.seen, &loaded.prelude);
            let at = (d.module_names.iter())
                .position(|n| n == m)
                .map(decls::ModId);
            let values = consts::Evaluator::new(&d, &loaded, &main);
            let t = check::check(
                &d,
                check::Program {
                    main: &main,
                    values: &[],
                    roots: &at.into_iter().collect::<Vec<_>>(),
                },
                &loaded,
                &values,
                Build {
                    judge: judge.cloned(),
                    checks,
                },
                &check::Ids::default(),
            );
            let ds = (staged(Stage::Decls, &d.errors, &loaded, &[]).into_iter()).chain(staged(
                Stage::Check,
                &t.errors,
                &loaded,
                &[],
            ));
            let own = |d: &Diag| {
                d.at.as_ref()
                    .map(|a| a.file.as_str())
                    .or(d.module.as_deref())
                    == Some(m)
            };
            (m.to_string(), ds.filter(own).collect())
        })
        .collect()
}

/// `events` adds the semantic events to the program, for both the interpreter and the C++.
pub(crate) fn compile(
    db: &Db,
    std: &Std,
    root: &FileId,
    events: bool,
    judge: Option<&Judge>,
    checks: Checks,
) -> Result<Compiled, Vec<Diag>> {
    let mut f = front(db, std, root, judge, checks);
    let Some((mut prog, own, _)) = back(&mut f, events) else {
        return Err(f.diags);
    };
    f.diags.extend(own);
    f.diags.sort_by_key(|d| d.stage);
    if !f.diags.is_empty() {
        return Err(f.diags);
    }
    let link = |es: &[diag::Msg]| -> Vec<Diag> {
        (es.iter())
            .map(|e| Diag::new(Stage::Link, e, &f.loaded))
            .collect()
    };
    if !prog.errors.is_empty() {
        return Err(link(&prog.errors));
    }
    if events {
        let answers = cpl_next_decls::views::Answers {
            whole: &f.g.whole,
            deferred: &f.g.deferred,
            globals: &f.g.globals,
        };
        inst::add_events(&mut prog, &answers);
    }
    let canon =
        inst::canonicalize(&mut prog, &cpl_next_own::expand_calls).map_err(|es| link(&es))?;
    let (opt, proved) = cpl_next_ir::opt::optimize(&canon, &prog);
    {
        use cpl_next_ir::verify::{accounted, instances, Stage};
        let g = prog.globals.len();
        let mut bad = instances(&canon, Stage::Canonical, g);
        bad.extend(instances(&opt, Stage::Optimized, g));
        bad.extend(
            (canon.iter().zip(&opt).enumerate())
                .flat_map(|(i, (c, o))| accounted(i, c, o, &proved))
                .map(diag::Msg::internal),
        );
        if !bad.is_empty() {
            return Err(link(&bad));
        }
    }
    let lines = (prog.origins.iter())
        .map(|o| f.loaded.program_line(o))
        .collect();
    Ok(Compiled {
        prog,
        lines,
        canon,
        opt,
        proved,
        limits: Limits::of(judge),
    })
}

/// An item a program can name, as hover and definition show it (contract 9), read through the
/// Scope and Signatures views.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The module it is declared in.
    pub module: String,
    /// Where its declaration is in that module's text.
    pub span: cpl_diag::Span,
    /// The 1-based line its declaration starts on.
    pub line: u32,
    /// Its first line past any decorators, as written, less a trailing `:`:
    /// `fn twice(n: Int) -> Int`.
    pub head: String,
    /// The `///` comment above it, past its decorators.
    pub doc: Option<String>,
    /// Whether it is a fn, so `head` is its signature.
    pub function: bool,
}

/// The item whose declaration starts at `at`.
fn item_of(f: &Front, at: &SourceId, function: bool) -> Item {
    let text = f.loaded.text_of(at);
    let span = text.span(at);
    let (line, col) = text.line_col(span.start);
    let from = (span.start + 1 - col) as usize;
    let head = (text.text[from..].lines())
        .find(|l| !l.trim_start().starts_with('@'))
        .unwrap_or("")
        .trim_end()
        .trim_end_matches(':')
        .to_string();
    let doc = cpl_next_syntax::doc::doc_above(&text.text, span.start as usize);
    Item {
        module: text.name.clone(),
        span,
        line,
        head,
        doc: (!doc.is_empty()).then_some(doc),
        function,
    }
}

fn item_start(item: &ItemId) -> SourceId {
    item.at(cpl_diag::Span::new(0, 0))
}

/// Each name the program's module scope gives an item with a declaration, and that item.
pub(crate) fn items(f: &Front) -> HashMap<String, Item> {
    use cpl_next_decls::views::TypeTables;
    let declared = |named: decls::Named| -> Option<SourceId> {
        Some(match named {
            decls::Named::Fn(id) => f.d.f(id).span.clone(),
            decls::Named::Struct(id) => item_start(&f.d.s(id).item),
            decls::Named::Alias(id) => f.d.alias(id).at.clone(),
            decls::Named::Const(id) => item_start(&f.d.consts[id.0].item),
            _ => return None,
        })
    };
    let program = f.d.program;
    (f.d.views[program.0].iter())
        .filter_map(|(name, x)| {
            let at = declared(*x)?;
            let function = matches!(x, decls::Named::Fn(_));
            Some((f.d.text(*name).to_string(), item_of(f, &at, function)))
        })
        .collect()
}

/// An expression of the program's own text, by its span, and its type.
pub(crate) struct TypedAt {
    pub(crate) span: cpl_diag::Span,
    ty: ty::Ty,
    /// The type parameters in scope where it is.
    generics: Vec<cpl_next_decls::sym::Sym>,
}

/// Each expression the program's own text holds.
pub(crate) fn types(f: &Front) -> Vec<TypedAt> {
    use cpl_next_decls::views::Items;
    let mut out = Vec::new();
    // A typed expression's span is from the start of the item it is in.
    let mut add = |item: &ItemId, generics: &[cpl_next_decls::sym::Sym], e: &check::TExpr| {
        let at = item.at(e.span);
        out.push(TypedAt {
            span: f.loaded.text_of(&at).span(&at),
            ty: e.ty.clone(),
            generics: generics.to_vec(),
        });
    };
    let ours = |item: &ItemId| (*item.module == *modules::PROGRAM).then(|| item.clone());
    // Each run of top-level statements follows the `Item` naming the item it is from.
    let mut item = None;
    for s in &f.t.main {
        match (s, &item) {
            (check::TStmt::Item(it), _) => item = ours(it),
            (s, Some(it)) => s.each(&mut |e| add(it, &[], e)),
            _ => {}
        }
    }
    for (id, typed) in &f.t.fns {
        let fd = Items::f(&f.d, *id);
        if let Some(it) = ours(&fd.span.item) {
            typed.body.each(&mut |e| add(&it, &fd.generics, e));
        }
    }
    out
}

/// The type of an expression, as tools read it (contract 9): it names no std type, since
/// std's types are ordinary declarations a tool reaches through [`Type::decl`].
#[derive(Clone, Copy)]
pub struct Type<'s> {
    front: &'s Front,
    ty: &'s ty::Ty,
    generics: &'s [cpl_next_decls::sym::Sym],
}

impl<'s> Type<'s> {
    pub(crate) fn new(front: &'s Front, at: &'s TypedAt) -> Type<'s> {
        Type {
            front,
            ty: &at.ty,
            generics: &at.generics,
        }
    }

    /// What it is past a lend, which a value's type does not show.
    fn shown(&self) -> &'s ty::Ty {
        let mut ty = self.ty;
        while let ty::Ty::Lent(inner) = ty {
            ty = inner;
        }
        ty
    }

    /// As a diagnostic writes it.
    pub fn text(&self) -> String {
        let f = self.front;
        check::show_ty(
            &f.d,
            f.d.roles.option,
            &f.t.domains,
            self.generics,
            &|t| t.clone(),
            self.ty,
        )
    }

    /// An integer's ends, `None` where it has none the checker knows.
    pub fn range(&self) -> Option<(Option<i64>, Option<i64>)> {
        match self.shown() {
            ty::Ty::Range(lo, hi) => Some((Some(*lo), Some(*hi))),
            ty::Ty::Int | ty::Ty::RangeOf(_) | ty::Ty::AnyRange(_) | ty::Ty::RangeTo(..) => {
                Some((None, None))
            }
            _ => None,
        }
    }

    /// The domain it is a position of, by name.
    pub fn domain(&self) -> Option<&'s str> {
        match self.shown() {
            ty::Ty::Dom(d) => (self.front.t.domains.get(*d)).map(|d| d.name.as_str()),
            _ => None,
        }
    }

    /// The struct or enum it is an instance of.
    pub fn decl(&self) -> Option<Item> {
        use cpl_next_decls::views::Items;
        match self.shown() {
            ty::Ty::Struct(id, ..) => {
                let s = Items::s(&self.front.d, *id);
                Some(item_of(self.front, &item_start(&s.item), false))
            }
            _ => None,
        }
    }

    /// Whether it implements the trait the program's scope names `name`, as the checker's
    /// impl lookup answers (the Signatures view).
    pub fn implements(&self, name: &str) -> bool {
        use cpl_next_decls::views::{Names, Signatures};
        let d = &self.front.d;
        match d.syms.get(name).and_then(|s| Names::named(d, d.program, s)) {
            Some(decls::Named::Trait(tr)) => d.impls().has_impl(self.shown(), tr),
            _ => false,
        }
    }

    /// What a `.` after a value of this type can name: its fields, its own fns, and the fns of
    /// each impl the checker's lookup finds for it, by name.
    pub fn members(&self) -> Vec<(String, Item)> {
        use cpl_next_decls::views::{Items, Signatures};
        let (f, ty) = (self.front, self.shown());
        let mut out = Vec::new();
        let fn_item = |id| item_of(f, &Items::f(&f.d, id).span, true);
        if let ty::Ty::Struct(id, ..) = ty {
            let s = Items::s(&f.d, *id);
            for ((name, _), at) in s.fields.iter().zip(&s.field_at) {
                out.push((f.d.text(*name).to_string(), item_of(f, at, false)));
            }
            out.extend(
                s.methods
                    .iter()
                    .map(|(name, id)| (f.d.text(*name).to_string(), fn_item(*id))),
            );
        }
        for i in 0..f.d.impls.len() {
            let im = Items::imp(&f.d, i);
            if !im.replaced
                && f.d.impls().impl_args(im, ty).is_some()
                && f.d.impls().has_impl(ty, im.trait_.tr)
            {
                out.extend(
                    im.methods
                        .iter()
                        .map(|(name, id)| (f.d.text(*name).to_string(), fn_item(*id))),
                );
            }
        }
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out.dedup_by(|a, b| a.0 == b.0);
        out
    }

    /// That declaration's type arguments.
    pub fn args(&self) -> Vec<Type<'s>> {
        match self.shown() {
            ty::Ty::Struct(_, args, _) => (args.iter()).map(|ty| Type { ty, ..*self }).collect(),
            _ => Vec::new(),
        }
    }
}

/// One unit of the program (contract 3): a fn it declares, or one of its top-level statements.
#[derive(Debug)]
pub struct Unit {
    /// The fn's name; `None` for a statement.
    pub name: Option<String>,
    /// Where it is in the program's text.
    pub span: cpl_diag::Span,
    /// The diagnostics placed in it.
    pub diags: Vec<Diag>,
    /// How many of its expressions the checker could not type (error nodes).
    pub untyped: usize,
    /// Whether lowering made its code: a fn's own function, a statement's part of `main`.
    pub lowered: bool,
}

/// `b` as `--dump-own` prints it: the fn and instance, the binding, its frozen and lent places,
/// each named by its variable (`$` and an index for a temp) and then its path.
fn bound_line(p: &inst::Program, globals: &[String], b: &own::Bound) -> String {
    let f = &p.insts[b.inst];
    let place = |pl: &own::Pl| {
        let mut out = match pl.root {
            own::Root::Local(l) => match f.names.get(l) {
                Some(n) if !n.is_empty() => n.clone(),
                _ => format!("${l}"),
            },
            own::Root::Global(g) => (globals.get(g).cloned()).unwrap_or_else(|| format!("$g{g}")),
        };
        for s in &pl.path {
            match s {
                own::Step::Field(k) => out.push_str(&format!(".{k}")),
                own::Step::Key(k) => out.push_str(&format!("[{k}]")),
            }
        }
        out
    };
    let all = |ps: &[own::Pl]| ps.iter().map(place).collect::<Vec<_>>().join(", ");
    let local = place(&own::Pl {
        root: b.local,
        path: Vec::new(),
    });
    let (fr, le) = (all(&b.frozen), all(&b.lent));
    format!("{}#{} {local}: frozen [{fr}] lent [{le}]\n", f.name, b.inst)
}

/// What `compile` checks and lowers, unit by unit, and every diagnostic: the editor's view of a
/// program that need not compile.
pub(crate) fn units(
    db: &Db,
    std: &Std,
    root: &FileId,
    judge: Option<&Judge>,
    checks: Checks,
) -> (Vec<Unit>, Vec<Diag>) {
    let mut f = front(db, std, root, judge, checks);
    if let Some((_, own, _)) = back(&mut f, false) {
        f.diags.extend(own);
        f.diags.sort_by_key(|d| d.stage);
    }
    let fns = (0..f.d.fns.len()).map(ty::FnId).filter(|id| {
        let fd = f.d.f(*id);
        // A derived impl's methods are made, not written: they are the struct's.
        let derived = matches!(fd.owner, Some(decls::Owner::Impl(i)) if f.d.impls[i].derived);
        f.d.of_program(fd.module) && !derived && fd.intrinsic.is_none() && !fd.bodiless()
    });
    let fns = fns.map(|id| {
        let fd = f.d.f(id);
        let lowered = f.g.funcs.contains_key(&ir::FuncKey::Decl(id));
        (Some(fd.name.clone()), fn_extent(db, &fd.span), lowered)
    });
    let statements = check::program_items(&f.loaded, &f.main)
        .into_iter()
        .map(|(off, s)| {
            let span = off.at(cpl_next_syntax::ast::AstNode::syntax(&s).span());
            (None, span, f.g.funcs.contains_key(&ir::FuncKey::Main))
        });
    let units = (fns.chain(statements))
        .map(|(name, span, lowered)| {
            let module = f.loaded.text_of(&span);
            let mut local = module.span(&span);
            // A statement owns its line's end, where what it is missing is said.
            let at_end = module.text.as_bytes().get(local.end as usize);
            if name.is_none() && at_end == Some(&b'\n') {
                local.end += 1;
            }
            let diags = (f.diags.iter())
                .filter(|d| {
                    (d.at.as_ref())
                        .is_some_and(|a| a.file == modules::PROGRAM && overlaps(a.span, local))
                })
                .cloned()
                .collect();
            Unit {
                name,
                span: local,
                diags,
                untyped: f.g.untyped.iter().filter(|u| overlaps_in(u, &span)).count(),
                lowered,
            }
        })
        .collect();
    (units, f.diags)
}

impl Compiled {
    /// The instances holding an error type (`cpl_next_ir::verify::error_typed`).
    pub fn error_typed(&self) -> Vec<String> {
        let callees = cpl_next_ir::verify::callees(&self.prog.insts);
        cpl_next_ir::verify::error_typed(&self.prog.insts, &callees, self.prog.globals.len())
    }

    /// Runs on the engine, the one evaluator, on the calling thread.
    pub fn run(&self, input: &str) -> Outcome {
        self.run_on(self.input(input))
    }

    /// Runs `input` on the engine, on the calling thread.
    pub fn run_on(&self, input: RunInput) -> Outcome {
        cpl_next_engine::run(&self.prog, &self.opt, input)
    }

    /// A run that reads `from` as the program reads and writes `to` as it flushes, under the
    /// limits of the judge the program was built for: a terminal, a pipe or an interactor.
    pub fn streamed(
        &self,
        from: Box<dyn std::io::Read + Send>,
        to: Box<dyn std::io::Write + Send>,
    ) -> RunInput {
        RunInput {
            io: Io::Stream(from),
            limits: self.limits,
            out: Some(to),
        }
    }

    /// A run of all of `input` under the limits of the judge the program was built for.
    pub fn input(&self, input: &str) -> RunInput {
        RunInput {
            limits: self.limits,
            ..RunInput::fixed(input)
        }
    }

    /// The program and fns the reference interpreter runs in `form`; only a test runs it.
    pub fn form(&self, form: Form) -> (&inst::Program, &[ir::Func<inst::CTy>]) {
        match form {
            Form::Raw => (&self.prog, &self.prog.insts),
            Form::Canonical => (&self.prog, &self.canon),
        }
    }

    /// The line of the user's file origin `o` is on; 0 for none.
    pub fn line(&self, o: u32) -> u32 {
        (o as usize).checked_sub(1).map_or(0, |i| self.lines[i])
    }

    /// What a trap writes to stderr: its code, line and message, then its call stack.
    pub fn said(&self, t: &Trap) -> String {
        let mut s = format!(
            "trap: {}: line {}: {}\n",
            t.code(),
            self.line(t.origin),
            t.msg
        );
        let stack = cpl_next_ir::run::call_stack(&self.prog.insts, t.calls.iter().copied(), |o| {
            self.line(o)
        });
        if stack.len() > 1 {
            s += &format!("call stack: {}\n", stack.join(" <- "));
        }
        s
    }

    pub fn cpp(&self) -> String {
        cpp::emit(&self.prog, &self.opt, &self.lines, &self.limits)
    }

    /// Every `@fast` fn runs its CP body instead of its op, which is what the fn means; the
    /// op is only a faster way to compute it, and `fast_ops_agree_with_their_bodies` holds
    /// the two together.
    pub fn without_fast(&mut self) {
        for f in (self.prog.insts.iter_mut())
            .chain(&mut self.canon)
            .chain(&mut self.opt)
        {
            f.fast = None;
        }
    }

    /// The checks optimized IR leaves out, each with the facts' reason being its site.
    pub fn proved(&self) -> &[cpl_next_ir::facts::Proved] {
        &self.proved
    }
}

#[cfg(test)]
mod tests;
