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

    let _ = std::fs::remove_dir_all(&dir);
}

/// Every lint record's `before.cp` and `after.cp` lint in the playground
/// exactly as in `cpl lint` and the editor, which share `lint_analysis`.
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
