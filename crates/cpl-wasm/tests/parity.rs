//! The playground against `cpl`.
//!
//! A program has to mean in the browser what it means on the command line.
//! Every program `cpl`'s own tests run (`crates/cpl-cli/tests/programs`)
//! and every hero program on the site (`web/content/landing/hero`) goes through both
//! here, and the two must agree on each byte: what the program prints, what
//! `cpl` says on standard error, the exit status, and the C++.
//!
//! This runs natively, where the playground is the same code as in the
//! `.wasm` file apart from the interpreter's stack. `tests/smoke.mjs` runs
//! the built module itself.
//!
//! Enforces `handbook/guidelines/commands-checklist.md#r10-one-entry-point-per-job`.

use cpl_json::Value;
use cpl_wasm::Playground;
use std::path::{Path, PathBuf};

fn programs() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut found = Vec::new();
    for dir in ["crates/cpl-cli/tests/programs", "web/content/landing/hero"] {
        let dir = root.join(dir);
        let entries = std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        found.extend(
            entries
                .map(|entry| entry.expect("readable entry").path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "cp")),
        );
    }
    found.sort();
    assert!(found.len() >= 10, "only {} programs", found.len());
    found
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("`{key}` is not a string in {value}"))
}

fn number(value: &Value, path: &[&str]) -> u32 {
    value
        .at(path)
        .as_u32()
        .unwrap_or_else(|| panic!("{path:?} is not a number in {value}"))
}

fn exit_code(value: &Value) -> u8 {
    let code = number(value, &["exitCode"]);
    assert_eq!(value.get("ok"), Some(&Value::Bool(code == 0)), "{value}");
    code as u8
}

/// The playground's `explain` is `cpl explain`'s, storage section included.
#[test]
fn explain_matches_cpl_explain() {
    let mut playground = Playground::new();
    for path in programs() {
        let name = path.to_str().expect("a UTF-8 path");
        let source = std::fs::read_to_string(&path).expect("readable program");
        let cli = cpl_cli::run_with_input(&["explain", name], b"");
        let explained = playground.explain(name, &source);
        if cli.code == 0 {
            assert_eq!(text(&explained, "text").as_bytes(), cli.stdout, "{name}");
        }
    }
}

#[test]
fn run_matches_cpl_run() {
    // One session for every program, as a page keeps one.
    let mut playground = Playground::new();
    for path in programs() {
        let name = path.to_str().expect("a UTF-8 path");
        let source = std::fs::read_to_string(&path).expect("readable program");
        let input = std::fs::read(path.with_extension("in")).unwrap_or_default();
        // `cpl run` plays a file's `@interactor` against it on a second
        // thread, which the playground does not have: there the program
        // stops at its first unanswered question.
        if source.contains("\n@interactor") {
            continue;
        }

        let cli = cpl_cli::run_with_input(&["run", name], &input);
        let run = playground.run(name, &source, &input);

        assert_eq!(run.stdout, cli.stdout, "{name}: stdout");
        assert_eq!(text(&run.result, "stderr"), cli.stderr, "{name}: stderr");
        assert_eq!(exit_code(&run.result), cli.code, "{name}: exit status");
    }
}

#[test]
fn build_matches_cpl_build() {
    let mut playground = Playground::new();
    for path in programs() {
        let name = path.to_str().expect("a UTF-8 path");
        let file = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a file name");
        let source = std::fs::read_to_string(&path).expect("readable program");

        let cli = cpl_cli::run(&["build", name, "-o", "-"]);
        let build = playground.build(file, &source);

        assert_eq!(exit_code(&build), cli.code, "{name}: exit status");
        match build.get("cpp").unwrap_or(&Value::Null) {
            Value::Null => assert!(cli.stdout.is_empty(), "{name}: only cpl wrote C++"),
            cpp => assert_eq!(
                cpp.as_str().expect("a string").as_bytes(),
                cli.stdout,
                "{name}: C++"
            ),
        }
    }
}

#[test]
fn check_matches_cpl_check() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("parity");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let mut playground = Playground::new();
    let cases = [
        ("ok", "let n: Int = read()\nprint(n * 2)\n"),
        ("syntax", "let n: Int = read(\nprint(n)\n"),
        (
            "index",
            "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nprint(a[n])\n",
        ),
        ("unknown", "print(total)\n"),
        ("types", "let n: Int = \"three\"\nprint(n)\n"),
    ];
    for (case, source) in cases {
        let path = dir.join(format!("{case}.cp"));
        std::fs::write(&path, source).expect("a writable scratch file");
        let name = path.to_str().expect("a UTF-8 path");

        let cli = cpl_cli::run(&["check", name]);
        let check = playground.check(name, source);

        assert_eq!(text(&check, "stderr"), cli.stderr, "{case}: stderr");
        assert_eq!(exit_code(&check), cli.code, "{case}: exit status");
        let diagnostics = check.at(&["diagnostics"]).as_array().unwrap_or(&[]);
        assert_eq!(diagnostics.is_empty(), cli.code == 0, "{case}: diagnostics");
    }
}

#[test]
fn a_failed_run_keeps_its_output_and_says_where_it_failed() {
    let source = "print(1)\nlet a: List<Int> = [1, 2, 3]\nlet i: Int = read()\nprint(a[i])\n";
    let run = Playground::new().run("main.cp", source, b"7");
    assert_eq!(run.stdout, b"1\n");
    assert_eq!(exit_code(&run.result), cpl_wasm::EXIT_FAILED);
    let diagnostics = run.result.at(&["diagnostics"]).as_array().unwrap_or(&[]);
    let [failure] = diagnostics else {
        panic!("one failure, not {}", run.result);
    };
    assert_eq!(text(failure, "code"), "R0101");
    assert_eq!(number(failure, &["range", "start", "line"]), 3);
    assert!(
        text(&run.result, "stderr").contains("main.cp:4:"),
        "{}",
        run.result
    );
}

#[test]
fn positions_are_utf16_code_units() {
    // `é` is two bytes and one code unit, `😀` four bytes and two.
    let source = "// é😀\nprint(total)\n";
    let check = Playground::new().check("main.cp", source);
    let diagnostics = check.at(&["diagnostics"]).as_array().unwrap_or(&[]);
    let [unknown] = diagnostics else {
        panic!("one diagnostic, not {check}");
    };
    let units: Vec<u16> = source.encode_utf16().collect();
    let (start, end) = (number(unknown, &["start"]), number(unknown, &["end"]));
    assert_eq!(
        String::from_utf16_lossy(&units[start as usize..end as usize]),
        "total"
    );
    assert_eq!(number(unknown, &["range", "start", "line"]), 1);
    assert_eq!(number(unknown, &["range", "start", "character"]), 6);
}

#[test]
fn hover_definition_and_symbols_answer_at_an_offset() {
    let source = "fn double(x: Int) -> Int:\n    x * 2\nlet n: Int = read()\nprint(double(n))\n";
    let mut playground = Playground::new();
    let call = source.rfind("double").expect("the call") as u32;

    let hover = playground.hover("main.cp", source, call);
    assert!(text(&hover, "text").contains("double"), "{hover}");
    assert_eq!(number(&hover, &["start"]), call);

    let definition = playground.definition("main.cp", source, call);
    assert_eq!(number(&definition, &["start"]), 3, "{definition}");

    assert_eq!(playground.hover("main.cp", source, 0), Value::Null);

    let symbols = playground.symbols("main.cp", source);
    let names: Vec<(&str, &str)> = symbols
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|symbol| (text(symbol, "kind"), text(symbol, "name")))
        .collect();
    assert_eq!(names, [("fn", "double"), ("value", "n")]);
}

#[test]
fn lsp_messages_reach_the_language_server() {
    let mut playground = Playground::new();
    let replies = playground.lsp(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let [reply] = replies.as_array().unwrap_or(&[]) else {
        panic!("one reply, not {replies}");
    };
    assert_eq!(number(reply, &["id"]), 1);
    assert!(!reply.at(&["result", "capabilities"]).is_null(), "{reply}");

    let replies = playground.lsp("{");
    let error = replies.as_array().unwrap_or(&[])[0].at(&["error"]);
    assert_eq!(error.get("code"), Some(&Value::Int(-32700)));
}

#[test]
fn error_and_trap_stderr_matches_cli_verbatim() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("parity-errors");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let mut playground = Playground::new();

    // Compile-time errors and warnings: `check` and `run` must match CLI stderr byte for byte.
    let check_cases = [
        ("e0103_syntax", "print(1 + )\n"),
        ("e0201_unknown", "print(missing_name)\n"),
        ("e0203_method", "let x = 5\nprint(x.to_str())\n"),
        ("e0302_untyped_read", "let k = read()\nprint(k)\n"),
        ("e0303_arity", "let k = read(Int)\nprint(k)\n"),
        ("w0401_unused_expr", "1 + 2\nprint(3)\n"),
    ];
    for (case, source) in check_cases {
        let path = dir.join(format!("{case}.cp"));
        std::fs::write(&path, source).expect("a writable scratch file");
        let name = path.to_str().expect("a UTF-8 path");

        let cli = cpl_cli::run(&["check", name]);
        let check = playground.check(name, source);
        assert_eq!(text(&check, "stderr"), cli.stderr, "{case}: check stderr");
        assert_eq!(exit_code(&check), cli.code, "{case}: check exit status");
    }

    // Runtime traps: `run` must match CLI stderr, stdout, and exit code 3 byte for byte.
    let trap_cases: [(&str, &str, &[u8]); 4] = [
        (
            "r0101_locals",
            "fn at(xs: List<Int>, k: Int) -> Int:\n    let j = k + 1\n    return xs[j]\n\nprint(at([1, 2], 1))\n",
            b"",
        ),
        (
            "r0001_bounds",
            "let a: List<Int> = [10, 20]\nlet i: Int = read()\nprint(a[i])\n",
            b"5\n",
        ),
        (
            "r0001_divzero",
            "let (a, b): (Int, Int) = read()\nprint(a / b)\n",
            b"10 0\n",
        ),
        (
            "r0001_range_read",
            "let n: 1..=10 = read()\nprint(n)\n",
            b"99\n",
        ),
    ];
    for (case, source, input) in trap_cases {
        let path = dir.join(format!("{case}.cp"));
        std::fs::write(&path, source).expect("a writable scratch file");
        let name = path.to_str().expect("a UTF-8 path");

        let cli = cpl_cli::run_with_input(&["run", name], input);
        let run = playground.run(name, source, input);
        assert_eq!(run.stdout, cli.stdout, "{case}: trap stdout");
        assert_eq!(
            text(&run.result, "stderr"),
            cli.stderr,
            "{case}: trap stderr"
        );
        assert_eq!(exit_code(&run.result), cli.code, "{case}: trap exit status");
    }
}

#[test]
fn stress_matches_cpl_stress_json() {
    let dir = std::env::temp_dir().join(format!("cpl-wasm-stress-parity-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let mut playground = Playground::new();

    // 1. Failing planted bug: CLI and wasm must produce the exact same JSON
    //    report on the same seed (apart from `saved`, which is filesystem-only).
    let buggy = "\
let n: 1..=10 = read()
let a: [0..n]1..=100 = read()
var best = 0
for i in 0..n:
    if i > 0: best max= a[i]
print(best)

@alt(program)
fn slow():
    var best = 0
    for x in a: best max= x
    print(best)
";
    let bug_path = dir.join("bug.cp");
    std::fs::write(&bug_path, buggy).unwrap();
    let bug_name = bug_path.to_str().unwrap();

    let cli = cpl_cli::run(&[
        "stress",
        bug_name,
        "--cases",
        "50",
        "--seed",
        "1",
        "--size",
        "6",
        "--jobs",
        "1",
        "--explain",
        "--format",
        "json",
    ]);
    assert_eq!(cli.code, 1, "CLI should find the bug: {}", cli.stderr);
    let cli_json = String::from_utf8(cli.stdout).unwrap();
    let wasm_json = playground.stress(bug_name, buggy, 50, 1, 6);

    let mut cli_val = cpl_json::parse(&cli_json).expect("valid CLI JSON");
    let wasm_val = cpl_json::parse(&wasm_json).expect("valid wasm JSON");
    // Normalize `saved` (filesystem path on CLI, null in wasm).
    if let Value::Object(members) = &mut cli_val {
        if let Some(Value::Object(ce)) = members.get_mut("counterexample") {
            if let Some(saved) = ce.get_mut("saved") {
                *saved = Value::Null;
            }
        }
    }
    assert_eq!(wasm_val, cli_val, "wasm:\n{wasm_json}\ncli:\n{cli_json}");
    assert_eq!(wasm_val.at(&["status"]).as_str(), Some("fail"));
    assert_eq!(
        wasm_val.at(&["counterexample", "raw_input"]).as_str(),
        Some("1\n1\n")
    );
    assert_eq!(
        wasm_val
            .at(&["counterexample", "explain", "divergence", "name"])
            .as_str(),
        Some("best")
    );

    // 2. Clean solution: both agree on status "ok" and case count.
    let clean = buggy.replace("if i > 0: best max= a[i]", "best max= a[i]");
    let clean_path = dir.join("clean.cp");
    std::fs::write(&clean_path, &clean).unwrap();
    let clean_name = clean_path.to_str().unwrap();
    let cli_ok = cpl_cli::run(&[
        "stress", clean_name, "--cases", "20", "--seed", "1", "--size", "6", "--jobs", "1",
        "--format", "json",
    ]);
    assert_eq!(cli_ok.code, 0, "{}", cli_ok.stderr);
    let wasm_ok = playground.stress(clean_name, &clean, 20, 1, 6);
    assert_eq!(wasm_ok, String::from_utf8(cli_ok.stdout).unwrap());

    // 3. A solution that never finishes: the same shrunk case, and the
    //    loop it goes round in named alike.
    let stuck = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/stress/cold-lower-bound-stuck.cp"),
    )
    .unwrap();
    let stuck_path = dir.join("stuck.cp");
    std::fs::write(&stuck_path, &stuck).unwrap();
    let stuck_name = stuck_path.to_str().unwrap();
    let cli_stuck = cpl_cli::run(&[
        "stress",
        stuck_name,
        "--cases",
        "5",
        "--seed",
        "1",
        "--size",
        "8",
        "--jobs",
        "1",
        "--explain",
        "--format",
        "json",
    ]);
    let cli_json = String::from_utf8(cli_stuck.stdout).unwrap();
    let mut cli_val = cpl_json::parse(&cli_json).expect("valid CLI JSON");
    if let Value::Object(members) = &mut cli_val {
        if let Some(Value::Object(ce)) = members.get_mut("counterexample") {
            if let Some(saved) = ce.get_mut("saved") {
                *saved = Value::Null;
            }
        }
    }
    let wasm_json = playground.stress(stuck_name, &stuck, 5, 1, 8);
    let wasm_val = cpl_json::parse(&wasm_json).expect("valid wasm JSON");
    assert_eq!(wasm_val, cli_val, "wasm:\n{wasm_json}\ncli:\n{cli_json}");
    assert_eq!(
        wasm_val
            .at(&["counterexample", "explain", "divergence", "name"])
            .as_str(),
        Some("while lo < hi"),
        "{wasm_json}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// Every lint record's `before.cp` and `after.cp` lint in the playground
/// exactly as in `cpl lint` and the editor, which share `lint_analysis`.
/// The playground's `fix` makes the file `cpl fix` writes.
#[test]
fn fix_matches_cpl_fix() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("parity-fix");
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let mut playground = Playground::new();
    let cases = [
        ("clean", "let n: Int = read()\nprint(n * 2)\n"),
        (
            "by_twin",
            "let xs = [\"bbb\", \"a\"]\nprint(min_by(xs, s => s.len))\n",
        ),
        (
            "elif",
            "let n: Int = read()\nif n > 0: print(1) else if n < 0: print(2)\n",
        ),
        ("upper_let", "let NEG = 1\nprint(NEG)\n"),
        ("left", "print(total)\n"),
    ];
    for (case, source) in cases {
        let path = dir.join(format!("{case}.cp"));
        std::fs::write(&path, source).expect("a writable scratch file");
        let name = path.to_str().expect("a UTF-8 path");

        let cli = cpl_cli::run(&["fix", name]);
        let written = std::fs::read_to_string(&path).expect("readable fixed file");
        let fix = playground.fix(name, source);
        if !matches!(case, "clean" | "left") {
            assert_ne!(written, source, "{case}: cpl fix changed nothing");
        }

        assert_eq!(text(&fix, "fixed"), written, "{case}: fixed text");
        assert_eq!(exit_code(&fix), cli.code, "{case}: exit status");
        assert_eq!(text(&fix, "stderr"), cli.stderr, "{case}: stderr");
    }
}

#[test]
fn the_playground_lints_like_cpl_lint() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/lints");
    let mut records: Vec<PathBuf> = std::fs::read_dir(&root)
        .expect("spec/lints")
        .map(|e| e.expect("entry").path())
        .collect();
    records.sort();
    assert!(records.len() >= 40, "only {} lint records", records.len());
    let mut playground = Playground::new();
    for record in records {
        for case in ["before.cp", "after.cp"] {
            let path = record.join(case);
            let source = std::fs::read_to_string(&path).expect("record file");
            let codes = |diags: &[cpl_diag::Diagnostic]| -> Vec<String> {
                diags.iter().map(|d| d.code.0.to_string()).collect()
            };
            let mut db = cpl_check::Database::new();
            db.set_file(case, source.as_str());
            let analysis = db.analyze(case).expect("record analyzes");
            let want = codes(&cpl_lint::lint_analysis(
                &analysis,
                &cpl_lint::Config::default(),
            ));
            let got: Vec<String> = playground
                .lint(case, &source)
                .get("diagnostics")
                .and_then(Value::as_array)
                .expect("diagnostics")
                .iter()
                .map(|d| text(d, "code").to_string())
                .collect();
            assert_eq!(got, want, "{}", path.display());
        }
    }
}

/// The playground's trace is `cpl run --trace --format json`'s document.
#[test]
fn trace_matches_cpl_run_trace() {
    let mut playground = Playground::new();
    for path in programs() {
        let name = path.to_str().expect("a UTF-8 path");
        let source = std::fs::read_to_string(&path).expect("readable program");
        if source.contains("\n@interactor") {
            continue;
        }
        let input = std::fs::read(path.with_extension("in")).unwrap_or_default();
        let cli = cpl_cli::run_with_input(&["run", name, "--trace", "--format", "json"], &input);
        // A page names its file without a folder, as the trace's title does.
        let file = path.file_name().unwrap().to_str().unwrap();
        let (_, json) = playground.trace_with_stack(file, &source, &input, cpl_wasm::MAX_STACK);
        let traced = cpl_json::parse(&json).expect("the playground's trace is JSON");
        if cli.code != 0 || traced.at(&["trace"]).is_null() {
            continue;
        }
        let document = cpl_json::parse(&String::from_utf8_lossy(&cli.stdout))
            .unwrap_or_else(|e| panic!("{name}: `cpl run --trace` printed no JSON: {e}"));
        if let Some(at) = first_mismatch(traced.at(&["trace"]), &document, String::new()) {
            panic!("{name}: the traces part at {at}");
        }
    }
}

/// The editor's fix-all is `cpl fix`: the same file, rewritten the same way.
#[test]
fn fix_all_matches_cpl_fix() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixes = root.join("crates/cpl-cli/tests/fixes");
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("fix-all-parity");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let mut sources: Vec<PathBuf> = std::fs::read_dir(&fixes)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    sources.sort();
    assert!(sources.len() >= 10, "only {} fix cases", sources.len());
    let mut differ = Vec::new();
    for (id, path) in sources.iter().enumerate() {
        let source = std::fs::read_to_string(path).unwrap();
        let copy = scratch.join(path.file_name().unwrap());
        std::fs::write(&copy, &source).unwrap();
        let name = copy.to_str().unwrap();
        cpl_cli::run_with_input(&["fix", name], b"");
        let by_cli = std::fs::read_to_string(&copy).unwrap();

        let mut playground = Playground::new();
        let uri = format!("file://{name}");
        playground.lsp(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
        let open = cpl_json::obj([
            ("jsonrpc", "2.0".into()),
            ("method", "textDocument/didOpen".into()),
            (
                "params",
                cpl_json::obj([(
                    "textDocument",
                    cpl_json::obj([
                        ("uri", uri.as_str().into()),
                        ("languageId", "cpl".into()),
                        ("version", 1.into()),
                        ("text", source.as_str().into()),
                    ]),
                )]),
            ),
        ]);
        playground.lsp(&open.to_string());
        let zero = cpl_json::obj([("line", 0.into()), ("character", 0.into())]);
        let ask = cpl_json::obj([
            ("jsonrpc", "2.0".into()),
            ("id", (id as i64 + 2).into()),
            ("method", "textDocument/codeAction".into()),
            (
                "params",
                cpl_json::obj([
                    (
                        "textDocument",
                        cpl_json::obj([("uri", uri.as_str().into())]),
                    ),
                    (
                        "range",
                        cpl_json::obj([("start", zero.clone()), ("end", zero)]),
                    ),
                    (
                        "context",
                        cpl_json::obj([
                            ("diagnostics", Vec::<Value>::new().into()),
                            ("only", vec![Value::from("source.fixAll")].into()),
                        ]),
                    ),
                ]),
            ),
        ]);
        let replies = playground.lsp(&ask.to_string());
        let reply = replies
            .as_array()
            .unwrap_or(&[])
            .iter()
            .find(|r| !r.at(&["result"]).is_null())
            .cloned()
            .unwrap_or(Value::Null);
        let actions = reply.at(&["result"]).as_array().unwrap_or(&[]).to_vec();
        let by_editor = match actions.first() {
            None => source.clone(),
            Some(action) => apply(
                &source,
                action
                    .at(&["edit", "changes", uri.as_str()])
                    .as_array()
                    .unwrap_or(&[]),
            ),
        };
        if by_editor != by_cli {
            differ.push(format!(
                "{}:\n--- cpl fix\n{by_cli}\n--- editor fix-all\n{by_editor}",
                path.file_name().unwrap().to_string_lossy()
            ));
        }
    }
    assert!(
        differ.is_empty(),
        "{} of {} differ:\n{}",
        differ.len(),
        sources.len(),
        differ.join("\n")
    );
}

/// `source` with LSP `edits` applied (ASCII text: a character is a byte).
fn apply(source: &str, edits: &[Value]) -> String {
    let starts: Vec<usize> = std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let at = |p: &Value| {
        let line = number(p, &["line"]) as usize;
        let start = starts.get(line).copied().unwrap_or(source.len());
        (start + number(p, &["character"]) as usize).min(source.len())
    };
    let mut spans: Vec<(usize, usize, &str)> = edits
        .iter()
        .map(|e| {
            (
                at(e.at(&["range", "start"])),
                at(e.at(&["range", "end"])),
                e.at(&["newText"]).as_str().unwrap_or(""),
            )
        })
        .collect();
    spans.sort_by_key(|s| std::cmp::Reverse(s.0));
    let mut out = source.to_string();
    for (start, end, text) in spans {
        out.replace_range(start..end, text);
    }
    out
}

/// Where two JSON values first differ, as a path, with both sides there.
fn first_mismatch(a: &Value, b: &Value, at: String) -> Option<String> {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            let keys: std::collections::BTreeSet<&String> = x.keys().chain(y.keys()).collect();
            keys.into_iter().find_map(|k| {
                let none = Value::Null;
                first_mismatch(
                    x.get(k).unwrap_or(&none),
                    y.get(k).unwrap_or(&none),
                    format!("{at}.{k}"),
                )
            })
        }
        (Value::Array(x), Value::Array(y)) => x
            .iter()
            .zip(y)
            .enumerate()
            .find_map(|(i, (p, q))| first_mismatch(p, q, format!("{at}[{i}]")))
            .or_else(|| {
                (x.len() != y.len()).then(|| format!("{at}: {} items vs {}", x.len(), y.len()))
            }),
        _ if a == b => None,
        _ => {
            let short = |v: &Value| v.to_string().chars().take(200).collect::<String>();
            Some(format!("{at}: {} vs {}", short(a), short(b)))
        }
    }
}

/// The browser's stress is the checked build `cpl stress` is: on the
/// overflow witness of `spec/lints/int_overflow_at_bounds`, both fail the
/// case as RE, as `cpl run` traps with R0202, instead of wrapping.
#[test]
fn stress_traps_an_overflow_as_cpl_stress_does() {
    let dir = std::env::temp_dir().join(format!("cpl-wasm-overflow-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let source =
        "let a: 0..=1000000000000 = read()\nlet b: 0..=1000000000 = read()\nprint(a * b)\n\n\
                  @gen\nfn at_bounds():\n    print(9223372037)\n    print(1000000000)\n";
    let path = dir.join("overflow.cp");
    std::fs::write(&path, source).unwrap();
    let name = path.to_str().unwrap();
    let cli = cpl_cli::run(&[
        "stress", name, "--cases", "1", "--seed", "1", "--format", "json",
    ]);
    assert_eq!(cli.code, 1, "{}", cli.stderr);
    let cli_json = String::from_utf8(cli.stdout).unwrap();
    let wasm_json = Playground::new().stress(name, source, 1, 1, 0);
    for json in [&cli_json, &wasm_json] {
        assert!(json.contains("\"verdict\":\"RE\""), "no RE in {json}");
    }
}
