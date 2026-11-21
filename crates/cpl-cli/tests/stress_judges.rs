//! `cpl stress` under a judge: every ecnerwala port with an interactor or
//! a checker is stressed on its generator's cases with that judge and must
//! pass, and a solution that answers `1` to everything must be caught, so
//! the judge is known to be the one ruling.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The checkers that compare with the original's answer, so the original
/// runs as the brute.
const NEEDS_ANSWER: &[&str] = &["cf2062g", "cf2187b"];

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proto/ports/ecnerwala")
}

/// `(port, "--interactor" or "--checker", judge)` for every judged port.
fn judged() -> Vec<(String, &'static str, PathBuf)> {
    let mut out = Vec::new();
    for (sub, flag) in [("interactors", "--interactor"), ("checkers", "--checker")] {
        for entry in std::fs::read_dir(dir().join(sub)).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            if path.extension().is_some_and(|ext| ext == "py") && !name.starts_with('_') {
                out.push((name, flag, path));
            }
        }
    }
    out.sort();
    out
}

fn stress(scratch: &Path, name: &str, flag: &str, judge: &Path, fast: Option<&Path>) -> String {
    let copy = scratch.join(format!("{name}.cp"));
    std::fs::copy(dir().join(format!("{name}.cp")), &copy).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cpl"));
    cmd.arg("stress").arg(&copy).arg(flag).arg(judge);
    cmd.arg("--gen").arg(dir().join(format!("gen/{name}.py")));
    cmd.args(["--cases", "20", "--format", "json"]);
    if NEEDS_ANSWER.contains(&name) {
        cmd.arg("--brute")
            .arg(dir().join(format!("{name}_orig.cpp")));
    }
    if let Some(fast) = fast {
        cmd.arg("--fast").arg(fast);
    }
    let out = cmd.output().expect("cpl runs");
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn every_judged_port_passes_stress_under_its_judge_and_a_wrong_one_is_caught() {
    let ports = judged();
    let count = |flag: &str| ports.iter().filter(|(_, f, _)| *f == flag).count();
    assert!(
        count("--interactor") >= 16 && count("--checker") >= 5,
        "{ports:?}"
    );
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("stress_judges");
    let _ = std::fs::remove_dir_all(&root);
    let wrong = root.join("wrong.py");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(&wrong, "print(1)\n").unwrap();
    let failures: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = ports
            .iter()
            .map(|(name, flag, judge)| {
                let (root, wrong) = (&root, &wrong);
                scope.spawn(move || {
                    let mut failures = Vec::new();
                    for (sub, fast, want) in [
                        ("right", None, "\"ok\""),
                        ("wrong", Some(wrong.as_path()), "\"fail\""),
                    ] {
                        let scratch = root.join(name).join(sub);
                        std::fs::create_dir_all(&scratch).unwrap();
                        let said = stress(&scratch, name, flag, judge, fast);
                        if !said.starts_with(&format!("{{\"status\":{want}")) {
                            failures.push(format!("{name} ({sub}, {flag}): {said}"));
                        }
                    }
                    failures
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
