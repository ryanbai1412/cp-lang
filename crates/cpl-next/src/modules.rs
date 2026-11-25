//! Contract 1: snapshot -> modules. Finds the files a program needs and joins them into one
//! source, so every later layer reports a place as an offset into it.
//!
//! A module is loaded only when a name from it can be meant: one the program's `use` lines
//! name, a prelude name the program's text mentions (R11.3), or one a loaded module's own
//! `use` lines name. Libraries get no prelude (R11.3.1).
use cpl_syntax::ast::{Stmt, StmtKind};
use std::collections::{HashMap, HashSet, VecDeque};

type Items = Vec<(u32, Vec<Stmt>)>;

pub struct Module {
    pub name: String,
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
    pub errors: Vec<String>,
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
        let root = format!("{}/../../std", env!("CARGO_MANIFEST_DIR"));
        let mut index: HashMap<String, Vec<String>> = HashMap::new();
        let mut dirs = vec![(std::path::PathBuf::from(root), "std".to_string())];
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

/// The pinned standard library's text of module `name` (`std.graph` is `std/graph.cp`).
fn std_source(name: &str) -> Option<String> {
    let rest = name.strip_prefix("std.")?;
    let path = format!(
        "{}/../../std/{}.cp",
        env!("CARGO_MANIFEST_DIR"),
        rest.replace('.', "/")
    );
    std::fs::read_to_string(path).ok()
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

/// The items, and each syntax error with its code and where it starts in `text`.
fn parse(text: &str) -> (Items, Vec<(u32, String)>) {
    let p = cpl_syntax::parse_file(text);
    let items = p.items.into_iter().map(|(o, it)| (o, it.stmts)).collect();
    let diags = (p.diagnostics.into_iter())
        .map(|d| (d.span.start, format!("{}: {}", d.code.0, d.message)))
        .collect();
    (items, diags)
}

/// `wanted`: std modules a check of the program found it needs (see `declaring`).
pub fn load(core: &str, user: &str, wanted: &[String]) -> Loaded {
    let mut errors = Vec::new();
    let (program, program_diags) = parse(user);
    let mut prelude = HashMap::new();
    if let Some(text) = std_source("std.prelude") {
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
    // The core's traits name std types (`Ord`'s `cmp` returns `std.algebra`'s `Ordering`).
    let (core_items, _) = parse(core);
    queue.extend(uses(&core_items).map(|(to, _, _)| (to, None)));
    while let Some((m, at)) = queue.pop_front() {
        if !seen.insert(m.clone()) {
            continue;
        }
        let Some(text) = std_source(&m) else {
            // `use std.rand.Rng` is an item of `std.rand`.
            match m.rsplit_once('.') {
                Some((module, _)) if std_source(module).is_some() => {
                    queue.push_back((module.to_string(), at))
                }
                _ => missing.push((m, at)),
            }
            continue;
        };
        let (items, diags) = parse(&text);
        errors.extend(diags.into_iter().map(|(_, d)| format!("{m}: {d}")));
        for (to, _, _) in uses(&items) {
            edges.push((m.clone(), to.clone()));
            queue.push_back((to, None));
        }
        libs.push((m, text, items));
    }
    if let Some(m) = cycle(&edges) {
        errors.push(format!(
            "{}: `{m}` is in an import cycle; two modules that need each other are one module (R11.2.2)",
            cpl_diag::codes::UNRESOLVED_MODULE.0
        ));
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
            items,
        });
        base
    };
    add(CORE, core, core_items);
    for (name, text, items) in libs {
        add(&name, &text, items);
    }
    let program_start = add(PROGRAM, user, program);
    let line = |at: u32| src[..(program_start + at) as usize].matches('\n').count() + 1;
    errors.extend((program_diags.into_iter()).map(|(at, d)| format!("line {}: {d}", line(at))));
    let code = cpl_diag::codes::UNRESOLVED_MODULE.0;
    for (m, at) in missing {
        errors.push(match at {
            Some(at) => format!("line {}: {code}: no module `{m}`", line(at)),
            None => format!("{code}: no module `{m}`"),
        });
    }
    Loaded {
        src,
        modules,
        program_start,
        prelude,
        errors,
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
