//! The wrong-answer workflow end to end, through the `cpl` binary, as
//! `handbook/product/workflows/wrong-answer.md` walks it: stress with no
//! brute finds a crash, stress with a brute finds a wrong answer, each
//! report names the next commands, the trace records the saved case, and
//! after each fix `cpl test` and `cpl stress` are green. Use cases
//! `stress/U1`, `stress/U4`, `stress/U13` and `stress/U14` in
//! `handbook/product/cli/stress-usecases.md`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cpl(args: &[&OsStr]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .output()
        .expect("cpl runs");
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text)
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("c.cp")
}

const READS: &str = "let n: 1..=10 = read()\nlet a: [n]0..=9 = read()\n";

const CRASHES: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i + 1])\nprint(best)\n";
const SKIPS_LAST: &str =
    "var best = 0\nfor i in 0..n - 1:\n    best = max(best, a[i])\nprint(best)\n";
const RIGHT: &str = "var best = 0\nfor i in 0..n:\n    best = max(best, a[i])\nprint(best)\n";

const BRUTE: &str = "
@alt(program)
fn brute():
    let n: 1..=10 = read()
    let a: [n]0..=9 = read()
    var m = 0
    for x in a: m = max(m, x)
    print(m)
";

fn saved(path: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(path.with_file_name("c.tests"))
        .map(|dir| {
            dir.map(|e| e.unwrap().file_name().into_string().unwrap())
                .filter(|n| n.ends_with(".in"))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn a_wrong_answer_goes_from_a_crash_and_a_counterexample_to_a_clean_stress() {
    let path = scratch("wrong-answer");
    let file = path.as_os_str();
    let stress = ["stress".as_ref(), file, "--cases".as_ref(), "200".as_ref()];
    let test = ["test".as_ref(), file];

    // No brute yet: a case fails only on a crash or the time limit.
    std::fs::write(&path, format!("{READS}{CRASHES}")).unwrap();
    let (code, report) = cpl(&stress);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "only runtime errors and the time limit fail a case",
        ": RE",
        "outside an array",
        "locals: ",
        "next: `cpl run ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 1, "{report}");

    // The trace records the case stress saved.
    let (_, traced) = cpl(&["run".as_ref(), file, "--trace".as_ref()]);
    assert!(traced.contains("the case saved last"), "{traced}");
    assert!(path.with_file_name("c.trace.html").is_file(), "{traced}");

    // Fixed: the saved case passes and stress runs clean.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(
        clean.contains("200 cases ran with no runtime error"),
        "{clean}"
    );

    // With a brute, the wrong answer that was left is found and explained.
    std::fs::write(&path, format!("{READS}{SKIPS_LAST}{BRUTE}")).unwrap();
    let mut explained = stress.to_vec();
    explained.push("--explain".as_ref());
    let (code, report) = cpl(&explained);
    assert_eq!(code, Some(1), "{report}");
    for said in [
        "that is: n = 1",
        "expected:",
        "look at line ",
        "after a fix: `cpl test ",
    ] {
        assert!(report.contains(said), "no {said:?} in:\n{report}");
    }
    assert_eq!(saved(&path).len(), 2, "{report}");
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(1), "{rerun}");

    // Fixed again: every saved case passes and the brute agrees.
    std::fs::write(&path, format!("{READS}{RIGHT}{BRUTE}")).unwrap();
    let (code, rerun) = cpl(&test);
    assert_eq!(code, Some(0), "{rerun}");
    assert!(rerun.contains("2/2 passed"), "{rerun}");
    let (code, clean) = cpl(&stress);
    assert_eq!(code, Some(0), "{clean}");
    assert!(clean.contains("200 cases agree"), "{clean}");
}
