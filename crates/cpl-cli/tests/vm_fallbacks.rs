//! Tree-walker fallback ratchet: every `(mode, tag)` kind of statement or
//! expression the VM hands to the tree-walker, over the programs in
//! `tests/programs`, untraced and traced. `vm_fallbacks.txt` lists the kinds
//! and may only shrink, until it is empty and the tree-walker is deleted. A
//! new program that hits a listed kind adds no debt; a new kind fails.
//!
//! A fix that removes fallbacks rewrites the list with
//! `CPL_BLESS_VM_FALLBACKS=1 cargo test -p cpl-cli --test vm_fallbacks`;
//! blessing refuses to add a line to a non-empty list.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn tests_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests")
}

fn fallbacks(program: &Path, traced: bool) -> Vec<String> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cmd.arg("run").arg(program);
    if traced {
        // JSON on stdout, so no `<file>.trace.html` lands beside the program.
        cmd.args(["--trace", "--format", "json"]);
    }
    let input = std::fs::read(program.with_extension("in")).unwrap_or_default();
    let mut child = cmd
        .env("CPL_VM_STATS", "json")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cpl");
    child.stdin.take().unwrap().write_all(&input).ok();
    let out = child.wait_with_output().expect("wait cpl");
    let stderr = String::from_utf8_lossy(&out.stderr);
    let Some(line) = stderr
        .lines()
        .rev()
        .find(|l| l.starts_with('{') && l.contains("\"total_fallbacks\""))
    else {
        return vec!["no-stats".into()];
    };
    let top = line
        .split("\"top\":[")
        .nth(1)
        .and_then(|s| s.split("],\"loop_top\"").next())
        .unwrap_or("");
    top.split("[\"")
        .skip(1)
        .filter_map(|s| s.split("\",").next())
        .map(str::to_string)
        .collect()
}

fn measure() -> BTreeSet<String> {
    let mut programs: Vec<PathBuf> = std::fs::read_dir(tests_dir().join("programs"))
        .expect("read programs")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    programs.sort();
    let jobs: Vec<(PathBuf, bool)> = programs
        .iter()
        .flat_map(|p| [(p.clone(), false), (p.clone(), true)])
        .collect();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<Vec<(PathBuf, bool)>> = jobs
        .chunks(jobs.len().div_ceil(threads))
        .map(<[_]>::to_vec)
        .collect();
    std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                s.spawn(move || {
                    let mut found = Vec::new();
                    for (p, traced) in chunk {
                        let mode = if traced { "traced" } else { "untraced" };
                        for tag in fallbacks(&p, traced) {
                            // Both engines compile an expression alike.
                            match tag.starts_with("expr:") {
                                true => found.push(tag),
                                false => found.push(format!("{mode} {tag}")),
                            }
                        }
                    }
                    found
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

#[test]
fn tree_walker_fallbacks_only_shrink() {
    let path = tests_dir().join("vm_fallbacks.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let baseline: BTreeSet<String> = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    let actual = measure();
    let added: Vec<_> = actual.difference(&baseline).collect();
    let removed: Vec<_> = baseline.difference(&actual).collect();
    let bless = std::env::var_os("CPL_BLESS_VM_FALLBACKS").is_some();
    // Blessing seeds an empty list; after that it only removes lines.
    assert!(
        added.is_empty() || (bless && baseline.is_empty()),
        "{} new tree-walker fallback(s); move them onto the VM instead:\n  {}",
        added.len(),
        added
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    if removed.is_empty() && added.is_empty() {
        return;
    }
    if bless {
        let header: String = text
            .lines()
            .take_while(|l| l.starts_with('#'))
            .map(|l| format!("{l}\n"))
            .collect();
        let body: String = actual.iter().map(|l| format!("{l}\n")).collect();
        std::fs::write(&path, header + &body).expect("write vm_fallbacks.txt");
        return;
    }
    panic!(
        "{} fallback(s) gone; tighten the ratchet with \
         `CPL_BLESS_VM_FALLBACKS=1 cargo test -p cpl-cli --test vm_fallbacks`:\n  {}",
        removed.len(),
        removed
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
