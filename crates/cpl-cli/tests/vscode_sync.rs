//! The VS Code extension's symbol tables (`editors/vscode/src/languageData.ts`)
//! and its grammar (`syntaxes/cp.tmLanguage.json`) name exactly what the
//! compiler's prelude and `stdlib/*.cp` ship. `npm test` in `editors/vscode`
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
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The names between `quote`s in `text`.
fn quoted(text: &str, quote: char) -> Vec<String> {
    text.split(quote).skip(1).step_by(2).filter(|name| is_word(name)).map(String::from).collect()
}

/// The string literals of `prelude.rs` from `from` to the next `end`.
fn rust_strings(prelude: &str, from: &str, end: &str) -> Vec<String> {
    let start = prelude.find(from).unwrap_or_else(|| panic!("no `{from}` in prelude.rs"));
    let len = prelude[start..].find(end).expect("the table ends");
    quoted(&prelude[start..start + len], '"')
}

/// One `export const NAME: readonly string[] = [..];` of `languageData.ts`.
fn table(ts: &str, name: &str) -> BTreeSet<String> {
    let head = format!("export const {name}: readonly string[] = [");
    let start = ts.find(&head).unwrap_or_else(|| panic!("no {name} in languageData.ts")) + head.len();
    let len = ts[start..].find("];").expect("the table ends");
    quoted(&ts[start..start + len], '\'').into_iter().collect()
}

/// The `(a|b|c)` of the grammar rule named `scope`.
fn alternation(grammar: &str, scope: &str) -> BTreeSet<String> {
    let rule = format!("\"name\": \"{scope}\"");
    let at = grammar.find(&rule).unwrap_or_else(|| panic!("no rule for {scope}"));
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
const UNLISTED: [&str; 8] = ["INF", "Opt", "Counter", "argmax", "argmin", "columns", "last", "take"];

#[test]
fn the_vscode_tables_and_grammar_name_what_the_compiler_and_stdlib_ship() {
    let prelude = read("crates/cpl-check/src/prelude.rs");
    let type_names = rust_strings(&prelude, "const TYPE_NAMES: &[&str] = &[", "];");
    let mut builtins: BTreeSet<String> = type_names.into_iter().collect();
    builtins.extend(rust_strings(&prelude, "pub fn value(", "\n}\n"));
    builtins.extend(rust_strings(&prelude, "const VALUE_NAMES: &[&str] = &[", "];"));
    builtins.extend(rust_strings(&prelude, "pub fn ty(", "\n}\n"));

    let mut imported = BTreeSet::new();
    let prelude_cp = read("stdlib/prelude.cp");
    let mut rest = prelude_cp.as_str();
    while let Some(pos) = rest.find("use std.") {
        rest = &rest[pos + "use std.".len()..];
        let Some(open) = rest.find('{') else { break };
        let Some(close) = rest[open + 1..].find('}') else { break };
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
    let mut files: Vec<PathBuf> = fs::read_dir(root().join("stdlib"))
        .expect("stdlib/")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cp"))
        .collect();
    files.sort();
    for file in files {
        for line in fs::read_to_string(&file).expect("a stdlib file").lines() {
            let Some(rest) = line.strip_prefix("pub ") else { continue };
            let mut words = rest.split(' ').peekable();
            let mut getter = false;
            while let Some(attr) = words.peek().filter(|word| word.starts_with('@')) {
                getter |= *attr == "@get";
                words.next();
            }
            let kinds = ["fn", "struct", "enum", "trait", "type", "const", "domain"];
            let Some(kind) = words.next().and_then(|word| kinds.into_iter().find(|kind| *kind == word)) else {
                continue;
            };
            let name: String =
                words.next().unwrap_or("").chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
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
    let tables = ["PRELUDE_TYPES", "STDLIB_TYPES", "PRELUDE_VALUES", "STDLIB_VALUES", "PRELUDE_CONSTANTS"];
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
        "editors/vscode/src/languageData.ts and syntaxes/cp.tmLanguage.json do not list {missing:?}: add each to the table for its kind and to the grammar rule of the same name"
    );
    let phantom: Vec<&String> = listed.difference(&shipped).collect();
    assert!(
        phantom.is_empty(),
        "editors/vscode/src/languageData.ts lists {phantom:?}, which neither prelude.rs nor stdlib/*.cp declares"
    );

    let grammar = read("editors/vscode/syntaxes/cp.tmLanguage.json");
    let scopes = [
        ("support.type.builtin.cp", "PRELUDE_TYPES"),
        ("support.type.stdlib.cp", "STDLIB_TYPES"),
        ("support.function.builtin.cp", "PRELUDE_VALUES"),
        ("support.function.stdlib.cp", "STDLIB_VALUES"),
        ("support.constant.stdlib.cp", "PRELUDE_CONSTANTS"),
    ];
    for (scope, name) in scopes {
        assert_eq!(alternation(&grammar, scope), table(&ts, name), "the grammar rule {scope} and {name} differ");
    }
}
