use crate::{Database, DeclKind, Stats};
use cpl_diag::{Diagnostic, Span};

const ROADS: &str = "\
let n: 1..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
domain E = 0..m

struct Road:
    u, v: V
    len: 1..=1e9
    cost: 1..=1e9

let roads: [E]Road = read()
var dist: [V]?Int = none
var deg: [V]Int = 0

fn link(mut g: [V]List<(V, Int)>, r: Road):
    g[r.u].push((r.v, r.len))
    g[r.v].push((r.u, r.len))

fn longer(a: Road, b: Road) -> Bool:
    a.len > b.len

var g: [V]List<(V, Int)> = []
var total = 0
for e in E:
    let r = roads[e]
    deg[r.u] += 1
    total += r.cost
    link(mut g, r)
print(total)
";

fn check(src: &str) -> Vec<Diagnostic> {
    let mut db = Database::new();
    db.set_file("a.cp", src);
    db.diagnostics("a.cp")
}

fn codes(src: &str) -> Vec<&'static str> {
    check(src).iter().map(|d| d.code.0).collect()
}

fn one(src: &str) -> Diagnostic {
    let diags = check(src);
    assert_eq!(
        diags.len(),
        1,
        "expected one diagnostic for {src:?}, got {diags:#?}"
    );
    diags.into_iter().next().unwrap()
}

/// `ROADS` with `old` replaced by `new`.
fn roads_with(old: &str, new: &str) -> String {
    assert!(ROADS.contains(old), "{old:?} not in ROADS");
    ROADS.replacen(old, new, 1)
}

fn hover(src: &str, needle: &str) -> String {
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let at = src
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in source")) as u32;
    db.hover("a.cp", at)
        .unwrap_or_else(|| panic!("no hover at {needle:?}"))
        .text
}

// --- Diagnostics -----------------------------------------------------------

#[test]
fn a_correct_program_has_no_diagnostics() {
    assert_eq!(check(ROADS), vec![]);
}

#[test]
fn indexing_with_the_wrong_domain_names_both_domains() {
    let src = roads_with("deg[r.u] += 1", "deg[e] += 1");
    let d = one(&src);
    assert_eq!(d.code.0, "E0301");
    assert_eq!(d.message, "`deg` is indexed by V, but `e` is an E");
    assert_eq!(d.span.text(&src), "e");
    assert_eq!(d.help.as_deref(), Some("index with a V"));
}

#[test]
fn indexing_with_a_plain_number_says_how_to_convert() {
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nfor i in 1..=n:\n    print(a[i])\n");
    assert_eq!(d.message, "`a` is indexed by V, but `i` is a 1..=n");
    assert_eq!(
        d.help.as_deref(),
        Some("convert it with `V(i)`, or write `a.raw[i]` to index by number")
    );
}

#[test]
fn a_grid_takes_all_its_indices_in_one_bracket() {
    let d = one("let n: 1..=9 = read()\ndomain R = 1..=n\ndomain C = 1..=n\nvar g: [R, C]Int = 0\nprint(g[R.first])\n");
    assert_eq!(d.code.0, "E0305");
    assert_eq!(d.message, "`g` takes 2 indices (R, C), but 1 was given");
}

#[test]
fn derived_domains_are_distinct_from_their_base() {
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nvar pre: [V.bounds]Int = 0\nfor i in V:\n    pre[i] = 1\n");
    assert_eq!(d.message, "`pre` is indexed by V.bounds, but `i` is a V");
}

#[test]
fn unknown_names_suggest_the_closest_one() {
    let src = roads_with("print(total)", "print(totl)");
    let d = one(&src);
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0201", "no `totl` in scope")
    );
    assert_eq!(d.help.as_deref(), Some("did you mean `total`?"));
    assert_eq!(d.fixes[0].edits, vec![(d.span, "total".to_string())]);

    let d = one(&roads_with("r: Road):", "r: Raod):"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0204", "no type or domain named `Raod`")
    );
    assert_eq!(d.help.as_deref(), Some("did you mean `Road`?"));
}

#[test]
fn unknown_fields_suggest_the_closest_one() {
    let d = one(&roads_with("total += r.cost", "total += r.cots"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0203", "Road has no field or method `cots`")
    );
    assert_eq!(d.help.as_deref(), Some("did you mean `cost`?"));
    let d = one(&roads_with("total += r.cost", "total += r.weight"));
    assert_eq!(
        d.help.as_deref(),
        Some("its fields are `u`, `v`, `len`, `cost`")
    );
}

#[test]
fn calls_check_count_types_and_names() {
    let d = one(&roads_with("link(mut g, r)", "link(mut g, r, 3)"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0303", "`link` takes 2 arguments (g, r), but 3 were given")
    );
    let d = one(&roads_with("link(mut g, r)", "link(mut g)"));
    assert_eq!(
        d.message,
        "`link` takes 2 arguments (g, r), but `r` is missing"
    );
    let d = one(&roads_with("link(mut g, r)", "link(mut g, e)"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0301", "`link` takes a Road for `r`, but `e` is an E")
    );
    let d = one(&roads_with("link(mut g, r)", "link(mut g, road: r)"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0310", "`link` has no parameter named `road`")
    );
    let d = one(&roads_with("print(total)", "print(total(1))"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0304", "`total` is an Int, not a fn")
    );
    let d = one(&roads_with(
        "let r = roads[e]",
        "let r = Road(roads[e].u, roads[e].v, 1)",
    ));
    assert_eq!(
        d.message,
        "`Road` has 4 fields (u, v, len, cost), but `cost` is missing"
    );
}

#[test]
fn mut_is_required_where_declared_and_where_passed() {
    let src = roads_with("link(mut g, r)", "link(g, r)");
    let d = one(&src);
    assert_eq!(d.code.0, "E0402");
    assert_eq!(
        d.message,
        "`link` changes `g`, so the argument must be passed as `mut g`"
    );
    assert_eq!(
        d.fixes[0].edits,
        vec![(Span::point(d.span.start), "mut ".to_string())]
    );

    let d = one(&roads_with(
        "var g: [V]List<(V, Int)> = []",
        "let g: [V]List<(V, Int)> = []",
    ));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0401", "`g` is a `let`, so it can't be passed as `mut`")
    );
}

#[test]
fn immutable_bindings_cannot_be_assigned() {
    let src = roads_with("total += r.cost", "n = 5");
    let d = one(&src);
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0401", "`n` is a `let`, so it can't be assigned")
    );
    assert_eq!(
        d.help.as_deref(),
        Some("declare it with `var n` to change it")
    );

    let d = one("fn f(x: Int) -> Int:\n    x += 1\n    x\n");
    assert_eq!(
        d.message,
        "`x` is a parameter without `mut`, so it can't be assigned"
    );
    assert_eq!(d.labels[0].message, "declared here");

    let d = one("for i in 0..3:\n    i = 2\n");
    assert_eq!(d.message, "`i` is a loop variable, so it can't be assigned");
    assert_eq!(
        codes("fn f(mut x: Int) -> Int:\n    x += 1\n    x\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn declared_types_are_checked_against_values_and_returns() {
    let d = one("let x: Bool = 5\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`x` is declared a Bool, but `5` is an Int")
    );
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nlet v: V = 3\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0301", "`v` is declared a V, but `3` is an Int")
    );
    assert_eq!(d.help.as_deref(), Some("convert it with `V(3)`"));
    let d = one("fn f(x: Int) -> Bool:\n    if x > 0:\n        return 1\n    x == 0\n");
    assert_eq!(d.message, "`f` returns a Bool, but `1` is an Int");
    let d = one("fn sq(x: Int) -> Str = x * x\n");
    assert_eq!(d.message, "`sq` returns a Str, but `x * x` is an Int");
    let d = one("let x = 1 + true\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0309", "`+` can't combine an Int and a Bool")
    );
    let d = one("let n = 3\nif n:\n    print(n)\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0307", "the condition needs a Bool, but `n` is an Int")
    );
    assert_eq!(d.help.as_deref(), Some("compare it: `n != 0`"));
}

#[test]
fn top_level_names_have_one_definition_and_are_set_before_use() {
    let src = "type R = Int\ntype R = Bool\n";
    let d = one(src);
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0202", "`R` is already defined in this file")
    );
    assert_eq!(d.labels[0].span, Span::new(5, 6));

    let d = one("print(total)\nvar total = 0\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0205", "`total` is used here before it is set on line 2")
    );
    let d = one("let x = x + 1\n");
    assert_eq!(d.message, "`x` is used in its own definition");
    // A fn may use a name that is set further down: it runs when called.
    assert_eq!(
        codes("fn show():\n    print(total)\nvar total = 0\nshow()\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn what_the_checker_does_not_know_is_not_an_error() {
    let src = "\
use std.graph
let n: 1..=2e5 = read()
domain V = 1..=n
let g = Graph.read(n)
let d = dijkstra(g, V.first)
var best = 0
for v in V:
    best max= d[v] ?? 0
    best = best + g.adj(v).len * 2
print(\" \".join(d[v] ?? -1 for v in V), best, xs.sorted().rev())
";
    assert_eq!(check(src), vec![]);
}

// --- One root cause, one diagnostic ------------------------------------------

#[test]
fn a_syntax_error_does_not_become_type_errors() {
    assert_eq!(
        codes(&roads_with("link(mut g, r)", "link(mut g, r")),
        vec!["E0101"]
    );
    assert_eq!(
        codes(&roads_with("let r = roads[e]", "let r =")),
        vec!["E0102"]
    );
    assert_eq!(
        codes(&roads_with("link(mut g, r)", "link(mut g r)")),
        vec!["E0103"]
    );
    assert_eq!(
        codes(&roads_with("for e in E:", "for e in E")),
        vec!["E0103"]
    );
    assert_eq!(
        codes(&roads_with("total += r.cost", "total += r.")),
        vec!["E0108"]
    );
    assert_eq!(
        codes(&roads_with("var total = 0", "var total = ")),
        vec!["E0102"]
    );
}

#[test]
fn an_unknown_name_is_reported_once_and_does_not_spread() {
    let src = "let a = missing\nlet b = a + 1\nlet c: Bool = b\nprint(a.x, b[c], a(1))\n";
    assert_eq!(codes(src), vec!["E0201"]);
    let src = "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [W]Int = read()\nfor v in V:\n    print(a[v] + 1)\n";
    assert_eq!(codes(src), vec!["E0204"]);
}

#[test]
fn separate_mistakes_are_each_reported() {
    let src = roads_with("deg[r.u] += 1", "deg[e] += 1")
        .replace("print(total)", "print(totl)")
        .replace("link(mut g, r)", "link(g, r");
    assert_eq!(codes(&src), vec!["E0301", "E0101", "E0201"]);
}

#[test]
fn diagnostics_are_in_source_order_with_file_positions() {
    let src = "let ok = 1\n\nfn f(x: Int) -> Int:\n    y + 1\n\nprint(zz)\n";
    let diags = check(src);
    assert_eq!(
        diags.iter().map(|d| d.span.text(src)).collect::<Vec<_>>(),
        vec!["y", "zz"]
    );
}

// --- Incremental reuse ---------------------------------------------------------

/// Runs `edit` on a database that has already analyzed `ROADS`, and returns
/// the work the edit caused.
fn work_after(new_text: &str) -> (Stats, Vec<Diagnostic>) {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    assert!(db.diagnostics("a.cp").is_empty());
    let before = db.stats();
    db.set_file("a.cp", new_text);
    let diags = db.diagnostics("a.cp");
    let after = db.stats();
    let delta = Stats {
        files_lexed: after.files_lexed - before.files_lexed,
        items_parsed: after.items_parsed - before.items_parsed,
        items_checked: after.items_checked - before.items_checked,
        items_reused: after.items_reused - before.items_reused,
        defs_computed: after.defs_computed - before.defs_computed,
    };
    (delta, diags)
}

const ROADS_ITEMS: u64 = 14;

#[test]
fn the_first_analysis_does_all_the_work_once() {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    db.analyze("a.cp");
    let stats = db.stats();
    assert_eq!(stats.files_lexed, 1);
    assert_eq!(stats.items_parsed, ROADS_ITEMS);
    assert_eq!(stats.items_checked, ROADS_ITEMS);
    assert_eq!(stats.items_reused, 0);
    // Asking again without an edit does nothing at all.
    db.analyze("a.cp");
    db.hover("a.cp", 4);
    assert_eq!(db.stats(), stats);
}

#[test]
fn editing_a_fn_body_rechecks_only_that_fn() {
    let (work, diags) = work_after(&roads_with(
        "g[r.v].push((r.u, r.len))",
        "g[r.v].push((r.u, r.len + 1))",
    ));
    assert!(diags.is_empty());
    assert_eq!(work.items_parsed, 1);
    assert_eq!(work.items_checked, 1);
    assert_eq!(work.items_reused, ROADS_ITEMS - 1);
    assert_eq!(
        work.defs_computed, 1,
        "the signature is recomputed and found unchanged"
    );
}

#[test]
fn editing_a_signature_rechecks_exactly_its_users() {
    // `link` is used by the `for` loop and nowhere else.
    let (work, diags) = work_after(&roads_with(
        "fn link(mut g: [V]List<(V, Int)>, r: Road):",
        "fn link(mut g: [V]List<(V, Int)>, r: Road, w: Int):",
    ));
    assert_eq!(diags.len(), 1);
    assert_eq!(
        diags[0].message,
        "`link` takes 3 arguments (g, r, w), but `w` is missing"
    );
    assert_eq!(work.items_parsed, 1);
    assert_eq!(work.items_checked, 2);
    assert_eq!(work.items_reused, ROADS_ITEMS - 2);
}

#[test]
fn moving_items_reuses_everything_and_moves_their_diagnostics() {
    let broken = roads_with("print(total)", "print(totl)");
    let mut db = Database::new();
    db.set_file("a.cp", broken.as_str());
    let first = db.diagnostics("a.cp");
    let before = db.stats();
    let moved = format!("// a comment\n\n\n{broken}");
    db.set_file("a.cp", moved.as_str());
    let second = db.diagnostics("a.cp");
    let after = db.stats();
    assert_eq!(after.items_parsed, before.items_parsed);
    assert_eq!(after.items_checked, before.items_checked);
    assert_eq!(after.items_reused - before.items_reused, ROADS_ITEMS);
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].span, first[0].span.shift(15));
    assert_eq!(second[0].span.text(&moved), "totl");
}

#[test]
fn changing_a_bound_rechecks_what_depends_on_it() {
    // `n` feeds `domain V`, and V feeds everything indexed by V. `E`, `m`,
    // and `print(total)` do not depend on it.
    let (work, diags) = work_after(&roads_with(
        "let n: 1..=2e5 = read()",
        "let n: 1..=2000 = read()",
    ));
    assert!(diags.is_empty());
    assert_eq!(work.items_parsed, 1);
    assert!(work.items_checked >= 6, "{work:?}");
    assert!(work.items_reused >= 3, "{work:?}");
    assert_eq!(work.items_checked + work.items_reused, ROADS_ITEMS);
}

#[test]
fn renaming_a_definition_reports_its_users_and_undoing_it_clears_them() {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    assert!(db.diagnostics("a.cp").is_empty());
    db.set_file(
        "a.cp",
        roads_with("var total = 0", "var sum_cost = 0").as_str(),
    );
    let diags = db.diagnostics("a.cp");
    assert_eq!(
        diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
        vec!["E0201", "E0201"]
    );
    let before = db.stats();
    db.set_file("a.cp", ROADS);
    assert!(db.diagnostics("a.cp").is_empty());
    // Going back re-parses nothing: the old trees are still cached.
    assert_eq!(db.stats().items_parsed, before.items_parsed);
}

#[test]
fn every_keystroke_of_a_typed_line_matches_a_fresh_check() {
    // Type a new statement one character at a time. After each keystroke
    // the warm database must agree exactly with a cold one.
    let line = "    if longer(r, roads[E.first]) and deg[r.v] > 0: total += 1\n";
    let at = ROADS.find("    link(mut g, r)").unwrap();
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    db.analyze("a.cp");
    for end in 0..=line.len() {
        let text = format!("{}{}{}", &ROADS[..at], &line[..end], &ROADS[at..]);
        db.set_file("a.cp", text.as_str());
        let warm = db.diagnostics("a.cp");
        let mut cold = Database::new();
        cold.set_file("a.cp", text.as_str());
        assert_eq!(
            warm,
            cold.diagnostics("a.cp"),
            "after typing {:?}",
            &line[..end]
        );
        assert!(
            warm.len() <= 1,
            "one edit point, at most one diagnostic; got {warm:#?} after {:?}",
            &line[..end]
        );
    }
    assert!(db.diagnostics("a.cp").is_empty());
}

#[test]
fn memos_do_not_grow_without_bound() {
    let mut db = Database::new();
    for i in 0..5000 {
        db.set_file(
            "a.cp",
            format!("{ROADS}let extra_{i} = {i}\nprint(extra_{i})\n").as_str(),
        );
        db.analyze("a.cp");
    }
    let (syntax, defs, checks) = db.memo_counts();
    assert!(
        syntax < 1000 && defs < 2000 && checks < 1000,
        "{syntax} {defs} {checks}"
    );
}

#[test]
fn files_are_independent() {
    let mut db = Database::new();
    db.set_file("a.cp", "let shared = 1\nprint(shared)\n");
    db.set_file("b.cp", "print(shared)\n");
    assert!(db.diagnostics("a.cp").is_empty());
    assert_eq!(db.diagnostics("b.cp").len(), 1);
    db.remove_file("a.cp");
    assert!(db.analyze("a.cp").is_none());
}

// --- Hover, definitions, outline ---------------------------------------------

#[test]
fn hover_shows_type_domain_and_storage() {
    assert_eq!(
        hover(ROADS, "dist: [V]"),
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max"
    );
    assert_eq!(
        hover(ROADS, "roads[e]"),
        "roads: [E]Road\n  domain E = 0..m, m <= 2e5\n  storage: 16 bytes per element, 3.2 MB max"
    );
    assert_eq!(hover(ROADS, "n: 1..=2e5"), "n: 1..=2e5\n  storage: 4 bytes");
    assert_eq!(hover(ROADS, "total += r.cost"), "total: Int");
    assert_eq!(hover(ROADS, "r.cost"), "r: Road\n  storage: 16 bytes");
    assert_eq!(
        hover(ROADS, "cost\n    link"),
        "cost: 1..=1e9\n  storage: 4 bytes"
    );
    assert_eq!(
        hover(ROADS, "e in E"),
        "e: E\n  domain E = 0..m, m <= 2e5\n  storage: 4 bytes"
    );
}

#[test]
fn hover_shows_declarations() {
    assert_eq!(
        hover(ROADS, "link(mut g, r)"),
        "fn link(mut g: [V]List<(V, Int)>, r: Road)"
    );
    assert_eq!(
        hover(ROADS, "longer(a"),
        "fn longer(a: Road, b: Road) -> Bool"
    );
    assert_eq!(
        hover(ROADS, "Road = read()"),
        "struct Road\n  u: V\n  v: V\n  len: 1..=1e9\n  cost: 1..=1e9\n  storage: 16 bytes"
    );
    assert_eq!(
        hover(ROADS, "V = 1..=n"),
        "domain V = 1..=n\n  at most 2e5 positions (n <= 2e5)\n  storage: 4 bytes per position"
    );
    assert_eq!(
        hover("enum Dir = L | R | U | D\nlet d = L\n", "Dir"),
        "enum Dir = L | R | U | D"
    );
}

#[test]
fn storage_follows_the_declared_bounds() {
    let small = roads_with("let n: 1..=2e5 = read()", "let n: 1..=200 = read()");
    assert_eq!(
        hover(&small, "deg: [V]"),
        "deg: [V]Int\n  domain V = 1..=n, n <= 200\n  storage: 8 bytes per element, 1.6 KB max"
    );
    assert_eq!(
        hover(&small, "V = 1..=n"),
        "domain V = 1..=n\n  at most 200 positions (n <= 200)\n  storage: 1 byte per position"
    );
}

#[test]
fn hover_works_while_the_file_is_broken() {
    // An unclosed call two lines up, a half-typed statement on this line.
    let src =
        roads_with("link(mut g, r)", "link(mut g, r").replace("print(total)", "print(total +");
    assert_eq!(
        hover(&src, "dist: [V]"),
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max"
    );
    assert_eq!(hover(&src, "total +"), "total: Int");
    assert_eq!(
        hover(&src, "g, r\n"),
        "g: [V]List<(V, Int)>\n  domain V = 1..=n, n <= 2e5"
    );
    assert_eq!(
        hover(&src, "roads[e]"),
        "roads: [E]Road\n  domain E = 0..m, m <= 2e5\n  storage: 16 bytes per element, 3.2 MB max"
    );
    // Inside the item that has the syntax error itself.
    let src = roads_with("total += r.cost", "total += r.cost +");
    assert_eq!(hover(&src, "cost +"), "cost: 1..=1e9\n  storage: 4 bytes");
}

#[test]
fn hover_is_empty_off_a_name() {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    assert_eq!(
        db.hover("a.cp", ROADS.find("= read()").unwrap() as u32),
        None
    );
    assert_eq!(db.hover("a.cp", ROADS.len() as u32 + 100), None);
    assert_eq!(db.hover("missing.cp", 0), None);
}

#[test]
fn definitions_point_at_the_declaring_name() {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    let at = |needle: &str| ROADS.find(needle).unwrap() as u32;
    let span = db.definition("a.cp", at("total += r.cost")).unwrap();
    assert_eq!(span, Span::new(at("total = 0"), at("total = 0") + 5));
    let span = db.definition("a.cp", at("r.cost")).unwrap();
    assert_eq!(span.text(ROADS), "r");
    assert_eq!(span.start, at("r = roads[e]"));
    let span = db.definition("a.cp", at("link(mut g, r)")).unwrap();
    assert_eq!(span.start, at("link(mut g: "));
    let span = db.definition("a.cp", at("Road = read()")).unwrap();
    assert_eq!(span.start, at("Road:"));
}

#[test]
fn the_outline_lists_top_level_declarations() {
    let mut db = Database::new();
    db.set_file("a.cp", ROADS);
    let symbols = db.symbols("a.cp");
    let listed: Vec<(&str, DeclKind)> = symbols.iter().map(|s| (s.name.as_str(), s.kind)).collect();
    assert_eq!(
        listed,
        vec![
            ("n", DeclKind::Value),
            ("m", DeclKind::Value),
            ("V", DeclKind::Domain),
            ("E", DeclKind::Domain),
            ("Road", DeclKind::Struct),
            ("roads", DeclKind::Value),
            ("dist", DeclKind::Value),
            ("deg", DeclKind::Value),
            ("link", DeclKind::Fn),
            ("longer", DeclKind::Fn),
            ("g", DeclKind::Value),
            ("total", DeclKind::Value),
        ]
    );
    assert_eq!(symbols[8].name_span.text(ROADS), "link");
    assert!(
        symbols[8].span.text(ROADS).starts_with("fn link")
            && symbols[8].span.text(ROADS).ends_with("r.len))")
    );
}
