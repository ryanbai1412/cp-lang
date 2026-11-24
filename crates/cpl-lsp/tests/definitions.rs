//! Go to definition over every conformance case and every std module
//! (`commands/R14`): the share of first uses of a name that resolve is a
//! floor per set in `definition-floors.json` that only rises, and every
//! definition a program's name gets in std lands on that name's
//! declaration, on a line `cpl doc` gives a docs page anchor, which is
//! where the site's editors send it (`/docs/ref/std/<module>#<anchor>`).
//! The wasm module is held to the same floors by `crates/cpl-wasm/tests/smoke.mjs`.

use cpl_json::{obj, Value};
use cpl_lsp::Server;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

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

fn cp_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Holds `set`'s measured share to its floor in `definition-floors.json`
/// both ways, as the debt ratchet does: under it is a regression, over it
/// is a gain the file must record, so the floors rise without anyone
/// remembering to raise them. The goal is 1000 in every set.
fn hold(set: &str, permille: usize) {
    let path = root().join("crates/cpl-lsp/tests/definition-floors.json");
    let floors = cpl_json::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let floor = floors.at(&[set]).as_u32().expect("a floor per set") as usize;
    assert!(
        permille >= floor,
        "{set}: definition fell to {permille} permille, under {floor}"
    );
    assert!(
        permille == floor,
        "{set}: definition rose to {permille} permille: set \"{set}\": {permille} in crates/cpl-lsp/tests/definition-floors.json"
    );
}

/// One definition asked: the file, the name, and where it went.
struct Asked {
    file: String,
    name: String,
    line: String,
    reply: Value,
}

/// Opens each file at its own path, asks definition at the first use of
/// every name (an identifier token: keywords, literals and a decorator's
/// own words are not names), and closes it.
fn ask_all(files: &[PathBuf]) -> Vec<Asked> {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    let mut id = 1;
    let mut out = Vec::new();
    for path in files {
        let src = std::fs::read_to_string(path).unwrap();
        let uri = format!("file://{}", path.display());
        let doc = || obj([("uri", uri.as_str().into())]);
        let item = obj([
            ("uri", uri.as_str().into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", src.as_str().into()),
        ]);
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([("textDocument", item)]),
        ));
        let mut seen = HashSet::new();
        let tokens = cpl_syntax::lex(&src).tokens;
        let words = decorator_words(&tokens, &src);
        for (at, token) in tokens.iter().enumerate() {
            let (start, end) = (token.span.start as usize, token.span.end as usize);
            let name = &src[start..end];
            if token.kind != cpl_syntax::TokenKind::Ident
                || cpl_syntax::LITERAL_WORDS.contains(&name)
                || words.contains(&at)
                || !seen.insert(name)
            {
                continue;
            }
            id += 1;
            let params = obj([("textDocument", doc()), ("position", position(&src, start))]);
            let replies = server.handle(&message(Some(id), "textDocument/definition", params));
            let reply = replies
                .into_iter()
                .find(|r| r.get("id").is_some())
                .unwrap_or(Value::Null);
            let line_start = src[..start].rfind('\n').map_or(0, |i| i + 1);
            let line_end = src[start..].find('\n').map_or(src.len(), |i| start + i);
            out.push(Asked {
                file: path
                    .strip_prefix(root())
                    .unwrap_or(path)
                    .display()
                    .to_string(),
                name: name.to_string(),
                line: src[line_start..line_end].trim().to_string(),
                reply: reply.at(&["result"]).clone(),
            });
        }
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", doc())]),
        ));
    }
    out
}

/// The tokens a decorator spells as its own vocabulary, which no program
/// declares: its name (`@lint`, `@!multitest`), its argument labels
/// (`from:`) and its bare-word arguments (the lint id, `allow`, `program`).
/// Names inside an argument's expression are still names.
fn decorator_words(tokens: &[cpl_syntax::Token], src: &str) -> HashSet<usize> {
    use cpl_syntax::TokenKind::{At, Colon, Comma, Ident, LParen, RParen};
    let kind = |i: usize| tokens.get(i).map(|t| t.kind);
    let mut words = HashSet::new();
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i].kind != At {
            i += 1;
            continue;
        }
        i += 1;
        if tokens
            .get(i)
            .is_some_and(|t| &src[t.span.start as usize..t.span.end as usize] == "!")
        {
            i += 1;
        }
        if kind(i) != Some(Ident) {
            continue;
        }
        words.insert(i);
        i += 1;
        if kind(i) != Some(LParen) {
            continue;
        }
        let mut depth = 0;
        while i < tokens.len() {
            match tokens[i].kind {
                LParen => depth += 1,
                RParen => depth -= 1,
                Ident if depth == 1 => {
                    let labelled = kind(i + 1) == Some(Colon);
                    let bare = matches!(kind(i - 1), Some(LParen | Comma))
                        && matches!(kind(i + 1), Some(Comma | RParen));
                    if labelled || bare {
                        words.insert(i);
                    }
                }
                _ => {}
            }
            i += 1;
            if depth == 0 {
                break;
            }
        }
    }
    words
}

#[test]
fn a_decorators_own_words_are_not_names() {
    let src = "@lint(chmin_chmax_if, from: \"x = min(x, e)\", holes: \"x, e\")\n@!multitest\n@memo(map)\n@tag(f(k), by: g)\nfn f(k: Int) = k\n";
    let tokens = cpl_syntax::lex(src).tokens;
    let words = decorator_words(&tokens, src);
    let mut marked: Vec<&str> = words
        .iter()
        .map(|&i| &src[tokens[i].span.start as usize..tokens[i].span.end as usize])
        .collect();
    marked.sort_unstable();
    assert_eq!(
        marked,
        [
            "by",
            "chmin_chmax_if",
            "from",
            "holes",
            "lint",
            "map",
            "memo",
            "multitest",
            "tag"
        ]
    );
}

/// [`ask_all`] over `files` in a few threads with room for deep programs.
fn ask_parallel(files: &[PathBuf]) -> Vec<Asked> {
    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(8);
    let chunk = files.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = files
            .chunks(chunk)
            .map(|files| {
                std::thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(scope, move || ask_all(files))
                    .unwrap()
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    })
}

fn locations(reply: &Value) -> Vec<&Value> {
    match reply {
        Value::Array(all) => all.iter().collect(),
        Value::Null => Vec::new(),
        one => vec![one],
    }
}

/// The share resolved, in permille, after printing the names most often
/// left unresolved.
fn resolved(set: &str, asked: &[Asked]) -> usize {
    let mut misses: BTreeMap<&str, (usize, String)> = BTreeMap::new();
    let mut hits = 0;
    for a in asked {
        if locations(&a.reply).is_empty() {
            let first = || format!("{}: {}", a.file, a.line);
            misses.entry(&a.name).or_insert_with(|| (0, first())).0 += 1;
        } else {
            hits += 1;
        }
    }
    let mut misses: Vec<_> = misses.into_iter().collect();
    misses.sort_by_key(|(name, (n, _))| (std::cmp::Reverse(*n), *name));
    for (name, (n, first)) in misses.iter().take(30) {
        println!("{set}: no definition {n:5}  {name}  (first: {first:.100})");
    }
    let permille = hits * 1000 / asked.len().max(1);
    println!(
        "{set}: definition {hits}/{} first uses of a name = {permille} permille",
        asked.len()
    );
    permille
}

/// The module and 0-based line of a location in a std file, and its text.
fn in_std(location: &Value) -> Option<(String, u32, u32, u32)> {
    let uri = location.at(&["uri"]).as_str()?;
    let module = uri.rsplit_once("/std/")?.1.strip_suffix(".cp")?;
    let start = location.at(&["range", "start"]);
    let end = location.at(&["range", "end"]);
    let line = start.at(&["line"]).as_u32()?;
    let (from, to) = (
        start.at(&["character"]).as_u32()?,
        end.at(&["character"]).as_u32()?,
    );
    Some((module.replace('/', "."), line, from, to))
}

/// Each std module's items, by the 1-based line `cpl doc` anchors them at.
fn documented() -> BTreeMap<String, BTreeMap<u32, Vec<String>>> {
    let modules = cpl_doc::cli::load(&[root().join("std")]).unwrap();
    let items = |m: &cpl_doc::Module| {
        let mut lines: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        for (_, item) in m.all_items() {
            lines.entry(item.line).or_default().push(item.name.clone());
        }
        lines
    };
    modules.iter().map(|m| (m.name.clone(), items(m))).collect()
}

/// Every place a definition sends a program's name into std is on a docs
/// page: a module's first line for a segment of a `use` path naming it,
/// else the name of an item its module's page has an anchor for.
fn std_landings(asked: &[Asked], failures: &mut Vec<String>) -> usize {
    let docs = documented();
    let mut landed = 0;
    for a in asked {
        for location in locations(&a.reply) {
            let Some((module, line, from, to)) = in_std(location) else {
                continue;
            };
            landed += 1;
            let site = format!("{}: `{}` in `{:.80}`", a.file, a.name, a.line);
            let Some(items) = docs.get(&module) else {
                failures.push(format!(
                    "{site} lands in std/{module}.cp, which has no docs page"
                ));
                continue;
            };
            if (line, from, to) == (0, 0, 0) && module.rsplit('.').next() == Some(a.name.as_str()) {
                continue;
            }
            let path = root()
                .join("std")
                .join(format!("{}.cp", module.replace('.', "/")));
            let text = std::fs::read_to_string(&path).unwrap_or_default();
            let declared = text
                .lines()
                .nth(line as usize)
                .and_then(|l| l.get(from as usize..to as usize));
            // The anchor the site opens: the last items at or above the line.
            let item = items.range(..=line + 1).next_back().map(|(_, names)| names);
            let named = |name: &str| declared == Some(name);
            if !item.is_some_and(|names| named(&a.name) || names.iter().any(|n| named(n))) {
                failures.push(format!(
                    "{site} lands on {declared:?} at std/{module}.cp:{}, under item {item:?}",
                    line + 1
                ));
            }
        }
    }
    landed
}

#[test]
fn every_conformance_case_resolves_its_names_and_std_lands_on_docs() {
    let mut files = Vec::new();
    cp_files(&root().join("spec/conformance"), &mut files);
    files.sort();
    assert!(files.len() >= 500, "{} conformance cases", files.len());
    let asked = ask_parallel(&files);
    let mut failures = Vec::new();
    let landed = std_landings(&asked, &mut failures);
    let permille = resolved("conformance", &asked);
    assert!(
        failures.is_empty(),
        "{} of {landed} definitions into std miss their name or its docs page:\n{}",
        failures.len(),
        failures.join("\n")
    );
    assert!(landed >= 1000, "only {landed} definitions reach std");
    hold("conformance", permille);
}

#[test]
fn every_std_module_resolves_its_names() {
    let mut files = Vec::new();
    cp_files(&root().join("std"), &mut files);
    files.sort();
    assert!(files.len() >= 30, "{} std modules", files.len());
    let asked = ask_parallel(&files);
    let permille = resolved("std", &asked);
    hold("std", permille);
}
