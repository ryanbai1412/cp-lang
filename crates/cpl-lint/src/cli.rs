//! `cpl lint`.

use crate::{
    apply_fixes, checker_lints, fix_checked_removed_syntax, fix_removed_syntax, is_taste, lint,
    Config, LINTS,
};
use cpl_diag::Severity;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl lint [--fix] [--taste] [path]...

Reports code that is legal but probably wrong, slow or unidiomatic. A
directory stands for every .cp file under it; no path at all means the
current directory. Levels come from [lints] in the nearest cp.toml and
from @allow(name) on an item.

options:
  --fix     apply the fixes that are safe to apply, then report what is left
  --taste   only the taste lints (L05xx, the plan/PORTING.md checklist):
            a line per file with findings, then the total by lint
  --list    print every lint with its code and default level
";

/// Runs `cpl fix` as an alias for `cpl lint --fix`.
pub fn run_fix(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    if args.iter().any(|a| a == "-h" || a == "--help") {
        let _ = write!(
            stdout,
            "usage: cpl fix [path]...\n\nAlias for `cpl lint --fix`.\n"
        );
        return 0;
    }
    let mut forwarded = Vec::with_capacity(args.len() + 1);
    forwarded.push("--fix".to_string());
    forwarded.extend(args.iter().cloned());
    run(&forwarded, stdout, stderr)
}

/// Runs `cpl lint`. Returns the exit code: 0 when nothing is at `deny`
/// (warnings do not fail), 1 when a `deny` lint fired or a file could not
/// be linted, 2 for a usage error.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let mut fix = false;
    let mut taste = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--fix" => fix = true,
            "--taste" => taste = true,
            "--list" => {
                for lint in LINTS {
                    let _ = writeln!(
                        stdout,
                        "{}  {:<22} {:<5}  {}",
                        lint.code.0,
                        lint.name,
                        lint.default.as_str(),
                        lint.summary
                    );
                }
                return 0;
            }
            "-h" | "--help" => {
                let _ = write!(stdout, "{USAGE}");
                return 0;
            }
            flag if flag.starts_with('-') => {
                let _ = write!(stderr, "error: unknown option `{flag}`\n\n{USAGE}");
                return 2;
            }
            path => paths.push(PathBuf::from(path)),
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    let mut files = Vec::new();
    for path in &paths {
        if let Err(e) = collect(path, &mut files) {
            let _ = writeln!(stderr, "error: can't read {}: {e}", path.display());
            return 1;
        }
    }
    files.sort();
    files.dedup();

    let (mut warnings, mut errors, mut fixed, mut failed) = (0usize, 0usize, 0usize, 0usize);
    let keep =
        |d: &cpl_diag::Diagnostic| !taste || LINTS.iter().any(|l| l.code == d.code && is_taste(l));
    let (mut tasteful, mut by_lint) = (0usize, BTreeMap::<&str, usize>::new());
    let mut bad_configs: Vec<PathBuf> = Vec::new();
    for file in &files {
        let name = file.display().to_string();
        let mut src = match std::fs::read_to_string(file) {
            Ok(src) => src,
            Err(e) => {
                let _ = writeln!(stderr, "error: can't read {name}: {e}");
                failed += 1;
                continue;
            }
        };
        let (config, found) = Config::discover(file);
        if let Some((toml, problems)) = found {
            // Once per cp.toml, however many files it covers.
            if !problems.is_empty() && !bad_configs.contains(&toml) {
                for problem in &problems {
                    let _ = writeln!(stderr, "warning: {}: {problem}", toml.display());
                }
                bad_configs.push(toml);
            }
        }
        if fix {
            let (text, applied) = fix_removed_syntax(&src);
            if applied > 0 {
                fixed += applied;
                src = text;
                if let Err(e) = std::fs::write(file, &src) {
                    let _ = writeln!(stderr, "error: can't write {name}: {e}");
                    failed += 1;
                }
            }
            let (text, applied) = fix_checked_removed_syntax(Some(file), &src);
            if applied > 0 {
                fixed += applied;
                src = text;
                if let Err(e) = std::fs::write(file, &src) {
                    let _ = writeln!(stderr, "error: can't write {name}: {e}");
                    failed += 1;
                }
            }
        }
        if !cpl_syntax::parse_file(&src).diagnostics.is_empty() {
            let _ = writeln!(
                stderr,
                "error: {name} has syntax errors and was not linted; `cpl check {name}` shows them"
            );
            failed += 1;
            continue;
        }
        let mut diagnostics = lint(&src, &config);
        diagnostics.retain(keep);
        if fix {
            // A fix can uncover another, and two that overlap take two
            // rounds; a file has finitely many of either.
            loop {
                let (text, applied) = apply_fixes(&src, &diagnostics);
                if applied == 0 {
                    break;
                }
                fixed += applied;
                src = text;
                diagnostics = lint(&src, &config);
                diagnostics.retain(keep);
            }
            if let Err(e) = std::fs::write(file, &src) {
                let _ = writeln!(stderr, "error: can't write {name}: {e}");
                failed += 1;
            }
        }
        diagnostics.extend(checker_lints(Some(file), &src, &config));
        diagnostics.retain(keep);
        diagnostics.sort_by_key(|d| (d.span.start, d.span.end, d.code));
        errors += diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        warnings += diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        if taste {
            if !diagnostics.is_empty() {
                let mut here = BTreeMap::<&str, usize>::new();
                for d in &diagnostics {
                    *here.entry(d.code.0).or_default() += 1;
                    *by_lint.entry(d.code.0).or_default() += 1;
                }
                let parts: Vec<String> = here.iter().map(|(c, n)| format!("{c}:{n}")).collect();
                let _ = writeln!(
                    stdout,
                    "{:>4}  {name}  {}",
                    diagnostics.len(),
                    parts.join(" ")
                );
                tasteful += 1;
            }
        } else if !diagnostics.is_empty() {
            let lines = cpl_diag::LineIndex::new(&src);
            for diagnostic in &diagnostics {
                let _ = writeln!(
                    stdout,
                    "{}",
                    cpl_diag::render(diagnostic, &name, &src, &lines)
                );
            }
        }
    }
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    let mut summary = format!(
        "{} checked, {}, {}",
        plural(files.len(), "file"),
        plural(warnings, "warning"),
        plural(errors, "error")
    );
    if fix {
        summary += &format!(
            ", {} applied",
            plural(fixed, "fix").replace("fixs", "fixes")
        );
    }
    if failed > 0 {
        summary += &format!(", {failed} not linted");
    }
    if taste {
        let parts: Vec<String> = by_lint.iter().map(|(c, n)| format!("{c}:{n}")).collect();
        let _ = writeln!(
            stdout,
            "{:>4}  total over {} of {}  {}",
            warnings + errors,
            plural(tasteful, "file"),
            files.len(),
            parts.join(" ")
        );
    }
    let _ = writeln!(stderr, "{summary}");
    (errors > 0 || failed > 0) as u8
}

/// Adds `path`, or every `.cp` file under it, to `files`.
fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !path.is_dir() {
        // Reading it is what reports a missing file.
        std::fs::metadata(path)?;
        files.push(path.to_path_buf());
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let name = entry.file_name();
        let hidden = name.to_string_lossy().starts_with('.');
        if child.is_dir() {
            if !hidden && name != "target" {
                collect(&child, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "cp") {
            files.push(child);
        }
    }
    Ok(())
}
