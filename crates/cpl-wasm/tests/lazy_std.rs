//! The std of a browser build: it carries `prelude` and `mod`, and the host
//! gives it the other modules (`std_needed`, then `provide_std`). One test,
//! since it makes this whole test process carry only what the browser does.

use cpl_json::{obj, Value};
use cpl_wasm::Playground;
use std::path::Path;

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

fn has(value: &Value, text: &str) -> bool {
    value.to_string().contains(text)
}

fn open_or_change(uri: &str, version: i64, text: &str) -> String {
    let doc = obj([
        ("uri", uri.into()),
        ("languageId", "cpl".into()),
        ("version", version.into()),
    ]);
    let (method, params) = match version {
        1 => {
            let Value::Object(mut doc) = doc else {
                unreachable!()
            };
            doc.insert("text".to_string(), text.into());
            (
                "textDocument/didOpen",
                obj([("textDocument", Value::Object(doc))]),
            )
        }
        _ => (
            "textDocument/didChange",
            obj([
                ("textDocument", doc),
                (
                    "contentChanges",
                    Value::Array(vec![obj([("text", text.into())])]),
                ),
            ]),
        ),
    };
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", method.into()),
        ("params", params),
    ])
    .to_string()
}

#[test]
fn a_program_checks_and_runs_once_the_host_gives_the_std_it_needs() {
    cpl_check::carry_only_embedded_std();
    let programs = root().join("crates/cpl-cli/tests/programs");
    let source = std::fs::read_to_string(programs.join("batch33_splitmix64.cp")).unwrap();
    let input = std::fs::read(programs.join("batch33_splitmix64.in")).unwrap();
    let expected = std::fs::read_to_string(programs.join("batch33_splitmix64.out")).unwrap();
    let mut playground = Playground::new();
    playground.lsp(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#);
    let uri = "file:///main.cp";
    let opened = playground.lsp(&open_or_change(uri, 1, &source));
    assert!(
        has(&opened, "std module `std.math` is not loaded yet"),
        "{opened}"
    );

    let before = playground.check("main.cp", &source);
    assert!(
        has(&before, "std module `std.math` is not loaded yet"),
        "{before}"
    );
    assert_eq!(before.at(&["needs"]).to_string(), r#"["math","range"]"#);

    let hover = playground.lsp(r#"{"jsonrpc":"2.0","id":2,"method":"textDocument/hover","params":{"textDocument":{"uri":"file:///main.cp"},"position":{"line":5,"character":10}}}"#);
    assert!(has(&hover, "fn splitmix64(x: U64) -> U64"), "{hover}");

    // `.` offers the methods of a module not given yet; its docs come with
    // `completionItem/resolve` once it is.
    let mut offered_items = Vec::new();
    for (text, line, character, method, module) in [
        (
            "let m: ?Int = None\nprint(m.)\n",
            1,
            8,
            "is_some_and",
            "opt",
        ),
        (
            "let xs = [3, 1]\nprint(xs.sorted().sc)\n",
            1,
            20,
            "scan",
            "prefix",
        ),
    ] {
        playground.lsp(&open_or_change("file:///dot.cp", 1, text));
        let asked = format!(
            r#"{{"jsonrpc":"2.0","id":3,"method":"textDocument/completion","params":{{"textDocument":{{"uri":"file:///dot.cp"}},"position":{{"line":{line},"character":{character}}}}}}}"#
        );
        let offered = playground.lsp(&asked);
        let item = offered.as_array().unwrap()[0]
            .at(&["result", "items"])
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item.at(&["label"]).as_str() == Some(method))
            .unwrap_or_else(|| panic!("{offered}"))
            .clone();
        assert_eq!(
            item.at(&["data", "module"]).as_str(),
            Some(module),
            "{item}"
        );
        assert_eq!(item.at(&["documentation"]), &Value::Null, "{item}");
        offered_items.push((item, module));
    }

    let needed = playground.std_needed("main.cp", &source);
    assert!(has(&needed, r#""math""#), "{needed}");
    let mut given = Vec::new();
    loop {
        let needed = playground.std_needed("main.cp", &source);
        let needed: Vec<String> = needed
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|m| m.as_str().unwrap_or_default().to_string())
            .collect();
        if needed.is_empty() {
            break;
        }
        for module in needed {
            let text = std::fs::read_to_string(root().join(format!("std/{module}.cp"))).unwrap();
            assert_eq!(
                playground.provide_std(&module, &text).to_string(),
                r#"{"changed":true}"#
            );
            given.push((module, text));
        }
    }
    for (item, module) in offered_items {
        let text = std::fs::read_to_string(root().join(format!("std/{module}.cp"))).unwrap();
        playground.provide_std(module, &text);
        let resolve = obj([
            ("jsonrpc", "2.0".into()),
            ("id", 4.into()),
            ("method", "completionItem/resolve".into()),
            ("params", item),
        ]);
        let resolved = playground.lsp(&resolve.to_string());
        let doc = resolved.as_array().unwrap()[0].at(&["result", "documentation", "value"]);
        assert!(doc.as_str().is_some_and(|d| !d.is_empty()), "{resolved}");
    }
    for (module, text) in &given {
        let again = playground.provide_std(module, text);
        assert_eq!(again.to_string(), r#"{"changed":false}"#, "{module}");
    }
    let unknown = playground.provide_std("no_such", "");
    assert!(has(&unknown, "no module `std.no_such`"), "{unknown}");
    assert!(has(
        &playground.provide_std("prelude", ""),
        r#""changed":false"#
    ));

    let after = playground.check("main.cp", &source);
    assert_eq!(after.at(&["ok"]), &Value::Bool(true), "{after}");
    assert_eq!(after.at(&["needs"]), &Value::Null);
    let run = playground.run("main.cp", &source, &input);
    assert_eq!(String::from_utf8_lossy(&run.stdout), expected);
    let changed = playground.lsp(&open_or_change(uri, 2, &source));
    assert!(!has(&changed, "not loaded yet"), "{changed}");
}
