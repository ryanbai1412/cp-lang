//! The editor reports exactly what the command line reports on every v0.3
//! file: the spec's conformance cases, the corpus's `spec.cp` ports and the
//! examples. The editor shows what `cpl check` and `cpl lint` both report.
//!
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.
//! Enforces `handbook/guidelines/commands-checklist.md#r10-one-entry-point-per-job`.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// One diagnostic as both sides can name it: severity, code, line and
/// column (1-based), and the message's first line.
type Diag = (String, String, usize, usize, String);

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

fn v03_files() -> Vec<PathBuf> {
    let root = repo();
    let mut files = Vec::new();
    cp_files(&root.join("spec/conformance"), &mut files);
    cp_files(&root.join("examples"), &mut files);
    files.extend(cpl_corpus::files(&root, "spec.cp"));
    files.sort();
    files
}

/// What `cpl <command>` renders for `files`, by file, run in-process so
/// each thread's database checks shared modules once.
fn rendered(command: &str, files: &[PathBuf], out: &mut BTreeMap<String, Vec<Diag>>) {
    let mut args = vec![command];
    args.extend(files.iter().map(|f| f.to_str().unwrap()));
    let run = cpl_cli::run(&args);
    let text = run.stderr + &String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some((severity, rest)) = ["error", "warning", "note"]
            .iter()
            .find_map(|s| line.strip_prefix(&format!("{s}[")).map(|rest| (*s, rest)))
        else {
            continue;
        };
        let Some((code, message)) = rest.split_once("]: ") else {
            continue;
        };
        let Some(at) = lines
            .get(i + 1)
            .and_then(|l| l.trim_start().strip_prefix("--> "))
        else {
            continue;
        };
        let mut parts = at.rsplitn(3, ':');
        let (col, line_no, path) = (
            parts.next().unwrap(),
            parts.next().unwrap(),
            parts.next().unwrap(),
        );
        out.entry(path.to_string()).or_default().push((
            severity.into(),
            code.into(),
            line_no.parse().unwrap(),
            col.parse().unwrap(),
            message.into(),
        ));
    }
}

/// `files` dealt round-robin into `n` parts, longest first, so no part is
/// one slow directory or the long files.
fn dealt(files: &[PathBuf], n: usize) -> Vec<Vec<PathBuf>> {
    let n = n.min(files.len()).max(1);
    let mut files = files.to_vec();
    files.sort_by_key(|f| std::cmp::Reverse(std::fs::metadata(f).map_or(0, |m| m.len())));
    (0..n)
        .map(|i| files.iter().skip(i).step_by(n).cloned().collect())
        .collect()
}

/// `f` on each of `parts`, one thread each, with a deep stack as `cpl`'s
/// own main thread has.
fn on_threads<R: Send>(parts: Vec<Vec<PathBuf>>, f: impl Fn(&[PathBuf]) -> R + Sync) -> Vec<R> {
    std::thread::scope(|s| {
        let f = &f;
        let workers: Vec<_> = parts
            .iter()
            .map(|part| {
                std::thread::Builder::new()
                    .stack_size(64 << 20)
                    .spawn_scoped(s, move || f(part))
                    .unwrap()
            })
            .collect();
        workers.into_iter().map(|w| w.join().unwrap()).collect()
    })
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(4, |n| n.get())
}

/// `rendered` for each of `commands` on `files`, split across one thread
/// per core; one thread runs every command on its files, so they share its
/// database.
fn rendered_parallel(commands: &[&str], files: &[PathBuf]) -> Vec<BTreeMap<String, Vec<Diag>>> {
    let parts = on_threads(dealt(files, threads()), |part| {
        commands
            .iter()
            .map(|command| {
                let mut out = BTreeMap::new();
                rendered(command, part, &mut out);
                out
            })
            .collect::<Vec<_>>()
    });
    let mut all = vec![BTreeMap::new(); commands.len()];
    for part in parts {
        for (all, out) in all.iter_mut().zip(part) {
            all.extend(out);
        }
    }
    all
}

fn frame(body: &str) -> Vec<u8> {
    format!("Content-Length: {}\r\n\r\n{body}", body.len()).into_bytes()
}

/// What `cpl lsp`, started with no edition set, publishes for `files`.
fn published(files: &[PathBuf]) -> BTreeMap<String, Vec<Diag>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("cpl lsp starts");
    let mut stdin = child.stdin.take().unwrap();
    let mut session = frame(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    for file in files {
        let text = std::fs::read_to_string(file).unwrap();
        let open = format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{}","languageId":"cp","version":1,"text":{}}}}}}}"#,
            file.display(),
            cpl_json::to_string(&text.as_str().into())
        );
        session.extend(frame(&open));
    }
    session.extend(frame(
        r#"{"jsonrpc":"2.0","id":2,"method":"shutdown","params":null}"#,
    ));
    session.extend(frame(r#"{"jsonrpc":"2.0","method":"exit","params":null}"#));
    let writer = std::thread::spawn(move || stdin.write_all(&session));
    let out = child.wait_with_output().expect("cpl lsp exits");
    writer.join().unwrap().unwrap();
    let mut reader = std::io::BufReader::new(&out.stdout[..]);
    let mut found = BTreeMap::new();
    while let Some(body) = cpl_lsp::read_message(&mut reader).unwrap() {
        let message = cpl_json::parse(&body).unwrap();
        if message.at(&["method"]).as_str() != Some("textDocument/publishDiagnostics") {
            continue;
        }
        let params = message.at(&["params"]);
        let uri = params.at(&["uri"]).as_str().unwrap();
        let diags = params
            .at(&["diagnostics"])
            .as_array()
            .unwrap()
            .iter()
            .map(|d| {
                let severity =
                    ["", "error", "warning", "note"][d.at(&["severity"]).as_usize().unwrap()];
                let message = d.at(&["message"]).as_str().unwrap();
                (
                    severity.to_string(),
                    d.at(&["code"]).as_str().unwrap().to_string(),
                    d.at(&["range", "start", "line"]).as_usize().unwrap() + 1,
                    d.at(&["range", "start", "character"]).as_usize().unwrap() + 1,
                    message.lines().next().unwrap_or("").to_string(),
                )
            });
        found.insert(
            uri.strip_prefix("file://").unwrap().to_string(),
            diags.collect(),
        );
    }
    found
}

/// The v0.3 files whose index in sorted order is `shard` modulo `SHARDS`,
/// so each test stays short and together they cover every file.
const SHARDS: usize = 8;

fn shard_of(shard: usize) -> Vec<PathBuf> {
    let files = v03_files();
    assert!(files.len() > 1000, "only {} v0.3 files found", files.len());
    files.into_iter().skip(shard).step_by(SHARDS).collect()
}

fn editor_parity(shard: usize) {
    let files = shard_of(shard);
    // `cpl lint` also shows what `cpl check` reported (the checker's L06xx
    // lints and the errors lints can't see past); the editor shows each
    // once. Lint shows exactly the errors `cpl check` shows.
    let [mut want, lints] =
        <[_; 2]>::try_from(rendered_parallel(&["check", "lint"], &files)).unwrap();
    let mut strays = Vec::new();
    for (file, diags) in lints {
        let checked: &mut Vec<Diag> = want.entry(file.clone()).or_default();
        let errors = |ds: &[Diag]| -> Vec<Diag> {
            ds.iter()
                .filter(|d| d.1.starts_with('E'))
                .cloned()
                .collect()
        };
        if errors(&diags) != errors(checked) {
            strays.push(format!(
                "{}:\n  cpl check: {:?}\n  cpl lint:  {:?}",
                file,
                errors(checked),
                errors(&diags)
            ));
        }
        let fresh: Vec<Diag> = diags.into_iter().filter(|d| !checked.contains(d)).collect();
        checked.extend(fresh);
    }
    assert!(
        strays.is_empty(),
        "{} files where `cpl lint` and `cpl check` show different errors:\n{}",
        strays.len(),
        strays.join("\n")
    );
    let mut got: BTreeMap<String, Vec<Diag>> = on_threads(dealt(&files, threads()), published)
        .into_iter()
        .flatten()
        .collect();
    let reported: usize = want.values().map(Vec::len).sum();
    assert!(
        reported > 500 / SHARDS,
        "only {reported} diagnostics to compare"
    );
    let mut differ = Vec::new();
    for file in &files {
        let key = file.display().to_string();
        let mut want = want.remove(&key).unwrap_or_default();
        let Some(mut got) = got.remove(&key) else {
            differ.push(format!("{key}: the editor published nothing"));
            continue;
        };
        want.sort();
        got.sort();
        if want != got {
            differ.push(format!(
                "{key}:\n  cpl check + lint: {want:?}\n  cpl lsp:          {got:?}"
            ));
        }
    }
    assert!(
        differ.is_empty(),
        "{} of {} files differ:\n{}",
        differ.len(),
        files.len(),
        differ.join("\n")
    );
}

fn json_parity(shard: usize) {
    let files = shard_of(shard);
    let mut want = BTreeMap::new();
    let outs = on_threads(dealt(&files, threads()), |part| {
        let mut text = BTreeMap::new();
        rendered("check", part, &mut text);
        let mut args = vec!["check"];
        args.extend(part.iter().map(|f| f.to_str().unwrap()));
        args.push("--json");
        let out = cpl_cli::run(&args);
        let json = cpl_json::parse(&String::from_utf8_lossy(&out.stdout)).expect("JSON");
        (text, json)
    });
    for (text, _) in &outs {
        want.extend(text.clone());
    }
    let mut got: BTreeMap<String, Vec<Diag>> = BTreeMap::new();
    for file in outs
        .iter()
        .flat_map(|(_, json)| json.at(&["files"]).as_array().unwrap())
    {
        let path = file.at(&["path"]).as_str().unwrap();
        for d in file.at(&["diagnostics"]).as_array().unwrap() {
            let s = |key: &str| d.at(&[key]).as_str().unwrap().to_string();
            let at = |key: &str| d.at(&["start", key]).as_i64().unwrap() as usize;
            let message = s("message").lines().next().unwrap_or("").to_string();
            got.entry(path.to_string()).or_default().push((
                s("severity"),
                s("code"),
                at("line"),
                at("col"),
                message,
            ));
        }
    }
    for diags in want.values_mut().chain(got.values_mut()) {
        diags.sort();
    }
    let differ: Vec<_> = want
        .keys()
        .chain(got.keys())
        .filter(|key| want.get(*key) != got.get(*key))
        .map(|key| {
            format!(
                "{key}:\n  text: {:?}\n  json: {:?}",
                want.get(key),
                got.get(key)
            )
        })
        .collect();
    assert!(differ.is_empty(), "{}", differ.join("\n"));
    assert!(want.values().map(Vec::len).sum::<usize>() > 100 / SHARDS);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_0() {
    editor_parity(0);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_1() {
    editor_parity(1);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_2() {
    editor_parity(2);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_3() {
    editor_parity(3);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_4() {
    editor_parity(4);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_5() {
    editor_parity(5);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_6() {
    editor_parity(6);
}

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file_7() {
    editor_parity(7);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_0() {
    json_parity(0);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_1() {
    json_parity(1);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_2() {
    json_parity(2);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_3() {
    json_parity(3);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_4() {
    json_parity(4);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_5() {
    json_parity(5);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_6() {
    json_parity(6);
}

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file_7() {
    json_parity(7);
}
