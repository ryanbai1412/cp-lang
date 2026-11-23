//! Mistakes a first-time user brings from C++ and Python, on the site's
//! practice problems (`web/content/practice/*/ref.cp`).
//!
//! Each habit is applied to up to three lines of each reference solution.
//! A copy scores when its first error is on the changed line, when that
//! error carries a fix, and when `cpl fix` turns the copy into a program
//! that passes every test in the problem's `tests.json`.
//!
//! `cargo test -p cpl-cli --test newcomer -- --nocapture` prints the
//! scoreboard; `NEWCOMER_SHOW=<habit>` prints the copies that miss.

use cpl_diag::{Diagnostic, LineIndex, Severity};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

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

/// A habit rewrites the whole program around line `i`, or is `None` when
/// it does not apply there.
type Habit = fn(&[&str], usize) -> Option<Vec<String>>;

fn indent(l: &str) -> usize {
    l.len() - l.trim_start().len()
}

fn code(l: &str) -> &str {
    l.split("//").next().unwrap_or(l)
}

fn replace_line(lines: &[&str], i: usize, new: String) -> Vec<String> {
    let mut out: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    out[i] = new;
    out
}

/// `from` in code (not in a string) replaced by `to`, once.
fn swap(lines: &[&str], i: usize, from: &str, to: &str) -> Option<Vec<String>> {
    let l = lines[i];
    let at = code(l).find(from)?;
    if l[..at].matches('"').count() % 2 == 1 {
        return None;
    }
    Some(replace_line(
        lines,
        i,
        format!("{}{to}{}", &l[..at], &l[at + from.len()..]),
    ))
}

fn header(l: &str) -> bool {
    let t = l.trim_start();
    ["if ", "for ", "while ", "elif ", "else:"]
        .iter()
        .any(|k| t.starts_with(k))
        && code(l).trim_end().ends_with(':')
}

const HABITS: &[(&str, Habit)] = &[
    ("`&&` for `and`", |ls, i| swap(ls, i, " and ", " && ")),
    ("`||` for `or`", |ls, i| swap(ls, i, " or ", " || ")),
    ("`!x` for `not x`", |ls, i| {
        let l = code(ls[i]);
        let at = l.find("not ")?;
        (at == 0 || l[..at].ends_with([' ', '(', '['])).then_some(())?;
        swap(ls, i, "not ", "!")
    }),
    ("`x++` for `x += 1`", |ls, i| {
        let l = code(ls[i]).trim_end();
        let name = l.trim_start().strip_suffix(" += 1")?;
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '_')
            .then_some(())?;
        Some(replace_line(
            ls,
            i,
            format!("{}{name}++", &ls[i][..indent(ls[i])]),
        ))
    }),
    ("`;` after a statement", |ls, i| {
        let l = ls[i].trim_end();
        (!l.is_empty() && !l.ends_with(':') && !l.contains("//")).then_some(())?;
        (i + 1 == ls.len() || indent(ls[i + 1]) <= indent(l)).then_some(())?;
        Some(replace_line(ls, i, format!("{l};")))
    }),
    ("`{ }` for a block", |ls, i| {
        let l = ls[i];
        header(l).then_some(())?;
        (!l.trim_start().starts_with("else") && !l.trim_start().starts_with("elif"))
            .then_some(())?;
        let w = indent(l);
        let mut end = i + 1;
        while end < ls.len() && (ls[end].trim().is_empty() || indent(ls[end]) > w) {
            end += 1;
        }
        (end > i + 1).then_some(())?;
        // An `elif`/`else` after the block keeps this a chain; skip it.
        let next = ls.get(end).map_or("", |n| n.trim_start());
        (!next.starts_with("elif") && !next.starts_with("else")).then_some(())?;
        let head = code(l).trim_end().strip_suffix(':')?;
        let mut out = replace_line(ls, i, format!("{head} {{"));
        out.insert(end, format!("{}}}", &l[..w]));
        Some(out)
    }),
    ("`else if` for `elif`", |ls, i| {
        ls[i].trim_start().starts_with("elif ").then_some(())?;
        swap(ls, i, "elif ", "else if ")
    }),
    ("C `for (...)` loop", |ls, i| {
        let l = ls[i];
        let t = l.trim_start().strip_prefix("for ")?.strip_suffix(':')?;
        let (v, r) = t.split_once(" in ")?;
        let (lo, hi) = r.split_once("..")?;
        let ok = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_');
        (ok(v) && ok(lo) && ok(hi)).then_some(())?;
        let w = &l[..indent(l)];
        Some(replace_line(
            ls,
            i,
            format!("{w}for (int {v} = {lo}; {v} < {hi}; {v}++):"),
        ))
    }),
    ("`.size()` for `.len`", |ls, i| {
        let l = code(ls[i]);
        let at = l.find(".len")?;
        (!l[at + 4..].starts_with(|c: char| c.is_alphanumeric() || c == '_')).then_some(())?;
        swap(ls, i, ".len", ".size()")
    }),
    ("`.push_back` for `.push`", |ls, i| {
        swap(ls, i, ".push(", ".push_back(")
    }),
    ("`//` for `/`", |ls, i| swap(ls, i, " / ", " // ")),
    ("`int` for `var`", |ls, i| {
        ls[i].trim_start().starts_with("var ").then_some(())?;
        swap(ls, i, "var ", "int ")
    }),
    ("`auto` for `let`", |ls, i| {
        ls[i].trim_start().starts_with("let ").then_some(())?;
        swap(ls, i, "let ", "auto ")
    }),
];

struct Case {
    habit: &'static str,
    problem: String,
    line: usize,
    src: String,
}

fn tests_of(json: &str) -> Vec<(String, String)> {
    let strings = |key: &str| -> Vec<String> {
        let mut out = Vec::new();
        let pat = format!("\"{key}\": \"");
        let mut rest = json;
        while let Some(at) = rest.find(&pat) {
            let mut s = String::new();
            let mut chars = rest[at + pat.len()..].chars();
            while let Some(c) = chars.next() {
                match c {
                    '"' => break,
                    '\\' => match chars.next() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some(c) => s.push(c),
                        None => {}
                    },
                    c => s.push(c),
                }
            }
            out.push(s);
            rest = chars.as_str();
        }
        out
    };
    strings("input")
        .into_iter()
        .zip(strings("expected"))
        .collect()
}

fn passes(src: &str, tests: &[(String, String)], n: usize) -> bool {
    let path = std::env::temp_dir().join(format!("cpl-newcomer-{}-{n}.cp", std::process::id()));
    std::fs::write(&path, src).unwrap();
    let ok = tests.iter().all(|(input, expected)| {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .arg("run")
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        out.status.success() && String::from_utf8_lossy(&out.stdout) == *expected
    });
    let _ = std::fs::remove_file(&path);
    ok
}

#[derive(Default, Clone, Copy)]
struct Tally {
    copies: u32,
    silent: u32,
    first_right: u32,
    fixable: u32,
    runs: u32,
    errors: u32,
}

#[test]
fn newcomer_scoreboard() {
    let dir = root().join("web/content/practice");
    let mut problems: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    problems.sort();
    let mut cases = Vec::new();
    let mut tests = BTreeMap::new();
    for p in &problems {
        let Ok(original) = std::fs::read_to_string(p.join("ref.cp")) else {
            continue;
        };
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        tests.insert(
            name.clone(),
            (
                original.clone(),
                tests_of(&std::fs::read_to_string(p.join("tests.json")).unwrap()),
            ),
        );
        let lines: Vec<&str> = original.lines().collect();
        for &(habit, edit) in HABITS {
            let mut taken = 0;
            for i in 0..lines.len() {
                if taken == 3 {
                    break;
                }
                if let Some(out) = edit(&lines, i) {
                    taken += 1;
                    cases.push(Case {
                        habit,
                        problem: name.clone(),
                        line: i,
                        src: out.join("\n") + "\n",
                    });
                }
            }
        }
    }
    let show = std::env::var("NEWCOMER_SHOW").ok();
    let mut by_habit: BTreeMap<&str, Tally> = BTreeMap::new();
    let mut crlf_differs = Vec::new();
    for (n, c) in cases.iter().enumerate() {
        let (original, tests) = &tests[&c.problem];
        let path = dir
            .join(&c.problem)
            .join("ref.cp")
            .to_string_lossy()
            .to_string();
        let errs = errors(&path, &c.src);
        let t = by_habit.entry(c.habit).or_default();
        t.copies += 1;
        if errs.is_empty() {
            t.silent += 1;
            continue;
        }
        t.errors += errs.len() as u32;
        let index = LineIndex::new(&c.src);
        let on = |s: cpl_diag::Span| index.line_of(s.start) as usize == c.line;
        let right = on(errs[0].span) || errs[0].labels.iter().any(|l| on(l.span));
        t.first_right += right as u32;
        t.fixable += !errs[0].fixes.is_empty() as u32;
        let (fixed, _) = cpl_cli::apply_error_fixes(&path, &c.src);
        // Windows: the same copy with CRLF line ends gets the same fixes.
        let (crlf, _) = cpl_cli::apply_error_fixes(&path, &c.src.replace('\n', "\r\n"));
        if crlf.replace("\r\n", "\n") != fixed {
            crlf_differs.push(format!("{} {}: {}", c.problem, c.line + 1, c.habit));
        }
        let runs =
            fixed == *original || (errors(&path, &fixed).is_empty() && passes(&fixed, tests, n));
        t.runs += runs as u32;
        if show.as_deref() == Some(c.habit) && !(right && runs) {
            let line = |s: &str| s.lines().nth(c.line).unwrap_or("").to_string();
            println!(
                "--- {} {}: {}\n    {} {}\n    got {}",
                c.problem,
                c.line + 1,
                line(&c.src),
                errs[0].code.0,
                errs[0].message,
                line(&fixed)
            );
        }
    }
    let mut all = Tally::default();
    println!(
        "{:<26} {:>6} {:>6} {:>7} {:>7} {:>6} {:>6}",
        "habit", "copies", "silent", "1st ok", "fixable", "runs", "errors"
    );
    for (h, t) in &by_habit {
        println!(
            "{h:<26} {:>6} {:>6} {:>7} {:>7} {:>6} {:>6}",
            t.copies, t.silent, t.first_right, t.fixable, t.runs, t.errors
        );
        all.copies += t.copies;
        all.silent += t.silent;
        all.first_right += t.first_right;
        all.fixable += t.fixable;
        all.runs += t.runs;
        all.errors += t.errors;
    }
    let pct = |a: u32| 100.0 * a as f64 / (all.copies - all.silent).max(1) as f64;
    println!(
        "{:<26} {:>6} {:>6} {:>6.1}% {:>6.1}% {:>5.1}% {:>6}",
        "all",
        all.copies,
        all.silent,
        pct(all.first_right),
        pct(all.fixable),
        pct(all.runs),
        all.errors
    );
    // Bars at the measured values; raise them as fixes land.
    assert!(
        all.copies >= 173,
        "the newcomer copies went missing ({})",
        all.copies
    );
    assert!(
        all.silent <= 1,
        "{} copies with a mistake checked clean",
        all.silent
    );
    assert!(
        pct(all.first_right) >= 99.0,
        "first error off the changed line ({:.1}%)",
        pct(all.first_right)
    );
    assert!(
        pct(all.runs) >= 98.0,
        "fewer than 98% of copies run right after `cpl fix` ({:.1}%)",
        pct(all.runs)
    );
    assert!(
        crlf_differs.is_empty(),
        "CRLF copies fixed differently: {crlf_differs:#?}"
    );
}
