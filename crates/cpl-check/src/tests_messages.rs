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
