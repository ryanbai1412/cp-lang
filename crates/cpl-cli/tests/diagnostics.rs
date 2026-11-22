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
    String::from_utf8_lossy(&out.stderr).into_owned()
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
