//! A run whose memory passes its limit stops with an error naming the cases in
//! flight, instead of being OOM-killed (and its CI runner lost) without a word.

use std::path::Path;
use std::process::Command;

#[test]
#[cfg(target_os = "linux")]
fn a_run_over_its_memory_limit_stops_and_names_its_cases() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let case = "conformance/core-07/range-folds-are-ranges-own";
    let out = Command::new(env!("CARGO_BIN_EXE_cpl-conform"))
        .args(["diff", case])
        .current_dir(&root)
        .env("CPL_CONFORM_MAX_RSS_MB", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(3), "{stderr}");
    assert!(stderr.contains("over the 1 MB limit"), "{stderr}");
    assert!(stderr.contains(case), "{stderr}");
}
