//! `spec/diagnostics.md` against the compiler (lints have their own
//! registry and checks in `cpl-lint`). Every code the compiler can
//! emit is in the catalogue; every catalogued code that is not marked
//! *(0.3)*, *(retired)* or *(reserved)* has a registry trigger that
//! produces exactly that code and nothing else, and a fix that checks (or
//! runs) clean. Codes marked as not emitted must not be defined.
//!
//! `cargo test -p cpl-cli --test catalogue -- --nocapture` prints the counts.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The rows of the catalogue's tables: code and whether it is emitted.
fn catalogue() -> Vec<(String, bool)> {
    let text = std::fs::read_to_string(root().join("spec/diagnostics.md")).unwrap();
    let mut out = Vec::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(code), Some(name)) = (cells.get(1), cells.get(2)) else { continue };
        if !code.starts_with(['E', 'W', 'R', 'L']) || !code[1..].starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let emitted = !["*(0.3)*", "*(retired)*", "*(reserved)*"].iter().any(|m| name.contains(m));
        match code.split_once('-') {
            Some((lo, hi)) => {
                let (p, lo, hi) = (&lo[..1], lo[1..].parse::<u32>().unwrap(), hi[1..].parse::<u32>().unwrap());
                out.extend((lo..=hi).map(|n| (format!("{p}{n:04}"), emitted)));
            }
            None => out.push((code.to_string(), emitted)),
        }
    }
    out
}

/// Every code `cpl_diag::codes` defines.
fn defined() -> BTreeSet<String> {
    let src = std::fs::read_to_string(root().join("crates/cpl-diag/src/lib.rs")).unwrap();
    src.split("Code(\"").skip(1).filter_map(|s| s.split_once('"')).map(|(c, _)| c.to_string()).collect()
}

/// The codes `cpl <mode>` reports for `src`: errors, then warnings.
fn reported(mode: &str, src: &str, input: &str, name: &str) -> (Vec<String>, Vec<String>) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("catalogue_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.cp"));
    std::fs::write(&file, src).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .arg(mode)
        .arg(&file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl starts");
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let out = child.wait_with_output().expect("cpl exits");
    let text = String::from_utf8_lossy(&out.stderr).to_string() + &String::from_utf8_lossy(&out.stdout);
    let codes = |kind: &str| -> Vec<String> {
        text.lines()
            .filter_map(|l| l.strip_prefix(kind)?.strip_prefix('[')?.split_once(']'))
            .map(|(c, _)| c.to_string())
            .collect()
    };
    (codes("error"), codes("warning"))
}

/// The first diagnostic `cpl check` reports for `src`, in process, to see
/// whether it carries a machine-applicable fix.
fn first_has_fix(src: &str) -> bool {
    let mut db = cpl_check::Database::default();
    db.set_file("trigger.cp", src);
    let analysis = db.analyze("trigger.cp").unwrap();
    let mut diags = cpl_ir::check(&analysis);
    diags.sort_by_key(|d| d.span.start);
    diags.first().is_some_and(|d| !d.fixes.is_empty())
}

#[test]
fn every_defined_code_is_catalogued() {
    let listed: BTreeSet<String> = catalogue().into_iter().map(|(c, _)| c).collect();
    let missing: Vec<String> = defined().difference(&listed).cloned().collect();
    assert!(missing.is_empty(), "defined but not in spec/diagnostics.md: {missing:?}");
}

#[test]
fn codes_marked_not_emitted_are_not_defined() {
    let defined = defined();
    let stale: Vec<String> =
        catalogue().into_iter().filter(|(c, emitted)| !emitted && defined.contains(c)).map(|(c, _)| c).collect();
    assert!(stale.is_empty(), "marked (0.3)/(retired)/(reserved) but defined, so emitted: {stale:?}");
}

#[test]
fn every_emitted_code_has_an_exact_trigger_and_a_clean_fix() {
    let mut failures = Vec::new();
    let (mut tested, mut with_fix_it, mut emitted) = (0, 0, 0);
    let mut no_fix_it = Vec::new();
    let catalogue: Vec<(String, bool)> = catalogue().into_iter().filter(|(c, _)| !c.starts_with('L')).collect();
    for (code, is_emitted) in catalogue.clone() {
        if !is_emitted {
            continue;
        }
        emitted += 1;
        let Some(info) = cpl_diag::registry::lookup(&code) else {
            failures.push(format!("{code}: no registry entry"));
            continue;
        };
        let mode = match info.mode {
            "internal" => continue,
            "check" => "check",
            "run" => "run",
            "build" => "build",
            other => {
                failures.push(format!("{code}: emitted, but its registry mode is {other:?}"));
                continue;
            }
        };
        let (errors, warnings) = reported(mode, info.trigger, info.input, &format!("{code}_trigger"));
        let exact = if code.starts_with('W') {
            errors.is_empty() && warnings.iter().all(|w| *w == code) && !warnings.is_empty()
        } else {
            errors == [code.clone()]
        };
        if !exact {
            failures.push(format!("{code}: trigger reports errors {errors:?}, warnings {warnings:?}"));
            continue;
        }
        let fix_mode = if mode == "build" { "check" } else { mode };
        let (errors, warnings) = reported(fix_mode, info.fix, info.input, &format!("{code}_fix"));
        if !errors.is_empty() || warnings.contains(&code) {
            failures.push(format!("{code}: fix still reports {errors:?} {warnings:?}"));
            continue;
        }
        tested += 1;
        if mode == "check" {
            if first_has_fix(info.trigger) {
                with_fix_it += 1;
            } else {
                no_fix_it.push(code);
            }
        }
    }
    let listed = catalogue.len();
    println!("catalogue: {listed} codes, {emitted} emitted, {tested} with an exact trigger and clean fix, {with_fix_it} with a fix-it");
    println!("compile-time codes without a fix-it on their trigger: {}", no_fix_it.join(" "));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
