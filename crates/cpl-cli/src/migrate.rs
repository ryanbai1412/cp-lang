//! `cpl migrate`: rewrites old spellings in every solution and library
//! file to the current ones (spec/core/11-modules-prelude.md, R11.4.3).
//!
//! A migration is a diagnostic in `MIGRATIONS` whose fix says the new
//! spelling; a removed form without a mechanical rewrite is reported, with
//! its help, for a hand edit. Recording pins waits on `cp.toml` pins
//! (decisions.md 251).

use crate::check::errors_in;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) const USAGE: &str = "\
usage: cpl migrate [path]... [--check] [--json]

Rewrites spellings removed in newer CP Lang versions in each .cp file
under the paths (default: the current folder), shows the diff, and lists
what needs a hand edit.

options:
  --check   change nothing; exit 1 if a file would change or needs an edit
  --json    print each file's changes as JSON
";

/// Diagnostics whose fix is a migration: a removed spelling (E0503).
const MIGRATIONS: &[cpl_diag::Code] = &[cpl_diag::codes::REMOVED_SYNTAX];

fn is_migration(d: &cpl_diag::Diagnostic) -> bool {
    MIGRATIONS.contains(&d.code)
}

/// `text` with every migration fix applied, how many, and the migrations
/// left that have no fix.
fn migrate_text(path: &str, text: &str) -> (String, usize, Vec<cpl_diag::Diagnostic>) {
    let mut db = cpl_check::Database::new();
    let mut text = text.to_string();
    let mut applied = 0;
    for _ in 0..8 {
        let mut found = errors_in(&mut db, path, &text);
        found.retain(is_migration);
        let (next, n) = cpl_lint::apply_fixes(&text, &found);
        if n == 0 || next == text {
            break;
        }
        applied += n;
        text = next;
    }
    let mut left = errors_in(&mut db, path, &text);
    left.retain(is_migration);
    (text, applied, left)
}

pub(crate) fn migrate(args: &[&str], stdout: &mut (dyn Write + Send), stderr: &mut String) -> u8 {
    let json = args.contains(&"--json");
    let check = args.contains(&"--check");
    let mut roots: Vec<&str> = args
        .iter()
        .copied()
        .filter(|a| !a.starts_with("--"))
        .collect();
    if roots.is_empty() {
        roots.push(".");
    }
    let mut files = Vec::new();
    for root in roots {
        if let Err(error) = collect(Path::new(root), &mut files) {
            *stderr += &format!("cpl: can't read `{root}`: {error}\n");
            return crate::EXIT_USAGE;
        }
    }
    files.sort();
    let mut code = 0;
    let mut reports = Vec::new();
    for file in &files {
        let path = file.display().to_string();
        let Ok(text) = std::fs::read_to_string(file) else {
            *stderr += &format!("cpl: can't read `{path}`\n");
            code = crate::EXIT_USAGE;
            continue;
        };
        let (migrated, applied, left) = migrate_text(&path, &text);
        let changed = migrated != text;
        if (changed || !left.is_empty()) && check {
            code = code.max(crate::EXIT_ERRORS);
        }
        if !left.is_empty() {
            code = code.max(crate::EXIT_ERRORS);
        }
        if changed && !check {
            if let Err(error) = std::fs::write(file, &migrated) {
                *stderr += &format!("cpl: can't write `{path}`: {error}\n");
                code = crate::EXIT_USAGE;
                continue;
            }
        }
        if json {
            let lines = cpl_diag::LineIndex::new(&migrated);
            let left = left.iter().map(|d| crate::json_out::diagnostic(d, &lines));
            reports.push(cpl_json::obj([
                ("path", cpl_json::Value::String(path)),
                ("applied", applied.into()),
                ("changed", cpl_json::Value::Bool(changed)),
                ("diagnostics", cpl_json::Value::Array(left.collect())),
            ]));
            continue;
        }
        if changed {
            let _ = stdout.write_all(diff(&path, &text, &migrated).as_bytes());
        }
        if !left.is_empty() {
            *stderr += &cpl_diag::render_all(&left, &path, &migrated);
        }
    }
    if json {
        let files = cpl_json::Value::Array(reports);
        let _ = writeln!(
            stdout,
            "{}",
            cpl_json::to_string(&cpl_json::obj([("files", files)]))
        );
    } else if code == 0 {
        let _ = writeln!(stdout, "{} files use the current spellings", files.len());
    }
    code
}

/// Adds `path`, or every `.cp` file under it, to `files`: brutes and
/// generators in `d.tests/` too, but not hidden folders or `target`.
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
        if child.is_dir() {
            if !name.to_string_lossy().starts_with('.') && name != "target" {
                collect(&child, files)?;
            }
        } else if child.extension().is_some_and(|e| e == "cp") {
            files.push(child);
        }
    }
    Ok(())
}

/// The changed lines between `old` and `new`, one hunk per run of changes,
/// as `-`/`+` lines under `@@ line N @@`.
fn diff(path: &str, old: &str, new: &str) -> String {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut out = format!("--- {path}\n+++ {path}\n");
    for (at, removed, added) in hunks(&a, &b) {
        out += &format!("@@ line {} @@\n", at + 1);
        for line in removed {
            out += &format!("-{line}\n");
        }
        for line in added {
            out += &format!("+{line}\n");
        }
    }
    out
}

type Hunk<'a> = (usize, Vec<&'a str>, Vec<&'a str>);

/// Runs of changed lines, from a longest common subsequence of the lines
/// between the common head and tail (migrations change few lines).
fn hunks<'a>(a: &[&'a str], b: &[&'a str]) -> Vec<Hunk<'a>> {
    let head = a.iter().zip(b).take_while(|(x, y)| x == y).count();
    let tail = a[head..]
        .iter()
        .rev()
        .zip(b[head..].iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (a_mid, b_mid) = (&a[head..a.len() - tail], &b[head..b.len() - tail]);
    if a_mid.len() * b_mid.len() > 4_000_000 {
        return vec![(head, a_mid.to_vec(), b_mid.to_vec())];
    }
    let (n, m) = (a_mid.len(), b_mid.len());
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if a_mid[i] == b_mid[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut out: Vec<Hunk<'a>> = Vec::new();
    let mut open: Option<Hunk<'a>> = None;
    while i < n || j < m {
        if i < n && j < m && a_mid[i] == b_mid[j] {
            out.extend(open.take());
            i += 1;
            j += 1;
            continue;
        }
        let hunk = open.get_or_insert_with(|| (head + i, Vec::new(), Vec::new()));
        if j < m && (i == n || lcs[i][j + 1] >= lcs[i + 1][j]) {
            hunk.2.push(b_mid[j]);
            j += 1;
        } else {
            hunk.1.push(a_mid[i]);
            i += 1;
        }
    }
    out.extend(open);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hunks_are_the_runs_of_changed_lines() {
        let a = ["a", "b", "c", "d", "e"];
        let b = ["a", "B", "c", "d", "E", "f"];
        assert_eq!(
            hunks(&a, &b),
            [(1, vec!["b"], vec!["B"]), (4, vec!["e"], vec!["E", "f"])]
        );
        assert!(hunks(&a, &a).is_empty());
    }

    #[test]
    fn an_old_spelling_is_rewritten_and_one_without_a_rewrite_is_left() {
        let old = "if true:\n    print(1)\nelse if false:\n    print(2)\n";
        let (new, applied, left) = migrate_text("a.cp", old);
        assert_eq!(
            (new.as_str(), applied),
            ("if true:\n    print(1)\nelif false:\n    print(2)\n", 1)
        );
        assert!(left.is_empty());
        let (same, applied, left) = migrate_text("a.cp", "for mut i in 0..3:\n    print(i)\n");
        assert_eq!(
            (same.as_str(), applied),
            ("for mut i in 0..3:\n    print(i)\n", 0)
        );
        assert_eq!(left.len(), 1);
    }

    #[test]
    fn other_errors_are_not_migrations() {
        let (same, applied, left) = migrate_text("a.cp", "let x: i32 = \"s\"\n");
        assert_eq!(
            (same.as_str(), applied, left.len()),
            ("let x: i32 = \"s\"\n", 0, 0)
        );
    }
}
