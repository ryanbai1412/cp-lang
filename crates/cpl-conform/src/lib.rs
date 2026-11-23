pub mod conformance;
pub mod corpus;
pub mod diff;
pub mod docs;
mod lsp;
pub mod runner;
pub mod scoreboard;
pub mod spec_blocks;
pub mod variants;

use std::path::{Path, PathBuf};

use scoreboard::Scoreboard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suite {
    Conformance,
    Corpus,
    Docs,
    SpecBlocks,
    Diff,
}

impl Suite {
    pub fn name(self) -> &'static str {
        match self {
            Suite::Conformance => "conformance",
            Suite::Corpus => "corpus",
            Suite::Docs => "docs",
            Suite::SpecBlocks => "spec-blocks",
            Suite::Diff => "diff",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "conformance" => Some(Suite::Conformance),
            "corpus" => Some(Suite::Corpus),
            "docs" => Some(Suite::Docs),
            "spec-blocks" | "spec_blocks" => Some(Suite::SpecBlocks),
            "diff" => Some(Suite::Diff),
            _ => None,
        }
    }

    pub const ALL: &'static [Suite] = &[
        Suite::Conformance,
        Suite::Corpus,
        Suite::Docs,
        Suite::SpecBlocks,
        Suite::Diff,
    ];

    pub const FAST: &'static [Suite] = &[
        Suite::Conformance,
        Suite::Corpus,
        Suite::Docs,
        Suite::SpecBlocks,
    ];
}

/// Locates the repository root (the directory containing `spec/conformance`).
pub fn find_repo_root() -> Result<PathBuf, String> {
    if let Ok(cwd) = std::env::current_dir() {
        for dir in cwd.ancestors() {
            if dir.join("spec").join("conformance").is_dir() {
                return Ok(dir.to_path_buf());
            }
        }
    }
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        if dir.join("spec").join("conformance").is_dir() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("cannot locate repository root containing `spec/conformance`".into())
}

pub fn default_scoreboard_path(root: &Path) -> PathBuf {
    root.join("research").join("scoreboard.json")
}

/// Runs the requested suites.
pub fn run_suites(root: &Path, suites: &[Suite], filter: Option<&str>) -> Scoreboard {
    let mut sb = Scoreboard::default();
    for &suite in suites {
        let report = match suite {
            Suite::Conformance => conformance::run(root, filter),
            Suite::Corpus => corpus::run(root, filter),
            Suite::Docs => docs::run(root, filter),
            Suite::SpecBlocks => spec_blocks::run(root, filter),
            Suite::Diff => diff::run(root, filter),
        };
        sb.suites.insert(suite.name().to_string(), report);
    }
    sb
}
