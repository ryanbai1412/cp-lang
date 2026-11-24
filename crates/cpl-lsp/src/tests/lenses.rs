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
    let lenses = |options: Value| lenses_of(&uri, src, options);
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

type Lens = (u64, String, Vec<String>);

/// The lenses the server sends for `src` open at `uri`, initialized with `options`.
fn lenses_of(uri: &str, src: &str, options: Value) -> Vec<Lens> {
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
                ("uri", uri.into()),
                ("languageId", "cp".into()),
                ("version", 1.into()),
                ("text", src.into()),
            ]),
        )]),
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
}

#[test]
fn lenses_run_the_cases_cpl_test_finds_in_each_layout() {
    // commands/R15: the user's own `a.tests/`, a sibling `tests/` as in
    // the corpus, `a.tests/` winning over `tests/`, and scratch inputs
    // with `a.samples/`. Each lens runs what `cpl_contest::problem` finds.
    let root = std::env::temp_dir().join(format!("cpl-lens-layouts-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let layouts: [(&str, &[&str], &[&str]); 4] = [
        (
            "own",
            &[
                "a.tests/2.in",
                "a.tests/10.in",
                "a.tests/10.out",
                "a.tests/big.in",
            ],
            &["2", "10", "big"],
        ),
        (
            "sibling",
            &["tests/1.in", "tests/1.out", "tests/3.in"],
            &["1", "3"],
        ),
        ("both", &["a.tests/1.in", "tests/7.in"], &["1"]),
        (
            "scratch",
            &["i2", "i1", "o1", "a.samples/1.in"],
            &["i1", "i2", "1"],
        ),
    ];
    let src = "let (a, b): (Int, Int) = read()\nprint(a + b)\n";
    for (layout, files, names) in layouts {
        let dir = root.join(layout);
        for f in files {
            let f = dir.join(f);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, "1 2\n").unwrap();
        }
        let file = dir.join("a.cp");
        std::fs::write(&file, src).unwrap();
        let samples = cpl_contest::problem::samples(&file);
        let found: Vec<&str> = samples.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(found, names, "{layout}");
        let path = file.display().to_string();
        let first = samples[0].input.display().to_string();
        let mut want: Vec<Lens> = vec![(
            0,
            format!("Run on {}", names[0]),
            vec!["run".into(), path.clone(), first.clone()],
        )];
        if samples.len() > 1 {
            let args = vec!["run".into(), path.clone(), "--samples".into()];
            want.push((0, format!("Run on all {} samples", samples.len()), args));
        }
        let trace = vec!["run".into(), path.clone(), first, "--trace".into()];
        want.push((0, format!("Trace on {}", names[0]), trace));
        if !cpl_contest::problem::cases(&file).is_empty() {
            want.push((0, "Test".into(), vec!["test".into(), path.clone()]));
        }
        assert_eq!(layout != "scratch", want.iter().any(|l| l.1 == "Test"));
        let uri = format!("file://{path}");
        assert_eq!(
            lenses_of(&uri, src, obj([("codeLens", true.into())])),
            want,
            "{layout}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn lenses_trace_each_fn_but_main_on_the_first_sample() {
    let dir = std::env::temp_dir().join(format!("cpl-lens-trace-fn-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("i1"), "3\n").unwrap();
    let path = dir.join("a.cp").display().to_string();
    let uri = format!("file://{path}");
    let input = dir.join("i1").display().to_string();
    let src = "fn dfs(v: Int) -> Int:\n    return v\n\nfn main():\n    print(dfs(read()))\n";
    let traces: Vec<Lens> = lenses_of(&uri, src, obj([("codeLens", true.into())]))
        .into_iter()
        .filter(|(_, title, _)| title.starts_with("Trace"))
        .collect();
    let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    assert_eq!(
        traces,
        [
            (
                0,
                "Trace on i1".to_string(),
                s(&["run", &path, &input, "--trace"])
            ),
            (
                0,
                "Trace".to_string(),
                s(&["run", &path, &input, "--trace", "dfs"])
            ),
        ]
    );
    std::fs::remove_dir_all(dir).unwrap();
}
