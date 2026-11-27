//! Contract 1: snapshot -> modules. Finds the modules a program needs. Each keeps its own text,
//! placed in one offset space so a span says both the module and the place in it.
//!
//! A module is loaded only when a name from it can be meant: one the program's `use` lines
//! name, a prelude name the program's text mentions (R11.3), or one a loaded module's own
//! `use` lines name. Libraries get no prelude (R11.3.1).
use crate::diag::Msg;
use cpl_diag::Span;
use cpl_syntax::ast::{Stmt, StmtKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Items = Vec<(u32, Vec<Stmt>)>;
/// Each syntax error with its code and where it is, and the spans the parser could not read.
type Unread = (Vec<Msg>, Vec<Span>);

pub struct Module {
    pub name: String,
    pub items: Arc<Items>,
}

/// A module's text at its place in the offset space.
#[derive(Clone)]
pub struct Text {
    pub name: String,
    pub start: u32,
    pub text: Arc<str>,
    /// Where each of its lines starts in `text`.
    lines: Arc<[u32]>,
}

impl Text {
    fn new(name: &str, start: u32, text: Arc<str>) -> Text {
        let lines = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i as u32 + 1))
            .collect();
        Text {
            name: name.to_string(),
            start,
            text,
            lines,
        }
    }
    /// The places it holds: its text and the end just past it.
    fn holds(&self, pos: u32) -> bool {
        self.start <= pos && pos <= self.start + self.text.len() as u32
    }
    /// The 1-based line and column of `pos`.
    pub fn line_col(&self, pos: u32) -> (u32, u32) {
        let local = pos - self.start;
        let line = self.lines.partition_point(|s| *s <= local);
        (line as u32, local - self.lines[line - 1] + 1)
    }
}

/// A module read and parsed, its items and syntax errors placed at its start.
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
    fn new(name: &str, start: u32, text: Arc<str>) -> Parsed {
        let (items, unread) = parse(&text);
        Parsed::placed(name, start, text, items, unread)
    }
    /// `items` and `unread`, read from `text` alone, placed at `start`.
    fn placed(name: &str, start: u32, text: Arc<str>, items: Items, unread: Unread) -> Parsed {
        let at = |s: Span| Span::new(start + s.start, start + s.end);
        let (diags, poisoned) = unread;
        let diags = (diags.into_iter())
            .map(|m| Msg {
                at: m.at.map(at),
                ..m
            })
            .collect();
        Parsed {
            text: Text::new(name, start, text),
            items: Arc::new(items.into_iter().map(|(o, ss)| (o + start, ss)).collect()),
            unread: (diags, poisoned.into_iter().map(at).collect()),
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
    end: u32,
}

impl Std {
    pub fn read() -> Std {
        const CORE_TEXT: &str = include_str!("../std.cp");
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
        let mut placed = vec![(CORE.to_string(), 0, CORE_TEXT.to_string())];
        let mut end = CORE_TEXT.len() as u32 + 1;
        for (name, text) in files {
            let len = text.len() as u32;
            placed.push((name, end, text));
            end += len + 1;
        }
        // Each module is read alone, so they are parsed side by side.
        let modules: Vec<Parsed> = std::thread::scope(|scope| {
            let parsing: Vec<_> = (placed.into_iter())
                .map(|(name, start, text)| {
                    scope.spawn(move || Parsed::new(&name, start, text.into()))
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
            end,
        }
    }
    pub(crate) fn decls(&self) -> &crate::decls::Prepared {
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
    /// Where the program's text starts: it is placed after every other module.
    pub program_start: u32,
    /// Each prelude name and the module it is from.
    pub prelude: HashMap<String, String>,
    pub errors: Vec<Msg>,
    /// What the parser could not read: no later layer reports inside these (contract 3).
    pub poisoned: Vec<Span>,
}

impl Loaded {
    /// The module `pos` is in.
    pub fn text_at(&self, pos: u32) -> &Text {
        (self.texts.iter().find(|t| t.holds(pos))).unwrap_or(&self.texts[0])
    }
    /// The source text `span` covers.
    pub fn text(&self, span: Span) -> &str {
        let t = self.text_at(span.start);
        let at = (span.start - t.start) as usize..(span.end.max(t.start) - t.start) as usize;
        t.text.get(at).unwrap_or("")
    }
    /// The program's text.
    pub fn program(&self) -> &str {
        &self.text_at(self.program_start).text
    }
    /// The program's 1-based line `pos` is on; 0 outside the program.
    pub fn program_line(&self, pos: u32) -> u32 {
        let t = self.text_at(pos);
        match t.start == self.program_start {
            true => t.line_col(pos).0,
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
fn uses(items: &[(u32, Vec<Stmt>)]) -> impl Iterator<Item = (String, &Stmt, u32)> {
    items
        .iter()
        .flat_map(|(o, ss)| ss.iter().map(move |s| (*o, s)))
        .filter_map(|(o, s)| match &s.kind {
            StmtKind::Use { path, .. } => Some((
                path.iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
                s,
                o + s.span.start,
            )),
            _ => None,
        })
}

/// The items, each syntax error with its code and where it is in `text`, and the spans of
/// `text` the parser could not read.
fn parse(text: &str) -> (Items, Unread) {
    let p = cpl_syntax::parse_file(text);
    let mut poisoned = Vec::new();
    let mut items = Vec::new();
    for (o, it) in p.items {
        poisoned.extend((it.poisoned.iter()).map(|s| Span::new(o + s.start, o + s.end)));
        items.push((o, it.stmts));
    }
    let diags = (p.diagnostics.into_iter())
        .map(|d| Msg {
            help: d.help,
            ..Msg::at(d.span, format!("{}: {}", d.code.0, d.message))
        })
        .collect();
    (items, (diags, poisoned))
}

/// `wanted`: std modules a check of the program found it needs (see `Std::declaring`).
pub fn load(std: &Std, user: &str, roots: &Roots, wanted: &[String]) -> Loaded {
    enum Got {
        Std(usize),
        Lib(Parsed),
    }
    let mut errors = Vec::new();
    let prelude = std.prelude.clone();
    let (program, program_unread) = parse(user);
    // A module and, for the program's own `use` lines, where it is named.
    let mut queue: VecDeque<(String, Option<u32>)> =
        uses(&program).map(|(m, _, at)| (m, Some(at))).collect();
    // Only a prelude name the text spells can be referred to, so the other modules need not load.
    let mentioned: HashSet<&str> = user
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .collect();
    let mut from_prelude: Vec<&String> = (prelude.iter())
        .filter(|(n, _)| mentioned.contains(n.as_str()))
        .map(|(_, m)| m)
        .collect();
    from_prelude.sort();
    queue.extend(from_prelude.into_iter().map(|m| (m.clone(), None)));
    queue.extend(wanted.iter().map(|m| (m.clone(), None)));
    // A library's modules and the program are placed after std, the program last.
    let mut next = std.end;
    let mut place = |name: &str, text: Arc<str>, items: Items, unread: Unread| {
        let start = next;
        next += text.len() as u32 + 1;
        Parsed::placed(name, start, text, items, unread)
    };
    let is_std = |m: &str| m.split_once('.').is_some_and(|(lib, _)| lib == "std");
    let exists = |m: &str| match is_std(m) {
        true => std.index.contains_key(m),
        false => roots.source(m).is_some(),
    };
    let mut got: Vec<Got> = Vec::new();
    let mut seen = HashSet::new();
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut missing: Vec<(String, Option<u32>)> = Vec::new();
    // The core's traits name std types (`Ord`'s `cmp` returns `std.algebra`'s `Ordering`).
    queue.extend(uses(&std.modules[0].items).map(|(to, _, _)| (to, None)));
    while let Some((m, at)) = queue.pop_front() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let g = match is_std(&m) {
            true => std.index.get(&m).copied().map(Got::Std),
            false => (roots.source(&m)).map(|text| {
                let (items, unread) = parse(&text);
                Got::Lib(place(&m, text.into(), items, unread))
            }),
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
        errors.push(Msg::from(format!(
            "{}: `{m}` is in an import cycle; two modules that need each other are one module (R11.2.2)",
            cpl_diag::codes::UNRESOLVED_MODULE.0
        )));
    }
    let mut texts: Vec<Text> = std.modules.iter().map(|p| p.text.clone()).collect();
    let mut modules: Vec<Module> = std.modules.iter().map(Parsed::module).collect();
    let mut seen = vec![false; modules.len()];
    seen[0] = true;
    let mut poisoned = Vec::new();
    let mut unread = |p: &Parsed, errors: &mut Vec<Msg>| {
        errors.extend(p.unread.0.iter().cloned());
        poisoned.extend(p.unread.1.iter().copied());
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
    let program = place(PROGRAM, user.into(), program, program_unread);
    let program_start = program.text.start;
    unread(&program, &mut errors);
    libs.push(program);
    for p in libs {
        texts.push(p.text.clone());
        modules.push(p.module());
        seen.push(true);
    }
    let code = cpl_diag::codes::UNRESOLVED_MODULE.0;
    for (m, a) in missing {
        let text = format!("{code}: no module `{m}`");
        errors.push(match a {
            Some(a) => Msg::at(Span::new(program_start + a, program_start + a), text),
            None => Msg::from(text),
        });
    }
    Loaded {
        texts,
        modules,
        seen,
        program_start,
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
