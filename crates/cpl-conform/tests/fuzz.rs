//! compiler-changes/R4: generated programs are valid, and both engines
//! agree on them at every `--checks` level.

use cpl_check::Session;
use cpl_codegen::Checks;
use cpl_conform::{fuzz, gen};

#[test]
fn a_seed_always_gives_the_same_program() {
    assert_eq!(gen::program(7), gen::program(7));
    assert_ne!(gen::program(7), gen::program(8));
}

#[test]
fn generated_programs_check_run_and_agree() {
    let cache_dir = std::env::temp_dir().join("cpl-conform-fuzz-test-cxx");
    let cxx = std::process::Command::new("g++")
        .arg("--version")
        .output()
        .is_ok();
    let levels = [Checks::None, Checks::Traps, Checks::All];
    let mut db = Session::new();
    let found: Vec<_> = (0..40)
        .filter_map(|seed| fuzz::check(&mut db, &cache_dir, seed, &levels, cxx && seed < 12).err())
        .map(|f| (f.seed, f.kind, f.detail))
        .collect();
    assert!(
        found.is_empty(),
        "rerun with `cpl-conform fuzz SEED 1`: {found:?}"
    );
}
