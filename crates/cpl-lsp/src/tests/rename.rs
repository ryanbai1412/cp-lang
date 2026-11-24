//! References and rename.

use super::*;

#[test]
fn references_find_locals_params_fns_structs_domains_and_fields() {
    // A local, and a shadowing one of the same name that is not it.
    let src = "fn f(n: Int) -> Int:\n    let x = n\n    let y = x| + x\n    let x = 5\n    x\n";
    assert_eq!(references_at(src), [(1, 8), (2, 12), (2, 16)]);
    // A parameter.
    assert_eq!(
        references_at("fn f(n|: Int) -> Int = n * n\n"),
        [(0, 5), (0, 22), (0, 26)]
    );
    // A fn, from a call.
    let src = "fn sq(n: Int) -> Int = n * n\nprint(sq(2) + s|q(3))\n";
    assert_eq!(references_at(src), [(0, 3), (1, 6), (1, 14)]);
    // A struct and a field, used through a value and inside a method.
    let src = format!("{ROAD}let r = Road(1, 2)\nprint(r.w|)\n");
    // The declaration and the access through a value.
    assert_eq!(references_at(&src), [(2, 4), (12, 8)]);
    let src = format!("{ROAD}let r = Ro|ad(1, 2)\n");
    assert_eq!(references_at(&src), [(0, 7), (10, 24), (10, 31), (11, 8)]);
    // A domain.
    let src = "domain V = 0..5\nvar a: [V]Int = 0\nfor v in V|:\n    a[v] = 1\n";
    assert_eq!(references_at(src), [(0, 7), (1, 8), (2, 9)]);
}

/// A local of the same name in a fn, a `for` or a lambda is another
/// binding, so references and rename leave it alone.
#[test]
fn references_skip_the_bindings_that_shadow_the_name() {
    let src = "let x| = 1\nfn f() -> Int:\n    let x = 2\n    x\nprint(x)\n";
    assert_eq!(references_at(src), [(0, 4), (4, 6)]);
    let src = "let x = 1\nfn f() -> Int:\n    let x| = 2\n    x\nprint(x)\n";
    assert_eq!(references_at(src), [(2, 8), (3, 4)]);
    let src = "let x = 1\nfor x| in 0..3:\n    print(x)\nprint(x)\n";
    assert_eq!(references_at(src), [(1, 4), (2, 10)]);
    let src = "let x| = 1\nlet f = (x) => x + 1\nprint(f(x))\n";
    assert_eq!(references_at(src), [(0, 4), (2, 8)]);
}

#[test]
fn rename_edits_every_reference_and_refuses_library_names() {
    let src = "fn sq(n: Int) -> Int = n * n\nprint(sq(2) + s|q(3))\n";
    let prepared = ask_at("textDocument/prepareRename", src);
    assert_eq!(prepared.at(&["placeholder"]).as_str(), Some("sq"));
    let edit = ask_at_with(
        "textDocument/rename",
        src,
        obj([("newName", "square".into())]),
    );
    let edits = edit.at(&["changes", URI]).as_array().unwrap_or(&[]);
    assert_eq!(starts(edits), [(0, 3), (1, 6), (1, 14)]);
    assert!(edits
        .iter()
        .all(|e| e.at(&["newText"]).as_str() == Some("square")));
    // Not a name, a keyword, or a name the prelude owns.
    let bad = ask_at_with("textDocument/rename", src, obj([("newName", "fn".into())]));
    assert!(
        bad.at(&["message"]).as_str().unwrap().contains("keyword"),
        "{bad:?}"
    );
    let bad = ask_at_with("textDocument/rename", src, obj([("newName", "a b".into())]));
    assert!(
        bad.at(&["message"])
            .as_str()
            .unwrap()
            .contains("not a name"),
        "{bad:?}"
    );
    let library = ask_at("textDocument/prepareRename", "print(gc|d(4, 6))\n");
    assert!(library.is_null(), "{library:?}");
    // `self` is the receiver's spelling, not a name the user chose.
    let method = "struct P(a: Int):\n    fn get(se|lf) -> Int = a\nprint(P(1).get())\n";
    let receiver = ask_at("textDocument/prepareRename", method);
    assert!(receiver.is_null(), "{receiver:?}");
    let receiver = ask_at_with(
        "textDocument/rename",
        method,
        obj([("newName", "me".into())]),
    );
    assert!(
        receiver
            .at(&["message"])
            .as_str()
            .is_some_and(|m| m.contains("receiver")),
        "{receiver:?}"
    );
}

#[test]
fn rename_reaches_every_open_file_and_the_library_that_declares_the_name() {
    let root = std::env::temp_dir().join(format!("cpl-rename-{}", std::process::id()));
    std::fs::create_dir_all(root.join("kit")).unwrap();
    std::fs::write(root.join("cp.toml"), "[libs]\nkit = \"kit\"\n").unwrap();
    let lib =
        "pub fn area(w: Int, h: Int) -> Int = w * h\npub fn square(s: Int) -> Int = area(s, s)\n";
    std::fs::write(root.join("kit/geo.cp"), lib).unwrap();
    let uri = |name: &str| format!("file://{}", root.join(name).display());
    let (a, b, geo) = (uri("a.cp"), uri("b.cp"), uri("kit/geo.cp"));
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    let open_at = |server: &mut Server, uri: &str, text: &str| {
        let doc = obj([
            ("uri", uri.into()),
            ("languageId", "cp".into()),
            ("version", 1.into()),
            ("text", text.into()),
        ]);
        let published = server.handle(&notification(
            "textDocument/didOpen",
            obj([("textDocument", doc)]),
        ));
        let diagnostics = published[0]
            .at(&["params", "diagnostics"])
            .as_array()
            .unwrap_or(&[]);
        assert!(diagnostics.is_empty(), "{published:?}");
    };
    open_at(&mut server, &a, "use kit.geo.{area}\nprint(area(2, 3))\n");
    open_at(
        &mut server,
        &b,
        "use kit.geo.{area}\nlet area2 = area(4, 4)\nprint(area(1, 1) + area2)\n",
    );
    let rename = |server: &mut Server, uri: &str, line: u32, col: u32| {
        let params = obj([
            ("textDocument", obj([("uri", uri.into())])),
            ("position", pos(line, col)),
            ("newName", "rect".into()),
        ]);
        server.handle(&request(2, "textDocument/rename", params))[0].clone()
    };
    let lines = |reply: &Value, uri: &str| -> Vec<(u32, u32)> {
        let edits = reply
            .at(&["result", "changes", uri])
            .as_array()
            .unwrap_or(&[]);
        starts(edits)
    };
    // From a use: the import and both calls here, the other open file, and
    // the library file, which is not open.
    let reply = rename(&mut server, &a, 1, 7);
    assert_eq!(lines(&reply, &a), [(0, 13), (1, 6)]);
    assert_eq!(lines(&reply, &b), [(0, 13), (1, 12), (2, 6)]);
    assert_eq!(lines(&reply, &geo), [(0, 7), (1, 31)]);
    // A file of the project that is not open, from the declaration too.
    std::fs::write(root.join("c.cp"), "use kit.geo.{area}\nprint(area(5, 5))\n").unwrap();
    std::fs::write(root.join("d.cp"), "fn area() -> Int = 1\nprint(area())\n").unwrap();
    let reply = rename(&mut server, &a, 1, 7);
    let c = root.join("c.cp").display().to_string();
    let c_uri = reply.at(&["result", "changes"]);
    let Value::Object(changed) = c_uri else {
        panic!("{reply:?}");
    };
    let c_key = changed
        .keys()
        .find(|k| k.ends_with("/c.cp"))
        .cloned()
        .unwrap_or_else(|| panic!("{c} not renamed: {reply:?}"));
    assert_eq!(lines(&reply, &c_key), [(0, 13), (1, 6)]);
    assert!(
        !changed.iter().any(|(k, _)| k.ends_with("/d.cp")),
        "{reply:?}"
    );
    std::fs::remove_file(root.join("c.cp")).unwrap();
    std::fs::remove_file(root.join("d.cp")).unwrap();
    // From the declaration, once the library is open too: each file once.
    open_at(&mut server, &geo, lib);
    let reply = rename(&mut server, &geo, 0, 8);
    assert_eq!(lines(&reply, &geo), [(0, 7), (1, 31)]);
    assert_eq!(lines(&reply, &a), [(0, 13), (1, 6)]);
    let Value::Object(changes) = reply.at(&["result", "changes"]) else {
        panic!("{reply:?}");
    };
    assert_eq!(changes.len(), 3, "{changes:?}");
    // A name the library already declares would collide.
    let params = obj([
        ("textDocument", obj([("uri", a.as_str().into())])),
        ("position", pos(1, 7)),
        ("newName", "square".into()),
    ]);
    let reply = server.handle(&request(3, "textDocument/rename", params));
    let message = reply[0].at(&["error", "message"]).as_str().unwrap_or("");
    assert!(
        message.contains("`square` is already a name in"),
        "{reply:?}"
    );
    std::fs::remove_dir_all(&root).unwrap();
    // The standard library's names stay its own.
    let (mut server, _) = open("use std.mod.{Comb}\nlet c = Comb<7>.new()\n");
    let params = obj([("textDocument", doc()), ("position", pos(1, 9))]);
    let reply = server.handle(&request(3, "textDocument/prepareRename", params));
    let message = reply[0].at(&["error", "message"]).as_str().unwrap_or("");
    assert!(message.contains("standard library"), "{reply:?}");
}

/// The new texts a rename at the cursor edits in, in source order.
fn renamed_texts(src: &str, to: &str) -> Vec<String> {
    let reply = ask_at_with("textDocument/rename", src, obj([("newName", to.into())]));
    let changes = reply
        .at(&["changes"])
        .as_object()
        .cloned()
        .unwrap_or_default();
    changes
        .values()
        .flat_map(|edits| edits.as_array().unwrap_or(&[]).to_vec())
        .map(|e| e.at(&["newText"]).as_str().unwrap_or("").to_string())
        .collect()
}

/// A shorthand argument keeps its label (`f(best:)` becomes
/// `f(best: top)`).
#[test]
fn rename_keeps_shorthand_argument_labels() {
    let src = "fn f(best: Int) -> Int = best + 1\nlet best| = 2\nprint(f(best:))\n";
    assert_eq!(renamed_texts(src, "top"), ["top", "best: top"]);
}

/// A variant `read` parses by its name keeps the name: the input spells it.
#[test]
fn rename_refuses_a_variant_the_input_spells() {
    let read = "enum Axis = |X | Y\nlet a = read<(d: Int, axis: Axis)>()\nprint(a.d)\n";
    let reply = ask_at_with("textDocument/prepareRename", read, obj([]));
    let why = reply.at(&["message"]).as_str().unwrap_or("");
    assert!(why.contains("how the input spells"), "{reply:?}");
    let unread = "enum Axis = |X | Y\nfn f(a: Axis) -> Int = 1\nprint(f(X))\n";
    assert_eq!(renamed_texts(unread, "Z"), ["Z", "Z"]);
}

/// A member (`s.len`, `dijkstra(..).dist`) is not the top-level value that
/// shares its name, so renaming the value leaves the member alone.
#[test]
fn rename_leaves_a_member_named_like_a_top_level_value() {
    let src = "let len| = 3\nlet s = \"ab\"\nprint(s.len + len)\n";
    assert_eq!(renamed_texts(src, "size"), ["size", "size"]);
    let src = "var dist| = 3\nlet g: [3]List<(to: Int, w: Int)> = [[], [], []]\ndist = dijkstra(g, from: [(0, 0)]).dist[0]!\nprint(dist)\n";
    assert_eq!(renamed_texts(src, "d"), ["d", "d", "d"]);
}

#[test]
fn rename_reaches_a_variant_named_through_its_enum() {
    let src = "enum Team = Ho|me | Away\nvar score: [Team]Int = 0\nscore[Team.Home] += 1\nprint(score[Team.Home])\n";
    assert_eq!(renamed_texts(src, "Host"), ["Host", "Host", "Host"]);
}

/// A domain a tuple pattern binds (`let (domain C, ids) = ..`) is declared
/// at its name, so its uses rename with it.
#[test]
fn rename_reaches_a_domain_a_tuple_pattern_binds() {
    let src = "let xs = [3, 1, 3]\nlet (domain C|, ids) = compress(xs)\nvar t: [C]Int = 0\nprint(t[ids[0]] + C.count)\n";
    assert_eq!(renamed_texts(src, "Rank"), ["Rank", "Rank", "Rank"]);
}

/// `g[v]` on a struct with an `at` method reads as `g.at(v)`, but the `v`
/// written there is still the local's.
#[test]
fn rename_reaches_an_index_into_a_struct_read_through_at() {
    let src = "domain V = 1..=3\nlet g = Graph.undirected([(u: V.first!, v: V.last!)])\nprint(V.find(v| => g[v].len > 1))\n";
    assert_eq!(renamed_texts(src, "w"), ["w", "w"]);
}
