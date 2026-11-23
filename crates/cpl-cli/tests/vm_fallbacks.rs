//! Tree-walker fallback ratchet: every `(mode, tag)` kind of statement or
//! expression the VM hands to the tree-walker, over the programs in
//! `tests/programs`, untraced and traced. `vm_fallbacks.txt` lists the kinds
//! and may only shrink, until it is empty and the tree-walker is deleted. A
//! new program that hits a listed kind adds no debt; a new kind fails.
//!
//! A fix that removes fallbacks rewrites the list with
//! `CPL_BLESS_VM_FALLBACKS=1 cargo test -p cpl-cli --test vm_fallbacks`;
//! blessing refuses to add a line to a non-empty list.
//!
//! Enforces `debugging/R4` (handbook/guidelines/debugging-checklist.md).

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
    // Every kind, not just the `top` 20 by count.
    let kinds = line
        .split("\"kinds\":[")
        .nth(1)
        .and_then(|s| s.rsplit_once("]}"))
        .map_or("", |(s, _)| s);
    kinds
        .split("\",\"")
        .map(|s| s.trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The statements the tree-walker ran, fallback or not.
fn walked(program: &Path, traced: bool) -> u64 {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cmd.arg("run").arg(program);
    if traced {
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
    stderr
        .split("\"walked\":")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no vm stats from {}:\n{stderr}", program.display()))
}

#[test]
fn the_code_above_multitest_runs_on_the_vm() {
    let program = tests_dir().join("programs/vm_multitest_setup.cp");
    for traced in [false, true] {
        assert_eq!(walked(&program, traced), 0, "traced: {traced}");
    }
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
                        for tag in fallbacks(&p, traced).iter().map(|t| unnumbered(t)) {
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

/// `tag` without the instance numbers in its fn names (`SortedBag_Int_13.pull`
/// is `SortedBag_Int.pull`), which shift whenever std gains an instance.
fn unnumbered(tag: &str) -> String {
    let once = unnumbered_once(tag);
    match once == tag {
        true => once,
        false => unnumbered(&once),
    }
}

fn unnumbered_once(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    let mut rest = tag;
    while let Some(at) = rest.find('_') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let digits = after.len() - after.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let ends = after[digits..]
            .chars()
            .next()
            .is_none_or(|c| matches!(c, '.' | '(' | ')' | ' '));
        if digits > 0 && ends {
            rest = &after[digits..];
        } else {
            out.push('_');
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

#[test]
fn unnumbered_drops_only_instance_numbers() {
    assert_eq!(
        unnumbered("traced fn:SortedBag_Int_13.pull(part)"),
        "traced fn:SortedBag_Int.pull(part)"
    );
    assert_eq!(
        unnumbered("untraced fn:m7_find_all_Str_193(feeding)"),
        "untraced fn:m7_find_all_Str(feeding)"
    );
    assert_eq!(
        unnumbered("traced stmt:Assign(Index1D,Set)"),
        "traced stmt:Assign(Index1D,Set)"
    );
    assert_eq!(
        unnumbered("traced fn:m1_get_or_insert_with_Int_Int_1_2(part)"),
        "traced fn:m1_get_or_insert_with_Int_Int(part)"
    );
}

#[test]
fn tree_walker_fallbacks_only_shrink() {
    let path = tests_dir().join("vm_fallbacks.txt");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let baseline: BTreeSet<String> = text
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(unnumbered)
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
