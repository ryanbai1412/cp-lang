//! Every `--json` output has the keys `spec/tools/cli.md` publishes for it:
//! each "`cpl X --json` prints `{...}`" sentence is parsed as a shape, every
//! shape is run on a fixture, and every command `cpl --help --json` says
//! has JSON output has a shape.

use cpl_json::Value;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::{Command, Output};

#[path = "support/cli_fixtures.rs"]
mod cli_fixtures;

#[derive(Debug)]
enum Shape {
    Any,
    Object(Vec<(String, bool, Shape)>),
    List(Box<Shape>),
}

/// Parses `{"a", "b"?, "c": [{"d"}]}`: a key marked `?` may be absent.
fn shape(text: &mut std::iter::Peekable<std::str::Chars>) -> Shape {
    let skip = |text: &mut std::iter::Peekable<std::str::Chars>| {
        while text.next_if(|c| c.is_whitespace() || *c == ',').is_some() {}
    };
    skip(text);
    match text.next() {
        Some('[') => {
            let inner = shape(text);
            skip(text);
            assert_eq!(text.next(), Some(']'));
            Shape::List(Box::new(inner))
        }
        Some('{') => {
            let mut keys = Vec::new();
            loop {
                skip(text);
                match text.next() {
                    Some('}') => return Shape::Object(keys),
                    Some('"') => {
                        let name: String = text.by_ref().take_while(|c| *c != '"').collect();
                        let optional = text.next_if_eq(&'?').is_some();
                        skip(text);
                        let inner = match text.next_if_eq(&':') {
                            Some(_) => shape(text),
                            None => Shape::Any,
                        };
                        keys.push((name, optional, inner));
                    }
                    other => panic!("bad shape at {other:?}"),
                }
            }
        }
        other => panic!("bad shape at {other:?}"),
    }
}

fn check(value: &Value, shape: &Shape, at: &str, errors: &mut Vec<String>) {
    match (shape, value) {
        (Shape::Any, _) | (_, Value::Null) => {}
        (Shape::List(inner), Value::Array(items)) => {
            for item in items {
                check(item, inner, &format!("{at}[]"), errors);
            }
        }
        (Shape::Object(keys), Value::Object(map)) => {
            for (name, value) in map {
                if let Some(why) = misnamed(name, value) {
                    errors.push(format!("{at}.{name}: {why}"));
                }
            }
            for (name, optional, inner) in keys {
                match map.get(name) {
                    Some(value) => check(value, inner, &format!("{at}.{name}"), errors),
                    None if *optional => {}
                    None => errors.push(format!("{at}: no `{name}`")),
                }
            }
            for name in map.keys().filter(|k| !keys.iter().any(|(n, ..)| n == *k)) {
                errors.push(format!("{at}: `{name}` is not in cli.md"));
            }
        }
        _ => errors.push(format!("{at}: not the published kind: {shape:?}")),
    }
}

/// cli.md's naming rule, where a key's spelling can show it: a time or
/// memory quantity ends in its unit, a count is `<noun>_count` rather than a
/// plural, and the source and the pass flag are `path` and `ok`.
fn misnamed(name: &str, value: &Value) -> Option<&'static str> {
    let units = ["_ms", "_ns", "_kb"];
    if matches!(name, "file" | "passed" | "total" | "ms" | "kb" | "nanos") {
        return Some("spelt as cli.md's naming rule forbids");
    }
    if ["time", "memory", "peak"].iter().any(|q| name.contains(q))
        && !units.iter().any(|u| name.ends_with(u))
    {
        return Some("a time or memory key without its unit");
    }
    if name.ends_with("_mb") {
        return Some("a size in MB; every JSON byte size is `_kb`");
    }
    if matches!(value, Value::Int(_))
        && name.ends_with('s')
        && name != "bytes"
        && !units.iter().any(|u| name.ends_with(u))
    {
        return Some("a plural key holding a count; write `<noun>_count`");
    }
    None
}

/// The published shapes, by the command words before the flag: each
/// "`cpl X --json` prints `{..}`", or "`cpl X --json` and `cpl Y --json`
/// print `{..}`".
fn shapes(cli: &str) -> Vec<(String, Shape)> {
    let flat = cli.replace('\n', " ");
    let mut found = Vec::new();
    for verb in ["` prints `", "` print `"] {
        for (at, _) in flat.match_indices(verb) {
            let after = &flat[at + verb.len()..];
            let span = &after[..after.find('`').unwrap()];
            if !span.starts_with(['{', '[']) {
                continue;
            }
            let mut before = &flat[..at];
            loop {
                let open = before.rfind('`').unwrap();
                let command = &before[open + 1..];
                if let Some(words) = command
                    .strip_prefix("cpl ")
                    .filter(|_| command.ends_with("--json") || command.ends_with("--format json"))
                {
                    let words: Vec<_> = words
                        .split(' ')
                        .filter(|w| {
                            !w.starts_with('<') && !matches!(*w, "--json" | "--format" | "json")
                        })
                        .collect();
                    found.push((words.join(" "), shape(&mut span.chars().peekable())));
                }
                match before[..open].strip_suffix("` and ") {
                    Some(earlier) => before = earlier,
                    None => break,
                }
            }
        }
    }
    found
}

fn cpl(dir: &Path, home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cpl"))
        .args(args)
        .current_dir(dir)
        .env("HOME", home)
        .env("CPL_HOME", home.join(".cpl"))
        .output()
        .expect("cpl runs")
}

#[test]
fn every_json_output_has_the_published_keys() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cli = std::fs::read_to_string(root.join("spec/tools/cli.md")).unwrap();
    let shapes = shapes(&cli);
    let dir = cli_fixtures::fixtures("json-contract");
    let home = dir.join("home");
    let trace = cpl(&dir, &home, &["run", "r.cp", "--trace", "--format", "json"]);
    std::fs::write(dir.join("t.json"), &trace.stdout).unwrap();
    let url = "https://atcoder.jp/contests/abc999/tasks/abc999_a";
    let runs: &[(&str, &[&str])] = &[
        ("new", &["new", "fresh", "--json"]),
        (
            "fetch",
            &["fetch", url, "got", "--html", "page.html", "--json"],
        ),
        ("check", &["check", "a.cp", "bad.cp", "--json"]),
        ("fix", &["fix", "bad.cp", "--json"]),
        ("migrate", &["migrate", "a.cp", "--json"]),
        ("lint", &["lint", "a.cp", "--json"]),
        ("lint --list", &["lint", "--list", "--json"]),
        ("stress --list", &["stress", "--list", "--json"]),
        ("fmt", &["fmt", "--check", "a.cp", "--json"]),
        ("judge", &["judge", "j.cp", "j.in", "j.right", "--json"]),
        ("judge", &["judge", "j.cp", "j.in", "j.wrong", "--json"]),
        ("build", &["build", "a.cp", "-o", "a.cpp", "--json"]),
        ("submit", &["submit", "a.cp", "--json"]),
        (
            "report",
            &["report", "a.cp", "--input", "a.tests/1.in", "--json"],
        ),
        ("setup", &["setup", "neovim", "--check", "--json"]),
        ("self list", &["self", "list", "--json"]),
        ("self channel", &["self", "channel", "--json"]),
        ("self rollback", &["self", "rollback", "--json"]),
        ("watch", &["watch", "a", "--once", "--json"]),
        ("--help", &["--help", "--json"]),
        ("run", &["run", "a.cp", "a.tests/1.in", "--format", "json"]),
        (
            "run --samples",
            &["run", "a.cp", "--samples", "--format", "json"],
        ),
        (
            "run --samples --list",
            &["run", "a.cp", "--samples", "--list", "--format", "json"],
        ),
        (
            "run --samples",
            &["run", "bad_index.cp", "--samples", "--format", "json"],
        ),
        ("judges", &["judges", "--json"]),
        ("vendor", &["vendor", "v.cp", "--json"]),
        ("explain", &["explain", "--json"]),
        ("explain d", &["explain", "a.cp", "--json"]),
        ("doc", &["doc", "--format", "json", "a.cp"]),
        ("test", &["test", "a", "--format", "json"]),
        ("test --list", &["test", "a", "--list", "--json"]),
        ("stress", &["stress", "same.cp", "--format", "json"]),
        ("stress", &["stress", "differ.cp", "--format", "json"]),
        ("locate", &["locate", "alt.cp", "a.tests/1.in", "--json"]),
        ("bench", &["bench", "a", "--format", "json"]),
        (
            "profile",
            &["profile", "a.cp", "a.tests/1.in", "--format", "json"],
        ),
        (
            "cover",
            &["cover", "a.cp", "a.tests/1.in", "--format", "json"],
        ),
        (
            "replay",
            &[
                "replay", "r.cp", "--trace", "t.json", "--step", "1", "--json",
            ],
        ),
    ];
    let mut errors = Vec::new();
    for (name, _) in &shapes {
        if !runs.iter().any(|(run, _)| run == name) {
            errors.push(format!("`cpl {name}`'s published shape has no run here"));
        }
    }
    for (name, args) in runs {
        let Some((_, shape)) = shapes.iter().find(|(n, _)| n == name) else {
            errors.push(format!("`cpl {name}` has no shape in cli.md"));
            continue;
        };
        let out = cpl(&dir, &home, args);
        let stdout = String::from_utf8_lossy(&out.stdout);
        // `watch` prints a line per round.
        let first = if *name == "watch" {
            stdout.lines().next().unwrap_or("")
        } else {
            &stdout
        };
        match cpl_json::parse(first) {
            Ok(value @ (Value::Object(_) | Value::Array(_))) => check(
                &value,
                shape,
                &format!("cpl {}", args.join(" ")),
                &mut errors,
            ),
            _ => errors.push(format!(
                "cpl {}: no JSON: {stdout}{}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr)
            )),
        }
    }

    // Each diagnostic's keys, from the sentence after `cpl check --json`'s shape.
    let flat = cli.replace('\n', " ");
    let start = flat.find("each diagnostic has ").unwrap() + "each diagnostic has ".len();
    let list = &flat[start..start + flat[start..].find(", positions").unwrap()];
    let mut depth = 0;
    let outside: String = list
        .chars()
        .filter(|c| {
            depth += (*c == '(') as i32 - (*c == ')') as i32;
            depth == 0 && *c != ')'
        })
        .collect();
    let published: BTreeSet<&str> = outside.split('`').skip(1).step_by(2).collect();
    let checked = cpl(&dir, &home, &["check", "bad.cp", "--json"]);
    let value = cpl_json::parse(&String::from_utf8_lossy(&checked.stdout)).unwrap();
    let diagnostic = &value.at(&["files"]).as_array().unwrap()[0]
        .at(&["diagnostics"])
        .as_array()
        .unwrap()[0];
    let keys: BTreeSet<&str> = diagnostic
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    if keys != published {
        errors.push(format!(
            "a diagnostic has {keys:?}; cli.md publishes {published:?}"
        ));
    }

    // Every command with JSON output has a published shape.
    let help = cpl(&dir, &home, &["--help", "--json"]);
    let help = cpl_json::parse(&String::from_utf8_lossy(&help.stdout)).unwrap();
    for command in help.at(&["commands"]).as_array().unwrap() {
        let name = command.at(&["name"]).as_str().unwrap();
        if !command.at(&["json"]).is_null()
            && !shapes
                .iter()
                .any(|(n, _)| n == name || n.starts_with(&format!("{name} ")))
        {
            errors.push(format!(
                "`cpl {name}` has JSON output but no shape in cli.md"
            ));
        }
    }
    eprintln!(
        "json contract: {} shapes, {} runs",
        shapes.len(),
        runs.len()
    );
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}
