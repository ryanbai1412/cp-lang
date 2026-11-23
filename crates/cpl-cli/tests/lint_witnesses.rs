//! A bound lint that says an input breaks the program names that input,
//! and the input really breaks it: each witness the lint prints is pinned
//! into the program's declared ranges, `cpl-stress` fills in the rest of a
//! valid input, and `cpl run` must trap on the line the lint flagged.
//! Rule: `errors/R16`, handbook/guidelines/errors-checklist.md#r16-a-warning-about-declared-ranges-shows-its-witness.

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

/// `(code, message, line)` of each warning in `cpl lint`'s output.
fn warnings(rendered: &str) -> Vec<(String, String, usize)> {
    let lines: Vec<&str> = rendered.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix("warning[") else {
            continue;
        };
        let Some((code, message)) = rest.split_once("]: ") else {
            continue;
        };
        let at = lines.get(i + 1).and_then(|l| l.trim().strip_prefix("--> "));
        let line_no = at.and_then(|a| a.rsplit(':').nth(1)?.parse().ok());
        if let Some(line_no) = line_no {
            out.push((code.to_string(), message.to_string(), line_no));
        }
    }
    out
}

/// The input values a warning names: "with `a = 5, b = 7`", "`b` can be 0",
/// "when `n` is 0", "its bounds allow 63" after "when `k` is above".
fn witnesses(message: &str) -> Vec<(String, i128)> {
    let ticked: Vec<&str> = message.split('`').skip(1).step_by(2).collect();
    let mut out = Vec::new();
    if let Some(at) = message.find("with `") {
        let assigns = message[at + 6..].split('`').next().unwrap_or("");
        for pair in assigns.split(", ") {
            if let Some((name, value)) = pair.split_once(" = ") {
                if let Ok(v) = value.trim().parse() {
                    out.push((name.trim().to_string(), v));
                }
            }
        }
    }
    let number_after = |s: &str| {
        s.split(|c: char| !c.is_ascii_digit())
            .find(|w| !w.is_empty())?
            .parse()
            .ok()
    };
    for (name, after) in ticked.iter().zip(message.split('`').skip(2).step_by(2)) {
        let named = |v: Option<i128>| v.map(|v| (name.to_string(), v));
        let found = if let Some(rest) = after
            .strip_prefix(" can be ")
            .or(after.strip_prefix(" is "))
        {
            if after.starts_with(" is above ") {
                named(message.split("bounds allow ").nth(1).and_then(number_after))
            } else {
                named(number_after(rest).filter(|_| rest.starts_with(|c: char| c.is_ascii_digit())))
            }
        } else {
            None
        };
        out.extend(found);
    }
    out
}

/// `src` with the declared range of the read `name` narrowed to `value..=value`.
fn pin(src: &str, name: &str, value: i128) -> Option<String> {
    let mut out = String::new();
    let mut done = false;
    for line in src.lines() {
        let mut line = line.to_string();
        if let Some((pat, rest)) = line
            .trim_start()
            .strip_prefix("let ")
            .and_then(|r| r.split_once(": "))
        {
            if let Some((ty, read)) = rest.split_once(" = read") {
                let names: Vec<&str> = pat.trim_matches(['(', ')']).split(", ").collect();
                let mut parts: Vec<String> = if names.len() > 1 {
                    ty.trim_matches(['(', ')'])
                        .split(", ")
                        .map(str::to_string)
                        .collect()
                } else {
                    vec![ty.to_string()]
                };
                if let Some(at) = names
                    .iter()
                    .position(|n| *n == name)
                    .filter(|_| parts.len() == names.len())
                {
                    parts[at] = format!("{value}..={value}");
                    let ty = if names.len() > 1 {
                        format!("({})", parts.join(", "))
                    } else {
                        parts.remove(0)
                    };
                    let indent = &line[..line.len() - line.trim_start().len()];
                    line = format!("{indent}let {pat}: {ty} = read{read}");
                    done = true;
                }
            }
        }
        out += &line;
        out.push('\n');
    }
    done.then_some(out)
}

#[test]
fn every_bound_lint_witness_traps_where_the_lint_says() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let scratch = std::env::temp_dir().join(format!("cpl-lint-witnesses-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let mut programs: Vec<_> = fs::read_dir(root.join("spec/lints"))
        .unwrap()
        .map(|e| e.unwrap().path().join("before.cp"))
        .filter(|p| p.exists())
        .collect();
    programs.sort();
    let (mut checked, mut wrong) = (0, Vec::new());
    for path in &programs {
        let src = fs::read_to_string(path).unwrap();
        let file = scratch.join("main.cp");
        fs::write(&file, &src).unwrap();
        let (_, linted) = cpl(&["lint", file.to_str().unwrap()]);
        for (code, message, line) in warnings(&linted) {
            if !TRAPPING.contains(&code.as_str()) {
                continue;
            }
            let shown = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .display()
                .to_string();
            let named = witnesses(&message);
            let pinned = named
                .iter()
                .try_fold(src.clone(), |s, (n, v)| pin(&s, n, *v));
            let Some(pinned) = pinned.filter(|_| !named.is_empty()) else {
                wrong.push(format!(
                    "{shown}: {code} names no input it can pin: {message}"
                ));
                continue;
            };
            let spec = cpl_stress::InputSpec::from_source(&pinned).unwrap();
            checked += 1;
            for seed in 1..=5 {
                let mut rng = cpl_stress::Rng::new(seed);
                let case =
                    cpl_stress::generate(&spec, &mut rng, cpl_stress::Size::Small(5)).unwrap();
                let input = scratch.join("main.in");
                fs::write(&input, case.render()).unwrap();
                let (status, ran) = cpl(&["run", file.to_str().unwrap(), input.to_str().unwrap()]);
                let at = format!("main.cp:{line}:");
                if status == Some(0) || !ran.contains("error[R") || !ran.contains(&at) {
                    wrong.push(format!("{shown}: {code} at line {line} claims {named:?} traps, but with input\n{}it gave\n{ran}", case.render()));
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
