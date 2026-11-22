//! `cpl replay`: one recorded call run again with its fn edited.

use std::path::{Path, PathBuf};
use std::process::Command;

const CASE: &str = "2\n1\n0 1\n3\n0 2\n0 1\n1 1\n";

fn cpl(dir: &Path, args: &[&str]) -> (u8, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
    (
        out.status.code().unwrap_or(-1) as u8,
        text(&out.stdout),
        text(&out.stderr),
    )
}

fn recorded(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("replay_{name}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/debugging/shows.cp");
    std::fs::copy(demo, dir.join("shows.cp")).unwrap();
    std::fs::write(dir.join("case.in"), CASE).unwrap();
    let (code, _, err) = cpl(&dir, &["run", "shows.cp", "case.in", "--trace"]);
    assert_eq!(code, 0, "{err}");
    dir
}

fn edit(dir: &Path, from: &str, to: &str) {
    let source = std::fs::read_to_string(dir.join("shows.cp")).unwrap();
    assert!(source.contains(from));
    std::fs::write(dir.join("shows.cp"), source.replace(from, to)).unwrap();
}

#[test]
fn a_fixed_fn_replays_the_failing_call_alone() {
    let dir = recorded("fixed");
    edit(
        &dir,
        "by: i => shows[i].start)",
        "by: i => shows[i].start + shows[i].len)",
    );
    let replay = [
        "replay",
        "shows.cp",
        "--trace",
        "shows.trace.html",
        "--input",
        "case.in",
        "--step",
    ];
    let (code, out, err) = cpl(&dir, &[&replay[..], &["13"]].concat());
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("call 2 of `most_watched()`, steps 13-23"),
        "{out}"
    );
    assert!(out.contains("returns 2 (before the edit: 1)"), "{out}");
    assert!(
        out.contains(
            "first change: step 17 line 12: `order` = [2, 1, 3] (before the edit: [1, 2, 3])"
        ),
        "{out}"
    );
    assert!(!out.contains("note:"), "no fallback: {out}");
    let (_, out, _) = cpl(&dir, &[&replay[..], &["3"]].concat());
    assert!(
        out.contains("call 1 of `most_watched()`") && out.contains("the call runs as recorded"),
        "{out}"
    );
    let (code, _, err) = cpl(&dir, &[&replay[..], &["2"]].concat());
    assert_eq!(code, 1);
    assert!(err.contains("no fn call is running at step 2"), "{err}");
}

#[test]
fn a_changed_signature_runs_the_whole_file_and_says_so() {
    let dir = recorded("signature");
    edit(
        &dir,
        "fn most_watched() -> Int:",
        "fn most_watched(from: Int) -> Int:",
    );
    edit(&dir, "print(most_watched())", "print(most_watched(0))");
    let (code, out, err) = cpl(
        &dir,
        &[
            "replay",
            "shows.cp",
            "--trace",
            "shows.trace.html",
            "--input",
            "case.in",
            "--step",
            "13",
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(
        out.contains("note: the edit changes what `most_watched` takes or returns"),
        "{out}"
    );
}
