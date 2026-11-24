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
        for line in cpl_config::table(text, "fmt") {
            let mut error = |message: String| {
                errors.push(StyleError {
                    line: line.line,
                    message,
                })
            };
            let Some((key, value)) = line.pair else {
                error(format!("expected `key = number`, found `{}`", line.text));
                continue;
            };
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
        // The browser build has no files to find a `cp.toml` in.
        if cfg!(target_family = "wasm") {
            return (Style::default(), None);
        }
        match cpl_config::discover(source) {
            Some((file, text)) => {
                let (style, errors) = Style::from_toml(&text);
                (style, Some((file, errors)))
            }
            None => (Style::default(), None),
        }
    }
}
