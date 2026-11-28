//! Every tool answers one snapshot the same way.
//!
//! Contract 9 (`handbook/design/next/contracts/09-facade.md`): a tool checks
//! and runs a program through `cpl-facade` and nothing else, so for one
//! snapshot the facade, `cpl`, the language server and the playground give
//! the same diagnostics and the same output. What they must all give is
//! what `spec/` says, written here by hand, not what one of them gave.

use cpl_facade::{Job, Layer, Request, RunInput, Session, Severity, Snapshot, World};
use cpl_json::Value;
use cpl_wasm::Playground;
use std::path::Path;

/// One diagnostic as every tool states it: code, whether it is an error,
/// 0-based line, 0-based column, and the message with its help, as the
/// language server words the two (its protocol has one text).
type Said = (String, bool, u32, u32, String);

struct Case {
    name: &'static str,
    source: &'static str,
    /// `spec/diagnostics.md`'s code at 1-based line and column.
    expect: &'static [(&'static str, u32, u32)],
}

const CASES: &[Case] = &[
    Case {
        name: "clean",
        source: "let n: Int = read()\nprint(n * 2)\n",
        expect: &[],
    },
    // spec/diagnostics.md: E0302 type_mismatch, "`x` is declared an Int, but `"abc"` is a Str".
    Case {
        name: "mismatch",
        source: "let n: Int = \"three\"\nprint(n)\n",
        expect: &[("E0302", 1, 14)],
    },
    // spec/diagnostics.md: E0201 unresolved_name.
    Case {
        name: "unknown",
        source: "let n: Int = read()\nprint(total)\n",
        expect: &[("E0201", 2, 7)],
    },
    // Both, in source order: one report per root cause.
    Case {
        name: "both",
        source: "let n: Int = \"three\"\nprint(n)\nprint(total)\n",
        expect: &[("E0302", 1, 14), ("E0201", 3, 7)],
    },
];

fn snapshot(path: &str, source: &str) -> Snapshot {
    Snapshot::new(World::Disk).with_file(path, source)
}

fn line_col(source: &str, offset: usize) -> (u32, u32) {
    let before = &source[..offset];
    let line = before.matches('\n').count();
    let col = offset - before.rfind('\n').map_or(0, |at| at + 1);
    (line as u32, col as u32)
}

fn worded(message: &str, help: Option<&str>) -> String {
    help.map_or(message.to_string(), |help| {
        format!("{message}\nhelp: {help}")
    })
}

fn from_facade(path: &str, source: &str) -> Vec<Said> {
    let session = Session::open(snapshot(path, source));
    let said = session.diagnostics(path, Layer::Built);
    said.iter()
        .map(|d| {
            let (line, col) = line_col(source, d.span.start as usize);
            let error = d.severity == Severity::Error;
            (
                d.code.0.to_string(),
                error,
                line,
                col,
                worded(&d.message, d.help.as_deref()),
            )
        })
        .collect()
}

fn text(value: &Value, path: &[&str]) -> String {
    let found = value.at(path).as_str();
    found
        .unwrap_or_else(|| panic!("{path:?} is not a string in {value}"))
        .to_string()
}

fn number(value: &Value, path: &[&str]) -> u32 {
    let found = value.at(path).as_u32();
    found.unwrap_or_else(|| panic!("{path:?} is not a number in {value}"))
}

/// `cpl check --json`'s shape: 1-based positions.
fn from_json(diagnostics: &Value) -> Vec<Said> {
    let all = diagnostics.as_array().expect("an array of diagnostics");
    all.iter()
        .map(|d| {
            (
                text(d, &["code"]),
                text(d, &["severity"]) == "error",
                number(d, &["start", "line"]) - 1,
                number(d, &["start", "col"]) - 1,
                worded(&text(d, &["message"]), d.at(&["help"]).as_str()),
            )
        })
        .collect()
}

fn from_cli(path: &str) -> Vec<Said> {
    let out = cpl_cli::run(&["check", path, "--json"]);
    let report =
        cpl_json::parse(&String::from_utf8_lossy(&out.stdout)).expect("JSON from cpl check");
    from_json(report.at(&["files"]).as_array().expect("files")[0].at(&["diagnostics"]))
}

/// The playground states a position as the editor does: a 0-based `range`.
fn from_playground(path: &str, source: &str) -> Vec<Said> {
    let check = Playground::new().check(path, source);
    let all = check
        .at(&["diagnostics"])
        .as_array()
        .expect("an array of diagnostics");
    all.iter()
        .map(|d| {
            (
                text(d, &["code"]),
                text(d, &["severity"]) == "error",
                number(d, &["range", "start", "line"]),
                number(d, &["range", "start", "character"]),
                worded(&text(d, &["message"]), d.at(&["help"]).as_str()),
            )
        })
        .collect()
}

fn obj<const N: usize>(members: [(&str, Value); N]) -> Value {
    Value::Object(
        members
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

/// What the language server publishes for the file on opening it. It also
/// publishes lints; what the compiler said is what has an `E` or `W` code.
fn from_lsp(path: &str, source: &str) -> Vec<Said> {
    let mut server = cpl_lsp::Server::new();
    let message = |method: &str, params: Value| {
        obj([
            ("jsonrpc", "2.0".into()),
            ("method", method.into()),
            ("params", params),
        ])
    };
    let mut start = message("initialize", obj([]));
    if let Value::Object(members) = &mut start {
        members.insert("id".to_string(), 1.into());
    }
    server.respond(&start);
    server.respond(&message("initialized", obj([])));
    let uri = format!("file://{path}");
    let doc = obj([
        ("uri", uri.as_str().into()),
        ("languageId", "cp".into()),
        ("version", 1.into()),
        ("text", source.into()),
    ]);
    let published = server.respond(&message(
        "textDocument/didOpen",
        obj([("textDocument", doc)]),
    ));
    let ours = published.iter().find(|m| {
        m.at(&["method"]).as_str() == Some("textDocument/publishDiagnostics")
            && m.at(&["params", "uri"]).as_str() == Some(&uri)
    });
    let ours = ours.unwrap_or_else(|| panic!("nothing published for {uri}: {published:?}"));
    let all = ours
        .at(&["params", "diagnostics"])
        .as_array()
        .unwrap_or(&[]);
    all.iter()
        .filter(|d| {
            d.at(&["code"])
                .as_str()
                .is_some_and(|c| c.starts_with(['E', 'W']))
        })
        .map(|d| {
            (
                text(d, &["code"]),
                number(d, &["severity"]) == 1,
                number(d, &["range", "start", "line"]),
                number(d, &["range", "start", "character"]),
                text(d, &["message"]),
            )
        })
        .collect()
}

fn scratch(test: &str, case: &str, source: &str) -> String {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(test);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    let path = dir.join(format!("{case}.cp"));
    std::fs::write(&path, source).expect("a writable scratch file");
    path.to_str().expect("a UTF-8 path").to_string()
}

#[test]
fn every_tool_gives_the_same_diagnostics_for_one_snapshot() {
    for case in CASES {
        let path = scratch("one-snapshot-check", case.name, case.source);
        let facade = from_facade(&path, case.source);
        let stated: Vec<_> = facade
            .iter()
            .map(|(code, _, line, col, _)| (code.as_str(), line + 1, col + 1))
            .collect();
        assert_eq!(
            stated, case.expect,
            "{}: the facade against spec/diagnostics.md",
            case.name
        );
        assert!(
            facade.iter().all(|said| said.1),
            "{}: each is an error",
            case.name
        );
        assert_eq!(from_cli(&path), facade, "{}: cpl check", case.name);
        assert_eq!(
            from_playground(&path, case.source),
            facade,
            "{}: the playground",
            case.name
        );
        assert_eq!(
            from_lsp(&path, case.source),
            facade,
            "{}: the language server",
            case.name
        );
    }
}

/// spec/core/10-io.md: `read()` takes whitespace-separated tokens and
/// `print` writes its value and a newline.
const RUNS: &[(&str, &str, &[u8], &[u8])] = &[
    (
        "double",
        "let n: Int = read()\nprint(n * 2)\n",
        b"21\n",
        b"42\n",
    ),
    (
        "sum",
        "let (a, b): (Int, Int) = read()\nprint(a + b)\nprint(a - b)\n",
        b"7 5\n",
        b"12\n2\n",
    ),
];

#[test]
fn every_tool_gives_the_same_output_for_one_snapshot() {
    for &(name, source, input, expect) in RUNS {
        let path = scratch("one-snapshot-run", name, source);
        let session = Session::open(snapshot(&path, source));
        let program = session
            .program(path.as_str(), &Job::program())
            .expect("it builds");
        let mut printed = Vec::new();
        let outcome = session.run(
            &program,
            RunInput::fixed(input),
            &mut printed,
            Request::default(),
        );
        assert!(outcome.trap.is_none(), "{name}: the facade's run trapped");
        assert_eq!(
            printed, expect,
            "{name}: the facade against spec/core/10-io.md"
        );

        let cli = cpl_cli::run_with_input(&["run", &path], input);
        assert_eq!(
            (cli.stdout.as_slice(), cli.code),
            (expect, 0),
            "{name}: cpl run"
        );
        let browser = Playground::new().run(&path, source, input);
        assert_eq!(browser.stdout, expect, "{name}: the playground");
        assert_eq!(
            browser.result.at(&["exitCode"]).as_u32(),
            Some(0),
            "{name}: the playground"
        );
    }
}

/// Contract 0: equal snapshots give byte-identical answers, whichever
/// session answers and whatever it answered before.
#[test]
fn equal_snapshots_give_identical_answers() {
    for case in CASES {
        let path = format!("/nowhere/{}.cp", case.name);
        let first = Session::open(snapshot(&path, case.source));
        // A session that held another text first, then this one.
        let mut second = Session::open(snapshot(&path, "print(1)\n"));
        second.diagnostics(path.as_str(), Layer::Built);
        second = second.with_file(path.as_str(), case.source);
        assert_eq!(first.snapshot(), second.snapshot(), "{}", case.name);
        for layer in [Layer::Typed, Layer::Program, Layer::Built] {
            let (a, b) = (
                first.diagnostics(path.as_str(), layer),
                second.diagnostics(path.as_str(), layer),
            );
            assert_eq!(format!("{a:?}"), format!("{b:?}"), "{}", case.name);
        }
        let cpp = |session: &Session| {
            let program = session.program(path.as_str(), &Job::program()).ok()?;
            Some(
                session
                    .emit_cpp(&program, "main.cp", &cpl_facade::Profile::default())
                    .text,
            )
        };
        assert_eq!(cpp(&first), cpp(&second), "{}: the C++", case.name);
        assert_eq!(
            cpp(&first).is_some(),
            case.expect.is_empty(),
            "{}",
            case.name
        );
    }
}

/// A snapshot is a value: changing a file makes a new one and leaves the
/// old one, and the session opened on it, answering as before.
#[test]
fn a_snapshot_never_changes_under_a_session() {
    let old = snapshot("/nowhere/a.cp", "print(total)\n");
    let session = Session::open(old.clone());
    let new = old.with_file("/nowhere/a.cp", "print(1)\n");
    assert_ne!(old, new);
    let codes = |session: &Session| {
        let said = session.diagnostics("/nowhere/a.cp", Layer::Built);
        said.iter()
            .map(|d| d.code.0.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(codes(&session), ["E0201"]);
    assert_eq!(codes(&Session::open(new)), [""; 0]);
    assert_eq!(
        codes(&session.with_file("/nowhere/a.cp", "print(1)\n")),
        [""; 0]
    );
    assert_eq!(
        codes(&session),
        ["E0201"],
        "the first session still holds the old snapshot"
    );
}
