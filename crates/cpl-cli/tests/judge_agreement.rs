//! Local verdicts match the judge's: `cpl test` and `cpl stress` compare
//! tokens with `cpl_stress::judge`, and the Python benches and scripts with
//! `corpus/lib/judge_tokens.py`. Both must give every pair below the same
//! verdict, and no other Python file may keep a tolerance formula of its own.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// (expected, got, accepted).
const PAIRS: &[(&str, &str, bool)] = &[
    ("42", "42", true),
    ("42", "43", false),
    ("YES", "yes", true),
    ("No", "NO", true),
    ("Possible", "possible", false),
    ("3", "3.0000001", true),
    ("3", "4", false),
    ("12", "012", false),
    // The benches' old isclose(rel_tol=1e-6) accepted this wrong answer.
    ("1000000000", "1000000001", false),
    ("1.0", "1.000001", true),
    ("1.0", "1.00001", false),
    ("1e9", "1000000001", true),
    ("-0.0", "0.0", true),
    ("+1.5", "1.5", true),
    (".5", "0.5", true),
    ("5.", "5.0", true),
    ("nan", "nan", true),
    ("nan", "NaN", false),
    ("inf", "1e309", false),
    ("1e309", "1e310", false),
    ("0x10", "16", false),
    ("1_000", "1000.0", false),
    ("e5", "1e5", false),
    ("1.2.3", "1.2.3", true),
    ("1.2.3", "1.2.4", false),
    ("1 2 3", "1 2 3", true),
    ("1\n2 3\n", "1 2\n3", true),
    ("1 2", "1 2 3", false),
    ("0.333333", "0.3333333333", true),
    ("100000000.5", "100000000.6", true),
    ("abc", "abd", false),
];

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn cpl_test_and_the_python_benches_judge_tokens_alike() {
    let python: String = PAIRS
        .iter()
        .map(|(e, g, _)| format!("{e:?}\t{g:?}\n").replace("\\n", "\\u000a"))
        .collect();
    let mut child = match Command::new("python3")
        .args([
            "-c",
            "import json, sys\n\
             sys.path.insert(0, 'corpus/lib')\n\
             from judge_tokens import same_tokens\n\
             for line in sys.stdin:\n\
             \x20   e, g = (json.loads(x) for x in line.rstrip('\\n').split('\\t'))\n\
             \x20   print(int(same_tokens(e, g)))",
        ])
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return eprintln!("skipped: no python3"),
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(python.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "judge_tokens.py failed");
    let verdicts: Vec<bool> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| l == "1")
        .collect();
    assert_eq!(verdicts.len(), PAIRS.len());
    let mut wrong = Vec::new();
    for ((expected, got, want), py) in PAIRS.iter().zip(verdicts) {
        let rust = cpl_stress::judge::first_difference(expected, got).is_none();
        if rust != *want || py != *want {
            wrong.push(format!(
                "{expected:?} vs {got:?}: want {want}, cpl {rust}, python {py}"
            ));
        }
    }
    assert!(wrong.is_empty(), "verdicts differ:\n{}", wrong.join("\n"));
}

#[test]
fn no_python_file_keeps_its_own_tolerance() {
    let listed = Command::new("git")
        .args(["ls-files", "*.py"])
        .current_dir(root())
        .output();
    let Ok(listed) = listed else {
        return eprintln!("skipped: no git");
    };
    let own: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .filter(|p| *p != "corpus/lib/judge_tokens.py")
        .filter(|p| {
            let text = std::fs::read_to_string(root().join(p)).unwrap_or_default();
            text.contains("rel_tol=1e-6") || text.contains("tol * max(1.0")
        })
        .map(str::to_string)
        .collect();
    assert!(
        own.is_empty(),
        "compare outputs with corpus/lib/judge_tokens.py instead: {own:?}"
    );
}
