//! Every public std item (`cpl_doc::extract` over `std/*.cp`) and every
//! built-in member row (`cpl_check::BUILTIN_MEMBERS`), used in a small
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
                       var st = Set<Int>()\nst.insert(4)\nst.insert(9)\nvar a: [0..3]Int = 0\na[1] = 7\n";

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
    let (mut depth, mut start) = (0i32, open + 1);
    for (i, c) in rest[open..]
        .char_indices()
        .skip(1)
        .map(|(i, c)| (i + open, c))
    {
        match c {
            '(' | '[' | '{' => depth += 1,
            '<' => depth += 1,
            '>' if !rest[..i].ends_with('-') && !rest[..i].ends_with('=') => depth -= 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            ')' | ',' | ';' if depth == 0 => {
                let piece = rest[start..i].trim();
                let piece = piece.strip_prefix("mut ").unwrap_or(piece);
                let piece = piece.strip_prefix("...").unwrap_or(piece);
                let word: String = piece
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !word.is_empty()
                    && word != "self"
                    && piece[word.len()..].trim_start().starts_with(':')
                {
                    names.push(word);
                }
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
            });
        }
    }
    cases
}

fn builtin_cases() -> Vec<Case> {
    cpl_check::BUILTIN_MEMBERS
        .iter()
        .filter_map(|m| {
            let program = format!("{FIXTURE}{}", program_of(m.example));
            let at = use_of(&program[FIXTURE.len()..], m.name, true)? + FIXTURE.len();
            let params = param_names(&format!("{}({})", m.name, m.params), m.name);
            Some(Case {
                who: format!("{}.{}", m.receiver.name(), m.name),
                program,
                at,
                name: m.name.to_string(),
                member: true,
                params,
                cost: documented_cost(m.doc),
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
    let missing = |s: &str| {
        case.params
            .iter()
            .filter(|p| !s.contains(&format!("{p}:")))
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
        format!("not offered in {typed:?}")
    });
    if let Some(item) = found {
        let detail = item.at(&["detail"]).as_str().unwrap_or("");
        let gone = missing(detail);
        tally.check(
            "completion detail names every parameter",
            gone.is_empty(),
            &case.who,
            || format!("{gone:?} missing from {detail:?}"),
        );
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
    let tally = run_all(&lib_cases(&root.join("std"), "std"), STD_URI);
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
fn own_names(text: &str) -> String {
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
            if names.contains(&word) && !(start > 0 && is_word(bytes[start - 1])) {
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

/// A personal library (`cp.toml`'s `[libs]`) holding a copy of every std
/// module gets what std gets: each item hovers, signs, completes and goes to
/// its definition as often as the same item imported from std.
#[test]
fn a_personal_library_gets_what_std_gets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("personal-lib");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("mine")).unwrap();
    std::fs::write(dir.join("cp.toml"), "[libs]\nmine = \"mine\"\n").unwrap();
    for entry in std::fs::read_dir(root.join("std")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "cp") {
            let text = std::fs::read_to_string(&path).unwrap();
            std::fs::write(
                dir.join("mine").join(path.file_name().unwrap()),
                own_names(&text),
            )
            .unwrap();
        }
    }
    let shown = dir.display().to_string().replace('\\', "/");
    let uri = format!(
        "file://{}{shown}/a.cp",
        if shown.starts_with('/') { "" } else { "/" }
    );
    let std = run_all(&lib_cases(&root.join("std"), "std"), STD_URI);
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
    println!("mine cases: {}", cases.len());
    assert!(cases.len() >= 1900, "only {} library cases", cases.len());
    let mine = run_all(&cases, &uri);
    print("mine", &mine);
    let mut only = Vec::new();
    for (what, fails) in &mine.fails {
        let theirs: Vec<&str> = std.fails.get(what).map_or(vec![], |f| {
            f.iter().filter_map(|f| f.split(':').next()).collect()
        });
        for f in fails {
            let who = f
                .split(':')
                .next()
                .unwrap_or("")
                .replacen("mine.", "std.", 1)
                .replace("_mine", "")
                .replace("Mine", "");
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
