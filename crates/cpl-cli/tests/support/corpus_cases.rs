//! The cases `scripts/corpus.py make` writes to `target/corpus-tests/<id>/` from
//! each package's tests.toml (problem-packages/R9). A test that runs a
//! package's cases makes them first, so it never depends on another job's cache.

use std::path::Path;
use std::process::Command;

/// Makes the cases of the packages `ids` (a no-op for those already made).
pub fn make(root: &Path, ids: &[&str]) {
    let out = Command::new("uv")
        .args(["run", "--quiet", "python", "scripts/corpus.py", "make"])
        .args(ids)
        .current_dir(root)
        .output()
        .expect("uv runs scripts/corpus.py make");
    assert!(
        out.status.success(),
        "scripts/corpus.py make {ids:?} failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
