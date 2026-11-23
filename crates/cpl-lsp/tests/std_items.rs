//! Every public std item (`cpl_doc::extract` over `std/*.cp`) and every
//! built-in member row (`cpl_check::BUILTIN_MEMBERS`), used in a small
//! program through `cpl_lsp::Server`: hover and signature help must show the
//! signature with every declared parameter name (`by:`, `cmp:`), hover the
//! documented cost, and completion must offer the item with a detail naming
//! the same parameters. The program is the item's first doc `example:` that
//! uses it. Each check's pass count is a floor that only goes up; failures
//! are printed as the backlog.
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

fn std_cases(root: &Path) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(root.join("std"))
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
            let head = if takes_self && owner.is_none() {
                format!("use std.{module}.{{{}}}\n", item.name)
            } else {
                format!("use std.{module}.{{{import}}}\n")
            };
            let program = format!("{head}{}", program_of(&example));
            let at = use_of(&program[head.len()..], &name, member).unwrap() + head.len();
            let who = match owner {
                Some(o) => format!("std.{module}.{o}.{}", item.name),
                None => format!("std.{module}.{}", item.name),
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

fn run(lsp: &mut Lsp, case: &Case, tally: &mut Tally) {
    let uri = "file:///std_items/a.cp";
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

fn report(name: &str, tally: &Tally, floors: &[(&str, usize)]) {
    for (what, fails) in &tally.fails {
        for f in fails.iter().take(400) {
            println!("{name} fails {what}: {f:.240}");
        }
    }
    let mut low = Vec::new();
    for (what, all) in &tally.all {
        let pass = tally.pass.get(what).copied().unwrap_or(0);
        println!("{name} {what}: {pass}/{all}");
        let floor = floors.iter().find(|(w, _)| w == what).map_or(0, |f| f.1);
        if pass < floor {
            low.push(format!("{what}: {pass} below {floor}"));
        }
    }
    assert!(low.is_empty(), "{name} fell below its floors: {low:?}");
}

#[test]
fn every_public_std_item_hovers_signs_and_completes_with_its_parameters() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut lsp = Lsp::new();
    let mut tally = Tally::default();
    for case in std_cases(&root) {
        run(&mut lsp, &case, &mut tally);
    }
    report(
        "std",
        &tally,
        &[
            ("hover", 1931),
            ("hover names every parameter", 1915),
            ("hover shows the documented cost", 1782),
            ("signature help names every parameter", 1565),
            ("completion offers it", 1783),
            ("completion detail names every parameter", 1708),
        ],
    );
}

#[test]
fn every_builtin_member_hovers_signs_and_completes_with_its_parameters() {
    let mut lsp = Lsp::new();
    let mut tally = Tally::default();
    for case in builtin_cases() {
        run(&mut lsp, &case, &mut tally);
    }
    report(
        "builtin",
        &tally,
        &[
            ("hover", 150),
            ("hover names every parameter", 147),
            ("hover shows the documented cost", 105),
            ("signature help names every parameter", 121),
            ("completion offers it", 149),
            ("completion detail names every parameter", 146),
        ],
    );
}
