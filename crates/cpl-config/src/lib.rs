//! The one reader of `cp.toml`: the nearest one above a source file, the
//! `key = value` lines of one of its tables, and [`SCHEMA`], every table
//! and key a consumer reads. Each tool parses its own values; none finds
//! the file or splits its lines itself.

use std::path::{Path, PathBuf};

/// A project's config file.
pub const FILE: &str = "cp.toml";

/// The nearest folder at or above `dir` that holds a [`FILE`].
pub fn project_dir(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .find(|d| d.join(FILE).is_file())
        .map(Path::to_path_buf)
}

/// The first `found(dir, text)` over each [`FILE`] at or above `dir`,
/// nearest first, read through `read`: the disk, or the files a browser
/// gives.
pub fn find_map<T>(
    dir: &Path,
    read: &mut dyn FnMut(&Path) -> Option<String>,
    mut found: impl FnMut(&Path, &str) -> Option<T>,
) -> Option<T> {
    dir.ancestors()
        .find_map(|d| read(&d.join(FILE)).and_then(|text| found(d, &text)))
}

/// The nearest [`FILE`] at or above `source`'s folder, and its text.
pub fn discover(source: &Path) -> Option<(PathBuf, String)> {
    let start = source.parent().unwrap_or(Path::new("."));
    let start = if start.as_os_str().is_empty() {
        Path::new(".")
    } else {
        start
    };
    let start = start.canonicalize().unwrap_or(start.to_path_buf());
    find_map(
        &start,
        &mut |f| std::fs::read_to_string(f).ok(),
        |dir, text| Some((dir.join(FILE), text.to_string())),
    )
}

/// One line of a table: `key = value`, or a line that is not one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Line<'a> {
    /// 1-based.
    pub line: usize,
    /// `Some((key, value))`, both trimmed; `None` for a line with no `=`.
    pub pair: Option<(&'a str, &'a str)>,
    /// The line without its comment, trimmed.
    pub text: &'a str,
}

/// The non-empty lines of the `[table]` table, comments stripped.
pub fn table<'a>(text: &'a str, table: &str) -> Vec<Line<'a>> {
    let mut out = Vec::new();
    let mut inside = false;
    for (index, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            inside = header.trim_end_matches(']').trim() == table;
            continue;
        }
        if inside {
            let pair = line.split_once('=').map(|(k, v)| (k.trim(), v.trim()));
            out.push(Line {
                line: index + 1,
                pair,
                text: line,
            });
        }
    }
    out
}

/// The line without its `# comment`, leaving a `#` inside a string alone.
pub fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (at, c) in line.char_indices() {
        match c {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..at],
            _ => {}
        }
    }
    line
}

/// A table of [`FILE`] and the keys its reader takes.
pub struct Table {
    pub name: &'static str,
    /// The crate that reads it.
    pub reader: &'static str,
    pub doc: &'static str,
    /// Each key and what it sets; `*` is any name the reader accepts.
    pub keys: &'static [(&'static str, &'static str)],
}

/// Every table a tool reads from [`FILE`].
pub const SCHEMA: &[Table] = &[
    Table {
        name: "fmt",
        reader: "cpl-fmt",
        doc: "The layout `cpl fmt` writes.",
        keys: &[
            ("indent_width", "spaces per indent, 2 to 8 (default 4)"),
            (
                "line_width",
                "the width a line wraps at, 40 to 200 (default 100)",
            ),
        ],
    },
    Table {
        name: "lints",
        reader: "cpl-lint",
        doc: "Lint levels: each lint or group is \"allow\", \"warn\" or \"deny\".",
        keys: &[
            (
                "memory_mb",
                "the memory a solution may use, for memory lints",
            ),
            ("*", "a lint or group name: its level"),
        ],
    },
    Table {
        name: "libs",
        reader: "cpl-check",
        doc: "Libraries a program may `use`, each by the folder it is in.",
        keys: &[(
            "*",
            "a library name: `\"path\"` or `{ path = \"path\" }`, relative to this file or `~/`",
        )],
    },
    Table {
        name: "limits",
        reader: "cpl-contest",
        doc: "A problem's limits, under its own `// limits:` line.",
        keys: &[
            ("time_ms", "the time limit in milliseconds"),
            ("memory_mb", "the memory limit in megabytes"),
        ],
    },
    Table {
        name: "judge",
        reader: "cpl-judge",
        doc: "The judge profile a build targets, under `--judge` and `// judge:`.",
        keys: &[("judge", "a profile's name in quotes")],
    },
    Table {
        name: "judge.*",
        reader: "cpl-judge",
        doc: "Changes to a judge profile; one that isn't built in says what it `extends`.",
        keys: &[
            ("extends", "the built-in profile it starts from"),
            ("*", "a profile key (spec/tools/judge-profiles.md)"),
        ],
    },
];

/// The table of `SCHEMA` a `[name]` header opens: `judge.fast` opens `judge.*`.
pub fn schema_table(name: &str) -> Option<&'static Table> {
    SCHEMA.iter().find(|t| match t.name.strip_suffix('*') {
        Some(prefix) => name.len() > prefix.len() && name.starts_with(prefix),
        None => t.name == name,
    })
}

/// Each line of `text` that `SCHEMA` has no place for, 1-based, with why:
/// an unknown table, a key its table doesn't have, or a key outside any
/// table. The values are their readers' to check.
pub fn check(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    // `None` before the first table; `Some(None)` in one `SCHEMA` lacks.
    let mut table = None;
    for (index, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(header) = line.strip_prefix('[') {
            let name = header.trim_end_matches(']').trim();
            table = Some(schema_table(name));
            if matches!(table, Some(None)) {
                out.push((index + 1, format!("`[{name}]` is not a `{FILE}` table")));
            }
            continue;
        }
        let key = line.split_once('=').map_or(line, |(k, _)| k.trim());
        match table {
            None => out.push((index + 1, format!("`{key}` is outside any table"))),
            Some(Some(t)) if !t.keys.iter().any(|(k, _)| *k == key || *k == "*") => {
                out.push((index + 1, format!("`[{}]` has no key `{key}`", t.name)))
            }
            Some(_) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `spec/tools/cp-toml.md`, the page `SCHEMA` writes.
    fn page() -> String {
        let mut out = String::from(
            "# `cp.toml`\n\n<!-- Generated from `SCHEMA` in crates/cpl-config/src/lib.rs; \
             `BLESS=1 cargo test -p cpl-config` rewrites it. -->\n\n\
             The nearest `cp.toml` at or above a program's folder configures the tools. \
             `cpl lint` reports a table or key not listed here.\n",
        );
        for t in SCHEMA {
            let name = t.name.replace('*', "<name>");
            out += &format!("\n## `[{name}]`\n\n{} Read by `{}`.\n\n", t.doc, t.reader);
            out += "| Key | Meaning |\n|---|---|\n";
            for (key, doc) in t.keys {
                let key = if *key == "*" { "<name>" } else { key };
                out += &format!("| `{key}` | {doc} |\n");
            }
        }
        out
    }

    #[test]
    fn the_cp_toml_page_is_the_schema() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/tools/cp-toml.md");
        if std::env::var_os("BLESS").is_some() {
            std::fs::write(path, page()).unwrap();
        }
        let on_disk = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            on_disk == page(),
            "spec/tools/cp-toml.md is stale: BLESS=1 cargo test -p cpl-config"
        );
    }

    #[test]
    fn check_names_what_the_schema_has_no_place_for() {
        let text = "top = 1\n[fmt]\nindent_width = 2\nindent = 2\n[lints]\nanything = \"deny\"\n\
                    [judge.fast]\nextends = \"cf\"\n[judges]\nx = 1\n";
        assert_eq!(
            check(text),
            vec![
                (1, "`top` is outside any table".to_string()),
                (4, "`[fmt]` has no key `indent`".to_string()),
                (9, "`[judges]` is not a `cp.toml` table".to_string()),
            ]
        );
        assert!(schema_table("judge.").is_none());
    }

    #[test]
    fn a_table_is_its_own_lines_without_comments() {
        let text = "[fmt]\nindent_width = 2 # two\n\n[lints]\nx = \"a#b\"\nbad\n";
        let lints = table(text, "lints");
        assert_eq!(lints[0].pair, Some(("x", "\"a#b\"")));
        assert_eq!(
            (lints[1].line, lints[1].pair, lints[1].text),
            (6, None, "bad")
        );
        assert_eq!(table(text, "fmt")[0].pair, Some(("indent_width", "2")));
        assert!(table(text, "judge").is_empty());
    }

    #[test]
    fn every_table_is_named_once() {
        let mut names: Vec<_> = SCHEMA.iter().map(|t| t.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), SCHEMA.len());
    }
}
