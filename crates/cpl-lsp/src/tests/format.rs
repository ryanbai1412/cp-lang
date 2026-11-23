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
