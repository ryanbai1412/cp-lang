//! A generator outside a call.

use crate::*;

#[test]
fn a_parenthesized_generator_folded_by_a_method_becomes_the_free_fold() {
    let src = "print((x * 2 for x in xs).max())\n";
    let diags = parse_file(src).diagnostics;
    assert_eq!(diags.len(), 1, "{diags:#?}");
    let d = &diags[0];
    assert_eq!(d.code.0, "E0103");
    let mut out = src.to_string();
    for (span, edit) in d.fixes[0].edits.iter().rev() {
        out.replace_range(span.start as usize..span.end as usize, edit);
    }
    assert_eq!(out, "print(max(x * 2 for x in xs))\n");
}
