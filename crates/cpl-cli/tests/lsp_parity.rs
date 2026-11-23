//! The editor reports exactly what the command line reports on every v0.3
//! file: the spec's conformance cases, the corpus's `spec.cp` ports and the
//! examples. The editor shows what `cpl check` and `cpl lint` both report.

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
    let ports = std::fs::read_dir(root.join("corpus/problems")).unwrap();
    files.extend(
        ports
            .flatten()
            .map(|p| p.path().join("spec.cp"))
            .filter(|p| p.is_file()),
    );
    files.sort();
    files
}

/// What `cpl <command>` renders for `files`, by file.
fn rendered(command: &str, files: &[PathBuf], out: &mut BTreeMap<String, Vec<Diag>>) {
    let run = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg(command)
        .args(files)
        .output()
        .expect("cpl runs");
    let text =
        String::from_utf8_lossy(&run.stderr).into_owned() + &String::from_utf8_lossy(&run.stdout);
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

#[test]
fn the_editor_reports_what_check_and_lint_report_on_every_v03_file() {
    let files = v03_files();
    assert!(files.len() > 1000, "only {} v0.3 files found", files.len());
    let mut want = BTreeMap::new();
    rendered("check", &files, &mut want);
    rendered("lint", &files, &mut want);
    let chunks: Vec<Vec<PathBuf>> = files
        .chunks(files.len().div_ceil(8))
        .map(<[_]>::to_vec)
        .collect();
    let workers: Vec<_> = chunks
        .into_iter()
        .map(|c| std::thread::spawn(move || published(&c)))
        .collect();
    let mut got = BTreeMap::new();
    for worker in workers {
        got.extend(worker.join().unwrap());
    }
    let reported: usize = want.values().map(Vec::len).sum();
    assert!(reported > 500, "only {reported} diagnostics to compare");
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

#[test]
fn check_json_reports_what_check_prints_on_every_v03_file() {
    let files = v03_files();
    let mut want = BTreeMap::new();
    rendered("check", &files, &mut want);
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .args(&files)
        .arg("--json")
        .output()
        .expect("cpl runs");
    let json = cpl_json::parse(&String::from_utf8_lossy(&out.stdout)).expect("JSON");
    let mut got: BTreeMap<String, Vec<Diag>> = BTreeMap::new();
    for file in json.at(&["files"]).as_array().unwrap() {
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
    assert!(want.values().map(Vec::len).sum::<usize>() > 100);
}
