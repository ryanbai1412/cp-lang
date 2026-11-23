//! Formatting: `cpl fmt` on the open buffer, in the file's `cp.toml` style.

use super::*;

#[test]
fn formatting_replaces_the_file_with_what_cpl_fmt_writes() {
    let (mut server, _) = open("let  x=1\nprint( x )\n");
    let reply = server.handle(&request(
        9,
        "textDocument/formatting",
        obj([("textDocument", doc())]),
    ));
    let edits = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    assert_eq!(edits.len(), 1);
    assert_eq!(
        edits[0].at(&["newText"]).as_str(),
        Some("let x = 1\nprint(x)\n")
    );
    assert_eq!(edits[0].at(&["range", "start", "line"]).as_u32(), Some(0));
    assert_eq!(edits[0].at(&["range", "end", "line"]).as_u32(), Some(2));

    // A file that is already formatted has nothing to change.
    let (mut server, _) = open("let x = 1\nprint(x)\n");
    let reply = server.handle(&request(
        9,
        "textDocument/formatting",
        obj([("textDocument", doc())]),
    ));
    assert!(reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .is_empty());
}

#[test]
fn formatting_follows_the_fmt_section_of_cp_toml() {
    let dir = std::env::temp_dir().join(format!("cpl-fmt-style-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let toml = "[fmt]\nindent_width = 2\n";
    std::fs::write(dir.join("cp.toml"), toml).unwrap();
    let uri = format!("file://{}", dir.join("a.cp").display());
    let src = "fn f() -> Int:\n    return 1\nprint(f())\n";
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
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
        "textDocument/formatting",
        obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
    ));
    let edits = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    let want = cpl_fmt::format_with(src, &cpl_fmt::Style::from_toml(toml).0).unwrap();
    assert_ne!(want, src);
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some(want.as_str()));
    std::fs::remove_dir_all(&dir).unwrap();
}

fn part(text: &str, method: &str, at: Value) -> Vec<Value> {
    let (mut server, _) = open(text);
    let key = if method == "textDocument/rangeFormatting" {
        "range"
    } else {
        "position"
    };
    let params = obj([("textDocument", doc()), (key, at)]);
    let reply = server.handle(&request(9, method, params));
    reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec()
}

#[test]
fn a_selection_formats_only_the_lines_it_touches() {
    let text = "let  a = 1\nlet b = 2\nprint( a+b )\n";
    let edits = part(
        text,
        "textDocument/rangeFormatting",
        obj([("start", pos(2, 0)), ("end", pos(2, 3))]),
    );
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(edits[0].at(&["range", "start"]), &pos(2, 0));
    assert_eq!(edits[0].at(&["range", "end"]), &pos(3, 0));
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some("print(a + b)\n"));
    // The first line is left as it is, unselected.
    let all = part(
        text,
        "textDocument/rangeFormatting",
        obj([("start", pos(0, 0)), ("end", pos(3, 0))]),
    );
    assert_eq!(all.len(), 2, "{all:?}");
}

#[test]
fn typing_a_newline_formats_the_line_just_finished() {
    let edits = part(
        "let  a = 1\nlet b = 2\nprint( a )\n",
        "textDocument/onTypeFormatting",
        pos(3, 0),
    );
    assert_eq!(edits.len(), 1, "{edits:?}");
    assert_eq!(edits[0].at(&["range", "start"]), &pos(2, 0));
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some("print(a)\n"));
}

#[test]
fn hunks_are_the_changed_runs_of_lines() {
    let old = "a\nb\nc\nd\ne\n";
    let new = "a\nB\nc\nd\nE\nf\n";
    let hunks = crate::server::format::hunks(old, new);
    let shown: Vec<_> = hunks
        .iter()
        .map(|(span, text)| (&old[span.start as usize..span.end as usize], text.as_str()))
        .collect();
    assert_eq!(shown, [("b\n", "B\n"), ("e\n", "E\nf\n")]);
}
