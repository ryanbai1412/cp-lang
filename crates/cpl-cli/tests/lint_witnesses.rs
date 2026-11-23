//! A bound lint that says an input breaks the program names that input,
//! and the input really breaks it: `cpl lint --save` writes each witness
//! as a case (the witness pinned into the program's declared reads,
//! `cpl-stress` filling in the rest), and `cpl run` on it must trap on the
//! line the lint flagged, as must four more fillings of the same witness.
//! Rule: `errors/R16`, handbook/guidelines/errors-checklist.md#r16-a-warning-about-declared-ranges-shows-its-witness.

use cpl_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

/// The lints whose claim is "this input traps here".
const TRAPPING: [&str; 5] = ["L0302", "L0304", "L0305", "L0306", "L0599"];

fn cpl(args: &[&str]) -> (Option<i32>, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).into_owned() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code(), text)
}

fn get<'a>(v: &'a Value, key: &str) -> &'a Value {
    match v {
        Value::Object(fields) => fields.get(key).unwrap_or(&Value::Null),
        _ => &Value::Null,
    }
}

fn items(v: &Value) -> &[Value] {
    match v {
        Value::Array(items) => items,
        _ => &[],
    }
}

/// A warning's code, line and witness.
type Warning = (String, i64, Vec<(String, i128)>);

/// Each warning `cpl lint --json` gives.
fn warnings(json: &str) -> Vec<Warning> {
    let v = cpl_json::parse(json).unwrap();
    let mut out = Vec::new();
    for file in items(get(&v, "files")) {
        for d in items(get(file, "diagnostics")) {
            let (Value::String(code), Value::Int(line)) =
                (get(d, "code"), get(get(d, "start"), "line"))
            else {
                continue;
            };
            let witness = items(get(d, "witness"))
                .iter()
                .filter_map(|w| match (get(w, "name"), get(w, "value")) {
                    (Value::String(n), Value::Int(v)) => Some((n.clone(), i128::from(*v))),
                    (Value::String(n), Value::String(v)) => Some((n.clone(), v.parse().ok()?)),
                    _ => None,
                })
                .collect();
            out.push((code.clone(), *line, witness));
        }
    }
    out
}

#[test]
fn every_bound_lint_witness_traps_where_the_lint_says() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scratch = std::env::temp_dir().join(format!("cpl-lint-witnesses-{}", std::process::id()));
    let mut programs: Vec<_> = fs::read_dir(root.join("spec/lints"))
        .unwrap()
        .map(|e| e.unwrap().path().join("before.cp"))
        .filter(|p| p.exists())
        .collect();
    programs.sort();
    let (mut checked, mut wrong) = (0, Vec::new());
    for path in &programs {
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).unwrap();
        let src = fs::read_to_string(path).unwrap();
        let file = scratch.join("main.cp");
        fs::write(&file, &src).unwrap();
        let file = file.to_str().unwrap();
        let shown = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .display()
            .to_string();
        let (_, json) = cpl(&["lint", "--json", file]);
        let json = json.lines().find(|l| l.starts_with('{')).unwrap_or("{}");
        let found: Vec<_> = warnings(json)
            .into_iter()
            .filter(|(code, ..)| TRAPPING.contains(&code.as_str()))
            .collect();
        if found.is_empty() {
            continue;
        }
        let (_, saved) = cpl(&["lint", "--save", file]);
        for (code, line, witness) in found {
            let case = scratch.join(format!("main.tests/lint-{code}-{line}.in"));
            if witness.is_empty() || !case.exists() {
                wrong.push(format!("{shown}: {code} at line {line} saved no case for its witness {witness:?}:\n{saved}"));
                continue;
            }
            checked += 1;
            let mut spec = cpl_stress::InputSpec::from_source(&src).unwrap();
            assert!(witness.iter().all(|(n, v)| spec.pin(n, *v)));
            let mut inputs = vec![fs::read_to_string(&case).unwrap()];
            for seed in 2..=5 {
                let mut rng = cpl_stress::Rng::new(seed);
                let made =
                    cpl_stress::generate(&spec, &mut rng, cpl_stress::Size::Small(5)).unwrap();
                inputs.push(made.render());
            }
            for input in inputs {
                let at = scratch.join("main.in");
                fs::write(&at, &input).unwrap();
                let (status, ran) = cpl(&["run", file, at.to_str().unwrap()]);
                if status == Some(0)
                    || !ran.contains("error[R")
                    || !ran.contains(&format!("main.cp:{line}:"))
                {
                    wrong.push(format!("{shown}: {code} at line {line} claims {witness:?} traps, but with input\n{input}it gave\n{ran}"));
                    break;
                }
            }
        }
    }
    let _ = fs::remove_dir_all(&scratch);
    assert!(
        checked >= TRAPPING.len(),
        "only {checked} witnesses checked"
    );
    assert!(
        wrong.is_empty(),
        "{} bound-lint claims are not true:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}
