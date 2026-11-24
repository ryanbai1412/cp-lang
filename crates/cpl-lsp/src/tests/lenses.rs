//! Code lenses: run, trace, test and stress, and Replay while debugging.

use super::*;

#[test]
fn replay_lenses_sit_above_each_solution_fn_when_asked() {
    let uri = "file:///tmp/replay.cp";
    let src = "fn most() -> Int:\n    return 1\n\n@alt(program)\nfn slow():\n    print(1)\nprint(most())\n";
    let replays = |options: Value| -> Vec<(u64, String)> {
        let mut server = Server::new();
        server.handle(&request(
            1,
            "initialize",
            obj([("initializationOptions", options)]),
        ));
        server.handle(&notification("initialized", obj([])));
        let doc = obj([
            ("uri", uri.into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", src.into()),
        ]);
        server.handle(&notification(
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        let reply = server.handle(&request(
            2,
            "textDocument/codeLens",
            obj([("textDocument", obj([("uri", uri.into())]))]),
        ));
        reply[0]
            .at(&["result"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .filter(|l| l.at(&["command", "command"]).as_str() == Some("cp.replayCall"))
            .map(|l| {
                let args = l.at(&["command", "arguments"]).as_array().unwrap();
                assert_eq!(args[0].as_str(), Some(uri));
                (
                    l.at(&["range", "start", "line"]).as_u64().unwrap(),
                    args[1].at(&["fn"]).as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    assert_eq!(replays(obj([])), vec![]);
    assert_eq!(
        replays(obj([("replayLenses", true.into())])),
        vec![(0, "most".to_string())]
    );
}

#[test]
fn code_lenses_run_trace_test_and_stress_with_the_first_sample_when_asked() {
    let dir = std::env::temp_dir().join(format!("cpl-lens-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("sum two.tests")).unwrap();
    std::fs::write(dir.join("sum two.tests/2.in"), "1 2\n").unwrap();
    std::fs::write(dir.join("sum two.tests/10.in"), "3 4\n").unwrap();
    let file = dir.join("sum two.cp");
    let path = file.display().to_string();
    let uri = format!("file://{}", path.replace(' ', "%20"));
    let src = "let (a, b): (Int, Int) = read()\nprint(a + b)\n@alt(program)\nfn slow() = 0\n";
    let lenses = |options: Value| -> Vec<(u64, String, Vec<String>)> {
        let mut server = Server::new();
        server.handle(&request(
            1,
            "initialize",
            obj([("initializationOptions", options)]),
        ));
        server.handle(&notification("initialized", obj([])));
        server.handle(&notification(
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", src.into()),
                ]),
            )]),
        ));
        let reply = server.handle(&request(
            2,
            "textDocument/codeLens",
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        reply[0]
            .at(&["result"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|l| {
                assert_eq!(l.at(&["command", "command"]).as_str(), Some("cpl.exec"));
                (
                    l.at(&["range", "start", "line"]).as_u64().unwrap(),
                    l.at(&["command", "title"]).as_str().unwrap().to_string(),
                    l.at(&["command", "arguments"])
                        .as_array()
                        .unwrap_or(&[])
                        .iter()
                        .map(|a| a.as_str().unwrap().to_string())
                        .collect(),
                )
            })
            .collect()
    };
    assert!(
        lenses(Value::Null).is_empty(),
        "VS Code draws its own lenses"
    );
    let input = dir.join("sum two.tests/2.in").display().to_string();
    let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    assert_eq!(
        lenses(obj([("codeLens", true.into())])),
        [
            (0, "Run on 2".to_string(), s(&["run", &path, &input])),
            (
                0,
                "Run on all 2 samples".to_string(),
                s(&["run", &path, "--samples"])
            ),
            (
                0,
                "Trace on 2".to_string(),
                s(&["run", &path, &input, "--trace"])
            ),
            (0, "Test".to_string(), s(&["test", &path])),
            (0, "Stress".to_string(), s(&["stress", &path])),
            (
                2,
                "Stress the program against slow".to_string(),
                s(&["stress", &path])
            ),
            (
                2,
                "Trace".to_string(),
                s(&["run", &path, &input, "--trace"])
            ),
        ]
    );
    std::fs::remove_dir_all(dir).unwrap();
}
