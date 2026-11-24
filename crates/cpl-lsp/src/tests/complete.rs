//! Completion and signature help.

use super::*;

#[test]
fn completion_after_a_trailing_dot_lists_fields_getters_and_methods() {
    let items = completions(&format!("{ROAD}let r = Road(1, 2)\nprint(r.|)\n"));
    assert_eq!(labels(&items), ["to", "w", "twice", "heavy"]);
    assert_eq!(detail_of(&items, "w"), "w: Int");
    assert_eq!(detail_of(&items, "twice"), "fn twice(self) -> Int");
    assert_eq!(items.iter().find(|i| i.0 == "heavy").unwrap().1, 10);
    // A fn without `self` belongs to the type, not to a value.
    assert!(!labels(&items).contains(&"make"), "{items:?}");
}

#[test]
fn dot_completion_after_a_method_call_lists_what_it_returns() {
    let items = completions("let xs = [3, 1]\nprint(xs.sorted().|)\n");
    assert!(labels(&items).contains(&"len"), "{items:?}");
    let items = completions("let xs = [3, 1]\nprint(xs.sorted().sorted().|)\n");
    assert!(labels(&items).contains(&"len"), "{items:?}");
    let items = completions("let m = 3\nprint((1..=m).|)\n");
    assert!(labels(&items).contains(&"len"), "{items:?}");
    let items = completions(&format!(
        "{ROAD}fn f() -> List<Road> = [Road(1, 2)]\nprint(f()[0].|)\n"
    ));
    assert!(labels(&items).contains(&"twice"), "{items:?}");
}

#[test]
fn dot_completion_works_on_a_line_that_does_not_parse_yet() {
    // The item ends mid-expression: the checker still types `r`.
    let items = completions(&format!("{ROAD}let r = Road(1, 2)\nlet x = r.|\n"));
    assert!(labels(&items).contains(&"twice"), "{items:?}");
    // And partway through a member's name, after a call.
    let items = completions(&format!(
        "{ROAD}fn f() -> Road = Road(1, 2)\nprint(f().tw|)\n"
    ));
    assert!(labels(&items).contains(&"twice"), "{items:?}");
}

#[test]
fn dot_completion_on_a_type_lists_what_it_has_without_a_value() {
    let items = completions(&format!(
        "{ROAD}enum Dir:\n    L\n    R\nprint(Dir.|)\nprint(Road.make(1))\n"
    ));
    assert_eq!(labels(&items), ["L", "R"]);
    let items = completions(&format!("{ROAD}let r = Road.|\n"));
    assert_eq!(labels(&items), ["make"]);
}

#[test]
fn dot_completion_on_a_library_type_substitutes_its_type_arguments() {
    let src = "domain V = 0..5\nvar t = SegTree(max, [V] v => 0)\nt.|\n";
    let items = completions(src);
    for method in ["get", "set", "fold", "update"] {
        assert!(
            labels(&items).contains(&method),
            "{method} missing: {items:?}"
        );
    }
    assert_eq!(detail_of(&items, "set"), "fn set(mut self, i: V, x: Int)");
}

#[test]
fn dot_completion_on_records_domains_and_positions() {
    let items = completions("let p = (x: 1, y: 2)\nprint(p.|)\n");
    assert_eq!(labels(&items), ["x", "y"]);
    let items = completions("domain V = 0..5\nprint(V.|)\n");
    assert!(labels(&items).contains(&"first") && labels(&items).contains(&"rev"));
    let items = completions("domain V = 0..5\nfor v in V:\n    print(v.|)\n");
    assert!(labels(&items).contains(&"ord") && labels(&items).contains(&"r"));
    let items = completions("domain V = 0..5\nlet xs = read<[V.|drop(1)]Int>()\nprint(xs)\n");
    assert!(labels(&items).contains(&"first"), "{items:?}");
}

/// The members offered on every domain and position are ones the checker types.
#[test]
fn builtin_domain_and_position_members_are_the_checkers() {
    use crate::complete::{DOMAIN_MEMBERS, POS_MEMBERS};
    let mut wrong = Vec::new();
    for (members, receiver) in [(DOMAIN_MEMBERS, "V"), (POS_MEMBERS, "v")] {
        for (name, _) in members {
            let src = format!("domain V = 0..5\nfor v in V:\n    let x = {receiver}.{name}\n");
            let (mut server, replies) = open(&src);
            // The back ends may not lower every one yet (E0501); the checker types them.
            let errors: Vec<&Value> = replies
                .iter()
                .filter(|d| d.at(&["code"]).as_str() != Some("E0501"))
                .collect();
            let hover = hover_at(&mut server, 2, 8);
            let text = hover.at(&["contents", "value"]).as_str().unwrap_or("");
            let ty_line = text.lines().nth(1).unwrap_or("");
            if !errors.is_empty() {
                wrong.push(format!("{receiver}.{name}: {errors:?}"));
            } else if !ty_line.starts_with("x: ") || ty_line == "x: ?" || ty_line.contains('_') {
                wrong.push(format!("{receiver}.{name} is untyped: {ty_line}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn completion_and_hover_see_names_from_a_declaration_that_did_not_parse() {
    let src = "let total = 2 * $x\nfn main(n: Int):\n    let half = n / $2\n    print(to|)\n";
    let names = labels(&completions(src))
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    assert!(
        names.contains(&"total".into()) && names.contains(&"half".into()),
        "{names:?}"
    );
    let hover = ask_at(
        "textDocument/hover",
        "let ($left, right) = (4, 6)\nprint(ri|ght)\n",
    );
    assert_eq!(
        hover.at(&["contents", "value"]).as_str(),
        Some("```cp\nright: Int\n```")
    );
}

#[test]
fn completion_offers_locals_declarations_and_the_prelude_with_signatures() {
    let src = format!("{ROAD}/// Adds one.\nfn inc(x: Int) -> Int = x + 1\nfn main(n: Int):\n    let total = n * 2\n    pr|\n");
    let items = completions(&src);
    let names = labels(&items);
    // Locals first, then this file, then the prelude, then keywords.
    for name in [
        "total", "n", "inc", "Road", "SegTree", "Dsu", "zip", "while",
    ] {
        assert!(names.contains(&name), "{name} missing: {names:?}");
    }
    assert!(names.iter().position(|n| *n == "total") < names.iter().position(|n| *n == "inc"));
    assert_eq!(detail_of(&items, "total"), "total: Int");
    assert_eq!(detail_of(&items, "inc"), "fn inc(x: Int) -> Int");
    let reply = ask_at("textDocument/completion", &src);
    let inc = reply
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .find(|i| i.at(&["label"]).as_str() == Some("inc"))
        .unwrap();
    assert_eq!(
        inc.at(&["documentation", "value"]).as_str(),
        Some("Adds one.")
    );
    let zip = detail_of(&items, "zip");
    assert!(zip.starts_with("fn zip"), "{zip}");
    // A local of another fn is not in scope.
    let items = completions("fn a():\n    let hidden = 1\nfn b():\n    h|\n");
    assert!(!labels(&items).contains(&"hidden"), "{items:?}");
}

#[test]
fn use_std_completes_module_names_and_then_their_exports() {
    let items = completions("use std.|\n");
    for module in ["segtree", "seq", "graph", "math"] {
        assert!(labels(&items).contains(&module), "{module}: {items:?}");
    }
    assert!(!labels(&items).contains(&"prelude"));
    assert!(items.iter().all(|(_, kind, _)| *kind == 9));
    let items = completions("use std.segtree.{Seg|}\n");
    assert!(labels(&items).contains(&"SegTree"), "{items:?}");
    let items = completions("use std.graph.{|}\n");
    assert!(labels(&items).contains(&"bfs01"), "{items:?}");
    assert!(
        !labels(&items)
            .iter()
            .any(|name| name.ends_with("_of_pairs") || name.ends_with("_of_triples")),
        "{items:?}"
    );
    assert!(completions("// see xs.|\n").is_empty());
}

#[test]
fn signature_help_tracks_the_argument_being_typed() {
    let decl = "/// Moves along.\nfn walk(from: Int, steps: Int, back: Bool) -> Int = from\n";
    let (label, active, params) = signature(&format!("{decl}print(walk(|))\n"));
    assert_eq!(label, "walk(from: Int, steps: Int, back: Bool) -> Int");
    assert_eq!(params, ["from: Int", "steps: Int", "back: Bool"]);
    assert_eq!(active, 0);
    let (_, active, _) = signature(&format!("{decl}print(walk(1, [2, 3].len, |))\n"));
    assert_eq!(active, 2);
    // Inside a nested call it is that call's signature.
    let (label, ..) = signature(&format!("{decl}print(walk(1, walk(|"));
    assert!(label.starts_with("walk("));
    let help = ask_at(
        "textDocument/signatureHelp",
        &format!("{decl}print(walk(1, |"),
    );
    assert_eq!(
        help.at(&["signatures"]).as_array().unwrap_or(&[])[0]
            .at(&["documentation", "value"])
            .as_str(),
        Some("Moves along.")
    );
    assert!(ask_at("textDocument/signatureHelp", "let x = 1|\n").is_null());
}

#[test]
fn signature_help_for_methods_and_library_fns() {
    let (label, active, _) = signature(&format!("{ROAD}let r = Road(1, 2)\nprint(Road.make(|))\n"));
    assert_eq!((label.as_str(), active), ("make(to: Int) -> Road", 0));
    let src = "domain V = 0..5\nvar t = SegTree<V, Int>.of(max, fn(v) -> 0)\nt.set(V.first, |)\n";
    let (label, active, params) = signature(src);
    assert!(label.starts_with("set("), "{label}");
    assert_eq!(params.len(), 2, "{label}");
    assert_eq!(active, 1);
    let (label, ..) = signature("print(gcd(4, |))\n");
    assert!(label.starts_with("gcd("), "{label}");
}

#[test]
fn inlay_hints_name_the_parameters_of_literal_arguments() {
    let src =
        "fn walk(from: Int, steps: Int) -> Int = from + steps\nlet a = 1\nprint(walk(a, 3))\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        9,
        "textDocument/inlayHint",
        obj([("textDocument", doc())]),
    ));
    let hints: Vec<(u32, u32, String, u32)> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|h| {
            (
                h.at(&["position", "line"]).as_u32().unwrap(),
                h.at(&["position", "character"]).as_u32().unwrap(),
                h.at(&["label"]).as_str().unwrap().to_string(),
                h.at(&["kind"]).as_u32().unwrap(),
            )
        })
        .collect();
    // `a` names itself; the literal `3` gets its parameter's name.
    assert_eq!(
        hints,
        [
            (1, 5, ": 1..=1".to_string(), 1),
            (2, 14, "steps:".to_string(), 2)
        ]
    );
}

#[test]
fn the_server_advertises_completion_and_signature_help() {
    let mut server = Server::default();
    let reply = server.handle(&request(1, "initialize", obj([])));
    let caps = reply[0].at(&["result", "capabilities"]);
    assert_eq!(
        caps.at(&["completionProvider", "triggerCharacters"])
            .as_array()
            .unwrap_or(&[]),
        [
            Value::from("."),
            Value::from("("),
            Value::from(","),
            Value::from("{")
        ]
    );
    assert_eq!(
        caps.at(&["signatureHelpProvider", "triggerCharacters"])
            .as_array()
            .unwrap_or(&[]),
        [Value::from("("), Value::from(",")]
    );
}

#[test]
fn dot_completion_on_library_structs_tells_properties_from_methods() {
    let items = completions("var q = Deque<Int>()\nq.|\n");
    let kind = |label: &str| {
        items
            .iter()
            .find(|i| i.0 == label)
            .unwrap_or_else(|| panic!("no `{label}` in {items:?}"))
            .1
    };
    // `len` and `first` are `@get` properties; `push` is called.
    assert_eq!((kind("len"), kind("first"), kind("push")), (10, 10, 2));
    let items = completions("var h = MinHeap<Int>()\nh.|\n");
    assert!(
        labels(&items).contains(&"push") && labels(&items).contains(&"len"),
        "{items:?}"
    );
}

#[test]
fn dot_completion_on_built_in_types_reads_the_compiler_catalog() {
    let kind_of = |items: &[(String, u32, String)], label: &str| {
        let item = items.iter().find(|i| i.0 == label);
        item.unwrap_or_else(|| panic!("no `{label}` in {items:?}"))
            .clone()
    };
    // A list: `len` reads as a property, `push` is called, `T` is `Int`.
    let items = completions("var xs = [3, 1, 2]\nxs.|\n");
    assert_eq!(kind_of(&items, "len").1, 10);
    assert_eq!(kind_of(&items, "push").1, 2);
    assert_eq!(kind_of(&items, "get").2, "get(i: Int) -> ?Int");
    // A string, a map with its key and value types, a set, an array.
    let items = completions("let s = \"hello\"\ns.|\n");
    assert!(labels(&items).contains(&"find_all"), "{items:?}");
    let items = completions("var m: Map<Int, Str> = Map()\nm.|\n");
    assert_eq!(kind_of(&items, "first").2, "first: ?(Int, Str)");
    assert_eq!(kind_of(&items, "pop_first").1, 2);
    let items = completions("var st: Set<Int> = Set()\nst.|\n");
    assert!(labels(&items).contains(&"first_ge"), "{items:?}");
    let items = completions("domain D = 0..3\nvar a: [D]Int = 0\na.|\n");
    assert_eq!(kind_of(&items, "fill").2, "fill(x: Int)");
    // On a half-typed line too.
    let items = completions("var xs = [3, 1, 2]\nlet k = xs.|\nprint(k)\n");
    assert!(labels(&items).contains(&"pop"), "{items:?}");
}

#[test]
fn signature_help_covers_built_in_methods() {
    let help = signature("var xs = [3, 1, 2]\nxs.insert(0, |)\n");
    assert_eq!(help.0, "insert(i: Int, x: Int)");
    assert_eq!(help.1, 1);
}

#[test]
fn bare_names_include_the_compiler_prelude() {
    let items = completions("let n = 3\npr|\n");
    let print = items
        .iter()
        .find(|i| i.0 == "print")
        .expect("print is offered");
    assert_eq!(print.1, 3);
    let items = completions("let n: I| = 3\n");
    assert!(
        labels(&items).contains(&"Int") && labels(&items).contains(&"I128"),
        "{items:?}"
    );
    // A stdlib fn that needs a `use` comes with it, after what is in scope.
    let reply = ask_at("textDocument/completion", "floor_s|\n");
    let z = reply
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .find(|i| i.at(&["label"]).as_str() == Some("floor_sum"))
        .cloned()
        .expect("floor_sum is offered");
    assert!(
        z.at(&["sortText"]).as_str().unwrap().starts_with('5'),
        "{z:?}"
    );
    assert_eq!(
        z.at(&["additionalTextEdits"]).as_array().unwrap_or(&[])[0]
            .at(&["newText"])
            .as_str(),
        Some("use std.math.{floor_sum}\n")
    );
}

#[test]
fn completion_ranks_first_what_fits_the_expected_type() {
    let decls = "fn area(w: Int, h: Int) -> Int = w * h\nfn name() -> Str = \"a\"\nlet k: Int = 3\nlet s = \"x\"\nvar c = 0\n";
    let fits = |src: String, want: &[&str], not: &[&str]| {
        let got = best_fits(&src);
        for w in want {
            assert!(
                got.iter().any(|g| g == w),
                "{w} should fit in {src:?}: {got:?}"
            );
        }
        for n in not {
            assert!(
                !got.iter().any(|g| g == n),
                "{n} should not fit in {src:?}: {got:?}"
            );
        }
    };
    let ints = ["area", "c", "k", "gcd"];
    let strs = ["name", "s"];
    // An argument: the parameter's type; a named one: that parameter's.
    fits(format!("{decls}print(area(k, |))\n"), &ints, &strs);
    fits(format!("{decls}print(area(h: |))\n"), &ints, &strs);
    // A `let` with its type written, and an assignment.
    fits(format!("{decls}let t: Str = |\n"), &strs, &ints);
    fits(format!("{decls}c = |\n"), &["area", "k"], &strs);
    fits(format!("{decls}c += |\n"), &["area", "k"], &strs);
    // After a dot, the members of the right type.
    assert_eq!(
        best_fits(&format!("{ROAD}let r = Road(1, 2)\nlet b: Bool = r.|\n")),
        ["heavy"]
    );
    // A fn wanted (`by:`, `cmp:`): the fns taking those parameters.
    let fns = "fn key(x: Int) -> Int = -x\nfn later(a: Int, b: Int) -> Bool = a > b\nfn word(s: Str) -> Int = 1\nlet n = 3\nlet xs = [3, 1]\n";
    fits(
        format!("{fns}print(xs.sorted(by: |))\n"),
        &["key"],
        &["later", "word", "n"],
    );
    fits(
        format!("{fns}print(xs.sorted(cmp: |))\n"),
        &["later"],
        &["key", "word", "n"],
    );
    fits(
        format!("{fns}print(xs.map(|))\n"),
        &["key"],
        &["later", "n"],
    );
    // Nothing expected: no ranking by type.
    assert_eq!(
        best_fits(&format!("{decls}print(|)\n")),
        Vec::<String>::new()
    );
}

#[test]
fn completion_ranks_types_above_values_where_a_type_goes() {
    let decls = "struct Edge:\n    to: Int\nlet edges = [Edge(to: 1)]\nlet n = 3\n";
    let order = |src: String| {
        let reply = ask_at("textDocument/completion", &src);
        let mut items: Vec<(String, String)> = reply
            .at(&["items"])
            .as_array()
            .unwrap_or(&[])
            .iter()
            .map(|i| {
                let sort = i.at(&["sortText"]).as_str().unwrap().to_string();
                (sort, i.at(&["label"]).as_str().unwrap().to_string())
            })
            .collect();
        items.sort();
        items
            .into_iter()
            .map(|(_, label)| label)
            .collect::<Vec<_>>()
    };
    let at = |names: &[String], name: &str| names.iter().position(|n| n == name);
    for typed in [
        "let e: |",
        "let o: ?|",
        "fn f(q: Int, r: |",
        "fn f() -> |",
        "let es: List<|",
        "let m: Map<Int, |",
        "struct G:\n    es: |",
    ] {
        let names = order(format!("{decls}{typed}\n"));
        for ty in ["Edge", "Int"] {
            assert!(
                at(&names, ty) < at(&names, "edges") && at(&names, ty) < at(&names, "n"),
                "{ty} should rank above values after {typed:?}: {names:?}"
            );
        }
    }
    // Where a value goes, values keep their place above the prelude's types.
    for typed in ["print(|", "f(by: |", "let k = |"] {
        let names = order(format!("{decls}{typed}\n"));
        assert!(
            at(&names, "edges") < at(&names, "Int"),
            "values should come first after {typed:?}: {names:?}"
        );
    }
}

#[test]
fn signature_help_shows_defaults_and_named_only_parameters() {
    let decl = "fn walk(from: Int, steps: Int = 1; back: Bool = false) -> Int = from\n";
    let (label, active, params) = signature(&format!("{decl}print(walk(1, |))\n"));
    assert_eq!(
        label,
        "walk(from: Int, steps: Int = 1; back: Bool = false) -> Int"
    );
    assert_eq!(
        params,
        ["from: Int", "steps: Int = 1", "back: Bool = false"]
    );
    assert_eq!(active, 1);
    // A named argument is its parameter, even one only passed by name.
    let (_, active, _) = signature(&format!("{decl}print(walk(1, back: |))\n"));
    assert_eq!(active, 2);
    // Past the positional ones, the first option not given yet.
    let (_, active, _) = signature(&format!("{decl}print(walk(1, 2, |))\n"));
    assert_eq!(active, 2);
    let (_, active, _) = signature(&format!("{decl}print(walk(1, 2, back: true, |))\n"));
    assert!(active > 2, "{active}");
    // A stdlib method's default, as its declaration writes it.
    let (label, ..) = signature("use std.mod.{Comb}\nlet c = Comb<7>.new(|)\n");
    assert!(label.starts_with("new(limit: Int = 0) -> Comb<"), "{label}");
}

#[test]
fn a_library_type_offers_its_api_not_its_internals() {
    let src = "domain V = 0..5\nvar t = SegTree(max, [V] v => 0)\nt.|\n";
    let items = completions(src);
    let shown = labels(&items);
    for internal in [
        "m",
        "t",
        "lz",
        "has_lz",
        "span",
        "log",
        "build",
        "push",
        "pull_node",
    ] {
        assert!(!shown.contains(&internal), "{internal} shown: {items:?}");
    }
    // Its methods rank first, and a documented field is still offered.
    assert!(
        shown.contains(&"fold") && shown.contains(&"source"),
        "{shown:?}"
    );
    let sorted = ask_at("textDocument/completion", src);
    let sort_of = |label: &str| {
        let items = sorted.at(&["items"]).as_array().unwrap_or(&[]);
        let item = items
            .iter()
            .find(|i| i.at(&["label"]).as_str() == Some(label));
        item.and_then(|i| i.at(&["sortText"]).as_str())
            .unwrap_or("")
            .to_string()
    };
    assert!(
        sort_of("fold") < sort_of("source"),
        "{} {}",
        sort_of("fold"),
        sort_of("source")
    );
    // Signature help offers the API and not the internals.
    let call = src.replace("t.|", "t.fold(|");
    assert!(
        !ask_at("textDocument/signatureHelp", &call).is_null(),
        "{call}"
    );
    let call = src.replace("t.|", "t.push(|");
    assert_eq!(ask_at("textDocument/signatureHelp", &call), Value::Null);
    // Hover on the type shows the same.
    let (mut server, _) = open(&src.replace("t.|\n", "print(t)\n"));
    let hover = hover_at(&mut server, 1, 10);
    let text = hover.at(&["contents", "value"]).as_str().unwrap_or("");
    assert!(
        text.contains("fn fold(") && text.contains("source:"),
        "{text}"
    );
    for internal in ["\n  lz:", "\n  m:", "fn push(", "fn build("] {
        assert!(!text.contains(internal), "{internal} in {text}");
    }
    // A type of this file shows everything it has.
    let items = completions(&format!("{ROAD}fn f(r: Road) -> Int = r.|\n"));
    assert!(labels(&items).contains(&"to"), "{items:?}");
}

#[test]
fn dot_completion_on_an_int_lists_the_fns_that_take_it_as_self() {
    let src = "fn twice(x: Int) -> Int = 2 * x\nfn half(self: Int) -> Int = self / 2\nlet n: Int = read()\nlet y = twice(n)\nlet z = y.|\n";
    let items = completions(src);
    for method in ["half", "isqrt", "divmod", "digits", "bit"] {
        assert!(labels(&items).contains(&method), "{method}: {items:?}");
    }
    assert!(!labels(&items).contains(&"twice"), "{items:?}");
    assert_eq!(detail_of(&items, "half"), "fn half(self: Int) -> Int");
    // A value whose type is a range of Int has Int's methods.
    let items = completions("let k: 1..=9 = read()\nprint(k.|)\n");
    assert!(labels(&items).contains(&"digits"), "{items:?}");
}

#[test]
fn inlay_hints_show_a_sum_of_ranged_cells_as_int() {
    let src = "let n: 1..=100 = read()\nlet a: [0..n]1..=1e9 = read()\nlet s = a.sum()\nprint(s)\n";
    let (mut server, _) = open(src);
    let reply = server.handle(&request(
        34,
        "textDocument/inlayHint",
        obj([
            ("textDocument", doc()),
            ("range", obj([("start", pos(0, 0)), ("end", pos(4, 0))])),
        ]),
    ));
    let labels: Vec<&str> = reply[0]
        .at(&["result"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|h| h.at(&["label"]).as_str().unwrap())
        .collect();
    assert_eq!(labels, [": Int"]);
}

/// `fn runs<S: Indexed, T, K>(self: S, by: ..)` called as `s.runs(..)`:
/// signature help's label, without the receiver and the type parameters.
fn as_called(detail: &str) -> String {
    let rest = detail.strip_prefix("fn ").unwrap_or(detail);
    let open = rest.find('(').unwrap();
    let name = rest[..open].split('<').next().unwrap();
    let args = &rest[open + 1..];
    let args = args
        .split_once(", ")
        .filter(|(first, _)| first.starts_with("self"));
    let args = args.map_or(&rest[open + 1..], |(_, rest)| rest);
    let args = args
        .strip_prefix("self: S)")
        .map_or(args.to_string(), |r| format!("){r}"));
    format!("{name}({args}")
}

// Hover, signature help and completion render one signature: the
// declaration as written, also for a library fn called as a method, whose
// checker fact is its free form (rule commands/R11).
#[test]
fn hover_signature_help_and_completion_agree_on_library_methods() {
    let calls = [
        ("let s = \"aab\"\nprint(s.", "runs", "().len())"),
        ("let xs = [3, 1, 2]\nprint(xs.", "distinct", "())"),
        ("let xs = [3, 1, 2]\nprint(xs.", "is_sorted", "())"),
        ("let xs = [3, 1, 2]\nprint(xs.", "deduped", "())"),
    ];
    for (before, name, after) in calls {
        let items = completions(&format!("{before}|{after}"));
        let detail = detail_of(&items, name).to_string();
        let hover = ask_at("textDocument/hover", &format!("{before}{name}|{after}"));
        let shown = hover.at(&["contents", "value"]).as_str().unwrap_or("");
        let line = shown.lines().nth(1).unwrap_or("");
        assert_eq!(line, detail, "hover on {name} disagrees with completion");
        let open = &after[..after.find('(').unwrap() + 1];
        let (label, ..) = signature(&format!("{before}{name}{open}|{}", &after[open.len()..]));
        assert_eq!(label, as_called(&detail), "signature help on {name}");
    }
}

#[test]
fn completion_offers_the_open_calls_argument_names_not_given_yet() {
    let items = completions("let xs = [3, 1]\nprint(xs.sorted(b|))\n");
    assert!(labels(&items).contains(&"by:"), "{items:?}");
    let items = completions("fn f(a: Int, scale: Int = 1) -> Int = a\nprint(f(1, sc|))\n");
    assert!(labels(&items).contains(&"scale:"), "{items:?}");
    assert!(
        !labels(&items).contains(&"a:"),
        "a is given by position: {items:?}"
    );
    let items = completions(&format!("{ROAD}let r = Road(to: 1, |)\n"));
    assert!(labels(&items).contains(&"w:"), "{items:?}");
    assert!(!labels(&items).contains(&"to:"), "to is given: {items:?}");
    // A value being written is not a name.
    let items = completions("let xs = [3, 1]\nprint(xs.sorted(by: x|))\n");
    assert!(!labels(&items).contains(&"by:"), "{items:?}");
}

#[test]
fn completion_works_after_a_dot_in_an_optional_an_interpolation_and_a_use() {
    let items = completions("let m: ?Int = None\nprint(m.|)\n");
    assert!(labels(&items).contains(&"is_some_and"), "{items:?}");
    let items = completions("let xs = [3, 1]\nprint(\"{xs.|}\")\n");
    assert!(labels(&items).contains(&"len"), "{items:?}");
    let items = completions("print(\"{{xs.|}}\")\n");
    assert!(items.is_empty(), "`{{` is a literal brace: {items:?}");
    let items = completions("use std.graph.|\n");
    assert!(labels(&items).contains(&"Graph"), "{items:?}");
}

fn triggered(src: &str, trigger: &str) -> Vec<String> {
    let context = obj([
        ("triggerKind", 2.into()),
        ("triggerCharacter", trigger.into()),
    ]);
    ask_at_with("textDocument/completion", src, obj([("context", context)]))
        .at(&["items"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .map(|item| {
            item.at(&["insertText"])
                .as_str()
                .or(item.at(&["label"]).as_str())
                .unwrap()
                .to_string()
        })
        .collect()
}

#[test]
fn a_typed_paren_comma_or_brace_completes_only_what_belongs_there() {
    let names = triggered("let xs = [3, 1]\nprint(xs.sorted(|))\n", "(");
    assert!(names.contains(&"by: ".to_string()), "{names:?}");
    assert!(
        !names.contains(&"xs".to_string()),
        "no scope names on `(`: {names:?}"
    );
    let names = triggered(
        "fn f(a: Int, scale: Int = 1) -> Int = a\nprint(f(1,|))\n",
        ",",
    );
    assert_eq!(
        names,
        [" scale: "],
        "after `,` the name comes with its space"
    );
    assert!(triggered("print(|)\n", "(")
        .iter()
        .all(|n| n.ends_with(": ")));
    let items = triggered("use std.graph.{|\n", "{");
    assert!(items.iter().any(|i| i == "Graph"), "{items:?}");
    assert!(triggered("let s = \"{|\"\n", "{").is_empty());
}

#[test]
fn completion_ranks_what_the_context_starts_with_first() {
    let first_of = |src: &str, label: &str| {
        let reply = ask_at("textDocument/completion", src);
        let items = reply
            .at(&["items"])
            .as_array()
            .map(<[_]>::to_vec)
            .unwrap_or_default();
        // The rank is the sortText's leading digit; the label follows it.
        let sort = |i: &Value| i.at(&["sortText"]).as_str().unwrap_or("~")[..1].to_string();
        let best = items.iter().map(sort).min().unwrap_or_default();
        let mine = items
            .iter()
            .find(|i| i.at(&["label"]).as_str() == Some(label))
            .map(sort);
        let labels: Vec<_> = items
            .iter()
            .take(12)
            .map(|i| i.at(&["label"]).as_str())
            .collect();
        assert_eq!(
            mine.as_deref(),
            Some(best.as_str()),
            "{label} in {src:?}: {labels:?}"
        );
    };
    // A match arm on `?T` starts with `Some` or `None`, before any std name.
    first_of("let m: ?Int = None\nmatch m:\n    |", "Some");
    first_of(
        "let m: ?Int = None\nmatch m:\n    Some(x) => print(x)\n    |",
        "None",
    );
    first_of(
        "enum Who:\n    A\n    B\nlet w = Who.A\nmatch w:\n    |",
        "B",
    );
    // A `@test` fn's body starts with `assert`.
    first_of("fn f(a: Int) -> Int = a\n@test\nfn t():\n    |", "assert");
    first_of("@test fn t():\n    let x = 1\n    |", "assert");
}

#[test]
fn signature_help_highlights_a_named_option_written_first() {
    let (label, active, params) =
        signature("fn f(; after: Int = 0) -> Int = after\nprint(f(after: |))\n");
    assert_eq!(label, "f(; after: Int = 0) -> Int");
    assert_eq!((active, params), (0, vec!["after: Int = 0".to_string()]));
    // A built-in method's options, by their labels.
    let (label, active, params) = signature("var s = Set<Int>()\nprint(s.drain(before: |))\n");
    assert!(
        label.starts_with("drain(; after: ?Int = none, before:"),
        "{label}"
    );
    assert_eq!(params[active as usize], "before: ?Int = none");
    let (_, active, params) = signature("var s = Set<Int>()\nprint(s.drain(while: |))\n");
    assert_eq!(params[active as usize], "while: (Int) -> Bool");
}
