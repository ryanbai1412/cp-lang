//! `cpl check` and `cpl fix`: the front end's errors for each file, and
//! the fixes they carry applied.

use std::io::Write;

use crate::{json_out, EXIT_ERRORS, EXIT_USAGE};
pub(crate) use cpl_ir::fix::errors_in;

/// Checks each file through the shared front end and renders what it
/// finds.
pub(crate) fn check(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let json = args.contains(&"--json");
    let files = args.iter().filter(|arg| **arg != "--json");
    let mut reports = Vec::new();
    let mut db = cpl_check::Session::new();
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
        let (config, _) = cpl_lint::Config::discover(std::path::Path::new(path));
        let diagnostics = cpl_lint::at_levels(&analysis, &cpl_ir::reported(&analysis), &config);
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

pub(crate) fn fix(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let json = args.contains(&"--json");
    let files = args.iter().filter(|arg| **arg != "--json");
    let mut reports = Vec::new();
    let mut db = cpl_check::Session::new();
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
        let (config, _) = cpl_lint::Config::discover(std::path::Path::new(path));
        let (fixed, applied, left) = cpl_lsp::fix::document(&mut db, path, &text, &config);
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
