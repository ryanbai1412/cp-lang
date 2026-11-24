//! The VS Code extension's symbol tables (`editors/vscode/src/languageData.ts`)
//! and its grammar (`syntaxes/cp.tmLanguage.json`) name exactly what the
//! compiler's prelude and `std/*.cp` (edition 0.3) ship. `npm test` in `editors/vscode`
//! checks the same and more, but only in the full suite; this runs in
//! `scripts/check.sh --fast`, so a new stdlib name cannot leave them behind.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &str) -> String {
    let path = root().join(path);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn is_word(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The names between `quote`s in `text`.
fn quoted(text: &str, quote: char) -> Vec<String> {
    text.split(quote)
        .skip(1)
        .step_by(2)
        .filter(|name| is_word(name))
        .map(String::from)
        .collect()
}

/// The string literals of `prelude.rs` from `from` to the next `end`.
fn rust_strings(prelude: &str, from: &str, end: &str) -> Vec<String> {
    let start = prelude
        .find(from)
        .unwrap_or_else(|| panic!("no `{from}` in prelude.rs"));
    let len = prelude[start..].find(end).expect("the table ends");
    quoted(&prelude[start..start + len], '"')
}

/// One `export const NAME: readonly string[] = [..];` of `languageData.ts`.
fn table(ts: &str, name: &str) -> BTreeSet<String> {
    let head = format!("export const {name}: readonly string[] = [");
    let start = ts
        .find(&head)
        .unwrap_or_else(|| panic!("no {name} in languageData.ts"))
        + head.len();
    let len = ts[start..].find("];").expect("the table ends");
    quoted(&ts[start..start + len], '\'').into_iter().collect()
}

/// The `(a|b|c)` of the grammar rule named `scope`.
fn alternation(grammar: &str, scope: &str) -> BTreeSet<String> {
    let rule = format!("\"name\": \"{scope}\"");
    let at = grammar
        .find(&rule)
        .unwrap_or_else(|| panic!("no rule for {scope}"));
    let rest = &grammar[at + rule.len()..];
    let rest = &rest[rest.find("\"match\": \"").expect("the rule has a match")..];
    let names = rest
        .split(['(', ')'])
        .find(|part| part.contains('|') && part.split('|').all(is_word))
        .unwrap_or_else(|| panic!("no names in the rule for {scope}"));
    names.split('|').map(String::from).collect()
}

/// Names the compiler's prelude knows that the tables leave out on purpose
/// (`UNLISTED_BUILTINS` in `test/unit/languageData.test.ts`).
const UNLISTED: [&str; 8] = [
    "INF", "Opt", "Counter", "argmax", "argmin", "columns", "last", "take",
];

#[test]
fn the_vscode_tables_and_grammar_name_what_the_compiler_and_stdlib_ship() {
    let prelude = read("crates/cpl-check/src/prelude.rs");
    let type_names = rust_strings(&prelude, "const TYPE_NAMES: &[&str] = &[", "];");
    let mut builtins: BTreeSet<String> = type_names.into_iter().collect();
    builtins.extend(rust_strings(&prelude, "pub fn value(", "\n}\n"));
    builtins.extend(rust_strings(
        &prelude,
        "const VALUE_NAMES: &[&str] = &[",
        "];",
    ));
    builtins.extend(rust_strings(&prelude, "pub fn ty(", "\n}\n"));

    let mut imported = BTreeSet::new();
    let prelude_cp = read("std/prelude.cp");
    let mut rest = prelude_cp.as_str();
    while let Some(pos) = rest.find("use std.") {
        rest = &rest[pos + "use std.".len()..];
        let Some(open) = rest.find('{') else { break };
        let Some(close) = rest[open + 1..].find('}') else {
            break;
        };
        imported.extend(
            rest[open + 1..open + 1 + close]
                .split(',')
                .map(|name| name.trim().to_string())
                .filter(|name| !name.is_empty()),
        );
        rest = &rest[open + 1 + close + 1..];
    }

    // A `@get` fn is read as a field, so it is a name of its own only when
    // the prelude imports it.
    let mut declared: BTreeMap<String, &str> = BTreeMap::new();
    let mut files: Vec<PathBuf> = fs::read_dir(root().join("std"))
        .expect("std/")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    files.sort();
    for file in files {
        for line in fs::read_to_string(&file).expect("a stdlib file").lines() {
            let leading_get = line.starts_with("@get pub ");
            let line = line.strip_prefix("@get ").unwrap_or(line);
            let Some(rest) = line.strip_prefix("pub ") else {
                continue;
            };
            let mut words = rest.split(' ').peekable();
            let mut getter = leading_get;
            while let Some(attr) = words.peek().filter(|word| word.starts_with('@')) {
                getter |= *attr == "@get";
                words.next();
            }
            let kinds = [
                "fn", "struct", "enum", "trait", "type", "const", "domain", "context",
            ];
            let Some(kind) = words
                .next()
                .and_then(|word| kinds.into_iter().find(|kind| *kind == word))
            else {
                continue;
            };
            let name: String = words
                .next()
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if is_word(&name)
                && !name.starts_with('_')
                && (!getter || imported.contains(&name))
                && !cpl_check::is_list_form_hook(&name)
            {
                declared.insert(name, kind);
            }
        }
    }

    let ts = read("editors/vscode/src/languageData.ts");
    let tables = [
        "PRELUDE_TYPES",
        "STDLIB_TYPES",
        "PRELUDE_VALUES",
        "STDLIB_VALUES",
        "PRELUDE_CONSTANTS",
        "STDLIB_CONSTANTS",
    ];
    let listed: BTreeSet<String> = tables.iter().flat_map(|name| table(&ts, name)).collect();

    let shipped: BTreeSet<String> = builtins
        .iter()
        .filter(|name| !UNLISTED.contains(&name.as_str()))
        .chain(declared.keys())
        .chain(&imported)
        .cloned()
        .collect();
    let missing: Vec<&String> = shipped.difference(&listed).collect();
    assert!(
        missing.is_empty(),
        "editors/vscode/src/languageData.ts and syntaxes/cp.tmLanguage.json do not list {missing:?}: run `npm run language-data` in editors/vscode, which regenerates both"
    );
    let phantom: Vec<&String> = listed.difference(&shipped).collect();
    assert!(
        phantom.is_empty(),
        "editors/vscode/src/languageData.ts lists {phantom:?}, which neither prelude.rs nor std/*.cp declares"
    );

    let grammar = read("editors/vscode/syntaxes/cp.tmLanguage.json");
    let scopes = [
        ("support.type.builtin.cp", "PRELUDE_TYPES"),
        ("support.type.stdlib.cp", "STDLIB_TYPES"),
        ("support.function.builtin.cp", "PRELUDE_VALUES"),
        ("support.function.stdlib.cp", "STDLIB_VALUES"),
    ];
    for (scope, name) in scopes {
        assert_eq!(
            alternation(&grammar, scope),
            table(&ts, name),
            "the grammar rule {scope} and {name} differ"
        );
    }
    let constants: BTreeSet<String> = ["PRELUDE_CONSTANTS", "STDLIB_CONSTANTS"]
        .iter()
        .flat_map(|name| table(&ts, name))
        .collect();
    assert_eq!(
        alternation(&grammar, "support.constant.stdlib.cp"),
        constants,
        "the grammar rule support.constant.stdlib.cp and PRELUDE_CONSTANTS + STDLIB_CONSTANTS differ"
    );
}

/// `cp.<key>` read as `getConfiguration('cp').get<..>('<key>', <default>)`
/// in the extension's sources: the default as written, spaces dropped and
/// quotes made double.
fn reads(sources: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for part in sources.split(".get<").skip(1) {
        let Some(call) = part.split_once(">(").map(|(_, rest)| rest) else {
            continue;
        };
        let Some(key) = call.strip_prefix('\'').and_then(|k| k.split_once('\'')) else {
            continue;
        };
        let default = key.1.trim_start_matches([',', ' ']);
        let mut depth = 0;
        let end = default
            .char_indices()
            .find(|&(_, c)| {
                match c {
                    '[' | '(' => depth += 1,
                    ']' if depth > 0 => depth -= 1,
                    ')' if depth > 0 => depth -= 1,
                    ')' => return true,
                    _ => {}
                }
                false
            })
            .map_or(default.len(), |(i, _)| i);
        let default: String = default[..end].replace('\'', "\"").replace(' ', "");
        out.insert(format!("cp.{}", key.0), default);
    }
    out
}

#[test]
fn every_vscode_setting_is_read_documented_and_defaults_alike() {
    let package = cpl_json::parse(&read("editors/vscode/package.json")).unwrap();
    let declared: BTreeMap<String, String> = package
        .at(&["contributes", "configuration", "properties"])
        .as_object()
        .expect("package.json declares its settings")
        .iter()
        .map(|(k, v)| (k.clone(), v.at(&["default"]).to_string().replace(' ', "")))
        .collect();
    assert!(
        declared.len() <= 15,
        "{} settings; keep them few",
        declared.len()
    );
    let src = root().join("editors/vscode/src");
    let mut sources = String::new();
    for entry in fs::read_dir(&src).unwrap().flatten() {
        if entry.path().extension().is_some_and(|e| e == "ts") {
            sources += &fs::read_to_string(entry.path()).unwrap();
        }
    }
    let read_by = reads(&sources);
    let readme = read("editors/vscode/README.md");
    let table = readme
        .split("## Settings")
        .nth(1)
        .and_then(|s| s.split("\n## ").next())
        .expect("README.md has a Settings section");
    let documented: BTreeMap<String, String> = table
        .lines()
        .filter_map(|l| {
            let cells: Vec<&str> = l.split(" | ").collect();
            let key = cells.first()?.strip_prefix("| `")?.strip_suffix('`')?;
            let default = cells.get(1)?.trim_matches('`').replace(' ', "");
            Some((key.to_string(), default))
        })
        .collect();
    let mut wrong = Vec::new();
    for (key, default) in &declared {
        // vscode-languageclient reads `<client id>.trace.server` itself.
        let read = if key == "cp.trace.server" {
            sources
                .contains("new LanguageClient('cp'")
                .then(|| default.clone())
        } else {
            read_by.get(key).cloned()
        };
        match read {
            None => wrong.push(format!("{key}: declared, never read")),
            Some(d) if &d != default => {
                wrong.push(format!("{key}: package.json default {default}, source {d}"))
            }
            _ => {}
        }
        match documented.get(key) {
            None => wrong.push(format!("{key}: not in README.md's Settings")),
            Some(d) if d != default => wrong.push(format!(
                "{key}: README.md default {d}, package.json {default}"
            )),
            _ => {}
        }
    }
    for key in read_by.keys().chain(documented.keys()) {
        if !declared.contains_key(key) {
            wrong.push(format!("{key}: read or documented, not declared"));
        }
    }
    wrong.sort();
    wrong.dedup();
    assert!(
        wrong.is_empty(),
        "settings out of step:\n{}",
        wrong.join("\n")
    );
}
