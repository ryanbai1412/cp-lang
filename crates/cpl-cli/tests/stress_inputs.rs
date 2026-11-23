//! Every input `cpl stress` generates for a program is one that program's
//! own `read()`s accept (`debugging/R10` in
//! `handbook/guidelines/debugging-checklist.md`): each program in `tests/programs`
//! and each corpus problem runs on a few generated cases, and none may stop
//! on a read. Interactive programs read the judge's replies, so they are
//! left to the interactor suites.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cpl_stress::gen::{generate, Size};
use cpl_stress::rng::Rng;
use cpl_stress::spec::InputSpec;

const SEEDS: u64 = 3;

fn programs() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found: Vec<PathBuf> = std::fs::read_dir(root.join("crates/cpl-cli/tests/programs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    for problem in std::fs::read_dir(root.join("corpus/problems")).unwrap() {
        let spec = problem.unwrap().path().join("spec.cp");
        if spec.is_file() {
            found.push(spec);
        }
    }
    found.sort();
    found
}

/// The trap a run on `input` stopped with, when it stopped on a `read()`.
fn read_trap(program: &Path, src: &str, input: &str) -> Option<String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("run")
        .arg(program)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).ok();
    let out = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<&str> = src.lines().collect();
    let at_read = stderr
        .lines()
        .filter_map(|l| l.trim().strip_prefix("--> "))
        .any(|at| {
            let line = at.rsplit(':').nth(1).and_then(|n| n.parse::<usize>().ok());
            line.is_some_and(|n| lines.get(n - 1).is_some_and(|l| l.contains("read()")))
        });
    let read_code = ["R0501", "R0201", "R0102"]
        .iter()
        .any(|c| stderr.contains(c));
    (stderr.contains("R0501") || at_read && read_code)
        .then(|| stderr.lines().next().unwrap_or("").to_string())
}

#[test]
fn every_generated_input_is_one_the_programs_reader_accepts() {
    let programs = programs();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunks: Vec<&[PathBuf]> = programs.chunks(programs.len().div_ceil(threads)).collect();
    let (mut checked, mut rejected) = (0, Vec::new());
    std::thread::scope(|s| {
        let handles: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                s.spawn(move || {
                    let (mut checked, mut rejected) = (0, Vec::new());
                    for program in chunk {
                        let src = std::fs::read_to_string(program).unwrap();
                        if src.contains("Wire()") || src.contains("@interactor") {
                            continue;
                        }
                        let Ok(spec) = InputSpec::from_source(&src) else {
                            continue;
                        };
                        if spec.is_empty() {
                            continue;
                        }
                        for seed in 0..SEEDS {
                            let Some(case) = generate(&spec, &mut Rng::new(seed), Size::Small(8))
                            else {
                                continue;
                            };
                            checked += 1;
                            let input = case.render();
                            if let Some(trap) = read_trap(program, &src, &input) {
                                let name = program.display();
                                rejected.push(format!("{name} seed {seed}: {trap}\n{input}"));
                                break;
                            }
                        }
                    }
                    (checked, rejected)
                })
            })
            .collect();
        for h in handles {
            let (c, r) = h.join().unwrap();
            checked += c;
            rejected.extend(r);
        }
    });
    assert!(checked > 100, "only {checked} cases were checked");
    assert!(
        rejected.is_empty(),
        "{} rejected:\n{}",
        rejected.len(),
        rejected.join("\n")
    );
}
