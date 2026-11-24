//! `--check` fails when the scoreboard records a passing case as failing, and
//! tells the author to `--write` it: a ratchet must not let a stale record live.

use cpl_conform::scoreboard::Scoreboard;
use std::path::Path;
use std::process::Command;

#[test]
fn check_fails_on_a_recorded_failure_that_now_passes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let case = "core-04/rebased-str-indexes-from-zero.cp";
    let mut board = Scoreboard::load(&root.join("research/scoreboard.json")).unwrap();
    let recorded = board
        .suites
        .get_mut("conformance")
        .unwrap()
        .cases
        .get_mut(case)
        .unwrap();
    assert_eq!(recorded, "pass", "{case} must pass for this test");
    *recorded = "fail".to_string();
    let dir = std::env::temp_dir().join(format!("cpl-conform-stale-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let stale = dir.join("scoreboard.json");
    board.save(&stale).unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_cpl-conform"))
        .args([
            "conformance",
            "rebased-str-indexes-from-zero",
            "--check",
            "--scoreboard",
        ])
        .arg(&stale)
        .current_dir(&root)
        .output()
        .unwrap();
    std::fs::remove_dir_all(&dir).ok();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "a stale record passed --check: {stderr}"
    );
    assert!(stderr.contains("records as failing now pass"), "{stderr}");
    assert!(
        stderr.contains("cargo run -p cpl-conform -- <suite> --write"),
        "{stderr}"
    );
    assert!(
        stderr.contains(&format!("conformance/{case}: recorded as fail but passes")),
        "{stderr}"
    );
}
