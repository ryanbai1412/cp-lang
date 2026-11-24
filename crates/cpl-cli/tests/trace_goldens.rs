//! A traced run of every program in `tests/programs` records the trace
//! saved next to it in `<name>.trace`, on the VM and on the tree-walker
//! the browser traces on while wasm builds leave out the VM. After a
//! change that is meant to change traces, run with `CPL_BLESS=1` and
//! review the diff.
//!
//! A saved trace is the run's variables and steps, one step a line, and
//! what it printed to standard error. Past 8 KB it keeps the first lines
//! and a hash of the rest, so a large run is still checked whole.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "support/trace_diff.rs"]
mod trace_diff;

const KEPT: usize = 8 << 10;

/// The trace `program` records, as saved, on the tree-walker if `tree`.
fn traced(program: &Path, tree: bool) -> String {
    let input = program.with_extension("in");
    let stdin = match std::fs::File::open(&input) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    };
    let mut cpl = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cpl.args([
        "run",
        program.to_str().unwrap(),
        "--trace",
        "--format",
        "json",
    ])
    .stdin(stdin);
    if tree {
        cpl.env("CPL_TRACE_ENGINE", "tree");
    } else {
        cpl.env_remove("CPL_TRACE_ENGINE");
    }
    let out = cpl.output().unwrap();
    let document = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
    // The title and source come first and depend on the checkout.
    let from = document.find("\"variables\":").unwrap_or(0);
    let steps = document[from..]
        .trim_end()
        .replace(",{\"id\":", ",\n{\"id\":");
    let name = program.file_name().unwrap().to_string_lossy();
    let stderr = String::from_utf8_lossy(&out.stderr)
        .replace("\r\n", "\n")
        .replace(program.to_str().unwrap(), &name);
    let mut saved = String::new();
    if steps.len() <= KEPT {
        saved.push_str(&steps);
        saved.push('\n');
    } else {
        let mut cut = steps[..KEPT].rfind('\n').unwrap_or(0);
        while !steps.is_char_boundary(cut) {
            cut -= 1;
        }
        let rest = &steps[cut..];
        saved.push_str(&steps[..cut]);
        saved.push_str(&format!(
            "\n... {} more lines, {} bytes, fnv64 {:016x}\n",
            rest.matches('\n').count(),
            rest.len(),
            fnv64(rest.as_bytes())
        ));
    }
    saved.push_str("stderr:\n");
    saved.push_str(&stderr);
    saved
}

/// What cpl prints when it fails itself: such a run is a bug to fix,
/// so no saved trace may hold it and blessing never writes one.
const CRASHED: &str = "this is a cpl bug";

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// Enforces `debugging/R2` (handbook/guidelines/debugging-checklist.md).
#[test]
fn every_program_records_its_saved_trace() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .collect();
    programs.sort();
    assert!(programs.len() > 150, "{} programs", programs.len());
    let bless = std::env::var_os("CPL_BLESS").is_some();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<_> = programs.chunks(programs.len().div_ceil(threads)).collect();
    let differ: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                scope.spawn(move || {
                    let mut differ = Vec::new();
                    for program in *chunk {
                        let golden = program.with_extension("trace");
                        if bless {
                            let trace = traced(program, false);
                            if !trace.contains(CRASHED) {
                                std::fs::write(&golden, trace).unwrap();
                            }
                        }
                        let Ok(want) = std::fs::read_to_string(&golden) else {
                            differ.push(format!("{} is not saved", golden.display()));
                            continue;
                        };
                        let want = want.replace("\r\n", "\n");
                        for (engine, tree) in [("vm", false), ("tree-walker", true)] {
                            let got = traced(program, tree);
                            if got.contains(CRASHED) || want.contains(CRASHED) {
                                differ.push(format!(
                                    "{} on the {engine}: cpl crashed, and a crash is never a saved trace",
                                    golden.file_name().unwrap().to_string_lossy()
                                ));
                            } else if got != want {
                                differ.push(format!(
                                    "{} on the {engine}:\n  {}",
                                    golden.file_name().unwrap().to_string_lossy(),
                                    trace_diff::classify(&got, &want).join("\n  ")
                                ));
                            }
                        }
                    }
                    differ
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect()
    });
    assert!(
        differ.is_empty(),
        "{} traces differ from the saved ones:\n{}\n\nIf the change is meant to move them, bisect any lost step first, then re-record \
         with `CPL_BLESS=1 cargo nextest run -p cpl-cli --test trace_goldens` and commit the \
         .trace files with the change.",
        differ.len(),
        differ.join("\n")
    );
}

/// What `program` prints to standard output, run untraced.
fn printed(program: &Path) -> String {
    let stdin = match std::fs::File::open(program.with_extension("in")) {
        Ok(file) => Stdio::from(file),
        Err(_) => Stdio::null(),
    };
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(["run", program.to_str().unwrap()])
        .env_remove("CPL_TRACE_ENGINE")
        .stdin(stdin)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n")
}

/// A JSON array of strings at the start of `json`, and the rest after it.
fn strings(json: &str) -> Option<(Vec<String>, &str)> {
    let mut chars = json.strip_prefix('[')?.char_indices();
    let mut lines = Vec::new();
    loop {
        match chars.next()? {
            (at, ']') => return Some((lines, &json[at + 2..])),
            (_, ',') => {}
            (_, '"') => {
                let mut line = String::new();
                loop {
                    match chars.next()?.1 {
                        '"' => break,
                        '\\' => match chars.next()?.1 {
                            'n' => line.push('\n'),
                            't' => line.push('\t'),
                            'r' => line.push('\r'),
                            'u' => {
                                let hex: String = chars.by_ref().take(4).map(|(_, c)| c).collect();
                                line.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                            }
                            c => line.push(c),
                        },
                        c => line.push(c),
                    }
                }
                lines.push(line);
            }
            _ => return None,
        }
    }
}

/// What a traced run had printed by its end: the output of each test
/// case's last step, case after case. `None` if the trace was cut short.
fn traced_stdout(document: &str) -> Option<Vec<String>> {
    if document.contains("\"truncated\":true") {
        return None;
    }
    let mut cases: Vec<(&str, Vec<String>)> = Vec::new();
    let mut rest = document;
    while let Some(at) = rest.find("\"stage\":\"") {
        rest = &rest[at + "\"stage\":\"".len()..];
        let stage = &rest[..rest.find('"')?];
        let Some(values) = rest[stage.len()..].strip_prefix("\",\"values\":{\"stdout\":") else {
            continue;
        };
        let (lines, after) = strings(values)?;
        match cases.last_mut() {
            Some((last, kept)) if *last == stage => *kept = lines,
            _ => cases.push((stage, lines)),
        }
        rest = after;
    }
    Some(cases.into_iter().flat_map(|(_, lines)| lines).collect())
}

/// Tracing never changes what a program does: every program prints the
/// same lines traced as untraced.
///
/// Enforces `debugging/R1` (handbook/guidelines/debugging-checklist.md).
#[test]
fn a_traced_run_prints_what_an_untraced_run_prints() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut programs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .collect();
    programs.sort();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<_> = programs.chunks(programs.len().div_ceil(threads)).collect();
    let differ: Vec<String> = std::thread::scope(|scope| {
        let workers: Vec<_> = chunks
            .iter()
            .map(|chunk| {
                scope.spawn(move || {
                    let mut differ = Vec::new();
                    for program in *chunk {
                        let name = program.file_name().unwrap().to_string_lossy().to_string();
                        let untraced: Vec<String> =
                            printed(program).lines().map(str::to_string).collect();
                        let mut cpl = Command::new(env!("CARGO_BIN_EXE_cpl"));
                        cpl.args([
                            "run",
                            program.to_str().unwrap(),
                            "--trace",
                            "--format",
                            "json",
                        ]);
                        cpl.env_remove("CPL_TRACE_ENGINE");
                        cpl.stdin(match std::fs::File::open(program.with_extension("in")) {
                            Ok(file) => Stdio::from(file),
                            Err(_) => Stdio::null(),
                        });
                        let document = String::from_utf8_lossy(&cpl.output().unwrap().stdout)
                            .replace("\r\n", "\n");
                        match traced_stdout(&document) {
                            Some(traced) if traced == untraced => {}
                            Some(traced) => differ.push(format!(
                                "{name}: traced {traced:?}\n  untraced {untraced:?}"
                            )),
                            None => {}
                        }
                    }
                    differ
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect()
    });
    assert!(
        differ.is_empty(),
        "{} programs print differently when traced:\n{}",
        differ.len(),
        differ.join("\n")
    );
}

/// A variable too large to keep whole is drawn by a window of its first
/// cells, not left empty.
#[test]
fn a_large_array_is_drawn_by_a_window() {
    let dir = std::env::temp_dir().join(format!("cpl-window-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let program = dir.join("big.cp");
    std::fs::write(
        &program,
        "var big = [0 for _ in 0..30000]\nbig[3] = 7\nprint(big[3])\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args([
            "run",
            program.to_str().unwrap(),
            "--trace",
            "--format",
            "json",
        ])
        .env_remove("CPL_TRACE_ENGINE")
        .output()
        .unwrap();
    let document = String::from_utf8_lossy(&out.stdout);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        document.contains("\"window\":true"),
        "the descriptor names the window: {}",
        &document[..document.len().min(600)]
    );
    let last = document.rfind("\"big\":[").expect("a step holds big");
    let drawn = &document[last + "\"big\":".len()..];
    let drawn = &drawn[..drawn.find(']').unwrap() + 1];
    assert!(
        drawn.starts_with("[0,0,0,7,0,"),
        "the window starts at the first cell: {}",
        &drawn[..drawn.len().min(80)]
    );
    assert_eq!(
        drawn.matches(',').count() + 1,
        500,
        "the window holds 500 cells"
    );
}

/// The private fields of each `pub struct` in `std/`: what a value of a
/// std type is made of inside, which a trace must never show.
fn std_private_fields() -> std::collections::BTreeMap<String, Vec<String>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let mut fields = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "cp") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = source.lines().collect();
        for (at, line) in lines.iter().enumerate() {
            let Some(rest) = line.strip_prefix("pub struct ") else {
                continue;
            };
            let name: String = (rest.chars())
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let mut private = Vec::new();
            let mut take = |param: &str| {
                let mut param = param.trim();
                while let Some(after) = param.strip_prefix('@') {
                    param = after
                        .split_once(' ')
                        .map_or("", |(_, rest)| rest)
                        .trim_start();
                }
                if param.starts_with("pub ") || param.starts_with("///") {
                    return;
                }
                if let Some((ident, _)) = param.split_once(": ") {
                    if !ident.is_empty() && ident.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        private.push(ident.to_string());
                    }
                }
            };
            if let Some(open) = rest.find('(') {
                if let Some(close) = rest.rfind(')') {
                    rest[open + 1..close].split(',').for_each(&mut take);
                }
            }
            if rest.trim_end().ends_with(':') {
                for body in lines[at + 1..]
                    .iter()
                    .take_while(|l| l.trim().is_empty() || l.starts_with("    "))
                {
                    if body.starts_with("    ") && !body.starts_with("     ") {
                        take(body);
                    }
                }
            }
            if !private.is_empty() {
                fields.insert(name, private);
            }
        }
    }
    fields
}

/// The private fields of `ty`'s struct that `value` shows.
fn shown_fields(value: &cpl_json::Value, private: &[String], found: &mut Vec<String>) {
    if let Some(object) = value.as_object() {
        for (key, inner) in object {
            if private.contains(key) && !found.contains(key) {
                found.push(key.clone());
            }
            shown_fields(inner, private, found);
        }
    } else if let Some(items) = value.as_array() {
        items
            .iter()
            .for_each(|item| shown_fields(item, private, found));
    }
}

/// Enforces the debugger register row "explanations use the program's
/// words": no saved trace shows a value of a std type as the private
/// fields it is made of.
#[test]
fn no_saved_trace_shows_the_insides_of_a_std_type() {
    let private = std_private_fields();
    assert!(
        private.len() > 20,
        "{} std structs with private fields",
        private.len()
    );
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs");
    let mut traces: Vec<PathBuf> = (std::fs::read_dir(&dir).unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "trace"))
        .collect();
    traces.sort();
    let mut leaks = Vec::new();
    let mut checked = 0;
    for trace in &traces {
        let text = std::fs::read_to_string(trace).unwrap();
        let source = std::fs::read_to_string(trace.with_extension("cp")).unwrap_or_default();
        let own = |name: &str| {
            (source.lines()).any(|line| {
                line.trim_start()
                    .strip_prefix("struct ")
                    .or_else(|| line.trim_start().strip_prefix("pub struct "))
                    .is_some_and(|rest| {
                        rest.starts_with(name)
                            && !rest[name.len()..]
                                .starts_with(|c: char| c.is_alphanumeric() || c == '_')
                    })
            })
        };
        let object = |line: &str, at: usize| {
            let end = at + trace_diff::object_len(&line[at..]);
            cpl_json::parse(&line[at..end]).ok()
        };
        let Some(variables) = (text.lines())
            .find_map(|line| line.find("\"variables\":").map(|at| object(line, at + 12)))
            .flatten()
        else {
            continue;
        };
        for line in text.lines() {
            let Some(step) = line.find("{\"id\":").and_then(|at| object(line, at)) else {
                continue;
            };
            for scope in ["values", "locals"] {
                let Some(values) = step.get(scope).and_then(cpl_json::Value::as_object) else {
                    continue;
                };
                for (name, value) in values {
                    let ty = (variables.get(name))
                        .and_then(|v| v.get("type"))
                        .and_then(cpl_json::Value::as_str)
                        .unwrap_or("");
                    let head = ty.split(['<', '(', '[']).next().unwrap_or("");
                    let Some(fields) = private.get(head).filter(|_| !own(head)) else {
                        continue;
                    };
                    checked += 1;
                    let mut found = Vec::new();
                    shown_fields(value, fields, &mut found);
                    if !found.is_empty() {
                        let file = trace.file_name().unwrap().to_string_lossy();
                        leaks.push(format!("{file}: `{name}: {ty}` shows {found:?}"));
                    }
                }
            }
        }
    }
    leaks.dedup();
    assert!(checked > 0, "no saved trace holds a value of a std type");
    assert!(
        leaks.is_empty(),
        "values shown as their insides:\n{}",
        leaks.join("\n")
    );
}
