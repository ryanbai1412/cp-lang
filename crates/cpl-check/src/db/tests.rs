use super::*;

struct MapLoader(Vec<(String, String, String)>);

impl Loader for MapLoader {
    fn load(&self, _from: &str, module: &[String]) -> Option<(String, String)> {
        let key = module.join(".");
        self.0
            .iter()
            .find(|(k, _, _)| k == &key)
            .map(|(_, path, text)| (path.clone(), text.clone()))
    }
}

#[test]
fn imported_module_still_checks_when_analyzed_directly() {
    let mut db = Database::with_loader(Box::new(MapLoader(vec![(
        "lib.bad".into(),
        "lib/bad.cp".into(),
        "pub fn f() -> Int:\n    let x: Int = \"oops\"\n    x\n".into(),
    )])));
    db.set_file("main.cp", "use lib.bad.{f}\nprint(f())\n");
    let main = db.analyze("main.cp").unwrap();
    assert!(main.diagnostics.is_empty(), "{:?}", main.diagnostics);
    let direct = db.analyze("lib/bad.cp").unwrap();
    assert!(!direct.diagnostics.is_empty());
}

#[test]
fn an_import_cycle_is_an_error_naming_the_whole_cycle() {
    let module = |name: &str, text: &str| {
        (
            format!("lib.{name}"),
            format!("lib/{name}.cp"),
            text.to_string(),
        )
    };
    let mut db = Database::with_loader(Box::new(MapLoader(vec![
        module("a", "use lib.b.{g}\npub fn f() -> Int: g()\n"),
        module("b", "use lib.c.{h}\npub fn g() -> Int: h()\n"),
        module("c", "use lib.a.{f}\npub fn h() -> Int: 1\n"),
    ])));
    db.set_file("main.cp", "use lib.a.{f}\nprint(f())\n");
    let main = db.analyze("main.cp").unwrap();
    let around = ["lib/a.cp", "lib/b.cp", "lib/c.cp", "lib/a.cp"].map(String::from);
    assert_eq!(db.import_cycles(), [around.to_vec()]);
    let messages = |a: &Analysis| {
        a.diagnostics
            .iter()
            .map(|d| d.message.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        messages(&main),
        ["`lib.a` is in an import cycle: a -> b -> c -> a"]
    );
    for file in ["lib/b.cp", "lib/c.cp", "lib/a.cp"] {
        let direct = db.analyze(file).unwrap();
        assert!(direct.has_errors(), "{file}: {:?}", direct.diagnostics);
    }
}

#[test]
fn a_top_level_move_points_at_the_move_after_an_edit_above_it() {
    let mut db = Database::new();
    for gap in ["", "let pad = 1\nprint(pad)\n"] {
        let src = format!("let a = [1, 2]\nlet b = a\n{gap}print(a.len, b.len)\n");
        db.set_file("a.cp", src.as_str());
        let diags = db.diagnostics("a.cp");
        assert_eq!(diags.len(), 1, "{diags:?}");
        let moved = src.find("= a").unwrap() as u32 + 2;
        assert_eq!(diags[0].labels[0].span, Span::new(moved, moved + 1));
        assert_eq!(diags[0].fixes[0].edits[0].0, Span::point(moved + 1));
    }
}

#[test]
fn a_var_cannot_take_a_lent_heap_parameter() {
    let mut db = Database::new();
    let src = "fn f(xs: List<Int>) -> List<Int>:\n    var ys = xs\n    ys\n";
    db.set_file("a.cp", src);
    let diags = db.diagnostics("a.cp");
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].code.0, "E0608");
    let at = src.find("= xs").unwrap() as u32 + 2;
    assert_eq!(diags[0].span, Span::new(at, at + 2));
    for ok in [
        "fn f(xs: List<Int>) -> List<Int>:\n    var ys = xs.clone()\n    ys\n",
        "fn f(xs: List<Int>) -> Int:\n    let ys = xs\n    ys.len\n",
        "fn f(n: Int) -> Int:\n    var m = n\n    m\n",
    ] {
        db.set_file("a.cp", ok);
        assert!(db.diagnostics("a.cp").is_empty(), "{ok}");
    }
}

#[test]
fn a_move_inside_a_top_level_if_or_loop_reaches_the_items_after_it() {
    let mut db = Database::new();
    for body in [
        "if xs.len > 1:\n    let ys = xs\n",
        "while false:\n    var ys = xs\n",
    ] {
        let src = format!("var xs = List(1, 2)\n{body}print(xs.len)\n");
        db.set_file("a.cp", src.as_str());
        let diags = db.diagnostics("a.cp");
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert_eq!(diags[0].code.0, "E0608");
        let moved = src.find("= xs").unwrap() as u32 + 2;
        assert_eq!(diags[0].labels[0].span, Span::new(moved, moved + 2));
    }
    db.set_file(
        "a.cp",
        "var xs = List(1, 2)\nif xs.len > 1:\n    let ys = xs\n    xs = ys\nprint(xs.len)\n",
    );
    assert!(db.diagnostics("a.cp").is_empty());
    let nested = "var xs = List(1, 2)\nfor i in 0..2:\n    if i > 0:\n        let ys = xs\n    print(xs.len)\n";
    db.set_file("a.cp", nested);
    let codes: Vec<_> = db.diagnostics("a.cp").iter().map(|d| d.code.0).collect();
    assert_eq!(codes, ["E0608"]);
}

#[test]
fn assigning_a_fixed_field_offers_var_where_it_is_declared_after_an_edit() {
    let mut db = Database::new();
    for gap in ["", "let pad = 1\nprint(pad)\n"] {
        let src = format!(
            "struct S:\n    n: Int\n    top: Int\n{gap}var s = S(n: 1, top: 0)\ns.top = 1\n"
        );
        db.set_file("a.cp", src.as_str());
        let diags = db.diagnostics("a.cp");
        assert_eq!(diags.len(), 1, "{diags:?}");
        let top = src.find("top:").unwrap() as u32;
        assert_eq!(
            diags[0].fixes[0].edits[0],
            (Span::point(top), "var ".to_string())
        );
    }
    // `u, v: V` has no place to put `var` for `v` alone.
    db.set_file(
        "a.cp",
        "struct P:\n    u, v: Int\nvar p = P(u: 1, v: 2)\np.v = 3\n",
    );
    assert!(db.diagnostics("a.cp")[0].fixes.is_empty());
}
