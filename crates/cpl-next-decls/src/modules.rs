//! Contract 1: snapshot -> modules. Finds the modules a program needs. Each keeps its own text,
//! and each of its items has a path (`ItemId`), so a place is an item and a span inside it.
//!
//! A module is loaded when a name from it can be meant: one the program's `use` lines name,
//! one the prelude names (R11.3), or one a loaded module's own `use` lines name. A method
//! resolves on its receiver's type, wherever the program spells it (R11.3.4). Libraries get
//! no prelude (R11.3.1).
use crate::diag::Msg;
use cpl_diag::Span;
use cpl_next_ir::ids::{ItemId, SourceId};
use cpl_syntax::ast::{Stmt, StmtKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// An item as the parser split it: its path and its statements, whose spans count from the
/// item's start.
pub struct Item {
    pub id: ItemId,
    pub stmts: Vec<Stmt>,
}

type Items = Vec<Item>;
/// Each syntax error with its code and where it is, and the places the parser could not read.
type Unread = (Vec<Msg>, Vec<SourceId>);

pub struct Module {
    pub name: String,
    pub items: Arc<Items>,
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
}

/// A module read and parsed.
struct Parsed {
    text: Text,
    items: Arc<Items>,
    unread: Unread,
}

impl Parsed {
    fn module(&self) -> Module {
        Module {
            name: self.text.name.clone(),
            items: self.items.clone(),
        }
    }
    fn new(name: &str, text: Arc<str>) -> Parsed {
        let (items, starts, unread) = parse(name, &text);
        Parsed {
            text: Text::new(name, text, starts),
            items: Arc::new(items),
            unread,
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
}

impl Std {
    pub fn read() -> Std {
        const CORE_TEXT: &str = include_str!("../../cpl-next/std.cp");
        let mut files = Vec::new();
        let mut dirs = vec![(std_root(), "std".to_string())];
        while let Some((dir, module)) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                let stem = path
                    .file_stem()
                    .map_or(String::new(), |s| s.to_string_lossy().into_owned());
                if path.is_dir() {
                    dirs.push((path, format!("{module}.{stem}")));
                } else if path.extension().is_some_and(|e| e == "cp") {
                    if let Ok(text) = std::fs::read_to_string(&path) {
                        files.push((format!("{module}.{stem}"), text));
                    }
                }
            }
        }
        files.sort();
        let placed = std::iter::once((CORE.to_string(), CORE_TEXT.to_string())).chain(files);
        // Each module is read alone, so they are parsed side by side.
        let modules: Vec<Parsed> = std::thread::scope(|scope| {
            let parsing: Vec<_> = placed
                .map(|(name, text)| scope.spawn(move || Parsed::new(&name, text.into())))
                .collect();
            (parsing.into_iter())
                .map(|p| p.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
                .collect()
        });
        let index = (modules.iter().enumerate())
            .map(|(i, p)| (p.text.name.clone(), i))
            .collect::<HashMap<_, _>>();
        let mut prelude = HashMap::new();
        if let Some(p) = index.get("std.prelude").map(|i| &modules[*i]) {
            for (module, s, _) in uses(&p.items) {
                if let StmtKind::Use { names, .. } = &s.kind {
                    for n in names {
                        prelude.insert(n.text.clone(), module.clone());
                    }
                }
            }
        }
        let decls = crate::decls::prepare(&modules.iter().map(Parsed::module).collect::<Vec<_>>());
        Std {
            modules,
            index,
            decls,
            prelude,
        }
    }
    pub fn decls(&self) -> &crate::decls::Prepared {
        &self.decls
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

fn std_root() -> PathBuf {
    PathBuf::from(format!("{}/../../std", env!("CARGO_MANIFEST_DIR")))
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
    /// The text of module `name`: `std.graph` is `graph.cp` in std's root, `kit.m` is `m.cp`
    /// in the root `[libs]` calls `kit`.
    fn source(&self, name: &str) -> Option<String> {
        let (lib, rest) = name.split_once('.')?;
        let (_, root) = self.0.iter().find(|(n, _)| n == lib)?;
        std::fs::read_to_string(root.join(format!("{}.cp", rest.replace('.', "/")))).ok()
    }
}

/// Each `use` line's module, the line, and where it is written.
fn uses(items: &[Item]) -> impl Iterator<Item = (String, &Stmt, SourceId)> {
    items
        .iter()
        .flat_map(|it| it.stmts.iter().map(move |s| (&it.id, s)))
        .filter_map(|(id, s)| match &s.kind {
            StmtKind::Use { path, .. } => Some((
                path.iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                s,
                SourceId {
                    item: id.clone(),
                    span: s.span,
                },
            )),
            _ => None,
        })
}

/// The items of module `module`, where each starts in `text`, each syntax error with its code
/// and where it is, and the places the parser could not read.
fn parse(module: &str, text: &str) -> (Items, HashMap<ItemId, u32>, Unread) {
    let p = cpl_syntax::parse_file(text);
    let module: Arc<str> = module.into();
    let mut nth: HashMap<String, u32> = HashMap::new();
    let (mut items, mut starts, mut poisoned) = (Vec::new(), Vec::new(), Vec::new());
    for (o, it) in p.items {
        let name = path_name(text, o, &it.stmts);
        let n = nth.entry(name.clone()).or_default();
        let id = ItemId {
            module: module.clone(),
            name: name.into(),
            nth: *n,
        };
        *n += 1;
        poisoned.extend((it.poisoned.iter()).map(|s| SourceId {
            item: id.clone(),
            span: *s,
        }));
        starts.push((id.clone(), o));
        items.push(Item {
            id,
            stmts: it.stmts,
        });
    }
    let file = ItemId::file(module);
    // A file's place, as the item it falls in.
    let within = |span: Span| match starts.iter().rev().find(|(_, o)| *o <= span.start) {
        Some((id, o)) => SourceId {
            item: id.clone(),
            span: Span::new(span.start - o, span.end - o),
        },
        None => SourceId {
            item: file.clone(),
            span,
        },
    };
    let diags = (p.diagnostics.into_iter())
        .map(|d| Msg {
            help: d.help,
            ..Msg::at(within(d.span), Msg::verdict(d.code, d.message))
        })
        .collect();
    (items, starts.into_iter().collect(), (diags, poisoned))
}

/// The name in an item's path: what it declares, an impl's head as written, or nothing for a
/// top-level statement.
fn path_name(text: &str, start: u32, stmts: &[Stmt]) -> String {
    let head = |s: &Stmt| {
        let line = text.get((start + s.span.start) as usize..).unwrap_or("");
        let line = line.lines().next().unwrap_or("").trim_end();
        line.strip_suffix(':').unwrap_or(line).to_string()
    };
    (stmts.iter())
        .find_map(|s| match &s.kind {
            StmtKind::Impl { .. } => Some(head(s)),
            _ => crate::decls::item_name(s).map(str::to_string),
        })
        .unwrap_or_default()
}

pub fn load(std: &Std, user: &str, roots: &Roots) -> Loaded {
    enum Got {
        Std(usize),
        Lib(Parsed),
    }
    let mut errors = Vec::new();
    let prelude = std.prelude.clone();
    let program = Parsed::new(PROGRAM, user.into());
    // A module and, for the program's own `use` lines, where it is named.
    let mut queue: VecDeque<(String, Option<SourceId>)> = uses(&program.items)
        .map(|(m, _, at)| (m, Some(at)))
        .collect();
    let mut from_prelude: Vec<&String> = prelude.values().collect();
    from_prelude.sort();
    from_prelude.dedup();
    queue.extend(from_prelude.into_iter().map(|m| (m.clone(), None)));
    let is_std = |m: &str| m.split_once('.').is_some_and(|(lib, _)| lib == "std");
    let exists = |m: &str| match is_std(m) {
        true => std.index.contains_key(m),
        false => roots.source(m).is_some(),
    };
    let mut got: Vec<Got> = Vec::new();
    let mut seen = HashSet::new();
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut missing: Vec<(String, Option<SourceId>)> = Vec::new();
    // The core's traits name std types (`Ord`'s `cmp` returns `std.algebra`'s `Ordering`).
    queue.extend(uses(&std.modules[0].items).map(|(to, _, _)| (to, None)));
    while let Some((m, at)) = queue.pop_front() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let g = match is_std(&m) {
            true => std.index.get(&m).copied().map(Got::Std),
            false => (roots.source(&m)).map(|text| Got::Lib(Parsed::new(&m, text.into()))),
        };
        let Some(g) = g else {
            // `use std.rand.Rng` is an item of `std.rand`.
            match m.rsplit_once('.') {
                Some((module, _)) if exists(module) => queue.push_back((module.to_string(), at)),
                _ => missing.push((m, at)),
            }
            continue;
        };
        let items = match &g {
            Got::Std(i) => &std.modules[*i].items,
            Got::Lib(p) => &p.items,
        };
        for (to, _, _) in uses(items) {
            edges.push((m.clone(), to.clone()));
            queue.push_back((to, None));
        }
        got.push(g);
    }
    if let Some(m) = cycle(&edges) {
        errors.push(Msg::verdict(cpl_diag::codes::UNRESOLVED_MODULE, format!("`{m}` is in an import cycle; two modules that need each other are one module (R11.2.2)")));
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
        texts.push(p.text.clone());
        modules.push(p.module());
        seen.push(true);
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
    }
}

/// A module on a cycle of `use` edges, if there is one.
fn cycle(edges: &[(String, String)]) -> Option<String> {
    let mut out: HashMap<&str, Vec<&str>> = HashMap::new();
    for (a, b) in edges {
        out.entry(a).or_default().push(b);
    }
    // 0 unvisited, 1 on the path, 2 done.
    let mut state: HashMap<&str, u8> = HashMap::new();
    fn visit<'a>(
        m: &'a str,
        out: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, u8>,
    ) -> Option<String> {
        match state.get(m) {
            Some(1) => return Some(m.to_string()),
            Some(2) => return None,
            _ => {}
        }
        state.insert(m, 1);
        for &n in out.get(m).into_iter().flatten() {
            if let Some(c) = visit(n, out, state) {
                return Some(c);
            }
        }
        state.insert(m, 2);
        None
    }
    let mut starts: Vec<&str> = out.keys().copied().collect();
    starts.sort_unstable();
    starts.into_iter().find_map(|m| visit(m, &out, &mut state))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An item's path and the spans inside it are the same whatever is added above it; only
    /// where it starts moves.
    #[test]
    fn an_item_above_leaves_the_path_and_spans_of_those_below() {
        let below = "struct S\nimpl Show for S:\n    fn show(self) -> Str: \"s\"\n\nfn g() -> Int:\n    let x = 2\n    x\n";
        let above = format!("fn f() -> Int: 1\n\n{below}");
        let items = |t: &str| {
            let (items, starts, _) = parse(PROGRAM, t);
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
            assert_eq!(x.stmts, y.stmts);
            assert_eq!(xs + 18, *ys);
        }
    }
}
