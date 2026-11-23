//! What an error says and offers, read at 3am.

use crate::tests::{check, messages};

fn fixed(src: &str) -> String {
    let diags = check(src);
    let fix = &diags[0].fixes[0];
    let mut out = src.to_string();
    for (span, text) in fix.edits.iter().rev() {
        out.replace_range(span.start as usize..span.end as usize, text);
    }
    out
}

#[test]
fn a_did_you_mean_keeps_values_and_types_apart() {
    use crate::check::closest;
    assert_eq!(closest("cnt", ["Int", "count"].into_iter()), None);
    assert_eq!(closest("vec", ["Vec"].into_iter()).as_deref(), Some("Vec"));
    assert_eq!(
        closest("Lst", ["List"].into_iter()).as_deref(),
        Some("List")
    );
}

#[test]
fn a_number_added_to_a_string_literal_goes_inside_it() {
    assert_eq!(fixed("let x = \"a\" + 1\n"), "let x = \"a{1}\"\n");
    assert_eq!(
        fixed("let n = 2\nprint(n + \"b\")\n"),
        "let n = 2\nprint(\"{n}b\")\n"
    );
    assert_eq!(
        messages(&check("let n = 2\nprint(n + \"b\")\n")),
        ["`+` can't combine an Int and a Str"]
    );
}

#[test]
fn one_place_lent_twice_says_so() {
    let src = "fn f(mut a: Int, mut b: Int):\n    a += b\nvar x = 0\nf(mut x, mut x)\n";
    let diags = check(src);
    assert_eq!(
        messages(&diags),
        ["`x` is lent to this call twice, both times `mut`: a call can't change one place through two names"]
    );
    assert_eq!(diags[0].help.as_deref(), Some("pass `x` once"));
}

#[test]
fn calling_a_number_suggests_the_product() {
    let diags = check("let x = 5\nprint(x(1))\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("to multiply, write `x * (1)`")
    );
}

#[test]
fn an_uppercase_literal_let_becomes_a_const() {
    assert_eq!(fixed("let N = 5\nprint(N)\n"), "const N = 5\nprint(N)\n");
    let diags = check("let N = read()\nprint(N)\n");
    assert!(diags.iter().all(|d| d.fixes.is_empty()), "{diags:?}");
    let diags = check("var N = 5\nN += 1\n");
    assert!(diags.iter().all(|d| d.fixes.is_empty()), "{diags:?}");
}

#[test]
fn a_misspelled_stdlib_module_names_the_closest() {
    let diags = check("use std.segtre\nprint(1)\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("did you mean `std.segtree`?")
    );
    assert_eq!(
        fixed("use std.segtre\nprint(1)\n"),
        "use std.segtree\nprint(1)\n"
    );
    let diags = check("use std.no_such_module\nprint(1)\n");
    assert!(diags[0].fixes.is_empty(), "{diags:?}");
}

#[test]
fn another_languages_true_and_null_are_spelled_here() {
    assert_eq!(
        fixed("let b = True\nprint(b)\n"),
        "let b = true\nprint(b)\n"
    );
    let diags = check("let x: ?Int = null\nprint(x ?? 0)\n");
    assert_eq!(diags[0].help.as_deref(), Some("CP Lang spells it `none`"));
}

#[test]
fn another_languages_member_names_point_at_ours() {
    let diags = check("let xs = [3, 1]\nprint(xs.length())\n");
    assert_eq!(
        diags[0].help.as_deref(),
        Some("did you mean `len`?"),
        "{diags:?}"
    );
}

#[test]
fn pythons_range_says_how_a_range_is_written() {
    let diags = check("for i in range(3):\n    print(i)\n");
    assert!(
        diags[0]
            .help
            .as_deref()
            .is_some_and(|h| h.contains("`range(n)` is `0..n`")),
        "{diags:?}"
    );
}

#[test]
fn pythons_conversions_say_how_ours_are_written() {
    let help = |src: &str| check(src)[0].help.clone().unwrap_or_default();
    assert!(help("let x = 5\nprint(str(x))\n").contains("`\"{x}\"`"));
    assert!(help("let s = \"1\"\nprint(int(s))\n").contains("s.parse<Int>()!"));
    assert!(help("let a = 4\nprint(math.sqrt(a))\n").contains("`x.sqrt`"));
    let d = check("let x = 5\nprint(x.toString())\n");
    assert!(d[0].message.starts_with("Int has no `toString`"), "{d:?}");
    assert!(
        d[0].help.as_deref().unwrap_or("").contains("`\"{x}\"`"),
        "{d:?}"
    );
}

#[test]
fn a_pairs_first_and_second_name_the_tuple_parts() {
    assert_eq!(
        fixed("let p = (1, 2)\nprint(p.first, p.second)\n"),
        "let p = (1, 2)\nprint(p.0, p.second)\n"
    );
    let d = check("let p = (1, 2)\nprint(p.first, p.second)\n");
    assert_eq!(d.len(), 2, "{d:?}");
    assert!(d[0].message.starts_with("a tuple has no `.first`"), "{d:?}");
    assert_eq!(d[1].fixes[0].edits[0].1, "1", "{d:?}");
}

#[test]
fn empty_braces_for_a_container_offer_its_empty_spelling() {
    let fix = |src: &str| fixed(src).lines().next().unwrap_or("").to_string();
    assert_eq!(
        fix("var m: Map<Int, Int> = {}\nm[1] = 2\n"),
        "var m: Map<Int, Int> = [:]"
    );
    assert_eq!(
        fix("var s: Set<Int> = {}\ns.insert(1)\n"),
        "var s: Set<Int> = Set()"
    );
    let d = check("var m: Map<Int, Int> = {}\n");
    assert!(d[0].message.contains("empty block"), "{d:?}");
}

#[test]
fn in_on_a_list_offers_the_scan() {
    let src = "let xs = [1, 2, 3]\nprint(yes(2 in xs), yes(4 not in xs))\n";
    let d = check(src);
    let scans: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(scans, ["xs.contains(2)", "not xs.contains(4)"], "{d:?}");
}
