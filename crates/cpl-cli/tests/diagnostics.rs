//! Golden diagnostics: `cpl check` on each `tests/diagnostics/*.cp`, and on
//! each planned-syntax example, must print exactly the `.err` beside it.
//! `BLESS=1 cargo test -p cpl-cli --test diagnostics` rewrites them.
//! Golden fixes: `cpl fix` on each `tests/fixes/*.cp` must give the
//! `.fixed` beside it, and that must check clean.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Errors, help and notes teach `Some(x)` / `None`, the spec's spelling; the
/// quoted source lines of a golden may still hold the old one.
#[test]
fn golden_messages_spell_options_some_none() {
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
            if let (true, Some(old)) = (message, cpl_diag::old_option_spelling_in_prose(t)) {
                panic!(
                    "{}: `{old}` is `None`/`Some(x)` now: {line}",
                    path.display()
                );
            }
        }
    }
}

/// Every fix a golden error offers names something that exists: after
/// `cpl fix`, no new unknown or private name (E0201, E0203, E0207) appears.
#[test]
fn every_golden_fix_names_something_that_exists() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("golden-fixes-exist");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let cpl = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .current_dir(&scratch)
            .output()
            .expect("cpl starts");
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    let unknown = |json: &str| {
        ["E0201", "E0203", "E0207"]
            .iter()
            .map(|c| json.matches(&format!("\"code\":\"{c}\"")).count())
            .sum::<usize>()
    };
    let mut wrong = Vec::new();
    let mut fixed = 0;
    let mut files: Vec<PathBuf> = fs::read_dir(dir())
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    files.sort();
    for file in files {
        let name = file.file_name().unwrap().to_string_lossy().to_string();
        fs::copy(&file, scratch.join(&name)).unwrap();
        let before = unknown(&cpl(&["check", "--json", &name]));
        if cpl(&["fix", "--json", &name]).contains("\"applied\":0,") {
            continue;
        }
        let after = cpl(&["check", "--json", &name]);
        fixed += 1;
        if unknown(&after) > before {
            let text = fs::read_to_string(scratch.join(&name)).unwrap();
            wrong.push(format!("{name}: after `cpl fix`\n{text}\n{after}"));
        }
    }
    assert!(
        fixed >= 20,
        "only {fixed} goldens checked; the scan is broken"
    );
    assert!(wrong.is_empty(), "{}", wrong.join("\n\n"));
}

/// An error points at the user's code, never into `std/`: a library gap is
/// shown at the user's call, with the std line in a note.
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
#[test]
fn real_mistakes_name_the_place_the_rule_and_a_fix() {
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("real-mistakes");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();
    let cpl = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
            .args(args)
            .current_dir(&scratch)
            .env("NO_COLOR", "1")
            .output()
            .expect("cpl starts");
        String::from_utf8_lossy(&out.stderr).to_string() + &String::from_utf8_lossy(&out.stdout)
    };
    let mistakes = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/mistakes");
    let mut wrong = Vec::new();
    for entry in fs::read_dir(mistakes).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let src = fs::read_to_string(&path).unwrap();
        let here: u32 = src
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("// here: "))
            .and_then(|n| n.trim().parse().ok())
            .unwrap_or_else(|| panic!("{name}: first line must be `// here: N`"));
        let file = format!("{name}.cp");
        fs::write(scratch.join(&file), &src).unwrap();
        let shown = cpl(&["check", &file]);
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
        cpl(&["fix", &file]);
        if !no_fix && !cpl(&["check", &file]).contains("no errors") {
            misses.push("`cpl fix` leaves errors".to_string());
        }
        if !misses.is_empty() {
            wrong.push(format!("{name}: {}\n{shown}", misses.join(", ")));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
