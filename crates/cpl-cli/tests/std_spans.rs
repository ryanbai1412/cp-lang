//! Generated std calls: every `pub fn` in `std/` is called with wrong
//! argument shapes, and no error may point into `std/`. A library gap is
//! shown at the user's call, with the std line in a note. The goldens
//! check this for the mistakes we wrote down; this checks it for every
//! function the library has.
//! Rule: `errors/R3`, handbook/guidelines/errors-checklist.md#r3-never-a-primary-span-inside-std.

use std::fs;
use std::path::Path;
use std::process::Command;

const CALLS: [&str; 5] = [
    "a.{f}()",
    "a.{f}(\"x\")",
    "a.{f}(1, 2)",
    "{f}(\"x\")",
    "{f}(a, x => x)",
];

fn programs() -> Vec<(String, String)> {
    let std = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut out = Vec::new();
    let mut mods: Vec<_> = fs::read_dir(&std)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    mods.sort();
    for path in mods {
        let module = path.file_stem().unwrap().to_str().unwrap().to_string();
        let src = fs::read_to_string(&path).unwrap();
        let mut seen = std::collections::BTreeSet::new();
        for line in src.lines() {
            let Some(rest) = line.strip_prefix("pub fn ") else {
                continue;
            };
            let f: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if f.is_empty() || !seen.insert(f.clone()) {
                continue;
            }
            for (i, call) in CALLS.iter().enumerate() {
                let call = call.replace("{f}", &f);
                let use_line = if module == "prelude" {
                    String::new()
                } else {
                    format!("use std.{module}.{{{f}}}\n")
                };
                out.push((
                    format!("{module}_{f}_{i}"),
                    format!("{use_line}let a = [3, 1, 2]\nprint({call})\n"),
                ));
            }
        }
    }
    out
}

/// Errors whose primary span is inside std, as `error ...\n  --> std/...`.
fn inside_std(files: &[std::path::PathBuf]) -> Vec<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .args(files)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout);
    let mut inside = Vec::new();
    let mut lines = text.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.starts_with("error") {
            continue;
        }
        if let Some(at) = lines
            .peek()
            .and_then(|l| l.trim_start().strip_prefix("--> "))
        {
            if at.starts_with("std/") || at.contains("/std/") {
                inside.push(format!("{line}\n  {at}"));
            }
        }
    }
    inside
}

/// Programs per `cpl check`: one process loads std once for the whole batch,
/// so a batch costs about what one program does on its own.
const BATCH: usize = 40;

#[test]
fn no_error_on_a_std_call_points_into_std() {
    let programs = programs();
    assert!(programs.len() > 1000, "only {} programs", programs.len());
    let dir = tempfile_dir();
    let files: Vec<_> = programs
        .iter()
        .map(|(name, src)| {
            let file = dir.join(format!("{name}.cp"));
            fs::write(&file, src).unwrap();
            file
        })
        .collect();
    let batches: Vec<_> = files.chunks(BATCH).zip(programs.chunks(BATCH)).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let inside: Vec<String> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut inside = Vec::new();
                    while let Some((files, programs)) =
                        batches.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
                    {
                        if inside_std(files).is_empty() {
                            continue;
                        }
                        for (file, (name, src)) in files.iter().zip(programs.iter()) {
                            for at in inside_std(std::slice::from_ref(file)) {
                                inside.push(format!("{name}: {at}\n{src}"));
                            }
                        }
                    }
                    inside
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    let _ = fs::remove_dir_all(&dir);
    eprintln!("{} generated std calls checked", programs.len());
    assert!(
        inside.is_empty(),
        "{} of {} generated std calls put the primary span inside std:\n{}",
        inside.len(),
        programs.len(),
        inside
            .iter()
            .take(20)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn tempfile_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cpl-std-spans-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}
