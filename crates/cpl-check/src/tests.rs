use crate::{Database, DeclKind, Stats};
use cpl_diag::{Diagnostic, Span};

mod checks;
mod cost;
mod domains;

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
var dist: [V]?Int = None
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

pub(crate) fn check(src: &str) -> Vec<Diagnostic> {
    let mut db = Database::new();
    db.set_file("a.cp", src);
    db.diagnostics("a.cp")
}

fn codes(src: &str) -> Vec<&'static str> {
    check(src).iter().map(|d| d.code.0).collect()
}

pub(crate) fn one(src: &str) -> Diagnostic {
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
#[ignore = "v0.3 lexes 3 files on the first analysis of ROADS, not 1; needs a call on whether built-in std counts"]
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
    // Both users of `total` lose it: one root cause, one error at the first.
    let diags = db.diagnostics("a.cp");
    let codes: Vec<_> = diags.iter().map(|d| d.code.0).collect();
    assert!(codes == ["E0201"] && diags[0].message.starts_with("no `total` in scope"));
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
    let line = "    if longer(r, roads[E.first!]) and deg[r.v] > 0: total += 1\n";
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
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max (None is a spare value: no flag)"
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
        "dist: [V]?Int\n  domain V = 1..=n, n <= 2e5\n  storage: 8 bytes per element, 1.6 MB max (None is a spare value: no flag)"
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
    let src = "struct Run:\n    len: Int\n    fn grown() -> Run = Run(len + 1)\nfn main():\n    let n = read<1..=9>()\n    domain R = 0..n\n    let best: [R]?Int = [R] r => None\n    print(best[R.first!], Run(1).grown().len)\n";
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

pub(crate) fn messages(diags: &[Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.message.as_str()).collect()
}

const DSU: &str = "\
pub struct Dsu<V>:
    up: [V]V
    parts: Int

    pub fn new() -> Dsu<V>:
        Dsu([V] v => v, V.count)

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
        ("x", "use y.{g}\npub fn f() -> Int = 1\n"),
        ("y", "use x.{f}\npub fn g() -> Int = 2\n"),
    ];
    let diags = check_with(&cycle, "use x.{f}\nprint(f())\n");
    assert_eq!(messages(&diags), ["`x` is in an import cycle: x -> y -> x"]);
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
        "fn two(mut xs: [V]Int, mut ys: [V]Int):\n    xs[V.first!] += 1\n    ys[V.first!] += 1\ntwo(mut a[l.l..r.l], mut a[r.l..r.r])\n",
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
        "fn two(mut xs: [V]Int, mut ys: [V]Int):\n    xs[V.first!] += 1\n    ys[V.first!] += 1\ntwo(mut a, mut a[l..=r])\n",
    );
    assert!(
        diags[0].message.ends_with("one contains the other"),
        "{diags:#?}"
    );
}

#[test]
fn a_let_bound_element_lends_only_that_element() {
    let lent = |body: &str| {
        check(&format!(
            "var rows = [[1, 2], [3]]\nlet r = rows[0]\n{body}\nprint(r.len)\n"
        ))
    };
    assert!(lent("rows[1].push(5)").is_empty());
    for body in ["rows[0].push(5)", "rows.pop()", "rows = []"] {
        let diags = lent(body);
        assert_eq!(diags.len(), 1, "{body}: {diags:#?}");
        assert_eq!(diags[0].code.0, "E0602");
    }
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
fn a_piece_of_a_read_view_keeps_the_lend_a_read() {
    let src = "let a = [10, 20, 30]\nlet s = a[0..3]\nprint(a[0])\nprint(List(s.take(2)))\n";
    assert_eq!(check(src), vec![]);
    let diags = check("var a = [10, 20, 30]\nlet s = a[0..3]\na[0] = 1\nprint(List(s.take(2)))\n");
    assert_eq!(diags.len(), 1, "{diags:#?}");
    assert_eq!(diags[0].code.0, "E0602");
}

#[test]
fn a_lent_slice_is_not_stored_assigned_or_returned() {
    // Storage declared after `a` in its block dies no later than `a`, so it
    // may keep a view of it (R05.7.8).
    for body in [
        "var keep = a[l..=r]\n",
        "var b: [V]Int = 0\nb = a[l..=r]\n",
        "var parts: List<[V]Int> = []\nparts.push(a[l..=r])\n",
    ] {
        assert_eq!(check(&format!("{ARRAY}{body}")), vec![], "{body}");
    }
    for (body, what) in [
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
        check(&format!("{ARRAY}var keep = a[l..=r].clone()\n")),
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
    assert!(index.help.unwrap().contains("I(i + 1)"));
    let arg = one(&format!("{head}for i in I: show(i - 1)\n"));
    assert_eq!(
        arg.message,
        "`show` takes an I for `i`, but `i - 1` is an Int"
    );
    // Bound, clamped or asserted, the step is a position again; and two
    // positions of one domain are an `Int` apart.
    for body in [
        "    if let j = i + 1: print(a[j])\n",
        "    print(a[i + 1 ?? I.last!])\n",
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
        "`..=i.r` is a closed range of cuts, which names no piece"
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
#[ignore = "spec/conformance/diagnostics/max-of-a-slice-is-optional.cp"]
fn the_max_of_a_slice_is_optional() {
    let head =
        "let n: 1..=5 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nlet f = V.first!\n";
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
        Some("loop over positions instead (`for i in 0..xs.len`), or over a copy (`for x in xs.clone()`)")
    );
    let called = one(&with_globals("for x in xs:\n    print(grow() + x)\n"));
    assert_eq!(
        called.message,
        "`grow` can't be called here: it changes `xs`, and the `for` loop is going over `xs`"
    );
    // Over positions or a copy, in another list, or after the loop.
    for rest in [
        "for i in 0..xs.len:\n    xs.push(i)\n    if xs.len > 9: break\n",
        "for x in xs.clone():\n    xs.push(x)\n",
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
        "fn go(mut x: Int, y: Int):\n    x += y\nif l != r: go(mut a[l], a[r])\n",
        "fn go(mut xs: [V]Int, y: Int):\n    xs[l] += y\ngo(mut a, a[r])\n",
    ] {
        let src = with_globals(rest);
        assert!(codes(&src).is_empty(), "{rest}: {:?}", check(&src));
    }
    // A read of an element that may be the `mut` one is an access too (R9).
    let src = with_globals("fn go(mut x: Int, y: Int):\n    x += y\ngo(mut a[l], a[r])\n");
    assert_eq!(codes(&src), vec!["E0601"]);
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
        Some("compute the right side into a `let` first, then assign: otherwise the order the two sides run in decides the result")
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
fn a_declared_domain_satisfies_a_domain_bound() {
    let src = "\
use std.seq.{buckets}
domain V = 0..3
domain W = 1..=3
let w: [V]W = [1, 1, 2]
let b = buckets(w)
";
    assert_eq!(check(src), vec![]);
}

#[test]
fn a_name_resolves_to_the_file_or_module_that_declares_it() {
    let sums = |src: &str| -> Vec<String> {
        let mut db = Database::new();
        db.set_file("a.cp", src);
        let analysis = db.analyze("a.cp").unwrap();
        (analysis.items.iter())
            .flat_map(|item| item.check.resolved.iter())
            .filter(|(_, r)| r.name == "sum")
            .map(|(_, r)| r.module.clone())
            .collect()
    };
    // A user's own `fn sum` is this file's, not the library's.
    assert_eq!(sums("fn sum(a: Int) -> Int = a\nprint(sum(2))\n"), [""]);
    assert_eq!(sums("let xs = [1, 2]\nprint(sum(xs))\n"), ["std.seq"]);
    assert_eq!(
        sums("use std.seq as s\nlet xs = [1, 2]\nprint(s.sum(xs))\n"),
        ["std.seq"]
    );
}

#[test]
fn a_reversed_view_concatenates_like_its_source() {
    let src = "let a = [3, 2, 1]\nlet b = [7, 8]\nlet c: List<Int> = a.rev + b\nlet d: List<Int> = b + a.rev\nprint(c, d)\n";
    assert_eq!(check(src), vec![]);
}

#[test]
fn a_prelude_math_fn_returns_an_f64() {
    let mut db = Database::new();
    db.set_file(
        "a.cp",
        "let n: Int = read()\nlet d = sqrt(n)\nprint(d, max(n, 2))\n",
    );
    let analysis = db.analyze("a.cp").unwrap();
    assert!(!analysis.has_errors());
    let items = &analysis.items;
    let hovers = || items.iter().flat_map(|item| item.check.hovers.iter());
    assert!(hovers().any(|h| h.name == "d" && h.ty == crate::Ty::Float));
    let resolved = || items.iter().flat_map(|item| item.check.resolved.iter());
    assert!(resolved().any(|(_, r)| r.name == "sqrt" && r.module == "prelude"));
    assert!(resolved().any(|(_, r)| r.name == "max" && r.module == "prelude"));
    assert_eq!(codes("let k: Int = sqrt(2.0)\nprint(k)\n"), ["E0302"]);
}

#[test]
fn an_index_with_no_key_does_not_crash_the_map_fold_check() {
    let src =
        "var cnt: Map<Int, Int> = Map()\nfor x in [3, 1]:\n    cnt[]x = (cnt.get(x) ?? 0) + 1\n";
    assert!(!codes(src).is_empty());
}

#[test]
fn a_library_fn_in_method_form_hovers_and_types_like_its_free_form() {
    let src = "let s = \"aab\"\nprint(s.runs(by: c => c).count())\nprint(runs(s, by: c => c).count())\nlet a = read<[3]Int>()\nlet k = 1 - a.count(x => x > 0)\nprint(k)\n";
    let method = hover(src, "runs(by");
    assert!(method.starts_with("fn runs<S, T, K>(self: S"), "{method}");
    assert_eq!(method, hover(src, "runs(s"));
    assert_eq!(hover(src, "k = "), "k: Int");
    let mut db = Database::new();
    db.set_file("a.cp", src);
    let an = db.analyze("a.cp").unwrap();
    let call = src.find("s.runs").unwrap() as u32;
    let (item, at) = an.item_at(call).unwrap();
    let shape = item.check.shapes.iter().find(|(span, _)| span.start == at);
    assert_eq!(
        shape.map(|(_, ty)| ty.to_string()).as_deref(),
        Some("Gen<Slice<Str>>")
    );
}

#[test]
fn a_fn_value_whose_param_is_wider_does_not_fit_a_narrower_fn_type() {
    // No subtyping (R02.11.3): `T` into `?T` converts a value, not a fn's parameters.
    let src = "domain P = 0..3\nfn later(a: ?P, b: ?P) -> ?P = b ?? a\nlet g: (?P, P) -> ?P = later\nprint(Int(g(None, P(1))!))\n";
    assert_eq!(codes(src), vec!["E0302"]);
    let wrapped = "domain P = 0..3\nfn later(a: ?P, b: ?P) -> ?P = b ?? a\nlet g: (?P, P) -> ?P = (a, b) => later(a, b)\nprint(Int(g(None, P(1))!))\n";
    assert_eq!(codes(wrapped), Vec::<&str>::new());
}

#[test]
fn a_list_map_lambda_param_takes_the_element_type() {
    let src = "let ys = [1, 2].map(x =>\n    let y = x + 1\n    y\n)\nprint(ys)\n";
    assert_eq!(hover(src, "x =>"), "x: Int");
    assert_eq!(hover(src, "y = x"), "y: Int");
}

#[test]
fn an_impl_method_wins_over_a_library_fn_whose_bound_fails() {
    let src = "use std.algebra.{Reversible}\nstruct Word(s: Str)\nimpl Reversible for Word:\n    fn rev(self) -> Word = Word(s + \"!\")\nprint(Word(\"ab\").rev().s)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_method_call_takes_the_fn_its_receiver_fits_from_any_module() {
    let modules = [
        (
            "lib.sq",
            "pub struct Sq(s: Int)\npub fn area(self: Sq) -> Int = self.s * self.s\n",
        ),
        (
            "lib.ci",
            "pub struct Ci(r: Int)\npub fn area(self: Ci) -> Int = 3 * self.r * self.r\n",
        ),
    ];
    let src = "use lib.sq.*\nuse lib.ci.*\nlet a: Int = Sq(2).area()\nlet b: Int = Ci(1).area()\nprint(a, b)\n";
    let mut db = Database::with_loader(Box::new(Modules(modules.to_vec())));
    db.set_file("a.cp", src);
    assert!(
        db.diagnostics("a.cp").is_empty(),
        "{:?}",
        db.diagnostics("a.cp")
    );
    let analysis = db.analyze("a.cp").unwrap();
    let chosen: Vec<&str> = (analysis.items.iter())
        .flat_map(|item| item.check.resolved.iter())
        .filter(|(_, r)| r.name == "area")
        .map(|(_, r)| r.module.as_str())
        .collect();
    assert_eq!(chosen, ["lib.sq", "lib.ci"]);
}

#[test]
fn a_method_two_modules_give_one_receiver_is_ambiguous() {
    let modules = [
        ("lib.a", "pub fn twice<T>(self: T) -> Int = 2\n"),
        ("lib.b", "pub fn twice<T>(self: T) -> Int = 3\n"),
    ];
    let diags = check_with(&modules, "use lib.a.*\nuse lib.b.*\nprint(5.twice())\n");
    assert_eq!(diags.len(), 1, "{diags:?}");
    assert_eq!(diags[0].code, cpl_diag::codes::AMBIGUOUS_METHOD);
    assert!(
        diags[0].message.contains("`lib.a` and `lib.b`"),
        "{}",
        diags[0].message
    );
}

#[test]
fn a_method_no_candidate_takes_says_what_the_free_form_says() {
    let diags = check("let a: ?List<Int> = [1, 2]\nprint(a.sum())\n");
    assert_eq!(
        messages(&diags),
        ["`?List<Int>` does not implement `Seq` required by `sum`"]
    );
    let own =
        check("let a: ?Int = 3\nprint(a.map(x => x + 1) ?? 0)\nprint([1, 2].map(x => x * 2))\n");
    assert!(own.is_empty(), "{own:?}");
}

#[test]
fn inside_if_t_bound_every_t_has_the_bound() {
    let src = "fn plus<T: Add>(a: T, b: T) -> T = a + b\nfn f<T>(o: ?T, d: T) -> ?T:\n    if T: Add:\n        if let x = o: return Some(plus(x, d) + d)\n    o\nprint(f(Some(2), 3)!)\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
    let outside = "fn g<T>(a: T, b: T) -> T:\n    if T: Add:\n        return a + b\n    a + b\n";
    assert_eq!(codes(outside), ["E0606"]);
}

#[test]
fn a_method_form_lambda_body_and_tuple_receiver_method_hover() {
    let src = "domain W = 0..3\nfn covers(xs: List<W>, k: Int) -> Bool = xs.len >= k\nlet both = [false, true].all(d => covers(W.filter(w => (Int(w) == 1) == d).collect(), 1))\nprint(yes(both))\nprint(yes((W, W).all((u, v) => u != v or u == v)))\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    assert!(
        hover(src, "covers(W").starts_with("fn covers"),
        "{}",
        hover(src, "covers(W")
    );
    assert_eq!(hover(src, "d)"), "d: Bool");
    assert!(
        hover(src, "all((u").contains("all"),
        "{}",
        hover(src, "all((u")
    );
}
