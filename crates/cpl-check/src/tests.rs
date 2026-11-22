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
fn a_domain_mismatch_labels_where_each_index_space_comes_from() {
    let src = "domain V = 1..=9\nfn f(m: 1..=9, b: [V]Int) -> Int:\n    return b[m]\n";
    let d = one(src);
    let labels: Vec<_> = d
        .labels
        .iter()
        .map(|l| (l.span.text(src), l.message.as_str()))
        .collect();
    assert_eq!(
        labels,
        [
            ("b", "`b` is declared over V here"),
            ("m", "`m` is declared as a 1..=9 here")
        ]
    );
}

#[test]
fn indexing_with_a_plain_number_says_how_to_convert() {
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nfor i in 1..=n:\n    print(a[i])\n");
    assert_eq!(d.message, "`a` is indexed by V, but `i` is a 1..=n");
    assert_eq!(
        d.help.as_deref(),
        Some("convert it with `V(i)`, or write `a.raw[i]` to index by number")
    );
    let src = "let n: 1..=9 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nfor i in 1..=n:\n    print(a[i])\n";
    let mut fixed = src.to_string();
    for (span, text) in d.fixes[0].edits.iter().rev() {
        fixed.replace_range(span.start as usize..span.end as usize, text);
    }
    assert!(fixed.ends_with("print(a[V(i)])\n"), "{fixed}");
}

#[test]
fn a_grid_takes_all_its_indices_in_one_bracket() {
    let d = one("let n: 1..=9 = read()\ndomain R = 1..=n\ndomain C = 1..=n\nvar g: [R, C]Int = 0\nfor c in C: print(g[c])\n");
    assert_eq!(d.code.0, "E0305");
    assert_eq!(d.message, "`g` takes 2 indices (R, C), but 1 was given");
}

#[test]
fn derived_domains_are_distinct_from_their_base() {
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nvar pre: [V.cuts]Int = 0\nfor i in V:\n    pre[i] = 1\n");
    assert_eq!(d.message, "`pre` is indexed by V.cuts, but `i` is a V");
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
fn a_capitalized_name_is_never_suggested_a_lowercase_local() {
    let d = one("let rope = 1\nprint(Rope)\nprint(rope)\n");
    assert_eq!(d.code.0, "E0201");
    assert!(
        !d.help.as_deref().unwrap_or("").contains("`rope`"),
        "{:?}",
        d.help
    );
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
        ("E0302", "`link` takes a Road for `r`, but `e` is an E")
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
        ("E0403", "`g` is a `let`, so it can't be passed as `mut`")
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
    let d = one("let n: 1..=9 = read()\ndomain V = 1..=n\nlet i: Int = 3\nlet v: V = i\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`v` is declared a V, but `i` is an Int")
    );
    assert_eq!(d.help.as_deref(), Some("convert it with `V(i)`"));
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
fn names_a_missing_module_may_have_brought_are_not_errors() {
    let src = "\
use kit.graph.*
let n: 1..=2e5 = read()
domain V = 1..=n
let g = Graph.read(n)
let d = dijkstra(g, V.first)
var best = 0
for v in V:
    best max= d[v] ?? 0
    best = best + g.adj(v).len * 2
print(\" \".join(d[v] ?? -1 for v in V), best, xs.sorted().rev)
";
    // The module is the one thing wrong; what it would have defined is
    // not reported a second time.
    let diag = one(src);
    assert_eq!(diag.code.0, "E0206");
    assert_eq!(diag.message, "no module `kit.graph`");
    assert_eq!(
        &src[diag.span.start as usize..diag.span.end as usize],
        "kit.graph"
    );
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
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max (none is a spare value: no flag)"
    );
    assert_eq!(
        hover(ROADS, "roads[e]"),
        "roads: [E]Road\n  domain E = 0..m, m <= 2e5\n  storage: 16 bytes per element, 3.2 MB max (fields at 0, 4, 8, 12)"
    );
    assert_eq!(hover(ROADS, "n: 1..=2e5"), "n: 1..=2e5\n  storage: 4 bytes");
    assert_eq!(hover(ROADS, "total += r.cost"), "total: Int");
    assert_eq!(
        hover(ROADS, "r.cost"),
        "r: Road\n  storage: 16 bytes (fields at 0, 4, 8, 12)"
    );
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
        "struct Road\n  u: V  // at 0, 4 bytes\n  v: V  // at 4, 4 bytes\n  len: 1..=1e9  // at 8, 4 bytes\n  cost: 1..=1e9  // at 12, 4 bytes\n  storage: 16 bytes, aligned to 4"
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
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max (none is a spare value: no flag)"
    );
    assert_eq!(hover(&src, "total +"), "total: Int");
    assert_eq!(
        hover(&src, "g, r\n"),
        "g: [V]List<(V, Int)>\n  domain V = 1..=n, n <= 2e5\n  storage: grows on the heap; (V, Int): 16 bytes each"
    );
    assert_eq!(
        hover(&src, "roads[e]"),
        "roads: [E]Road\n  domain E = 0..m, m <= 2e5\n  storage: 16 bytes per element, 3.2 MB max (fields at 0, 4, 8, 12)"
    );
    // Inside the item that has the syntax error itself.
    let src = roads_with("total += r.cost", "total += r.cost +");
    assert_eq!(hover(&src, "cost +"), "cost: 1..=1e9\n  storage: 4 bytes");
}

#[test]
fn hover_shows_the_doc_comment_and_members_jump_to_their_declaration() {
    let src = "\
/// A point on the grid.
/// Both coordinates are 0-based.
struct P:
    x: Int
    y: Int

    /// The Manhattan length.
    @inline
    fn norm(self) -> Int:
        abs(x) + abs(y)

/// Twice `v`.
fn twice(v: Int) -> Int:
    2 * v

let p = P(1, 2)
print(twice(p.norm()) + p.x)
";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let at = |needle: &str| src.find(needle).unwrap() as u32;
    let hover = db.hover("a.cp", at("twice(p")).unwrap();
    assert_eq!(hover.doc.as_deref(), Some("Twice `v`."));
    let hover = db.hover("a.cp", at("P(1")).unwrap();
    assert_eq!(
        hover.doc.as_deref(),
        Some("A point on the grid.\nBoth coordinates are 0-based.")
    );
    let hover = db.hover("a.cp", at("norm()")).unwrap();
    assert_eq!(hover.doc.as_deref(), Some("The Manhattan length."));
    assert_eq!(db.hover("a.cp", at("p = P")).unwrap().doc, None);

    let def = db.definition("a.cp", at("norm()")).unwrap();
    assert_eq!(def, Span::new(at("norm(self)"), at("norm(self)") + 4));
    let def = db.definition("a.cp", at("x)\n")).unwrap();
    assert_eq!(def, Span::new(at("x: Int"), at("x: Int") + 1));
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
fn a_type_named_inside_a_fn_or_its_own_body_has_a_definition() {
    let src = "struct Run:\n    len: Int\n    fn grown() -> Run = Run(len + 1)\nfn main():\n    let n = read<1..=9>()\n    domain R = 0..n\n    let best: [R]?Int = [R] r => none\n    print(best[R.first!], Run(1).grown().len)\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let at = |needle: &str| src.find(needle).unwrap() as u32;
    let span = db.definition("a.cp", at("R]?Int")).unwrap();
    assert_eq!(span.start, at("R = 0..n"));
    let span = db.definition("a.cp", at("Run = Run")).unwrap();
    assert_eq!(span.start, at("Run:"));
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

// --- Generics, traits, modules, lends ------------------------------------------

/// Modules held in memory, by dotted name: `("std.algebra", "pub struct ...")`.
struct Modules(Vec<(&'static str, &'static str)>);

impl crate::Loader for Modules {
    fn load(&self, _from: &str, module: &[String]) -> Option<(String, String)> {
        let name = module.join(".");
        let (_, text) = self.0.iter().find(|(n, _)| *n == name)?;
        Some((format!("{name}.cp"), text.to_string()))
    }
}

fn check_with(modules: &[(&'static str, &'static str)], src: &str) -> Vec<Diagnostic> {
    let mut db = Database::with_loader(Box::new(Modules(modules.to_vec())));
    db.set_file("a.cp", src);
    db.diagnostics("a.cp")
}

#[test]
fn editing_a_file_leaves_the_modules_it_imports_analyzed() {
    let modules = [("lib.m", "pub fn one() -> Int = 1\n")];
    let mut db = Database::with_loader(Box::new(Modules(modules.to_vec())));
    db.set_file("a.cp", "use lib.m.{one}\nprint(one())\n");
    assert!(db.diagnostics("a.cp").is_empty());
    let before = db.stats().files_lexed;
    db.set_file("a.cp", "use lib.m.{one}\nprint(one() + 1)\n");
    assert!(db.diagnostics("a.cp").is_empty());
    assert_eq!(db.stats().files_lexed - before, 1);

    db.set_file("lib.m.cp", "pub fn two() -> Int = 2\n");
    let diags = db.diagnostics("a.cp");
    assert!(!diags.is_empty(), "an edit to an import reaches its users");
}

fn messages(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.message.as_str()).collect()
}

const DSU: &str = "\
pub struct Dsu<V>:
    up: [V]V
    parts: Int

    pub fn new() -> Dsu<V>:
        Dsu([V] v => v, V.K)

    pub fn find(mut self, v: V) -> V:
        if up[v] != v: up[v] = self.find(up[v])
        up[v]

    pub @get fn count(self) -> Int = parts

fn hidden() -> Int = 1
";

#[test]
#[ignore = "spec/conformance/core-05/a-user-method-on-self-does-not-change-its-receiver.cp"]
fn a_generic_struct_is_used_with_the_types_given_there() {
    let src = "\
let n: 1..=9 = read()
domain City = 1..=n
struct Pair<A, B>(first: A, second: B):
    fn swap(self) -> Pair<B, A> = Pair(second, first)
let p = Pair(City.first!, 'x')
let c: City = p.first
let q = p.swap()
let d: Char = q.first
";
    assert_eq!(check(src), vec![]);
    // `p.first!` is a `City`, not a `Char`: the argument fixed `A`.
    let diag = one(&src.replace("let c: City = p.first!", "let c: Char = p.first!"));
    assert_eq!(
        diag.message,
        "`c` is declared a Char, but `p.first!` is a City"
    );
    // The swapped pair's parameters swapped too.
    let diag = one(&src.replace("let d: Char = q.first", "let d: City = q.first"));
    assert_eq!(
        diag.message,
        "`d` is declared a City, but `q.first` is a Char"
    );
}

#[test]
fn a_generic_fn_reads_its_type_parameters_off_the_arguments() {
    let src = "\
fn pick<T>(a: T, b: T, first: Bool) -> T:
    if first: a else: b
let x: Int = pick(1, 2, true)
let s: Str = pick(\"a\", \"b\", false)
let y = pick::<Int>(3, 4, true)
";
    assert_eq!(check(src), vec![]);
    let diag = one(&src.replace("pick(1, 2, true)", "pick(1, \"two\", true)"));
    assert_eq!(
        diag.message,
        "`pick` takes an Int for `b`, but `\"two\"` is a Str"
    );
    let diag = one(&src.replace(
        "let x: Int = pick(1, 2, true)",
        "let x: Str = pick(1, 2, true)",
    ));
    assert_eq!(
        diag.message,
        "`x` is declared a Str, but `pick(1, 2, true)` is an Int"
    );
    let diag = one(&src.replace("pick::<Int>(3", "pick::<Int, Int>(3"));
    assert_eq!(
        diag.message,
        "`pick` takes 1 type argument, but 2 were given"
    );
}

#[test]
fn new_is_the_constructor_and_get_methods_read_as_fields() {
    let src = format!(
        "{DSU}let n: 1..=9 = read()\ndomain V = 1..=n\nvar d = Dsu<V>()\nlet r: V = d.find(V.first!)\nlet k: Int = d.count\n"
    );
    assert_eq!(check(&src), vec![]);
    // Outside the struct, `Dsu(..)` is `new`, which takes nothing.
    let diag = one(&src.replace("Dsu<V>()", "Dsu<V>(1, 2)"));
    assert_eq!(diag.message, "`Dsu<V>` takes 0 arguments, but 2 were given");
    let diag = one(&src.replace("let k: Int = d.count", "let k: Str = d.count"));
    assert_eq!(
        diag.message,
        "`k` is declared a Str, but `d.count` is an Int"
    );
}

#[test]
fn traits_bounds_and_impls_are_checked() {
    let src = "\
trait Shape:
    fn area(self) -> Int
    fn name(self) -> Str = \"shape\"
struct Sq(side: Int)
impl Shape for Sq:
    fn area(self) -> Int = side * side
fn total<T: Shape + Ord>(xs: List<T>) -> Int:
    sum(x.area() for x in xs)
print(Sq(3).area(), Sq(3).name())
";
    assert_eq!(check(src), vec![]);
    let diag = one(&src.replace(
        "    fn area(self) -> Int = side * side\n",
        "    fn name(self) -> Str = \"sq\"\n",
    ));
    assert_eq!(diag.message, "`impl Shape for Sq` is missing `area`");
    let diag = one(&src.replace(
        "fn area(self) -> Int = side * side",
        "fn area(self) -> Int = side * side\n    fn perimeter(self) -> Int = 4 * side",
    ));
    assert_eq!(diag.message, "`Shape` has no fn `perimeter`");
    let diag = one(&src.replace("T: Shape + Ord", "T: Shape + Sq"));
    assert_eq!(diag.message, "`Sq` is not a trait");
    let diag = one(&src.replace("T: Shape + Ord", "T: Shwo + Ord"));
    assert_eq!(diag.message, "no trait named `Shwo`");
    let twice = format!("{src}impl Shape for Sq:\n    fn area(self) -> Int = 0\n");
    assert_eq!(
        one(&twice).message,
        "`impl Shape for Sq` is already defined in this file"
    );
}

#[test]
fn an_operator_on_a_struct_is_its_impl() {
    let src = "\
struct Pt(x: Int, y: Int)
impl Add for Pt:
    fn add(a: Pt, b: Pt) -> Pt = Pt(a.x + b.x, a.y + b.y)
let p: Pt = Pt(1, 2) + Pt(3, 4)
";
    assert_eq!(check(src), vec![]);
    let diag = one(&src.replace("Pt(1, 2) + Pt(3, 4)", "Pt(1, 2) - Pt(3, 4)"));
    assert_eq!(diag.code.0, "E0606");
    assert_eq!(
        diag.message,
        "`-` needs `impl Sub for Pt`, and there is none"
    );
}

#[test]
fn imports_bring_public_names_and_their_types() {
    let src = "\
use std.dsu.{Dsu}
let n: 1..=9 = read()
domain V = 1..=n
var d = Dsu<V>()
let r: V = d.find(V.first!)
";
    let modules = [("std.dsu", DSU)];
    assert_eq!(check_with(&modules, src), vec![]);
    let diags = check_with(&modules, &src.replace("let r: V", "let r: Str"));
    assert_eq!(
        messages(&diags),
        ["`r` is declared a Str, but `d.find(V.first!)` is a V"]
    );
    // Glob, whole-module and aliased imports.
    for (import, call) in [
        ("use std.dsu.*", "Dsu<V>()"),
        ("use std.dsu", "dsu.Dsu<V>()"),
        ("use std.dsu as sets", "sets.Dsu<V>()"),
    ] {
        let src = src
            .replace("use std.dsu.{Dsu}", import)
            .replace("Dsu<V>()", call);
        assert_eq!(check_with(&modules, &src), vec![], "{import}");
    }
}

#[test]
fn a_bad_import_is_one_error_on_the_use() {
    let modules = [("std.dsu", DSU)];
    let diags = check_with(&modules, "use std.dsu.{Dsv}\n");
    assert_eq!(messages(&diags), ["`std.dsu` has no `Dsv`"]);
    assert_eq!(diags[0].help.as_deref(), Some("did you mean `Dsu`?"));
    let diags = check_with(&modules, "use std.dsu.{hidden}\n");
    assert_eq!(messages(&diags), ["`hidden` is private to `std.dsu`"]);
    assert_eq!(diags[0].code.0, "E0207");
    let diags = check_with(&modules, "use std.dsv.{Dsu}\nlet d = Dsu()\n");
    assert_eq!(
        messages(&diags),
        ["no module `std.dsv`, and not in the stdlib (yet)"]
    );
    // A name that was not imported is not in scope, even though the
    // module that defines it was loaded.
    let diags = check_with(&modules, "use std.dsu.{Dsu}\nprint(hidden())\n");
    assert_eq!(
        messages(&diags),
        ["no `hidden` in scope, and not in the stdlib (yet)"]
    );
    let cycle = [
        ("a.x", "use a.y.{g}\npub fn f() -> Int = 1\n"),
        ("a.y", "use a.x.{f}\npub fn g() -> Int = 2\n"),
    ];
    let diags = check_with(&cycle, "use a.x.{f}\nprint(f())\n");
    assert_eq!(diags, vec![], "the cycle is reported in the modules");
}

#[test]
fn editing_a_module_rechecks_what_uses_it() {
    let mut db = Database::with_loader(Box::new(Modules(vec![])));
    db.set_file("lib.m.cp", "pub fn f() -> Int = 1\n");
    struct Open;
    impl crate::Loader for Open {
        fn load(&self, _from: &str, module: &[String]) -> Option<(String, String)> {
            // The editor holds the module; what the loader reads is stale.
            (module.join(".") == "lib.m").then(|| ("lib.m.cp".to_string(), "stale".to_string()))
        }
    }
    let mut db2 = Database::with_loader(Box::new(Open));
    db2.set_file("lib.m.cp", "pub fn f() -> Int = 1\n");
    db2.set_file("a.cp", "use lib.m.{f}\nlet x: Int = f()\n");
    assert_eq!(db2.diagnostics("a.cp"), vec![]);
    db2.set_file("lib.m.cp", "pub fn f() -> Str = \"one\"\n");
    assert_eq!(
        messages(&db2.diagnostics("a.cp")),
        ["`x` is declared an Int, but `f()` is a Str"]
    );
    drop(db);
}

const ARRAY: &str = "\
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
let l: V = read()
let r: V = read()
";

#[test]
fn two_mut_lends_of_one_array_need_to_be_apart() {
    let lent = |body: &str| check(&format!("{ARRAY}{body}"));
    // Known apart: a guard, an assert, an early return, a strict order.
    for body in [
        "if l != r:\n    swap(mut a[l], mut a[r])\n",
        "assert l != r\nswap(mut a[l], mut a[r])\n",
        "fn go(i: V, j: V):\n    if i == j: return\n    swap(mut a[i], mut a[j])\n",
        "fn go(i: V, j: V):\n    assert i < j\n    swap(mut a[i], mut a[j])\n",
        "fn two(mut xs: [V]Int, mut ys: [V]Int):\n    xs[V.first!] += 1\ntwo(mut a[l.l..r.l], mut a[r.l..r.r])\n",
    ] {
        assert_eq!(lent(body), vec![], "{body}");
    }
    let diag = &lent("swap(mut a[l], mut a[r])\n")[0];
    assert_eq!(diag.code.0, "E0601");
    assert_eq!(
        diag.message,
        "`a[l]` and `a[r]` may be the same storage, and both are lent `mut` to this call: Array.at lends disjoint parts only when `i != j`, and that is not known here"
    );
    // The fact is gone once a name in it is assigned.
    let diags = lent("var i = l\nif i != r:\n    i = r\n    swap(mut a[i], mut a[r])\n");
    assert_eq!(diags.len(), 1, "{diags:#?}");
    // And at the top of a loop that assigns it.
    let diags = lent(
        "var i = l\nassert i != r\nwhile a[l] < 100:\n    swap(mut a[i], mut a[r])\n    i = r\n",
    );
    assert_eq!(diags.len(), 1, "{diags:#?}");
    let diags = lent(
        "fn two(mut xs: [V]Int, mut ys: [V]Int):\n    xs[V.first!] += 1\ntwo(mut a, mut a[l..=r])\n",
    );
    assert!(
        diags[0].message.ends_with("one contains the other"),
        "{diags:#?}"
    );
}

#[test]
fn a_let_bound_slice_keeps_its_array_lent_until_its_last_use() {
    let lent = |body: &str| check(&format!("{ARRAY}{body}"));
    assert_eq!(
        lent("let part = a[l..=r]\nprint(sum(part))\na[l] = 0\n"),
        vec![]
    );
    let diags = lent("let part = a[l..=r]\na[l] = 0\nprint(part[l])\n");
    assert_eq!(diags.len(), 1, "{diags:#?}");
    assert_eq!(diags[0].code.0, "E0602");
    assert!(diags[0]
        .message
        .starts_with("`a[l]` can't be changed here: `a` is lent to `part`"));
    // Reading is fine while the slice is only read; not while it changes.
    assert_eq!(
        lent("let part = a[l..=r]\nprint(a[l])\nprint(part[l])\n"),
        vec![]
    );
    let diags = lent("let part = a[l..=r]\npart.sort()\nprint(a[l])\npart.reverse()\n");
    assert!(
        diags[0].message.contains("is mutably lent to `part`"),
        "{diags:#?}"
    );
    // The same inside one block.
    let diags = lent("fn f():\n    let part = a[l..=r]\n    a[l] = 0\n    print(part[l])\n");
    assert_eq!(diags.len(), 1, "{diags:#?}");
}

#[test]
fn a_lent_slice_is_not_stored_assigned_or_returned() {
    for (body, what) in [
        ("var keep = a[l..=r]\n", "`var keep`"),
        ("var b: [V]Int = 0\nb = a[l..=r]\n", "this assignment"),
        (
            "var parts: List<[V]Int> = []\nparts.push(a[l..=r])\n",
            "`.push(...)`",
        ),
        (
            "fn top(xs: [V]Int) -> [V]Int:\n    xs[l..=r]\n",
            "returning it",
        ),
        (
            "fn top(xs: [V]Int) -> [V]Int:\n    return xs[l..=r]\n",
            "`return`",
        ),
    ] {
        let diags = check(&format!("{ARRAY}{body}"));
        assert_eq!(diags.len(), 1, "{body}: {diags:#?}");
        assert_eq!(diags[0].code.0, "E0603");
        assert!(
            diags[0]
                .message
                .starts_with(&format!("{what} would keep the lent slice")),
            "{}",
            diags[0].message
        );
    }
    // A copy is an owned value.
    assert_eq!(
        check(&format!("{ARRAY}var keep = a[l..=r].copy()\n")),
        vec![]
    );
}

const BAG: &str = "\
let n: 2..=9 = read()
domain V = 1..=n
struct Bag:
    cnt: [V]Int
    fn at(self, i: V) -> lends mut Int:
        law disjoint(i, j) = i != j
        lend mut cnt[i]
    fn other(self, i: V) -> lends mut Int:
        lend mut cnt[i]
    fn peek(self, i: V) -> lends Int:
        lend cnt[i]
fn move(mut from: Int, mut to: Int):
    to += from
    from = 0
let zero: [V]Int = 0
var b = Bag(zero)
let i: V = read()
let j: V = read()
";

#[test]
fn a_projection_lends_by_its_law() {
    let lent = |body: &str| check(&format!("{BAG}{body}"));
    assert_eq!(lent("if i != j:\n    move(mut b.at(i), mut b.at(j))\nb.at(i) += 1\nlet x: Int = b.peek(i)\n"), vec![]);
    let diags = lent("move(mut b.at(i), mut b.at(j))\n");
    assert!(
        diags[0].message.ends_with(
            "Bag.at lends disjoint parts only when `(i != j)`, and that is not known here"
        ),
        "{diags:#?}"
    );
    let diags = lent("if i != j:\n    move(mut b.other(i), mut b.other(j))\n");
    assert!(
        diags[0]
            .message
            .ends_with("Bag.other declares no `law disjoint`, so two of its lends always conflict"),
        "{diags:#?}"
    );
    let diags = lent("if i != j:\n    move(mut b.at(i), mut b.other(j))\n");
    assert!(
        diags[0]
            .message
            .contains("different projections of the same value"),
        "{diags:#?}"
    );
    let diags = lent("b.peek(i) = 3\n");
    assert_eq!(
        messages(&diags),
        ["`b.peek(i)` is lent read-only: `Bag.peek` is declared `lends Int`"]
    );
}

#[test]
fn a_projection_ends_in_one_lend_of_a_part_of_self() {
    let with = |old: &str, new: &str| {
        assert!(BAG.contains(old));
        check(&BAG.replacen(old, new, 1))
    };
    // v2.2: the body ends with the place itself.
    let diags = with("        lend cnt[i]\n", "        cnt[i]\n");
    assert_eq!(messages(&diags), Vec::<&str>::new());
    let diags = with("        lend cnt[i]\n", "        let c = cnt[i]\n");
    assert_eq!(
        messages(&diags),
        ["`peek` is declared `lends`, so every path through it must reach one `lend place`"]
    );
    let diags = with("        lend cnt[i]\n", "        lend mut cnt[i]\n");
    assert_eq!(
        messages(&diags),
        ["`peek` is declared `lends` without `mut`, so it can't `lend mut`"]
    );
    let diags = with("        lend cnt[i]\n", "        lend zero[i]\n");
    assert_eq!(
        messages(&diags),
        ["a projection may only lend part of `self`; `zero[i]` is not reached through a field"]
    );
    let diags = check(&format!("{BAG}fn f(k: V):\n    lend zero[k]\n"));
    assert_eq!(diags.len(), 1, "{diags:#?}");
    assert_eq!(diags[0].code.0, "E0604");
}

#[test]
fn code_after_a_lend_is_the_write_back_and_needs_mut_self() {
    let tally = |receiver: &str| {
        format!(
            "let n: 2..=9 = read()\ndomain V = 1..=n\nstruct Tally:\n    cnt: [V]Int\n    distinct: Int\n    fn at({receiver}, v: V) -> lends mut Int:\n        if cnt[v] != 0: distinct -= 1\n        lend mut cnt[v]\n        if cnt[v] != 0: distinct += 1\n"
        )
    };
    assert_eq!(check(&tally("mut self")), vec![]);
    let diags = check(&tally("self"));
    assert!(messages(&diags)
        .contains(&"code after the `lend` changes `self`, so `at` takes `mut self`"));
    // One `lend`, and a statement of the body itself.
    let twice = tally("mut self").replace(
        "        lend mut cnt[v]\n",
        "        lend mut cnt[v]\n        lend mut cnt[v]\n",
    );
    assert_eq!(
        messages(&check(&twice)),
        ["`at` is declared `lends`, so every path through it must reach one `lend place`"]
    );
    let nested = tally("mut self").replace(
        "        lend mut cnt[v]\n",
        "        if distinct > 0:\n            lend mut cnt[v]\n",
    );
    assert_eq!(check(&nested)[0].code.0, "E0604");
}

#[test]
fn boundaries_sit_on_either_side_of_a_position() {
    let src = format!("{ARRAY}var pre: [V.cuts]Int = 0\nprint(pre[r.r] - pre[l.l], pre[r.r] - pre[r.l(2)], sum(a[l.l..r.r]))\n");
    assert_eq!(check(&src), vec![]);
    let diag = one(&format!("{ARRAY}print(a[l.r])\n"));
    assert_eq!(diag.message, "`a` is indexed by V, but `l.r` is a V.cuts");
    assert!(diag
        .help
        .as_deref()
        .unwrap()
        .contains("use `.l` / `.r` for the boundaries"));
}

#[test]
fn division_and_remainder_are_euclidean() {
    // `x % 7` is `0..=6` whatever the sign of `x`: it indexes a 7-array.
    assert_eq!(
        check("domain D = 0..7\nlet x: -100..=100 = read()\nlet r: 0..=6 = x % 7\nlet q: -15..=14 = x / 7\nprint(D(r), q)\n"),
        vec![]
    );
    let hover = |src: &str, name: &str| {
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let at = src.find(name).unwrap() as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    let text = hover("domain D = 0..(-7 % 3)\n", "D").unwrap();
    assert!(text.contains("2"), "-7 % 3 is 2, not -1: {text}");
    let text = hover("domain E = 0..(0 - (-7 / 2))\n", "E").unwrap();
    assert!(text.contains("4"), "-7 / 2 is -4, not -3: {text}");
}

#[test]
fn a_step_is_optional_until_it_is_handled() {
    let head = "let n: 2..=9 = read()\ndomain I = 1..=n\nlet a: [I]Int = read()\nfn show(i: I): print(i)\n";
    let index = one(&format!("{head}for i in I: print(a[i + 1])\n"));
    assert_eq!(index.message, "`a` is indexed by I, but `i + 1` is an Int");
    assert!(index.help.unwrap().contains("(i + 1)!"));
    let arg = one(&format!("{head}for i in I: show(i - 1)\n"));
    assert_eq!(
        arg.message,
        "expected I, got ?I: `i - 1` may step outside I"
    );
    // Bound, clamped or asserted, the step is a position again; and two
    // positions of one domain are an `Int` apart.
    for body in [
        "    if let j = i + 1: print(a[j])\n",
        "    print(a[i + 1 ?? I.last])\n",
        "    print(a[(i + 1)!])\n",
        "    show((i - 1)!)\n",
        "    let d: Int = I.last - i\n    print(d)\n",
    ] {
        let src = format!("{head}for i in I:\n{body}");
        assert!(codes(&src).is_empty(), "{src}: {:?}", check(&src));
    }
}

#[test]
fn a_domain_holds_at_most_an_i32_of_values() {
    let big = one("domain V = 0..=3000000000\n");
    assert!(big
        .message
        .starts_with("a domain holds at most 2^31 - 1 values"));
    assert!(codes("domain V = 0..2147483647\n").is_empty());
}

#[test]
fn a_range_or_span_stays_in_one_domain() {
    let head =
        "let n: 2..=9 = read()\ndomain I = 1..=n\ndomain J = 1..=n\nlet a: [I]Int = read()\n";
    let msg = |body: &str| one(&format!("{head}{body}")).message;
    assert_eq!(
        msg("print(sum(a[I.first!..J.last!]))\n"),
        "range between I and J"
    );
    assert_eq!(
        msg("print(span(I.first!, J.last!).len)\n"),
        "span of an I and a J"
    );
    assert_eq!(
        msg("for i in I: print(sum(a[..=i.r]))\n"),
        "`..=` ends at a position; a range up to the boundary `i.r` is written `..`"
    );
    for body in [
        "print(sum(a[I.first!..=I.last!]))\n",
        "for i in I: print(sum(a[..i.r]), sum(a[i.l..]))\n",
        "print(span(I.last!, I.first!).len)\n",
        "for i in I: print(span(i.l, I.last!).len)\n",
    ] {
        let src = format!("{head}{body}");
        assert!(codes(&src).is_empty(), "{src}: {:?}", check(&src));
    }
}

#[test]
fn the_max_of_a_slice_is_optional() {
    let head = "let n: 1..=5 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nlet f = V.first\n";
    let empty = one(&format!("{head}let k: Int = a[f..f].max()\n"));
    assert_eq!(
        empty.message,
        "`k` is declared an Int, but `a[f..f].max()` is an ?Int"
    );
    for body in [
        "let k: Int = a[f..f].max() ?? 0\n",
        "let k: Int = max(a)\n",
        "let k: Int = min(a[f], 3)\n",
    ] {
        let src = format!("{head}{body}print(k)\n");
        assert!(codes(&src).is_empty(), "{src}: {:?}", check(&src));
    }
}

#[test]
fn let_else_takes_an_optional() {
    let tuple = one("fn f(a: ?Int) -> Int:\n    let (x, y) = (a, 3) else: return 0\n    x + y\n");
    assert_eq!(
        tuple.message,
        "let-else needs a ?T for `3`, and it is an Int"
    );
    let plain = one("fn f(a: Int) -> Int:\n    let x = a else: return 0\n    x\n");
    assert_eq!(plain.message, "let-else needs a ?T, and `a` is an Int");
    let ok = "fn f(a: ?Int, b: ?Int) -> Int:\n    let (x, y) = (a, b) else: return 0\n    let z = a else: return 1\n    x + y + z\n";
    assert!(codes(ok).is_empty(), "{:?}", check(ok));
    let refutable =
        "fn f(t: (Int, ?Int)) -> Int:\n    let (a, Some(b)) = t else: return 0\n    a + b\n";
    assert!(codes(refutable).is_empty(), "{:?}", check(refutable));
}

#[test]
fn a_try_stays_out_of_branches() {
    let head = "fn pick(a: ?Int, use_it: Bool) -> ?Int:\n";
    for body in [
        "    let v = if use_it: a? + 1 else: 0\n    v * 2\n",
        "    let f = x => a? + x\n    f(1)\n",
        "    sum(a? + i for i in 0..3)\n",
    ] {
        let diag = one(&format!("{head}{body}"));
        assert!(
            diag.message.starts_with("`a?` is inside a branch"),
            "{body}"
        );
        assert_eq!(diag.help.as_deref(), Some("bind it first: `let x = a?`"));
    }
    // Straight-line code, and a statement of an `if` that is a statement.
    for body in [
        "    let v = a? + 1\n    v * 2\n",
        "    if use_it:\n        let v = a?\n        return v\n    a? + 1\n",
        "    if use_it: a? + 1 else: 0\n",
    ] {
        let src = format!("{head}{body}");
        assert!(codes(&src).is_empty(), "{src}: {:?}", check(&src));
    }
}

#[test]
fn a_fold_into_a_map_needs_an_identity() {
    let divide = one("var m: HashMap<Int, Int> = []\nm[4] /= 2\n");
    assert!(divide
        .message
        .ends_with("`/` has no identity to start it from"));
    assert_eq!(
        divide.help.as_deref(),
        Some("say what a missing key means: `m[4] = (m.get(4) ?? start) / x`")
    );
    let named = one("fn longer(x: Int, y: Int) -> Int: if y > x: y else: x\nvar m: Map<Int, Int> = []\nm[4] longer= 2\n");
    assert!(named
        .message
        .ends_with("`longer` has no identity to start it from"));
    // `+`, `*`, `gcd` and the rest start from their identity, `min` and
    // `max` store the value, and a `?Int` entry starts as `none`.
    for body in [
        "m[4] += 2\n",
        "m[4] *= 2\n",
        "m[4] min= 2\n",
        "m[4] gcd= 2\n",
        "m[4] = 7\n",
    ] {
        let src = format!("var m: Map<Int, Int> = []\n{body}");
        assert!(codes(&src).is_empty(), "{src}: {:?}", check(&src));
    }
    let array = "domain V = 1..=5\nvar a: [V]Int = 8\na[V(1)] /= 2\n";
    assert!(codes(array).is_empty(), "{:?}", check(array));
}

/// A list, a fn that grows it, and an array with two positions.
const GLOBALS: &str = "\
let n: 2..=9 = read()
domain V = 1..=n
var a: [V]Int = read()
var xs: List<Int> = [1, 2, 3]
let l = V(1)
let r = V(2)
fn grow() -> Int:
    xs.push(4)
    return 0
fn clobber():
    a[V(1)] = 99
fn outer():
    clobber()
fn peek() -> Int:
    return a[V(1)]
fn total(ys: List<Int>, k: Int) -> Int:
    return sum(ys) + k
";

fn with_globals(rest: &str) -> String {
    format!("{GLOBALS}{rest}")
}

#[test]
fn a_fn_that_changes_a_var_cant_run_while_it_is_lent() {
    for (call, name) in [("clobber()", "clobber"), ("outer()", "outer")] {
        let diag = one(&with_globals(&format!(
            "let part = a[l..=r]\n{call}\nprint(sum(part))\n"
        )));
        assert_eq!(diag.code.0, "E0602");
        assert_eq!(
            diag.message,
            format!("`{name}` can't be called here: it changes `a`, and `a` is lent to `part` until its last use")
        );
    }
    // Reading it is fine, and so is changing it once `part` is done with.
    for rest in [
        "let part = a[l..=r]\nprint(peek())\nprint(sum(part))\n",
        "let part = a[l..=r]\nprint(sum(part))\nclobber()\n",
        "let part = a[l..=r]\nprint(grow())\nprint(sum(part))\n",
    ] {
        let src = with_globals(rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
}

#[test]
fn a_loop_lends_what_it_goes_over() {
    let pushed = one(&with_globals(
        "for x in xs:\n    if xs.len < 1000: xs.push(x)\n",
    ));
    assert_eq!(
        pushed.message,
        "`xs` can't be changed here: the `for` loop is going over `xs`, and that lends it for reading until the loop ends"
    );
    assert_eq!(
        pushed.help.as_deref(),
        Some("loop over positions instead (`for i in 0..xs.len`), or over a copy (`for x in xs.copy()`)")
    );
    let called = one(&with_globals("for x in xs:\n    print(grow() + x)\n"));
    assert_eq!(
        called.message,
        "`grow` can't be called here: it changes `xs`, and the `for` loop is going over `xs`"
    );
    // Over positions or a copy, in another list, or after the loop.
    for rest in [
        "for i in 0..xs.len:\n    xs.push(i)\n    if xs.len > 9: break\n",
        "for x in xs.copy():\n    xs.push(x)\n",
        "var ys: List<Int> = []\nfor x in xs:\n    ys.push(x)\n",
        "for x in xs:\n    print(x)\nxs.push(1)\nprint(grow())\n",
        "for x in xs:\n    print(peek() + x)\n",
    ] {
        let src = with_globals(rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
}

#[test]
fn an_argument_cant_change_what_another_lends() {
    let diag = one(&with_globals("print(total(xs, grow()))\n"));
    assert_eq!(
        diag.message,
        "`grow()` changes `xs` while `xs` is lent to this call"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("do it in a statement before the call")
    );
    // A number is copied before the call, and a statement before it is
    // the fix.
    for rest in [
        "print(total(xs, peek()))\n",
        "let k = grow()\nprint(total(xs, k))\n",
        "fn add(x: Int, k: Int) -> Int: x + k\nprint(add(xs[0], grow()))\n",
    ] {
        let src = with_globals(rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
}

#[test]
fn a_fn_cant_be_handed_what_it_changes_directly() {
    let src = with_globals(
        "fn go(mut x: Int, i: V):\n    a[i] += x\n    x += 1\n    if i != V.last:\n        go(mut x, (i + 1)!)\ngo(mut a[l], l)\n",
    );
    let diag = one(&src);
    assert_eq!(
        diag.message,
        "`a[l]` can't be passed to `go`: `go` also changes `a` directly, so it would reach the same storage under two names"
    );
    // By value, or a fn that keeps to its parameters. A list handed over
    // without `mut` is a value too: it does not move when the fn changes
    // the `var` it came from.
    for rest in [
        "fn grow_and_sum(ys: List<Int>) -> Int:\n    xs.push(99)\n    sum(ys)\nprint(grow_and_sum(xs))\n",
        "fn go(x: Int, i: V):\n    a[i] += x\ngo(a[l], l)\n",
        "fn go(mut x: Int, y: Int):\n    x += y\ngo(mut a[l], a[r])\n",
    ] {
        let src = with_globals(rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
}

#[test]
fn a_call_forgets_what_was_known_about_the_vars_it_changes() {
    let head = "var i = V(1)\nfn mv(mut x: Int, mut y: Int):\n    y += x\n    x = 0\nfn step():\n    i = V(2)\n";
    let diag = one(&with_globals(&format!(
        "{head}if i != r:\n    step()\n    mv(mut a[i], mut a[r])\n"
    )));
    assert!(
        diag.message.ends_with(
            "Array.at lends disjoint parts only when `i != j`, and that is not known here"
        ),
        "{}",
        diag.message
    );
    for body in [
        "if i != r:\n    mv(mut a[i], mut a[r])\n    step()\n",
        "if i != r:\n    clobber()\n    mv(mut a[i], mut a[r])\n",
    ] {
        let src = with_globals(&format!("{head}{body}"));
        assert!(codes(&src).is_empty(), "{body}: {:?}", check(&src));
    }
}

#[test]
fn the_right_side_cant_change_the_place_assigned() {
    let popped = one(&with_globals("xs[0] = xs.pop()!\n"));
    assert!(
        popped
            .message
            .starts_with("`xs` is changed on the right side of an assignment to `xs[0]`"),
        "{}",
        popped.message
    );
    let inc = "fn inc(mut x: Int) -> Int:\n    x += 1\n    return x\n";
    let compound = one(&with_globals(&format!("{inc}a[l] += inc(mut a[l])\n")));
    assert!(
        compound
            .message
            .starts_with("`a[l]` is changed on the right side of an assignment to `a[l]`"),
        "{}",
        compound.message
    );
    assert_eq!(
        compound.help.as_deref(),
        Some("compute the right side first (`let t = ...`), then assign")
    );
    for rest in [
        format!("{inc}assert l != r\na[l] += inc(mut a[r])\n"),
        format!("{inc}var t = 0\nt = inc(mut t)\n"),
        "let last = xs.pop()!\nxs[0] = last\n".to_string(),
    ] {
        let src = with_globals(&rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
}

#[test]
fn a_min_assign_evaluates_its_place_once() {
    // So a place that calls a fn with an effect is fine.
    let src = "var c = 0\nfn nx() -> Int:\n    c += 1\n    return c\nvar b: [0..9]Int = 0\nb[nx()] max= 4\n";
    assert!(codes(src).is_empty(), "{:?}", check(src));
}

#[test]
fn what_a_fn_changes_is_a_dependency_of_its_callers() {
    // `clobber` starts changing `a`: the call two statements below it is
    // checked again, though its own text and `clobber`'s type are the same.
    let quiet = with_globals("let part = a[l..=r]\nclobber()\nprint(sum(part))\n")
        .replace("    a[V(1)] = 99\n", "    print(99)\n");
    let mut db = Database::new();
    db.set_file("a.cp", quiet.as_str());
    assert!(db.diagnostics("a.cp").is_empty());
    let loud = quiet.replace("    print(99)\n", "    a[V(1)] = 99\n");
    db.set_file("a.cp", loud.as_str());
    let diags = db.diagnostics("a.cp");
    assert_eq!(diags.len(), 1, "{diags:#?}");
    assert_eq!(diags[0].span.text(&loud), "clobber()");
    db.set_file("a.cp", quiet.as_str());
    assert!(db.diagnostics("a.cp").is_empty());
}

#[test]
fn a_generic_fn_takes_its_type_arguments_after_two_colons() {
    let head = "domain A = 0..3\nfn firsts<D>(k: Int) -> Int: k + D.len\n";
    assert!(codes(&format!("{head}print(firsts<A>(1), firsts::<A>(1))\n")).is_empty());
    let diag = one(&format!("{head}print(firsts < A)\n"));
    assert_eq!(
        diag.message,
        "`firsts` is a generic fn and is only called, as `firsts(...)`: its type parameters come from its arguments"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("to name one that no argument gives, write `firsts<T>(...)`")
    );
    let ok = format!("{head}print(firsts::<A>(1))\nlet k = 2\nprint(yes(k < 3))\n");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

#[test]
fn a_heap_starts_empty() {
    let diag = one("var h = MinHeap<Int>(by: x => -x)\n");
    // `MinHeap` is the prelude's: the struct of `spec/std/heap.cp`.
    assert_eq!(diag.message, "`MinHeap<Int>` has no parameter named `by`");
    let ok = "var h = MinHeap<(Int, Int)>()\nh.push((-3, 3))\nvar xs = List<Int>()\nxs.push(1)\n";
    assert!(codes(ok).is_empty(), "{:?}", check(ok));
}

/// dream/cf2183f reads its tree with no `use`: `Tree` is the prelude's,
/// as `Graph` and `Rooted` are.
#[test]
fn a_tree_reads_without_a_use() {
    let src = "domain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    assert!(codes(src).is_empty(), "{:?}", check(src));
}

/// The tree dreams that the stdlib covers check clean, so a change that breaks
/// their spelling (`t.up(+, finish:)`, `t.virtual`, `t.below`) fails here.
#[test]
fn the_tree_dreams_check() {
    for dream in ["cf2035f", "cf2124i", "cf2183d2", "cf2237i1"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../corpus/problems/{dream}/dream.cp"));
        let found = check(&std::fs::read_to_string(path).unwrap());
        assert!(found.is_empty(), "{dream}: {found:?}");
    }
}

#[test]
fn reading_an_enum_needs_its_tags() {
    let rest = "let n: 1..=3 = read()\ndomain Row = 1..=n\nlet g: [Row, Row]Tile = read()\n";
    let diag = one(&format!("enum Tile:\n    X\n    O\n{rest}"));
    assert_eq!(
        diag.message,
        "reading a `Tile` needs to know how its variants are written in the input"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("put `@tags(\"X\", \"O\")` above the enum")
    );
    let ok = format!("@tags(\"X\", \"O\")\nenum Tile:\n    X\n    O\n{rest}");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

/// A missing stdlib `use` goes into the file's line for that module, else
/// below its other `use`s, else above its first line of code.
#[test]
fn an_import_joins_its_module_line_or_the_file_header() {
    let cases = [
        (
            "use std.math.{gcd}\nprint(1)\n",
            "use std.math.{gcd, lcm}\nprint(1)\n",
        ),
        (
            "use std.math.{}\nprint(1)\n",
            "use std.math.{lcm}\nprint(1)\n",
        ),
        (
            "use std.seq.{zip}\nuse std.dsu.{Dsu}\nprint(1)\n",
            "use std.seq.{zip}\nuse std.dsu.{Dsu}\nuse std.math.{lcm}\nprint(1)\n",
        ),
        (
            "// a header\n\nprint(1)\n",
            "// a header\n\nuse std.math.{lcm}\nprint(1)\n",
        ),
        (
            "use std.seq.{zip}",
            "use std.seq.{zip}\nuse std.math.{lcm}\n",
        ),
    ];
    for (text, want) in cases {
        let (span, edit) = crate::import_edit(text, "math", "lcm");
        let mut got = text.to_string();
        got.replace_range(span.start as usize..span.end as usize, &edit);
        assert_eq!(got, want);
    }
    // The checker's quick fix uses it, wherever in the file the name is.
    let src = "use std.math.{gcd}\nlet a = gcd(4, 6)\nprint(floor_sum(a, 3, 1, 0))\n";
    let fix = &one(src).fixes[0];
    assert_eq!(
        fix.edits,
        vec![(Span::point(17), ", floor_sum".to_string())]
    );
}

/// `STD_NAMES` is what the hint for a missing `use` is read from. It is
/// built from `spec/std/` itself, sorted for the binary search, without
/// the names that more than one module declares.
#[test]
fn the_std_name_table_is_read_off_the_standard_library() {
    let names = crate::prelude::std_names();
    assert!(names.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert_eq!(crate::prelude::std_module("Fenwick"), Some("fenwick"));
    assert_eq!(crate::prelude::std_module("no_such_name"), None);
    assert_eq!(crate::prelude::std_module("runs"), Some("seq"));
}

/// The compiler carries `spec/std/` without its tests, line for line, so
/// a span into a bundled module still points at the right line.
#[test]
fn the_bundled_stdlib_drops_tests_and_keeps_every_line() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/std");
    let modules = crate::modules::std_modules();
    assert!(modules.iter().any(|(name, _)| *name == "seq"));
    for (name, bundled) in modules {
        let file = std::fs::read_to_string(root.join(format!("{name}.cp"))).expect("a module file");
        assert!(!bundled.contains("@test"), "{name} keeps a test");
        assert_eq!(bundled.lines().count(), file.lines().count(), "{name}");
        for (kept, line) in bundled.lines().zip(file.lines()) {
            assert!(kept.is_empty() || kept == line, "{name}: {kept}");
        }
    }
    let seq = crate::modules::bundled_std(&["seq".to_string()]).expect("seq is bundled");
    assert!(seq.contains("pub fn bsearch<D>"));
}

/// `spec/std/README.md` is the design of the whole surface, so a public
/// name of `spec/std/` it never mentions is a name nobody designed.
#[test]
fn the_stdlib_design_names_every_public_name() {
    let design = include_str!("../../../spec/std/README.md");
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let words: std::collections::HashSet<&str> = design.split(|c: char| !word(c)).collect();
    let missing: Vec<&str> = crate::prelude::std_names()
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !words.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "spec/std/README.md does not mention {missing:?}"
    );
}

/// `@memo` on a fn that neither back end keeps a table for is an error,
/// not a fn that quietly runs every time.
#[test]
fn memo_on_a_fn_it_cannot_remember_is_reported() {
    let memo = |src: &str| {
        let d = one(src);
        assert_eq!(d.code.0, "E0311", "{d:?}");
        // The decorator is what to change.
        assert_eq!(
            &src[d.span.start as usize..d.span.end as usize],
            "@memo",
            "{d:?}"
        );
        assert!(d.help.is_some(), "{d:?}");
        d.message
    };
    assert_eq!(
        memo("@memo\nfn take(mut xs: List<Int>, i: Int) -> Int:\n    xs.push(i)\n    i\n"),
        "`@memo` can't remember `take`: it changes its argument `xs`"
    );
    assert_eq!(
        memo("struct S:\n    k: Int\n    @memo\n    fn get(self, i: Int) -> Int: self.k + i\n"),
        "`@memo` can't remember `get`: it is a fn of `S`"
    );
    assert_eq!(
        memo("@memo\nfn upto(n: Int) -> yields Int:\n    for i in 0..n: yield i\n"),
        "`@memo` can't remember `upto`: it yields its values one by one"
    );
    assert_eq!(
        memo("@memo\nfn same<T>(x: T) -> T: x\n"),
        "`@memo` can't remember the generic fn `same`"
    );
    // No value: written `-> ()`, a body that only acts, a tail `print`.
    for body in [
        "fn show(x: Int) -> ():\n    print(x)\n",
        "fn show(x: Int):\n    var t = x\n    t += 1\n",
        "fn show(x: Int):\n    print(x)\n",
    ] {
        assert_eq!(
            memo(&format!("@memo\n{body}")),
            "`@memo` has nothing to remember: `show` gives no value"
        );
    }
    // A fn inside a fn is held to the same.
    assert_eq!(
        memo("fn outer() -> Int:\n    @memo\n    fn inner(mut ys: List<Int>) -> Int: ys.len\n    var ys: List<Int> = []\n    inner(mut ys)\n"),
        "`@memo` can't remember `inner`: it changes its argument `ys`"
    );
}

#[test]
fn memo_on_a_fn_with_a_value_and_plain_arguments_is_fine() {
    for src in [
        "@memo\nfn fib(n: Int) -> Int:\n    if n < 2: return n\n    fib(n - 1) + fib(n - 2)\nprint(fib(50))\n",
        // The return type is read off the body.
        "@memo\nfn twice(n: Int):\n    if n > 3: return 5\n    n * 2\nprint(twice(4))\n",
        "@memo\nfn sq(x: Int) -> Int = x * x\nprint(sq(3))\n",
        "let n: 1..=20 = read()\ndomain V = 0..n\n@memo(into: dp)\nfn ways(v: V) -> Int:\n    if let u = v + 1: return ways(u) + 1\n    1\nprint(ways(V.first!))\n",
        "fn outer(k: Int) -> Int:\n    @memo\n    fn inner(i: Int) -> Int:\n        if i == 0: return k\n        inner(i - 1) + 1\n    inner(5)\nprint(outer(2))\n",
    ] {
        assert_eq!(check(src), Vec::new(), "{src}");
    }
}

/// `adj.out(u)` reads a table of neighbour lists. On any other table the
/// checker says so, as `cpl run` and `cpl build` do.
#[test]
fn out_on_a_table_that_holds_no_lists_is_reported() {
    let rest = "let n = 3\ndomain V = 0..n\n";
    let d = one(&format!(
        "{rest}let a: [V]Int = [V] v => 0\nprint(a.out(V(1)))\n"
    ));
    assert_eq!(d.code.0, "E0203", "{d:?}");
    assert_eq!(d.help.as_deref(), Some("index it: `a[v]`"));
    let ok = format!("{rest}var adj: [V]List<V> = [V] v => []\nprint(adj.out(V(0)).len)\n");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

/// The branch an `if const` does not pick is not compiled, so it is not
/// checked; one whose test waits for a type argument still is.
#[test]
fn the_untaken_branch_of_an_if_const_is_not_checked() {
    let skipped =
        "const FAST = false\nif const FAST:\n    print(no_such_name)\nelse:\n    print(1)\n";
    assert!(codes(skipped).is_empty(), "{:?}", check(skipped));
    let picked = "if const 1 < 2:\n    print(1)\nelse:\n    print(no_such_name)\n";
    assert!(codes(picked).is_empty(), "{:?}", check(picked));
    let taken = "const FAST = true\nif const FAST:\n    print(no_such_name)\n";
    assert_eq!(codes(taken).len(), 1, "{:?}", check(taken));
}

/// A test for `none` leaves the name holding a value where the test says
/// so, and nowhere the name is assigned again.
#[test]
fn a_tested_optional_reads_as_its_value() {
    let head = "fn f(x: ?Int) -> Int:\n";
    for body in [
        "    if x == none: return 0\n    x + 1\n",
        "    if x != none: return x + 1\n    0\n",
        "    if x == none or x < 0: return 0\n    x\n",
        "    if x != none: return x! + 1\n    x ?? 0\n",
    ] {
        let src = format!("{head}{body}");
        assert!(codes(&src).is_empty(), "{src}\n{:?}", check(&src));
    }
    // Assigned in the code the test covers: still an optional there.
    let src = "fn g(x: ?Int) -> Int:\n    var y = x\n    if y == none: return 0\n    y = none\n    y + 1\n";
    assert!(!codes(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_match_on_plain_variants_needs_an_arm_for_each() {
    let with = |arms: &str| format!("enum C: R | G | B\nfn f(c: C) -> Int:\n    match c:\n{arms}");
    let d = one(&with("        R: 1\n        G: 2\n"));
    assert_eq!(d.message, "this `match` has no arm for `B`");
    let d = one(&with("        R if true: 1\n        G | B: 2\n"));
    assert_eq!(d.message, "this `match` has no arm for `R`");
    for arms in [
        "        R: 1\n        G | B: 2\n",
        "        R: 1\n        _: 2\n",
        "        R: 1\n        other: 2\n",
    ] {
        assert!(check(&with(arms)).is_empty(), "{:?}", check(&with(arms)));
    }
}

#[test]
fn a_getter_reads_the_same_with_empty_parentheses() {
    let src = "struct P(x: Int):\n    pub @get fn twice(self) -> Int = 2 * x\nfn f(p: P) -> Int = p.twice() + p.twice\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_method_that_calls_a_mut_self_method_takes_mut_self() {
    let src = |receiver: &str| {
        format!(
            "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\n    fn peek({receiver}) -> Int:\n        self.bump()\n        n\n"
        )
    };
    assert_eq!(check(&src("mut self")), vec![]);
    assert_eq!(
        messages(&check(&src("self"))),
        ["this method takes `self` without `mut`, so `self` can't be changed by `bump`"]
    );
    // A field of `self` is part of it.
    let nested = "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\nstruct Pair:\n    a: C\n    fn peek(self) -> Int:\n        a.bump()\n        a.n\n";
    assert_eq!(
        messages(&check(nested)),
        ["this method takes `self` without `mut`, so `self` can't be changed by `bump`"]
    );
}

#[test]
fn a_range_written_as_a_structs_type_argument_says_to_name_it() {
    let diags = check("let n = 4\nvar d = Dsu<0..n>()\n");
    assert!(
        messages(&diags).contains(&"`Dsu` takes a domain by name, and `0..n` is a range"),
        "{diags:?}"
    );
    assert_eq!(
        check("let n = 4\ndomain V = 0..n\nvar d = Dsu<V>()\n"),
        vec![]
    );
}

#[test]
fn a_mut_self_method_needs_a_receiver_that_can_change() {
    let decl = "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\n";
    assert_eq!(check(&format!("{decl}var c = C(0)\nc.bump()\n")), vec![]);
    assert_eq!(
        check(&format!("{decl}fn f(mut c: C):\n    c.bump()\n")),
        vec![]
    );
    assert_eq!(
        messages(&check(&format!("{decl}let c = C(0)\nc.bump()\n"))),
        ["`c` is a `let`, so it can't be changed by `bump`"]
    );
    assert_eq!(
        messages(&check(&format!(
            "{decl}fn f():\n    let c = C(0)\n    c.bump()\n"
        ))),
        ["`c` is a `let`, so it can't be changed by `bump`"]
    );
    assert_eq!(
        messages(&check(&format!("{decl}fn f(c: C):\n    c.bump()\n"))),
        ["`c` is a parameter without `mut`, so it can't be changed by `bump`"]
    );
}

#[test]
fn items_gives_each_element_with_its_index() {
    let src = "domain I = 0..3\nlet a: [I]Int = [4, 5, 6]\nlet xs = [true, false]\n";
    // A position of `I` for `[I]T`, an `Int` for a list.
    assert_eq!(
        codes(&format!("{src}for (i, x) in a.items: print(a[i] + x)\n")),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in a.items:\n    let j: Int = i\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in a.items:\n    let s: Str = x\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in xs.items:\n    let n: Int = x\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}let kept = xs.items\nlet k: Int = kept[0].0\n"
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!(
            "{src}let kept = xs.items\nlet k: Int = kept[0].1\n"
        )),
        ["E0302"]
    );
}

#[test]
fn items_of_a_str_a_map_and_a_grid() {
    let src = concat!(
        "domain R = 0..2\n",
        "var g: [R, 0..3]Int = 0\n",
        "var m = Map<Str, Int>()\n",
        "let s = \"abc\"\n",
    );
    let ok = concat!(
        "for (i, c) in s.items:\n    let n: Int = i\n    let d: Char = c\n",
        "for (k, v) in m.items:\n    let key: Str = k\n    let n: Int = v\n",
        "for ((r, c), x) in g.items:\n    let row: R = r\n    let n: Int = c + x\n",
    );
    assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new());
    assert_eq!(
        codes(&format!(
            "{src}for (i, c) in s.items:\n    let n: Int = c\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (k, v) in m.items:\n    let n: Int = k\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for ((r, c), x) in g.items:\n    let n: Int = r\n"
        )),
        ["E0302"]
    );
}

#[test]
fn a_row_takes_the_larger_cells_of_a_row_of_its_own_type() {
    let src = concat!(
        "domain V = 0..3\ndomain W = 0..3\n",
        "var a: [V]Int = 0\nvar b: [V]Int = 1\nvar w: [W]Int = 0\nvar s: [V]Str = \"\"\n",
        "var xs = [1, 2]\nvar best = 0\n",
    );
    for ok in [
        "a max= b\n",
        "if a min= b: print(a)\n",
        "xs max= [3, 0]\n",
        "best max= 4\n",
        "print(yes(best min= 2))\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
    for bad in [
        "a max= w\n",
        "print(yes(a min= s))\n",
        "a max= 3\n",
        "let hit = xs max= [\"a\"]\n",
    ] {
        assert_eq!(codes(&format!("{src}{bad}")), ["E0302"], "{bad}");
    }
}

#[test]
fn a_filled_list_holds_the_element_type_asked_for() {
    let src =
        "type Z = Mod<998244353>\nfn zeros(k: Int) -> List<Z>:\n    return List.filled(k, 0)\n";
    assert_eq!(
        codes(&format!(
            "{src}let a: List<Z> = List.filled(3, 0)\nlet z: Z = a[0]\n"
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!("{src}let a: List<Z> = List.filled(3, \"x\")\n")),
        ["E0302"]
    );
}

#[test]
fn a_row_of_a_grid_takes_the_larger_cells_of_another_row() {
    let src = concat!(
        "domain K = 0..3\n",
        "var g: [K, 0..4]Int = 0\nvar s: [K, 0..4]Str = \"\"\nvar row: [0..4]Int = 0\n",
        "let (k, v) = (K(0), K(2))\n",
    );
    for ok in [
        "g[k, ..] max= g[v, ..]\n",
        "if g[k, ..] min= g[v, ..]: print(g[k, ..])\n",
        "g[.., 1] max= g[.., 2]\n",
        "g[k, 1..3] min= g[v, 0..2]\n",
        "g[k, ..] max= row\n",
        "g[k, ..] max= [1, 2, 3, 4]\n",
        // One value goes against every cell of the part.
        "g[k, ..] max= 3\n",
        "g[k, 1..3] += 2\n",
        "g[.., 1] min= row[0]\n",
        "row[1..3] -= 1\nrow[..] max= 0\n",
        "if row[..] min= 2: print(row)\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
    for bad in [
        "g[k, ..] max= s[v, ..]\n",
        "print(yes(g[k, ..] min= s[v, ..]))\n",
        "g[k, ..] max= \"x\"\n",
        "row[..] min= 2.5\n",
    ] {
        assert_eq!(codes(&format!("{src}{bad}")), ["E0302"], "{bad}");
    }
}

#[test]
fn a_list_written_out_beside_a_plus_holds_the_element_type_asked_for() {
    let src = "type Z = Mod<998244353>\nlet rest: List<Z> = [Z(2), Z(3)]\nfn pad(a: List<Z>) -> List<Z>:\n    return [0] + a + [0]\n";
    for ok in [
        "let xs: List<Z> = [0] + rest\nlet z: Z = xs[0]\n",
        "let xs: List<Z> = rest + [0, 1]\n",
        "let xs: List<Z> = [1] + [2] + rest\n",
        "let ys: List<F64> = [1] + [2.5]\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
}

/// `xs.sort(by:)` sorts where the list is: a key that reaches the list,
/// by name or through a fn, would read or grow it mid-sort (a
/// use-after-free in the C++). `sorted(by:)` works on a copy.
#[test]
fn a_sort_key_that_reaches_its_own_list_is_refused() {
    const FNS: &str = "var xs = [3, 1, 2]\nvar n = 0\nfn grow(x: Int) -> Int:\n    xs.push(1)\n    x\nfn peek(x: Int) -> Int = x + xs.len\nfn count(x: Int) -> Int:\n    n += 1\n    x\n";
    for call in [
        "xs.sort(by: grow)\n",
        "xs.sort(by: peek)\n",
        "xs.sort(by: x => xs[0] - x)\n",
        "xs.sort(by: x => grow(x))\n",
    ] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0602"],
            "{call}"
        );
    }
    for call in [
        "xs.sort(by: count)\n",
        "xs.sort(by: x => -x)\n",
        "let ys = xs.sorted(by: grow)\n",
        "var ys = [1]\nys.sort(by: peek)\n",
    ] {
        assert_eq!(check(&format!("{FNS}{call}")), vec![], "{call}");
    }
}

/// `cpl run` no longer stops a call that touches a variable it was lent
/// `mut`, so the check has to: one variable twice in a call is refused
/// here, while a fn value reading it during the call is not.
#[test]
fn one_variable_lent_twice_to_a_call_is_refused() {
    const FNS: &str = "fn two(mut x: List<Int>, mut y: List<Int>): x.push(y.len)\nfn one(mut x: List<Int>, y: List<Int>): x.push(y.len)\nvar xs = [1]\n";
    for call in ["two(mut xs, mut xs)\n", "one(mut xs, xs)\n"] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0601"],
            "{call}"
        );
    }
    assert_eq!(
        check(&format!(
            "{FNS}var ys = [2]\ntwo(mut xs, mut ys)\none(mut xs, ys)\n"
        )),
        vec![]
    );
    let closure = "domain P = 0..3\nvar m = Map<Str, P>()\nprint(m.get_or_insert_with(\"a\", () => P(m.len)))\n";
    assert_eq!(check(closure), vec![]);
}

#[test]
fn a_try_is_refused_where_it_is_not_always_run() {
    for source in [
        "fn f(c: Bool, x: ?Int) -> ?Bool:\n    some(c and x? > 0)\n",
        "fn f(c: Bool, x: ?Int) -> ?Bool:\n    some(c or 1 + x? > 0)\n",
        "fn f(p: ?Int, x: ?Int) -> ?Int:\n    some(p ?? (1 + x?))\n",
        "fn f(n: Int, x: ?Int) -> ?Int:\n    var i = 0\n    while i + x? < n: i += 1\n    some(i)\n",
    ] {
        let found = codes(source);
        assert_eq!(found, ["E0309"], "{source}");
    }
}

/// `debug(..)` is left out of the C++ a judge runs, so what it is given
/// must change nothing: a `pop`, a `read` or a `mut` call inside would
/// happen in `cpl run` and not on the judge.
#[test]
fn debug_of_something_that_changes_the_program_is_refused() {
    const FNS: &str = "var q = [1, 2, 3]\nvar total = 0\nvar y = 4\nfn add(k: Int) -> Int:\n    total += k\n    total\nfn shout(k: Int) -> Int:\n    print(k)\n    k\nfn inc(mut x: Int) -> Int:\n    x += 1\n    x\nfn twice(k: Int) -> Int = k * 2\n";
    for call in [
        "debug(q.pop())\n",
        "debug(add(2))\n",
        "debug(shout(1))\n",
        "debug(inc(mut y))\n",
        "debug(q.len, twice(add(1)))\n",
        "let n: Int = 0\ndebug(read() + n)\n",
    ] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0312"],
            "{call}"
        );
        assert!(
            diags[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("debug(x)")),
            "{call}"
        );
    }
    for call in [
        "debug(q, q.len, twice(3), q.sorted(), q.max())\n",
        "let x = q.pop()\ndebug(x)\n",
        "debug(twice(y) + 1)\n",
    ] {
        assert_eq!(check(&format!("{FNS}{call}")), vec![], "{call}");
    }
}

#[test]
fn a_dropped_closer_inside_a_call_or_index_is_the_only_error() {
    // `f(a[i, j)` closes the call, so the call or index sees one argument
    // too many or too few; that is the missing `]`, not a second error.
    for (port, good, bad) in [
        ("cf2165f", "latest.set(a[j], j)", "latest.set(a[j, j)"),
        ("cf2096b", "max(l[i], r[i]) for", "max(l[i, r[i]) for"),
        (
            "cf2096h",
            "(even_prod[a], odd_prod[a], even_prod[b], odd_prod[b]) =",
            "(even_prod[a, odd_prod[a], even_prod[b], odd_prod[b]) =",
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../corpus/problems/{port}/port.cp"));
        let src = std::fs::read_to_string(path).unwrap();
        assert!(src.contains(good), "{port}: {good}");
        assert_eq!(
            codes(&src.replacen(good, bad, 1)),
            ["E0101"],
            "{port}: {bad}"
        );
    }
}

#[test]
fn an_unreadable_character_is_reported_where_it_is() {
    // The lexer poisons the character's span so nothing else is reported
    // there; its own diagnostic must not fall to that poison.
    let src = "let $x = 1\nprint(x)\n";
    let diags = check(src);
    assert_eq!(diags[0].code.0, "E0004", "{diags:#?}");
    assert_eq!(diags[0].span, Span::new(4, 5));
}
#[test]
fn fields_loops_check_each_field_and_refuse_what_has_none() {
    let src =
        "struct P(x: Int, s: Str)\nlet p = P(1, \"a\")\nfor f in fields(p):\n    print(f + 1)\n";
    let d = one(src);
    assert_eq!(d.code.0, "E0309", "{d:#?}");
    let d = one("for f in fields(3):\n    print(f)\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        (
            "E0302",
            "`fields` walks the fields of a struct or a tuple, not of a `Int`"
        )
    );
    let d = one("for f in fields((1, 2)):\n    if f > 1:\n        break\n");
    assert_eq!(d.code.0, "E0501");
    let d = one("for (a, b) in fields((1, 2), (1, 2, 3)):\n    print(a)\n");
    assert_eq!(d.code.0, "E0302");
    assert!(
        codes("fn fields(x: Int) -> List<Int> = [x]\nfor f in fields(3):\n    print(f)\n")
            .is_empty()
    );
}

#[test]
fn names_from_a_declaration_that_failed_to_parse_stay_bound() {
    // Uses of `l`, `b` and `f` are not "not in scope" and say nothing about
    // types: the one error is the one root cause.
    let src = "let ($l, b) = (4, 6)\nprint(l + b)\nfn $f(a: Int) -> Int:\n    return a + 1\nprint(f(2) + l)\n";
    assert_eq!(codes(src), ["E0004", "E0004"], "{:#?}", check(src));
}

#[test]
fn a_recovered_name_still_reports_errors_unrelated_to_it() {
    let src = "let ($l, b) = (4, 6)\nprint(l + b)\nprint(missing)\n";
    assert_eq!(codes(src), ["E0004", "E0201"], "{:#?}", check(src));
}
#[test]
fn a_fields_loop_over_an_enum_checks_each_variant_payload() {
    let head = "enum Op:\n    Add(Int, Int)\n    Name(Str)\n    Stop\n";
    assert!(codes(&format!("{head}for f in fields(Op.Stop):\n    print(f)\n")).is_empty());
    let d = one(&format!(
        "{head}for f in fields(Op.Stop):\n    print(f + 1)\n"
    ));
    assert_eq!(d.code.0, "E0309", "{d:#?}");
}

#[test]
fn a_fields_constructor_gives_each_field_a_value_of_its_type() {
    let head = "struct P(a: Int, b: Str)\nstruct Q(a: Int)\nlet p = P(1, \"x\")\n";
    assert!(codes(&format!(
        "{head}let a = P(read() for _ in fields(P))\nlet b = P(f for f in fields(p))\n"
    ))
    .is_empty());
    let d = one(&format!("{head}let b = P(1 for _ in fields(P))\n"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "the field `b` is a Str, but `1` is an Int")
    );
    let d = one(&format!("{head}let c = Q(f for f in fields(p))\n"));
    assert_eq!(
        d.message,
        "`fields` here walks values of the type it builds, `Q`; this is a `P`"
    );
    let d = one(&format!("{head}let c = P(f for f in fields(P))\n"));
    assert_eq!(
        d.help.as_deref(),
        Some("write `_`, as in `P(read() for _ in fields(P))`")
    );
}

#[test]
fn a_lazy_parameter_takes_the_expression_its_lambda_gives_back() {
    let head = "fn or_else<T>(a: ?T, lazy d: () -> T) -> T:\n    if let v = a:\n        return v\n    d()\n";
    assert!(codes(&format!(
        "{head}let x: ?Int = 3\nprint(or_else(x, 4), or_else(x, d: 5), or_else(x, () => 6))\n"
    ))
    .is_empty());
    let d = one(&format!(
        "{head}let x: ?Int = 3\nprint(or_else(x, \"a\"))\n"
    ));
    assert_eq!(d.code.0, "E0302", "{d:#?}");
    let d = one("fn f(lazy d: Int) -> Int = 0\n");
    assert_eq!(
        (d.code.0, d.help.as_deref()),
        ("E0302", Some("write `lazy d: () -> Int`"))
    );
    assert!(codes("let lazy = 3\nfn g(lazy: Int) -> Int = lazy\nprint(g(lazy))\n").is_empty());
}

#[test]
fn a_variadic_parameter_is_a_tuple_of_its_arguments() {
    assert!(codes(
        "fn count<Ts>(..xs: Ts) -> Int:\n    var n = 0\n    for _ in fields(xs):\n        n += 1\n    n\nprint(count(), count(1, \"a\", 2.5))\n"
    )
    .is_empty());
    assert!(
        codes("fn tag<Ts>(t: Str, ..xs: Ts):\n    print(t)\ntag(\"a\", 1, 2)\ntag(\"b\")\n")
            .is_empty()
    );
    let d = one("fn total(..xs: Int) -> Int = 0\nprint(total(1, 2))\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`..xs` collects its arguments into a tuple, so its type is a type parameter, not `Int`")
    );
}

#[test]
fn self_in_trait_range_bounds_and_generic_optional_impls_check() {
    let trait_src = "\
trait Index:
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self
";
    assert!(codes(trait_src).is_empty(), "{:#?}", check(trait_src));

    let impl_src = "\
impl<T: Add> Add for ?T:
    fn add(a, b) -> ?T = if let x = a, y = b: x + y else: none
impl<T: Eq> Eq for ?T
";
    assert!(codes(impl_src).is_empty(), "{:#?}", check(impl_src));

    let assoc_src = "\
trait Unwrap:
    type Out
    fn unwrap(self) -> Out
";
    assert!(codes(assoc_src).is_empty(), "{:#?}", check(assoc_src));
}

#[test]
fn fresh_domain_escapes_are_rejected_with_e0313() {
    let cases = [
        // Returning a local fresh domain without an existential return type.
        "fn bad(k: Int) -> [0..k]Int:\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    a\n",
        // Returning a value of a local fresh domain from a function.
        "fn bad(k: Int):\n    domain Local = 0..k\n    return Local(0)\n",
        // Assigning a fresh-domain value to an outer variable.
        "var saved: Int = 0\nif true:\n    domain Inner = 0..5\n    let v = Inner(0)\n    var outer: ?Inner = none\n    outer = v\n",
        // Storing a fresh-domain value into an outer list.
        "var bag = []\nif true:\n    domain Inner = 0..5\n    bag.push(Inner(0))\n",
        // Capturing a fresh-domain value in a returned closure.
        "fn make(k: Int) -> () -> Int:\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    () => a[Local(0)]\n",
    ];
    for (idx, src) in cases.iter().enumerate() {
        let diags = check(src);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.0).collect();
        if idx == 2 {
            // `outer` is declared inside the same `if` block as `Inner`; no escape.
            assert!(!codes.contains(&"E0313"), "case {idx}: {diags:#?}");
        } else {
            assert!(
                codes.contains(&"E0313"),
                "case {idx} expected E0313, got {diags:#?}"
            );
        }
    }
}

#[test]
fn existential_domain_returns_and_runtime_read_fields_typecheck() {
    let src = "\
fn make_seq(k: Int) -> (domain P, [P]Int):
    domain Local = 0..k
    let a: [Local]Int = 0
    (Local, a)

fn make_pair(a: Int, b: Int) -> (domain A, domain B, [A]Int, [B]Int):
    domain X = 0..a
    domain Y = 0..b
    (X, Y, [X] i => i.to_idx, [Y] j => j.to_idx)

domain V = read(1..=100)
let m: 0..=100 = read()
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
let (domain Q, vals) = make_seq(3)
let (domain A, domain B, xs, ys) = make_pair(2, 4)
print(edges.len, Q.count, vals[Q(0)], A.count, B.count, xs[A(0)], ys[B(0)])
";
    assert_eq!(check(src), vec![]);
}

#[test]
fn sym_storage_and_domain_iteration_typecheck() {
    let ok = "\
domain V = 1..=5
var adj: [Sym<V>]Bool = false
for (i, j) in V.pairs:
    adj[i, j] = true
    let p = (j, i)
    if adj[p]: print(i, j)
for i in V:
    for j in V.after(i): print(i, j)
    for j in V.before(i): print(j, i)
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
domain V = 1..=5
domain U = 1..=5
var adj: [Sym<V>]Bool = false
let _ = adj[V.first, U.first]
";
    assert_eq!(codes(bad), vec![cpl_diag::codes::TYPE_MISMATCH.0]);
}
#[test]
fn typed_lambda_parameter_annotations_are_checked() {
    let ok = "\
fn apply(x: Int, f: fn(Int) -> Int) -> Int = f(x)
print(apply(5, (a: Int) => a + 1))
";
    assert_eq!(check(ok), vec![]);

    let bad_param = "\
fn apply(x: Int, f: fn(Int) -> Int) -> Int = f(x)
print(apply(5, (a: Str) => 1))
";
    assert_eq!(codes(bad_param), vec![cpl_diag::codes::TYPE_MISMATCH.0]);

    let bad_body = "let f = (a: Str) => a + 1\n";
    assert_eq!(codes(bad_body), vec![cpl_diag::codes::BAD_OPERANDS.0]);

    let bad_ty = "let f = (a: Nope) => a\n";
    assert_eq!(codes(bad_ty), vec![cpl_diag::codes::UNKNOWN_TYPE.0]);
}

#[test]
fn a_field_named_without_self_says_where_it_is() {
    let diags = check(
        "struct P(z: Int, lnds: Int):\n    fn up(self) -> Int = lnds + 1\n    fn fresh() -> Int = lnds\nprint(P(1, 2).up())\n",
    );
    assert_eq!(messages(&diags), ["no `lnds` in scope"]);
    assert_eq!(
        diags[0].help.as_deref(),
        Some("`lnds` is a field of `P`, and this method takes no `self`: add `self` as its first parameter")
    );
}

#[test]
fn a_name_in_no_scope_and_no_library_says_so_and_offers_the_closest_library_name() {
    let diags = check("print(heap_free_search(3))\n");
    assert_eq!(
        messages(&diags),
        ["no `heap_free_search` in scope, and not in the stdlib (yet)"]
    );
    let name = crate::prelude::std_names()
        .iter()
        .map(|(n, _)| *n)
        .find(|n| {
            n.len() > 5
                && n.chars().all(|c| c.is_ascii_lowercase())
                && crate::check::closest(
                    &format!("{n}x"),
                    crate::prelude::VALUE_NAMES.iter().copied(),
                )
                .is_none()
        })
        .expect("a stdlib function no prelude name is close to");
    let typo = format!("{name}x");
    let diags = check(&format!("print({typo}(1))\n"));
    assert!(
        diags[0].message.ends_with("and not in the stdlib (yet)"),
        "{diags:#?}"
    );
    let help = diags[0].help.as_deref().unwrap_or_default();
    assert!(
        help.starts_with(&format!("did you mean `{name}`, from `std.")),
        "{help}"
    );
    let fix = &diags[0].fixes[0];
    assert_eq!(fix.edits.len(), 2, "rename and import: {fix:#?}");
}

#[test]
fn heterogeneous_operator_traits_check_and_reject_mismatched_rhs() {
    let ok = "\
struct Vec:
    x: F64
    y: F64

impl Add for Vec:
    fn add(a, b) = Vec(a.x + b.x, a.y + b.y)

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k, a.y * k)

impl Div<F64> for Vec:
    fn div(a, k) = Vec(a.x / k, a.y / k)

struct Mat:
    a: F64

impl Mul for Mat:
    fn mul(x, y) = Mat(x.a * y.a)

impl Mul<Vec> for Mat:
    type Out = Vec
    fn mul(lhs, rhs) -> Out = Vec(lhs.a * rhs.x, lhs.a * rhs.y)

let u = Vec(1.0, 2.0)
let v: Vec = (u + u) * 2.0 / 4.0
let m = Mat(3.0)
let mv: Vec = m * v
let m2: Mat = m * m
print(mv.x, m2.a)
";
    let diags = check(ok);
    assert!(diags.is_empty(), "expected no errors, got: {diags:#?}");

    let bad = "\
struct Vec:
    x: F64

impl Mul<F64> for Vec:
    fn mul(a, k) = Vec(a.x * k)

let u = Vec(1.0)
let bad = u * \"nope\"
print(bad.x)
";
    let diags = check(bad);
    assert!(
        diags.iter().any(|d| d.code == cpl_diag::codes::MISSING_IMPL
            && d.message.contains("impl Mul<Str> for Vec")),
        "expected MISSING_IMPL for Vec * Str, got: {diags:#?}"
    );
}

#[test]
fn enumerate_points_to_items() {
    let free = check("let xs = [1, 2]\nfor (i, x) in enumerate(xs): print(i, x)\n");
    assert!(!free.is_empty());
    assert!(
        free[0]
            .help
            .as_deref()
            .unwrap_or_default()
            .contains(".items"),
        "expected `.items` hint on bare `enumerate`: {free:#?}"
    );
    let method = check("let xs = [1, 2]\nfor (i, x) in xs.enumerate(): print(i, x)\n");
    assert!(!method.is_empty());
    assert!(
        method[0].message.contains(".items")
            || method[0]
                .help
                .as_deref()
                .unwrap_or_default()
                .contains(".items"),
        "expected `.items` hint on `.enumerate()`: {method:#?}"
    );
}

#[test]
fn slice_and_reverse_views_are_readonly_and_cannot_outlive_mutated_base() {
    // Mutating the base while a slice or reverse view is still live is E0602.
    assert_eq!(
        codes("var s = \"abcdef\"\nlet v = s[1..4]\ns.push('x')\nprint(v)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var s = \"abcdef\"\nlet r = s.rev\ns.push('x')\nprint(r)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet v = xs[1..3]\nxs.push(5)\nprint(v)\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet r = xs.rev\nxs.push(5)\nprint(r)\n"),
        ["E0602"]
    );

    // Mutating through a read-only view is E0401.
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nlet v = xs[1..3]\nv.push(5)\n"),
        ["E0401"]
    );
    assert_eq!(codes("var xs = [1, 2, 3, 4]\nxs.rev.sort()\n"), ["E0401"]);
    assert_eq!(codes("let xs = [1, 2, 3, 4]\nxs[1..3][0] = 9\n"), ["E0401"]);
    assert_eq!(codes("var s = \"abcdef\"\ns[1..4][0] = 'x'\n"), ["E0401"]);

    // Returning a view escapes its lending block (E0603); copying explicitly succeeds.
    assert_eq!(codes("fn bad(s: Str) -> Str:\n    s[1..3]\n"), ["E0603"]);
    assert_eq!(
        codes("fn bad(xs: List<Int>) -> List<Int>:\n    xs.rev\n"),
        ["E0603"]
    );
    assert_eq!(
        codes(concat!(
            "fn ok_s(s: Str) -> Str:\n    s[1..3].to_str()\n",
            "fn ok_l(xs: List<Int>) -> List<Int>:\n    xs.rev.to_list()\n",
            "var s = \"abcdef\"\nlet owned = s[1..4].to_str()\ns.push('x')\nprint(owned)\n",
        )),
        Vec::<&str>::new()
    );

    // First-class `.rev` view on a 1D domain array: valid value, E0602 on base mutation, E0603 on escape.
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "struct Tree(at: [Node]Int)\n",
            "let a: [Node]Int = [Node] i => i.ord\n",
            "let t = Tree(a)\n",
            "let post = t.at.rev\n",
            "print(post.len, post[0])\n",
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "var a: [Node]Int = [Node] i => i.ord\n",
            "let v = a.rev\n",
            "a[Node.first!] = 9\n",
            "print(v[0])\n",
        )),
        ["E0602"]
    );
    assert_eq!(
        codes(concat!(
            "domain Node = 1..=3\n",
            "fn leak(xs: [Node]Int) -> List<Int>:\n    xs.rev\n",
        )),
        ["E0603"]
    );
}

#[test]
fn domain_subsets_enum_arrays_seq_ops_and_cuts_typecheck() {
    let ok = "\
enum Sign: neg = -1, zero = 0, pos = 1
domain V = 1..=5
domain Item = 1..=5
domain Cut = 0..=5
let edges: List<(u: V, v: V)> = [(V(1), V(2))]
let adj = adjacency(edges, into: Bitset)
let full: Bitset<V> = V.all
let diff = V.all - adj[V(1)]
for v in diff: print(v)
let vals: [Sign]Int = [Sign] s => Int(s)
for s in Sign: print(vals[s])
let (evens, odds) = V.partition(v => Int(v) % 2 == 0)
let w: [V]Int = [V] v => Int(v)
let order = V.sort(by: w)
let groups = V.group(by: v => Int(v) % 2)
let picked = V.first(v => Int(v) > 2)
let p: [Item]Cut = [Item] i => Cut(Int(i) - 1)
let s: [Item]Cut = [Item] i => Cut(Int(i))
let (lo, hi) = minmax(p[Item(1)], s[Item(1)])
let r = lo..hi
var st = SegTree<V.cuts, Int>(+, fill: 0)
var lst = LazySegTree<V.cuts, Int>(AddMax, fill: 0)
var fw = Fenwick<V.cuts, Int>()
let v0 = V(2)
st.set(v0.r, 5)
lst.set(v0.r, 7)
lst.apply(..=v0.l, 1)
fw.add(v0.r, 3)
print(full.len, evens.len, odds.len, order.len, groups.len, picked, r.len, st.fold(..=v0.r), lst.all, fw.sum(..=v0.r))
";
    assert_eq!(check(ok), vec![]);

    let bad_cut = "\
domain Item = 1..=5
domain Cut = 0..=5
let a: [Cut]Int = 0
let _ = a[Item(1)]
";
    assert_eq!(codes(bad_cut), vec![cpl_diag::codes::WRONG_DOMAIN.0]);
}

#[test]
fn destructured_domain_in_scope_for_nested_fns() {
    let ok = "\
fn make(k: Int) -> (domain P, [P]Int):
    domain L = 0..k
    (L, [L] i => i.to_idx)

let (domain C, comp) = make(3)
var total = 0
fn bump(c: C, v: Int):
    total += comp[c] + v

for c in C: bump(c, 1)

fn outer() -> Int:
    let (domain K, xs) = make(4)
    fn pick(k: K) -> Int = xs[k]
    var s = 0
    for k in K: s += pick(k)
    s

print(total, outer())
";
    assert_eq!(check(ok), vec![]);
}


#[test]
fn generic_struct_ctor_unifies_caller_type_params_in_nested_fields() {
    let ok = "\
domain V = 0..3
struct P<V, A>:
    back: [V]?(from: V, via: A)
struct Q<V, A>:
    back: [V]?(V, A)
fn mk_rec<S, M>(s: S, m: M) -> P<S, M>:
    var b: [S]?(from: S, via: M) = none
    b[s] = (from: s, via: m)
    P(b)
fn mk_tup<S, M>(s: S, m: M) -> Q<S, M>:
    var b: [S]?(S, M) = none
    b[s] = (s, m)
    Q(b)
fn mk_one_renamed<V, M>(s: V, m: M) -> P<V, M>:
    var b: [V]?(from: V, via: M) = none
    b[s] = (from: s, via: m)
    P(b)
print(mk_rec(V(1), 2).back[V(1)]!.via, mk_tup(V(1), 3).back[V(1)]!.1, mk_one_renamed(V(1), 4).back[V(1)]!.via)
";
    assert_eq!(check(ok), vec![]);
}

#[test]
fn product_domains_typecheck_hover_and_reject_swapped_factors() {
    let ok = "\
domain V = 0..3
domain K = 1..=2
domain State = (V, K)
var dp: [State]Int = 0
for s in State:
    let (v, k) = s
    dp[s] = v.ord * 10 + k.label
for (v, k) in State:
    dp[(v, k)] += dp[v, k]
let s0 = State(V(1), K(2))
let _ = (s0.0, s0.1)

enum Dir = L | R
domain Named = (v: V, d: Dir, ok: Bool)
var seen: [Named]Int = 0
seen[(V(1), Dir.R, true)] = 42
let ns = Named(v: V(1), d: Dir.R, ok: true)
let _ = (seen[ns], ns.v, ns.d, ns.ok)

fn local_prod(n: Int, m: Int) -> Int:
    domain A = 0..n
    domain B = 0..m
    domain P = (A, B)
    var t: [P]Int = 0
    t[(A(0), B(0))]
";
    assert_eq!(check(ok), vec![]);
    // LSP hover reports product domain definition and combined max size (3 * 2 = 6).
    let state_hover = hover(ok, "State =");
    assert!(
        state_hover.contains("domain State = (V, K)")
            && state_hover.contains("at most 6 positions"),
        "unexpected hover: {state_hover}"
    );

    let bad = "\
domain V = 0..3
domain K = 0..2
domain State = (V, K)
var dp: [State]Int = 0
print(dp[(K(0), V(0))])
print(dp[K(0), V(0)])
print(State(K(0), V(0)))
";
    let errs = check(bad);
    assert!(
        errs.len() >= 3 && errs.iter().all(|d| d.code == cpl_diag::codes::WRONG_DOMAIN),
        "expected E0301 on swapped product factors, got: {errs:?}"
    );
}

#[test]
fn partial_and_total_array_inverse_typecheck() {
    let ok = "\
domain P = 1..=5
domain V = 0..4
enum Dir = N | E | S | W

var a: [P]?V = none
let inv_a: [V]?P = a.inverse()
let _ = inv_a[V(0)]

let n: 1..=5 = 5
var b: [P]?(1..=n) = none
let inv_b: [1..=n]?P = b.inverse()
let _ = inv_b[1]

var c: [P]?Dir = none
let inv_c: [Dir]?P = c.inverse()
let _ = inv_c[Dir.N]

domain I = 0..3
domain J = 1..=3
let perm: [I]J = [J(1), J(2), J(3)]
let inv_perm: [J]I = perm.inverse()
let _ = inv_perm[J(1)]
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
domain P = 1..=5
var xs: [P]?Int = none
let _ = xs.inverse()
";
    assert!(
        !check(bad).is_empty(),
        "expected error for inverse on `[P]?Int` (unbounded Int is not an Index)"
    );
}

#[test]
fn pow_method_is_rejected_and_suggests_star_star() {
    let bad = "\
let a = 2
let b = a.pow(10)
print(b)
";
    let errs = check(bad);
    assert!(
        errs.iter()
            .any(|e| e.message.contains("power uses `**`, not `.pow(...)`")),
        "expected `.pow` error suggesting `**`, got: {errs:?}"
    );
}

#[test]
fn builtin_getters_reject_empty_parentheses_with_fix() {
    let bad = "\
let xs = [1, 2, 3]
let s = \"abc\"
print(xs.len(), xs.is_empty(), xs.first(), xs.last(), xs.rev(), xs.items(), s.len(), s.rev())
";
    let errs = check(bad);
    assert_eq!(errs.len(), 8, "expected 8 getter errors, got: {errs:#?}");
    for e in &errs {
        assert!(
            e.message.contains("is read without parentheses:"),
            "unexpected message: {}",
            e.message
        );
        assert!(!e.fixes.is_empty(), "expected fix-it on: {e:#?}");
    }
    let ok = "\
let xs = [1, 2, 3]
print(xs.len, xs.is_empty, xs.first, xs.last, xs.rev, xs.items, xs.first(x => x > 1), xs.last(1))
";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );
}

#[test]
fn pos_back_ahead_and_range_drop_take_typecheck() {
    let src = "\
domain D = 1..=5
let p: D = D(3)
let a: ?D = p.ahead(2)
let b: ?D = p.back(1)
let r: Range<D> = D(1)..=D(5)
let r2: Range<D> = r.drop(1).take(2)
let r3: Range<Int> = (0..10).drop(2).take(3)
print(a == none, b == none, r2.len, r3.len)
";
    assert!(
        check(src).is_empty(),
        "expected clean check, got: {:?}",
        check(src)
    );
}
#[test]
fn existential_return_compress_hover_and_escape() {
    let ok = "\
fn compress_vals<D>(xs: [D]Int) -> (domain C, [D]C, [C]Int):
    var uniq: List<Int> = []
    for i in D:
        uniq.push(xs[i])
    uniq.sort()
    uniq = uniq.deduped()
    domain C = 0..uniq.len
    let idx: [D]C = [D] i => C(bsearch_first(0..uniq.len, k => uniq[k] >= xs[i])!)
    let vals: [C]Int = [C] c => uniq[c.ord]
    (C, idx, vals)

domain I = 0..4
let a: [I]Int = [30, 10, 30, 20]
let (domain V, mapped, vals) = compress_vals(a)
print(V.len, mapped[I(0)].ord, vals[mapped[I(3)]])
";
    assert_eq!(check(ok), vec![]);
    let v_hover = hover(ok, "V, mapped");
    assert!(
        v_hover.contains("V"),
        "expected existential domain hover for V, got: {v_hover}"
    );
    let mapped_hover = hover(ok, "mapped,");
    assert!(
        mapped_hover.contains("[I]V"),
        "expected `[I]V` hover for mapped, got: {mapped_hover}"
    );
    let vals_hover = hover(ok, "vals) =");
    assert!(
        vals_hover.contains("[V]Int"),
        "expected `[V]Int` hover for vals, got: {vals_hover}"
    );

    let bad = "\
fn bad_compress<D>(xs: [D]Int) -> [D]Int:
    domain C = 0..2
    let p: [C]Int = 0
    p
";
    let errs = check(bad);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::DOMAIN_ESCAPES),
        "expected E0313 on escaping local domain, got: {errs:?}"
    );
}

#[test]
fn struct_array_field_defaults_and_optional_map_filter_typecheck() {
    let ok = "\
type Z = Mod<998_244_353>
type Vol = 1..=5
struct Barrels:
    count: [Vol]Int = 0
    delta: [0..=4]Z = 0

let b = Barrels()
let _ = (b.count[1], b.delta[0])

domain P = 1..=4
fn is_even(p: P) -> Bool = p.label % 2 == 0
let x: ?P = some(P(2))
let y: ?Bool = x.map(is_even)
let z: ?Int = x.filter(p => p.label > 1).map(p => p.ord)
let n_opt: ?P = none
let w: ?Int = n_opt.map(p => p.ord)
let _ = (y ?? false, z ?? -1, w ?? -1)
";
    assert_eq!(check(ok), vec![]);
}
#[test]
fn unary_plus_requires_numeric_operand() {
    let ok = "let a = +1\nlet b = +2.5\nprint(a, b)\n";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );

    let bad = "let xs = +[1, 2]\nlet s = +\"hi\"\nprint(xs, s)\n";
    let errs = check(bad);
    assert_eq!(errs.len(), 2, "expected 2 E0302 errors, got: {errs:?}");
    assert!(
        errs.iter()
            .all(|e| e.code == cpl_diag::codes::TYPE_MISMATCH),
        "expected E0302 for unary `+` on non-numeric types, got: {errs:?}"
    );
}

#[test]
fn sum_domain_enum_tuple_record_index_and_collision_diag() {
    let ok = "\
use std.flow.{FlowGraph}
use std.dsu.{Dsu}

domain Row = 0..2
domain Col = 0..3

enum Node:
    Src
    Snk
    Cell(Row, Col)

let _: Int = Node.count
var vals: [Node]Int = 0
for v in Node:
    vals[v] = match v:
        Src => 0
        Snk => 1
        Cell(r, c) => r.ord + c.ord
var g = FlowGraph<Node>()
g.add(Node.Src, Node.Cell(Row(0), Col(0)), 1)
let _ = g.max_flow(Node.Src, Node.Snk)

var dsu = Dsu<Node>()
dsu.unite(Node.Src, Node.Snk)

var fg_tup = FlowGraph<(Row, Col)>()
fg_tup.add((Row(0), Col(0)), (Row(1), Col(2)), 2)
let _ = fg_tup.max_flow((Row(0), Col(0)), (Row(1), Col(2)))

var fg_rec = FlowGraph<(r: Row, c: Col)>()
fg_rec.add((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2)), 3)
let _ = fg_rec.max_flow((r: Row(0), c: Col(0)), (r: Row(1), c: Col(2)))

fn walk_anon() -> Int:
    var total = 0
    for (r, c) in (Row, Col):
        total += r.ord + c.ord
    let xs = [r.ord + c.ord for (r, c) in (Row, Col)]
    total + xs.len
let _ = walk_anon()
";
    assert_eq!(check(ok), vec![]);

    let collision = "\
domain Row = 0..2
domain Col = 0..2
domain Cell = (Row, Col)
enum Node:
    Src
    Cell(Row, Col)
";
    let errs = check(collision);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::DUPLICATE_DEFINITION),
        "expected E0202 at declaration when variant collides with domain, got: {errs:?}"
    );
}
#[test]
fn range_lo_hi_are_rejected_with_start_end_fixes_and_user_fields_stay() {
    let ok = "\
struct Seg(lo: Int, hi: Int)
let s = Seg(1, 5)
let r = 2..=4
print(s.lo, s.hi, r.start, r.end, r.last, r.len)
";
    assert!(
        check(ok).is_empty(),
        "expected clean check, got: {:?}",
        check(ok)
    );

    let bad = "let r = 2..=4\nprint(r.lo, r.hi)\n";
    let errs = check(bad);
    assert_eq!(errs.len(), 2, "expected 2 E0503 errors, got: {errs:?}");
    assert!(
        errs.iter()
            .all(|e| e.code == cpl_diag::codes::REMOVED_SYNTAX && !e.fixes.is_empty()),
        "expected E0503 with fixes for Range.lo/.hi, got: {errs:?}"
    );
    assert_eq!(errs[0].fixes[0].edits[0].1, "start");
    assert_eq!(errs[1].fixes[0].edits[0].1, "end");
}
#[test]
fn array_map_preserves_index_domain() {
    let ok = "\
enum Side:
    X
    Y

struct Run(at: Int, end: Int, d: Int):
    fn shift(self, k: Int) -> Run = Run(at + k, end + k, d)

struct Block(seen: [Side]Run, real: [Side]Run):
    fn each(self, f: Run -> Run) -> Block = Block(seen.map(f), real.map(f))

domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => r.ord * 10 + c.ord
let g2: [R, C]Bool = g.map(x => x % 2 == 0)
let nested: [Side][C]Int = [Side] s => [C] c => c.ord
let nested2: [Side][C]Int = nested.map(row => row.map(x => x + 1))
let xs: List<Int> = [1, 2, 3]
let ys: List<Int> = xs.map(x => x * 2)
let zs: List<Int> = xs[0..2].map(x => x + 1)
let cs: List<Int> = \"ab\".map(c => Int(c == 'a'))
let _ = (g2, nested2, ys, zs, cs)
";
    assert_eq!(check(ok), vec![]);

    let wrong_domain = "\
domain A = 0..3
domain B = 0..3
let a: [A]Int = 0
let b: [B]Int = a.map(x => x + 1)
print(b)
";
    let errs = check(wrong_domain);
    assert!(
        errs.iter()
            .any(|d| d.code == cpl_diag::codes::TYPE_MISMATCH),
        "expected E0302 when assigning `[A]Int` map result to `[B]Int`, got: {errs:?}"
    );
}


#[test]
fn enum_array_literal_coercion_checks_variant_count() {
    let ok = "\
enum Side = X | Y
struct Run(l: Int, r: Int)
struct Block(seen: [Side]Run)
let a: [Side]Int = [1, 2]
let b = Block(seen: [Run(0, 1), Run(1, 2)])
print(a[X], b.seen[Y].r)
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
enum Side = X | Y
let a: [Side]Int = [1, 2, 3]
print(a[X])
";
    let errs = check(bad);
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert_eq!(errs[0].code.0, "E0302");
    assert!(
        errs[0].message.contains("holds 2 elements") && errs[0].message.contains("found 3"),
        "{errs:?}"
    );
}

/// The cf2183f dream checks clean: its tree lines (`read<Tree<V>>()`,
/// `t.at.rev`, `t.pos`, `t.below`, `t.sub`) and its 2D prefix sums.
#[test]
fn dream_cf2183f_checks() {
    let src = include_str!("../../../corpus/problems/cf2183f/dream.cp");
    let found = check(src);
    assert!(found.is_empty(), "{found:?}");
}

/// A failed `use` keeps the prelude off only the names it lists: the
/// missing name is one error, and what the prelude gives still checks.
#[test]
fn a_failed_import_drops_only_its_names_from_the_prelude() {
    let src = "use std.graph.{no_such}\ndomain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    let found = check(src);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].message.contains("no_such"), "{found:?}");
    let src = "use std.nope.{Rooted}\ndomain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    assert_eq!(check(src).len(), 1, "{:?}", check(src));
}

#[test]
fn mutable_slice_views_lend_and_reject_aliasing() {
    // In-place mutation through a mutable slice of a `var` list or `[D]T` succeeds.
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nxs[1..3].sort()\nxs[1..3][0] = 9\nprint(xs)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nv.sort()\nprint(xs)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet a = xs[0..2]\nlet b = xs[2..4]\na.sort()\nb.sort()\nprint(xs)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("domain D = 0..4\nvar a: [D]Int = [D] i => 4 - i.ord\nlet v = a[D(1)..D(3)]\nv.sort()\nprint(a)\n"),
        Vec::<&str>::new()
    );

    // Mutating through a slice of a `let` list or fixed-length mutators are E0401.
    assert_eq!(codes("let xs = [4, 3, 2, 1]\nxs[1..3].sort()\n"), ["E0401"]);
    assert_eq!(
        codes("var xs = [1, 2, 3, 4]\nxs[1..3].push(5)\n"),
        ["E0401"]
    );

    // Aliasing the parent (write or read) while a mutable slice is live is E0602.
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nxs[2] = 9\nv.sort()\n"),
        ["E0602"]
    );
    assert_eq!(
        codes("var xs = [4, 3, 2, 1]\nlet v = xs[1..3]\nprint(xs[2])\nv.sort()\n"),
        ["E0602"]
    );
    assert!(
        codes("var xs = [4, 3, 2, 1]\nlet a = xs[0..3]\nlet b = xs[1..4]\na.sort()\nb.sort()\n")
            .iter()
            .all(|c| *c == "E0602")
            && !codes(
                "var xs = [4, 3, 2, 1]\nlet a = xs[0..3]\nlet b = xs[1..4]\na.sort()\nb.sort()\n"
            )
            .is_empty()
    );
    assert_eq!(
        codes("domain D = 0..4\nvar a: [D]Int = [D] i => 4 - i.ord\nlet v = a[D(1)..D(3)]\na[D(2)] = 0\nv.sort()\n"),
        ["E0602"]
    );
}

#[test]
fn bounds_ranges_and_domain_ranges_check_per_decision_124() {
    assert_eq!(
        codes(
            "domain I = 1..=5\n\
             var s: Set<I> = Set()\n\
             fn walk(c: I, stop: Cut<I>) -> Int = s.range(c.l..stop).count()\n"
        ),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(
            "domain Seg = 0..2\n\
             domain PosB = 0..4\n\
             var piece: [Seg]Range<PosB.cuts> = PosB.cuts.first..PosB.cuts.first\n\
             piece[Seg(0)] = PosB.cuts.first..PosB.cuts.last\n"
        ),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(
            "domain Pos = 1..=5\n\
             fn tally(ls: Range<Pos>) -> Int = ls.len\n\
             let k = tally(Pos)\n"
        ),
        Vec::<&str>::new()
    );
}

#[test]
fn an_int_literal_is_a_position_of_an_integer_domain() {
    let src = "let n: 1..=9 = read()\ndomain V = 0..n\nvar best: [V.cuts]V.cuts = 0\nlet v: V = 0\nbest[0] = 1\nfn f(k: Int):\n    domain K = 0..=k\n    var t: [K]Int = 0\n    t[0] = 1\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_struct_declared_in_a_fn_is_a_type_and_a_constructor_there() {
    let src = "fn f(n: Int) -> Int:\n    struct Acc(sum: Int, count: Int)\n    var best: ?Acc = none\n    best = Acc(n, 1)\n    best!.sum\n";
    assert_eq!(check(src), Vec::new());
}

#[test]
fn inline_modules_extern_fns_and_pattern_params_check() {
    let src = "mod m:\n    extern fn ask(x: Int) -> Int\n    fn swap((a, b): (Int, Int)) -> (Int, Int) = (b, a)\n    fn solve(n: Int) -> Int = swap((ask(n), n)).0\n";
    assert_eq!(check(src), Vec::new());
}

#[test]
fn list_literals_fill_arrays_inside_tuples_and_match_arms() {
    let src = "\
type Site = 0..10
fn pick(k: 0..=1, a: Site, b: Site) -> ([2]Site, [1]Site) = match k:
    0 => ([a, b], [a])
    1 => ([b, a], [b])
";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn leading_index_gives_the_row_view() {
    let src = "domain R = 0..3\ndomain C = 0..4\nvar g: [R, C]Int = 0\nlet row: [C]Int = g[R(1)]\nprint(row[C(2)])\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert_eq!(
        codes("domain R = 0..3\ndomain C = 0..4\nvar g: [R, C]Int = 0\nprint(g[C(1)])\n"),
        ["E0305"]
    );
}

#[test]
fn seq_parameter_takes_anything_walkable() {
    let src = "domain P = 0..5\nfn total(xs: Seq<P>) -> Int:\n    var t = 0\n    for x in xs: t += Int(x.l)\n    t\nprint(total(P.range))\nprint(total([P(1), P(2)]))\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn reversed_range_keeps_its_element_type() {
    let src = "domain P = 0..4\nlet km: 0..=9 = read()\nvar dp: [0..=km, P]Int = 0\nvar row: [P]Int = 0\nfor k in (0..=km).rev:\n    dp[k] = row\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn domain_wrap_gives_a_position() {
    let src = "domain P = 0..5\nvar a: [P]Int = 0\nfor p in P: a[P.wrap(Int(p) - 1)] += 1\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn enums_and_product_domains_take_at() {
    let src = "enum L = A | B | C
domain P = (rows: 0..3, cols: 0..4)
fn f(i: Int) -> Int:
    let l = L.at(i) ?? L.A
    let p = P.at((i, i + 1)) else: return 0
    p.rows + p.cols
";
    assert!(codes(src).is_empty(), "{:?}", check(src));
}

#[test]
fn recursive_let_lambda_checks_and_var_self_ref_is_error() {
    assert_eq!(
        codes("let fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)\nprint(fact(5))\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var fact = (n: Int) -> Int => if n == 0: 1 else: n * fact(n - 1)\nprint(fact(5))\n"),
        ["E0406"]
    );
}

#[test]
fn bounds_literals_and_cut_range_conversion() {
    let ok = "\
domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len
var c: V.bounds = 0
c = 1
var dp: [V.bounds]Int = 0
for i in V.bounds:
    for j in V.bounds.first..i:
        dp[i] += cost(j..i)
let a: [V]Int = 1
let s = a[V.bounds.first..V.bounds.last]
print(c, dp[V.bounds.last], s.len)
";
    assert_eq!(codes(ok), Vec::<&str>::new());

    // Inclusive cut range `j..=i` must not convert to `Range<V>`.
    let bad_incl = "\
domain V = 0..5
fn cost(piece: Range<V>) -> Int = piece.len
for i in V.bounds:
    for j in V.bounds.first..i:
        print(cost(j..=i))
";
    assert_eq!(codes(bad_incl), ["E0301"]);
}

#[test]
fn chained_comparisons_reject_mixed_direction_and_equality() {
    assert_eq!(
        codes("let x = 3\nprint(yes(1 < x <= 5), yes(5 >= x > 1), yes(x == 3), yes(x != 4))\n"),
        Vec::<&str>::new()
    );
    assert_eq!(codes("let x = 3\nprint(1 < x > 2)\n"), ["E0111"]);
    assert_eq!(
        codes("let a = 1, b = 1, c = 1\nprint(yes(a == b == c))\n"),
        ["E0111"]
    );
    assert_eq!(
        codes("let a = 1, b = 2, c = 2\nprint(yes(a < b == c))\n"),
        ["E0111"]
    );
    assert_eq!(
        codes("let a = 1, b = 2, c = 3\nprint(yes(a != b != c))\n"),
        ["E0111"]
    );
}

#[test]
fn moved_from_heap_values_are_rejected() {
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a\nprint(a.len, b.len)\n"),
        ["E0608"]
    );
    assert_eq!(
        codes("let s = \"hi\"\nlet t = s\nprint(s.len, t.len)\n"),
        ["E0608"]
    );
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a.copy()\nprint(a.len, b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("let a = [1, 2]\nlet b = a\nprint(b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var a = [1, 2]\nlet b = a\na = [3]\nprint(a.len, b.len)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("let x = 10\nlet y = x\nprint(x, y)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("fn f():\n    let a = [1, 2]\n    let b = a\n    print(a.len, b.len)\nf()\n"),
        ["E0608"]
    );
}

#[test]
fn an_index_of_the_arrays_own_domain_is_proven_by_its_type() {
    let proofs = |src: &str| {
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        let mut out = Vec::new();
        for item in analysis.items.iter() {
            for t in &item.check.typed_indexes {
                let access = t.access.shift(item.offset);
                out.push(format!(
                    "{} {}",
                    &src[access.start as usize..access.end as usize],
                    t.proof
                ));
            }
        }
        out
    };
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus/problems/atcoder_dp_l/port.cp"
    );
    let port = std::fs::read_to_string(path).unwrap();
    assert_eq!(
        proofs(&port),
        [
            "a[l] l: I, the index space of a",
            "a[l] l: I, the index space of a",
            "a[r] r: I, the index space of a",
        ]
    );
    let src = "let n: 1..=10 = read()
domain V = 1..=n
domain E = 1..n
domain R = 0..3
domain C = 0..4
var g: [R, C]Int = 0
let xs: [V]Int = read()
let ys: [E]Int = read()
for e in E:
    g[R.first, C.last] += xs[V.first] + ys[e]
print(xs[V.first..=V.last].sum())
";
    assert_eq!(
        proofs(src),
        [
            "g[R.first, C.last] R.first: R, C.last: C, the index spaces of g",
            "xs[V.first] V.first: V, the index space of xs",
            "ys[e] e: E, the index space of ys",
        ]
    );
    let pairs = "let n: 1..=10 = read()
domain V = 1..=n
domain K = 0..3
domain State = (V, K)
domain R = 0..3
domain C = 0..4
var dp: [State]Int = 0
var g: [R, C]Int = 0
let p: (R, C) = (R.first, C.last)
for v in V:
    dp[v, K.first] = g[p]
print(dp[V.first, K.last])
";
    assert_eq!(
        proofs(pairs),
        [
            "dp[v, K.first] v: V, K.first: K, the index spaces of dp",
            "g[p] p: (R, C), the index space of g",
            "dp[V.first, K.last] V.first: V, K.last: K, the index spaces of dp",
        ]
    );
    let wrong = src.replace("ys[e]", "xs[e]");
    assert!(!codes(&wrong).is_empty());
    assert!(!proofs(&wrong).iter().any(|p| p.starts_with("xs[e]")));
}

#[test]
fn a_prelude_call_hovers_its_doc_and_the_cost_of_its_shape() {
    let src = "let n: 1..=10 = read()\nfn f(a: Int, xs: List<Int>) -> Int:\n    \
               abs(a) + max(a, n) + min(xs) + isqrt(n)\n";
    let at = |needle: &str| src.find(needle).unwrap() as u32;
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let abs = db.hover("a.cp", at("abs")).unwrap();
    assert_eq!((abs.span.start, abs.span.end), (at("abs"), at("abs") + 3));
    assert_eq!(abs.text, "abs: built-in\ncost: O(1)");
    assert!(abs.doc.unwrap().starts_with("The absolute value"));
    let cost = |db: &mut Database, needle| db.hover("a.cp", at(needle)).unwrap().text;
    assert_eq!(cost(&mut db, "max("), "max: built-in\ncost: O(1)");
    assert_eq!(cost(&mut db, "min("), "min: built-in\ncost: O(n)");
    assert!(cost(&mut db, "isqrt").ends_with("\ncost: O(1)"));
    assert_eq!(cost(&mut db, "read"), "read: built-in\ncost: O(1)");
}

#[test]
fn min_and_max_of_scalars_have_their_type() {
    let src =
        "let (n, k) = read<(1..=9, 0..=99)>()\nlet m = min(n, k)\nlet b = max(n, n)\nprint(m, b)\n";
    assert_eq!(hover(src, "m ="), "m: Int");
    assert!(
        hover(src, "b =").starts_with("b: 1..=9"),
        "{}",
        hover(src, "b =")
    );
}

#[test]
fn a_name_bound_to_read_of_a_type_has_that_type() {
    let src = "let n = read<1..=100>()\nlet xs = read<[n](a: Int, b: Bool)>()\nprint(n, xs.len)\n";
    assert_eq!(hover(src, "n ="), "n: 1..=100\n  storage: 1 byte");
    assert!(
        hover(src, "xs =").starts_with("xs: [n](a: Int, b: Bool)\n  storage: 16 bytes per element")
    );
}

#[test]
fn print_and_read_cost_o1_only_for_values_of_a_fixed_size() {
    let src = "fn f(xs: List<Int>, s: Str):\n    let (a, b): (Int, 1..=9) = read()\n    \
               print(a, \"is {b}\")\n    print(xs)\n    print(s)\n    let t: Str = read()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).unwrap().text
    };
    assert_eq!(hover(&mut db, 0, "read"), "read: built-in\ncost: O(1)");
    assert_eq!(hover(&mut db, 0, "print"), "print: built-in\ncost: O(1)");
    assert_eq!(
        hover(&mut db, 1, "print"),
        "print: built-in\ncost: O(xs.len)"
    );
    assert_eq!(hover(&mut db, 2, "print"), "print: built-in");
    assert_eq!(hover(&mut db, 1, "read"), "read: built-in");
}

#[test]
fn ask_conversions_and_rand_cost_o1_for_values_of_a_fixed_size() {
    let src = "fn f(c: Char, s: Str):\n    let r: 0..=1 = ask(\"?\", c)\n    \
               let t: Int = ask(s)\n    print(Int(c) + Int(r == 1) + rand(1, 6) + Int(s))\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    let o1 = |name: &str| Some(format!("{name}: built-in\ncost: O(1)"));
    assert_eq!(hover(&mut db, 0, "ask"), o1("ask"));
    assert_eq!(hover(&mut db, 1, "ask").as_deref(), Some("ask: built-in"));
    assert_eq!(hover(&mut db, 0, "Int("), o1("Int"));
    assert_eq!(hover(&mut db, 1, "Int("), o1("Int"));
    assert_eq!(hover(&mut db, 0, "rand"), o1("rand"));
    assert_eq!(hover(&mut db, 2, "Int("), None);
}

#[test]
fn a_method_cost_names_the_receivers_domain_for_the_structs() {
    let src =
        "domain V = 0..10\nfn f(bit: Fenwick<V, Int>, g: Fenwick2D<V, V, Int>, v: V) -> Int:\n    \
               bit.sum(..v) + g.sum(.., ..)\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        let text = db.hover("a.cp", at).unwrap().text;
        text.lines()
            .find_map(|l| l.strip_prefix("cost: "))
            .map(str::to_string)
    };
    assert_eq!(cost(&mut db, "sum(..v)").as_deref(), Some("O(log |V|)"));
    assert_eq!(
        cost(&mut db, "sum(.., ..)").as_deref(),
        Some("O(log |V| * log |V|)")
    );
}

#[test]
fn read_with_a_fixed_type_argument_and_a_choice_of_fixed_text_cost_o1() {
    let src = "fn f(ok: Bool) -> Int:\n    print(if ok: \"YES\" else: \"NO\")\n    \
               print(if ok: \"YES\" else: str(1))\n    read<Int>() + read<(Int, Char)>().0\n\
               fn g() -> Str:\n    read<Str>()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let hover = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(hover(&mut db, 0, "print").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 1, "print"), None);
    assert_eq!(hover(&mut db, 0, "read").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 1, "read").as_deref(), Some("O(1)"));
    assert_eq!(hover(&mut db, 2, "read"), None);
}


#[test]
fn integer_domain_positions_and_plain_enums_read_and_print_in_o1() {
    let src = "domain Pos = 1..=10\nenum Axis = X | Y\nenum Shape = Dot | Box(Int)\n\
               fn ask(lo: Pos, hi: Pos) -> Int:\n    print(\"?\", lo, hi)\n    read()\n\
               fn f() -> Axis:\n    let a: Axis = read()\n    print(a)\n    a\n\
               fn g() -> Shape:\n    read()\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, nth: usize, name: &str| {
        let at = src.match_indices(name).nth(nth).unwrap().0 as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(cost(&mut db, 0, "print").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 0, "read").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 1, "read").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 1, "print").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, 2, "read"), None);
}


#[test]
fn a_number_made_into_a_position_or_a_modular_number_costs_o1() {
    let src = "const MOD = 7\ntype Z = Mod<MOD>\ndomain Node = 0..10\ndomain Cell = Node * Node\n\
               fn f(i: Int, s: Str) -> Int:\n    let a = Node(i)\n    let c = Cell(a, a)\n    let z = Z(i)\n    \
               let w = Z(s.len)\n    a.ord + z.v\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let cost = |db: &mut Database, needle: &str| {
        let at = src.find(needle).unwrap() as u32;
        db.hover("a.cp", at).and_then(|h| {
            h.text
                .lines()
                .find_map(|l| l.strip_prefix("cost: "))
                .map(str::to_string)
        })
    };
    assert_eq!(cost(&mut db, "Node(i)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Cell(a").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Z(i)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "Z(s.len)").as_deref(), Some("O(1)"));
    assert_eq!(cost(&mut db, "domain Node").as_deref(), None);
}

#[test]
fn a_domains_nth_at_and_wrap_and_a_positions_offset_cost_o1() {
    let src = "domain V = 0..10\nfn f(i: Int, v: V) -> Int:\n    let a = V.nth(i)\n    let b = V.at(i)\n    \
               let c = V.wrap(i)\n    let d = v.offset(2)\n    a.ord + b.ord + c.ord + d!.ord\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    assert!(db
        .diagnostics("a.cp")
        .iter()
        .all(|d| d.severity != cpl_diag::Severity::Error));
    let hover = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    assert_eq!(
        hover(&mut db, "nth").as_deref(),
        Some("nth: fn(x: Int) -> V\ncost: O(1)")
    );
    for name in ["at(", "wrap", "offset"] {
        let text = hover(&mut db, name).unwrap();
        assert!(text.ends_with("\ncost: O(1)"), "{name}: {text}");
    }
}

#[test]
fn abs_of_an_integer_is_an_integer_so_a_position_made_of_it_costs_o1() {
    let src = "domain Node = 1..=10\nfn r(i: Int, j: 0..=5) -> Node:\n    let m = min(i, j)\n    let a = abs(max(i, 3))\n    \
               Node(min(m, a))\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    assert!(db
        .diagnostics("a.cp")
        .iter()
        .all(|d| d.severity != cpl_diag::Severity::Error));
    let hover = |db: &mut Database, name: &str| {
        let at = src.find(name).unwrap() as u32;
        db.hover("a.cp", at).map(|h| h.text)
    };
    assert_eq!(hover(&mut db, "m =").as_deref(), Some("m: Int"));
    assert_eq!(hover(&mut db, "a =").as_deref(), Some("a: Int"));
    assert!(hover(&mut db, "Node(min")
        .unwrap()
        .ends_with("\ncost: O(1)"));
}

#[test]
fn printing_an_array_or_list_of_fixed_size_values_costs_its_size() {
    let src = "let n: 1..=100 = read()\ndomain V = 1..=n\nvar d: [V]Int = 1\nvar g: [V, V]Char = '.'\nlet xs: List<Int> = read()\nlet ws: List<Str> = read()\n\
               print(d)\nprint(\"?\", d, g)\nlet r: Int = ask(\"?\", g)\nprint(xs)\nprint(ws)\nprint(r)\n";
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let mut hover = |name: &str| {
        db.hover("a.cp", src.find(name).unwrap() as u32)
            .map(|h| h.text)
            .unwrap_or_default()
    };
    assert!(
        hover("print(d)").ends_with("\ncost: O(|V|)"),
        "{}",
        hover("print(d)")
    );
    assert!(hover("print(\"?\", d").ends_with("\ncost: O(|V| + |V| * |V|)"));
    assert!(
        hover("ask(").ends_with("\ncost: O(|V| * |V|)"),
        "{}",
        hover("ask(")
    );
    assert!(
        hover("print(xs)").ends_with("\ncost: O(xs.len)"),
        "{}",
        hover("print(xs)")
    );
    assert!(
        !hover("print(ws)").contains("cost: O("),
        "{}",
        hover("print(ws)")
    );
    assert!(hover("print(r)").ends_with("\ncost: O(1)"));
    let src = "let n: 1..=100 = read()\ndomain V = 1..=n\nfn show(d: [V]Int, g: [V, V]Char):\n    print(d)\n    let r: Int = ask(\"?\", g)\n    print(r)\n";
    db.set_file("b.cp", src);
    let at = src.find("print(d)").unwrap() as u32;
    let costs = db.costs_in("b.cp", at, src.len() as u32);
    let costs: Vec<_> = costs.iter().map(|(_, n, c)| format!("{n} {c}")).collect();
    assert_eq!(costs, ["print O(|V|)", "ask O(|V| * |V|)"]);
}

#[test]
fn checks_with_field_names_types_and_fn_uses() {
    let mut db = Database::new();
    db.set_file(
        "ok.cp",
        "context opt:\n    k: Int = 1\nwith io(base: 1, sep: \",\", precision: 2)\nwith opt(k: 5):\n    let x: Int = opt.k\n    print(x)\nfn f() -> Int uses opt = opt.k\n",
    );
    let diags = db.diagnostics("ok.cp");
    assert!(diags.is_empty(), "{diags:?}");

    db.set_file("bad_field.cp", "with io(nope: 1):\n    print(1)\n");
    let codes_bf: Vec<_> = db.diagnostics("bad_field.cp").iter().map(|d| d.code).collect();
    assert!(codes_bf.contains(&cpl_diag::codes::UNKNOWN_FIELD), "{codes_bf:?}");

    db.set_file("bad_type.cp", "with io(base: 2):\n    print(1)\n");
    let codes_bt: Vec<_> = db.diagnostics("bad_type.cp").iter().map(|d| d.code).collect();
    assert!(codes_bt.contains(&cpl_diag::codes::TYPE_MISMATCH), "{codes_bt:?}");

    db.set_file("bad_uses.cp", "fn f() uses nope:\n    print(1)\n");
    let codes_bu: Vec<_> = db.diagnostics("bad_uses.cp").iter().map(|d| d.code).collect();
    assert!(codes_bu.contains(&cpl_diag::codes::UNRESOLVED_NAME), "{codes_bu:?}");
}
