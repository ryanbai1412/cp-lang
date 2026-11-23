//! Generates the lint registry from the records in `spec/lints/<name>/`:
//! one `pub const <NAME>: Lint` per record and `LINTS`, every lint in code
//! order. A record's `lint.toml` holds `code`, `level`, `title` and `summary`,
//! each a one-line basic string.

use std::fmt::Write as _;
use std::path::PathBuf;

const KEYS: [&str; 4] = ["code", "level", "title", "summary"];
const FILES: [(&str, &str); 4] = [
    ("explanation", "explain.md"),
    ("trigger", "before.cp"),
    ("fix", "after.cp"),
    ("near_miss", "near_miss.cp"),
];

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let root = manifest.join("../../spec/lints");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut records = Vec::new();
    for entry in std::fs::read_dir(&root).expect("spec/lints") {
        let dir = entry.expect("a readable spec/lints").path();
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
        let fields = fields(&text, &toml.display().to_string());
        records.push((fields[0].clone(), name, fields, dir));
    }
    records.sort();
    let mut out = String::new();
    for (code, name, fields, dir) in &records {
        let level = match fields[1].as_str() {
            "allow" => "Allow",
            "warn" => "Warn",
            "deny" => "Deny",
            other => {
                panic!("spec/lints/{name}/lint.toml: level {other:?} is not allow, warn or deny")
            }
        };
        writeln!(out, "pub const {}: Lint = Lint {{", name.to_uppercase()).unwrap();
        writeln!(
            out,
            "    code: Code({code:?}),\n    name: {name:?},\n    default: Level::{level},"
        )
        .unwrap();
        writeln!(
            out,
            "    title: {:?},\n    summary: {:?},",
            fields[2], fields[3]
        )
        .unwrap();
        for (field, file) in FILES {
            let path = dir.join(file);
            println!("cargo:rerun-if-changed={}", path.display());
            writeln!(
                out,
                "    {field}: include_str!({:?}),",
                path.display().to_string()
            )
            .unwrap();
        }
        out.push_str("};\n");
    }
    out.push_str("\n/// Every lint, in code order.\npub const LINTS: &[Lint] = &[\n");
    for (_, name, _, _) in &records {
        writeln!(out, "    {},", name.to_uppercase()).unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by cargo")).join("lints.rs");
    std::fs::write(dest, out).expect("a writable OUT_DIR");
}

/// The values of `KEYS` in `text`, in that order; anything else is an error.
fn fields(text: &str, path: &str) -> Vec<String> {
    let mut found: Vec<Option<String>> = vec![None; KEYS.len()];
    for line in text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let (key, value) = line
            .split_once(" = ")
            .unwrap_or_else(|| panic!("{path}: expected `key = \"value\"`, got {line:?}"));
        let at = KEYS
            .iter()
            .position(|k| *k == key)
            .unwrap_or_else(|| panic!("{path}: unknown key {key:?}; the keys are {KEYS:?}"));
        found[at] = Some(unquote(value, path));
    }
    found
        .into_iter()
        .zip(KEYS)
        .map(|(v, k)| v.unwrap_or_else(|| panic!("{path}: missing {k:?}")))
        .collect()
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
