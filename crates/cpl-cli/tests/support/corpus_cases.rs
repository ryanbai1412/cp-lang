//! The cases `cpl_contest::package` makes in `target/corpus-tests/<id>/` from
//! each package's tests.toml (problem-packages/R9): the one expansion `cpl test`
//! and `scripts/corpus.py make` use too. A test that runs a package's cases
//! makes them first, so it never depends on another job's cache.

use std::path::Path;

/// Makes the cases of the packages `ids` (a no-op for those already made).
pub fn make(root: &Path, ids: &[&str]) {
    for id in ids {
        let package = root.join("corpus/problems").join(id);
        let out = cpl_contest::package::cases_dir(&package);
        if let Err(why) = cpl_contest::package::make(&package, &out) {
            panic!("making {id}'s cases failed: {why}");
        }
    }
}
