//! How well the front end recovers from broken code: one diagnostic per
//! root cause, at the right line.
//!
//! Two corpora. The code blocks of `plan/1x-*.md` are mostly valid 0.2.2
//! with some planned syntax mixed in. The broken ports take every
//! `proto/ports/ecnerwala` port, which checks clean, and break one line at
//! a time with a small set of edits a contestant makes by accident. Each
//! broken copy has exactly one root cause, so every error past the first
//! is a cascade, and the first error should be on the line that was
//! broken.
//!
//! `cargo test -p cpl-cli --test recovery -- --nocapture` prints the
//! scoreboard; the test fails when a number gets worse than its bar.

use cpl_diag::{Diagnostic, LineIndex, Severity};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The errors `cpl check` reports for `src`, read as if it were at `path`
/// so that its `cp.toml` libraries resolve.
fn errors(path: &str, src: &str) -> Vec<Diagnostic> {
    let mut db = cpl_check::Database::new();
    db.set_file(path, src);
    let analysis = db.analyze(path).expect("the file was just set");
    let mut out: Vec<Diagnostic> = cpl_ir::check(&analysis)
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    out.sort_by_key(|d| d.span.start);
    out
}

/// The accidental edits, each applied to one line. `None` when the edit
/// does not apply to the line.
type Edit = fn(&str) -> Option<String>;

const EDITS: &[(&str, Edit)] = &[
    ("drop the last closer", |l| {
        let at = l.rfind([')', ']'])?;
        Some(format!("{}{}", &l[..at], &l[at + 1..]))
    }),
    ("drop the first closer", |l| {
        let at = l.find([')', ']'])?;
        (l.rfind([')', ']']) != Some(at)).then(|| format!("{}{}", &l[..at], &l[at + 1..]))
    }),
    ("drop the block colon", |l| {
        let t = l.trim_end();
        (t.ends_with(':') && !t.trim_start().starts_with("//"))
            .then(|| t[..t.len() - 1].to_string())
    }),
    ("cut the value after `=`", |l| {
        let t = l.trim_start();
        if !(t.starts_with("let ") || t.starts_with("var ") || t.starts_with("domain ")) {
            return None;
        }
        let at = l.find(" = ")?;
        Some(l[..at + 2].to_string())
    }),
    ("double an operator", |l| {
        let at = [" + ", " - ", " * ", " < ", " == "]
            .iter()
            .filter_map(|op| l.find(op))
            .min()?;
        Some(format!("{} *{}", &l[..at + 2], &l[at + 2..]))
    }),
    ("garble a declared name", |l| {
        let t = l.trim_start();
        let kw = ["let (", "var (", "fn ", "struct ", "enum "]
            .iter()
            .find(|kw| t.starts_with(**kw))?;
        let at = l.len() - t.len() + kw.len();
        Some(format!("{}${}", &l[..at], &l[at..]))
    }),
];

struct Broken {
    path: String,
    edit: &'static str,
    line: u32,
    src: String,
}

/// Every port broken by every edit on up to three of its lines, picked
/// the same way on every run.
fn broken_ports() -> Vec<Broken> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(root().join("proto/ports/ecnerwala"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "cp"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let src = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = src.lines().collect();
        for (edit, apply) in EDITS {
            let hits: Vec<(usize, String)> = lines
                .iter()
                .enumerate()
                .filter(|(_, l)| !l.trim_start().starts_with("//"))
                .filter_map(|(i, l)| apply(l).map(|new| (i, new)))
                .collect();
            let picks: Vec<usize> = match hits.len() {
                0 => vec![],
                1..=3 => (0..hits.len()).collect(),
                n => vec![0, n / 2, n - 1],
            };
            for k in picks {
                let (i, new) = &hits[k];
                let mut copy: Vec<&str> = lines.clone();
                copy[*i] = new;
                out.push(Broken {
                    path: path.to_string_lossy().into_owned(),
                    edit,
                    line: *i as u32,
                    src: copy.join("\n") + "\n",
                });
            }
        }
    }
    out
}

#[derive(Default, Clone, Copy)]
struct Tally {
    files: u32,
    silent: u32,
    errors: u32,
    cascaded: u32,
    first_right: u32,
}

impl Tally {
    fn add(&mut self, other: Tally) {
        self.files += other.files;
        self.silent += other.silent;
        self.errors += other.errors;
        self.cascaded += other.cascaded;
        self.first_right += other.first_right;
    }

    fn row(&self, name: &str) -> String {
        let reported = (self.files - self.silent).max(1);
        format!(
            "{name:<26} {:>5} {:>6} {:>6} {:>8} {:>7.1}%",
            self.files,
            self.silent,
            self.errors,
            self.cascaded,
            100.0 * self.first_right as f64 / reported as f64
        )
    }
}

fn score(b: &Broken) -> Tally {
    let errs = errors(&b.path, &b.src);
    if std::env::var("RECOVERY_SHOW").is_ok_and(|e| e == b.edit) && errs.len() > 1 {
        let index = LineIndex::new(&b.src);
        let lines: Vec<&str> = b.src.lines().collect();
        println!("--- broke line {}: {}", b.line + 1, lines[b.line as usize]);
        for d in &errs {
            let l = index.line_of(d.span.start);
            println!(
                "    {}:{} {}  | {}",
                l + 1,
                d.code.0,
                d.message,
                lines.get(l as usize).unwrap_or(&"")
            );
        }
    }
    let index = LineIndex::new(&b.src);
    let mut t = Tally {
        files: 1,
        ..Tally::default()
    };
    if errs.is_empty() {
        t.silent = 1;
        return t;
    }
    t.errors = errs.len() as u32;
    t.cascaded = t.errors - 1;
    t.first_right = (index.line_of(errs[0].span.start) == b.line) as u32;
    t
}

fn scoreboard_broken_ports() -> Tally {
    let broken = broken_ports();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = broken.len().div_ceil(threads).max(1);
    let per_edit: Vec<BTreeMap<&str, Tally>> = std::thread::scope(|s| {
        let handles: Vec<_> = broken
            .chunks(chunk)
            .map(|part| {
                s.spawn(move || {
                    let mut m: BTreeMap<&str, Tally> = BTreeMap::new();
                    for b in part {
                        m.entry(b.edit).or_default().add(score(b));
                    }
                    m
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut merged: BTreeMap<&str, Tally> = BTreeMap::new();
    for m in per_edit {
        for (k, v) in m {
            merged.entry(k).or_default().add(v);
        }
    }
    println!("\nbroken ports                 files silent errors cascaded first-right");
    let mut total = Tally::default();
    for (edit, t) in &merged {
        println!("  {}", t.row(edit));
        total.add(*t);
    }
    println!("  {}", total.row("all"));
    total
}

/// The untagged fenced blocks of `plan/1x-*.md` that hold cp code.
fn plan_blocks() -> Vec<(String, String)> {
    let mut docs: Vec<PathBuf> = std::fs::read_dir(root().join("plan"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            name.starts_with('1') && name.ends_with(".md")
        })
        .collect();
    docs.sort();
    let mut out = Vec::new();
    for doc in docs {
        let text = std::fs::read_to_string(&doc).unwrap();
        let name = doc.file_name().unwrap().to_string_lossy().to_string();
        let mut block: Option<(usize, String)> = None;
        for (i, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("```") {
                match block.take() {
                    Some((start, body)) => {
                        let code = body.lines().any(|l| {
                            let t = l.trim_start();
                            ["let ", "var ", "fn ", "for ", "print(", "domain "]
                                .iter()
                                .any(|k| t.starts_with(k))
                        });
                        if code && !body.lines().any(|l| l.starts_with("$ ")) {
                            out.push((format!("{name}:{}", start + 1), body));
                        }
                    }
                    None if line.trim() == "```" => block = Some((i, String::new())),
                    None => {}
                }
            } else if let Some((_, body)) = &mut block {
                body.push_str(line);
                body.push('\n');
            }
        }
    }
    out
}

/// Errors in a plan block that name something an earlier error's line
/// declared: `C` unresolved after `let (domain C, part) = ...` failed.
fn cascades_in(src: &str, errs: &[Diagnostic]) -> u32 {
    let index = LineIndex::new(src);
    let lines: Vec<&str> = src.lines().collect();
    let mut bad_lines: Vec<u32> = Vec::new();
    let mut count = 0;
    for d in errs {
        let line = index.line_of(d.span.start);
        let named = d.span.text(src);
        let cascade = !named.is_empty()
            && named.chars().all(|c| c.is_alphanumeric() || c == '_')
            && bad_lines.iter().any(|&l| {
                l < line
                    && lines.get(l as usize).is_some_and(|text| {
                        text.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                            .any(|w| w == named)
                    })
            });
        if cascade {
            count += 1;
        } else {
            bad_lines.push(line);
        }
    }
    count
}

struct PlanTally {
    blocks: u32,
    failing: u32,
    errors: u32,
    cascaded: u32,
    generic_first: u32,
    planned_first: u32,
}

fn scoreboard_plan() -> PlanTally {
    let blocks = plan_blocks();
    let mut t = PlanTally {
        blocks: blocks.len() as u32,
        failing: 0,
        errors: 0,
        cascaded: 0,
        generic_first: 0,
        planned_first: 0,
    };
    for (where_, src) in &blocks {
        let errs = errors(&root().join("plan/snippet.cp").to_string_lossy(), src);
        if errs.is_empty() {
            continue;
        }
        t.failing += 1;
        t.errors += errs.len() as u32;
        t.cascaded += cascades_in(src, &errs);
        let first = &errs[0];
        if std::env::var("RECOVERY_SHOW").is_ok_and(|e| e == "plan") {
            let line = src[..first.span.start as usize].lines().count().max(1);
            let text = src.lines().nth(line - 1).unwrap_or("");
            println!("--- {where_}: {} {}  | {text}", first.code.0, first.message);
        }
        if first.message.starts_with("expected ") {
            t.generic_first += 1;
        }
        if first.message.starts_with("not implemented yet") {
            t.planned_first += 1;
        }
    }
    println!(
        "\nplan/1x-*.md blocks: {} blocks, {} with errors, {} errors, {} cascaded, \
         first error a generic `expected ...` in {}, `not implemented yet` in {}",
        t.blocks, t.failing, t.errors, t.cascaded, t.generic_first, t.planned_first
    );
    t
}

#[test]
fn recovery_scoreboard() {
    let plan = scoreboard_plan();
    let ports = scoreboard_broken_ports();
    assert!(plan.blocks > 100, "the plan code blocks went missing");
    assert!(ports.files > 1000, "the broken ports went missing");
    assert_eq!(ports.silent, 0, "a broken port checked clean");
    let reported = (ports.files - ports.silent) as f64;
    assert!(
        ports.first_right as f64 >= 0.995 * reported,
        "first-error accuracy fell below 99.5%"
    );
    assert!(
        ports.cascaded as f64 <= 0.05 * ports.files as f64,
        "cascaded-error rate rose above 0.05 per broken port ({} / {})",
        ports.cascaded,
        ports.files
    );
}
