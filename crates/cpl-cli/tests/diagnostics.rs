//! Golden diagnostics: `cpl check` on each `tests/diagnostics/*.cp`, and on
//! each planned-syntax example, must print exactly the `.err` beside it.
//! `BLESS=1 cargo test -p cpl-cli --test diagnostics` rewrites them.
//! Golden fixes: `cpl fix` on each `tests/fixes/*.cp` must give the
//! `.fixed` beside it, and that must check clean.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "support/parallel.rs"]
mod parallel;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/diagnostics")
}

fn rendered(file: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(file.file_name().unwrap())
        .current_dir(file.parent().unwrap())
        .env("NO_COLOR", "1")
        .output()
        .expect("cpl starts");
    mask_std_lines(&String::from_utf8_lossy(&out.stderr))
}

/// `std/strings.cp:1121` becomes `std/strings.cp:_`: a golden must not break on every std edit.
fn mask_std_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("std/") {
        let after = &rest[at..];
        let Some(end) = after.find(".cp:") else { break };
        let path = &after[..end];
        let digits = after[end + 4..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits == 0 || path.contains(char::is_whitespace) {
            out.push_str(&rest[..at + 4]);
            rest = &rest[at + 4..];
            continue;
        }
        out.push_str(&rest[..at + end + 4]);
        out.push('_');
        rest = &after[end + 4 + digits..];
        if let Some(col) = rest.strip_prefix(':') {
            let n = col.bytes().take_while(u8::is_ascii_digit).count();
            if n > 0 {
                out.push_str(":_");
                rest = &col[n..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn planned_name(feature: &str) -> String {
    let words: Vec<String> = feature
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();
    format!("planned_{}", words.join("_"))
}

#[test]
fn golden_diagnostics() {
    let bless = std::env::var_os("BLESS").is_some();
    for planned in cpl_syntax::planned::PLANNED {
        let path = dir().join(format!("{}.cp", planned_name(planned.feature)));
        if bless || !path.exists() {
            fs::write(&path, planned.example).unwrap();
        }
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            planned.example,
            "{} is generated from `PLANNED`; rerun with BLESS=1",
            path.display()
        );
    }
    let mut sources: Vec<PathBuf> = fs::read_dir(dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    sources.sort();
    for src in &sources {
        let stem = src.file_stem().unwrap().to_string_lossy();
        assert!(
            !stem.starts_with("planned_")
                || cpl_syntax::planned::PLANNED
                    .iter()
                    .any(|p| planned_name(p.feature) == stem),
            "{} is not in `PLANNED` any more; delete it and its .err",
            src.display()
        );
    }
    let mut wrong = Vec::new();
    for src in &sources {
        let got = rendered(src);
        let golden = src.with_extension("err");
        if bless {
            fs::write(&golden, &got).unwrap();
            continue;
        }
        let want = fs::read_to_string(&golden).unwrap_or_default();
        if got != want {
            wrong.push(format!("--- {}\n{got}", golden.display()));
        }
    }
    assert!(
        wrong.is_empty(),
        "rerun with BLESS=1 if intended:\n{}",
        wrong.join("\n")
    );
}

#[test]
fn every_planned_feature_cites_a_section_of_the_design() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for planned in cpl_syntax::planned::PLANNED {
        let Some(see) = planned.see else { continue };
        let (path, section) = see.split_once(" §").expect("`<file> §<section>`");
        let doc = fs::read_to_string(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(
            doc.lines()
                .any(|l| l.starts_with('#') && format!("{l} ").contains(&format!(" {section} "))),
            "{path} has no section {section}"
        );
    }
}

#[test]
fn golden_fixes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixes");
    let mut sources: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    sources.sort();
    assert!(!sources.is_empty());
    for src in &sources {
        let path = src.to_string_lossy();
        let (fixed, applied) = cpl_cli::apply_error_fixes(&path, &fs::read_to_string(src).unwrap());
        assert!(applied > 0, "{path}: no fix applied");
        assert_eq!(
            fixed,
            fs::read_to_string(src.with_extension("fixed")).unwrap(),
            "{path}"
        );
        let (_, again) = cpl_cli::apply_error_fixes(&path, &fixed);
        assert_eq!(again, 0, "{path}: the fixed file still has a fix to apply");
    }
}

/// Every fix a golden diagnostic offers, applied, leaves source that parses
/// with no syntax error it didn't already have: no fix text is a guess.
#[test]
fn every_golden_fix_parses_once_applied() {
    let syntax_errors = |text: &str| -> Vec<String> {
        (cpl_syntax::parse_file(text).diagnostics.into_iter())
            .filter(|d| d.severity == cpl_diag::Severity::Error)
            .map(|d| d.message)
            .collect()
    };
    let mut sources: Vec<PathBuf> = fs::read_dir(dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    sources.sort();
    let mut wrong = Vec::new();
    for src in &sources {
        let path = src.to_string_lossy();
        let text = fs::read_to_string(src).unwrap();
        let (fixed, applied) = cpl_cli::apply_error_fixes(&path, &text);
        if applied == 0 {
            continue;
        }
        let before = syntax_errors(&text);
        let new: Vec<String> = (syntax_errors(&fixed).into_iter())
            .filter(|m| !before.contains(m))
            .collect();
        if !new.is_empty() {
            wrong.push(format!("{path}: {}", new.join("; ")));
        }
    }
    assert!(
        wrong.is_empty(),
        "golden fixes that don't parse:\n{}",
        wrong.join("\n")
    );
}

/// The spellings `spec/GLOSSARY.md` rules out (its "Never" column), found in
/// `text`. A backticked term is code: a `quoted` span that is the term, or
/// for punctuation starts with it and a space (`&mut x`). A plain term is prose, outside the
/// quotes.
/// Terms the glossary qualifies with a parenthesis need a reader to judge.
fn off_glossary(text: &str) -> Option<String> {
    static TERMS: std::sync::OnceLock<Vec<(String, bool)>> = std::sync::OnceLock::new();
    let terms = TERMS.get_or_init(|| {
        let glossary = include_str!("../../../spec/GLOSSARY.md");
        let mut out = Vec::new();
        for row in glossary.lines().filter(|l| l.starts_with("| ")).skip(1) {
            let never = row
                .trim()
                .trim_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or("");
            for t in never.split([',', ';']).map(str::trim) {
                let code = t.len() > 2 && t.starts_with('`') && t.ends_with('`');
                if t.is_empty() || t.contains('(') || (!code && t.contains('`')) {
                    continue;
                }
                out.push((t.trim_matches('`').to_string(), code));
            }
        }
        out
    });
    let pieces: Vec<&str> = text.split('`').collect();
    for (term, code) in terms {
        let hit = pieces.iter().enumerate().any(|(i, piece)| {
            let quoted = i % 2 == 1;
            if *code {
                let piece = piece.trim();
                let word = term.chars().all(char::is_alphanumeric);
                quoted
                    && (piece == term
                        || !word
                            && piece
                                .strip_prefix(term.as_str())
                                .is_some_and(|r| r.starts_with(' ')))
            } else {
                !quoted && {
                    let lower = piece.to_lowercase();
                    lower.match_indices(term.as_str()).any(|(at, _)| {
                        let edge =
                            |c: Option<char>| !c.is_some_and(|c| c.is_alphanumeric() || c == '-');
                        edge(lower[..at].chars().next_back())
                            && edge(lower[at + term.len()..].chars().next())
                    })
                }
            }
        });
        if hit {
            return Some(term.clone());
        }
    }
    cpl_diag::old_option_spelling_in_prose(text).map(str::to_string)
}

/// Errors, help, notes, `cpl explain` pages and lint records use the
/// glossary's terms. The quoted source lines of a golden may hold anything.
/// Rule: `errors/R5`, handbook/guidelines/errors-checklist.md#r5-name-what-the-user-wrote-in-the-specs-words.
#[test]
fn messages_use_the_glossary() {
    let mut bad = Vec::new();
    for entry in fs::read_dir(dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "err") {
            continue;
        }
        for line in fs::read_to_string(&path).unwrap().lines() {
            let t = line.trim_start();
            let message = ["error", "warning", "= help", "= note"]
                .iter()
                .any(|p| t.starts_with(p));
            if let (true, Some(term)) = (message, off_glossary(t)) {
                bad.push(format!("{}: `{term}`: {line}", path.display()));
            }
        }
    }
    for info in cpl_diag::REGISTRY {
        for text in [info.title, info.summary, info.explanation] {
            if let Some(term) = off_glossary(text) {
                bad.push(format!("{} in the registry: `{term}`: {text}", info.code.0));
            }
        }
    }
    let spec = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec");
    for kind in ["lints", "checker-lints"] {
        for entry in fs::read_dir(spec.join(kind)).unwrap() {
            let path = entry.unwrap().path().join("lint.toml");
            let Ok(record) = fs::read_to_string(&path) else {
                continue;
            };
            for line in record.lines() {
                let prose = ["title", "message", "help", "note"]
                    .iter()
                    .any(|k| line.starts_with(k));
                if let (true, Some(term)) = (prose, off_glossary(line)) {
                    bad.push(format!("{}: `{term}`: {line}", path.display()));
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "spellings spec/GLOSSARY.md rules out:\n{}",
        bad.join("\n")
    );
}

/// Every fix a golden error offers names something that exists: after
/// `cpl fix`, no new unknown or private name (E0201, E0203, E0207) appears.
#[test]
fn every_golden_fix_names_something_that_exists() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("golden-fixes-exist");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let cpl = |args: &[&str]| String::from_utf8_lossy(&cpl_cli::run(args).stdout).to_string();
    let unknown = |json: &str| {
        ["E0201", "E0203", "E0207"]
            .iter()
            .map(|c| json.matches(&format!("\"code\":\"{c}\"")).count())
            .sum::<usize>()
    };
    let mut files: Vec<PathBuf> = fs::read_dir(dir())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    files.sort();
    // In process on every core: a `cpl` per golden and step was most of the time.
    let each = parallel::parallel(&files, |file| {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        let copy = scratch.join(&name);
        fs::copy(file, &copy).unwrap();
        let copy = copy.to_str().unwrap();
        let before = unknown(&cpl(&["check", "--json", copy]));
        if cpl(&["fix", "--json", copy]).contains("\"applied\":0,") {
            return None;
        }
        let after = cpl(&["check", "--json", copy]);
        if unknown(&after) > before {
            let text = fs::read_to_string(copy).unwrap();
            return Some(Some(format!("{name}: after `cpl fix`\n{text}\n{after}")));
        }
        Some(None)
    });
    let fixed = each.iter().flatten().count();
    let wrong: Vec<String> = each.into_iter().flatten().flatten().collect();
    assert!(
        fixed >= 20,
        "only {fixed} goldens checked; the scan is broken"
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

/// An error points at the user's code, never into `std/`: a library gap is
/// shown at the user's call, with the std line in a note.
/// Rule: `errors/R3`, handbook/guidelines/errors-checklist.md#r3-never-a-primary-span-inside-std.
#[test]
fn no_golden_error_points_into_std() {
    let mut inside = Vec::new();
    for entry in fs::read_dir(dir()).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "err") {
            continue;
        }
        for line in fs::read_to_string(&path).unwrap().lines() {
            if let Some(at) = line.trim_start().strip_prefix("--> ") {
                if at.starts_with("std/") || at.contains("/std/") {
                    inside.push(format!("{}: {line}", path.display()));
                }
            }
        }
    }
    assert!(
        inside.is_empty(),
        "primary span inside std:\n{}",
        inside.join("\n")
    );
}

/// Real mistakes, each `tests/mistakes/<name>.cp` with `// here: N` on the
/// line to edit. Graded on three things: the first error points at line
/// `N`, it says what to do (`help`), and `cpl fix` leaves a program that
/// checks clean, unless line 2 starts `// no fix` because no one edit is
/// right. This is the register row "Real mistakes get errors that name the
/// place, the rule and a working fix" in `handbook/domains/diagnostics.md`.
/// Rule: `errors/R15`, handbook/guidelines/errors-checklist.md#r15-every-mistake-a-user-hits-becomes-a-case; `errors/R2`, handbook/guidelines/errors-checklist.md#r2-the-primary-span-is-the-token-to-edit; `errors/R8`, handbook/guidelines/errors-checklist.md#r8-give-a-fix-or-say-what-to-do.
#[test]
fn real_mistakes_name_the_place_the_rule_and_a_fix() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("real-mistakes");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let cpl = |args: &[&str]| {
        let out = cpl_cli::run(args);
        out.stderr + &String::from_utf8_lossy(&out.stdout)
    };
    let mistakes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/mistakes");
    let mut paths: Vec<PathBuf> = (fs::read_dir(mistakes).unwrap())
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    // In process on every core: a `cpl` per mistake and step was most of the time.
    let wrong: Vec<String> = parallel::parallel(&paths, |path| {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let src = fs::read_to_string(path).unwrap();
        let here: u32 = src
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("// here: "))
            .and_then(|n| n.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name}: first line must be `// here: N`"));
        let copy = scratch.join(format!("{name}.cp"));
        fs::write(&copy, &src).unwrap();
        let file = copy.to_str().unwrap();
        let shown = cpl(&["check", file]);
        let at = shown
            .lines()
            .find_map(|l| l.trim_start().strip_prefix("--> "))
            .and_then(|l| l.rsplit(':').nth(1))
            .and_then(|n| n.parse::<u32>().ok());
        let mut misses = Vec::new();
        if at != Some(here) {
            misses.push(format!("points at line {at:?}, not {here}"));
        }
        if !shown.contains("= help") {
            misses.push("no help".to_string());
        }
        let no_fix = src
            .lines()
            .nth(1)
            .is_some_and(|l| l.starts_with("// no fix"));
        cpl(&["fix", file]);
        if !no_fix && !cpl(&["check", file]).contains("no errors") {
            misses.push("`cpl fix` leaves errors".to_string());
        }
        (!misses.is_empty()).then(|| format!("{name}: {}\n{shown}", misses.join(", ")))
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Whether `text` is `template` with each `{}` hole filled.
fn says(template: &str, text: &str) -> bool {
    let shown = template.replace("{{", "{").replace("}}", "}");
    let mut parts = shown.split("{}");
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = text.strip_prefix(first) else {
        return false;
    };
    let parts: Vec<&str> = parts.collect();
    for (i, part) in parts.iter().enumerate() {
        if i + 1 == parts.len() {
            return rest.len() > part.len() && rest.ends_with(part);
        }
        match rest.get(1..).and_then(|r| r.find(part)) {
            Some(at) => rest = &rest[1 + at + part.len()..],
            None => return false,
        }
    }
    rest.is_empty()
}

/// Every `cpl_diag::habits` row, run on its mistake program
/// `tests/habits/<key>.cp`: an error of the row's code on the line marked
/// `// <- here` (or named by a `// here: N` line) gives the row's help or fix title, and `cpl fix` leaves the
/// mistake checking unless the row says why no one edit is right.
#[test]
fn every_habit_row_gives_its_help_where_its_mistake_is() {
    let rows = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/habits");
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("habit-rows");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let cpl = |args: &[&str]| String::from_utf8_lossy(&cpl_cli::run(args).stdout).to_string();
    let all: Vec<_> = cpl_diag::habits::all().collect();
    let mut keys = std::collections::BTreeSet::new();
    for row in &all {
        assert!(keys.insert(row.key), "two rows are keyed `{}`", row.key);
    }
    // In process on every core: a `cpl` per row and step was most of the time.
    let mut wrong: Vec<String> = parallel::parallel(&all, |row| {
        let name = format!("{}.cp", row.key);
        let Ok(mistake) = fs::read_to_string(rows.join(&name)) else {
            return Some(format!("{}: no tests/habits/{name}", row.key));
        };
        let copy = scratch.join(&name);
        fs::write(&copy, &mistake).unwrap();
        let file = copy.to_str().unwrap();
        let here = (mistake.lines().find_map(|l| l.strip_prefix("// here: ")))
            .and_then(|n| n.trim().parse().ok())
            .or_else(|| {
                mistake
                    .lines()
                    .position(|l| l.ends_with("// <- here"))
                    .map(|i| i as i64 + 1)
            });
        let shown = cpl_json::parse(&cpl(&["check", "--json", file])).expect("cpl check --json");
        let diags = shown.at(&["files"]).as_array().unwrap_or_default()[0].at(&["diagnostics"]);
        let gives = diags.as_array().unwrap_or_default().iter().any(|d| {
            let texts = std::iter::once(d.at(&["help"])).chain(
                d.at(&["fixes"])
                    .as_array()
                    .unwrap_or_default()
                    .iter()
                    .map(|f| f.at(&["title"])),
            );
            d.at(&["code"]).as_str() == Some(row.code.0)
                && d.at(&["start", "line"]).as_i64() == here
                && texts.filter_map(|t| t.as_str()).any(|t| says(row.help, t))
        });
        let mut misses = Vec::new();
        if !gives {
            misses.push(format!(
                "no {} with the row's text on the `// <- here` line",
                row.code.0
            ));
        }
        cpl(&["fix", file]);
        let fixed = cpl_json::parse(&cpl(&["check", "--json", file])).is_ok_and(|v| {
            v.at(&["files"]).as_array().unwrap_or_default()[0]
                .at(&["ok"])
                .as_bool()
                == Some(true)
        });
        if row.no_fix.is_empty() && !fixed {
            misses.push("`cpl fix` leaves errors".to_string());
        }
        if !row.no_fix.is_empty() && fixed {
            misses.push("`cpl fix` mends it, so drop `no_fix`".to_string());
        }
        (!misses.is_empty()).then(|| format!("{}: {}", row.key, misses.join(", ")))
    })
    .into_iter()
    .flatten()
    .collect();
    for entry in fs::read_dir(&rows).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().to_string();
        if !keys.contains(name.trim_end_matches(".cp")) {
            wrong.push(format!("tests/habits/{name} has no row"));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} row(s) wrong:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
