//! The one Rust corpus walker: the problem packages under `corpus/problems`,
//! sorted by id, the way `scripts/corpus.py ls` lists them. Every Rust tool and
//! test reads the corpus through here.

use std::path::{Path, PathBuf};

/// Where the packages live, relative to the repository root.
pub const PACKAGES: &str = "corpus/problems";

/// The repository root.
pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every package directory under `root`, sorted by id.
pub fn packages(root: &Path) -> Vec<PathBuf> {
    let Ok(dirs) = std::fs::read_dir(root.join(PACKAGES)) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = dirs
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}

/// Each package's `leaf` file (`spec.cp`, `port.cp`, ...) that exists, sorted by id.
pub fn files(root: &Path, leaf: &str) -> Vec<PathBuf> {
    packages(root)
        .into_iter()
        .map(|d| d.join(leaf))
        .filter(|p| p.is_file())
        .collect()
}

/// The package directory of problem `id`, whether or not it exists.
pub fn package(root: &Path, id: &str) -> PathBuf {
    root.join(PACKAGES).join(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_packages_and_their_files_sorted_by_id() {
        let dir = std::env::temp_dir().join(format!("cpl-corpus-{}", std::process::id()));
        for (id, leaf) in [("b", "spec.cp"), ("a", "spec.cp"), ("c", "port.cp")] {
            std::fs::create_dir_all(package(&dir, id)).unwrap();
            std::fs::write(package(&dir, id).join(leaf), "").unwrap();
        }
        std::fs::write(dir.join(PACKAGES).join("README.md"), "").unwrap();
        let ids = |v: Vec<PathBuf>| -> Vec<String> {
            v.iter()
                .map(|p| {
                    p.strip_prefix(dir.join(PACKAGES))
                        .unwrap()
                        .display()
                        .to_string()
                })
                .collect()
        };
        assert_eq!(ids(packages(&dir)), ["a", "b", "c"]);
        assert_eq!(ids(files(&dir, "spec.cp")), ["a/spec.cp", "b/spec.cp"]);
        assert!(files(&dir.join("absent"), "spec.cp").is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn finds_the_repository_corpus() {
        assert!(files(&root(), "port.cp").len() > 100);
    }
}
