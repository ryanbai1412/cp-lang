//! `cpl doc`.

use crate::{examples, extract, html, json, links, Module};
use std::io::Write;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "\
usage: cpl doc [options] [path]...

Writes documentation for the declarations and `///` comments of every .cp
file given. A directory stands for every .cp file under it; no path at
all means the current directory.

options:
  --out DIR          where the site goes (default: target/doc)
  --format html      a static site: index.html and one page per file (default)
  --format json      one JSON document on stdout, for another generator
  --title TEXT       the site's name (default: Documentation)
  --source-url URL   link every item to URL/<path>#L<line>
  --test             check the examples in doc comments instead of writing docs
";

/// Runs `cpl doc`. Returns the exit code: 0 on success, 1 when a file
/// could not be read or written or an example failed, 2 for a usage error.
pub fn run(args: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let mut out_dir = PathBuf::from("target/doc");
    let mut options = html::Options::default();
    let mut as_json = false;
    let mut test = false;
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = |flag: &str| {
            args.next()
                .cloned()
                .ok_or_else(|| cpl_diag::usage::needs_value(flag))
        };
        let result = match arg.as_str() {
            "--out" => value("--out").map(|dir| out_dir = PathBuf::from(dir)),
            "--title" => value("--title").map(|title| options.title = title),
            "--source-url" => value("--source-url").map(|url| options.source_url = Some(url)),
            "--format" => value("--format").and_then(|format| match format.as_str() {
                "html" | "json" => {
                    as_json = format == "json";
                    Ok(())
                }
                other => Err(format!("`--format` is html or json, not `{other}`")),
            }),
            "--test" => {
                test = true;
                Ok(())
            }
            "-h" | "--help" => {
                let _ = write!(stdout, "{USAGE}");
                return 0;
            }
            flag if flag.starts_with('-') => Err(cpl_diag::usage::no_option("doc", flag)),
            path => {
                paths.push(PathBuf::from(path));
                Ok(())
            }
        };
        if let Err(message) = result {
            let _ = write!(stderr, "cpl: {message}\n\n{USAGE}");
            return 2;
        }
    }
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }
    let mut files = Vec::new();
    for path in &paths {
        if let Err(e) = collect(path, &mut files) {
            let _ = writeln!(stderr, "cpl: can't read {}: {e}", path.display());
            return 1;
        }
    }
    files.sort();
    files.dedup();

    let mut modules: Vec<Module> = Vec::new();
    for file in &files {
        match std::fs::read_to_string(file) {
            Ok(src) => modules.push(extract(&src, &file.display().to_string())),
            Err(e) => {
                let _ = writeln!(stderr, "cpl: can't read {}: {e}", file.display());
                return 1;
            }
        }
    }

    let broken = links::unresolved(&modules);
    for link in &broken {
        let _ = writeln!(
            stderr,
            "{}:{}: [`{}`] in the comment of `{}` names no item; link a name in scope, or write plain `{}`",
            link.path, link.line, link.text, link.item, link.text
        );
    }
    let links_code = u8::from(!broken.is_empty());
    if test {
        return run_examples(&modules, stdout, stderr).max(links_code);
    }
    if as_json {
        let _ = write!(stdout, "{}", json::render(&modules));
        return links_code;
    }
    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        let _ = writeln!(stderr, "cpl: can't create {}: {e}", out_dir.display());
        return 1;
    }
    for (name, contents) in html::render(&modules, &options) {
        let path = out_dir.join(&name);
        if let Err(e) = std::fs::write(&path, contents) {
            let _ = writeln!(stderr, "cpl: can't write {}: {e}", path.display());
            return 1;
        }
    }
    let items: usize = modules.iter().map(|m| m.all_items().count()).sum();
    let undocumented = modules
        .iter()
        .flat_map(|m| m.all_items())
        .filter(|(_, item)| item.doc.is_empty())
        .count();
    let _ = writeln!(
        stderr,
        "documented {} in {}, {undocumented} without a comment; open {}",
        plural(items, "item"),
        plural(modules.len(), "file"),
        out_dir.join("index.html").display()
    );
    0
}

fn plural(n: usize, word: &str) -> String {
    format!("{n} {word}{}", if n == 1 { "" } else { "s" })
}

fn run_examples(modules: &[Module], stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let (mut checked, mut failed) = (0, 0);
    for module in modules {
        let report = examples::check(module);
        checked += report.checked;
        failed += report.failures.len();
        for failure in &report.failures {
            let what = if failure.diagnostics.is_empty() {
                "is prose; write it as backticked code"
            } else {
                "does not parse"
            };
            let _ = writeln!(
                stdout,
                "FAIL {}:{}: the example for `{}` {what}",
                module.path, failure.line, failure.item
            );
            // Positions inside the example, shown against its own text.
            let name = format!("{}:{} (example)", module.path, failure.line);
            let lines = cpl_diag::LineIndex::new(&failure.code);
            for diagnostic in &failure.diagnostics {
                let _ = writeln!(
                    stdout,
                    "{}",
                    cpl_diag::render(diagnostic, &name, &failure.code, &lines)
                );
            }
        }
    }
    let _ = writeln!(
        stderr,
        "{} checked, {failed} failed",
        plural(checked, "example")
    );
    (failed > 0) as u8
}

/// Adds `path`, or every `.cp` file under it, to `files`.
fn collect(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !path.is_dir() {
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
