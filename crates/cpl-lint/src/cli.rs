//! `cpl lint`.

use crate::{
    apply_lint_fixes_where, checked_analysis, fix_checked_removed_syntax_in, fix_removed_syntax,
    is_taste, lint_analysis, Config, LINTS,
};
use cpl_diag::{Diagnostic, Severity};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl lint [--fix] [--taste] [--save] [path]...

Reports code that is legal but probably wrong, slow or unidiomatic. A
directory stands for every .cp file under it; no path at all means the
current directory. Levels come from [lints] in the nearest cp.toml and
from with lint(name: allow) in the code.

options:
  --fix     apply the fixes that are safe to apply, then report what is left
  --taste   only the taste lints (L05xx, the plan/PORTING.md checklist):
            a line per file with findings, then the total by lint
  --save    write the input a bound lint names (L0302, L0304, L0305, L0306,
            L0599) as a case in <file>.tests/, which `cpl test` runs
  --list    print every lint with its code and default level; with --json,
            every lint's and checker lint's record (spec/tools/cli.md)
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
pub use crate::Errors;

/// `--save`: writes a test case from each diagnostic's witness and returns
/// what to tell the user; `cpl-cli` hands it in, as the input generator is
/// `cpl-stress`'s.
pub type Save = dyn Fn(&std::path::Path, &str, &[Diagnostic]) -> Vec<String>;

pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    run_with(args, stdout, stderr, None, None, None)
}

/// `run`, taking `--json` when `json` can print it: `cpl-lint` cannot see
/// the CLI's serializer, so `cpl-cli` hands it in.
pub fn run_with(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    json: Option<&Json>,
    check: Option<&Errors>,
    save: Option<&Save>,
) -> u8 {
    let checker_only = |analysis: &cpl_check::Analysis| analysis.diagnostics.clone();
    let reported_of: &Errors = check.unwrap_or(&checker_only);
    // One database for the run, as the editor keeps one: std and shared
    // modules are checked once, and every step below reads the same analysis.
    let mut db = cpl_check::Session::new();
    let mut as_json = false;
    let mut fix = false;
    let mut taste = false;
    let mut to_save = false;
    let mut list = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--fix" => fix = true,
            "--taste" => taste = true,
            "--json" if json.is_some() || args.iter().any(|a| a == "--list") => as_json = true,
            "--save" if save.is_some() => to_save = true,
            "--list" => list = true,
            "-h" | "--help" => {
                let _ = write!(stdout, "{USAGE}");
                return 0;
            }
            flag if flag.starts_with('-') => {
                let message = cpl_diag::usage::no_option("lint", flag);
                let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
                return 2;
            }
            path => paths.push(PathBuf::from(path)),
        }
    }
    if list {
        if as_json {
            let _ = writeln!(stdout, "{}", list_json());
            return 0;
        }
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
    // Shown after the files: a name only a library's `@lint` declares is
    // known once that library has been linted.
    let mut bad_configs: Vec<(PathBuf, Vec<crate::ConfigError>)> = Vec::new();
    for file in &files {
        let name = file.display().to_string();
        let key = file.to_string_lossy().into_owned();
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
            if !problems.is_empty() && !bad_configs.iter().any(|(t, _)| *t == toml) {
                bad_configs.push((toml, problems));
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
            let (text, applied) = fix_checked_removed_syntax_in(&mut db, &key, &src);
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
                db.set_file(&key, src.as_str());
                let mut errors = (db.analyze(&key).as_deref())
                    .map(reported_of)
                    .unwrap_or_default();
                errors.retain(|d| d.severity == Severity::Error);
                show(stdout, &errors, &name, &src);
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
        db.set_file(&key, src.as_str());
        let mut analysis = db.analyze(&key).expect("the file was just set");
        let mut diagnostics = lint_analysis(&analysis, &config);
        diagnostics.retain(keep);
        if fix {
            let (text, applied) =
                apply_lint_fixes_where(&mut db, &key, src, &config, reported_of, &keep);
            fixed += applied;
            src = text;
            db.set_file(&key, src.as_str());
            analysis = db.analyze(&key).expect("the file was just set");
            diagnostics = lint_analysis(&analysis, &config);
            diagnostics.retain(keep);
            if let Err(e) = std::fs::write(file, &src) {
                let _ = writeln!(stderr, "error: can't write {name}: {e}");
                failed += 1;
            }
        }
        let found = checked_analysis(&analysis, &reported_of(&analysis), &config);
        let problems = found.problems;
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
        if let Some(save) = save.filter(|_| to_save) {
            for line in save(file, &src, &diagnostics) {
                let _ = writeln!(stderr, "{line}");
            }
        }
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
    for (toml, problems) in &bad_configs {
        for problem in problems {
            if problem
                .lint
                .as_deref()
                .is_some_and(crate::library::is_declared)
            {
                continue;
            }
            let _ = writeln!(stderr, "warning: {}: {problem}", toml.display());
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

/// `cpl lint --list --json`: `{"lints": [...]}`, every record in
/// `spec/lints/` and `spec/checker-lints/` in code order, keyed as the
/// record's `lint.toml` keys and file names.
pub fn list_json() -> String {
    use cpl_json::{obj, Value};
    let text = |s: &str| Value::String(s.to_string());
    let maybe = |s: Option<&str>| s.map_or(Value::Null, text);
    let mut all: Vec<&crate::Lint> = LINTS.iter().chain(crate::CHECKER_RECORDS).collect();
    all.sort_by_key(|l| (l.code.0, l.name));
    let lints = all
        .into_iter()
        .map(|l| {
            obj([
                ("code", text(l.code.0)),
                ("name", text(l.name)),
                ("dir", text(l.dir)),
                ("level", text(l.default.as_str())),
                ("group", text(l.group.as_str())),
                ("title", text(l.title)),
                ("summary", text(l.summary)),
                ("rule", maybe(l.rule)),
                ("no_fix", maybe(l.no_fix)),
                ("removed", maybe(l.removed)),
                ("family", maybe(l.family)),
                ("practice", maybe(l.practice)),
                ("explain", text(l.explanation.trim_end())),
                ("before", text(l.trigger.trim_end())),
                ("after", text(l.fix.trim_end())),
                ("near_miss", text(l.near_miss.trim_end())),
            ])
        })
        .collect();
    cpl_json::to_string(&obj([("lints", Value::Array(lints))]))
}
