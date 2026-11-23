//! Type definition, implementations, `use` links and pulled diagnostics.

use super::*;

const ROADS: &str = "\
struct Road:
    to: Int
trait Sized:
    fn size(self) -> Int
impl Sized for Road:
    fn size(self) -> Int = self.to
impl Road:
    fn twice(self) -> Int = self.to * 2
fn far(r: Road) -> Int = r.size()
print(far(Road(to: 3)))
";

fn at(server: &mut Server, method: &str, line: u32, character: u32) -> Value {
    let reply = server.handle(&request(
        7,
        method,
        obj([("textDocument", doc()), ("position", pos(line, character))]),
    ));
    reply[0].at(&["result"]).clone()
}

fn starts(locations: &Value) -> Vec<Value> {
    let locations = locations.as_array().unwrap_or(&[]);
    locations
        .iter()
        .map(|l| l.at(&["range", "start"]).clone())
        .collect()
}

#[test]
fn type_definition_jumps_from_a_value_to_its_struct() {
    let (mut server, diagnostics) = open(ROADS);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let def = at(&mut server, "textDocument/typeDefinition", 8, 25);
    assert_eq!(def.at(&["uri"]).as_str(), Some(URI));
    assert_eq!(def.at(&["range", "start"]), &pos(0, 7));
    assert_eq!(def.at(&["range", "end"]), &pos(0, 11));
    // A value of a built-in type has no declaration to jump to.
    let (mut server, _) = open("let n = 3\nprint(n)\n");
    assert_eq!(
        at(&mut server, "textDocument/typeDefinition", 1, 6),
        Value::Null
    );
}

#[test]
fn type_definition_jumps_from_a_record_value_to_the_alias_naming_it() {
    let (mut server, diagnostics) =
        open("type P = (x: Int, y: Int)\nlet p: P = (x: 1, y: 2)\nlet q = p\nprint(q.x)\n");
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for (line, character) in [(1, 4), (2, 4), (3, 6)] {
        let def = at(&mut server, "textDocument/typeDefinition", line, character);
        assert_eq!(
            def.at(&["range", "start"]),
            &pos(0, 5),
            "at {line}:{character}"
        );
        assert_eq!(
            def.at(&["range", "end"]),
            &pos(0, 6),
            "at {line}:{character}"
        );
    }
}

#[test]
fn implementation_lists_every_impl_of_a_type_or_a_trait() {
    let (mut server, _) = open(ROADS);
    let road = at(&mut server, "textDocument/implementation", 0, 8);
    assert_eq!(starts(&road), [pos(4, 5), pos(6, 5)]);
    let sized = at(&mut server, "textDocument/implementation", 2, 8);
    assert_eq!(starts(&sized), [pos(4, 5)]);
}

#[test]
fn use_lines_link_to_the_module_file() {
    let (mut server, _) = open("use std.mod.{Mod}\nprint(Mod<7>(3).v)\n");
    let reply = server.handle(&request(
        7,
        "textDocument/documentLink",
        obj([("textDocument", doc())]),
    ));
    let links = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    assert_eq!(links.len(), 1, "{links:?}");
    assert_eq!(links[0].at(&["range", "start"]), &pos(0, 4));
    assert_eq!(links[0].at(&["range", "end"]), &pos(0, 11));
    let target = links[0].at(&["target"]).as_str().unwrap_or("");
    assert!(target.ends_with("std/mod.cp"), "{target}");
}

#[test]
fn diagnostics_can_be_pulled_too() {
    let (mut server, pushed) = open("print(total)\n");
    let reply = server.handle(&request(
        7,
        "textDocument/diagnostic",
        obj([("textDocument", doc())]),
    ));
    assert_eq!(reply[0].at(&["result", "kind"]).as_str(), Some("full"));
    let pulled = reply[0].at(&["result", "items"]).as_array().unwrap_or(&[]);
    assert_eq!(pulled.len(), pushed.len());
    assert_eq!(pulled[0].at(&["code"]), pushed[0].at(&["code"]));
}

const CALLS: &str = "\
fn sq(x: Int) -> Int = x * x
fn twice(x: Int) -> Int = sq(x) + sq(x)
fn solve() -> Int = twice(3)
print(solve() + sq(2))
";

fn call_item(server: &mut Server, line: u32, character: u32) -> Value {
    let items = at(server, "textDocument/prepareCallHierarchy", line, character);
    let items = items.as_array().unwrap_or(&[]).to_vec();
    assert_eq!(items.len(), 1, "{items:?}");
    items[0].clone()
}

fn calls(server: &mut Server, method: &str, item: &Value) -> Vec<(String, usize)> {
    let reply = server.handle(&request(8, method, obj([("item", item.clone())])));
    let side = if method.ends_with("incomingCalls") {
        "from"
    } else {
        "to"
    };
    reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|call| {
            let name = call.at(&[side, "name"]).as_str().unwrap_or("").to_string();
            (
                name,
                call.at(&["fromRanges"])
                    .as_array()
                    .map_or(0, <[Value]>::len),
            )
        })
        .collect()
}

#[test]
fn call_hierarchy_shows_callers_and_callees() {
    let (mut server, diagnostics) = open(CALLS);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let sq = call_item(&mut server, 0, 4);
    assert_eq!(sq.at(&["name"]).as_str(), Some("sq"));
    let incoming = calls(&mut server, "callHierarchy/incomingCalls", &sq);
    assert_eq!(
        incoming,
        [("twice".to_string(), 2), ("top level".to_string(), 1)]
    );
    // From a call site too.
    let twice = call_item(&mut server, 2, 21);
    assert_eq!(twice.at(&["name"]).as_str(), Some("twice"));
    let outgoing = calls(&mut server, "callHierarchy/outgoingCalls", &twice);
    assert_eq!(outgoing, [("sq".to_string(), 2)]);
    // A value is not a call hierarchy item.
    let items = at(&mut server, "textDocument/prepareCallHierarchy", 0, 23);
    assert_eq!(items, Value::Null);
}

#[test]
fn outgoing_calls_include_method_calls() {
    let (mut server, diagnostics) = open(
        "struct Road:\n    to: Int\nimpl Road:\n    fn twice(self) -> Int = self.to * 2\nfn far(r: Road) -> Int = r.twice()\nprint(far(Road(to: 3)))\n",
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let far = call_item(&mut server, 4, 4);
    let outgoing = calls(&mut server, "callHierarchy/outgoingCalls", &far);
    assert_eq!(outgoing, [("twice".to_string(), 1)]);
}

#[test]
fn organize_imports_is_a_source_action() {
    let (mut server, _) =
        open("use std.rand.{Rng}\nuse std.deque.{Deque}\nlet d = Deque<Int>()\nprint(d.len())\n");
    let params = obj([
        ("textDocument", doc()),
        ("range", obj([("start", pos(0, 0)), ("end", pos(0, 0))])),
        (
            "context",
            obj([
                ("diagnostics", Vec::<Value>::new().into()),
                ("only", vec!["source.organizeImports".into()].into()),
            ]),
        ),
    ]);
    let reply = server.handle(&request(9, "textDocument/codeAction", params));
    let actions = reply[0].at(&["result"]).as_array().unwrap_or(&[]).to_vec();
    let [organize] = actions.as_slice() else {
        panic!("{actions:?}")
    };
    assert_eq!(
        organize.at(&["kind"]).as_str(),
        Some("source.organizeImports")
    );
    let edits = organize
        .at(&["edit", "changes", URI])
        .as_array()
        .unwrap_or(&[])
        .to_vec();
    assert_eq!(
        edits[0].at(&["newText"]).as_str(),
        Some("use std.deque.{Deque}\n")
    );
    assert_eq!(edits[0].at(&["range", "end"]), &pos(2, 0));
}

#[test]
fn linked_editing_covers_a_local_name_and_nothing_else() {
    let src = "fn f(n: Int) -> Int = n * n\nprint(f(2))\n";
    let (mut server, _) = open(src);
    let linked = at(&mut server, "textDocument/linkedEditingRange", 0, 5);
    let ranges = linked.at(&["ranges"]).as_array().unwrap_or(&[]).to_vec();
    let columns: Vec<_> = ranges
        .iter()
        .map(|r| r.at(&["start", "character"]).clone())
        .collect();
    assert_eq!(columns, vec![5.into(), 22.into(), 26.into()], "{linked:?}");
    // `print` is the prelude's: editing it here would not rename it.
    assert_eq!(
        at(&mut server, "textDocument/linkedEditingRange", 1, 2),
        Value::Null
    );
}

#[test]
fn semantic_tokens_delta_sends_one_splice() {
    let (mut server, _) = open("let a = 1\nlet b = a\n");
    let tokens = |server: &mut Server, previous: Option<&str>| {
        let reply = server.handle(&request(
            9,
            match previous {
                Some(_) => "textDocument/semanticTokens/full/delta",
                None => "textDocument/semanticTokens/full",
            },
            obj([
                ("textDocument", doc()),
                (
                    "previousResultId",
                    previous.map_or(Value::Null, Value::from),
                ),
            ]),
        ));
        reply[0].at(&["result"]).clone()
    };
    let full = tokens(&mut server, None);
    let id = full.at(&["resultId"]).as_str().unwrap_or("").to_string();
    server.handle(&did_change((1, 9), (1, 9), " + a"));
    let delta = tokens(&mut server, Some(&id));
    let edits = delta.at(&["edits"]).as_array().unwrap_or(&[]).to_vec();
    assert_eq!(edits.len(), 1, "{delta:?}");
    let mut data: Vec<Value> = full.at(&["data"]).as_array().unwrap_or(&[]).to_vec();
    let start = edits[0].at(&["start"]).as_i64().unwrap_or(0) as usize;
    let deleted = edits[0].at(&["deleteCount"]).as_i64().unwrap_or(0) as usize;
    let inserted = edits[0].at(&["data"]).as_array().unwrap_or(&[]).to_vec();
    data.splice(start..start + deleted, inserted);
    let fresh = tokens(&mut server, None);
    assert_eq!(Value::Array(data), fresh.at(&["data"]).clone());
    // A full request in between (VS Code's own, for a visible editor) leaves
    // the earlier id usable.
    let earlier = fresh.at(&["resultId"]).as_str().unwrap_or("").to_string();
    tokens(&mut server, None);
    assert!(
        tokens(&mut server, Some(&earlier))
            .at(&["edits"])
            .as_array()
            .is_some(),
        "a delta from the set before the last"
    );
    // An unknown previous id gets the whole set.
    assert!(tokens(&mut server, Some("nope"))
        .at(&["data"])
        .as_array()
        .is_some());
}

#[test]
fn a_token_edit_is_the_minimal_splice() {
    use crate::server::tokens::edit;
    assert_eq!(edit(&[1, 2, 3], &[1, 2, 3]), (3, 0, &[][..]));
    assert_eq!(edit(&[1, 2, 3], &[1, 9, 3]), (1, 1, &[9][..]));
    assert_eq!(edit(&[1, 2], &[1, 2, 2]), (2, 0, &[2][..]));
    assert_eq!(edit(&[2, 2, 2], &[2]), (1, 2, &[][..]));
    assert_eq!(edit(&[], &[4]), (0, 0, &[4][..]));
}

/// editors/lsp-matrix.json: every request the server answers, with what its
/// reply must say. Each editor's smoke test runs the same table.
#[test]
fn every_request_the_server_answers_is_in_the_editor_matrix() {
    const ALL: &[&str] = &[
        "textDocument/hover",
        "textDocument/completion",
        "completionItem/resolve",
        "textDocument/signatureHelp",
        "textDocument/declaration",
        "textDocument/definition",
        "textDocument/typeDefinition",
        "textDocument/implementation",
        "textDocument/references",
        "textDocument/documentHighlight",
        "textDocument/documentSymbol",
        "textDocument/codeAction",
        "codeAction/resolve",
        "textDocument/codeLens",
        "codeLens/resolve",
        "textDocument/documentLink",
        "documentLink/resolve",
        "textDocument/documentColor",
        "textDocument/colorPresentation",
        "textDocument/formatting",
        "textDocument/rangeFormatting",
        "textDocument/onTypeFormatting",
        "textDocument/rename",
        "textDocument/prepareRename",
        "textDocument/foldingRange",
        "textDocument/selectionRange",
        "textDocument/prepareCallHierarchy",
        "callHierarchy/incomingCalls",
        "callHierarchy/outgoingCalls",
        "textDocument/prepareTypeHierarchy",
        "typeHierarchy/supertypes",
        "typeHierarchy/subtypes",
        "textDocument/semanticTokens/full",
        "textDocument/semanticTokens/full/delta",
        "textDocument/semanticTokens/range",
        "textDocument/linkedEditingRange",
        "textDocument/moniker",
        "textDocument/inlayHint",
        "inlayHint/resolve",
        "textDocument/inlineValue",
        "textDocument/diagnostic",
        "workspace/diagnostic",
        "workspace/symbol",
        "workspaceSymbol/resolve",
        "workspace/executeCommand",
        "workspace/willCreateFiles",
        "workspace/willRenameFiles",
        "workspace/willDeleteFiles",
    ];
    let matrix = cpl_json::parse(include_str!("../../../../editors/lsp-matrix.json")).unwrap();
    let dir = std::env::temp_dir().join(format!("cpl-matrix-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let Value::Object(files) = matrix.at(&["files"]) else {
        panic!("no files")
    };
    for (name, body) in files {
        std::fs::write(dir.join(name), body.as_str().unwrap()).unwrap();
    }
    let path = dir.join("main.cp").display().to_string();
    let uri = crate::uri::uri_of_path(&path);
    let mut server = Server::new();
    server.handle(&request(1, "initialize", obj([])));
    server.handle(&notification("initialized", obj([])));
    let text = std::fs::read_to_string(dir.join("main.cp")).unwrap();
    server.handle(&notification(
        "textDocument/didOpen",
        obj([(
            "textDocument",
            obj([
                ("uri", uri.as_str().into()),
                ("languageId", "cp".into()),
                ("version", 1.into()),
                ("text", text.into()),
            ]),
        )]),
    ));
    let mut results: Vec<(String, Value)> = Vec::new();
    fn earlier(value: &Value, results: &[(String, Value)]) -> Value {
        match value {
            Value::String(s) if s.starts_with("$result:") => {
                let (method, key) = s["$result:".len()..].rsplit_once(':').unwrap();
                let found = &results.iter().rev().find(|(m, _)| m == method).unwrap().1;
                match key.parse::<usize>() {
                    Ok(i) => found.as_array().unwrap()[i].clone(),
                    Err(_) => found.at(&[key]).clone(),
                }
            }
            Value::Object(fields) => Value::Object(
                fields
                    .iter()
                    .map(|(k, v)| (k.clone(), earlier(v, results)))
                    .collect(),
            ),
            other => other.clone(),
        }
    }
    let cases = matrix.at(&["requests"]).as_array().unwrap();
    for (i, case) in cases.iter().enumerate() {
        let method = case.at(&["method"]).as_str().unwrap();
        let Value::Object(mut params) = earlier(case.at(&["params"]), &results) else {
            panic!("{method}: params are not an object")
        };
        params.insert("textDocument".into(), obj([("uri", uri.as_str().into())]));
        let reply = server.handle(&request(100 + i as u32, method, Value::Object(params)));
        let result = reply[0].at(&["result"]).clone();
        let got = result.to_string();
        let missing: Vec<&str> = case
            .at(&["expect"])
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .filter(|want| !got.contains(want))
            .collect();
        assert!(
            missing.is_empty(),
            "{method} lacks {missing:?}: {}",
            reply[0]
        );
        results.push((method.to_string(), result));
    }
    let tested: Vec<&str> = cases
        .iter()
        .filter_map(|c| c.at(&["method"]).as_str())
        .collect();
    for method in ALL {
        let reply = server.handle(&request(
            9,
            method,
            obj([("textDocument", obj([("uri", uri.as_str().into())]))]),
        ));
        let unsupported = reply[0].at(&["error", "code"]).as_i64() == Some(-32601);
        assert!(
            unsupported || tested.contains(method),
            "the server answers {method}, but editors/lsp-matrix.json has no case for it"
        );
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
