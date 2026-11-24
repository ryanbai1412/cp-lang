//! Every hill-climb port (`corpus/hillclimb.json`), opened whole: the server
//! publishes diagnostics, and answers hover and go-to-definition on the first
//! use of every name and completion after every `name.`, without an internal
//! error. Hover on a declared array, optional or struct says what it stores.
//! Correctness only: latency is `tests/latency.rs`'s job, since a wall-clock
//! limit here fails whenever the machine is loaded.

use cpl_json::{obj, Value};

#[path = "../../cpl-cli/tests/support/corpus_programs.rs"]
mod corpus_programs;
use cpl_lsp::Server;
use std::collections::HashSet;
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

/// Byte offsets of the first occurrence of each identifier outside comments
/// and strings, and of each `.` that follows one.
fn sites(src: &str) -> (Vec<usize>, Vec<usize>) {
    let (mut names, mut dots, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
    let mut at = 0;
    for line in src.split_inclusive('\n') {
        let code = line.split("//").next().unwrap_or("");
        let bytes = code.as_bytes();
        let mut i = 0;
        let mut quoted = false;
        while i < bytes.len() {
            let c = bytes[i];
            if c == b'"' {
                quoted = !quoted;
            }
            if !quoted && (c.is_ascii_alphabetic() || c == b'_') {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                if seen.insert(&code[start..i]) {
                    names.push(at + start);
                }
                if bytes.get(i) == Some(&b'.') && bytes.get(i + 1) != Some(&b'.') {
                    dots.push(at + i + 1);
                }
                continue;
            }
            i += 1;
        }
        at += line.len();
    }
    (names, dots)
}

/// Go to definition over first uses of names: how many it asked, how many
/// answered with a location, and the names it left unresolved.
#[derive(Default)]
struct Definitions {
    asked: usize,
    resolved: usize,
    misses: std::collections::BTreeMap<String, usize>,
    /// Each unresolved name's first line, `file: text`.
    first: std::collections::BTreeMap<String, String>,
}

/// Opens `src`, asks hover and definition at every first use of a name and
/// completion after every `name.`, then closes it. Records each missing or
/// failed reply, each diagnostic outside the text, and each request slower
/// than a second (a hang) in `failures`; returns whether some hover reports
/// storage.
fn walk(
    server: &mut Server,
    id: &mut u32,
    name: &str,
    src: &str,
    failures: &mut Vec<String>,
    defs: &mut Definitions,
) -> bool {
    let uri = format!("file:///ports/{name}");
    let doc = || obj([("uri", uri.as_str().into())]);
    let open = server.handle(&message(
        None,
        "textDocument/didOpen",
        obj([(
            "textDocument",
            obj([
                ("uri", uri.as_str().into()),
                ("languageId", "cp".into()),
                ("version", 1.into()),
                ("text", src.into()),
            ]),
        )]),
    ));
    let published: Vec<&Value> = open
        .iter()
        .filter(|m| m.at(&["method"]).as_str() == Some("textDocument/publishDiagnostics"))
        .collect();
    if published.is_empty() {
        failures.push(format!("{name}: no diagnostics published on open"));
    }
    let lines: Vec<&str> = src.split('\n').collect();
    for diagnostic in published
        .iter()
        .flat_map(|m| m.at(&["params", "diagnostics"]).as_array().unwrap_or(&[]))
    {
        for end in ["start", "end"] {
            let at = diagnostic.at(&["range", end]);
            let (line, col) = (at.at(&["line"]).as_u32(), at.at(&["character"]).as_u32());
            let fits = match (line, col) {
                (Some(line), Some(col)) => lines
                    .get(line as usize)
                    .is_some_and(|text| col as usize <= text.encode_utf16().count()),
                _ => false,
            };
            if !fits {
                failures.push(format!(
                    "{name}: diagnostic {end} outside the text: {diagnostic}"
                ));
            }
        }
    }
    let (names, dots) = sites(src);
    let mut stores = false;
    let asks = names
        .iter()
        .flat_map(|&at| [("textDocument/hover", at), ("textDocument/definition", at)])
        .chain(dots.iter().map(|&at| ("textDocument/completion", at)));
    for (method, at) in asks {
        *id += 1;
        let params = obj([("textDocument", doc()), ("position", position(src, at))]);
        let reply: String = server
            .handle(&message(Some(*id), method, params))
            .iter()
            .map(Value::to_string)
            .collect();
        if reply.is_empty()
            || reply.contains("internal error in cpl")
            || reply.contains("\"error\"")
        {
            failures.push(format!("{name}: {method} at byte {at}: {reply:.200}"));
        }
        stores |= method == "textDocument/hover" && reply.contains("storage:");
        let word: String = src[at..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        // `1_000` and `2e5` are numbers, not names.
        let in_number = src[..at].ends_with(|c: char| c.is_ascii_digit());
        let a_name = !in_number
            && !cpl_syntax::KEYWORDS.contains(&word.as_str())
            && !cpl_syntax::LITERAL_WORDS.contains(&word.as_str());
        if method == "textDocument/definition" && a_name {
            defs.asked += 1;
            if reply.contains("\"uri\"") {
                defs.resolved += 1;
            } else {
                let line_start = src[..at].rfind('\n').map_or(0, |i| i + 1);
                let line_end = src[at..].find('\n').map_or(src.len(), |i| at + i);
                let line = format!("{name}: {}", src[line_start..line_end].trim());
                defs.first.entry(word.clone()).or_insert(line);
                *defs.misses.entry(word).or_default() += 1;
            }
        }
    }
    server.handle(&message(
        None,
        "textDocument/didClose",
        obj([("textDocument", doc())]),
    ));
    stores
}

fn server() -> Server {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    server
}

#[test]
fn every_hill_climb_port_gets_hover_definition_and_completion() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("corpus/hillclimb.json")).unwrap();
    let list = cpl_json::parse(&list).unwrap();
    let ids: Vec<&str> = list
        .at(&["problems"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| p.as_str())
        .collect();
    assert_eq!(ids.len(), 100);

    let mut server = server();
    let mut id = 1;
    let mut failures = Vec::new();
    let mut storage = 0;
    let mut defs = Definitions::default();
    for pid in &ids {
        let src =
            std::fs::read_to_string(root.join(format!("corpus/problems/{pid}/spec.cp"))).unwrap();
        storage += usize::from(walk(
            &mut server,
            &mut id,
            &format!("{pid}.cp"),
            &src,
            &mut failures,
            &mut defs,
        ));
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
    let mut misses: Vec<_> = defs.misses.iter().collect();
    misses.sort_by_key(|(w, n)| (std::cmp::Reverse(**n), (*w).clone()));
    for (word, n) in misses.iter().take(40) {
        let first = defs.first.get(*word).map_or("", String::as_str);
        println!("no definition {n:4}  {word}  (first: {first:.100})");
    }
    let permille = defs.resolved * 1000 / defs.asked.max(1);
    println!(
        "definition: {}/{} first uses of a name = {}.{}%",
        defs.resolved,
        defs.asked,
        permille / 10,
        permille % 10
    );
    // In tenths of a percent; raise it as definition reaches more names.
    assert!(permille >= 645, "definition fell to {permille} permille");
    assert!(
        storage >= 88,
        "only {storage} of 100 ports show storage on some hover"
    );
}

fn cp_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

/// Every corpus program (`spec.cp`, `port.cp`, `dream.cp`) and every
/// conformance case, walked as the hill-climb ports are; broken and rejected
/// programs included.
#[test]
fn every_corpus_program_and_conformance_case_is_served() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    cp_files(&root.join("corpus/problems"), &mut files);
    files.extend(corpus_programs::v02_snapshot_ports(&root));
    let corpus = files.len();
    cp_files(&root.join("spec/conformance"), &mut files);
    assert!(
        corpus >= 1000 && files.len() - corpus >= 500,
        "{corpus} corpus files, {} cases",
        files.len() - corpus
    );
    files.sort();

    let threads = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .min(8);
    let chunk = files.len().div_ceil(threads);
    let failures: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = files
            .chunks(chunk)
            .map(|files| {
                let root = &root;
                scope.spawn(move || {
                    let mut server = server();
                    let mut id = 1;
                    let mut failures = Vec::new();
                    for path in files {
                        let src = std::fs::read_to_string(path).unwrap();
                        let name = path
                            .strip_prefix(root)
                            .unwrap()
                            .to_string_lossy()
                            .replace('/', "_");
                        walk(
                            &mut server,
                            &mut id,
                            &name,
                            &src,
                            &mut failures,
                            &mut Definitions::default(),
                        );
                    }
                    failures
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    });
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
