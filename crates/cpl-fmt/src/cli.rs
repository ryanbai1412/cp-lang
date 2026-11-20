//! `cpl fmt`.

use crate::{format, FormatError};
use cpl_diag::LineIndex;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl fmt [--check] [path]...

Rewrites each .cp file in the canonical style. A directory stands for every
.cp file under it; no path at all means the current directory.

options:
  --check    change nothing; list the files that would change and exit 1
  --stdin    format standard input to standard output
";

/// Runs `cpl fmt`. Returns the exit code: 0 when every file is formatted,
/// 1 when `--check` found one that is not or a file could not be formatted,
/// 2 for a usage error.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let mut check = false;
    let mut stdin = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            "--stdin" => stdin = true,
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
    if stdin {
        let mut src = String::new();
        if let Err(e) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut src) {
            let _ = writeln!(stderr, "error: can't read standard input: {e}");
            return 1;
        }
        return match format(&src) {
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
    for file in &files {
        let name = file.display().to_string();
        let src = match std::fs::read_to_string(file) {
            Ok(src) => src,
            Err(e) => {
                let _ = writeln!(stderr, "error: can't read {name}: {e}");
                failed += 1;
                continue;
            }
        };
        match format(&src) {
            Ok(out) if out == src => {}
            Ok(out) => {
                changed += 1;
                if check {
                    let _ = writeln!(stdout, "would reformat {name}");
                } else if let Err(e) = std::fs::write(file, out) {
                    let _ = writeln!(stderr, "error: can't write {name}: {e}");
                    failed += 1;
                } else {
                    let _ = writeln!(stdout, "formatted {name}");
                }
            }
            Err(e) => {
                report(stderr, &name, &src, &e);
                failed += 1;
            }
        }
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
    (failed > 0 || (check && changed > 0)) as u8
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
