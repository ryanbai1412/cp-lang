//! Generates the lint registry from the records in `spec/lints/<name>/`:
//! one `pub const <NAME>: Lint` per record and `LINTS`, every lint in code
//! order; and from `spec/checker-lints/<name>/`, `CHECKER_RECORDS` and
//! `CHECKER_LINTS`, which have no `rule`. A record's `lint.toml` holds `code`, `level`, `group`, `title`,
//! `summary` and `rule` (the checklist rule it enforces, as `<topic>/R<n>`,
//! which `handbook/guidelines/<topic>-checklist.md` must define), each a one-line basic
//! string, and `no_fix` when the lint has no machine fix: why not;
//! `removed` when `before.cp` uses a removed spelling: the checker error it
//! shows (every other `before.cp` passes `cpl check`). The
//! optional `family` and `practice` place it in the idiom catalog
//! (`spec/idioms/lints.md`), for the site's lint pages. `replaces = "whole"`
//! marks a lint whose std call replaces its span whole, so a finding inside
//! it is not shown; `replaces = "structure"` also names a std structure, so it
//! is quiet in a library module, where such structures are written.
//! `namesake = "quiet"` marks a lint whose message names the call to write,
//! so it is quiet inside a fn of that name.

use std::fmt::Write as _;
use std::path::PathBuf;

const KEYS: [&str; 6] = ["code", "level", "title", "summary", "group", "rule"];
const OPTIONAL: [&str; 6] = [
    "no_fix", "family", "practice", "removed", "replaces", "namesake",
];
const FILES: [(&str, &str); 4] = [
    ("explanation", "explain.md"),
    ("trigger", "before.cp"),
    ("fix", "after.cp"),
    ("near_miss", "near_miss.cp"),
];

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-changed=build.rs");
    let lints = records(&manifest, "spec/lints");
    let checker = records(&manifest, "spec/checker-lints");
    // The playground lints but never explains (the site's lint pages come
    // from `cpl lint --list --json`), so the wasm build leaves the long
    // texts out: about 29 KB gzipped.
    let texts = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32");
    let mut out = String::new();
    for record in &lints {
        check_rule(&manifest.join("../.."), &record.1, &record.2[5]);
        writeln!(
            out,
            "pub const {}: Lint = {};",
            record.1.to_uppercase(),
            lint(record, texts)
        )
        .unwrap();
    }
    out.push_str("\n/// Every lint, in code order.\npub const LINTS: &[Lint] = &[\n");
    for (_, name, _, _) in &lints {
        writeln!(out, "    {},", name.to_uppercase()).unwrap();
    }
    out.push_str("];\n");
    // Read without `LINTS` so a caller that wants only names doesn't link
    // every lint's record into cpl.wasm.
    out.push_str("\n/// Every lint's name and title, in code order.\npub const LINT_TITLES: &[(&str, &str)] = &[\n");
    for (_, name, fields, _) in &lints {
        writeln!(out, "    ({name:?}, {:?}),", fields[2]).unwrap();
    }
    out.push_str("];\n");
    out.push_str("\n/// The records in `spec/checker-lints/`, for `cpl lint --list --json`.\npub const CHECKER_RECORDS: &[Lint] = &[\n");
    for record in &checker {
        writeln!(out, "    {},", lint(record, texts)).unwrap();
    }
    out.push_str("];\n");
    out.push_str("\n/// Every name the spec gives a checker lint, in code order.\npub const CHECKER_LINTS: &[CheckerLint] = &[\n");
    for (code, name, fields, _) in &checker {
        writeln!(
            out,
            "    checker::named({code:?}, {name:?}, Group::{}),",
            group(fields, "spec/checker-lints", name)
        )
        .unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo")).join("lints.rs");
    std::fs::write(dest, out).expect("a writable OUT_DIR");
}

/// A lint record: its code, name, `KEYS` then `OPTIONAL` values, and directory.
type Record = (String, String, Vec<String>, PathBuf);

/// The records under `root`, in code order. `spec/checker-lints` records
/// name no checklist `rule`: the checker emits them, not a lint pass.
fn records(manifest: &std::path::Path, root: &str) -> Vec<Record> {
    let dir = manifest.join("../..").join(root);
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{root}: {e}")) {
        let dir = entry.unwrap_or_else(|e| panic!("{root}: {e}")).path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir
            .file_name()
            .unwrap()
            .to_str()
            .expect("a UTF-8 lint name")
            .to_string();
        let toml = dir.join("lint.toml");
        println!("cargo:rerun-if-changed={}", toml.display());
        let text =
            std::fs::read_to_string(&toml).unwrap_or_else(|e| panic!("{}: {e}", toml.display()));
        let fields = fields(
            &text,
            &format!("{root}/{name}/lint.toml"),
            root == "spec/lints",
        );
        records.push((fields[0].clone(), name, fields, dir));
    }
    records.sort();
    for pair in records.windows(2) {
        if pair[0].0 == pair[1].0 && root == "spec/lints" {
            panic!(
                "spec/lints/{} and spec/lints/{} both claim {}; run scripts/new-lint.py --next for a free code",
                pair[0].1, pair[1].1, pair[0].0
            );
        }
    }
    records
}

fn group(fields: &[String], root: &str, name: &str) -> &'static str {
    match fields[4].as_str() {
        "correctness" => "Correctness",
        "perf" => "Perf",
        "idiom" => "Idiom",
        "style" => "Style",
        "pedantic" => "Pedantic",
        other => panic!(
            "{root}/{name}/lint.toml: group {other:?} is not correctness, perf, idiom, style or pedantic"
        ),
    }
}

/// `record` as a `Lint` expression.
fn lint((code, name, fields, dir): &Record, texts: bool) -> String {
    let root = dir.parent().unwrap().file_name().unwrap().to_str().unwrap();
    let root = format!("spec/{root}");
    let level = match fields[1].as_str() {
        "allow" => "Allow",
        "warn" => "Warn",
        "deny" => "Deny",
        other => panic!("{root}/{name}/lint.toml: level {other:?} is not allow, warn or deny"),
    };
    let some = |at: usize| fields.get(at).filter(|v| !v.is_empty());
    let mut out = format!(
        "Lint {{\n    code: Code({code:?}),\n    name: {name:?},\n    dir: {:?},\n    default: Level::{level},\n    group: Group::{},\n",
        format!("{root}/{name}"),
        group(fields, &root, name)
    );
    writeln!(
        out,
        "    title: {:?},\n    summary: {:?},\n    rule: {:?},\n    no_fix: {:?},\n    family: {:?},\n    practice: {:?},\n    removed: {:?},",
        fields[2],
        fields[3],
        some(5),
        some(6),
        some(7),
        some(8),
        some(9)
    )
    .unwrap();
    let replaces = fields.get(10).map_or("", |v| v.as_str());
    assert!(
        ["", "whole", "structure"].contains(&replaces),
        "{name}: `replaces` is \"whole\", \"structure\" or absent"
    );
    writeln!(out, "    replaces_whole: {},", !replaces.is_empty()).unwrap();
    writeln!(out, "    std_structure: {},", replaces == "structure").unwrap();
    let namesake = fields.get(11).map_or("", |v| v.as_str());
    assert!(
        ["", "quiet"].contains(&namesake),
        "{name}: `namesake` is \"quiet\" or absent"
    );
    writeln!(out, "    quiet_in_namesake: {},", namesake == "quiet").unwrap();
    for (field, file) in FILES {
        let path = dir.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        if texts {
            writeln!(
                out,
                "    {field}: include_str!({:?}),",
                path.display().to_string()
            )
            .unwrap();
        } else {
            writeln!(out, "    {field}: \"\",").unwrap();
        }
    }
    out.push('}');
    out
}

/// Panics unless `rule` is a rule id, `idiomatic-code/R3`, that its
/// checklist `handbook/guidelines/idiomatic-code-checklist.md` defines with a
/// heading `### idiomatic-code/R3` or `### R3. <title>`.
fn check_rule(root: &std::path::Path, name: &str, rule: &str) {
    let at = format!("spec/lints/{name}/lint.toml: rule {rule:?}");
    let (topic, id) = rule
        .split_once('/')
        .filter(|(_, id)| {
            id.strip_prefix('R')
                .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        })
        .unwrap_or_else(|| panic!("{at} is not `<topic>/R<n>`"));
    let doc = format!("handbook/guidelines/{topic}-checklist.md");
    let path = root.join(&doc);
    println!("cargo:rerun-if-changed={}", path.display());
    let text = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{at}: no {doc}"));
    let found = text
        .lines()
        .filter_map(|l| l.strip_prefix("### "))
        .any(|h| {
            let h = h
                .strip_prefix(topic)
                .and_then(|h| h.strip_prefix('/'))
                .unwrap_or(h);
            h.strip_prefix(id)
                .is_some_and(|rest| !rest.starts_with(|c: char| c.is_ascii_digit()))
        });
    assert!(found, "{at}: {doc} has no rule {id}");
}

/// The values of `KEYS` in `text`, in that order; anything else is an error.
fn fields(text: &str, path: &str, rule: bool) -> Vec<String> {
    let mut found: Vec<Option<String>> = vec![None; KEYS.len()];
    let mut optional = vec![String::new(); OPTIONAL.len()];
    for line in text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let (key, value) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("{path}: expected `key = \"value\"`, got {line:?}"));
        if let Some(at) = OPTIONAL.iter().position(|k| *k == key) {
            optional[at] = unquote(value, path);
            continue;
        }
        let at = KEYS.iter().position(|k| *k == key).unwrap_or_else(|| {
            panic!("{path}: unknown key {key:?}; the keys are {KEYS:?} and {OPTIONAL:?}")
        });
        found[at] = Some(unquote(value, path));
    }
    let mut fields: Vec<String> = found
        .into_iter()
        .zip(KEYS)
        .map(|(v, k)| match v {
            Some(v) => v,
            None if k == "rule" && !rule => String::new(),
            None => panic!("{path}: missing {k:?}"),
        })
        .collect();
    fields.extend(optional);
    fields
}

/// A TOML basic string's contents, with its `\"` and `\\` escapes.
fn unquote(value: &str, path: &str) -> String {
    let inner = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or_else(|| panic!("{path}: {value} is not a one-line \"string\""));
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            other => panic!("{path}: unsupported escape \\{other:?} in {value}"),
        }
    }
    out
}
