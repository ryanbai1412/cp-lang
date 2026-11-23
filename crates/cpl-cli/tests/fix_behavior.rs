//! A quick fix keeps what the program does: every runnable program in the
//! repository (lint records' `before.cp`, the conformance cases and the
//! `tests/programs` and `tests/parity` programs, with the `.in` beside
//! each, and the corpus ports on their three smallest tests) runs to the
//! same exit status and output after `cpl fix`.
//!
//! Only programs that run cleanly before the fix are held to this: a fix
//! for a checker error is meant to change what happens.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SHARDS: usize = 8;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn programs() -> Vec<String> {
    fn walk(dir: &Path, nested: bool, keep: &dyn Fn(&Path) -> bool, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.unwrap().path();
            if path.is_dir() {
                if nested {
                    walk(&path, nested, keep, out);
                }
            } else if keep(&path) {
                out.push(path);
            }
        }
    }
    let cp = |p: &Path| p.extension().is_some_and(|e| e == "cp");
    let before = |p: &Path| p.file_name().is_some_and(|n| n == "before.cp");
    let r = root();
    let mut paths = Vec::new();
    walk(&r.join("spec/lints"), true, &before, &mut paths);
    walk(&r.join("spec/conformance"), true, &cp, &mut paths);
    walk(
        &r.join("crates/cpl-cli/tests/programs"),
        false,
        &cp,
        &mut paths,
    );
    walk(
        &r.join("crates/cpl-cli/tests/parity"),
        false,
        &cp,
        &mut paths,
    );
    // A port runs on its tests; one with none has nothing to compare.
    for dir in std::fs::read_dir(r.join("corpus/problems")).unwrap() {
        let dir = dir.unwrap().path();
        if dir.join("port.cp").exists() && !port_inputs(&dir).is_empty() {
            paths.push(dir.join("port.cp"));
        }
    }
    let mut out: Vec<String> = paths
        .iter()
        .map(|p| {
            p.strip_prefix(&r)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    out.sort();
    assert!(out.len() > 1300, "found only {} programs", out.len());
    out
}

/// The three smallest `tests/*.in` of a port's directory.
fn port_inputs(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir.join("tests")) else {
        return Vec::new();
    };
    let mut ins: Vec<PathBuf> = entries
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "in"))
        .collect();
    // The smallest run fastest in a debug build and still exercise the fix.
    ins.sort_by_key(|p| (std::fs::metadata(p).unwrap().len(), p.clone()));
    ins.truncate(3);
    ins
}

/// What `rel` runs on: a port its tests, any other program the `.in` beside it.
fn inputs(src: &Path) -> Vec<Vec<u8>> {
    let read = |p: &Path| std::fs::read(p).unwrap();
    if src.file_name().is_some_and(|n| n == "port.cp") {
        return port_inputs(src.parent().unwrap())
            .iter()
            .map(|p| read(p))
            .collect();
    }
    vec![std::fs::read(src.with_extension("in")).unwrap_or_default()]
}

/// `(exit status, stdout)` of `cpl run path` on `input`, `None` past the
/// time limit (a fix must not make a program loop either).
fn run(path: &Path, input: &[u8]) -> Option<(i32, Vec<u8>)> {
    use std::io::{Read, Write};
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", path.to_str().unwrap()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        stdout.read_to_end(&mut out).map(|_| out)
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break Some(status);
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let out = reader.join().unwrap().unwrap_or_default();
    status.map(|s| (s.code().unwrap_or(-1), out))
}

/// What fixing `rel` from `text` to `fixed` changed, when it changed what
/// the program does; `file` is the scratch copy to run.
fn changed(rel: &str, file: &Path, text: &str, fixed: &str) -> Option<String> {
    let inputs = inputs(&root().join(rel));
    std::fs::write(file, text).unwrap();
    let mut before = Vec::new();
    for input in &inputs {
        let r = run(file, input);
        let clean = matches!(r, Some((0, _)));
        before.push(r);
        if !clean {
            return None;
        }
    }
    std::fs::write(file, fixed).unwrap();
    let after: Vec<_> = inputs.iter().map(|i| run(file, i)).collect();
    let show = |r: &Option<(i32, Vec<u8>)>| match r {
        Some((code, _)) => format!("exit {code}"),
        None => "no exit in 20 s".to_string(),
    };
    let (b, a) = before.iter().zip(&after).find(|(b, a)| b != a)?;
    Some(format!(
        "{rel}: {} -> {}\n--- before\n{text}\n--- fixed\n{fixed}\n--- output after\n{}",
        show(b),
        show(a),
        String::from_utf8_lossy(a.as_ref().map_or(&[][..], |a| &a.1))
    ))
}

fn shard(n: usize) {
    let scratch = std::env::temp_dir().join(format!("cpl-fix-behavior-{}-{n}", std::process::id()));
    let mut copies = Vec::new();
    for (i, rel) in programs().into_iter().enumerate() {
        if i % SHARDS != n {
            continue;
        }
        let src = root().join(&rel);
        let dir = scratch.join(i.to_string());
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join(src.file_name().unwrap());
        let text = std::fs::read_to_string(&src).unwrap();
        std::fs::write(&file, &text).unwrap();
        copies.push((rel, file, text));
    }
    // One `cpl fix` over the shard checks the stdlib once; only the
    // programs it edits are run.
    let mut args = vec!["fix".to_string()];
    args.extend(
        copies
            .iter()
            .map(|(_, file, _)| file.to_string_lossy().into_owned()),
    );
    cpl_cli::run(&args);
    let mut wrong = Vec::new();
    for (rel, file, text) in &copies {
        let fixed = std::fs::read_to_string(file).unwrap();
        if fixed != *text {
            wrong.extend(changed(rel, file, text, &fixed));
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        wrong.is_empty(),
        "`cpl fix` changed what these programs do:\n\n{}",
        wrong.join("\n\n")
    );
}

#[test]
fn fixes_keep_behaviour_0() {
    shard(0);
}
#[test]
fn fixes_keep_behaviour_1() {
    shard(1);
}
#[test]
fn fixes_keep_behaviour_2() {
    shard(2);
}
#[test]
fn fixes_keep_behaviour_3() {
    shard(3);
}
#[test]
fn fixes_keep_behaviour_4() {
    shard(4);
}
#[test]
fn fixes_keep_behaviour_5() {
    shard(5);
}
#[test]
fn fixes_keep_behaviour_6() {
    shard(6);
}
#[test]
fn fixes_keep_behaviour_7() {
    shard(7);
}
