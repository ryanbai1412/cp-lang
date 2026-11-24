//! The VS Code extension's settings and decorators agree with what the
//! extension reads and the compiler accepts. Its name tables and grammar are
//! generated from the compiler's prelude and `std/*.cp` at compile time
//! (`editors/vscode/scripts/language-data.js`, decision 572), so they can't drift.

use std::collections::BTreeMap;
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

/// `cp.<key>` read as `getConfiguration('cp').get<..>('<key>', <default>)`,
/// with or without the type argument, in the extension's sources: the default
/// as written, spaces dropped and quotes made double.
fn reads(sources: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for part in sources.split(".get").skip(1) {
        let call = match part.strip_prefix('<') {
            Some(typed) => typed.split_once(">(").map(|(_, rest)| rest),
            None => part.strip_prefix('('),
        };
        let Some(call) = call else {
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
        .at(&["contributes", "configuration"])
        .as_array()
        .expect("package.json declares its settings in sections")
        .iter()
        .flat_map(|section| {
            section
                .at(&["properties"])
                .as_object()
                .expect("a section's settings")
                .iter()
        })
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
    // The `cp` section VS Code sends the server (`synchronize`), with
    // package.json's defaults filled in: what `cpl.exec` reads to complete a run.
    let server = read("crates/cpl-lsp/src/server/exec.rs");
    let synced = sources.contains("configurationSection: 'cp'");
    let server_reads: Vec<String> = server
        .split("setting(&[")
        .skip(1)
        .filter_map(|s| s.split_once("])"))
        .map(|(path, _)| format!("cp.{}", path.replace(['"', ' '], "").replace(',', ".")))
        .collect();
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
        } else if synced && server_reads.contains(key) {
            Some(default.clone())
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

/// Every `@name(...)` the editor plugins write, document or test is one the
/// compiler accepts: a spelling it rejects as removed (E0503) cannot linger
/// in a README, a setting or a lens.
#[test]
fn the_editors_spell_only_decorators_the_compiler_accepts() {
    let mut stale = Vec::new();
    let mut dirs = vec![root().join("editors")];
    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).expect("a dir") {
            let path = entry.expect("an entry").path();
            let name = path.file_name().unwrap().to_string_lossy();
            if path.is_dir() {
                if !matches!(
                    name.as_ref(),
                    "node_modules" | "out" | "build" | ".gradle" | "target"
                ) {
                    dirs.push(path);
                }
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for (at, _) in text.match_indices('@') {
                let rest = &text[at + 1..];
                let len = rest
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(rest.len());
                let word = &rest[..len];
                if !is_word(word)
                    || at > 0 && text[..at].ends_with(|c: char| c.is_ascii_alphanumeric())
                {
                    continue;
                }
                let args = rest[len..]
                    .strip_prefix('(')
                    .and_then(|r| r.find(')').map(|end| &r[..end]))
                    .filter(|a| a.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
                let decorator = match args {
                    Some(a) => format!("@{word}({a})"),
                    None => format!("@{word}"),
                };
                let program = format!("{decorator}\nfn f():\n    print(1)\n");
                let parsed = cpl_syntax::parse_file(&program);
                if parsed.diagnostics.iter().any(|d| d.code.0 == "E0503") {
                    let rel = path
                        .strip_prefix(root())
                        .unwrap_or(&path)
                        .display()
                        .to_string();
                    stale.push(format!("{rel}: {decorator}"));
                }
            }
        }
    }
    stale.sort();
    stale.dedup();
    assert!(
        stale.is_empty(),
        "removed decorator spellings in editors/:\n{}",
        stale.join("\n")
    );
}
