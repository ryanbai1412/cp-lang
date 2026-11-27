//! Contract 1: snapshot -> modules. Finds the files a program needs and joins them into one
//! source, so every later layer reports a place as an offset into it.
//!
//! A module is loaded only when a name from it can be meant: one the program's `use` lines
//! name, a prelude name the program's text mentions (R11.3), or one a loaded module's own
//! `use` lines name. Libraries get no prelude (R11.3.1).
use crate::diag::Msg;
use cpl_diag::Span;
use cpl_syntax::ast::{Stmt, StmtKind};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

type Items = Vec<(u32, Vec<Stmt>)>;
/// Each syntax error with its code and where it is, and the spans the parser could not read.
type Unread = (Vec<Msg>, Vec<Span>);

pub struct Module {
    pub name: String,
    /// Where its text starts in the joined source.
    pub start: u32,
    pub items: Vec<(u32, Vec<Stmt>)>,
}

pub struct Loaded {
    pub src: String,
    /// The core first and the program last.
    pub modules: Vec<Module>,
    /// Where the program's text starts in `src`.
    pub program_start: u32,
    /// Each prelude name and the module it is from.
    pub prelude: HashMap<String, String>,
    pub errors: Vec<Msg>,
    /// What the parser could not read: no later layer reports inside these (contract 3).
    pub poisoned: Vec<Span>,
}

pub const CORE: &str = "core";
pub const PROGRAM: &str = "program";

/// The std modules not yet loaded that declare a method `names` calls and found nowhere: a
/// method call needs no import (R11.3.4), so a check that finds none asks for these.
pub fn declaring(names: &[String], loaded: &[Module]) -> Vec<String> {
    let index = std_methods();
    let mut out: Vec<String> = (names.iter())
        .flat_map(|n| index.get(n).into_iter().flatten())
        .filter(|m| !loaded.iter().any(|l| &l.name == *m))
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every free method std declares, by name: `pub fn sorted<S: Seq<T>, ..>(self: S, ..)` in
/// `std.seq` is `sorted`. Std is pinned, so this is read once.
fn std_methods() -> &'static HashMap<String, Vec<String>> {
    static INDEX: std::sync::OnceLock<HashMap<String, Vec<String>>> = std::sync::OnceLock::new();
    INDEX.get_or_init(|| {
        let mut index: HashMap<String, Vec<String>> = HashMap::new();
        let mut dirs = vec![(std_root(), "std".to_string())];
        while let Some((dir, module)) = dirs.pop() {
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                let stem = path
                    .file_stem()
                    .map_or(String::new(), |s| s.to_string_lossy().into_owned());
                if path.is_dir() {
                    dirs.push((path, format!("{module}.{stem}")));
                    continue;
                }
                if path.extension().is_none_or(|e| e != "cp") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                // A top-level fn only: one indented under a type is that type's, found with it.
                let top = text.lines().filter_map(|l| {
                    let l = if l.starts_with('@') {
                        l.split_once(" pub fn ")?.1
                    } else {
                        l.strip_prefix("pub fn ")?
                    };
                    Some(l)
                });
                for decl in top {
                    let name_len = decl.len()
                        - decl
                            .trim_start_matches(|c: char| c.is_alphanumeric() || c == '_')
                            .len();
                    let Some(open) = decl.find('(') else { continue };
                    let first = decl[open + 1..].trim_start();
                    if name_len > 0 && (first.starts_with("self") || first.starts_with("mut self"))
                    {
                        index
                            .entry(decl[..name_len].to_string())
                            .or_default()
                            .push(format!("{module}.{stem}"));
                    }
                }
            }
        }
        index
    })
}

/// Where each library's modules are: `std`, the pinned stdlib, then the roots the nearest
/// `cp.toml` at or above the program names under `[libs]` (R11.2.1).
pub struct Roots(Vec<(String, PathBuf)>);

fn std_root() -> PathBuf {
    PathBuf::from(format!("{}/../../std", env!("CARGO_MANIFEST_DIR")))
}

impl Roots {
    /// The pinned stdlib alone.
    pub fn std() -> Roots {
        Roots(vec![("std".to_string(), std_root())])
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
                    if !roots.iter().any(|(n, _)| n == name) {
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

/// `wanted`: std modules a check of the program found it needs (see `declaring`).
pub fn load(core: &str, user: &str, roots: &Roots, wanted: &[String]) -> Loaded {
    let mut errors = Vec::new();
    let (program, (program_diags, program_poisoned)) = parse(user);
    let mut prelude = HashMap::new();
    if let Some(text) = roots.source("std.prelude") {
        let (items, _) = parse(&text);
        for (module, s, _) in uses(&items) {
            if let StmtKind::Use { names, .. } = &s.kind {
                for n in names {
                    prelude.insert(n.text.clone(), module.clone());
                }
            }
        }
    }
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
    let mut libs: Vec<(String, String, Items)> = Vec::new();
    let mut seen = HashSet::new();
    let mut edges: Vec<(String, String)> = Vec::new();
    let mut missing: Vec<(String, Option<u32>)> = Vec::new();
    let mut lib_diags: Vec<(usize, Unread)> = Vec::new();
    // The core's traits name std types (`Ord`'s `cmp` returns `std.algebra`'s `Ordering`).
    let (core_items, _) = parse(core);
    queue.extend(uses(&core_items).map(|(to, _, _)| (to, None)));
    while let Some((m, at)) = queue.pop_front() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let Some(text) = roots.source(&m) else {
            // `use std.rand.Rng` is an item of `std.rand`.
            match m.rsplit_once('.') {
                Some((module, _)) if roots.source(module).is_some() => {
                    queue.push_back((module.to_string(), at))
                }
                _ => missing.push((m, at)),
            }
            continue;
        };
        let (items, unread) = parse(&text);
        lib_diags.push((libs.len(), unread));
        for (to, _, _) in uses(&items) {
            edges.push((m.clone(), to.clone()));
            queue.push_back((to, None));
        }
        libs.push((m, text, items));
    }
    if let Some(m) = cycle(&edges) {
        errors.push(Msg::from(format!(
            "{}: `{m}` is in an import cycle; two modules that need each other are one module (R11.2.2)",
            cpl_diag::codes::UNRESOLVED_MODULE.0
        )));
    }
    let mut src = String::new();
    let mut modules = Vec::new();
    let mut add = |name: &str, text: &str, items: Vec<(u32, Vec<Stmt>)>| {
        let base = src.len() as u32;
        src.push_str(text);
        src.push('\n');
        let items = items.into_iter().map(|(o, ss)| (o + base, ss)).collect();
        modules.push(Module {
            name: name.to_string(),
            start: base,
            items,
        });
        base
    };
    add(CORE, core, core_items);
    let mut lib_starts = Vec::new();
    for (name, text, items) in libs {
        lib_starts.push(add(&name, &text, items));
    }
    let program_start = add(PROGRAM, user, program);
    let at = |base: u32, s: Span| Span::new(base + s.start, base + s.end);
    let placed = |base: u32| {
        move |m: Msg| Msg {
            at: m.at.map(|a| at(base, a)),
            ..m
        }
    };
    let mut poisoned = Vec::new();
    for (lib, (diags, p)) in lib_diags {
        errors.extend(diags.into_iter().map(placed(lib_starts[lib])));
        poisoned.extend(p.into_iter().map(|s| at(lib_starts[lib], s)));
    }
    errors.extend(program_diags.into_iter().map(placed(program_start)));
    poisoned.extend(program_poisoned.into_iter().map(|s| at(program_start, s)));
    let code = cpl_diag::codes::UNRESOLVED_MODULE.0;
    for (m, a) in missing {
        let text = format!("{code}: no module `{m}`");
        errors.push(match a {
            Some(a) => Msg::at(at(program_start, Span::new(a, a)), text),
            None => Msg::from(text),
        });
    }
    Loaded {
        src,
        modules,
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
