//! How well the front end recovers from broken code: one diagnostic per
//! root cause, at the right line.
//!
//! Two corpora. The code blocks of `plan/1x-*.md` are mostly valid 0.2.2
//! with some planned syntax mixed in. The broken ports take every
//! `corpus/sets/ecnerwala.txt` port, which checks clean, and break one line at
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
///
/// Each thread keeps one database, so the stdlib and the unbroken items of
/// a port are checked once rather than once per broken copy, the way the
/// editor sees a stream of edits.
fn errors(path: &str, src: &str) -> Vec<Diagnostic> {
    thread_local! {
        static DB: std::cell::RefCell<cpl_check::Database> = Default::default();
    }
    let analysis = DB.with_borrow_mut(|db| {
        db.set_file(path, src);
        db.analyze(path).expect("the file was just set")
    });
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
        let code = &l[..l.find("//").unwrap_or(l.len())];
        let at = [" + ", " - ", " * ", " < ", " == "]
            .iter()
            .filter_map(|op| code.find(op))
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

/// Mistakes a contestant from C++ or Python makes, each with one right
/// answer: the fix-it benchmark.
const MISTAKES: &[(&str, Edit)] = &[
    ("`&&` for `and`", |l| word_swap(l, " and ", " && ")),
    ("`||` for `or`", |l| word_swap(l, " or ", " || ")),
    ("`!x` for `not x`", |l| word_swap(l, "not ", "!")),
    ("`..<` for `..`", |l| {
        let at = l.match_indices("..").map(|(i, _)| i).find(|&i| {
            !l[i + 2..].starts_with(['=', '.', '<']) && !l[..i].ends_with('.') && !l[..i].ends_with('"')
        })?;
        Some(format!("{}..<{}", &l[..at], &l[at + 2..]))
    }),
    ("`let` for a `var`", |l| {
        let t = l.trim_start();
        let at = l.len() - t.len();
        t.starts_with("var ").then(|| format!("{}let {}", &l[..at], &t[4..]))
    }),
    ("misspell a field or method", |l| {
        let t = l.trim_start();
        if t.starts_with("fn ") || t.starts_with("struct ") || t.starts_with("use ") {
            return None;
        }
        let (at, _) = l.match_indices('.').find(|&(i, _)| {
            let name: String = l[i + 1..].chars().take_while(|c| c.is_ascii_lowercase() || *c == '_').collect();
            name.len() >= 4 && !l[..i].ends_with(['.', ' ', '(', '[']) && !l[..i].ends_with(char::is_numeric)
        })?;
        let end = at + 1 + l[at + 1..].chars().take_while(|c| c.is_ascii_lowercase() || *c == '_').count();
        Some(format!("{}{}", &l[..end - 1], &l[end..]))
    }),
];

/// `from` in code (not in a string or comment) replaced by `to`, once.
fn word_swap(l: &str, from: &str, to: &str) -> Option<String> {
    let code = l.split("//").next().unwrap_or(l);
    let at = code.find(from)?;
    if code[..at].matches('"').count() % 2 == 1 {
        return None;
    }
    if from == "not " && at > 0 && !code[..at].ends_with([' ', '(', '[']) {
        return None;
    }
    Some(format!("{}{to}{}", &l[..at], &l[at + from.len()..]))
}

struct Broken {
    path: String,
    edit: &'static str,
    line: u32,
    src: String,
    original: String,
}

/// Every port broken by every edit on up to three of its lines, picked
/// the same way on every run.
fn broken_ports(edits: &'static [(&'static str, Edit)]) -> Vec<Broken> {
    let set = std::fs::read_to_string(root().join("corpus/sets/ecnerwala.txt")).unwrap();
    let mut paths: Vec<PathBuf> = set
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|id| root().join(format!("corpus/problems/{id}/port.cp")))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let src = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = src.lines().collect();
        for (edit, apply) in edits {
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
                    original: src.clone(),
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
    /// Errors that carry a fix.
    fixable: u32,
    /// Files that applying the fixes turns back into the port.
    restored: u32,
}

impl Tally {
    fn add(&mut self, other: Tally) {
        self.files += other.files;
        self.silent += other.silent;
        self.errors += other.errors;
        self.cascaded += other.cascaded;
        self.first_right += other.first_right;
        self.fixable += other.fixable;
        self.restored += other.restored;
    }

    fn row(&self, name: &str, restore: bool) -> String {
        let reported = (self.files - self.silent).max(1);
        format!(
            "{name:<28} {:>5} {:>6} {:>6} {:>8} {:>7.1}% {:>6.1}% {:>8}",
            self.files,
            self.silent,
            self.errors,
            self.cascaded,
            100.0 * self.first_right as f64 / reported as f64,
            100.0 * self.fixable as f64 / self.errors.max(1) as f64,
            if restore {
                format!("{:.1}%", 100.0 * self.restored as f64 / self.files as f64)
            } else {
                "-".into()
            }
        )
    }
}

/// The first error of a broken port and what its first fix does.
#[derive(Default, Clone, Copy)]
struct FirstFix {
    count: u32,
    /// It carries a fix.
    fixed: u32,
    /// Applying the fix gives back the port's own line.
    restored: u32,
    /// Applying the fix leaves no error at all.
    clean: u32,
}

impl FirstFix {
    fn add(&mut self, other: FirstFix) {
        self.count += other.count;
        self.fixed += other.fixed;
        self.restored += other.restored;
        self.clean += other.clean;
    }
}

fn apply(src: &str, edits: &[(cpl_diag::Span, String)]) -> String {
    let mut edits: Vec<_> = edits.iter().collect();
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    let mut out = src.to_string();
    for (span, text) in edits {
        out.replace_range(span.start as usize..span.end as usize, text);
    }
    out
}

/// How `b` scores, and the code of its first error with what its fix does.
fn score(b: &Broken, restore: bool) -> (Tally, Option<(String, FirstFix)>) {
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
        return (t, None);
    }
    t.errors = errs.len() as u32;
    t.cascaded = t.errors - 1;
    t.first_right = (index.line_of(errs[0].span.start) == b.line) as u32;
    let mut first = FirstFix {
        count: 1,
        ..FirstFix::default()
    };
    if let Some(fix) = errs[0].fixes.first() {
        let fixed = apply(&b.src, &fix.edits);
        first.fixed = 1;
        let line = |src: &str| src.lines().nth(b.line as usize).map(str::to_string);
        first.restored = (line(&fixed) == line(&b.original)) as u32;
        if !restore {
            first.clean = errors(&b.path, &fixed).is_empty() as u32;
        }
    }
    t.fixable = errs.iter().filter(|d| !d.fixes.is_empty()).count() as u32;
    if !restore {
        return (t, Some((errs[0].code.0.to_string(), first)));
    }
    let fixed = (t.fixable > 0).then(|| {
        let (once, _) = cpl_lint::apply_fixes(&b.src, &errs);
        cpl_cli::apply_error_fixes(&b.path, &once).0
    });
    t.restored = fixed.as_deref().is_some_and(|f| f == b.original) as u32;
    if std::env::var("RECOVERY_SHOW").is_ok_and(|e| e == "unrestored") && t.restored == 0 {
        let at = |s: &str| s.lines().nth(b.line as usize).unwrap_or("").to_string();
        let got = fixed.as_deref().map_or("no fix".to_string(), at);
        println!("--- {} {}: {}\n    {}\n    got {got}", b.path, b.edit, at(&b.original), errs[0].message);
    }
    (t, Some((errs[0].code.0.to_string(), first)))
}

/// With `restore`, also applies every fix as `cpl fix` would and counts
/// the copies that turn back into the port.
fn scoreboard_broken_ports(title: &str, edits: &'static [(&'static str, Edit)], restore: bool) -> Tally {
    let broken = broken_ports(edits);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let ports: Vec<&[Broken]> = broken.chunk_by(|a, b| a.path == b.path).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    type Part<'a> = (BTreeMap<&'a str, Tally>, BTreeMap<String, FirstFix>);
    let per_edit: Vec<Part> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut m: BTreeMap<&str, Tally> = BTreeMap::new();
                    let mut codes: BTreeMap<String, FirstFix> = BTreeMap::new();
                    let ordering = std::sync::atomic::Ordering::Relaxed;
                    while let Some(port) = ports.get(next.fetch_add(1, ordering)) {
                        for b in *port {
                            let (tally, first) = score(b, restore);
                            m.entry(b.edit).or_default().add(tally);
                            if let Some((code, fix)) = first {
                                codes.entry(code).or_default().add(fix);
                            }
                        }
                    }
                    (m, codes)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut merged: BTreeMap<&str, Tally> = BTreeMap::new();
    let mut codes: BTreeMap<String, FirstFix> = BTreeMap::new();
    for (m, part_codes) in per_edit {
        for (k, v) in m {
            merged.entry(k).or_default().add(v);
        }
        for (code, fix) in part_codes {
            codes.entry(code).or_default().add(fix);
        }
    }
    println!("\n{title:<30} files silent errors cascaded first-right fix-it restored");
    let mut total = Tally::default();
    for (edit, t) in &merged {
        println!("  {}", t.row(edit, restore));
        total.add(*t);
    }
    println!("  {}", total.row("all", restore));
    let mut by_count: Vec<(String, FirstFix)> = codes.into_iter().collect();
    by_count.sort_by_key(|(code, f)| (std::cmp::Reverse(f.count), code.clone()));
    println!("\nfirst error by code          count  with a fix  restores  clean");
    for (code, f) in &by_count {
        println!(
            "  {code:<26} {:>5} {:>11} {:>9} {:>6}",
            f.count, f.fixed, f.restored, f.clean
        );
    }
    total
}

/// The fenced cp blocks of `spec/core/*.md` and `spec/lib/*.md`.
fn plan_blocks() -> Vec<(String, String)> {
    let mut docs: Vec<PathBuf> = Vec::new();
    for dir in ["spec/core", "spec/lib", "spec/lib/rationale", "spec/tools"] {
        if let Ok(entries) = std::fs::read_dir(root().join(dir)) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("md") {
                    docs.push(p);
                }
            }
        }
    }
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
                    None if matches!(line.trim(), "```" | "```cp") => block = Some((i, String::new())),
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
        "\nspec/{{core,lib}}/*.md blocks: {} blocks, {} with errors, {} errors, {} cascaded, \
         first error a generic `expected ...` in {}, `not implemented yet` in {}",
        t.blocks, t.failing, t.errors, t.cascaded, t.generic_first, t.planned_first
    );
    t
}

#[test]
fn recovery_scoreboard() {
    let plan = scoreboard_plan();
    let ports = scoreboard_broken_ports("broken ports", EDITS, false);
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

/// Each common mistake carries a fix, and applying the fixes gives back
/// the program the mistake was made in.
#[test]
fn fix_it_scoreboard() {
    let ports = scoreboard_broken_ports("common mistakes", MISTAKES, true);
    assert!(ports.files > 500, "the mistake copies went missing");
    let reported = (ports.files - ports.silent) as f64;
    assert!(
        ports.fixable as f64 >= 0.88 * ports.errors as f64,
        "fewer than 88% of the errors from common mistakes carry a fix ({} / {})",
        ports.fixable,
        ports.errors
    );
    assert!(
        ports.restored as f64 >= 0.95 * reported,
        "fixes restored fewer than 95% of the copies with a mistake ({} / {reported})",
        ports.restored
    );
}
