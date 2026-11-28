//! Contract 1: snapshot -> modules. Finds the modules a program needs. Each keeps its own text,
//! and each of its items has a path (`ItemId`), so a place is an item and a span inside it.
//!
//! A module is loaded when a name from it can be meant: one the program's `use` lines name,
//! one the prelude names (R11.3), or one a loaded module's own `use` lines name. A method
//! resolves on its receiver's type, wherever the program spells it (R11.3.4). Libraries get
//! no prelude (R11.3.1).
use crate::diag::Msg;
use cpl_diag::Span;
use cpl_next_db::db::Db;
use cpl_next_db::items::{self as queries, ItemAt, ItemOf, ModuleFile, Parse, Source, SOURCE_EXT};
use cpl_next_ir::ids::FileId;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_next_syntax::ast::{self, AstNode};
use cpl_next_syntax::{NodeKind, SyntaxNode};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An item as the parser split it: its path and its syntax, rooted at the item so that its
/// spans count from the item's start.
pub struct Item {
    pub id: ItemId,
    pub item: ast::Item,
}

type Items = Vec<Item>;
/// Each syntax error with its code and where it is, and the places the parser could not read.
type Unread = (Vec<Msg>, Vec<SourceId>);

pub struct Module {
    pub name: String,
    pub items: Arc<Items>,
    /// The module an inline `mod name:` sits in (R11.1.3).
    pub within: Option<String>,
}

/// A module's text, and where each of its items starts in it.
#[derive(Clone)]
pub struct Text {
    pub name: String,
    pub text: Arc<str>,
    /// Where each of its lines starts in `text`.
    lines: Arc<[u32]>,
    starts: Arc<HashMap<ItemId, u32>>,
}

impl Text {
    fn new(name: &str, text: Arc<str>, starts: HashMap<ItemId, u32>) -> Text {
        let lines = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i as u32 + 1))
            .collect();
        Text {
            name: name.to_string(),
            text,
            lines,
            starts: Arc::new(starts),
        }
    }
    /// The 1-based line and column of `pos`.
    pub fn line_col(&self, pos: u32) -> (u32, u32) {
        let line = self.lines.partition_point(|s| *s <= pos);
        (line as u32, pos - self.lines[line - 1] + 1)
    }
    /// `at`'s span in this text.
    pub fn span(&self, at: &SourceId) -> Span {
        let start = self.starts.get(&at.item).copied().unwrap_or(0);
        Span::new(start + at.span.start, start + at.span.end)
    }

    /// Byte `pos` of the text, from the start of the last item that starts at or before it.
    pub fn at_offset(&self, pos: u32) -> Option<SourceId> {
        let (id, start) = (self.starts.iter())
            .filter(|(_, start)| **start <= pos)
            .max_by_key(|(id, start)| (**start, id.name.clone(), id.nth))?;
        Some(id.at(Span::new(pos - start, pos - start)))
    }
}

/// A module read and parsed.
struct Parsed {
    text: Text,
    items: Arc<Items>,
    unread: Unread,
    /// Each inline `mod`'s module and the module it sits in, inner ones first.
    inline: Vec<(String, String, Arc<Items>)>,
}

impl Parsed {
    fn module(&self) -> Module {
        Module {
            name: self.text.name.clone(),
            items: self.items.clone(),
            within: None,
        }
    }
    /// Its inline modules, then itself: each with the file's text, under its own name.
    fn modules(&self) -> impl Iterator<Item = (Text, Module)> + '_ {
        let inline = self.inline.iter().map(|(name, within, items)| {
            let text = Text {
                name: name.clone(),
                ..self.text.clone()
            };
            let m = Module {
                name: name.clone(),
                items: items.clone(),
                within: Some(within.clone()),
            };
            (text, m)
        });
        inline.chain(std::iter::once((self.text.clone(), self.module())))
    }
    /// Module `name`, whose text `file` holds, read through `db`'s item queries; `None` when
    /// `db` has no `Source` for `file`.
    fn read(db: &Db, file: FileId, name: &str) -> Option<Parsed> {
        // A failed read is reported by `parse`, which asks again.
        let text = db.input::<Source>(&file).unwrap_or_default()?;
        let FileItems {
            items,
            starts,
            unread,
            inline,
        } = parse(db, file, name);
        Some(Parsed {
            text: Text::new(name, text, starts),
            items: Arc::new(items),
            unread,
            inline,
        })
    }
    /// Module `name` with no text: a program `db` lacks, or std's db refusing a file.
    fn empty(name: &str, unread: Unread) -> Parsed {
        Parsed {
            text: Text::new(name, "".into(), HashMap::new()),
            items: Arc::new(Vec::new()),
            unread,
            inline: Vec::new(),
        }
    }
}

/// Std, read once and never written after: the core and every std module, each parsed at a
/// place no program moves, and their declarations. Every program loads its std modules from
/// one `Std`, and places its own modules after `Std::end`.
pub struct Std {
    /// The core first, then std's modules by name.
    modules: Vec<Parsed>,
    index: HashMap<String, usize>,
    decls: crate::decls::Prepared,
    /// Each prelude name and the module it is from.
    prelude: HashMap<String, String>,
    /// The module marked `@role(prelude)`, whose `use`s every program has (R11.6).
    pub prelude_module: Option<String>,
}

impl Std {
    /// The std this binary was built with.
    pub fn read() -> Std {
        Std::from_files(
            STD_FILES
                .iter()
                .map(|(m, t)| (m.to_string(), t.to_string()))
                .collect(),
        )
    }

    /// The std in the directory `root`, in place of the one this binary was built with.
    pub fn read_dir(root: &Path) -> Std {
        Std::from_files(read_dir_modules(root))
    }

    fn from_files(mut files: Vec<(String, String)>) -> Std {
        const CORE_TEXT: &str = include_str!("../../cpl-next/std.cp");
        files.sort();
        let placed = std::iter::once((CORE.to_string(), CORE_TEXT.to_string())).chain(files);
        // Each module is read alone, so they are parsed side by side.
        let db = &Db::default();
        let modules: Vec<Parsed> = std::thread::scope(|scope| {
            let parsing: Vec<_> = placed
                .map(|(name, text)| {
                    let file = FileId::new(&std_path(&name));
                    scope.spawn(move || match db.set::<Source>(file.clone(), text.into()) {
                        Ok(()) => Parsed::read(db, file, &name)
                            .unwrap_or_else(|| Parsed::empty(&name, Default::default())),
                        Err(e) => {
                            Parsed::empty(&name, (vec![Msg::internal(e.to_string())], vec![]))
                        }
                    })
                })
                .collect();
            (parsing.into_iter())
                .map(|p| p.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
                .collect()
        });
        let index = (modules.iter().enumerate())
            .map(|(i, p)| (p.text.name.clone(), i))
            .collect::<HashMap<_, _>>();
        let mut prelude = HashMap::new();
        if let Some(p) = modules.iter().find(|p| is_prelude(&p.items)) {
            for (module, u, _) in uses(&p.items) {
                for n in u.names().iter().flat_map(ast::UseNames::names) {
                    prelude.insert(n.text().to_string(), module.clone());
                }
            }
        }
        let decls = crate::decls::prepare(&modules.iter().map(Parsed::module).collect::<Vec<_>>());
        let prelude_module = (modules.iter())
            .find(|p| is_prelude(&p.items))
            .map(|p| p.text.name.clone());
        Std {
            modules,
            index,
            decls,
            prelude,
            prelude_module,
        }
    }
    pub fn decls(&self) -> &crate::decls::Prepared {
        &self.decls
    }
    /// Std's modules by name, the core not among them.
    pub fn module_names(&self) -> impl Iterator<Item = &str> {
        self.modules[1..].iter().map(|p| p.text.name.as_str())
    }
}

pub struct Loaded {
    /// Each module's text, in the order of `modules`.
    pub texts: Vec<Text>,
    /// The core first, then every std module, the program's libraries, and the program last.
    pub modules: Vec<Module>,
    /// Per module, whether the program loads it.
    pub seen: Vec<bool>,
    /// Each prelude name and the module it is from.
    pub prelude: HashMap<String, String>,
    pub errors: Vec<Msg>,
    /// What the parser could not read: no later layer reports inside these (contract 3).
    pub poisoned: Vec<SourceId>,
    /// Each module whose file the db has no `Source` for, and that file. Loading treats it as
    /// absent; the caller reads the files it can, sets their `Source` and loads again.
    pub missing: Vec<(String, PathBuf)>,
    /// Each module's place in `texts`.
    index: HashMap<String, usize>,
}

impl Loaded {
    /// The text of the module `at` is in.
    pub fn text_of(&self, at: &SourceId) -> &Text {
        let i = self.index.get(&*at.item.module).copied();
        &self.texts[i.unwrap_or(0)]
    }
    /// The source text `at` covers.
    pub fn text(&self, at: &SourceId) -> &str {
        let (t, s) = (self.text_of(at), self.text_of(at).span(at));
        t.text
            .get(s.start as usize..s.end.max(s.start) as usize)
            .unwrap_or("")
    }
    /// The program's text.
    pub fn program(&self) -> &str {
        &self.texts[self.index[PROGRAM]].text
    }
    /// The program's 1-based line `at` is on; 0 outside the program.
    pub fn program_line(&self, at: &SourceId) -> u32 {
        match &*at.item.module == PROGRAM {
            true => {
                (self.text_of(at))
                    .line_col(self.text_of(at).span(at).start)
                    .0
            }
            false => 0,
        }
    }
}

pub const CORE: &str = "core";
pub const PROGRAM: &str = "program";

/// Where each library's modules are: the roots the nearest `cp.toml` at or above the program
/// names under `[libs]` (R11.2.1). `std` is the pinned stdlib, read into a `Std`.
pub struct Roots(Vec<(String, PathBuf)>);

/// Std's modules as `std/` was when this binary was built, by module name.
const STD_FILES: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/std_files.rs"));

/// The file std module `name` is read from, from std's parent: `std.graph` is `std/graph.cp`,
/// the name `read_dir_modules` gives it.
pub fn std_path(name: &str) -> String {
    format!("{}.{SOURCE_EXT}", name.replace('.', "/"))
}

/// The modules of a std directory, by module name.
fn read_dir_modules(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut dirs = vec![(root.to_path_buf(), "std".to_string())];
    while let Some((dir, module)) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let stem = path
                .file_stem()
                .map_or(String::new(), |s| s.to_string_lossy().into_owned());
            if path.is_dir() {
                dirs.push((path, format!("{module}.{stem}")));
            } else if path.extension().is_some_and(|e| e == SOURCE_EXT) {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    files.push((format!("{module}.{stem}"), text));
                }
            }
        }
    }
    files
}

impl Roots {
    /// No library but std.
    pub fn std() -> Roots {
        Roots(Vec::new())
    }
    /// `std` and the libraries a `cp.toml` at or above `dir` names; a nearer one wins a name.
    pub fn from(dir: &Path) -> Roots {
        let Roots(mut roots) = Roots::std();
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let read = &mut |f: &Path| std::fs::read_to_string(f).ok();
        cpl_config::find_map(&dir, read, |at, config| {
            for line in cpl_config::table(config, "libs") {
                let Some((name, value)) = line.pair else {
                    continue;
                };
                let path = value.strip_prefix('"').and_then(|v| v.split('"').next());
                if let Some(path) = path {
                    if name != "std" && !roots.iter().any(|(n, _)| n == name) {
                        roots.push((name.to_string(), at.join(path)));
                    }
                }
            }
            None::<()>
        });
        Roots(roots)
    }
    /// The `Libraries` input of a program these roots are for.
    pub fn libraries(&self) -> Arc<[(Arc<str>, PathBuf)]> {
        self.0
            .iter()
            .map(|(n, p)| (n.as_str().into(), p.clone()))
            .collect()
    }
}

/// Each `use` line's module, the line, and where it is written.
/// Whether a module is marked `@role(prelude)`: a decorator above its first `use` (R11.6).
fn is_prelude(items: &[Item]) -> bool {
    let Some(ast::Decl::Use(u)) = items.first().and_then(|it| it.item.decl()) else {
        return false;
    };
    let marked = u.decorators().any(|x| {
        let attr = x
            .name()
            .and_then(|n| crate::attrs::Decorator::live(n.text()));
        attr.and_then(|attr| {
            crate::decls::form_role(&crate::decls::forms::AttrForm {
                attr,
                node: Some(x),
            })
        }) == Some(crate::decls::Role::Prelude)
    });
    marked
}

fn uses(items: &[Item]) -> impl Iterator<Item = (String, ast::UseDecl, SourceId)> + '_ {
    items.iter().filter_map(|it| match it.item.decl()? {
        ast::Decl::Use(u) => {
            let path = u.path().map(|t| t.text().to_string());
            let at = SourceId {
                item: it.id.clone(),
                span: u.syntax().span(),
            };
            Some((path.collect::<Vec<_>>().join("."), u, at))
        }
        _ => None,
    })
}

/// What [`parse`] reads from one file.
struct FileItems {
    items: Items,
    /// Where each item starts in the file's text.
    starts: HashMap<ItemId, u32>,
    /// Each syntax error with its code and where it is, and the places the parser could not read.
    unread: Unread,
    /// Each inline `mod`: its module, the module it is written in, and its items.
    inline: Vec<(String, String, Arc<Items>)>,
}

/// The items of module `module`, whose text `file` holds, where each starts in the text, each
/// syntax error with its code and where it is, and the places the parser could not read. The
/// items and their ids are `db`'s `items`, `item` and `item_at` answers; this only reads.
fn parse(db: &Db, file: FileId, module: &str) -> FileItems {
    let mut out = FileItems {
        items: Vec::new(),
        starts: HashMap::new(),
        unread: (Vec::new(), Vec::new()),
        inline: Vec::new(),
    };
    let read = || -> Result<_, cpl_next_db::db::DbError> {
        let mut items = Vec::new();
        for id in db.get::<queries::Items>(&(file.clone(), module.into()))? {
            let (Some(it), Some(at)) = (db.get::<ItemOf>(&id)?, db.get::<ItemAt>(&id)?) else {
                continue;
            };
            if let Some(item) = ast::Item::cast(SyntaxNode::new_root(it.green)) {
                items.push((Item { id, item }, at));
            }
        }
        let diags = db.get::<Parse>(&file)?.map(|p| p.diagnostics);
        Ok((items, diags.unwrap_or_default()))
    };
    let (items, diags) = match read() {
        Ok(r) => r,
        Err(e) => {
            out.unread.0.push(Msg::internal(e.to_string()));
            return out;
        }
    };
    let mut starts: Vec<(ItemId, u32)> = items.iter().map(|(i, at)| (i.id.clone(), *at)).collect();
    starts.sort_by_key(|(_, at)| *at);
    for (it, _) in &items {
        out.unread
            .1
            .extend(unread(it.item.syntax()).map(|span| it.id.at(span)));
    }
    // `items` is in the order `items(FileId)` gives: an inline `mod m:` (R11.1.3), then its
    // body's items, named `m.name`, which are a module of their own.
    let mut items = items.into_iter().map(|(i, _)| i);
    let mut inline = Vec::new();
    while let Some(it) = items.next() {
        let inner = it.item.decl().and_then(|d| match d {
            ast::Decl::Mod(m) => Some(m),
            _ => None,
        });
        if let Some(m) = inner {
            take_mod(&m, &mut items, module, &mut inline);
        }
        out.items.push(it);
    }
    out.inline = inline;
    let file_id = ItemId::file(file.clone(), module.into());
    // A file's place, as the item it falls in.
    let within = |span: Span| match starts.iter().rev().find(|(_, o)| *o <= span.start) {
        Some((id, o)) => id.at(Span::new(span.start - o, span.end - o)),
        None => file_id.at(span),
    };
    out.unread.0.extend(diags.iter().map(|d| {
        Msg {
            help: d.help.clone(),
            fixes: (d.fixes.iter())
                .map(|f| cpl_next_ir::msg::Fix {
                    title: f.title.clone(),
                    edits: (f.edits.iter())
                        .map(|(s, t)| (within(*s), t.clone()))
                        .collect(),
                })
                .collect(),
            ..Msg::at(within(d.span), Msg::verdict(d.code, d.message.clone()))
        }
    }));
    out.starts = starts.into_iter().collect();
    out
}

/// The items of inline module `m`, written in module `at`: each next in `items`, with the
/// modules inside it after it.
fn take_mod(
    m: &ast::ModDecl,
    items: &mut impl Iterator<Item = Item>,
    at: &str,
    inline: &mut Vec<(String, String, Arc<Items>)>,
) {
    let name = m.name().map_or(String::new(), |t| t.text().to_string());
    let module = match at == PROGRAM {
        true => name,
        false => format!("{at}.{name}"),
    };
    let slot = inline.len();
    inline.push((module.clone(), at.to_string(), Arc::new(Vec::new())));
    let mut own = Vec::new();
    for _ in m.items() {
        let Some(it) = items.next() else {
            break;
        };
        if let Some(ast::Decl::Mod(inner)) = it.item.decl() {
            take_mod(&inner, items, &module, inline);
        }
        own.push(it);
    }
    inline[slot].2 = Arc::new(own);
}

/// The places in `item` the parser could not read, item-relative, leaving out the items of an
/// inline module's body, which are items of their own.
fn unread(item: &SyntaxNode) -> impl Iterator<Item = Span> {
    let mut out = Vec::new();
    let mut stack: Vec<SyntaxNode> = item.children().collect();
    while let Some(n) = stack.pop() {
        match n.kind() {
            NodeKind::Item => {}
            NodeKind::Error => out.push(n.span()),
            _ => stack.extend(n.children()),
        }
    }
    out.sort_by_key(|s| (s.start, s.end));
    out.into_iter()
}

/// The program in file `root` and every module it reaches from `std` and the libraries `db`'s
/// `Libraries(root)` names, each file read through `db`'s item queries, so a session's edit
/// re-parses only the file it touches. It only reads `db`: a file without a `Source` is in
/// `Loaded::missing`.
pub fn load(db: &Db, std: &Std, root: &FileId) -> Loaded {
    enum Got {
        Std(usize),
        Lib(Parsed),
    }
    let mut errors = Vec::new();
    let prelude = std.prelude.clone();
    let mut missing_files: Vec<(String, PathBuf)> = Vec::new();
    let program = Parsed::read(db, root.clone(), PROGRAM).unwrap_or_else(|| {
        missing_files.push((PROGRAM.into(), PathBuf::from(root.path())));
        Parsed::empty(PROGRAM, Default::default())
    });
    let lib_file = |m: &str| {
        db.get::<ModuleFile>(&(root.clone(), m.into()))
            .ok()
            .flatten()
    };
    // A module and, for the program's own `use` lines, where it is named.
    let mut queue: VecDeque<(String, Option<SourceId>)> = uses(&program.items)
        .map(|(m, _, at)| (m, Some(at)))
        .collect();
    let mut seen = HashSet::new();
    // A module and another its `use` lines name, and where.
    let mut edges: Vec<(String, String, Option<SourceId>)> = Vec::new();
    // An inline module is loaded with its file; what it names is loaded as the file's is.
    let inline =
        |p: &Parsed, queue: &mut VecDeque<_>, edges: &mut Vec<_>, seen: &mut HashSet<String>| {
            seen.extend(p.inline.iter().map(|(m, _, _)| m.clone()));
            for (m, _, items) in &p.inline {
                for (to, _, at) in uses(items) {
                    // `use a.f` names an item of the inline module `a`.
                    let module = match to.rsplit_once('.') {
                        Some((a, _)) if !seen.contains(&to) && seen.contains(a) => a.to_string(),
                        _ => to.clone(),
                    };
                    edges.push((m.clone(), module, Some(at.clone())));
                    queue.push_back((to, Some(at)));
                }
            }
        };
    inline(&program, &mut queue, &mut edges, &mut seen);
    let mut from_prelude: Vec<&String> = prelude.values().collect();
    from_prelude.sort();
    from_prelude.dedup();
    queue.extend(from_prelude.into_iter().map(|m| (m.clone(), None)));
    let is_std = |m: &str| m.split_once('.').is_some_and(|(lib, _)| lib == "std");
    let exists = |m: &str| match is_std(m) {
        true => std.index.contains_key(m),
        false => lib_file(m).is_some_and(|f| matches!(db.input::<Source>(&f), Ok(Some(_)))),
    };
    let mut got: Vec<Got> = Vec::new();
    let mut missing: Vec<(String, Option<SourceId>)> = Vec::new();
    // The core's traits name std types (`Ord`'s `cmp` returns `std.algebra`'s `Ordering`).
    queue.extend(uses(&std.modules[0].items).map(|(to, _, _)| (to, None)));
    while let Some((m, at)) = queue.pop_front() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let g = match is_std(&m) {
            true => std.index.get(&m).copied().map(Got::Std),
            false => lib_file(&m).and_then(|file| {
                let p = Parsed::read(db, file.clone(), &m);
                if p.is_none() {
                    missing_files.push((m.clone(), PathBuf::from(file.path())));
                }
                p.map(Got::Lib)
            }),
        };
        let Some(g) = g else {
            // `use std.rand.Rng` is an item of `std.rand`.
            match m.rsplit_once('.') {
                Some((module, _)) if exists(module) || seen.contains(module) => {
                    queue.push_back((module.to_string(), at))
                }
                _ => missing.push((m, at)),
            }
            continue;
        };
        let items = match &g {
            Got::Std(i) => &std.modules[*i].items,
            Got::Lib(p) => &p.items,
        };
        for (to, _, at) in uses(items) {
            edges.push((m.clone(), to.clone(), Some(at)));
            queue.push_back((to, None));
        }
        if let Got::Lib(p) = &g {
            inline(p, &mut queue, &mut edges, &mut seen);
        }
        got.push(g);
    }
    if let Some((m, at)) = cycle(&edges) {
        let text = Msg::verdict(cpl_diag::codes::UNRESOLVED_MODULE, format!("`{m}` is in an import cycle; two modules that need each other are one module (R11.2.2)"));
        errors.push(match at {
            Some(at) => Msg::at(at.clone(), text),
            None => text,
        });
    }
    let mut texts: Vec<Text> = std.modules.iter().map(|p| p.text.clone()).collect();
    let mut modules: Vec<Module> = std.modules.iter().map(Parsed::module).collect();
    let mut seen = vec![false; modules.len()];
    seen[0] = true;
    let mut poisoned = Vec::new();
    let mut unread = |p: &Parsed, errors: &mut Vec<Msg>| {
        errors.extend(p.unread.0.iter().cloned());
        poisoned.extend(p.unread.1.iter().cloned());
    };
    let mut libs = Vec::new();
    for g in got {
        match g {
            Got::Std(i) => {
                seen[i] = true;
                unread(&std.modules[i], &mut errors);
            }
            Got::Lib(p) => {
                unread(&p, &mut errors);
                libs.push(p);
            }
        }
    }
    unread(&program, &mut errors);
    libs.push(program);
    for p in libs {
        for (t, m) in p.modules() {
            texts.push(t);
            modules.push(m);
            seen.push(true);
        }
    }
    let code = cpl_diag::codes::UNRESOLVED_MODULE;
    for (m, a) in missing {
        let text = Msg::verdict(code, format!("no module `{m}`"));
        errors.push(match a {
            Some(a) => Msg::at(a, text),
            None => text,
        });
    }
    Loaded {
        modules,
        index: (texts.iter().enumerate())
            .map(|(i, t)| (t.name.clone(), i))
            .collect(),
        texts,
        seen,
        prelude,
        errors,
        poisoned,
        missing: missing_files,
    }
}

/// A module on a cycle of `use` edges, if there is one, and where it names the next.
fn cycle(edges: &[(String, String, Option<SourceId>)]) -> Option<(String, Option<&SourceId>)> {
    let mut out: HashMap<&str, Vec<(&str, Option<&SourceId>)>> = HashMap::new();
    for (a, b, at) in edges {
        out.entry(a).or_default().push((b, at.as_ref()));
    }
    // 0 unvisited, 1 on the path, 2 done.
    type Out<'a> = HashMap<&'a str, Vec<(&'a str, Option<&'a SourceId>)>>;
    fn visit<'a>(
        m: &'a str,
        out: &Out<'a>,
        state: &mut HashMap<&'a str, u8>,
        path: &mut Vec<(&'a str, Option<&'a SourceId>)>,
    ) -> Option<(String, Option<&'a SourceId>)> {
        match state.get(m) {
            Some(1) => {
                let i = path.iter().position(|(p, _)| *p == m)?;
                return Some((m.to_string(), path[i].1));
            }
            Some(2) => return None,
            _ => {}
        }
        state.insert(m, 1);
        for &(n, at) in out.get(m).into_iter().flatten() {
            path.push((m, at));
            if let Some(c) = visit(n, out, state, path) {
                return Some(c);
            }
            path.pop();
        }
        state.insert(m, 2);
        None
    }
    let mut state: HashMap<&str, u8> = HashMap::new();
    let mut starts: Vec<&str> = out.keys().copied().collect();
    starts.sort_unstable();
    starts
        .into_iter()
        .find_map(|m| visit(m, &out, &mut state, &mut Vec::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Head;

    fn parse_text(t: &str) -> FileItems {
        let (db, file) = (Db::default(), FileId::new(PROGRAM));
        db.set::<Source>(file.clone(), t.into()).unwrap();
        parse(&db, file, PROGRAM)
    }

    /// An item's path and the spans inside it are the same whatever is added above it; only
    /// where it starts moves.
    #[test]
    fn an_item_above_leaves_the_path_and_spans_of_those_below() {
        let below = "struct S\nimpl Show for S:\n    fn show(self) -> Str: \"s\"\n\nfn g() -> Int:\n    let x = 2\n    x\n";
        let above = format!("fn f() -> Int: 1\n\n{below}");
        let items = |t: &str| {
            let FileItems { items, starts, .. } = parse_text(t);
            (items.into_iter())
                .filter(|i| &*i.id.name != "f")
                .map(|i| {
                    let at = starts[&i.id];
                    (i, at)
                })
                .collect::<Vec<_>>()
        };
        let (a, b) = (items(below), items(&above));
        assert_eq!(a.len(), 3);
        assert_eq!(&*a[1].0.id.name, "impl Show for S");
        for ((x, xs), (y, ys)) in a.iter().zip(&b) {
            assert_eq!(x.id, y.id);
            assert_eq!(x.item.syntax().green(), y.item.syntax().green());
            assert_eq!(xs + 18, *ys);
        }
    }

    /// A module's items are the same answer after an edit inside a body or a line added above.
    #[test]
    fn a_body_edit_or_a_line_above_leaves_the_items_equal() {
        let heads = |t: &str| {
            let m = Module {
                name: PROGRAM.into(),
                items: Arc::new(parse_text(t).items),
                within: None,
            };
            crate::items::items(&m)
        };
        let src = "struct S\npub fn answer() -> Int:\n    let x = 2\n    x\nprint(answer())\n";
        let a = heads(src);
        assert_eq!(heads(&src.replace("2", "3")), a);
        assert_eq!(heads(&format!("\n\n{src}")), a);
        let exports: Vec<Vec<&str>> = a.iter().map(|h| h.exports().collect()).collect();
        assert_eq!(exports, [vec![], vec!["answer"], vec![]]);
        let two = heads("pub fn f() -> Int = 1\nfn g() -> Int = 2\n");
        let all: Vec<&str> = two.iter().flat_map(Head::exports).collect();
        assert_eq!(all, ["f"]);
    }
}
