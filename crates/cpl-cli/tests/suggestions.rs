//! A suggestion names something that exists: wherever an error says
//! "did you mean `x`?", writing `x` in place of the underlined text makes
//! that error go away. Checked over every diagnostics golden, every
//! catalogue trigger, and generated typos of every std fn and method.
//! Rule: `errors/R7`, handbook/guidelines/errors-checklist.md#r7-only-suggest-what-exists.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn check(file: &Path, src: &str) -> String {
    fs::write(file, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg("check")
        .arg(file)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout)
}

/// Each "did you mean" in `rendered`: code, line, column, underline
/// length and the suggested text.
/// A suggestion from another module ("did you mean `x`, from `std.m`?")
/// carries the module, whose `use` it takes.
#[allow(clippy::type_complexity)]
fn suggestions(rendered: &str) -> Vec<(String, usize, usize, usize, String, Option<String>)> {
    let mut out = Vec::new();
    for block in rendered
        .split("\nerror[")
        .skip(usize::from(!rendered.starts_with("error[")))
    {
        let block = block.trim_start_matches("error[");
        let code = block.split(']').next().unwrap_or("").to_string();
        let Some(at) = block
            .lines()
            .find_map(|l| l.trim_start().strip_prefix("--> "))
        else {
            continue;
        };
        let mut parts = at.rsplit(':');
        let (Some(col), Some(line)) = (parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(col), Ok(line)) = (col.parse::<usize>(), line.parse::<usize>()) else {
            continue;
        };
        let Some(len) = block
            .lines()
            .map(|l| l.chars().filter(|&c| c == '^').count())
            .find(|&n| n > 0)
        else {
            continue;
        };
        for l in block.lines() {
            if let Some(rest) = l.trim_start().strip_prefix("= help: did you mean `") {
                if let Some((x, after)) = rest.split_once('`') {
                    let module = after
                        .strip_prefix(", from `")
                        .and_then(|m| m.split_once('`'))
                        .map(|(m, _)| m.to_string());
                    out.push((code.clone(), line, col, len, x.to_string(), module));
                }
            }
        }
    }
    out
}

fn replaced(src: &str, line: usize, col: usize, len: usize, with: &str) -> Option<String> {
    let start = src
        .split_inclusive('\n')
        .take(line - 1)
        .map(str::len)
        .sum::<usize>()
        + col
        - 1;
    src.is_char_boundary(start + len)
        .then(|| format!("{}{with}{}", &src[..start], &src[start + len..]))
}

fn cases() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut cases = Vec::new();
    let mut goldens: Vec<PathBuf> = fs::read_dir(root.join("tests/diagnostics"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    goldens.sort();
    for p in goldens {
        let name = p.file_stem().unwrap().to_str().unwrap().to_string();
        cases.push((name, fs::read_to_string(&p).unwrap()));
    }
    for info in cpl_diag::REGISTRY
        .iter()
        .filter(|i| i.mode == "check" && !i.trigger.is_empty())
    {
        cases.push((info.code.0.to_string(), info.trigger.to_string()));
    }
    let mut names = std::collections::BTreeSet::new();
    for e in fs::read_dir(root.join("../../std")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|e| e != "cp") {
            continue;
        }
        for line in fs::read_to_string(&p).unwrap().lines() {
            let Some(rest) = line.strip_prefix("pub fn ") else {
                continue;
            };
            let f: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if f.len() >= 4 {
                names.insert((
                    f.clone(),
                    rest[f.len()..].starts_with("(self") || rest[f.len()..].contains("(self"),
                ));
            }
        }
    }
    for (f, method) in names {
        let typo = format!("{}{}", &f[..1], &f[2..]);
        let call = if method {
            format!("a.{typo}()")
        } else {
            format!("{typo}(a)")
        };
        cases.push((
            format!("typo_{f}"),
            format!("let a = [3, 1, 2]\nprint({call})\n"),
        ));
    }
    cases
}

#[test]
fn every_did_you_mean_names_something_that_exists() {
    let dir = std::env::temp_dir().join(format!("cpl-suggestions-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let cases = cases();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let results: Vec<(usize, Vec<String>)> = std::thread::scope(|s| {
        cases
            .chunks(cases.len().div_ceil(threads))
            .enumerate()
            .map(|(t, chunk)| {
                let dir = &dir;
                s.spawn(move || {
                    let (mut seen, mut wrong) = (0, Vec::new());
                    let file = dir.join(format!("t{t}.cp"));
                    for (name, src) in chunk {
                        for (code, line, col, len, x, module) in suggestions(&check(&file, src)) {
                            seen += 1;
                            let Some(mut fixed) = replaced(src, line, col, len, &x) else {
                                continue;
                            };
                            let mut line = line;
                            if let Some(m) = module {
                                fixed = format!("use {m}.{{{x}}}\n{fixed}");
                                line += 1;
                            }
                            let again = check(&file, &fixed);
                            let still = suggestions(&again)
                                .iter()
                                .any(|s| s.0 == code && s.1 == line && s.2 == col)
                                || again.contains(&format!("error[{code}]"))
                                    && again.contains(&format!(":{line}:{col}\n"));
                            if still {
                                wrong.push(format!(
                                    "{name}: `{x}` at {line}:{col} still gives {code}"
                                ));
                            }
                        }
                    }
                    (seen, wrong)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    });
    let _ = fs::remove_dir_all(&dir);
    let seen: usize = results.iter().map(|r| r.0).sum();
    let wrong: Vec<String> = results.into_iter().flat_map(|r| r.1).collect();
    eprintln!("{} programs, {seen} suggestions tried", cases.len());
    assert!(
        seen > 100,
        "only {seen} suggestions: the generator stopped producing typos"
    );
    assert!(
        wrong.is_empty(),
        "{} suggestions do not exist:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
