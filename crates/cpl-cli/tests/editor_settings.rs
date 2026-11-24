//! Every editor exposes exactly the settings `spec/tools/lsp.md` lists (Rlsp.9):
//! each one read from the plugin's own declarations, with the default the
//! spec gives, named in that editor's README, at most 15 per editor. A setting
//! that appears, disappears or changes its default without the spec fails here.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_PER_EDITOR: usize = 15;

/// Setting to default, normalised: spaces dropped and quotes made double.
type Settings = BTreeMap<String, Option<String>>;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn norm(value: &str) -> String {
    value
        .trim()
        .trim_matches('`')
        .replace('\'', "\"")
        .replace(' ', "")
}

fn files(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if !path.ends_with("test") && !path.ends_with("node_modules") {
                files(&path, exts, out);
            }
        } else if path
            .extension()
            .is_some_and(|e| exts.iter().any(|x| e == *x))
        {
            out.push(path);
        }
    }
}

fn word(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        .unwrap_or(text.len());
    &text[..end]
}

fn vscode() -> Settings {
    let package = cpl_json::parse(&read(&root().join("editors/vscode/package.json"))).unwrap();
    package
        .at(&["contributes", "configuration", "properties"])
        .as_object()
        .expect("package.json declares its settings")
        .iter()
        .map(|(k, v)| (k.clone(), Some(norm(&v.at(&["default"]).to_string()))))
        .collect()
}

/// `get(g:, 'cpl_x', default)`, `vim.g.cpl_x == false` (default true) and
/// bare `g:cpl_x` reads in the Vim and Lua sources; comments are skipped.
fn neovim() -> Settings {
    let mut paths = Vec::new();
    files(&root().join("editors/neovim"), &["vim", "lua"], &mut paths);
    files(&root().join("editors/vim"), &["vim"], &mut paths);
    let mut defaults: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in paths {
        for line in read(&path).lines() {
            let code = line.trim_start();
            if code.starts_with('"') || code.starts_with("--") {
                continue;
            }
            for (i, _) in code.match_indices("cpl_") {
                let before = &code[..i];
                let name = word(&code[i..]);
                let after = code[i + name.len()..].trim_start();
                let default = if before.ends_with("get(g:, '") {
                    let rest = after
                        .trim_start_matches('\'')
                        .trim_start_matches([',', ' ']);
                    Some(norm(&rest[..rest.find(')').unwrap_or(rest.len())]))
                } else if before.ends_with("vim.g.") {
                    if let Some(rest) = after.strip_prefix("or ") {
                        Some(norm(rest.split([' ', ')']).next().unwrap_or(rest)))
                    } else {
                        (after.starts_with("== false") || after.starts_with("~= false"))
                            .then(|| "true".to_string())
                    }
                } else if before.ends_with("g:") {
                    None
                } else {
                    continue;
                };
                let seen = defaults.entry(format!("g:{name}")).or_default();
                seen.extend(default);
            }
        }
    }
    defaults
        .into_iter()
        .map(|(k, mut ds)| {
            ds.sort();
            ds.dedup();
            assert!(ds.len() <= 1, "{k}: read with defaults {ds:?}");
            (k, ds.pop())
        })
        .collect()
}

/// `(defcustom name default ...)`: Emacs's user options.
fn emacs() -> Settings {
    let mut paths = Vec::new();
    files(&root().join("editors/emacs"), &["el"], &mut paths);
    let mut out = Settings::new();
    for path in paths.iter().filter(|p| !p.ends_with("smoke.el")) {
        for line in read(path).lines() {
            if let Some(rest) = line.trim_start().strip_prefix("(defcustom ") {
                let name = word(rest);
                out.insert(name.to_string(), Some(norm(&rest[name.len()..])));
            }
        }
    }
    out
}

/// `var name by kind(default)` in a `BaseState` the plugin persists.
fn jetbrains() -> Settings {
    let mut paths = Vec::new();
    files(
        &root().join("editors/jetbrains/src/main"),
        &["kt"],
        &mut paths,
    );
    let mut out = Settings::new();
    for path in paths {
        let mut in_state = false;
        for line in read(&path).lines() {
            let code = line.trim();
            if code.contains(": BaseState()") {
                in_state = true;
            } else if in_state && code == "}" {
                in_state = false;
            } else if let Some(rest) = code.strip_prefix("var ").filter(|_| in_state) {
                let name = word(rest);
                let default = rest
                    .split_once('(')
                    .and_then(|(_, d)| d.strip_suffix(')'))
                    .map(norm);
                out.insert(name.to_string(), default);
            }
        }
    }
    out
}

/// The `LspSettings` fields the extension reads, under its server's id in
/// `extension.toml`: `s.<field>` and the binary's `b.<field>`. Zed itself
/// supplies the defaults, so they are not compared.
fn zed() -> Settings {
    let manifest = read(&root().join("editors/zed/extension.toml"));
    let id = manifest
        .lines()
        .find_map(|l| l.strip_prefix("[language_servers.")?.strip_suffix(']'))
        .expect("extension.toml names its language server");
    let source = read(&root().join("editors/zed/src/lib.rs"));
    let mut out = Settings::new();
    for (prefix, under) in [
        ("|s| s.", format!("lsp.{id}.")),
        ("|b| b.", format!("lsp.{id}.binary.")),
    ] {
        for (i, _) in source.match_indices(prefix) {
            let field = word(&source[i + prefix.len()..]);
            if field != "binary" {
                out.insert(format!("{under}{field}"), None);
            }
        }
    }
    out
}

/// `| Editor | `setting` | `default` | ... |` rows of spec/tools/lsp.md.
fn spec() -> BTreeMap<String, Settings> {
    let mut out: BTreeMap<String, Settings> = BTreeMap::new();
    for line in read(&root().join("spec/tools/lsp.md")).lines() {
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split(" | ")
            .map(str::trim)
            .collect();
        let [editor, setting, default, ..] = cells[..] else {
            continue;
        };
        let Some(setting) = setting.strip_prefix('`').and_then(|s| s.strip_suffix('`')) else {
            continue;
        };
        out.entry(editor.to_string())
            .or_default()
            .insert(setting.to_string(), Some(norm(default)));
    }
    out
}

/// The README section a user reads for an editor's settings.
fn docs(editor: &str) -> String {
    let (file, heading) = match editor {
        "VS Code" => ("editors/vscode/README.md", "## Settings"),
        "Neovim and Vim" => ("editors/README.md", "## Neovim"),
        "Emacs" => ("editors/README.md", "## Emacs"),
        "JetBrains" => ("editors/README.md", "## JetBrains"),
        "Zed" => ("editors/README.md", "## Zed"),
        other => panic!("{other}: no README section known"),
    };
    let text = read(&root().join(file));
    let section = text
        .split(heading)
        .nth(1)
        .unwrap_or_else(|| panic!("{file} has no `{heading}` section"));
    section.split("\n## ").next().unwrap().to_string()
}

#[test]
fn every_editor_exposes_exactly_the_settings_the_spec_lists() {
    let found: BTreeMap<String, Settings> = [
        ("VS Code", vscode()),
        ("Neovim and Vim", neovim()),
        ("Emacs", emacs()),
        ("JetBrains", jetbrains()),
        ("Zed", zed()),
    ]
    .into_iter()
    .map(|(e, s)| (e.to_string(), s))
    .collect();
    let spec = spec();
    let mut wrong = Vec::new();
    for editor in spec.keys().filter(|e| !found.contains_key(*e)) {
        wrong.push(format!(
            "{editor}: in the spec, but no plugin is read for it"
        ));
    }
    for (editor, settings) in &found {
        if settings.len() > MAX_PER_EDITOR {
            wrong.push(format!(
                "{editor}: {} settings; keep them few",
                settings.len()
            ));
        }
        let listed = spec.get(editor).cloned().unwrap_or_default();
        let readme = docs(editor);
        for (key, default) in settings {
            let Some(spec_default) = listed.get(key) else {
                wrong.push(format!(
                    "{editor}: {key} is exposed but not in spec/tools/lsp.md"
                ));
                continue;
            };
            match default {
                None if editor != "Zed" => {
                    wrong.push(format!("{editor}: {key} has no default the plugin states"))
                }
                Some(d) if Some(d) != spec_default.as_ref() => wrong.push(format!(
                    "{editor}: {key} defaults to {d} in the plugin, {} in the spec",
                    spec_default.as_deref().unwrap_or("?")
                )),
                _ => {}
            }
            let lua = key.strip_prefix("g:").map(|k| format!("vim.g.{k}"));
            if !readme.contains(&format!("`{key}`")) && !lua.is_some_and(|k| readme.contains(&k)) {
                wrong.push(format!("{editor}: {key} is not in its README"));
            }
        }
        for key in listed.keys().filter(|k| !settings.contains_key(*k)) {
            wrong.push(format!(
                "{editor}: {key} is in the spec but the plugin has no such setting"
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "editor settings out of step with spec/tools/lsp.md:\n{}",
        wrong.join("\n")
    );
}
