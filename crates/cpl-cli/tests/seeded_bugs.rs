//! The seeded-bug benchmark: every `bench/stress/<bug>.cp` is a port with
//! a classic bug planted in it and the correct fn as its `@alt`. One
//! `cpl stress <bug>.cp` must find the bug and shrink it to
//! `bench/stress/<bug>.min`, the smallest input that shows it. The time to
//! that minimal counterexample is printed per bug, with the median.
//!
//! What passes is decided by the input alone: a fixed seed and case
//! count, and a run time limit no loaded machine reaches. How fast a bug
//! is found is asserted in cases, the same on any machine; the seconds
//! are printed, never asserted, so a busy gate can't fail it.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const STRESS: [&str; 8] = [
    "--seed",
    "1",
    "--cases",
    "1000",
    "--time-limit",
    "60000",
    "--format",
    "json",
];

fn bugs() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/stress");
    let mut bugs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    bugs.sort();
    bugs
}

/// The JSON string value of `key` in `json`, unescaped (top-level or nested under `counterexample`).
fn string_field(json: &str, key: &str) -> Option<String> {
    let root = cpl_json::parse(json).ok()?;
    root.get(key)
        .or_else(|| root.get("counterexample")?.get(key))
        .and_then(cpl_json::Value::as_str)
        .map(str::to_string)
}

/// The bug classes a seeded bug may name on its `// class:` line: the P0
/// classes of `AGENTS.md` goal 2, then `logic` for a wrong algorithm.
const CLASSES: [&str; 12] = [
    "off-by-one",
    "index-space",
    "overflow",
    "boundaries",
    "multitest-leak",
    "io-flush",
    "sentinel",
    "complexity",
    "mutation",
    "modulus",
    "floating-point",
    "logic",
];

/// What `bug`'s `// class:` line names.
fn class_of(bug: &Path) -> Option<String> {
    let source = std::fs::read_to_string(bug).ok()?;
    let line = source
        .lines()
        .find_map(|line| line.strip_prefix("// class: "))?;
    Some(line.trim().to_string())
}

#[test]
fn every_seeded_bug_names_its_bug_class() {
    let mut count = [0usize; CLASSES.len()];
    for bug in bugs() {
        let class = class_of(&bug);
        let at = class
            .as_deref()
            .and_then(|c| CLASSES.iter().position(|&k| k == c));
        let Some(at) = at else {
            panic!(
                "{}: no `// class:` line naming one of {CLASSES:?}",
                bug.display()
            );
        };
        count[at] += 1;
    }
    for (class, n) in CLASSES.iter().zip(count) {
        println!("{class:<16} {n}");
    }
    let p0 = CLASSES.len() - 1;
    let covered = count[..p0].iter().filter(|&&n| n > 0).count();
    println!("{covered} of {p0} P0 classes seeded");
}

/// `run` on every bug, a few at a time.
fn each<B: Sync, T: Send>(bugs: &[B], run: impl Fn(&B) -> T + Sync) -> Vec<T> {
    let threads = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(4);
    let next = std::sync::atomic::AtomicUsize::new(0);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let at = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(bug) = bugs.get(at) else { break done };
                        done.push(run(bug));
                    }
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    })
}

/// `cpl stress <bug>` on a copy of `bug` in `scratch`, so the saved
/// regression test lands outside the repo, with `args` after the file.
fn stress(bug: &Path, scratch: &Path, args: &[&str]) -> std::process::Output {
    let copy = scratch.join(bug.file_name().unwrap());
    std::fs::copy(bug, &copy).unwrap();
    // A bug that never ends names a short limit, so each case it hangs on stops fast.
    let source = std::fs::read_to_string(bug).unwrap();
    let limit = (source.lines())
        .find_map(|line| line.strip_prefix("// time-limit-ms: "))
        .map(|ms| ["--time-limit", ms.trim()]);
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("stress")
        .arg(&copy)
        .args(args)
        .args(limit.iter().flatten())
        .output()
        .expect("cpl runs")
}

/// A fresh scratch folder for the copies of the bugs one test stresses.
fn scratch(test: &str) -> PathBuf {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    scratch
}

#[test]
fn stress_finds_every_seeded_bug_and_shrinks_it_to_the_minimal_input() {
    let bugs = bugs();
    assert!(bugs.len() >= 21, "only {} seeded bugs", bugs.len());
    let scratch = scratch("seeded_bugs");
    let run = |bug: &PathBuf| -> (String, Duration, Option<i64>, Option<String>) {
        let name = bug.file_stem().unwrap().to_string_lossy().to_string();
        let minimal = std::fs::read_to_string(bug.with_extension("min")).unwrap();
        let start = Instant::now();
        let out = stress(bug, &scratch, &STRESS);
        let took = start.elapsed();
        let json = String::from_utf8_lossy(&out.stdout).to_string();
        let found = string_field(&json, "raw_input");
        let case = (cpl_json::parse(&json).ok())
            .and_then(|root| root.get("counterexample")?.get("case")?.as_i64());
        let wrong = (found.as_deref() != Some(minimal.as_str())).then(|| {
            format!(
                "{name}: wanted\n{minimal}got {found:?}\n{json}{}",
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (name, took, case, wrong)
    };
    let mut results = each(&bugs, run);
    results.sort_by(|a, b| a.0.cmp(&b.0));
    let mut times = Vec::new();
    let mut cases = Vec::new();
    let mut wrong = Vec::new();
    for (name, took, case, failure) in results {
        let verdict = if failure.is_some() {
            "WRONG"
        } else {
            "minimal"
        };
        let class = class_of(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../bench/stress")
                .join(format!("{name}.cp")),
        )
        .unwrap_or_default();
        println!(
            "{name:<32} {class:<16} {:>6.2} s  case {:>4}  {verdict}",
            took.as_secs_f64(),
            case.unwrap_or(0)
        );
        times.push(took);
        cases.push((case.unwrap_or(i64::MAX), name));
        wrong.extend(failure);
    }
    times.sort();
    let median = times[times.len() / 2];
    println!(
        "median {:.2} s over {} seeded bugs",
        median.as_secs_f64(),
        times.len()
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    // How fast a bug is found, in the cases it takes: the same on any machine.
    cases.sort();
    let (median, slowest) = (cases[cases.len() / 2].0, cases.last().unwrap());
    println!("found at case {median} (median), {} at most", slowest.0);
    assert!(
        median <= CASES_TO_BUG_MEDIAN && slowest.0 <= CASES_TO_BUG_MAX,
        "the median bug is found at case {median} (at most {CASES_TO_BUG_MEDIAN}), {} at case {} (at most {CASES_TO_BUG_MAX})",
        slowest.1,
        slowest.0
    );
}

/// The case a seeded bug is found at: the median and the slowest. Lower
/// them as stress finds bugs sooner; they may never rise.
const CASES_TO_BUG_MEDIAN: i64 = 1;
const CASES_TO_BUG_MAX: i64 = 170;

/// How many seeded bugs `cpl stress --explain` points at the very line
/// the bug was planted on, the line ending in `// planted`. Raise it when
/// the explanation gets better; it may never drop.
const NAMED: usize = 29;

/// The lines a `cpl stress --explain` report blames: where the debug run
/// stopped, and where the solution first gave a value the brute did not.
fn blamed(report: &str) -> Vec<u32> {
    let number = |text: &str| -> Option<u32> {
        let digits: String = text.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    };
    (report.lines())
        .filter_map(|line| {
            let line = line.trim_start();
            if line.starts_with("--> ") {
                number(line.rsplit_once(".cp:")?.1)
            } else if let Some(rest) = line.strip_prefix("look at line ") {
                number(rest)
            } else if line.starts_with("solution ") {
                number(line.rsplit_once(" at line ")?.1)
            } else {
                None
            }
        })
        .collect()
}

/// Enforces `debugging/R6` (handbook/guidelines/debugging-checklist.md).
#[test]
fn explain_names_the_line_each_bug_was_planted_on() {
    let bugs = bugs();
    let scratch = scratch("seeded_bugs_explain");
    let args = &STRESS[..STRESS.len() - 2];
    let mut results = each(&bugs, |bug| {
        let name = bug.file_stem().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(bug).unwrap();
        let planted = (source.lines().position(|line| line.ends_with("// planted")))
            .unwrap_or_else(|| panic!("{name}: no line ends in `// planted`"))
            as u32
            + 1;
        let out = stress(bug, &scratch, &[args, &["--explain"]].concat());
        let report = String::from_utf8_lossy(&out.stdout).to_string();
        (name, planted, blamed(&report))
    });
    results.sort();
    let mut named = 0;
    for (name, planted, blamed) in &results {
        let verdict = if blamed.contains(planted) {
            named += 1;
            "named"
        } else {
            "missed"
        };
        println!("{name:<32} planted at {planted:<3} blamed {blamed:?} {verdict}");
    }
    println!(
        "{named} of {} seeded bugs named at their line",
        results.len()
    );
    assert!(named >= NAMED, "only {named} named, down from {NAMED}");
}

/// One-token changes a mutant makes, each a classic slip.
const MUTATIONS: [(&str, &str); 12] = [
    (" < ", " <= "),
    (" <= ", " < "),
    (" > ", " >= "),
    (" >= ", " > "),
    (" == ", " != "),
    (" + 1", " + 2"),
    (" - 1", ""),
    (" += ", " -= "),
    (" + ", " - "),
    (" * ", " + "),
    ("min(", "max("),
    ("max(", "min("),
];

/// The lines `fn` spans in `lines`, from its header: one line for
/// `fn f() = expr`, else the header and its indented body.
fn fn_lines(lines: &[&str], header: usize) -> std::ops::Range<usize> {
    let mut end = header + 1;
    while end < lines.len() && (lines[end].starts_with(' ') || lines[end].trim().is_empty()) {
        end += 1;
    }
    while lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    header..end
}

/// Mutants of a seeded bug the explanation was never tuned on: its
/// solution replaced by its correct `@alt` fn with one mutation, each
/// with the line the mutation is on. The `@alt` stays the brute.
fn mutants(bug: &Path) -> Vec<(String, String, u32)> {
    let source = std::fs::read_to_string(bug).unwrap();
    let source = source
        .replace("  // planted", "")
        .replace(" // planted", "");
    let lines: Vec<&str> = source.lines().collect();
    let Some(alt) = lines.iter().position(|l| l.starts_with("@alt(")) else {
        return Vec::new();
    };
    let solution = &lines[alt]["@alt(".len()..lines[alt].find(')').unwrap()];
    let Some(header) = (lines.iter()).position(|l| l.starts_with(&format!("fn {solution}(")))
    else {
        return Vec::new();
    };
    let solved = fn_lines(&lines, header);
    let correct = fn_lines(&lines, alt + 1);
    let brute = &lines[alt + 1]["fn ".len()..lines[alt + 1].find('(').unwrap()];
    // A brute that calls itself takes time exponential in the input: as the
    // solution, no case of it would finish.
    if (lines[correct.start + 1..correct.end].iter()).any(|l| l.contains(&format!("{brute}("))) {
        return Vec::new();
    }
    let mut body: Vec<String> = lines[correct].iter().map(|l| l.to_string()).collect();
    body[0] = body[0].replacen(&format!("fn {brute}("), &format!("fn {solution}("), 1);
    let stem = bug.file_stem().unwrap().to_string_lossy();
    let mut found = Vec::new();
    for (k, line) in body.iter().enumerate() {
        let code = line.split("//").next().unwrap();
        // A header only changes in the expression of `fn f() = expr`.
        let from = if k == 0 {
            code.find(" = ").map_or(code.len(), |at| at + 3)
        } else {
            0
        };
        for (was, now) in MUTATIONS {
            for (at, _) in code.match_indices(was).filter(|(at, _)| *at >= from) {
                let next = code[at + was.len()..].chars().next();
                if was.ends_with('1') && next.is_some_and(|c| c.is_ascii_alphanumeric()) {
                    continue;
                }
                let mutated = format!("{}{now}{}", &line[..at], &line[at + was.len()..]);
                let mut file: Vec<String> = lines[..solved.start]
                    .iter()
                    .map(|l| l.to_string())
                    .collect();
                file.extend(body[..k].iter().cloned());
                file.push(mutated);
                file.extend(body[k + 1..].iter().cloned());
                file.extend(lines[solved.end..].iter().map(|l| l.to_string()));
                let name = format!("{stem}-m{}", found.len());
                found.push((name, file.join("\n") + "\n", (solved.start + k + 1) as u32));
            }
        }
    }
    found
}

/// How many mutants `cpl stress --explain` points at the mutated line,
/// and how many it must at least find. Raise them as explanations get
/// better; they may never drop.
const MUTANTS_FOUND: usize = 53;
const MUTANTS_NAMED: usize = 46;

#[test]
fn explain_names_the_line_of_mutants_it_was_not_tuned_on() {
    let scratch = scratch("explain_mutants");
    let sources = scratch.join("mutants");
    std::fs::create_dir_all(&sources).unwrap();
    let mutants: Vec<(PathBuf, u32)> = (bugs().iter())
        .flat_map(|bug| mutants(bug))
        .map(|(name, source, line)| {
            let path = sources.join(format!("{name}.cp"));
            std::fs::write(&path, source).unwrap();
            (path, line)
        })
        .collect();
    let args = [
        "--seed",
        "1",
        "--cases",
        "1000",
        "--time-limit",
        "2000",
        "--explain",
    ];
    // Each mutant stresses in a folder of its own, so no saved case of one
    // decides another.
    let mut results = each(&mutants, |(path, line)| {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let dir = scratch.join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        let report = String::from_utf8_lossy(&stress(path, &dir, &args).stdout).to_string();
        (name, *line, report.contains("FAIL"), blamed(&report))
    });
    results.sort();
    let (mut found, mut named) = (0, 0);
    for (name, line, caught, blamed) in &results {
        let verdict = match (caught, blamed.contains(line)) {
            (false, _) => "not found",
            (true, true) => "named",
            (true, false) => "missed",
        };
        found += usize::from(*caught);
        named += usize::from(*caught && blamed.contains(line));
        println!("{name:<36} mutated {line:<3} blamed {blamed:?} {verdict}");
    }
    println!(
        "{named} of {found} found mutants named at their line, of {}",
        mutants.len()
    );
    assert!(
        found >= MUTANTS_FOUND,
        "only {found} mutants found, down from {MUTANTS_FOUND}"
    );
    assert!(
        named >= MUTANTS_NAMED,
        "only {named} named, down from {MUTANTS_NAMED}"
    );
}

/// How many mutants of each corpus port are stressed, spread over its lines.
const PER_PORT: usize = 8;

/// How many of those mutants stress kills. Raise it as stress finds more;
/// it may never drop.
const CORPUS_MUTANTS_KILLED: usize = 10;

/// Mutants of a corpus port whose solution has an `@alt`: one mutation
/// each, outside the `@alt` fn, at most [`PER_PORT`] spread evenly.
fn port_mutants(port: &PathBuf) -> Vec<(String, String)> {
    let source = std::fs::read_to_string(port).unwrap();
    let lines: Vec<&str> = source.lines().collect();
    let Some(alt) = lines.iter().position(|l| l.starts_with("@alt")) else {
        return Vec::new();
    };
    let brute = fn_lines(&lines, alt + 1);
    let id = port
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy();
    let mut all = Vec::new();
    for (k, line) in lines.iter().enumerate() {
        let code = line.split("//").next().unwrap();
        if k == alt || brute.contains(&k) || code.trim_start().starts_with('@') {
            continue;
        }
        for (was, now) in MUTATIONS {
            for (at, _) in code.match_indices(was) {
                let next = code[at + was.len()..].chars().next();
                if was.ends_with('1') && next.is_some_and(|c| c.is_ascii_alphanumeric()) {
                    continue;
                }
                let mut file: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
                file[k] = format!("{}{now}{}", &line[..at], &line[at + was.len()..]);
                all.push(file.join("\n") + "\n");
            }
        }
    }
    let step = all.len().div_ceil(PER_PORT).max(1);
    (all.into_iter().step_by(step).enumerate())
        .map(|(i, source)| (format!("{id}-m{i}"), source))
        .collect()
}

/// Bugs nobody planted: one-token mutants of every corpus port that has
/// an `@alt` to check it against, and the share of them stress kills.
#[test]
fn stress_kills_mutants_of_corpus_ports_with_an_alt() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let listed = Command::new("uv")
        .args([
            "run",
            "--quiet",
            "python",
            "scripts/corpus.py",
            "ls",
            "--file",
            "port.cp",
        ])
        .current_dir(&root)
        .output()
        .expect("uv runs scripts/corpus.py ls");
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let ports: Vec<PathBuf> = (String::from_utf8_lossy(&listed.stdout).lines())
        .map(|line| root.join(line))
        .filter(|port| {
            std::fs::read_to_string(port).is_ok_and(|s| s.lines().any(|l| l.starts_with("@alt")))
        })
        .collect();
    let scratch = scratch("corpus_mutants");
    let args = ["--seed", "1", "--cases", "300", "--time-limit", "2000"];
    // Whether stress fails the file; `None` when it can't run it at all.
    let caught = |path: &PathBuf, name: &str| -> Option<bool> {
        let dir = scratch.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let out = stress(path, &dir, &args);
        match (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).contains("FAIL"),
        ) {
            (_, true) => Some(true),
            (Some(0), false) => Some(false),
            _ => None,
        }
    };
    let id = |port: &PathBuf| {
        port.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    };
    let passing: Vec<PathBuf> = each(&ports, |port| (port.clone(), caught(port, &id(port))))
        .into_iter()
        .filter_map(|(port, caught)| match caught {
            Some(false) => Some(port),
            _ => {
                println!(
                    "{}: fails its own @alt unmutated, so it isn't mutated",
                    id(&port)
                );
                None
            }
        })
        .collect();
    assert!(
        passing.len() >= 4,
        "only {} of {} ports pass",
        passing.len(),
        ports.len()
    );
    let sources = scratch.join("mutants");
    std::fs::create_dir_all(&sources).unwrap();
    let mutants: Vec<PathBuf> = (passing.iter().flat_map(port_mutants))
        .map(|(name, source)| {
            let path = sources.join(format!("{name}.cp"));
            std::fs::write(&path, source).unwrap();
            path
        })
        .collect();
    let mut results = each(&mutants, |path| {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let caught = caught(path, &name);
        (name, caught)
    });
    results.sort();
    let run: Vec<(String, bool)> = (results.into_iter())
        .filter_map(|(name, caught)| Some((name, caught?)))
        .collect();
    for (name, killed) in &run {
        println!("{name:<28} {}", if *killed { "killed" } else { "survived" });
    }
    let killed = run.iter().filter(|(_, killed)| *killed).count();
    println!(
        "stress killed {killed} of {} mutants of {} corpus ports",
        run.len(),
        passing.len()
    );
    assert!(
        killed >= CORPUS_MUTANTS_KILLED,
        "only {killed} mutants killed, down from {CORPUS_MUTANTS_KILLED}"
    );
}
