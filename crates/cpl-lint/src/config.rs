//! Lint levels from `cp.toml`.
//!
//! ```toml
//! [lints]
//! style = "allow"         # a whole group: correctness, perf, idiom, style, pedantic
//! float_equality = "deny"
//! shadows_top_level = "allow"
//! memory_mb = 64          # the budget `memo_key_space` checks against
//! ```

use crate::CHECKER_LINTS;
use crate::{any_lint_named, closest_lint, Group, Level, Lint};
use cpl_diag::Code;
use std::collections::HashMap;
use std::path::Path;

/// The memory limit most judges use, in megabytes.
pub const DEFAULT_MEMORY_MB: u64 = 256;

/// Lint levels and limits for one project.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Config {
    levels: HashMap<&'static str, Level>,
    /// Levels of names no built-in lint has: a library's `@lint` may.
    named: HashMap<String, Level>,
    /// Levels set for a whole group; a lint's own entry wins.
    groups: HashMap<Group, Level>,
    /// What `memo_key_space` compares a memo table against.
    pub memory_mb: u64,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            levels: HashMap::new(),
            named: HashMap::new(),
            groups: HashMap::new(),
            memory_mb: DEFAULT_MEMORY_MB,
        }
    }
}

/// A line of `cp.toml` that `[lints]` cannot use.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ConfigError {
    /// 1-based.
    pub line: usize,
    pub message: String,
    /// The name, when the line sets a lint no built-in lint has: a
    /// library may declare it with `@lint`.
    pub lint: Option<String>,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl Config {
    /// The level `lint` runs at.
    pub fn level(&self, lint: &Lint) -> Level {
        self.levels
            .get(lint.name)
            .or_else(|| self.named.get(lint.name))
            .or_else(|| self.groups.get(&lint.group))
            .copied()
            .unwrap_or(lint.default)
    }

    pub fn set(&mut self, lint: &'static Lint, level: Level) {
        self.levels.insert(lint.name, level);
    }

    /// The level a lint the checker emits runs at; they warn by default.
    pub fn checker_level(&self, code: Code) -> Level {
        CHECKER_LINTS
            .iter()
            .filter(|lint| lint.code == code)
            .find_map(|lint| {
                self.levels
                    .get(lint.name)
                    .or_else(|| self.groups.get(&lint.group))
                    .copied()
            })
            .unwrap_or(Level::Warn)
    }

    /// Reads the `[lints]` table of a `cp.toml`. Other tables belong to
    /// other tools and are skipped. Every bad line is reported; the good
    /// ones still apply.
    pub fn from_toml(text: &str) -> (Config, Vec<ConfigError>) {
        let mut config = Config::default();
        let mut errors = Vec::new();
        for line in cpl_config::table(text, "lints") {
            let mut error = |message: String| {
                errors.push(ConfigError {
                    line: line.line,
                    message,
                    lint: None,
                })
            };
            let Some((key, value)) = line.pair else {
                error(format!(
                    "expected `name = \"level\"`, found `{}`",
                    line.text
                ));
                continue;
            };
            if key == "memory_mb" {
                match value.parse::<u64>() {
                    Ok(mb) if mb > 0 => config.memory_mb = mb,
                    _ => error(format!(
                        "`memory_mb` is a number of megabytes, not `{value}`"
                    )),
                }
                continue;
            }
            let level = |value: &str| {
                value
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .and_then(Level::parse)
            };
            if let Some(group) = Group::parse(key) {
                match level(value) {
                    Some(level) => {
                        config.groups.insert(group, level);
                    }
                    None => error(format!(
                        "the level of `{key}` is \"allow\", \"warn\" or \"deny\", not `{value}`"
                    )),
                }
                continue;
            }
            let Some(name) = any_lint_named(key) else {
                error(match closest_lint(key) {
                    Some(close) => {
                        format!("there is no lint called `{key}`; did you mean `{close}`?")
                    }
                    None => format!("there is no lint called `{key}`"),
                });
                if let Some(level) = level(value) {
                    config.named.insert(key.to_string(), level);
                    errors.last_mut().unwrap().lint = Some(key.to_string());
                }
                continue;
            };
            match level(value) {
                Some(level) => {
                    config.levels.insert(name, level);
                }
                None => error(format!(
                    "the level of `{key}` is \"allow\", \"warn\" or \"deny\", not `{value}`"
                )),
            }
        }
        errors.extend(
            cpl_config::check(text)
                .into_iter()
                .map(|(line, message)| ConfigError {
                    line,
                    message,
                    lint: None,
                }),
        );
        (config, errors)
    }

    /// The config for a source file: the nearest `cp.toml` at or above its
    /// directory, or the defaults when there is none. Returns the path it
    /// read with any bad lines.
    pub fn discover(source: &Path) -> (Config, Option<(std::path::PathBuf, Vec<ConfigError>)>) {
        match cpl_config::discover(source) {
            Some((file, text)) => {
                let (config, errors) = Config::from_toml(&text);
                (config, Some((file, errors)))
            }
            None => (Config::default(), None),
        }
    }
}
