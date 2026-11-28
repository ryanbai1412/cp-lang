//! `spec/diagnostics.md` against the compiler (lints have their own
//! registry and checks in `cpl-lint`). Every code the compiler can
//! emit is in the catalogue; every catalogued code that is not marked
//! *(0.3)*, *(retired)* or *(reserved)* has a registry trigger that
//! produces exactly that code and nothing else, and a fix that checks (or
//! runs) clean. Codes marked *(0.3)* or *(retired)* must not be defined.
//!
//! `cargo test -p cpl-cli --test catalogue -- --nocapture` prints the counts.
//!
//! Rule: `errors/R13`, handbook/guidelines/errors-checklist.md#r13-every-code-is-catalogued-with-a-trigger-a-fix-and-a-page.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The rows of the catalogue's tables: code and whether it is emitted.
fn catalogue() -> Vec<(String, bool)> {
    let text = ["spec/diagnostics.md", "spec/lints.md"]
        .map(|f| std::fs::read_to_string(root().join(f)).unwrap())
        .concat();
    let mut out = Vec::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(code), Some(name)) = (cells.get(1), cells.get(2)) else {
            continue;
        };
        if !code.starts_with(['E', 'W', 'R', 'L'])
            || !code[1..].starts_with(|c: char| c.is_ascii_digit())
        {
            continue;
        }
        let emitted = !["*(0.3)*", "*(retired)*", "*(reserved)*"]
            .iter()
            .any(|m| name.contains(m));
        match code.split_once('-') {
            Some((lo, hi)) => {
                let (p, lo, hi) = (
                    &lo[..1],
                    lo[1..].parse::<u32>().unwrap(),
                    hi[1..].parse::<u32>().unwrap(),
                );
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
    src.split("Code(\"")
        .skip(1)
        .filter_map(|s| s.split_once('"'))
        .map(|(c, _)| c.to_string())
        .collect()
}

/// The codes `cpl <mode>` reports for `src`: errors, then warnings.
fn reported(mode: &str, src: &str, input: &str, name: &str) -> (Vec<String>, Vec<String>) {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("catalogue_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.cp"));
    std::fs::write(&file, src).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    let mut child = cmd
        .arg(mode)
        .arg(&file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cpl starts");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("cpl exits");
    let text =
        String::from_utf8_lossy(&out.stderr).to_string() + &String::from_utf8_lossy(&out.stdout);
    let codes = |kind: &str| -> Vec<String> {
        text.lines()
            .filter_map(|l| l.strip_prefix(kind)?.strip_prefix('[')?.split_once(']'))
            .map(|(c, _)| c.to_string())
            .collect()
    };
    (codes("error"), codes("warning"))
}

/// Codes marked *(next)*: only cpl-next emits them until it ships as `cpl`.
fn next_only() -> BTreeSet<String> {
    let text = std::fs::read_to_string(root().join("spec/diagnostics.md")).unwrap();
    text.lines()
        .filter_map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            (cells.get(2)?.contains("*(next)*")).then(|| cells[1].to_string())
        })
        .collect()
}

/// A *(next)* code's trigger reports exactly it on cpl-next and its fix
/// nothing, while old `cpl` does not report it, so the marker stays true.
fn next_only_holds(
    code: &str,
    mode: &str,
    info: &cpl_diag::registry::CodeInfo,
) -> Result<(), String> {
    let (old, _) = reported(mode, info.trigger, info.input, &format!("{code}_old"));
    if old.iter().any(|c| c == code) {
        return Err(format!(
            "{code}: marked *(next)*, but old cpl reports it too"
        ));
    }
    let trigger = next_reported(mode, info.trigger, info.input, &format!("{code}_next"))?;
    if trigger != [code] {
        return Err(format!("{code}: cpl-next trigger reports {trigger:?}"));
    }
    let fix = next_reported(mode, info.fix, info.input, &format!("{code}_next_fix"))?;
    match fix.is_empty() {
        true => Ok(()),
        false => Err(format!("{code}: cpl-next fix still reports {fix:?}")),
    }
}

/// The codes `cpl-next` (built beside `cpl`) reports for `src`.
fn next_reported(mode: &str, src: &str, input: &str, name: &str) -> Result<Vec<String>, String> {
    let next = Path::new(env!("CARGO_BIN_EXE_cpl")).with_file_name(if cfg!(windows) {
        "cpl-next.exe"
    } else {
        "cpl-next"
    });
    if !next.exists() {
        return Err(format!(
            "a *(next)* code needs {}; build it: cargo build -p cpl-next",
            next.display()
        ));
    }
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("catalogue_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join(format!("{name}.cp"));
    std::fs::write(&file, src).unwrap();
    let mut cmd = Command::new(next);
    cmd.arg(&file);
    if mode == "check" {
        cmd.arg("--check");
    }
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cpl-next starts: {e}"))?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("cpl-next exits");
    let is_code = |c: &&str| {
        c.len() == 5 && c.starts_with(['E', 'W', 'R']) && c[1..].chars().all(|d| d.is_ascii_digit())
    };
    Ok(String::from_utf8_lossy(&out.stderr)
        .lines()
        .filter_map(|l| l.split(": ").find(is_code).map(str::to_string))
        .collect())
}

/// `src` with the fix-it of its first diagnostic applied, if it has one.
fn apply_first_fix_it(src: &str) -> Option<String> {
    let mut db = cpl_facade::Session::default();
    db = db.with_file("trigger.cp", src);
    let mut diags = db.diagnostics("trigger.cp", cpl_facade::Layer::Built);
    diags.sort_by_key(|d| d.span.start);
    let mut edits = diags.first()?.fixes.first()?.edits.clone();
    edits.sort_by_key(|(span, _)| std::cmp::Reverse(span.start));
    let mut out = src.to_string();
    for (span, text) in edits {
        out.replace_range(span.start as usize..span.end as usize, &text);
    }
    Some(out)
}

#[test]
fn every_defined_code_is_catalogued() {
    let listed: BTreeSet<String> = catalogue().into_iter().map(|(c, _)| c).collect();
    let missing: Vec<String> = defined().difference(&listed).cloned().collect();
    assert!(
        missing.is_empty(),
        "defined but not in spec/diagnostics.md or spec/lints.md: {missing:?}"
    );
}

#[test]
fn codes_marked_not_emitted_are_not_defined() {
    let defined = defined();
    let reserved = |c: &str| cpl_diag::registry::lookup(c).is_some_and(|i| i.mode == "reserved");
    // A *(0.3)* code may already be emitted: its trigger then reports exactly
    // it, and its fix nothing.
    let edition_only = |c: &str| {
        cpl_diag::registry::lookup(c).is_some_and(|i| {
            i.mode == "check"
                && reported("check", i.trigger, i.input, &format!("{c}_spec_trigger")).0 == [c]
                && reported("check", i.fix, i.input, &format!("{c}_spec_fix"))
                    .0
                    .is_empty()
        })
    };
    let stale: Vec<String> = catalogue()
        .into_iter()
        .filter(|(c, emitted)| !emitted && defined.contains(c) && !reserved(c) && !edition_only(c))
        .map(|(c, _)| c)
        .collect();
    assert!(
        stale.is_empty(),
        "marked (0.3) or (retired) but defined, so emitted: {stale:?}"
    );
}

#[test]
fn every_emitted_code_has_an_exact_trigger_and_a_clean_fix() {
    let mut failures = Vec::new();
    let (mut tested, mut with_fix_it, mut emitted) = (0, 0, 0);
    let mut no_fix_it = Vec::new();
    let next = next_only();
    let catalogue: Vec<(String, bool)> = catalogue()
        .into_iter()
        .filter(|(c, _)| !c.starts_with('L'))
        .collect();
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
                failures.push(format!(
                    "{code}: emitted, but its registry mode is {other:?}"
                ));
                continue;
            }
        };
        if next.contains(&code) {
            match next_only_holds(&code, mode, info) {
                Ok(()) => tested += 1,
                Err(why) => failures.push(why),
            }
            continue;
        }
        let (errors, warnings) =
            reported(mode, info.trigger, info.input, &format!("{code}_trigger"));
        let exact = if code.starts_with('W') {
            errors.is_empty() && warnings.iter().all(|w| *w == code) && !warnings.is_empty()
        } else {
            errors == [code.clone()]
        };
        if !exact {
            failures.push(format!(
                "{code}: trigger reports errors {errors:?}, warnings {warnings:?}"
            ));
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
            match apply_first_fix_it(info.trigger) {
                Some(fixed) => {
                    let (errors, _) = reported("check", &fixed, "", &format!("{code}_fix_it"));
                    if errors.contains(&code) {
                        failures.push(format!("{code}: its fix-it leaves the error:\n{fixed}"));
                    }
                    with_fix_it += 1;
                }
                None => no_fix_it.push(code),
            }
        }
    }
    let listed = catalogue.len();
    println!("catalogue: {listed} codes, {emitted} emitted, {tested} with an exact trigger and clean fix, {with_fix_it} with a fix-it");
    println!(
        "compile-time codes without a fix-it on their trigger: {}",
        no_fix_it.join(" ")
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Single-code rows of a catalogue text: code, name, and the marker
/// (*(retired)*, *(reserved)*, *(0.3)*) if any.
fn named_rows(text: &str) -> std::collections::BTreeMap<String, (String, Option<&'static str>)> {
    let mut out = std::collections::BTreeMap::new();
    for line in text.lines() {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let (Some(code), Some(name)) = (cells.get(1), cells.get(2)) else {
            continue;
        };
        if code.contains('-')
            || !code.starts_with(['E', 'W', 'R', 'L'])
            || !code[1..].starts_with(|c: char| c.is_ascii_digit())
        {
            continue;
        }
        let marker = ["*(retired)*", "*(reserved)*", "*(0.3)*", "*(next)*"]
            .into_iter()
            .find(|m| name.contains(m));
        let base = name.split_whitespace().next().unwrap_or("").to_string();
        out.insert(code.to_string(), (base, marker));
    }
    out
}

/// The catalogue as committed at `rev`, or `None` outside a git checkout.
fn catalogue_at(rev: &str) -> Option<String> {
    let mut text = String::new();
    let mut found = false;
    for f in ["spec/diagnostics.md", "spec/lints.md"] {
        let out = Command::new("git")
            .args(["show", &format!("{rev}:{f}")])
            .current_dir(root())
            .output()
            .ok()?;
        if out.status.success() {
            found = true;
            text.push_str(&String::from_utf8_lossy(&out.stdout));
        }
    }
    found.then_some(text)
}

/// A code keeps its meaning forever: once catalogued, it is never deleted
/// or renamed, and once retired it stays retired, so an old error message
/// or a search for the code still finds what it meant. A *(reserved)* or
/// *(0.3)* code may be given its name. Checked for the change being made
/// (the working tree against `HEAD`) and for the last commit.
/// Rule: `errors/R14`, handbook/guidelines/errors-checklist.md#r14-a-code-never-changes-meaning.
#[test]
fn codes_never_change_meaning() {
    let now = ["spec/diagnostics.md", "spec/lints.md"]
        .map(|f| std::fs::read_to_string(root().join(f)).unwrap())
        .concat();
    let pairs = [("HEAD", Some(now)), ("HEAD~1", catalogue_at("HEAD"))];
    let mut broken = Vec::new();
    let mut compared = 0;
    for (old_rev, new) in pairs {
        let (Some(old), Some(new)) = (catalogue_at(old_rev), new) else {
            continue;
        };
        compared += 1;
        let (old, new) = (named_rows(&old), named_rows(&new));
        for (code, (name, marker)) in &old {
            let open = matches!(marker, Some("*(reserved)*" | "*(0.3)*"));
            match new.get(code) {
                None => broken.push(format!("{code} `{name}` was deleted (since {old_rev})")),
                Some((n, _)) if n != name && !open => {
                    broken.push(format!("{code} was `{name}`, now `{n}` (since {old_rev})"))
                }
                Some((_, m)) if *marker == Some("*(retired)*") && *m != Some("*(retired)*") => {
                    broken.push(format!(
                        "{code} `{name}` was retired and is back (since {old_rev})"
                    ))
                }
                _ => {}
            }
        }
    }
    eprintln!("catalogue compared with {compared} earlier versions");
    assert!(
        broken.is_empty(),
        "a code changed meaning; give the new meaning a new code and mark the old one *(retired)*:\n{}",
        broken.join("\n")
    );
}

/// Every code links to a page the site builds (`errors/R13`,
/// handbook/guidelines/errors-checklist.md#r13-every-code-is-catalogued-with-a-trigger-a-fix-and-a-page):
/// `/errors/<code>` comes from a registry entry, `/lints/<code>` from a lint
/// record under `spec/lints/` or `spec/checker-lints/`.
#[test]
fn every_code_links_to_a_page_the_site_builds() {
    let mut records = BTreeSet::new();
    for dir in ["spec/lints", "spec/checker-lints"] {
        for entry in std::fs::read_dir(root().join(dir)).unwrap() {
            let toml = std::fs::read_to_string(entry.unwrap().path().join("lint.toml"))
                .unwrap_or_default();
            if let Some(code) = toml
                .lines()
                .find_map(|l| l.strip_prefix("code = \"")?.strip_suffix('"'))
            {
                records.insert(code.to_string());
            }
        }
    }
    let mut codes = defined();
    codes.extend(cpl_diag::REGISTRY.iter().map(|i| i.code.0.to_string()));
    codes.extend(records.iter().cloned());
    let broken: Vec<String> = codes
        .iter()
        .filter(|c| {
            let built = if c.starts_with('L') {
                records.contains(*c)
            } else {
                cpl_diag::registry::lookup(c).is_some()
            };
            cpl_diag::registry::page(c).is_none() || !built
        })
        .cloned()
        .collect();
    assert!(codes.len() > 200, "only {} codes found", codes.len());
    assert!(
        broken.is_empty(),
        "codes whose page link the site does not build: {broken:?}"
    );
}
