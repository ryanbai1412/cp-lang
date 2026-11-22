//! Integration tests for `cpl report` and the ICE crash bundle flow.

use std::path::{Path, PathBuf};
use std::process::Command;

fn cpl_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cpl")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cpl-report-test-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn find_report_file(dir: &Path) -> PathBuf {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("cpl-report-") && n.ends_with(".md"))
        })
        .collect();
    assert_eq!(
        found.len(),
        1,
        "expected 1 cpl-report-*.md in {dir:?}, got {found:?}"
    );
    found.pop().unwrap()
}

#[test]
fn report_on_trap_shrinks_redacts_paths_and_prints_issue_url() {
    let dir = scratch("trap");
    let nested = dir.join("deep/secret_folder");
    std::fs::create_dir_all(&nested).unwrap();

    let src_path = nested.join("trap_case.cp");
    let src = "\
let a: List<Int> = [10, 20]
let i: Int = read()
print(a[i])
";
    std::fs::write(&src_path, src).unwrap();

    // Multi-line input with trailing junk lines so `--shrink` drops them while
    // keeping the out-of-bounds index trap.
    let in_path = nested.join("input.txt");
    std::fs::write(&in_path, "5\nextra_line_one\nextra_line_two\n").unwrap();

    let out = Command::new(cpl_bin())
        .current_dir(&dir)
        .args([
            "report",
            src_path.to_str().unwrap(),
            "--input",
            in_path.to_str().unwrap(),
            "--shrink",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "stdout:\n{stdout}\nstderr:\n{stderr}");
    assert!(stdout.contains("wrote cpl-report-"), "stdout:\n{stdout}");
    assert!(stdout.contains("included:"), "stdout:\n{stdout}");
    assert!(
        stdout
            .contains("https://github.com/ryanbai1412/cp-lang/issues/new?template=bug.yml&title="),
        "expected prefilled issue URL for small report:\n{stdout}"
    );

    let report_path = find_report_file(&dir);
    let md = std::fs::read_to_string(&report_path).unwrap();

    // Environment & sections present.
    assert!(md.contains("## Environment"), "{md}");
    assert!(md.contains("- **cpl**: `cpl "), "{md}");
    assert!(md.contains("- **OS / arch**:"), "{md}");
    assert!(md.contains("- **g++**:"), "{md}");
    assert!(md.contains("- **judge profile**:"), "{md}");
    assert!(md.contains("## Source (`trap_case.cp`)"), "{md}");
    assert!(md.contains("## Interpreter"), "{md}");
    assert!(md.contains("## C++ build"), "{md}");
    assert!(md.contains("## Comparison"), "{md}");

    // `--shrink` reduced the input.
    assert!(md.contains("shrunk from"), "expected shrink note:\n{md}");
    assert!(
        !md.contains("extra_line_two"),
        "expected shrunk input:\n{md}"
    );

    // Privacy: no absolute paths or secret_folder anywhere in the report.
    assert!(
        !md.contains(src_path.to_str().unwrap()),
        "absolute source path leaked:\n{md}"
    );
    assert!(
        !md.contains(in_path.to_str().unwrap()),
        "absolute input path leaked:\n{md}"
    );
    assert!(
        !md.contains("secret_folder"),
        "parent directory leaked:\n{md}"
    );
    assert!(
        !md.contains(dir.to_str().unwrap()),
        "temp directory leaked:\n{md}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn ice_hook_saves_crash_bundle_and_report_last_includes_backtrace() {
    let dir = scratch("ice");
    let crash_dir = dir.join("crashes");
    let src_path = dir.join("repro.cp");
    std::fs::write(&src_path, "print(1 + 2)\n").unwrap();

    // 1. Trigger a real panic in the `cpl` binary.
    let ice = Command::new(cpl_bin())
        .current_dir(&dir)
        .env("CPL_CRASH_DIR", &crash_dir)
        .env("CPL_FORCE_ICE", "1")
        .args(["check", src_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!ice.status.success(), "ICE must exit non-zero");
    let ice_stderr = String::from_utf8_lossy(&ice.stderr);
    assert!(
        ice_stderr.contains("this is a cpl bug: run `cpl report --last`"),
        "missing ICE hint on stderr:\n{ice_stderr}"
    );

    // 2. Verify crash bundle was saved under CPL_CRASH_DIR/<ts>/.
    let bundles: Vec<PathBuf> = std::fs::read_dir(&crash_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(bundles.len(), 1, "expected 1 crash bundle in {crash_dir:?}");
    assert!(bundles[0].join("meta.txt").is_file());
    assert!(bundles[0].join("backtrace.txt").is_file());
    assert!(bundles[0].join("source.cp").is_file());

    // 3. Run `cpl report --last` and verify the generated report.
    let rep = Command::new(cpl_bin())
        .current_dir(&dir)
        .env("CPL_CRASH_DIR", &crash_dir)
        .args(["report", "--last"])
        .output()
        .unwrap();

    let rep_stdout = String::from_utf8_lossy(&rep.stdout);
    let rep_stderr = String::from_utf8_lossy(&rep.stderr);
    assert!(
        rep.status.success(),
        "cpl report --last failed:\nstdout:\n{rep_stdout}\nstderr:\n{rep_stderr}"
    );
    assert!(
        rep_stdout.contains("ICE backtrace"),
        "stdout:\n{rep_stdout}"
    );

    let report_path = find_report_file(&dir);
    let md = std::fs::read_to_string(&report_path).unwrap();
    assert!(md.contains("## ICE backtrace"), "{md}");
    assert!(md.contains("forced ICE for testing"), "{md}");
    assert!(md.contains("## Source (`repro.cp`)"), "{md}");
    assert!(
        !md.contains(dir.to_str().unwrap()),
        "absolute path leaked in --last report:\n{md}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}
