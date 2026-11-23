//! `cpl check` and `cpl fix`: the front end's errors for each file, and
//! the fixes they carry applied.

use std::io::Write;

use crate::{json_out, EXIT_ERRORS, EXIT_USAGE};
pub(crate) use cpl_ir::fix::{apply_error_fixes_in, errors_in};

/// Checks each file through the shared front end and renders what it
/// finds.
pub(crate) fn check(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let json = args.contains(&"--json");
    let files = args.iter().filter(|arg| **arg != "--json");
    let mut reports = Vec::new();
    let mut db = cpl_check::Database::new();
    let mut code = 0;
    for path in files {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        };
        db.set_file(path, text.as_str());
        // What `cpl run` and `cpl build` would reject, `cpl check` rejects.
        let analysis = db.analyze(path).expect("the file was just set");
        let diagnostics = cpl_ir::reported(&analysis);
        let failed = diagnostics
            .iter()
            .any(|d| d.severity == cpl_diag::Severity::Error);
        if json {
            let lines = cpl_diag::LineIndex::new(&text);
            let diagnostics = diagnostics.iter().map(|d| json_out::diagnostic(d, &lines));
            reports.push(cpl_json::obj([
                ("path", cpl_json::Value::String(path.to_string())),
                ("ok", cpl_json::Value::Bool(!failed)),
                ("diagnostics", cpl_json::Value::Array(diagnostics.collect())),
            ]));
        } else if !diagnostics.is_empty() {
            *stderr += &cpl_diag::render_all(&diagnostics, path, &text);
        }
        if json {
            if failed && code == 0 {
                code = EXIT_ERRORS;
            }
        } else if !failed {
            let _ = writeln!(stdout, "{path}: no errors");
        } else if code == 0 {
            code = EXIT_ERRORS;
        }
    }
    if json {
        let files = cpl_json::Value::Array(reports);
        let _ = writeln!(
            stdout,
            "{}",
            cpl_json::to_string(&cpl_json::obj([("files", files)]))
        );
    }
    code
}

/// The errors `cpl check` prints for `text` read as the file at `path`;
/// `cpl lint` shows this same list.
pub(crate) fn reported_errors(path: &std::path::Path, text: &str) -> Vec<cpl_diag::Diagnostic> {
    let key = path.to_string_lossy();
    let mut db = cpl_check::Database::new();
    db.set_file(&key, text);
    let analysis = db.analyze(&key).expect("the file was just set");
    let mut out = cpl_ir::reported(&analysis);
    out.retain(|d| d.severity == cpl_diag::Severity::Error);
    out
}

/// `text` with the lint fixes `cpl lint --fix` would make, and how many;
/// each round lints the analysis `db` has, so a clean file is checked once.
fn apply_lint_fixes(db: &mut cpl_check::Database, path: &str, mut text: String) -> (String, usize) {
    let (config, _) = cpl_lint::Config::discover(std::path::Path::new(path));
    let mut applied = 0;
    for _ in 0..8 {
        db.set_file(path, text.as_str());
        let analysis = db.analyze(path).expect("the file was just set");
        let (next, n) = cpl_lint::apply_fixes(&text, &cpl_lint::lint_analysis(&analysis, &config));
        if n == 0 {
            break;
        }
        applied += n;
        text = next;
    }
    (text, applied)
}

pub(crate) fn fix(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let json = args.contains(&"--json");
    let files = args.iter().filter(|arg| **arg != "--json");
    let mut reports = Vec::new();
    let mut db = cpl_check::Database::new();
    let mut code = 0;
    for path in files {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) => {
                *stderr += &format!("cpl: can't read `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        };
        let (fixed, mut applied, left) = apply_error_fixes_in(&mut db, path, &text);
        let fixed = if left.is_empty() {
            let (linted, n) = apply_lint_fixes(&mut db, path, fixed);
            applied += n;
            linted
        } else {
            fixed
        };
        if applied > 0 {
            if let Err(error) = std::fs::write(path, &fixed) {
                *stderr += &format!("cpl: can't write `{path}`: {error}\n");
                code = EXIT_USAGE;
                continue;
            }
        }
        if !left.is_empty() && code == 0 {
            code = EXIT_ERRORS;
        }
        if json {
            let lines = cpl_diag::LineIndex::new(&fixed);
            let diagnostics = left.iter().map(|d| json_out::diagnostic(d, &lines));
            reports.push(cpl_json::obj([
                ("path", cpl_json::Value::String(path.to_string())),
                ("applied", applied.into()),
                ("ok", cpl_json::Value::Bool(left.is_empty())),
                ("diagnostics", cpl_json::Value::Array(diagnostics.collect())),
            ]));
            continue;
        }
        if !left.is_empty() {
            *stderr += &cpl_diag::render_all(&left, path, &fixed);
        }
        let fixes = if applied == 1 { "fix" } else { "fixes" };
        let _ = match left.len() {
            0 => writeln!(stdout, "{path}: {applied} {fixes} applied, no errors"),
            n => writeln!(stdout, "{path}: {applied} {fixes} applied, {n} left"),
        };
    }
    if json {
        let files = cpl_json::Value::Array(reports);
        let _ = writeln!(
            stdout,
            "{}",
            cpl_json::to_string(&cpl_json::obj([("files", files)]))
        );
    }
    code
}
