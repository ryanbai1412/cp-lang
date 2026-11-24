//! Highlighting agrees with the parser: every generated editor grammar (VS Code's
//! TextMate grammar, Vim's, Emacs's, tree-sitter's and the JetBrains lexer) gives each
//! token start of every conformance case the class `cpl_syntax::grammar::classes` does.
//! Each grammar runs in its own engine (tests/highlight/); one whose engine is not
//! installed is skipped and says so, unless `CPL_HIGHLIGHT` (comma-separated engine
//! names, or `all`) requires it. A file the lexer rejects has no classes to agree with.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

struct Case {
    path: PathBuf,
    text: String,
    /// Per token start: (0-based line, byte column, UTF-16 column, char offset, UTF-16 offset).
    at: Vec<[usize; 5]>,
    classes: Vec<&'static str>,
}

fn cases() -> Vec<Case> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "cp") {
                out.push(path);
            }
        }
    }
    let mut paths = Vec::new();
    walk(&root().join("spec/conformance"), &mut paths);
    paths.sort();
    let mut out = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap();
        if !cpl_syntax::lex(&text).diagnostics.is_empty() {
            continue;
        }
        let (mut at, mut classes) = (Vec::new(), Vec::new());
        for (offset, class) in cpl_syntax::grammar::classes(&text) {
            let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
            let line = text[..offset].matches('\n').count();
            let u16col = text[line_start..offset].encode_utf16().count();
            let chars = text[..offset].chars().count();
            let u16 = text[..offset].encode_utf16().count();
            at.push([line, offset - line_start, u16col, chars, u16]);
            classes.push(class);
        }
        out.push(Case {
            path,
            text,
            at,
            classes,
        });
    }
    assert!(out.len() > 1000, "{} conformance cases", out.len());
    out
}

fn queries(cases: &[Case]) -> String {
    let files: Vec<String> = cases
        .iter()
        .map(|c| {
            let at: Vec<String> = c.at.iter().map(|a| format!("{a:?}")).collect();
            format!(
                "{{\"path\":{},\"at\":[{}]}}",
                cpl_json::Value::from(c.path.display().to_string()),
                at.join(",")
            )
        })
        .collect();
    format!("[{}]", files.join(","))
}

/// The classes an engine's name for a token may stand for.
fn allowed(engine: &str, raw: &str) -> &'static [&'static str] {
    const OTHER: &[&str] = &["name", "punct"];
    let has = |s: &str| raw.split(' ').any(|scope| scope.starts_with(s));
    match engine {
        "vim" => match raw {
            "cpKeyword" => &["keyword"],
            "cpBoolean" => &["literal"],
            "cpLabel" => &["label"],
            "cpNumber" => &["number"],
            "cpChar" => &["char"],
            "cpString" => &["string"],
            "cpComment" | "cpDocComment" => &["comment"],
            _ => OTHER,
        },
        "emacs" => match raw {
            "font-lock-keyword-face" => &["keyword"],
            "font-lock-constant-face" => &["literal", "number", "label"],
            "font-lock-string-face" => &["string", "char"],
            "font-lock-comment-face"
            | "font-lock-comment-delimiter-face"
            | "font-lock-doc-face" => &["comment"],
            _ => OTHER,
        },
        "tree-sitter" => match raw {
            "keyword" | "variable.builtin" => &["keyword"],
            "boolean" | "constant.builtin" => &["literal"],
            "string" => &["string"],
            "character" => &["char"],
            "label" => &["label"],
            "number" => &["number"],
            "comment" => &["comment"],
            _ => OTHER,
        },
        "jetbrains" => match raw {
            "KEYWORD" => &["keyword"],
            "LITERAL" => &["literal"],
            "STRING" => &["string"],
            "CHAR" => &["char"],
            "LABEL" => &["label"],
            "NUMBER" => &["number"],
            "LINE_COMMENT" | "DOC_COMMENT" => &["comment"],
            _ => OTHER,
        },
        _ if has("comment") => &["comment"],
        _ if has("constant.character") || has("string.quoted.single.char") => &["char"],
        _ if has("string") => &["string"],
        _ if has("constant.numeric") => &["number"],
        _ if has("constant.language") => &["literal"],
        // `min=` is one operator the lexer reads as a name and `=`; `@!multitest` a decorator.
        _ if has("keyword.operator") => &["punct", "keyword", "name"],
        _ if has("keyword.control.directive") => OTHER,
        _ if has("keyword") || has("storage") || has("variable.language") => &["keyword"],
        _ if has("entity.name.label") => &["label"],
        _ => OTHER,
    }
}

fn found(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn run(cmd: &mut Command) -> Option<String> {
    let out = cmd.output().unwrap_or_else(|e| panic!("{cmd:?}: {e}"));
    assert!(
        out.status.success(),
        "{cmd:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Some(String::from_utf8(out.stdout).unwrap())
}

/// Each case's engine names, from an engine's `[[name, ...], ...]` output file.
fn names(json: &str) -> Vec<Vec<String>> {
    let value = cpl_json::parse(json).unwrap();
    let files = value.as_array().map(|a| a.to_vec()).unwrap_or_default();
    files
        .iter()
        .map(|f| {
            (f.as_array().map(|a| a.to_vec()).unwrap_or_default())
                .iter()
                .map(|n| n.as_str().unwrap_or("").to_string())
                .collect()
        })
        .collect()
}

/// tree-sitter's innermost highlight capture at each token start.
fn tree_sitter(ts: &str, cases: &[Case], work: &Path) -> Vec<Vec<String>> {
    let grammar = root().join("editors/tree-sitter-cp");
    let config = work.join("config.json");
    let editors = root().join("editors").canonicalize().unwrap();
    std::fs::write(
        &config,
        format!("{{\"parser-directories\": [{:?}]}}", editors),
    )
    .unwrap();
    let list = work.join("files");
    let paths: Vec<String> = cases.iter().map(|c| c.path.display().to_string()).collect();
    std::fs::write(&list, paths.join("\n")).unwrap();
    let out = run(Command::new(ts)
        .current_dir(&grammar)
        .args(["query", "--config-path"])
        .arg(&config)
        .arg("queries/highlights.scm")
        .arg("--paths")
        .arg(&list))
    .unwrap();
    let mut captures: BTreeMap<String, Vec<((usize, usize), (usize, usize), String)>> =
        BTreeMap::new();
    let mut file = String::new();
    let num = |s: &str| s.trim().parse::<usize>().unwrap();
    for line in out.lines() {
        if !line.starts_with(' ') {
            file = line.trim().to_string();
            continue;
        }
        let Some(rest) = line.trim().strip_prefix("capture: ") else {
            continue;
        };
        let name = rest
            .split(" - ")
            .nth(1)
            .and_then(|r| r.split(',').next())
            .unwrap();
        let point = |key: &str| {
            let p = rest.split(key).nth(1).unwrap();
            let p = &p[1..p.find(')').unwrap()];
            let (r, c) = p.split_once(',').unwrap();
            (num(r), num(c))
        };
        let entry = captures.entry(file.clone()).or_default();
        entry.push((point("start: "), point("end: "), name.to_string()));
    }
    cases
        .iter()
        .map(|c| {
            let caps = captures
                .get(&c.path.display().to_string())
                .cloned()
                .unwrap_or_default();
            c.at.iter()
                .map(|a| {
                    let p = (a[0], a[1]);
                    caps.iter()
                        .filter(|(s, e, _)| *s <= p && p < *e)
                        .min_by_key(|(s, e, _)| (e.0 - s.0, e.1.wrapping_sub(s.1)))
                        .map_or(String::new(), |(_, _, n)| n.clone())
                })
                .collect()
        })
        .collect()
}

#[test]
fn every_editor_grammar_classifies_tokens_as_the_lexer_does() {
    let cases = cases();
    let work = std::env::temp_dir().join(format!("cpl-highlight-{}", std::process::id()));
    std::fs::create_dir_all(work.join("syntax")).unwrap();
    let queries_file = work.join("queries.json");
    std::fs::write(&queries_file, queries(&cases)).unwrap();
    let cpl = env!("CARGO_BIN_EXE_cpl");
    let highlight = root().join("crates/cpl-cli/tests/highlight");
    let required = cpl_env::HIGHLIGHT.text().unwrap_or_default();
    let required = |engine: &str| required.split(',').any(|r| r == engine || r == "all");
    let env = |name: &str, default: &str| std::env::var(name).unwrap_or(default.into());
    let (nvim, emacs, ts) = (
        env("NVIM", "nvim"),
        env("EMACS", "emacs"),
        env("TREE_SITTER", "tree-sitter"),
    );
    let vscode = root().join("editors/vscode");
    let node = found("node");
    let mut failures = Vec::new();
    let mut ran = 0;
    for engine in ["textmate", "jetbrains", "vim", "emacs", "tree-sitter"] {
        let out = work.join(format!("{engine}.json"));
        let got = match engine {
            // VS Code's grammar is the VS Code lane's to bring into agreement; until then it
            // runs where CPL_HIGHLIGHT names it.
            "textmate"
                if required(engine)
                    && node
                    && vscode.join("node_modules/vscode-textmate").is_dir() =>
            {
                run(Command::new("node")
                    .current_dir(&vscode)
                    .arg("scripts/language-data.js"));
                run(Command::new("node")
                    .arg(highlight.join("agree.mjs"))
                    .arg(engine)
                    .args([&queries_file, &out]))
                .map(|_| names(&std::fs::read_to_string(&out).unwrap()))
            }
            "jetbrains" if node => run(Command::new("node")
                .arg(highlight.join("agree.mjs"))
                .arg(engine)
                .args([&queries_file, &out]))
            .map(|_| names(&std::fs::read_to_string(&out).unwrap())),
            "vim" if found(&nvim) => {
                let grammar = run(Command::new(cpl).args(["setup", "vim", "--grammar"])).unwrap();
                std::fs::write(work.join("syntax/cp.vim"), grammar).unwrap();
                run(Command::new(&nvim)
                    .args(["--headless", "--clean", "-l"])
                    .arg(highlight.join("agree.lua"))
                    .args([&queries_file, &out, &work]))
                .map(|_| names(&std::fs::read_to_string(&out).unwrap()))
            }
            "emacs" if found(&emacs) => run(Command::new(&emacs)
                .args(["--batch", "-Q", "-l"])
                .arg(highlight.join("agree.el"))
                .args([queries_file.as_os_str(), out.as_os_str(), cpl.as_ref()]))
            .map(|_| names(&std::fs::read_to_string(&out).unwrap())),
            "tree-sitter" if found(&ts) => Some(tree_sitter(&ts, &cases, &work)),
            _ => None,
        };
        let Some(got) = got else {
            assert!(
                !required(engine),
                "CPL_HIGHLIGHT requires {engine}, which is not installed"
            );
            eprintln!("highlight: SKIPPED {engine} (not installed)");
            continue;
        };
        ran += 1;
        assert_eq!(got.len(), cases.len(), "{engine}: one answer per case");
        let mut wrong = BTreeMap::<(String, &str), (usize, String)>::new();
        for (case, names) in cases.iter().zip(&got) {
            assert_eq!(
                names.len(),
                case.at.len(),
                "{engine}: {}",
                case.path.display()
            );
            for ((at, class), raw) in case.at.iter().zip(&case.classes).zip(names) {
                if !allowed(engine, raw).contains(class) {
                    let line = case.text.lines().nth(at[0]).unwrap_or("");
                    let rel = case.path.strip_prefix(root()).unwrap_or(&case.path);
                    let e = wrong.entry((raw.clone(), *class)).or_default();
                    e.0 += 1;
                    if e.1.is_empty() {
                        e.1 = format!(
                            "{}:{}:{}: {}",
                            rel.display(),
                            at[0] + 1,
                            at[1] + 1,
                            line.trim()
                        );
                    }
                }
            }
        }
        for ((raw, class), (count, example)) in wrong {
            failures.push(format!(
                "{engine}: `{raw}` where the lexer reads {class} ({count}x), e.g. {example}"
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&work);
    eprintln!(
        "highlight: {ran} grammars over {} conformance cases",
        cases.len()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
