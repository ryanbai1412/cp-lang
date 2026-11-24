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

/// The nearest [`FILE`] at or above `source`'s folder, and its text.
pub fn discover(source: &Path) -> Option<(PathBuf, String)> {
    let start = source.parent().unwrap_or(Path::new("."));
    let start = if start.as_os_str().is_empty() {
        Path::new(".")
    } else {
        start
    };
    let start = start.canonicalize().unwrap_or(start.to_path_buf());
    start.ancestors().find_map(|dir| {
        let file = dir.join(FILE);
        std::fs::read_to_string(&file).ok().map(|text| (file, text))
    })
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
];

#[cfg(test)]
mod tests {
    use super::*;

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
