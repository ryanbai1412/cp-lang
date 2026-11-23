//! A quick fix keeps what the program does: every runnable program in the
//! repository (lint records' `before.cp`, the conformance cases and the
//! `tests/programs` and `tests/parity` programs, with the `.in` beside
//! each) runs to the same exit status and output after `cpl fix`.
//!
//! Only programs that run cleanly before the fix are held to this: a fix
//! for a checker error is meant to change what happens.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SHARDS: usize = 8;

/// Programs whose fixes still change behaviour; each is a lint bug, and a
/// file leaves this list when its fixes keep its output (the test says so).
const KNOWN: &[&str] = &[
    "crates/cpl-cli/tests/programs/dream_gaps.cp",
    "crates/cpl-cli/tests/programs/reflect_fields.cp",
    "crates/cpl-cli/tests/programs/reflect_variants.cp",
    "crates/cpl-cli/tests/programs/repeat_loop.cp",
    "crates/cpl-cli/tests/programs/variadic_pack.cp",
    "spec/conformance/core-01/range-forms.cp",
    "spec/conformance/core-01/shared-annotation.cp",
    "spec/conformance/core-05/closure-captures-at-call.cp",
    "spec/conformance/core-05/exercise-17-closure-borrows-per-call.cp",
    "spec/conformance/core-05/local-var-captured-by-reference.cp",
    "spec/conformance/core-06/rb1-q06-closure-mutates-capture.cp",
    "spec/conformance/core-06/two-closures-share-var.cp",
    "spec/conformance/std-tree/rooted-single-preorder.cp",
];

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
    assert!(out.len() > 1000, "found only {} programs", out.len());
    out
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

/// What fixing `rel` changed, when it changed what the program does.
fn changed(rel: &str, scratch: &Path) -> Option<String> {
    let src = root().join(rel);
    let text = std::fs::read_to_string(&src).unwrap();
    let input = std::fs::read(src.with_extension("in")).unwrap_or_default();
    std::fs::create_dir_all(scratch).unwrap();
    let file = scratch.join(src.file_name().unwrap());
    std::fs::write(&file, &text).unwrap();
    let before = run(&file, &input);
    let fixed = match before {
        Some((0, _)) => {
            cpl_cli::run(&["fix", file.to_str().unwrap()]);
            std::fs::read_to_string(&file).unwrap()
        }
        _ => text.clone(),
    };
    let after = if fixed == text {
        before.clone()
    } else {
        run(&file, &input)
    };
    let _ = std::fs::remove_dir_all(scratch);
    let show = |r: &Option<(i32, Vec<u8>)>| match r {
        Some((code, _)) => format!("exit {code}"),
        None => "no exit in 20 s".to_string(),
    };
    (after != before).then(|| {
        format!(
            "{rel}: {} -> {}\n--- before\n{text}\n--- fixed\n{fixed}\n--- output after\n{}",
            show(&before),
            show(&after),
            String::from_utf8_lossy(after.as_ref().map_or(&[][..], |a| &a.1))
        )
    })
}

fn shard(n: usize) {
    let scratch = std::env::temp_dir().join(format!("cpl-fix-behavior-{}-{n}", std::process::id()));
    let mut wrong = Vec::new();
    let mut fixed_known = Vec::new();
    for (i, rel) in programs().iter().enumerate() {
        if i % SHARDS != n {
            continue;
        }
        let diff = changed(rel, &scratch.join(i.to_string()));
        match (diff, KNOWN.contains(&rel.as_str())) {
            (Some(d), false) => wrong.push(d),
            (None, true) => fixed_known.push(rel.clone()),
            _ => {}
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        wrong.is_empty(),
        "`cpl fix` changed what these programs do:\n\n{}",
        wrong.join("\n\n")
    );
    assert!(
        fixed_known.is_empty(),
        "fixing these now keeps their output; take them off KNOWN: {fixed_known:?}"
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

#[test]
fn known_programs_exist() {
    let all = programs();
    for k in KNOWN {
        assert!(all.iter().any(|p| p == k), "{k} is not a program here");
    }
}
