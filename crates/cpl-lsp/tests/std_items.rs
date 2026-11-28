//! Every public std item (`cpl_doc::extract` over `std/*.cp`) and every
//! built-in member row (`cpl_facade::today::BUILTIN_MEMBERS`), used in a small
//! program through `cpl_lsp::Server`: hover and signature help must show the
//! signature with every declared parameter name (`by:`, `cmp:`), hover the
//! documented cost, and completion must offer the item with a detail naming
//! the same parameters. The program is the item's first doc `example:` that
//! uses it. Each check's failure count has a ceiling that only goes down, and
//! a check with no ceiling allows none; failures are printed as the backlog.
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r11-ask-the-analysis`
//! and `handbook/guidelines/commands-checklist.md#r14-measured-over-the-corpus`.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::collections::BTreeMap;
use std::path::Path;

#[path = "../../cpl-cli/tests/support/shards.rs"]
mod shards;

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn position(src: &str, offset: usize) -> Value {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = src[..offset].matches('\n').count() as u32;
    let col = src[line_start..offset].encode_utf16().count() as u32;
    obj([("line", line.into()), ("character", col.into())])
}

/// The values every built-in row's `example` is written against, as in
/// `crates/cpl-cli/tests/builtin_catalog.rs`.
const FIXTURE: &str = "var xs = [3, 1, 2]\nvar s = \"hello\"\nvar m = Map<Int, Int>()\nm[1] = 10\nm[5] = 50\n\
                       var st = Set<Int>()\nst.insert(4)\nst.insert(9)\nvar a: [0..3]Int = 0\na[1] = 7\n\
                       var hm = HashMap<Int, Int>()\nhm[1] = 10\nhm[5] = 50\nvar hs = HashSet<Int>()\nhs.insert(4)\nhs.insert(9)\n";

const STD_URI: &str = "file:///std_items/a.cp";
const BUILTIN_URI: &str = "file:///std_items/builtin.cp";

struct Lsp {
    server: Server,
    id: u32,
}

impl Lsp {
    fn new() -> Lsp {
        let mut server = Server::new();
        server.handle(&message(Some(1), "initialize", obj([])));
        server.handle(&message(None, "initialized", obj([])));
        Lsp { server, id: 1 }
    }

    fn open(&mut self, uri: &str, text: &str) {
        self.server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", text.into()),
                ]),
            )]),
        ));
    }

    fn close(&mut self, uri: &str) {
        self.server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.into())]))]),
        ));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.id += 1;
        let reply = self.server.handle(&message(Some(self.id), method, params));
        reply
            .into_iter()
            .find(|m| !m.at(&["id"]).is_null())
            .map_or(Value::Null, |m| m.at(&["result"]).clone())
    }

    fn ask(&mut self, method: &str, uri: &str, text: &str, offset: usize) -> Value {
        self.id += 1;
        let reply = self.server.handle(&message(
            Some(self.id),
            method,
            obj([
                ("textDocument", obj([("uri", uri.into())])),
                ("position", position(text, offset)),
            ]),
        ));
        reply
            .into_iter()
            .find(|m| !m.at(&["id"]).is_null())
            .map_or(Value::Null, |m| m.at(&["result"]).clone())
    }
}

/// The parameter names of a signature as written, `self` left out:
/// `pub fn sort<S, K>(mut self: S, by: T -> K = x => x; desc: Bool)` gives
/// `by`, `desc`.
fn param_names(signature: &str, name: &str) -> Vec<String> {
    params(signature, name)
        .into_iter()
        .map(|(l, _, _)| l)
        .collect()
}

/// Each parameter's label, the name a call passes it by, with whether it is
/// a named option, written after `;`: `while ok: (T) -> Bool` is `while`.
fn params_of(signature: &str, name: &str) -> Vec<(String, bool)> {
    params(signature, name)
        .into_iter()
        .map(|(l, _, named)| (l, named))
        .collect()
}

/// Each parameter's label, name and whether it is a named option.
fn params(signature: &str, name: &str) -> Vec<(String, String, bool)> {
    let Some(at) = signature.find(name) else {
        return Vec::new();
    };
    let rest = &signature[at + name.len()..];
    let mut depth = 0i32;
    let mut open = None;
    for (i, c) in rest.char_indices() {
        match c {
            '<' => depth += 1,
            '>' if !rest[..i].ends_with('-') && !rest[..i].ends_with('=') => depth -= 1,
            '(' if depth == 0 => {
                open = Some(i);
                break;
            }
            _ => {}
        }
    }
    let Some(open) = open else {
        return Vec::new();
    };
    let mut names = Vec::new();
    let (mut depth, mut start, mut named) = (0i32, open + 1, false);
    for (i, c) in rest[open..]
        .char_indices()
        .skip(1)
        .map(|(i, c)| (i + open, c))
    {
        match c {
            '(' | '[' | '{' => depth += 1,
            // `Map<K, V>` nests; `a < b` in a default does not.
            '<' if rest[..i].ends_with(|p: char| p.is_alphanumeric()) => depth += 1,
            '>' if !rest[..i].ends_with('-') && !rest[..i].ends_with('=') => depth -= 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ')' | ',' | ';' if depth == 0 => {
                let piece = rest[start..i].trim();
                // `mut self`, `var other: T`: the binding's mode is not its name.
                let piece = (piece
                    .strip_prefix("mut ")
                    .or_else(|| piece.strip_prefix("var ")))
                .unwrap_or(piece);
                let piece = piece.trim_start_matches('.');
                let word: String = piece
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                // `while ok: (T) -> Bool` is passed by its label, `while`.
                let after = piece[word.len()..].trim_start();
                let inner = after.trim_start_matches(|c: char| c.is_alphanumeric() || c == '_');
                let labelled = inner.len() < after.len() && inner.trim_start().starts_with(':');
                if !word.is_empty() && word != "self" && (after.starts_with(':') || labelled) {
                    let inner = after[..after.len() - inner.len()].to_string();
                    let inner = if labelled { inner } else { word.clone() };
                    names.push((word, inner, named));
                }
                named |= c == ';';
                start = i + 1;
                if c == ')' {
                    break;
                }
            }
            _ => {}
        }
    }
    names
}

/// The documented cost: the first `O(..)` of the doc's `cost:` line, else of
/// the doc, as plain text.
fn documented_cost(doc: &str) -> Option<String> {
    let line = doc
        .lines()
        .find_map(|l| l.trim().strip_prefix("cost:"))
        .unwrap_or(doc);
    let text = cpl_diag::plain::text(line);
    let start = text.find("O(")?;
    let mut depth = 0;
    for (i, c) in text[start..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[start..start + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// One use to check: the program, where the item's name is in it, and what
/// its declaration says.
struct Case {
    who: String,
    program: String,
    at: usize,
    name: String,
    member: bool,
    params: Vec<String>,
    cost: Option<String>,
    /// The parameters as the docs show them, each with whether it is a
    /// named option.
    shape: Vec<(String, bool)>,
}

fn is_word(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Where `name` is used in `code` as a word, after a `.` for a member.
fn use_of(code: &str, name: &str, member: bool) -> Option<usize> {
    let bytes = code.as_bytes();
    code.match_indices(name).map(|(i, _)| i).find(|&i| {
        let end = i + name.len();
        let before = i.checked_sub(1).map(|j| bytes[j]);
        let free = before.is_none_or(|b| !is_word(b) && b != b'.');
        (if member { before == Some(b'.') } else { free })
            && bytes.get(end).is_none_or(|&b| !is_word(b))
    })
}

/// An example's statements as a program, the last kept as a value.
fn program_of(example: &str) -> String {
    let mut lines: Vec<String> = example.split("; ").map(str::to_string).collect();
    if let Some(last) = lines.last_mut() {
        let t = last.trim_start();
        if !["let ", "var ", "for ", "print", "if "]
            .iter()
            .any(|k| t.starts_with(k))
            && !t.contains(" = ")
        {
            *last = format!("let got = {last}");
        }
    }
    lines.join("\n") + "\n"
}

/// The public items of the `.cp` modules in `dir`, imported as
/// `use {lib}.<module>`.
fn lib_cases(dir: &Path, lib: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    files.sort();
    for path in files {
        let module = path.file_stem().unwrap().to_string_lossy().to_string();
        if module == "prelude" || module == "spec" {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        let doc = cpl_doc::extract(&src, &path.to_string_lossy());
        // A library has no prelude: its example uses its module's other names.
        let every: Vec<&str> = doc
            .items
            .iter()
            .filter(|i| i.signature.trim_start().starts_with("pub "))
            .filter(|i| matches!(i.kind, cpl_doc::Kind::Fn | cpl_doc::Kind::Struct))
            .map(|i| i.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        for (owner, item) in doc.all_items() {
            let owner_item = owner.and_then(|o| doc.items.iter().find(|i| i.name == o));
            let public = |i: &cpl_doc::Item| i.signature.trim_start().starts_with("pub ");
            if !public(item) || owner_item.is_some_and(|o| !public(o)) {
                continue;
            }
            if !matches!(item.kind, cpl_doc::Kind::Fn | cpl_doc::Kind::Struct) {
                continue;
            }
            let takes_self =
                item.signature.contains("(self") || item.signature.contains("(mut self");
            let member = owner.is_some() || takes_self;
            // A constructor is used by its struct's name.
            let called = item.doc.contains(&format!(".{}(", item.name));
            let (name, member) = match (owner, item.name.as_str()) {
                (Some(o), "new") if !called => (o.to_string(), false),
                _ => (item.name.clone(), member),
            };
            let example = item.doc.lines().find_map(|l| {
                let code = l.trim().strip_prefix("example: `")?.strip_suffix('`')?;
                use_of(code, &name, member).map(|_| code.to_string())
            });
            let Some(example) = example else {
                continue;
            };
            let import = owner.unwrap_or(&item.name);
            let head = if lib != "std" {
                format!("use {lib}.{module}.{{{}}}\n", every.join(", "))
            } else if takes_self && owner.is_none() {
                format!("use {lib}.{module}.{{{}}}\n", item.name)
            } else {
                format!("use {lib}.{module}.{{{import}}}\n")
            };
            let program = format!("{head}{}", program_of(&example));
            let at = use_of(&program[head.len()..], &name, member).unwrap() + head.len();
            let who = match owner {
                Some(o) => format!("{lib}.{module}.{o}.{}", item.name),
                None => format!("{lib}.{module}.{}", item.name),
            };
            cases.push(Case {
                who,
                program,
                at,
                name,
                member,
                params: param_names(&item.signature, &item.name),
                cost: documented_cost(&item.doc).or_else(|| item.cost.clone()),
                shape: params_of(&item.signature, &item.name),
            });
        }
    }
    cases
}

fn builtin_cases() -> Vec<Case> {
    cpl_facade::today::BUILTIN_MEMBERS
        .iter()
        .filter_map(|m| {
            let program = format!("{FIXTURE}{}", program_of(m.example));
            let at = use_of(&program[FIXTURE.len()..], m.name, true)? + FIXTURE.len();
            let declared = format!("{}({})", m.name, m.params);
            let params = param_names(&declared, m.name);
            Some(Case {
                who: format!("{}.{}", m.receiver.name(), m.name),
                program,
                at,
                name: m.name.to_string(),
                member: true,
                params,
                cost: documented_cost(m.doc),
                shape: params_of(&declared, m.name),
            })
        })
        .collect()
}

#[derive(Default)]
struct Tally {
    pass: BTreeMap<&'static str, usize>,
    all: BTreeMap<&'static str, usize>,
    fails: BTreeMap<&'static str, Vec<String>>,
}

impl Tally {
    fn add(&mut self, other: Tally) {
        for (what, n) in other.pass {
            *self.pass.entry(what).or_default() += n;
        }
        for (what, n) in other.all {
            *self.all.entry(what).or_default() += n;
        }
        for (what, fails) in other.fails {
            self.fails.entry(what).or_default().extend(fails);
        }
    }

    fn check(&mut self, what: &'static str, ok: bool, who: &str, why: impl FnOnce() -> String) {
        *self.all.entry(what).or_default() += 1;
        if ok {
            *self.pass.entry(what).or_default() += 1;
        } else {
            self.fails
                .entry(what)
                .or_default()
                .push(format!("{who}: {}", why()));
        }
    }
}

fn run(lsp: &mut Lsp, case: &Case, uri: &str, tally: &mut Tally) {
    let text = &case.program;
    lsp.open(uri, text);
    let hover = lsp.ask("textDocument/hover", uri, text, case.at + 1);
    let shown = hover
        .at(&["contents", "value"])
        .as_str()
        .unwrap_or("")
        .to_string();
    let code = shown.split("\n```").next().unwrap_or("").to_string();
    let mut surfaces = Surfaces::default();
    if !shown.is_empty() {
        let doc = shown.split_once("\n```\n\n").map_or("", |(_, d)| d);
        surfaces.add("hover", &code, doc);
    }
    let missing = |s: &str| {
        case.params
            .iter()
            // `while:`, or `while ok:` as the declaration writes it.
            .filter(|p| {
                let labelled = |at: usize| {
                    let rest = s[at + p.len()..].strip_prefix(' ').unwrap_or("");
                    let inner = rest.trim_start_matches(|c: char| c.is_alphanumeric() || c == '_');
                    inner.len() < rest.len() && inner.starts_with(':')
                };
                !s.contains(&format!("{p}:"))
                    && !s.match_indices(p.as_str()).any(|(at, _)| labelled(at))
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    tally.check("hover", !shown.is_empty(), &case.who, || {
        format!("no hover in {text:?}")
    });
    if !shown.is_empty() {
        let gone = missing(&code);
        tally.check(
            "hover names every parameter",
            gone.is_empty(),
            &case.who,
            || format!("{gone:?} missing from {code:?}"),
        );
        if let Some(cost) = &case.cost {
            // In the program's own sizes: `O(|self|)` shows as `O(n)`.
            let ok = code.lines().any(|l| l.starts_with("cost: O("));
            tally.check("hover shows the documented cost", ok, &case.who, || {
                format!("want {cost} in {code:?}")
            });
        }
    }
    if let Some(module) = case.who.split('.').nth(1).filter(|_| uri != BUILTIN_URI) {
        let def = lsp.ask("textDocument/definition", uri, text, case.at + 1);
        let target = def.as_array().and_then(|a| a.first()).unwrap_or(&def);
        let reached = target
            .at(&["uri"])
            .as_str()
            .or(target.at(&["targetUri"]).as_str())
            .unwrap_or("")
            .to_string();
        tally.check(
            "definition reaches its module",
            reached.ends_with(&format!("/{module}.cp")),
            &case.who,
            || format!("went to {reached:?} from {text:?}"),
        );
    }
    let end = case.at + case.name.len();
    if text[end..].starts_with('(') {
        let sig = lsp.ask("textDocument/signatureHelp", uri, text, end + 1);
        let labels: Vec<String> = sig
            .at(&["signatures"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|s| s.at(&["label"]).as_str().unwrap_or("").to_string())
            .collect();
        let label = labels
            .iter()
            .find(|l| missing(l).is_empty())
            .or(labels.first())
            .cloned()
            .unwrap_or_default();
        if let Some(chosen) = sig
            .at(&["signatures"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .find(|s| s.at(&["label"]).as_str() == Some(label.as_str()))
        {
            let doc = chosen
                .at(&["documentation", "value"])
                .as_str()
                .unwrap_or("");
            surfaces.add("signature help", &label, doc);
        }
        let gone = missing(&label);
        tally.check(
            "signature help names every parameter",
            !label.is_empty() && gone.is_empty(),
            &case.who,
            || format!("{gone:?} missing from {label:?}"),
        );
        // Each named option, written as the first argument, is the one the
        // signature highlights.
        let options = label.find(';').map_or(0, |i| label[..i].chars().count());
        let names: Vec<String> = case
            .params
            .iter()
            .filter(|p| {
                let at = label
                    .find(&format!("{p}:"))
                    .map(|i| label[..i].chars().count());
                options > 0 && at.is_some_and(|at| at > options)
            })
            .cloned()
            .collect();
        for name in names {
            let written = format!("{}{name}: {}", &text[..end + 1], &text[end + 1..]);
            let cursor = end + 1 + name.len() + 2;
            lsp.close(uri);
            lsp.open(uri, &written);
            let sig = lsp.ask("textDocument/signatureHelp", uri, &written, cursor);
            lsp.close(uri);
            lsp.open(uri, text);
            let first = &sig
                .at(&["signatures"])
                .as_array()
                .unwrap_or(&[])
                .first()
                .cloned();
            let shown = first.as_ref().and_then(|s| {
                let chars: Vec<char> = s.at(&["label"]).as_str()?.chars().collect();
                let active = sig.at(&["activeParameter"]).as_u32()? as usize;
                let range = s
                    .at(&["parameters"])
                    .as_array()?
                    .get(active)?
                    .at(&["label"]);
                let range = range.as_array()?;
                let (a, b) = (
                    range.first()?.as_u32()? as usize,
                    range.get(1)?.as_u32()? as usize,
                );
                Some(chars.get(a..b.min(chars.len()))?.iter().collect::<String>())
            });
            let shown = shown.unwrap_or_default();
            tally.check(
                "signature help shows the active named option",
                shown.starts_with(&format!("{name}:")),
                &case.who,
                || {
                    format!(
                        "`{name}:` written, {shown:?} highlighted in {:?}",
                        first
                            .as_ref()
                            .map(|s| s.at(&["label"]).as_str().unwrap_or("").to_string())
                    )
                },
            );
        }
    }
    lsp.close(uri);
    // Completion mid-edit: the line typed up to just after the `.`, or up to
    // the name's first letter.
    let keep = if case.member { 0 } else { 1 };
    let eol = text[end..].find('\n').map_or(text.len(), |i| end + i);
    let typed = format!("{}{}", &text[..case.at + keep], &text[eol..]);
    lsp.open(uri, &typed);
    let reply = lsp.ask("textDocument/completion", uri, &typed, case.at + keep);
    lsp.close(uri);
    let items = reply
        .at(&["items"])
        .as_array()
        .or(reply.as_array())
        .unwrap_or(&[])
        .to_vec();
    let found = items
        .iter()
        .find(|i| i.at(&["label"]).as_str() == Some(case.name.as_str()));
    tally.check("completion offers it", found.is_some(), &case.who, || {
        let line = typed[..case.at + keep].rsplit('\n').next().unwrap_or("");
        format!("not offered after {line:?}")
    });
    if let Some(item) = found {
        let detail = item.at(&["detail"]).as_str().unwrap_or("");
        let doc = match item.at(&["documentation", "value"]).as_str() {
            Some(doc) => doc.to_string(),
            None => lsp
                .request("completionItem/resolve", item.clone())
                .at(&["documentation", "value"])
                .as_str()
                .unwrap_or("")
                .to_string(),
        };
        surfaces.add("completion", detail, &doc);
        let gone = missing(detail);
        tally.check(
            "completion detail names every parameter",
            gone.is_empty(),
            &case.who,
            || format!("{gone:?} missing from {detail:?}"),
        );
    }
    surfaces.agree(case, tally);
}

/// What each editor surface showed for one item: its text and its doc.
#[derive(Default)]
struct Surfaces(Vec<(&'static str, String, String)>);

impl Surfaces {
    fn add(&mut self, surface: &'static str, text: &str, doc: &str) {
        self.0.push((surface, text.to_string(), doc.to_string()));
    }

    /// Every surface that showed the item renders the signature the docs
    /// show, the same parameters in the same order with the same named
    /// options, and the same doc comment as the docs.
    fn agree(&self, case: &Case, tally: &mut Tally) {
        for (surface, text, _) in &self.0 {
            // A struct's completion is its declaration, then the call `new` takes.
            let call = format!("\n{}(", case.name);
            let text = text.rfind(&call).map_or(text.as_str(), |i| &text[i + 1..]);
            let shape = params_of(text, &case.name);
            let signs = text.contains(&format!("{}(", case.name));
            tally.check(
                "surfaces show the docs' parameters",
                !signs || shape == case.shape,
                &case.who,
                || {
                    format!(
                        "{surface} shows {shape:?} in {text:?}, the docs {:?}",
                        case.shape
                    )
                },
            );
        }
    }
}

/// Runs the cases on one server per core, dealt round-robin since a
/// module's cases cost alike, and adds up the tallies.
fn run_all(cases: &[Case], uri: &str) -> Tally {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let tallies: Vec<Tally> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|first| {
                scope.spawn(move || {
                    let mut lsp = Lsp::new();
                    let mut tally = Tally::default();
                    for case in cases.iter().skip(first).step_by(threads) {
                        run(&mut lsp, case, uri, &mut tally);
                    }
                    tally
                })
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    });
    let mut all = Tally::default();
    for tally in tallies {
        all.add(tally);
    }
    all
}

fn print(name: &str, tally: &Tally) {
    for (what, fails) in &tally.fails {
        for f in fails.iter().take(400) {
            println!("{name} fails {what}: {f:.240}");
        }
    }
    for (what, all) in &tally.all {
        let pass = tally.pass.get(what).copied().unwrap_or(0);
        println!("{name} {what}: {pass}/{all}");
    }
}

/// Gates each check's failures, not its passes, so deleting an item never
/// trips it and a new failing item always does.
fn report(name: &str, tally: &Tally, ceilings: &[(&str, usize)]) {
    print(name, tally);
    let mut over = Vec::new();
    for (what, all) in &tally.all {
        let fails = all - tally.pass.get(what).copied().unwrap_or(0);
        let ceiling = ceilings.iter().find(|(w, _)| w == what).map_or(0, |c| c.1);
        if fails > ceiling {
            over.push(format!("{what}: {fails} failures, ceiling {ceiling}"));
        } else if fails < ceiling {
            println!("{name} {what}: {fails} failures, lower the ceiling from {ceiling}");
        }
    }
    for (what, _) in ceilings {
        if !tally.all.contains_key(*what) {
            over.push(format!("{what}: a ceiling for a check that never ran"));
        }
    }
    assert!(over.is_empty(), "{name} went over its ceilings: {over:?}");
}

#[test]
fn every_public_std_item_hovers_signs_and_completes_with_its_parameters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cases = lib_cases(&root.join("crates/cpl-check/frozen/std"), "std");
    let tally = run_all(&cases, STD_URI);
    report(
        "std",
        &tally,
        &[
            ("hover", 34),
            ("hover names every parameter", 19),
            ("hover shows the documented cost", 214),
            ("signature help names every parameter", 60),
            ("completion offers it", 69),
            ("completion detail names every parameter", 90),
            ("definition reaches its module", 87),
        ],
    );
}

#[test]
fn every_builtin_member_hovers_signs_and_completes_with_its_parameters() {
    let tally = run_all(&builtin_cases(), BUILTIN_URI);
    report(
        "builtin",
        &tally,
        &[
            ("hover names every parameter", 3),
            ("completion detail names every parameter", 3),
        ],
    );
}

/// A std module's text with its top-level public names given a `_mine` (or
/// `Mine`) suffix everywhere in the file, docs included, so the library's
/// items don't meet std's through the prelude.
fn own_names(text: &str, module: &str) -> String {
    let doc = cpl_doc::extract(text, "mine.cp");
    let names: Vec<&str> = doc
        .items
        .iter()
        .filter(|i| i.signature.trim_start().starts_with("pub "))
        .filter(|i| matches!(i.kind, cpl_doc::Kind::Fn | cpl_doc::Kind::Struct))
        // A `= todo` body is the compiler's own, found by its std name; a name
        // also declared as a member would rename the member too.
        .filter(|i| {
            let declares = |l: &str| {
                [format!("fn {}<", i.name), format!("fn {}(", i.name)]
                    .iter()
                    .any(|d| l.contains(d.as_str()))
            };
            !text
                .lines()
                .any(|l| declares(l) && (l.starts_with(' ') || l.trim_end().ends_with("= todo")))
        })
        .map(|i| i.name.as_str())
        .collect();
    let (examined, own) = own_member_uses(text, module, &names);
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 1024);
    let mut i = 0;
    while i < text.len() {
        let start = i;
        if is_word(bytes[i]) {
            while i < text.len() && is_word(bytes[i]) {
                i += 1;
            }
            let word = &text[start..i];
            out.push_str(word);
            // `D.count` is the domain parameter's own member, not the
            // module's `count`.
            let on_type = text[..start].strip_suffix('.').is_some_and(|before| {
                let head = before.trim_end_matches(|c: char| c.is_ascii_alphanumeric() || c == '_');
                let receiver = &before[head.len()..];
                receiver.len() == 1 && receiver.starts_with(|c: char| c.is_ascii_uppercase())
            });
            // `w.map(.len)` on an option is the option's own `map`.
            let built_in = text[..start].ends_with('.')
                && cpl_facade::today::BUILTIN_MEMBERS
                    .iter()
                    .any(|m| m.name == word);
            let on_type = match examined.contains(&start) {
                true => !own.contains(&start),
                false => on_type || built_in,
            };
            // `lo min= x` is the compound-assignment token, not a call.
            let assigns = text[i..].starts_with('=') && !text[i..].starts_with("==");
            if names.contains(&word)
                && !on_type
                && !assigns
                && !(start > 0 && is_word(bytes[start - 1]))
            {
                let upper = word.starts_with(|c: char| c.is_ascii_uppercase());
                out.push_str(if upper { "Mine" } else { "_mine" });
            }
        } else {
            let c = text[i..].chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// The `.name` uses of the module's own `names` in its doc examples, by byte
/// offset in `text`, and those of them the checker resolves to the module's
/// own declaration: `V(0).l` is a position's built-in `l`, not range's.
fn own_member_uses(
    text: &str,
    module: &str,
    names: &[&str],
) -> (
    std::collections::HashSet<usize>,
    std::collections::HashSet<usize>,
) {
    let mut session = cpl_facade::Session::default();
    let head = format!("use std.{module}.{{{}}}\n", names.join(", "));
    let (mut examined, mut own) = (
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
    );
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        let mut rest = 0;
        while line.trim_start().starts_with("///") {
            let Some(open) = line[rest..].find('`').map(|o| rest + o + 1) else {
                break;
            };
            let Some(len) = line[open..].find('`') else {
                break;
            };
            let code = &line[open..open + len];
            rest = open + len + 1;
            let uses: Vec<usize> = names
                .iter()
                .flat_map(|n| {
                    let dotted = format!(".{n}");
                    let at: Vec<(usize, usize)> = code
                        .match_indices(&dotted)
                        .map(|(u, _)| (u + 1, n.len()))
                        .collect();
                    at
                })
                .filter(|&(u, n)| {
                    !code[u + n..].starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
                })
                .map(|(u, _)| u)
                .collect();
            if uses.is_empty() {
                continue;
            }
            let program = format!("{head}{}\n", code.replace("; ", "\n"));
            session = session.with_file("/w/example.cp", program.as_str());
            // Prose like `s.min()` names no value, so the checker can't say.
            let checks = session
                .diagnostics("/w/example.cp", cpl_facade::Layer::Typed)
                .iter()
                .all(|d| d.severity != cpl_diag::Severity::Error);
            for u in uses.into_iter().filter(|_| checks) {
                let shift = code[..u].matches("; ").count();
                let offset = (head.len() + u - shift + 1) as u32;
                let def = session.definition("/w/example.cp", offset);
                examined.insert(at + open + u);
                // `[5, 6, 7][1..].to_list()` takes the `Slice` syntax builds, which the
                // checker reads as its source: a copy's own `SliceMine` can't take it.
                let dot = offset - 2;
                let facts = session.facts("/w/example.cp");
                let receiver = (facts.iter())
                    .flat_map(|f| f.types_in(cpl_diag::Span { start: 0, end: dot }))
                    .filter(|(s, _)| s.end == dot)
                    .max_by_key(|(s, _)| s.end - s.start)
                    .map(|(_, ty)| ty.clone());
                let takes = def.as_ref().and_then(|d| {
                    let decl = text[d.span.start as usize..].lines().next()?;
                    let ty = decl.split_once("self: ")?.1;
                    Some(ty[..ty.find(|c: char| !is_word(c as u8)).unwrap_or(ty.len())].to_string())
                });
                let built = takes.is_some_and(|t| {
                    names.contains(&t.as_str())
                        && !matches!(&receiver, Some(cpl_facade::today::Ty::Named { name, .. }) if *name == t)
                });
                if !built && def.is_some_and(|d| d.path.ends_with(&format!("/{module}.cp"))) {
                    own.insert(at + open + u);
                }
            }
        }
        at += line.len();
    }
    (examined, own)
}

/// A copy of a module renames a member used on a value its own fn built,
/// and keeps std's name for one used on a value syntax built.
#[test]
fn a_copy_keeps_std_names_on_values_syntax_builds() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("std/seq.cp")).unwrap();
    let copy = own_names(&text, "seq");
    assert!(copy.contains("`[5, 6, 7][1..].to_list() == [6, 7]`"));
    assert!(copy.contains("pub fn to_list_mine<"));
    assert!(copy.contains("a.filter_mine(x => x > 0).collect_mine() == []"));
}

const SHARDS: usize = 4;

/// The std item a personal-library item copies; the pair shares a shard.
fn std_who(who: &str) -> String {
    who.replacen("mine.", "std.", 1)
        .replace("_mine", "")
        .replace("Mine", "")
}

/// std copied into `dir/mine` with its names given their own suffixes, as a
/// library `dir/cp.toml` names: its renamed items' cases and the file URI
/// they open as.
fn personal_library(dir: &Path) -> (Vec<Case>, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir.join("mine")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nmine = \"mine\"\n").unwrap();
    let modules: Vec<_> = std::fs::read_dir(root.join("crates/cpl-check/frozen/std"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    // Each module's copy checks its own examples, so the copies go a core each.
    std::thread::scope(|scope| {
        for path in &modules {
            scope.spawn(move || {
                let text = std::fs::read_to_string(path).unwrap();
                std::fs::write(
                    dir.join("mine").join(path.file_name().unwrap()),
                    own_names(&text, &path.file_stem().unwrap().to_string_lossy()),
                )
                .unwrap();
            });
        }
    });
    let shown = dir.display().to_string().replace('\\', "/");
    let uri = format!(
        "file://{}{shown}/a.cp",
        if shown.starts_with('/') { "" } else { "/" }
    );
    // Only the renamed items: the rest meet std's through the prelude.
    let mut cases = lib_cases(&dir.join("mine"), "mine");
    cases.retain(|c| {
        c.who
            .split('.')
            .nth(2)
            .is_some_and(|n| n.ends_with("Mine") || n.ends_with("_mine"))
    });
    // An example that reaches the item through a std name (`xs.rev()`) tests std.
    let own = |t: &str| t.contains("Mine") || t.contains("_mine");
    cases.retain(|c| c.program.lines().skip(1).any(own));
    (cases, uri)
}

fn library_dir(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn library_keys() -> Vec<String> {
    let (cases, _) = personal_library(&library_dir("personal-lib-keys"));
    cases.iter().map(|c| std_who(&c.who)).collect()
}

#[test]
fn a_personal_library_has_over_1900_cases() {
    let (cases, _) = personal_library(&library_dir("personal-lib-count"));
    println!("mine cases: {}", cases.len());
    assert!(cases.len() >= 1900, "only {} library cases", cases.len());
}

/// A personal library (`cp.toml`'s `[libs]`) holding a copy of every std
/// module gets what std gets: each item hovers, signs, completes and goes to
/// its definition as often as the same item imported from std.
/// Shard `k` holds the items whose std copy hashes to it.
fn a_personal_library_gets_what_std_gets(k: usize) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (cases, uri) = personal_library(&library_dir(&format!("personal-lib-{k}")));
    let cases = shards::mine(cases, |c| std_who(&c.who), k, SHARDS);
    let std_cases = lib_cases(&root.join("crates/cpl-check/frozen/std"), "std");
    let std = run_all(
        &shards::mine(std_cases, |c| c.who.clone(), k, SHARDS),
        STD_URI,
    );
    let mine = run_all(&cases, &uri);
    print("mine", &mine);
    let mut only = Vec::new();
    for (what, fails) in &mine.fails {
        let theirs: Vec<&str> = std.fails.get(what).map_or(vec![], |f| {
            f.iter().filter_map(|f| f.split(':').next()).collect()
        });
        for f in fails {
            let who = std_who(f.split(':').next().unwrap_or(""));
            if !theirs.contains(&who.as_str()) {
                only.push(format!("{what}: {f:.300}"));
            }
        }
    }
    assert!(
        only.is_empty(),
        "the library falls behind std on {} items:\n{}",
        only.len(),
        only.join("\n")
    );
}

shards::shard_tests!(
    a_personal_library_gets_what_std_gets,
    library_keys(),
    the_library_shards_cover_every_case,
    [
        a_personal_library_gets_what_std_gets_0 = 0,
        a_personal_library_gets_what_std_gets_1 = 1,
        a_personal_library_gets_what_std_gets_2 = 2,
        a_personal_library_gets_what_std_gets_3 = 3,
    ]
);
