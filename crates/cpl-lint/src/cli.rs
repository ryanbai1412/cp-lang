//! `cpl lint`.

use crate::{
    apply_fixes, check_errors, checked, fix_checked_removed_syntax, fix_removed_syntax, is_taste,
    lint, Config, CHECKER_LINTS, LINTS,
};
use cpl_diag::{Diagnostic, Severity};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl lint [--fix] [--taste] [path]...

Reports code that is legal but probably wrong, slow or unidiomatic. A
directory stands for every .cp file under it; no path at all means the
current directory. Levels come from [lints] in the nearest cp.toml and
from with lint(name: allow) in the code.

options:
  --fix     apply the fixes that are safe to apply, then report what is left
  --taste   only the taste lints (L05xx, the plan/PORTING.md checklist):
            a line per file with findings, then the total by lint
  --list    print every lint with its code and default level
  --json    print the files and their findings as `cpl check --json` does
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
/// One linted file, for a caller that prints `--json`.
pub struct FileReport {
    pub path: String,
    pub src: String,
    /// Linted, checks, and no lint at `deny`.
    pub ok: bool,
    pub diagnostics: Vec<cpl_diag::Diagnostic>,
}

/// Turns the reports into the text `--json` prints.
pub type Json = dyn Fn(&[FileReport]) -> String;

/// The errors `cpl check` reports for a file's text, so `cpl lint` shows
/// the same list: `cpl-lint` cannot lower to IR, so `cpl-cli` hands it in.
pub type Errors = dyn Fn(&Path, &str) -> Vec<Diagnostic>;

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    run_with(args, stdout, stderr, None, None)
}

/// `run`, taking `--json` when `json` can print it: `cpl-lint` cannot see
/// the CLI's serializer, so `cpl-cli` hands it in.
pub fn run_with(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    json: Option<&Json>,
    check: Option<&Errors>,
) -> u8 {
    let errors_of = |file: &Path, src: &str| match check {
        Some(check) => check(file, src),
        None => check_errors(Some(file), src),
    };
    let mut as_json = false;
    let mut fix = false;
    let mut taste = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--fix" => fix = true,
            "--taste" => taste = true,
            "--json" if json.is_some() => as_json = true,
            "--list" => {
                for lint in LINTS {
                    let _ = writeln!(
                        stdout,
                        "{}  {:<26} {:<11} {:<5}  {}",
                        lint.code.0,
                        lint.name,
                        lint.group.as_str(),
                        lint.default.as_str(),
                        crate::plain_math(lint.summary)
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
    let mut unchecked = 0usize;
    let mut reports = Vec::new();
    let plural = |n: usize, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
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
        let parsed = cpl_syntax::parse_file(&src);
        if !parsed.diagnostics.is_empty() {
            if !as_json && !taste {
                // What `cpl check` shows, so each root cause is reported
                // once, as in the editor.
                show(stdout, &errors_of(file, &src), &name, &src);
            }
            let _ = writeln!(
                stderr,
                "error: {name} has syntax errors and was not linted; fix them and lint again"
            );
            failed += 1;
            reports.push(FileReport {
                path: name,
                src,
                ok: false,
                diagnostics: Vec::new(),
            });
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
        let found = checked(Some(file), &src, &config);
        let problems: Vec<Diagnostic> = match check {
            Some(_) => (errors_of(file, &src).into_iter())
                .filter(|d| !CHECKER_LINTS.iter().any(|l| l.code == d.code))
                .collect(),
            None => found.problems,
        };
        if !problems.is_empty() {
            let _ = writeln!(
                stderr,
                "error: {name} has {} lints cannot see past; they are shown with the lints",
                plural(problems.len(), "type error")
            );
            unchecked += 1;
            if !taste {
                errors += problems.len();
            }
        }
        diagnostics.extend(found.lints);
        diagnostics.retain(keep);
        crate::drop_repeats(&mut diagnostics, &problems);
        diagnostics.sort_by_key(|d| (d.span.start, d.span.end, d.code));
        errors += diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        warnings += diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();
        if as_json {
            let denied = diagnostics.iter().any(|d| d.severity == Severity::Error);
            reports.push(FileReport {
                path: name,
                src,
                ok: problems.is_empty() && !denied,
                diagnostics,
            });
        } else if taste {
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
        } else {
            // Type errors go with the lints, so one `cpl lint` (and Vim's
            // `:make`) shows everything wrong with the file.
            let mut shown = problems;
            shown.extend(diagnostics);
            shown.sort_by_key(|d| (d.span.start, d.span.end, d.code));
            show(stdout, &shown, &name, &src);
        }
    }
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
    if unchecked > 0 {
        summary += &format!(", {unchecked} not checking");
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
    match json {
        Some(json) if as_json => {
            let _ = writeln!(stdout, "{}", json(&reports));
        }
        _ => {
            let _ = writeln!(stderr, "{summary}");
        }
    }
    (errors > 0 || failed > 0 || unchecked > 0) as u8
}

fn show(stdout: &mut dyn Write, diagnostics: &[Diagnostic], name: &str, src: &str) {
    let lines = cpl_diag::LineIndex::new(src);
    for diagnostic in diagnostics {
        let _ = writeln!(
            stdout,
            "{}",
            cpl_diag::render(diagnostic, name, src, &lines)
        );
    }
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
