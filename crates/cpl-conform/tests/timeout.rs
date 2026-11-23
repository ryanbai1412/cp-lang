//! A case that never finishes ends at its time limit, so a conformance run
//! always finishes.

use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn a_case_that_never_finishes_times_out() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut db = cpl_check::Database::new();
    for case in [
        "spec/conformance/core-07/range-folds-are-ranges-own.cp",
        "spec/conformance/core-10/spec-block-multitest-sieve.cp",
    ] {
        let path = root.join(case).to_string_lossy().into_owned();
        let text = std::fs::read_to_string(&path).unwrap();
        let Ok(compiled) = cpl_conform::runner::compile_program(&mut db, &path, &text) else {
            panic!("{case} does not compile");
        };
        let started = Instant::now();
        let ran = cpl_conform::runner::run_interp_within(&compiled.program, &path, &text, b"", 1);
        assert!(started.elapsed() < Duration::from_secs(10), "{case}");
        if let Err(error) = ran {
            assert_eq!(error, "the interpreter timed out after 1s", "{case}");
        }
    }
}
