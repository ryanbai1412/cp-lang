//! Every fix a diagnostic carries is a code action, and fix-all is `cpl
//! fix`: over the conformance cases, examples, corpus `spec.cp` ports, lint
//! records and the fix and mistake cases, each fix `cpl check --json` or
//! `cpl lint --json` reports is offered by the editor as one quick fix on
//! that diagnostic, with the same title and the same rewritten text, and
//! the editor's `source.fixAll` leaves each file byte for byte as `cpl fix`
//! does.
//!
//! Each file is copied, with the `cp.toml` files above it, to a scratch
//! tree first, so `cpl fix` can rewrite it and every side reads the same
//! configuration.
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r10-one-entry-point-per-job`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use cpl_json::{obj, Value};
use cpl_lsp::Server;

const SHARDS: usize = 8;

/// A fix as both sides can name it: its title and the text after applying it.
type Fix = (String, String);

/// One diagnostic with fixes: code, byte span, first message line, and its fixes.
struct Fixable {
    code: String,
    start: usize,
    end: usize,
    message: String,
    fixes: Vec<Fix>,
}

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn cp_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            cp_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "cp") {
            out.push(path);
        }
    }
}

/// The shard's files, repository-relative.
fn shard_of(shard: usize) -> Vec<PathBuf> {
    let root = repo();
    let mut files = Vec::new();
    for dir in [
        "spec/conformance",
        "spec/lints",
        "examples",
        "crates/cpl-cli/tests/fixes",
        "crates/cpl-cli/tests/mistakes",
    ] {
        cp_files(&root.join(dir), &mut files);
    }
    files.extend(cpl_corpus::files(&root, "spec.cp"));
    let mut files: Vec<PathBuf> = files
        .iter()
        .map(|f| f.strip_prefix(&root).unwrap().to_path_buf())
        .collect();
    files.sort();
    files.dedup();
    assert!(files.len() > 1000, "only {} files found", files.len());
    files.into_iter().skip(shard).step_by(SHARDS).collect()
}

/// `files` copied under `scratch` at the same relative paths, with each
/// `cp.toml` between them and the repository root.
fn mirrored(files: &[PathBuf], scratch: &Path) -> Vec<PathBuf> {
    let root = repo();
    let _ = std::fs::remove_dir_all(scratch);
    files
        .iter()
        .map(|rel| {
            let copy = scratch.join(rel);
            std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
            std::fs::copy(root.join(rel), &copy).unwrap();
            for dir in rel.ancestors().skip(1) {
                let config = root.join(dir).join("cp.toml");
                if config.is_file() {
                    std::fs::copy(&config, scratch.join(dir).join("cp.toml")).unwrap();
                }
            }
            copy
        })
        .collect()
}

/// `f` over `items` split across one thread per core, results in order.
fn parallel<T: Sync, R: Send>(items: &[T], f: impl Fn(&[T]) -> Vec<R> + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|s| {
        let f = &f;
        let workers: Vec<_> = items
            .chunks(items.len().div_ceil(threads).max(1))
            .map(|chunk| s.spawn(move || f(chunk)))
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    })
}

fn applied(text: &str, edits: Vec<(usize, usize, String)>) -> String {
    let edits = edits
        .into_iter()
        .map(|(start, end, new)| (cpl_diag::Span::new(start as u32, end as u32), new))
        .collect();
    cpl_diag::splice(text, edits).expect("a fix's edits apply")
}

/// The diagnostics with fixes `cpl <command> --json` reports for
/// `files`; a file `cpl` fails on without its JSON is in `crashed`.
fn reported(
    command: &str,
    files: &[PathBuf],
    crashed: &mut Vec<String>,
) -> BTreeMap<String, Vec<Fixable>> {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg(command)
        .args(files)
        .arg("--json")
        .output()
        .expect("cpl runs");
    if out.status.code().is_none_or(|code| code > 1) {
        if let [file] = files {
            let why = String::from_utf8_lossy(&out.stderr);
            crashed.push(format!(
                "{}: `cpl {command}` failed: {}",
                file.display(),
                why.trim()
            ));
            return BTreeMap::new();
        }
        return files
            .iter()
            .flat_map(|file| reported(command, std::slice::from_ref(file), crashed))
            .collect();
    }
    let json = cpl_json::parse(&String::from_utf8_lossy(&out.stdout)).expect("JSON");
    let mut found = BTreeMap::new();
    for file in json.at(&["files"]).as_array().unwrap() {
        let path = file.at(&["path"]).as_str().unwrap();
        let text = std::fs::read_to_string(path).unwrap();
        let offset = |v: &Value| v.at(&["offset"]).as_usize().unwrap();
        let fixable = file
            .at(&["diagnostics"])
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| !d.at(&["fixes"]).as_array().unwrap().is_empty())
            .map(|d| Fixable {
                code: d.at(&["code"]).as_str().unwrap().to_string(),
                start: offset(d.at(&["start"])),
                end: offset(d.at(&["end"])),
                message: first_line(d.at(&["message"])),
                fixes: d
                    .at(&["fixes"])
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|fix| {
                        let edits = fix.at(&["edits"]).as_array().unwrap().iter().map(|e| {
                            let new = e.at(&["text"]).as_str().unwrap().to_string();
                            (offset(e.at(&["start"])), offset(e.at(&["end"])), new)
                        });
                        let title = fix.at(&["title"]).as_str().unwrap().to_string();
                        (title, applied(&text, edits.collect()))
                    })
                    .collect(),
            });
        found.insert(path.to_string(), fixable.collect());
    }
    found
}

fn first_line(message: &Value) -> String {
    message
        .as_str()
        .unwrap()
        .lines()
        .next()
        .unwrap_or("")
        .to_string()
}

fn position(text: &str, offset: usize) -> Value {
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = text[..offset].matches('\n').count();
    let character = text[line_start..offset].encode_utf16().count();
    obj([("line", line.into()), ("character", character.into())])
}

fn offset_of(text: &str, position: &Value) -> usize {
    let line = position.at(&["line"]).as_usize().unwrap();
    let character = position.at(&["character"]).as_usize().unwrap();
    let line_start = match line {
        0 => 0,
        n => text
            .match_indices('\n')
            .nth(n - 1)
            .map_or(text.len(), |(i, _)| i + 1),
    };
    let mut units = 0;
    for (i, c) in text[line_start..].char_indices() {
        if units >= character || c == '\n' {
            return line_start + i;
        }
        units += c.len_utf16();
    }
    text.len()
}

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

/// The code actions of kind `only` the server offers on `start..end`.
fn actions(
    server: &mut Server,
    uri: &str,
    text: &str,
    span: (usize, usize),
    only: &str,
) -> Vec<Value> {
    let range = obj([
        ("start", position(text, span.0)),
        ("end", position(text, span.1)),
    ]);
    let reply = server.handle(&message(
        Some(2),
        "textDocument/codeAction",
        obj([
            ("textDocument", obj([("uri", uri.into())])),
            ("range", range),
            (
                "context",
                obj([
                    ("diagnostics", Value::Array(Vec::new())),
                    ("only", Value::Array(vec![only.into()])),
                ]),
            ),
        ]),
    ));
    reply
        .iter()
        .find_map(|r| r.at(&["result"]).as_array())
        .unwrap_or(&[])
        .iter()
        .filter(|a| {
            a.at(&["kind"])
                .as_str()
                .is_some_and(|k| k.starts_with(only))
        })
        .cloned()
        .collect()
}

/// `text` after an action's edit to `uri`.
fn after(action: &Value, uri: &str, text: &str) -> String {
    let edits = action
        .at(&["edit", "changes", uri])
        .as_array()
        .unwrap_or(&[]);
    let edits = edits.iter().map(|e| {
        let new = e.at(&["newText"]).as_str().unwrap().to_string();
        let range = e.at(&["range"]);
        (
            offset_of(text, range.at(&["start"])),
            offset_of(text, range.at(&["end"])),
            new,
        )
    });
    applied(text, edits.collect())
}

/// What the editor offers for each file: its problems with the quick
/// fixes, and the text after its fix-all.
fn editor(
    files: &[PathBuf],
    want: &BTreeMap<String, Vec<Fixable>>,
) -> Vec<(String, Vec<String>, String)> {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut out = Vec::new();
    for file in files {
        let path = file.to_str().unwrap();
        let text = std::fs::read_to_string(file).unwrap();
        let uri = format!("file://{path}");
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", text.as_str().into()),
                ]),
            )]),
        ));
        let mut problems = Vec::new();
        for d in want.get(path).map_or(&[][..], Vec::as_slice) {
            let mut got: Vec<Fix> = actions(&mut server, &uri, &text, (d.start, d.end), "quickfix")
                .iter()
                .filter(|a| {
                    a.at(&["diagnostics"])
                        .as_array()
                        .unwrap_or(&[])
                        .iter()
                        .any(|on| {
                            on.at(&["code"]).as_str() == Some(&d.code)
                                && first_line(on.at(&["message"])) == d.message
                                && offset_of(&text, on.at(&["range", "start"])) == d.start
                                && offset_of(&text, on.at(&["range", "end"])) == d.end
                        })
                })
                .map(|a| {
                    (
                        a.at(&["title"]).as_str().unwrap().to_string(),
                        after(a, &uri, &text),
                    )
                })
                .collect();
            let mut fixes = d.fixes.clone();
            got.sort();
            fixes.sort();
            if got != fixes {
                let titles = |fs: &[Fix]| fs.iter().map(|f| f.0.clone()).collect::<Vec<_>>();
                let lines = cpl_diag::LineIndex::new(&text);
                let at = lines.line_col(d.start as u32);
                problems.push(format!(
                    "{path}:{}:{} {} `{}`: fixes {:?}, code actions {:?}{}",
                    at.line + 1,
                    at.col + 1,
                    d.code,
                    d.message,
                    titles(&fixes),
                    titles(&got),
                    if titles(&fixes) == titles(&got) {
                        " (different edits)"
                    } else {
                        ""
                    }
                ));
            }
        }
        let fixed = actions(&mut server, &uri, &text, (0, 0), "source.fixAll")
            .first()
            .map_or(text.clone(), |a| after(a, &uri, &text));
        server.handle(&message(
            None,
            "textDocument/didClose",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        out.push((path.to_string(), problems, fixed));
    }
    out
}

/// `cpl fix` over `files`, one file at a time after a run `cpl` fails on;
/// each file it fails on is in `crashed`.
fn fix(files: &[PathBuf], crashed: &mut Vec<String>) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("fix")
        .args(files)
        .output()
        .expect("cpl fix runs");
    if out.status.code().is_none_or(|code| code > 1) {
        match files {
            [file] => crashed.push(format!(
                "{}: `cpl fix` failed: {}",
                file.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            _ => files
                .iter()
                .for_each(|file| fix(std::slice::from_ref(file), crashed)),
        }
    }
}

fn fix_actions(shard: usize) {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("fix-actions-{shard}"));
    let files = mirrored(&shard_of(shard), &scratch);
    let mut want = BTreeMap::<String, Vec<Fixable>>::new();
    let mut problems = Vec::new();
    for command in ["check", "lint"] {
        let found = parallel(&files, |chunk| {
            let mut crashed = Vec::new();
            vec![(reported(command, chunk, &mut crashed), crashed)]
        });
        for (found, crashed) in found {
            problems.extend(crashed);
            for (path, fixable) in found {
                let seen = want.entry(path).or_default();
                for d in fixable {
                    let key = |s: &Fixable| (s.start, s.end, s.code.clone(), s.message.clone());
                    if !seen.iter().any(|s| key(s) == key(&d)) {
                        seen.push(d);
                    }
                }
            }
        }
    }
    let fixes: usize = want.values().flatten().map(|d| d.fixes.len()).sum();
    assert!(fixes > 40 / SHARDS, "only {fixes} fixes reported");
    // A file `cpl` crashed on would crash the in-process server too; it
    // already fails the test above.
    let files: Vec<PathBuf> = files
        .into_iter()
        .filter(|f| {
            !problems
                .iter()
                .any(|p| p.starts_with(&format!("{}: ", f.display())))
        })
        .collect();
    let edited = parallel(&files, |chunk| editor(chunk, &want));
    problems.extend(parallel(&files, |chunk| {
        let mut crashed = Vec::new();
        fix(chunk, &mut crashed);
        crashed
    }));
    for (path, offered, by_editor) in edited {
        problems.extend(offered);
        let by_cli = std::fs::read_to_string(&path).unwrap();
        if by_editor != by_cli {
            problems.push(format!("{path}: fix-all differs from `cpl fix`"));
        }
    }
    let shown = problems
        .iter()
        .map(|p| p.replace(&format!("{}/", scratch.display()), ""));
    assert!(
        problems.is_empty(),
        "{}",
        shown.collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_0() {
    fix_actions(0);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_1() {
    fix_actions(1);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_2() {
    fix_actions(2);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_3() {
    fix_actions(3);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_4() {
    fix_actions(4);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_5() {
    fix_actions(5);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_6() {
    fix_actions(6);
}

#[test]
fn every_fix_is_a_code_action_and_fix_all_is_cpl_fix_7() {
    fix_actions(7);
}
