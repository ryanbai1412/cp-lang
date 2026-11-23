//! The two layout choices a project may change, from `cp.toml`:
//!
//! ```toml
//! [fmt]
//! indent_width = 2   # spaces per level, 2 to 8; 4 by default
//! line_width = 80    # columns, 40 to 200; 100 by default
//! ```
//!
//! Everything else about the layout is fixed.

use std::path::{Path, PathBuf};

/// How `cpl fmt` lays a file out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    /// Spaces per indentation level.
    pub indent: usize,
    /// Lines are broken to stay within this many columns where a break is
    /// legal.
    pub line_width: usize,
}

impl Default for Style {
    fn default() -> Style {
        Style {
            indent: crate::INDENT,
            line_width: crate::MAX_WIDTH,
        }
    }
}

/// A bad line in `[fmt]`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StyleError {
    /// 1-based.
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for StyleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Style {
    /// Reads the `[fmt]` table of a `cp.toml`; other tables are skipped.
    /// Every bad line is reported, and the good ones still apply.
    pub fn from_toml(text: &str) -> (Style, Vec<StyleError>) {
        let mut style = Style::default();
        let mut errors = Vec::new();
        let mut in_fmt = false;
        for (index, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(table) = line.strip_prefix('[') {
                in_fmt = table.trim_end_matches(']').trim() == "fmt";
                continue;
            }
            if !in_fmt {
                continue;
            }
            let mut error = |message: String| {
                errors.push(StyleError {
                    line: index + 1,
                    message,
                })
            };
            let Some((key, value)) = line.split_once('=') else {
                error(format!("expected `key = number`, found `{line}`"));
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            let (slot, range) = match key {
                "indent_width" => (&mut style.indent, 2..=8),
                "line_width" => (&mut style.line_width, 40..=200),
                _ => {
                    error(format!(
                        "`[fmt]` has `indent_width` and `line_width`, not `{key}`"
                    ));
                    continue;
                }
            };
            match value.parse::<usize>() {
                Ok(n) if range.contains(&n) => *slot = n,
                _ => error(format!(
                    "`{key}` is a number from {} to {}, not `{value}`",
                    range.start(),
                    range.end()
                )),
            }
        }
        (style, errors)
    }

    /// The style for a source file: the nearest `cp.toml` at or above its
    /// directory, or the defaults. Returns the path it read with any bad
    /// lines.
    pub fn discover(source: &Path) -> (Style, Option<(PathBuf, Vec<StyleError>)>) {
        let start = source.parent().unwrap_or(Path::new("."));
        let start = if start.as_os_str().is_empty() {
            Path::new(".")
        } else {
            start
        };
        let start = start.canonicalize().unwrap_or(start.to_path_buf());
        for dir in start.ancestors() {
            let candidate = dir.join("cp.toml");
            if let Ok(text) = std::fs::read_to_string(&candidate) {
                let (style, errors) = Style::from_toml(&text);
                return (style, Some((candidate, errors)));
            }
        }
        (Style::default(), None)
    }
}
