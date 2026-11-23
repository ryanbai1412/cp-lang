//! `cpl fmt`.

use crate::{format_with, FormatError, Style};
use cpl_diag::LineIndex;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl fmt [--check] [--json] [path]...

Rewrites each .cp file in the canonical style. A directory stands for every
.cp file under it, so `cpl fmt .` formats the whole project. With no path,
standard input is formatted to standard output, or with `--check` the
current directory is checked; files are only rewritten when named.

options:
  --check    change nothing; list the files that would change and exit 1
  --stdin    format standard input to standard output, in the style of
             the cp.toml nearest the current directory
  --json     print one JSON object instead: `files`, each with its `path`,
             whether it `changed` (or would, under --check), `ok`, the
             `diagnostics` that stopped formatting and any other `error`
";

/// One file `cpl fmt` looked at, for a caller that prints `--json`.
pub struct FileReport {
    pub path: String,
    pub src: String,
    /// Formatting changed it, or would under `--check`.
    pub changed: bool,
    /// Formatted, or already in the canonical style.
    pub ok: bool,
    /// What the lexer could not read, when that stopped formatting.
    pub diagnostics: Vec<cpl_diag::Diagnostic>,
    /// Any other reason the file was left alone.
    pub error: Option<String>,
}

/// Turns the reports into the text `--json` prints.
pub type Json = dyn Fn(&[FileReport]) -> String;

/// Runs `cpl fmt`. Returns the exit code: 0 when every file is formatted,
/// 1 when `--check` found one that is not or a file could not be formatted,
/// 2 for a usage error.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    run_with(args, stdout, stderr, None)
}

/// `run`, taking `--json` when `json` can print it: `cpl-fmt` cannot see
/// the shared diagnostic JSON, which lives with the CLI.
pub fn run_with(
    args: &[String],
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    json: Option<&Json>,
) -> u8 {
    let mut check = false;
    let mut stdin = false;
    let mut as_json = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--stdin" => stdin = true,
            "--json" if json.is_some() => as_json = true,
            "-h" | "--help" => {
                let _ = write!(stdout, "{USAGE}");
                return 0;
            }
            flag if flag.starts_with('-') => {
                let _ = write!(stderr, "cpl: unknown option `{flag}`\n\n{USAGE}");
                return 2;
            }
            path => paths.push(PathBuf::from(path)),
        }
    }
    if stdin && as_json {
        let _ = write!(
            stderr,
            "cpl: `--json` reports on files, not `--stdin`\n\n{USAGE}"
        );
        return 2;
    }
    if paths.is_empty() && !check && !as_json {
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            let _ = write!(
                stderr,
                "cpl: name what to format: `cpl fmt .` rewrites every .cp file under the \
                 current directory, `cpl fmt --stdin` formats standard input\n\n{USAGE}"
            );
            return 2;
        }
        stdin = true;
    }
    if stdin {
        let mut src = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut src) {
            let _ = writeln!(stderr, "error: can't read standard input: {e}");
            return 1;
        }
        let (style, read) = Style::discover(Path::new("<stdin>.cp"));
        for error in read.into_iter().flat_map(|(toml, errors)| {
            errors
                .into_iter()
                .map(move |e| format!("{}: {e}", toml.display()))
        }) {
            let _ = writeln!(stderr, "{error}");
        }
        return match format_with(&src, &style) {
            Ok(out) => {
                let _ = write!(stdout, "{out}");
                0
            }
            Err(e) => {
                report(stderr, "<stdin>", &src, &e);
                1
            }
        };
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

    let (mut changed, mut failed) = (0usize, 0usize);
    let mut reported = std::collections::HashSet::new();
    let mut reports = Vec::new();
    for file in &files {
        let name = file.display().to_string();
        let mut entry = FileReport {
            path: name.clone(),
            src: String::new(),
            changed: false,
            ok: false,
            diagnostics: Vec::new(),
            error: None,
        };
        match std::fs::read_to_string(file) {
            Ok(src) => entry.src = src,
            Err(e) => entry.error = Some(format!("can't read {name}: {e}")),
        }
        if entry.error.is_none() {
            let (style, read) = Style::discover(file);
            if let Some((toml, errors)) = read {
                if reported.insert(toml.clone()) {
                    for error in errors {
                        let _ = writeln!(stderr, "{}: {error}", toml.display());
                    }
                }
            }
            match format_with(&entry.src, &style) {
                Ok(out) if out == entry.src => entry.ok = true,
                Ok(out) => {
                    entry.changed = true;
                    if !check {
                        if let Err(e) = std::fs::write(file, out) {
                            entry.error = Some(format!("can't write {name}: {e}"));
                        }
                    }
                    entry.ok = entry.error.is_none();
                }
                Err(e) => {
                    entry.error = Some(format!("not formatted: {e}"));
                    if let FormatError::Lex(diags) = e {
                        entry.diagnostics = diags;
                    }
                }
            }
        }
        changed += entry.changed as usize;
        failed += !entry.ok as usize;
        if !as_json {
            print_text(&entry, check, stdout, stderr);
        }
        reports.push(entry);
    }
    let code = (failed > 0 || (check && changed > 0)) as u8;
    if let (true, Some(json)) = (as_json, json) {
        let _ = writeln!(stdout, "{}", json(&reports));
        return code;
    }
    let files_word = |n: usize| if n == 1 { "file" } else { "files" };
    let verb = if check { "would change" } else { "changed" };
    let _ = writeln!(
        stderr,
        "{} {} checked, {changed} {verb}{}",
        files.len(),
        files_word(files.len()),
        if failed > 0 {
            format!(", {failed} not formatted")
        } else {
            String::new()
        }
    );
    code
}

/// The lines a person reads for one file.
fn print_text(entry: &FileReport, check: bool, stdout: &mut dyn Write, stderr: &mut dyn Write) {
    let name = &entry.path;
    let lines = LineIndex::new(&entry.src);
    for diag in &entry.diagnostics {
        let _ = writeln!(
            stderr,
            "{}",
            cpl_diag::render(diag, name, &entry.src, &lines)
        );
    }
    if let Some(error) = &entry.error {
        if error.starts_with("can't") {
            let _ = writeln!(stderr, "error: {error}");
        } else {
            let _ = writeln!(stderr, "{name}: {error}");
        }
    } else if entry.changed && check {
        let _ = writeln!(stdout, "would reformat {name}");
    } else if entry.changed {
        let _ = writeln!(stdout, "formatted {name}");
    }
}

fn report(stderr: &mut dyn Write, name: &str, src: &str, error: &FormatError) {
    match error {
        FormatError::Lex(diags) => {
            let lines = LineIndex::new(src);
            for diag in diags {
                let _ = writeln!(stderr, "{}", cpl_diag::render(diag, name, src, &lines));
            }
            let _ = writeln!(stderr, "{name}: not formatted: {error}");
        }
        FormatError::Internal(_) => {
            let _ = writeln!(stderr, "{name}: not formatted: {error}");
        }
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
