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
fn a_loop_that_moves_an_outer_value_finds_it_moved_next_round() {
    let mut db = Database::new();
    for body in ["let ys = xs", "keep.push(xs)"] {
        let src = format!(
            "fn f():\n    var xs = List(1, 2)\n    var keep: List<List<Int>> = []\n    for i in 0..2:\n        {body}\n    print(keep.len)\n"
        );
        db.set_file("a.cp", src.as_str());
        let diags = db.diagnostics("a.cp");
        assert_eq!(diags.len(), 1, "{diags:?}");
        assert_eq!(diags[0].code.0, "E0608");
        let at = src.rfind("xs").unwrap() as u32;
        assert_eq!(diags[0].span, Span::new(at, at + 2));
    }
    for ok in [
        "        keep.push(xs.clone())",
        "        let xs = List(i)\n        keep.push(xs)",
        "        keep.push(xs)\n        break",
        "        let all = \"\"\n        keep.push(List(all.len))",
    ] {
        let src = format!(
            "fn f():\n    var xs = List(1, 2)\n    var keep: List<List<Int>> = []\n    for i in 0..2:\n{ok}\n    print(keep.len)\n"
        );
        db.set_file("a.cp", src.as_str());
        assert!(db.diagnostics("a.cp").is_empty(), "{src}");
    }
    db.set_file(
        "a.cp",
        "fn f():\n    var xs = List(1, 2)\n    var keep: List<List<Int>> = []\n    keep.push(xs)\n    print(xs.len)\n",
    );
    let codes: Vec<_> = db.diagnostics("a.cp").iter().map(|d| d.code.0).collect();
    assert_eq!(codes, ["E0608"]);
}

#[test]
fn a_move_inside_a_top_level_if_or_loop_reaches_the_items_after_it() {
    let mut db = Database::new();
    for body in [
        "if xs.len > 1:\n    let ys = xs\n",
        "while false:\n    var ys = xs\n    break\n",
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

#[test]
fn std_and_prelude_paths_match_with_either_separator() {
    for path in [
        "std/heap.cp",
        "/home/u/cpl/std/heap.cp",
        r"C:\cpl\std\heap.cp",
    ] {
        assert!(is_std_path(path), "{path}");
    }
    for path in ["heap.cp", "src/a.cp", "/home/u/std", r"C:\cpl\stdx\heap.cp"] {
        assert!(!is_std_path(path), "{path}");
    }
    assert!(is_prelude(r"C:\cpl\std\prelude.cp"));
    assert!(is_prelude("std/prelude.cp"));
}

#[test]
fn an_imported_body_is_checked_when_its_facts_are_asked_for() {
    let lib = "pub fn f(x: Int) -> Int:\n    x * 2\n";
    let mut db = Database::with_loader(Box::new(MapLoader(vec![(
        "lib.a".into(),
        "lib/a.cp".into(),
        lib.into(),
    )])));
    db.set_file("main.cp", "use lib.a.{f}\nprint(f(3))\n");
    db.analyze("main.cp").expect("analyzes");
    let at = lib.find("x * 2").unwrap() as u32;
    let body = Span::new(at, at + 5);
    let imported = (db.files.get("lib/a.cp"))
        .and_then(|f| f.module.clone())
        .expect("main imports lib.a");
    assert_eq!(
        imported.facts().type_of(body),
        None,
        "an import skips bodies"
    );
    let facts = db
        .body_facts("lib/a.cp", body)
        .expect("the body is checked");
    assert_eq!(facts.type_of(body), Some(&Ty::Int));
}

/// A getter whose struct's methods read it again is cut short while it is
/// worked out; the cut-short def must not be the one kept.
#[test]
fn a_getter_its_own_struct_reads_types_the_same_on_every_read() {
    let lib = "pub struct Wrap<S>(pub src: S)\n\
               @get pub fn wrap<S>(self: S) -> Wrap<S> = Wrap(self)\n\
               impl Wrap<S>:\n    pub fn twice(self) -> Wrap<Wrap<S>> = self.wrap\n    pub fn size(self) -> Int = 1\n";
    let mut db = Database::with_loader(Box::new(MapLoader(vec![(
        "lib.w".into(),
        "lib/w.cp".into(),
        lib.into(),
    )])));
    let main = "use lib.w.{Wrap, wrap}\nlet n: Str = [1, 2].wrap.size()\n";
    for _ in 0..2 {
        db.set_file("main.cp", main);
        let got = db.analyze("main.cp").unwrap();
        let messages: Vec<_> = got.diagnostics.iter().map(|d| d.message.clone()).collect();
        assert!(
            messages.iter().any(|m| m.contains("is an Int")),
            "{messages:?}"
        );
        db.set_file("main.cp", "use lib.w.{wrap}\nlet m = [1].wrap.twice()\n");
        db.analyze("main.cp").unwrap();
    }
}

fn silent_unknowns_of(src: &str) -> (Vec<String>, usize) {
    let mut db = Database::new();
    db.set_file("main.cp", src);
    let a = db.analyze("main.cp").unwrap();
    let spans = (a.silent_unknowns().iter())
        .map(|s| src[s.start as usize..s.end as usize].to_string())
        .collect();
    (spans, a.diagnostics.len())
}

#[test]
fn an_unknown_type_with_no_error_there_is_a_silent_unknown() {
    assert_eq!(
        silent_unknowns_of("print([])\n"),
        (vec!["[]".to_string()], 0)
    );
    let (reported, errors) = silent_unknowns_of("print(nope)\n");
    assert!(reported.is_empty() && errors > 0, "{reported:?}");
    let callee = "fn g(xs: List<Int>) -> Int:\n    xs.map(x => x * 2).sum()\nprint(g([1]))\n";
    assert_eq!(silent_unknowns_of(callee), (vec![], 0));
}

#[test]
fn a_generic_operator_impl_gives_its_result_at_the_operands_type_args() {
    let user = "struct W<T>(v: T)\nimpl Add for W<T>:\n    fn add(a: W<T>, b: W<T>) -> W<T> = a\nprint((W(2) + W(3)).v + 1)\n";
    assert_eq!(silent_unknowns_of(user), (vec![], 0));
    let mat = "domain V = 0..2\nlet adj = Mat([V, V] (i, j) => 1)\nprint((adj * adj).sum() / 2)\n";
    assert_eq!(silent_unknowns_of(mat), (vec![], 0));
}

#[test]
fn a_def_cut_by_its_own_cycle_does_not_replace_the_whole_one() {
    let lib = "@get pub fn boxed(self: Int) -> Boxed = Boxed(self)\n\n/// A box.\npub struct Boxed(pub v: Int):\n    pub fn get(self) -> Int = self.v\n";
    let mut db = Database::with_loader(Box::new(MapLoader(vec![(
        "lib.b".into(),
        "lib/b.cp".into(),
        lib.into(),
    )])));
    let src = "use lib.b.{boxed}\nlet b = 3.boxed\nlet got: Int = b.get()\n";
    let at = src.find("boxed\n").unwrap() as u32 + 1;
    db.set_file("a.cp", src);
    let first = db.hover("a.cp", at).map(|h| h.text);
    db.set_file("b.cp", src);
    let second = db.hover("b.cp", at).map(|h| h.text);
    assert!(
        first
            .as_deref()
            .is_some_and(|t| t.starts_with("boxed: Boxed")),
        "{first:?}"
    );
    assert_eq!(first, second);
    assert!(
        db.diagnostics("b.cp").is_empty(),
        "{:?}",
        db.diagnostics("b.cp")
    );
}

/// Jumping to a std method's definition analyzes std's module; a getter it
/// was cut short on must still type on the next file.
#[test]
fn a_std_getter_types_the_same_after_jumping_into_std() {
    let mut db = Database::new();
    let src = "let xs = [1, 2, 3]\nlet r = xs.ring\nlet s = xs.sorted()\nprint(r.windows(2).len, s.len)\n";
    let ring = src.find("ring").unwrap() as u32 + 1;
    let sorted = src.find("sorted").unwrap() as u32 + 1;
    db.set_file("a.cp", src);
    assert!(
        db.diagnostics("a.cp").is_empty(),
        "{:?}",
        db.diagnostics("a.cp")
    );
    let first = db.hover("a.cp", ring).map(|h| h.text);
    assert!(
        first
            .as_deref()
            .is_some_and(|t| t.starts_with("ring: Ring<Int, Int>")),
        "{first:?}"
    );
    for at in [ring, sorted] {
        let def = db.definition_location("a.cp", at);
        assert!(
            def.as_ref().is_some_and(|d| d.path.ends_with("seq.cp")),
            "{def:?}"
        );
    }
    db.set_file("b.cp", src);
    assert_eq!(db.hover("b.cp", ring).map(|h| h.text), first);
    assert!(
        db.diagnostics("b.cp").is_empty(),
        "{:?}",
        db.diagnostics("b.cp")
    );
}

#[test]
fn forms_typed_by_where_they_go_are_never_silent() {
    let src = "domain V = 0..2\nlet n: Int = read()\nvar g: [V, V]0..=9 = read()\nvar adj: [V]List<Int> = []\nlet e: List<Int> = []\nvar a = [1]\na.push(2)\nlet r = rand(1, 3)\nfor w in read<[n]Int>(): print(w)\nprint(g[V(0), V(1)], adj[V(0)].len, e.len, a.len, r)\n";
    assert_eq!(silent_unknowns_of(src), (vec![], 0));
}

#[test]
fn a_bound_reaches_its_supers_methods_typed() {
    // `Tall: Height<Int>` reaches `height` as `Int`; `Table<U>: Positional<U>`
    // reaches std's `Sized.len` and `Positional.nth`.
    let src = "trait Height<T>:\n    fn height(self) -> T\ntrait Tall: Height<Int>\nfn h<S: Tall>(s: S) -> Int = s.height()\ntrait Table<T>: Positional<T>\nfn mid<S: Table<U>, U>(s: S) -> U = s.nth(s.len / 2)\n";
    assert_eq!(silent_unknowns_of(src), (vec![], 0));
}

#[test]
fn a_place_under_a_lends_mut_index_beside_a_get_is_typed() {
    let src = "struct C:\n    var n: Int\n    fn bump(mut self, by: Int):\n        n += by\nstruct Node:\n    var c: C\n    var kids: List<Int>\nstruct Pool:\n    _xs: List<Node>\n    pub fn at(mut self, i: Int) -> lends mut Node:\n        lend mut _xs[i]\n    pub fn get(self, i: Int) -> Int = _xs[i].c.n\nvar p = Pool([Node(C(1), [2])])\np[0].c.bump(4)\np[0].c.n = 2\np[0].kids[0] += 1\np[0].kids.push(3)\n";
    assert_eq!(silent_unknowns_of(src), (vec![], 0));
    let read = format!("{src}let n: Int = p[0].abs()\n");
    assert_eq!(silent_unknowns_of(&read).1, 0);
}

#[test]
fn a_silent_unknown_names_its_cause() {
    let mut db = Database::new();
    db.set_file("main.cp", "print([])\n");
    let a = db.analyze("main.cp").unwrap();
    let causes: Vec<&str> = a.silent_unknown_causes().iter().map(|(_, c)| *c).collect();
    assert_eq!(causes, ["list"]);
}

#[test]
fn analyze_text_sets_the_file_and_reuses_an_unchanged_analysis() {
    let mut db = Database::new();
    let first = db.analyze_text("a.cp", "print(1)\n");
    assert!(Arc::ptr_eq(&first, &db.analyze_text("a.cp", "print(1)\n")));
    assert!(Arc::ptr_eq(&first, &db.analyze("a.cp").unwrap()));
    let edited = db.analyze_text("a.cp", "let x: Int = \"no\"\n");
    assert!(!edited.diagnostics.is_empty());
    assert_eq!(
        db.file_text("a.cp").as_deref(),
        Some("let x: Int = \"no\"\n")
    );
}

#[test]
fn a_generic_domain_position_has_a_typed_rank() {
    let src = "fn rank<V: Domain>(xs: List<V>) -> Int:\n    var s = 0\n    for u in xs: s += u.ord + u.idx\n    s\ndomain W = 0..4\nprint(rank([W(1), W(3)]))\n";
    assert_eq!(silent_unknowns_of(src), (vec![], 0));
}

#[test]
fn a_slice_walks_its_source_keys_typed() {
    let graph = "domain V = 0..3\nlet g = Graph.undirected([(V(0), V(1)), (V(1), V(2))])\nfor (a, w) in g[V(1)].items: print(a + w.ord)\n";
    assert_eq!(silent_unknowns_of(graph), (vec![], 0));
    let list = "let xs = [5, 6, 7]\nlet s = xs[1..]\nfor (i, x) in s.items: print(i + x)\n";
    assert_eq!(silent_unknowns_of(list), (vec![], 0));
}

#[test]
fn a_std_yields_method_and_argmin_are_typed() {
    let pairs = "let xs: List<Int> = [1, 2, 3]\nfor (a, b) in xs.cyclic_pairs(): print(a + b)\nfor (a, b) in xs.pairs(): print(a * b)\n";
    assert_eq!(silent_unknowns_of(pairs), (vec![], 0));
    let sized = "let n: Int = read()\ndomain T = 0..3\nlet b: [n]T = read()\nfor (a, c) in b.cyclic_pairs(): print(min(a, c))\n";
    assert_eq!(silent_unknowns_of(sized), (vec![], 0));
    let arg = "domain R = 0..3\nlet t: [R]Int = [R] r => 5 - r.ord\nlet m = t.argmin()!\nprint(t[m])\nlet xs = [3, 1]\nprint(xs.argmax()!)\nlet i: ?R = t.argmax()\nprint(i)\n";
    assert_eq!(silent_unknowns_of(arg), (vec![], 0));
}

#[test]
fn a_free_fold_and_a_stepped_range_type_by_the_seq_element() {
    let sorted =
        "let xs = [3, 1, 2]\nlet s = sorted([x * 2 for x in xs])\nfor v in s: print(v + s.len)\n";
    assert_eq!(silent_unknowns_of(sorted), (vec![], 0));
    let step = "let r = 0..10\nlet q = r.step(2)\nfor v in q: print(v + q.len)\nfor w in (0..9).step(3): print(w + 1)\n";
    assert_eq!(silent_unknowns_of(step), (vec![], 0));
}

#[test]
fn a_static_fns_untyped_lambda_takes_the_generic_its_body_binds() {
    let src = "domain V = 0..5\nfn up(v: V) -> ?V = None\nstruct Walk<V>(next: V -> ?V):\n    pub fn of(next: V -> ?V) -> Walk<V> = Walk<V>(next)\n    pub fn of2<W>(next: W -> ?W) -> Walk<W> = Walk<W>(next)\n    pub fn steps(self, v: V) -> Int = 0\nprint(Walk.of(v => up(v)).steps(V(3)))\nprint(Walk.of2(w => up(w)).steps(V(3)))\n";
    assert_eq!(silent_unknowns_of(src), (vec![], 0));
}

#[test]
fn a_struct_view_walks_its_own_seq_element_at_a_bound_and_a_join() {
    let base = "struct Twice<A, T>(a: A)\nimpl Seq<(T, T)> for Twice<A: Seq<T>, T>:\n    fn iter(self) -> yields (T, T):\n        for x in a: yield (x, x)\nfn twice<A: Seq<T>, T>(a: A) -> Twice<A, T> = Twice(a)\n";
    let join =
        format!("{base}let joined = [(0, 0)] + twice([4, 5])\nprint(joined.len, joined[2].1)\n");
    assert_eq!(silent_unknowns_of(&join), (vec![], 0));
    let nested = "struct Both<A, B, T, U>(a: A, b: B)\nimpl Seq<(T, U)> for Both<A: Seq<T>, B: Positional<U>, T, U>:\n    fn iter(self) -> yields (T, U):\n        var i = 0\n        for x in a:\n            if i == b.len: return\n            yield (x, b.nth(i))\n            i += 1\nimpl Sized for Both<A: Sized, B: Sized, T, U>:\n    @get fn len(self) -> Int = min(a.len, b.len)\nimpl Positional<(T, U)> for Both<A: Positional<T>, B: Positional<U>, T, U>:\n    fn nth(self, k: Int) -> (T, U) = (a.nth(k), b.nth(k))\nfn both<A: Seq<T>, B: Positional<U>, T, U>(a: A, b: B) -> Both<A, B, T, U> = Both(a, b)\nfor ((u, v), (l, r)) in both([(1, 2), (3, 4)], both([10, 20], [\"a\", \"b\"])):\n    print(u + v + l, r)\n";
    assert_eq!(silent_unknowns_of(nested), (vec![], 0));
}

#[test]
fn method_fits_imports_a_files_methods_once_per_analysis() {
    let mut db = Database::new();
    db.set_file("main.cp", "let xs = [3, 1]\nprint(xs)\n");
    let names = ["sorted", "sum", "no_such_method"];
    let first = db.method_fits("main.cp", &Ty::Int, &names);
    let analysis = db.analyze("main.cp").unwrap();
    let scope = db.memo_calling("main.cp", &analysis, &names);
    assert!(Arc::ptr_eq(
        &scope,
        &db.memo_calling("main.cp", &analysis, &names)
    ));
    assert_eq!(db.method_fits("main.cp", &Ty::Int, &names), first);
    assert!(!Arc::ptr_eq(
        &scope,
        &db.memo_calling("main.cp", &analysis, &names[..2])
    ));

    db.set_file("main.cp", "let xs = [3, 2, 1]\nprint(xs)\n");
    let edited = db.analyze("main.cp").unwrap();
    assert!(!Arc::ptr_eq(
        &scope,
        &db.memo_calling("main.cp", &edited, &names)
    ));
    db.remove_file("main.cp");
    assert!(db.callings.is_empty());
}

#[test]
fn a_for_over_windows_or_chunks_types_each_view() {
    for walk in [
        "xs.windows(2): print(q.sum())",
        "xs.chunks(2): print(q.len)",
        "List(xs).rekey(xs.keys).windows(2): print(List(q))",
        "xs.sorted().rekey(1..=3).windows(2): print(q.sum())",
    ] {
        let src = format!("let xs = [3, 1, 4]\nfor q in {walk}\n");
        assert_eq!(silent_unknowns_of(&src), (vec![], 0), "{src}");
    }
}
