//! Completion in every context, one case per context the completion use
//! cases name (`handbook/product/editor/completion-usecases.md`): after
//! `.`, argument names, names in scope, `use` paths, types and patterns.
//! Each case is a document with the cursor at `$0` and the labels
//! completion must offer there; an empty list means "offers something".

use cpl_json::{obj, Value};
use cpl_lsp::Server;

fn message(id: Option<u32>, method: &str, params: Value) -> Value {
    let mut members = vec![("jsonrpc".to_string(), Value::from("2.0"))];
    if let Some(id) = id {
        members.push(("id".to_string(), id.into()));
    }
    members.push(("method".to_string(), method.into()));
    members.push(("params".to_string(), params));
    Value::Object(members.into_iter().collect())
}

fn position(src: &str, offset: usize) -> Value {
    let line_start = src[..offset].rfind('\n').map_or(0, |i| i + 1);
    let line = src[..offset].matches('\n').count() as u32;
    let col = src[line_start..offset].encode_utf16().count() as u32;
    obj([("line", line.into()), ("character", col.into())])
}

const CASES: &[(&str, &str, &[&str])] = &[
    ("dot/field", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(e.$0)", &["to","w"]),
    ("dot/list", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.$0)", &["len"]),
    ("dot/prefix", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.so$0)", &["sorted"]),
    ("dot/after-call", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(runs(\"aab\").$0)", &[]),
    ("dot/after-index", "let g: List<List<Int>> = [[1]]\nprint(g[0].$0)", &["len"]),
    ("dot/chain", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.sorted().$0)", &["len"]),
    ("dot/loop-var", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nfor x in es:\n    print(x.$0)", &["to"]),
    ("dot/lambda-param", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(es.map(x => x.$0))", &["to"]),
    ("dot/lambda-param-int", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.map(x => x.$0))", &[]),
    ("dot/self", "struct P:\n    x: Int\n    fn get(self) -> Int = self.$0\n", &["x"]),
    ("dot/self-block", "struct P:\n    x: Int\n    fn get(self) -> Int:\n        self.$0\n", &["x"]),
    ("dot/optional", "let m: ?Int = None\nprint(m.$0)", &[]),
    ("dot/tuple", "let p = (1, 2)\nprint(p.$0)", &[]),
    ("dot/map", "var m: Map<Int, Int> = {}\nm.$0", &[]),
    ("dot/str", "let s = \"ab\"\nprint(s.$0)", &["len"]),
    ("dot/int-range", "let n: 1..=10 = read()\nprint(n.$0)", &[]),
    ("dot/std-type", "use std.segtree.*\nlet t = SegTree(min, [3, 1, 2])\nprint(t.$0)", &["fold"]),
    ("dot/static", "use std.graph.{Graph}\nlet g = Graph.$0", &["undirected"]),
    ("dot/enum", "enum Dir:\n    Up\n    Down\nlet d = Dir.$0", &["Up","Down"]),
    ("dot/domain", "domain V = 0..5\nprint(V.$0)", &[]),
    ("dot/incomplete-after", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nlet y = xs.$0\nlet z = \n", &["len"]),
    ("dot/broken-before", "fn f(:\nstruct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.$0)", &["len"]),
    ("dot/multiline-call", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(\n    xs.$0\n)", &["len"]),
    ("dot/fstring", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(\"{xs.$0}\")", &["len"]),
    ("dot/eof-no-newline", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nxs.$0", &["len"]),
    ("dot/in-fn-param", "fn f(ys: List<Int>) -> Int:\n    ys.$0\n", &["len"]),
    ("dot/in-fn-local", "fn f() -> Int:\n    let ys = [1]\n    ys.$0\n", &["len"]),
    ("named/std-empty", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.sorted($0))", &["by"]),
    ("named/std-prefix", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.sorted(b$0))", &["by"]),
    ("named/user-fn", "fn f(a: Int, scale: Int = 1) -> Int = a\nprint(f(1, sc$0))", &["scale"]),
    ("named/ctor", "struct Edge:\n    to: Int\n    w: Int\nlet e = Edge(t$0", &["to"]),
    ("named/ctor-second", "struct Edge:\n    to: Int\n    w: Int\nlet e = Edge(to: 1, $0)", &["w"]),
    ("name/prefix-call", "print(ru$0)", &["runs"]),
    ("name/local", "let total = 3\nprint(tot$0)", &["total"]),
    ("name/param", "fn f(count: Int) -> Int:\n    co$0\n", &["count"]),
    ("name/lambda-param", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint(xs.map(value => val$0))", &["value"]),
    ("name/loop-var", "for idx in 0..3:\n    print(id$0)", &["idx"]),
    ("name/later-fn", "print(hel$0)\nfn helper() -> Int = 1\n", &["helper"]),
    ("name/struct", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nlet f = Ed$0", &["Edge"]),
    ("name/keyword", "whi$0", &["while"]),
    ("name/empty", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nprint($0)", &["xs"]),
    ("name/unimported", "let t = SegT$0", &["SegTree"]),
    ("name/annotation", "@$0\nfn brute():\n    print(1)\n", &[]),
    ("use/module", "use std.$0", &["graph","segtree"]),
    ("use/module-prefix", "use std.gr$0", &["graph"]),
    ("use/items", "use std.graph.{$0}", &["Graph"]),
    ("use/items-prefix", "use std.graph.{Gr$0}", &["Graph"]),
    ("use/items-second", "use std.graph.{Graph, $0}", &[]),
    ("use/item-dot", "use std.graph.$0", &["Graph"]),
    ("type/let", "let x: Li$0", &["List"]),
    ("type/let-user", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nlet f: Ed$0", &["Edge"]),
    ("type/generic-arg", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nlet ys: List<Ed$0>", &["Edge"]),
    ("type/param", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nfn f(q: Ed$0)", &["Edge"]),
    ("type/return", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nfn f() -> Ed$0", &["Edge"]),
    ("type/field", "struct Edge:\n    to: Int\nstruct G:\n    es: List<Ed$0>\n", &["Edge"]),
    ("type/std", "use std.segtree.*\nlet t: SegT$0", &["SegTree"]),
    ("type/opt", "struct Edge:\n    to: Int\n    w: Int\nlet es: List<Edge> = []\nlet e = Edge(to: 1, w: 2)\nlet xs = [3, 1, 2]\nlet o: ?Ed$0", &["Edge"]),
    ("match/variant", "enum Dir:\n    Up\n    Down\nlet d = Dir.Up\nmatch d:\n    Dir.U$0", &["Up"]),
];

#[test]
fn completion_offers_what_each_context_needs() {
    let mut server = Server::new();
    server.handle(&message(Some(1), "initialize", obj([])));
    server.handle(&message(None, "initialized", obj([])));
    let mut misses = Vec::new();
    for (id, (context, marked, want)) in (2..).zip(CASES) {
        let at = marked.find("$0").expect("a cursor");
        let text = marked.replacen("$0", "", 1);
        let uri = format!("file:///contexts/{}.cp", context.replace('/', "-"));
        server.handle(&message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", text.as_str().into()),
                ]),
            )]),
        ));
        let reply = server.handle(&message(
            Some(id),
            "textDocument/completion",
            obj([
                ("textDocument", obj([("uri", uri.as_str().into())])),
                ("position", position(&text, at)),
            ]),
        ));
        let labels: Vec<&str> = reply
            .iter()
            .filter_map(|r| r.at(&["result", "items"]).as_array())
            .flatten()
            .filter_map(|item| item.at(&["label"]).as_str())
            .collect();
        let offered = |w: &&str| {
            labels
                .iter()
                .any(|l| l == w || l.strip_suffix(':') == Some(w))
        };
        let ok = if want.is_empty() {
            !labels.is_empty()
        } else {
            want.iter().all(offered)
        };
        if !ok {
            misses.push(format!(
                "{context}: wanted {want:?}, got {:?}",
                &labels[..labels.len().min(8)]
            ));
        }
    }
    assert!(
        misses.is_empty(),
        "completion misses:\n{}",
        misses.join("\n")
    );
}
