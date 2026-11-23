//! An error about a name the file declares on another line shows that
//! declaration as a label, so the user sees both halves without
//! searching: `x = 2` on a `let x` points at the `let`. Checked over
//! every diagnostics golden and every catalogue trigger.
//! Rule: `errors/R4`, handbook/guidelines/errors-checklist.md#r4-a-label-marks-the-other-half.

use std::fs;
use std::path::Path;
use std::process::Command;

/// Lines (1-based) where `src` declares `name`.
fn declared_on(src: &str, name: &str) -> Vec<usize> {
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let after = |s: &str, at: usize| !s[at..].starts_with(is_word);
    let mut out = Vec::new();
    for (i, line) in src.lines().enumerate() {
        let t = line.trim_start();
        let mut hit = false;
        for kw in [
            "let mut ", "let ", "var ", "fn ", "struct ", "domain ", "enum ", "for ",
        ] {
            if let Some(r) = t.strip_prefix(kw) {
                if r.starts_with(name) && after(r, name.len()) {
                    hit = true;
                }
            }
        }
        if let Some(r) = t.strip_prefix("fn ") {
            if let Some((_, params)) = r.split_once('(') {
                for p in params.split([',', ')']) {
                    let p = p.trim().trim_start_matches("mut ");
                    if p.strip_prefix(name)
                        .is_some_and(|r| r.trim_start().starts_with(':'))
                    {
                        hit = true;
                    }
                }
            }
        }
        if hit {
            out.push(i + 1);
        }
    }
    out
}

/// Errors in `rendered` that name a declaration elsewhere in `src` but
/// show no label.
fn unlabelled(src: &str, rendered: &str) -> Vec<String> {
    let mut out = Vec::new();
    let blocks = rendered
        .split("\nerror[")
        .map(|b| b.trim_start_matches("error["));
    for block in blocks.filter(|b| b.starts_with(['E', 'W', 'R', 'L'])) {
        let mut lines = block.lines();
        let header = lines.next().unwrap_or("");
        let Some(primary) = block
            .lines()
            .find_map(|l| l.trim_start().strip_prefix("--> "))
            .and_then(|at| at.rsplit(':').nth(1))
            .and_then(|n| n.parse::<usize>().ok())
        else {
            continue;
        };
        let labelled = block.lines().any(|l| {
            l.trim_start()
                .strip_prefix('|')
                .is_some_and(|r| r.trim_start().starts_with('-'))
        });
        if labelled {
            continue;
        }
        let message = header.split_once("]: ").map_or("", |(_, m)| m);
        for (i, piece) in message.split('`').enumerate() {
            let ident = !piece.is_empty()
                && piece.starts_with(|c: char| c.is_alphabetic() || c == '_')
                && piece.chars().all(|c| c.is_alphanumeric() || c == '_');
            if i % 2 == 1 && ident {
                let elsewhere: Vec<_> = declared_on(src, piece)
                    .into_iter()
                    .filter(|&l| l != primary)
                    .collect();
                if !elsewhere.is_empty() {
                    out.push(format!(
                        "error[{header}: `{piece}` is declared on line {elsewhere:?}, with no label"
                    ));
                    break;
                }
            }
        }
    }
    out
}

fn check(src: &str, name: &str, dir: &Path) -> String {
    let file = dir.join(format!("{name}.cp"));
    fs::write(&file, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(&file)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout)
}

#[test]
fn errors_label_the_declaration_they_name() {
    let dir = std::env::temp_dir().join(format!("cpl-labels-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let mut cases: Vec<(String, String)> = Vec::new();
    let goldens = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/diagnostics");
    let mut paths: Vec<_> = fs::read_dir(&goldens)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    paths.sort();
    for p in paths {
        let name = p.file_stem().unwrap().to_str().unwrap().to_string();
        cases.push((name, fs::read_to_string(&p).unwrap()));
    }
    for info in cpl_diag::REGISTRY
        .iter()
        .filter(|i| i.mode == "check" && !i.trigger.is_empty())
    {
        cases.push((info.code.0.to_string(), info.trigger.to_string()));
    }
    let mut missing = Vec::new();
    for (name, src) in &cases {
        for e in unlabelled(src, &check(src, name, &dir)) {
            missing.push(format!("{name}: {e}"));
        }
    }
    let _ = fs::remove_dir_all(&dir);
    eprintln!("{} programs checked", cases.len());
    assert!(
        missing.is_empty(),
        "{} errors with no label:\n{}",
        missing.len(),
        missing.join("\n")
    );
}
