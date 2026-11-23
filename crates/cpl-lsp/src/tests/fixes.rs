//! Quick fixes: from diagnostics, lints and their `cpl fix` codemods.

use super::*;

#[test]
fn quick_fixes_come_from_the_diagnostics() {
    let (mut server, diagnostics) = open(&SOURCE.replace("print(total)", "print(totl)"));
    assert_eq!(diagnostics.len(), 1);
    let reply = server.handle(&request(
        5,
        "textDocument/codeAction",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(7, 6)), ("end", pos(7, 10))])),
            ("context", obj([("diagnostics", diagnostics.into())])),
        ]),
    ));
    let actions = reply[0].at(&["result"]).as_array().unwrap_or(&[]);
    assert_eq!(actions.len(), 1);
    assert_eq!(
        actions[0].at(&["title"]).as_str(),
        Some("change to `total`")
    );
    let edits = actions[0]
        .at(&["edit", "changes"])
        .at(&[URI])
        .as_array()
        .unwrap_or(&[]);
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some("total"));
    assert_eq!(edits[0].at(&["range", "start"]), &pos(7, 6));
    assert_eq!(edits[0].at(&["range", "end"]), &pos(7, 10));
}

/// The most frequent first errors over the broken ports
/// (`crates/cpl-cli/tests/recovery.rs` prints the table), each broken into
/// a real port and undone by its one-click fix.
#[test]
fn quick_fixes_undo_the_most_frequent_mistakes_in_a_real_port() {
    assert_fixes_restore(
        "cf2062g",
        &[
            // E0101: a call left open.
            (
                "let p: [I]I = read()",
                "let p: [I]I = read(",
                "E0101",
                "insert `)`",
            ),
            // E0103: a block without its colon.
            (
                "fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:",
                "fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>",
                "E0103",
                "insert `:`",
            ),
            // E0102: an operator typed twice.
            (
                "abs(i.ord - j.ord) + abs(",
                "abs(i.ord - j.ord) + * abs(",
                "E0102",
                "remove the extra `*`",
            ),
            // E0004: a stray character.
            ("fn sort_perm<D>", "fn $sort_perm<D>", "E0004", "remove `$`"),
        ],
    );
}

/// A stdlib name the port forgot to import is one quick fix away, and the
/// `use` lands where the port had it.
#[test]
fn quick_fixes_import_a_stdlib_name_where_the_file_imports_its_module() {
    assert_fixes_restore_src(
        "use std.math.{floor_sum}\nprint(floor_sum(4, 3, 2, 1))\n",
        &[(
            "use std.math.{floor_sum}\n",
            "",
            "E0201",
            "add `use std.math.{floor_sum}`",
        )],
    );
    assert_fixes_restore_src(
        "use std.math.{gcd, floor_sum}\nprint(gcd(6, 4) + floor_sum(4, 3, 2, 1))\n",
        &[(
            "{gcd, floor_sum}",
            "{gcd}",
            "E0201",
            "add `use std.math.{floor_sum}`",
        )],
    );
}

#[test]
fn completion_offers_unimported_stdlib_names_with_their_use() {
    let src = "use std.math.{gcd}\nprint(floor_s|)\n";
    let reply = ask_at("textDocument/completion", src);
    let items = reply.at(&["items"]).as_array().unwrap_or(&[]);
    let item = items
        .iter()
        .find(|i| i.at(&["label"]).as_str() == Some("floor_sum"))
        .unwrap_or_else(|| panic!("no floor_sum in {items:?}"));
    assert_eq!(
        item.at(&["labelDetails", "description"]).as_str(),
        Some("std.math")
    );
    assert!(item
        .at(&["detail"])
        .as_str()
        .unwrap()
        .starts_with("fn floor_sum"));
    let edits = item.at(&["additionalTextEdits"]).as_array().unwrap_or(&[]);
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some(", floor_sum"));
    assert_eq!(edits[0].at(&["range", "start", "line"]).as_u32(), Some(0));
    assert_eq!(
        edits[0].at(&["range", "start", "character"]).as_u32(),
        Some(17)
    );
    // An imported name, or one the prelude brings, needs no `use`.
    let src = "use std.math.{gcd}\nprint(gc|)\n";
    let reply = ask_at("textDocument/completion", src);
    let gcd: Vec<&Value> = reply
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter(|i| i.at(&["label"]).as_str() == Some("gcd"))
        .collect();
    assert_eq!(gcd.len(), 1);
    assert!(gcd[0]
        .at(&["additionalTextEdits"])
        .as_array()
        .unwrap_or(&[])
        .is_empty());
}

/// A closer dropped mid-line is put back where it was, before the `:` or
/// `=` it can't enclose, not at the end of the file.
#[test]
fn quick_fixes_close_a_bracket_where_it_was_dropped() {
    assert_fixes_restore(
        "cf2084c",
        &[(
            "target[mid] = I(n / 2 + 1)",
            "target[mid = I(n / 2 + 1)",
            "E0101",
            "insert `]`",
        )],
    );
    assert_fixes_restore(
        "cf2150c",
        &[(
            "if let (nxt_k, nxt_v) = deltas",
            "if let (nxt_k, nxt_v = deltas",
            "E0101",
            "insert `)`",
        )],
    );
}

#[test]
fn quick_fixes_cover_c_spellings_and_mutation() {
    let src = SOURCE.replace("var total", "let total").replace(
        "for v in V:",
        "for v in V:\n    if v > 1 && v < n:\n        total += 0",
    );
    let (mut server, diagnostics) = open(&src);
    let reply = server.handle(&request(
        6,
        "textDocument/codeAction",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(12, 0))])),
            ("context", obj([("diagnostics", diagnostics.into())])),
        ]),
    ));
    let titles: Vec<&str> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|a| a.at(&["title"]).as_str())
        .collect();
    assert_eq!(
        titles,
        vec!["replace with `and`", "declare `total` with `var`"]
    );
}

/// Each spelling the language replaced, put back into a real port, is
/// reported with the `cpl fix` codemod as its quick fix, and the fix
/// restores the port.
#[test]
fn replaced_spellings_get_their_cpl_fix_codemod_as_a_quick_fix() {
    for (port, good, bad, code) in [
        (
            "cf2066b",
            "in a.items if x != 0",
            "in a.items where x != 0",
            "E0503",
        ),
        ("agc077a", "@!multitest\n", "--- multitest ---\n", "E0503"),
        (
            "cf2062h",
            "Z(2) ** inside - ways",
            "Z(2).pow(inside) - ways",
            "L0205",
        ),
        ("cf2066b", "(0..b.len).rev:", "(0..b.len()).rev:", "L0206"),
        ("cf2053i1", "reach.start", "reach.lo", "E0503"),
    ] {
        let mut server = Server::new();
        let (uri, text, diagnostics) = open_port(&mut server, port, (good, bad));
        let old = diagnostics
            .iter()
            .find(|d| d.at(&["code"]).as_str() == Some(code))
            .unwrap_or_else(|| panic!("{port}: no {code} for `{bad}` in {diagnostics:?}"));
        let (_, fixed) = quick_fixed(&mut server, &uri, &text, old);
        assert_eq!(
            fixed,
            text.replacen(bad, good, 1),
            "{port}: the fix for `{bad}`"
        );
    }
}

/// Fix-all is `cpl fix`: the removed syntax first, which lets the file
/// parse, then the lints it uncovers.
#[test]
fn fix_all_rewrites_removed_syntax_then_the_lints_it_uncovers() {
    let mut server = Server::new();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/problems/cf2066b/port.cp");
    let original = std::fs::read_to_string(&path).unwrap();
    let (uri, text, _) = open_port(
        &mut server,
        "cf2066b",
        ("(0..b.len).rev:", "(0..b.len()).rev:"),
    );
    let text = text.replacen("in a.items if x != 0", "in a.items where x != 0", 1);
    let change = obj([
        (
            "textDocument",
            obj([("uri", uri.as_str().into()), ("version", 2.into())]),
        ),
        (
            "contentChanges",
            vec![obj([("text", text.as_str().into())])].into(),
        ),
    ]);
    server.handle(&notification("textDocument/didChange", change));
    let params = obj([
        ("textDocument", obj([("uri", uri.as_str().into())])),
        (
            "range",
            obj([
                ("start", obj([("line", 0.into()), ("character", 0.into())])),
                ("end", obj([("line", 0.into()), ("character", 0.into())])),
            ]),
        ),
        (
            "context",
            obj([
                ("diagnostics", Vec::<Value>::new().into()),
                ("only", vec!["source.fixAll".into()].into()),
            ]),
        ),
    ]);
    let reply = server.handle(&request(2, "textDocument/codeAction", params));
    let actions = reply[0].at(&["result"]).as_array().unwrap_or(&[]);
    let [fix_all] = actions else {
        panic!("{actions:?}")
    };
    assert_eq!(fix_all.at(&["kind"]).as_str(), Some("source.fixAll.cpl"));
    let edits = fix_all
        .at(&["edit", "changes", uri.as_str()])
        .as_array()
        .unwrap_or(&[]);
    assert_eq!(edits[0].at(&["newText"]).as_str(), Some(original.as_str()));
}
